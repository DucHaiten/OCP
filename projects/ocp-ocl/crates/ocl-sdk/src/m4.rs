use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{check_project_with_lock, project_layout, SdkError};

const MAX_COMPONENTS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSpecV1 {
    pub name: String,
    pub phase: String,
    pub provides: Vec<String>,
    pub requires: Vec<String>,
    pub template_rel: String,
    pub hash64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhenotypeSpecV1 {
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyProofV1 {
    pub catalog_hash64: String,
    pub phenotype_hash64: String,
    pub generated_hash64: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeSummary {
    pub selected_components: usize,
    pub generated_files: usize,
    pub generated_module: PathBuf,
    pub proof_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifySummary {
    pub ok: bool,
    pub selected_components: usize,
}

pub fn load_component_catalog(registry_root: &Path) -> Result<Vec<ComponentSpecV1>, SdkError> {
    let components_dir = registry_root.join("components");
    if !components_dir.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing components registry directory: {}",
            components_dir.display()
        )));
    }

    let mut specs = Vec::new();
    for entry in fs::read_dir(&components_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let kv = parse_kv_file(&path)?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("component")
            .to_string();
        let name = kv.get("name").cloned().unwrap_or(stem.clone());
        let phase = kv
            .get("phase")
            .cloned()
            .unwrap_or_else(|| "infer".to_string());
        let template_rel = kv
            .get("template")
            .cloned()
            .unwrap_or_else(|| format!("templates/{stem}.ocl"));
        let provides = parse_csv(kv.get("provides"));
        let requires = parse_csv(kv.get("requires"));

        let canonical = format!(
            "name={name};phase={phase};provides={};requires={};template={template_rel}",
            provides.join(","),
            requires.join(",")
        );
        specs.push(ComponentSpecV1 {
            name,
            phase,
            provides,
            requires,
            template_rel,
            hash64: fnv1a64_hex(&canonical),
        });
    }

    specs.sort_by(|a, b| {
        phase_rank(&a.phase)
            .cmp(&phase_rank(&b.phase))
            .then(a.name.cmp(&b.name))
            .then(a.hash64.cmp(&b.hash64))
    });
    Ok(specs)
}

pub fn load_phenotype_spec(path: &Path) -> Result<PhenotypeSpecV1, SdkError> {
    let kv = parse_kv_file(path)?;
    let mut components = Vec::new();
    if let Some(csv) = kv.get("components") {
        for token in csv.split(',') {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                components.push(trimmed.to_string());
            }
        }
    }

    let raw = fs::read_to_string(path)?;
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("component=") {
            let name = strip_quotes(rest.trim());
            if !name.is_empty() {
                components.push(name.to_string());
            }
        }
    }

    components.sort();
    components.dedup();
    if components.is_empty() {
        return Err(SdkError::MissingProject(format!(
            "phenotype has no components: {}",
            path.display()
        )));
    }
    if components.len() > MAX_COMPONENTS {
        return Err(SdkError::MissingProject(format!(
            "phenotype components exceed cap {}",
            MAX_COMPONENTS
        )));
    }

    Ok(PhenotypeSpecV1 { components })
}

pub fn compose_phenotype(
    project_root: &Path,
    phenotype_path: &Path,
    registry_root: &Path,
    locked: bool,
) -> Result<ComposeSummary, SdkError> {
    check_project_with_lock(project_root, locked)?;

    let phenotype = load_phenotype_spec(phenotype_path)?;
    let catalog = load_component_catalog(registry_root)?;
    let catalog_map: BTreeMap<String, ComponentSpecV1> = catalog
        .into_iter()
        .map(|spec| (spec.name.clone(), spec))
        .collect();

    let mut selected = Vec::new();
    for comp in &phenotype.components {
        let Some(spec) = catalog_map.get(comp) else {
            return Err(SdkError::MissingProject(format!(
                "compose failed: missing component `{comp}` in registry"
            )));
        };
        selected.push(spec.clone());
    }

    let provided: BTreeSet<String> = selected
        .iter()
        .flat_map(|spec| spec.provides.iter().cloned())
        .collect();
    for spec in &selected {
        for req in &spec.requires {
            if !provided.contains(req) {
                return Err(SdkError::MissingProject(format!(
                    "compose failed: unsatisfied requirement `{req}` for component `{}`",
                    spec.name
                )));
            }
        }
    }

    selected.sort_by(|a, b| {
        phase_rank(&a.phase)
            .cmp(&phase_rank(&b.phase))
            .then(a.name.cmp(&b.name))
            .then(a.hash64.cmp(&b.hash64))
    });

    let layout = project_layout(project_root);
    let generated_dir = layout.root.join("src").join("generated");
    fs::create_dir_all(&generated_dir)?;

    let mut generated_files = Vec::new();
    for spec in &selected {
        let file_name = format!("{}.ocl", sanitize_name(&spec.name));
        let out_path = generated_dir.join(file_name);
        let template_path = registry_root.join(&spec.template_rel);
        let body = if template_path.exists() {
            fs::read_to_string(&template_path)?
        } else {
            format!(
                "module generated.{};\nlet ready = true;\ncondition(ready);\n",
                sanitize_name(&spec.name)
            )
        };
        fs::write(&out_path, body)?;
        generated_files.push(out_path);
    }

    let module_path = generated_dir.join("mod.ocl");
    let mut module_body = String::from("module generated.mod;\n");
    module_body.push_str("let component_count = ");
    module_body.push_str(&selected.len().to_string());
    module_body.push_str(";\n");
    module_body.push_str("let generated_ready = true;\ncondition(generated_ready);\n");
    fs::write(&module_path, module_body)?;
    generated_files.push(module_path.clone());

    let components: Vec<String> = selected.iter().map(|s| s.name.clone()).collect();
    let proof = AssemblyProofV1 {
        catalog_hash64: hash_catalog(&selected),
        phenotype_hash64: fnv1a64_hex(&components.join(",")),
        generated_hash64: hash_generated_files(&generated_files)?,
        components,
    };
    let proof_path = layout.root.join("assembly_proof.toml");
    fs::write(&proof_path, encode_proof(&proof))?;

    Ok(ComposeSummary {
        selected_components: selected.len(),
        generated_files: generated_files.len(),
        generated_module: module_path,
        proof_path,
    })
}

pub fn verify_assembly(
    project_root: &Path,
    phenotype_path: &Path,
    registry_root: &Path,
    locked: bool,
) -> Result<VerifySummary, SdkError> {
    check_project_with_lock(project_root, locked)?;

    let phenotype = load_phenotype_spec(phenotype_path)?;
    let catalog = load_component_catalog(registry_root)?;
    let catalog_map: BTreeMap<String, ComponentSpecV1> = catalog
        .into_iter()
        .map(|spec| (spec.name.clone(), spec))
        .collect();

    let mut selected = Vec::new();
    for comp in &phenotype.components {
        let Some(spec) = catalog_map.get(comp) else {
            return Err(SdkError::MissingProject(format!(
                "verify failed: missing component `{comp}` in registry"
            )));
        };
        selected.push(spec.clone());
    }
    selected.sort_by(|a, b| {
        phase_rank(&a.phase)
            .cmp(&phase_rank(&b.phase))
            .then(a.name.cmp(&b.name))
            .then(a.hash64.cmp(&b.hash64))
    });

    let proof_path = project_root.join("assembly_proof.toml");
    if !proof_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing assembly proof: {}",
            proof_path.display()
        )));
    }
    let proof_raw = fs::read_to_string(&proof_path)?;
    let proof = decode_proof(&proof_raw)?;

    let expected_components: Vec<String> = selected.iter().map(|s| s.name.clone()).collect();
    if proof.components != expected_components {
        return Err(SdkError::LockMismatch(
            "assembly proof mismatch: component list changed".to_string(),
        ));
    }

    let expected_catalog_hash = hash_catalog(&selected);
    if proof.catalog_hash64 != expected_catalog_hash {
        return Err(SdkError::LockMismatch(
            "assembly proof mismatch: catalog hash changed".to_string(),
        ));
    }

    let expected_phenotype_hash = fnv1a64_hex(&expected_components.join(","));
    if proof.phenotype_hash64 != expected_phenotype_hash {
        return Err(SdkError::LockMismatch(
            "assembly proof mismatch: phenotype hash changed".to_string(),
        ));
    }

    let generated_dir = project_root.join("src").join("generated");
    if !generated_dir.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing generated directory: {}",
            generated_dir.display()
        )));
    }
    let generated_files = collect_generated_ocl_files(&generated_dir)?;
    let expected_generated_hash = hash_generated_files(&generated_files)?;
    if proof.generated_hash64 != expected_generated_hash {
        return Err(SdkError::LockMismatch(
            "assembly proof mismatch: generated hash changed".to_string(),
        ));
    }

    Ok(VerifySummary {
        ok: true,
        selected_components: selected.len(),
    })
}

fn parse_kv_file(path: &Path) -> Result<BTreeMap<String, String>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut out = BTreeMap::new();
    for line in raw.lines() {
        let cleaned = line.split('#').next().unwrap_or_default().trim();
        if cleaned.is_empty() {
            continue;
        }
        let Some((k, v)) = cleaned.split_once('=') else {
            continue;
        };
        let key = k.trim().to_string();
        let value = strip_quotes(v.trim()).to_string();
        out.insert(key, value);
    }
    Ok(out)
}

fn parse_csv(value: Option<&String>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for token in value.split(',') {
        let t = token.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}

fn phase_rank(phase: &str) -> u8 {
    match phase {
        "sense" => 0,
        "infer" => 1,
        "commit" => 2,
        "render" => 3,
        _ => 9,
    }
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

fn hash_catalog(selected: &[ComponentSpecV1]) -> String {
    let mut chunks = Vec::new();
    for spec in selected {
        chunks.push(format!(
            "{}|{}|{}|{}|{}",
            spec.name,
            spec.phase,
            spec.provides.join(","),
            spec.requires.join(","),
            spec.hash64
        ));
    }
    fnv1a64_hex(&chunks.join("\n"))
}

fn collect_generated_ocl_files(generated_dir: &Path) -> Result<Vec<PathBuf>, SdkError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(generated_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ocl") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn hash_generated_files(files: &[PathBuf]) -> Result<String, SdkError> {
    let mut chunks = Vec::new();
    for file in files {
        let content = fs::read_to_string(file)?;
        chunks.push(format!("{}|{}", file.to_string_lossy(), content));
    }
    Ok(fnv1a64_hex(&chunks.join("\n")))
}

fn encode_proof(proof: &AssemblyProofV1) -> String {
    let mut out = String::new();
    out.push_str("version=1\n");
    out.push_str("catalog_hash64=");
    out.push_str(&proof.catalog_hash64);
    out.push('\n');
    out.push_str("phenotype_hash64=");
    out.push_str(&proof.phenotype_hash64);
    out.push('\n');
    out.push_str("generated_hash64=");
    out.push_str(&proof.generated_hash64);
    out.push('\n');
    for component in &proof.components {
        out.push_str("component=");
        out.push_str(component);
        out.push('\n');
    }
    out
}

fn decode_proof(raw: &str) -> Result<AssemblyProofV1, SdkError> {
    let kv = parse_kv_from_raw(raw);
    let mut components = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(comp) = trimmed.strip_prefix("component=") {
            let c = strip_quotes(comp.trim());
            if !c.is_empty() {
                components.push(c.to_string());
            }
        }
    }
    components.sort();
    components.dedup();

    let Some(catalog_hash64) = kv.get("catalog_hash64").cloned() else {
        return Err(SdkError::LockMismatch(
            "invalid assembly_proof.toml: missing catalog_hash64".to_string(),
        ));
    };
    let Some(phenotype_hash64) = kv.get("phenotype_hash64").cloned() else {
        return Err(SdkError::LockMismatch(
            "invalid assembly_proof.toml: missing phenotype_hash64".to_string(),
        ));
    };
    let Some(generated_hash64) = kv.get("generated_hash64").cloned() else {
        return Err(SdkError::LockMismatch(
            "invalid assembly_proof.toml: missing generated_hash64".to_string(),
        ));
    };

    Ok(AssemblyProofV1 {
        catalog_hash64,
        phenotype_hash64,
        generated_hash64,
        components,
    })
}

fn parse_kv_from_raw(raw: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in raw.lines() {
        let cleaned = line.split('#').next().unwrap_or_default().trim();
        if cleaned.is_empty() {
            continue;
        }
        let Some((k, v)) = cleaned.split_once('=') else {
            continue;
        };
        out.insert(k.trim().to_string(), strip_quotes(v.trim()).to_string());
    }
    out
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
