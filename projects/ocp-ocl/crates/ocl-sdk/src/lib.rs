use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

pub mod m4;

pub use m4::{
    compose_phenotype, load_component_catalog, load_phenotype_spec, verify_assembly,
    AssemblyProofV1, ComponentSpecV1, ComposeSummary, PhenotypeSpecV1, VerifySummary,
};
use ocl_runtime_core::{check_file, normalize_text, run_file, RuntimeCoreError};

#[derive(Debug)]
pub enum SdkError {
    Io(std::io::Error),
    Runtime(RuntimeCoreError),
    MissingProject(String),
    FmtMismatch(Vec<String>),
    LockMismatch(String),
}

impl Display for SdkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Runtime(err) => write!(f, "{err}"),
            Self::MissingProject(msg) => write!(f, "{msg}"),
            Self::FmtMismatch(paths) => {
                write!(f, "format check failed for {} file(s): ", paths.len())?;
                write!(f, "{}", paths.join(", "))
            }
            Self::LockMismatch(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<RuntimeCoreError> for SdkError {
    fn from(value: RuntimeCoreError) -> Self {
        Self::Runtime(value)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub deps_lock: PathBuf,
    pub src_main: PathBuf,
    pub tests_dir: PathBuf,
    pub bundle_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSummary {
    pub files_checked: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSummary {
    pub steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactorSummary {
    pub ticks: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSummary {
    pub tests_run: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FmtSummary {
    pub files_touched: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildSummary {
    pub files_bundled: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockSyncSummary {
    pub deps_synced: usize,
}

pub fn project_layout(root: &Path) -> ProjectLayout {
    ProjectLayout {
        root: root.to_path_buf(),
        manifest: root.join("Ocl.toml"),
        deps_lock: root.join("deps.lock"),
        src_main: root.join("src").join("main.ocl"),
        tests_dir: root.join("tests"),
        bundle_dir: root.join(".oclbundle"),
    }
}

pub fn init_project(root: &Path) -> Result<ProjectLayout, SdkError> {
    let layout = project_layout(root);
    fs::create_dir_all(layout.src_main.parent().expect("src path must have parent"))?;
    fs::create_dir_all(&layout.tests_dir)?;

    if !layout.manifest.exists() {
        let project_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .replace('-', "_");
        let manifest = format!(
            "[package]\nname = \"{project_name}\"\nversion = \"0.1.0\"\n\n[targets]\ndefault = \"main\"\n\n[dependencies]\n"
        );
        fs::write(&layout.manifest, manifest)?;
    }

    if !layout.deps_lock.exists() {
        fs::write(&layout.deps_lock, "version=1\n")?;
    }

    if !layout.src_main.exists() {
        let sample = "let ready = true;\ncondition(ready);\n";
        fs::write(&layout.src_main, sample)?;
    }

    let smoke = layout.tests_dir.join("smoke.ocl");
    if !smoke.exists() {
        fs::write(smoke, "let t = true;\ncondition(t);\n")?;
    }

    Ok(layout)
}

fn verify_project_exists(layout: &ProjectLayout) -> Result<(), SdkError> {
    if !layout.manifest.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            layout.manifest.display()
        )));
    }
    if !layout.src_main.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing entry source: {}",
            layout.src_main.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestDep {
    name: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockDep {
    name: String,
    version: String,
    hash64: String,
}

fn parse_manifest_dependencies(manifest: &str) -> Result<Vec<ManifestDep>, SdkError> {
    let mut deps = Vec::new();
    let mut in_dependencies = false;

    for raw in manifest.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            return Err(SdkError::MissingProject(
                "invalid [dependencies] entry in Ocl.toml".to_string(),
            ));
        };
        let name = k.trim();
        let version = v.trim().trim_matches('"');
        if name.is_empty() || version.is_empty() {
            return Err(SdkError::MissingProject(
                "dependency name/version must not be empty".to_string(),
            ));
        }
        deps.push(ManifestDep {
            name: name.to_string(),
            version: version.to_string(),
        });
    }

    deps.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(deps)
}

fn to_lock_deps(deps: &[ManifestDep]) -> Vec<LockDep> {
    let mut out: Vec<LockDep> = deps
        .iter()
        .map(|dep| {
            let digest_input = format!("{}@{}", dep.name, dep.version);
            LockDep {
                name: dep.name.clone(),
                version: dep.version.clone(),
                hash64: fnv1a64_hex(&digest_input),
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    out
}

fn encode_lock_v1(lock_deps: &[LockDep]) -> String {
    let mut out = String::from("version=1\n");
    for dep in lock_deps {
        out.push_str("dep=");
        out.push_str(&dep.name);
        out.push('|');
        out.push_str(&dep.version);
        out.push('|');
        out.push_str(&dep.hash64);
        out.push('\n');
    }
    out
}

fn parse_lock_v1(text: &str) -> Result<Vec<LockDep>, SdkError> {
    let mut has_version = false;
    let mut deps = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        let Some(payload) = line.strip_prefix("dep=") else {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock line, expected `dep=...`".to_string(),
            ));
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 3 {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock dep format, expected `dep=name|version|hash64`".to_string(),
            ));
        }
        if parts[2].len() != 16 || !parts[2].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock hash64 format (expected 16 hex chars)".to_string(),
            ));
        }
        deps.push(LockDep {
            name: parts[0].to_string(),
            version: parts[1].to_string(),
            hash64: parts[2].to_ascii_lowercase(),
        });
    }

    if !has_version {
        return Err(SdkError::LockMismatch(
            "deps.lock missing `version=1` header".to_string(),
        ));
    }

    deps.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(deps)
}

fn read_expected_lock(layout: &ProjectLayout) -> Result<Vec<LockDep>, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest)?;
    Ok(to_lock_deps(&deps))
}

fn verify_lock_consistency(layout: &ProjectLayout) -> Result<Vec<LockDep>, SdkError> {
    let expected = read_expected_lock(layout)?;
    if !layout.deps_lock.exists() {
        return Err(SdkError::LockMismatch(format!(
            "missing deps.lock: {} (run `ocl lock sync <project_dir>`)",
            layout.deps_lock.display()
        )));
    }
    let raw = fs::read_to_string(&layout.deps_lock)?;
    let current = parse_lock_v1(&raw)?;
    if current != expected {
        return Err(SdkError::LockMismatch(
            "deps.lock mismatch with Ocl.toml dependencies (run `ocl lock sync <project_dir>`)"
                .to_string(),
        ));
    }
    Ok(current)
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn sync_deps_lock_v1(root: &Path) -> Result<LockSyncSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let expected = read_expected_lock(&layout)?;
    fs::write(&layout.deps_lock, encode_lock_v1(&expected))?;
    Ok(LockSyncSummary {
        deps_synced: expected.len(),
    })
}

fn collect_ocl_files(base: &Path, out: &mut Vec<PathBuf>) -> Result<(), SdkError> {
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ocl_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("ocl") {
            out.push(path);
        }
    }
    Ok(())
}

fn gather_project_ocl_files(layout: &ProjectLayout) -> Result<Vec<PathBuf>, SdkError> {
    let mut files = Vec::new();
    collect_ocl_files(&layout.root.join("src"), &mut files)?;
    collect_ocl_files(&layout.tests_dir, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn check_project(root: &Path) -> Result<CheckSummary, SdkError> {
    check_project_with_lock(root, false)
}

pub fn check_project_with_lock(root: &Path, locked: bool) -> Result<CheckSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    let files = gather_project_ocl_files(&layout)?;
    if files.is_empty() {
        return Err(SdkError::MissingProject(
            "project has no .ocl sources under src/ or tests/".to_string(),
        ));
    }
    for (idx, path) in files.iter().enumerate() {
        check_file(path, idx as u32 + 1)?;
    }
    Ok(CheckSummary {
        files_checked: files.len(),
    })
}

pub fn run_project(root: &Path) -> Result<RunSummary, SdkError> {
    run_project_with_lock(root, false)
}

pub fn run_project_with_lock(root: &Path, locked: bool) -> Result<RunSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    let out = run_file(&layout.src_main, 1, 4096)?;
    Ok(RunSummary { steps: out.steps })
}

pub fn run_reactor_ticks(root: &Path, ticks: u32) -> Result<ReactorSummary, SdkError> {
    run_reactor_ticks_with_lock(root, ticks, false)
}

pub fn run_reactor_ticks_with_lock(
    root: &Path,
    ticks: u32,
    locked: bool,
) -> Result<ReactorSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    let source = fs::read_to_string(&layout.src_main)?;
    if !source.contains("on_event") {
        return Err(SdkError::MissingProject(
            "reactor mode requires `fn on_event(...)` in src/main.ocl".to_string(),
        ));
    }
    let mut total_steps = 0u32;
    for _ in 0..ticks {
        let out = run_file(&layout.src_main, 1, 4096)?;
        total_steps = total_steps.saturating_add(out.steps);
    }
    Ok(ReactorSummary { ticks, total_steps })
}

pub fn test_project(root: &Path) -> Result<TestSummary, SdkError> {
    test_project_with_lock(root, false)
}

pub fn test_project_with_lock(root: &Path, locked: bool) -> Result<TestSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    let mut tests = Vec::new();
    collect_ocl_files(&layout.tests_dir, &mut tests)?;
    tests.sort();
    for (idx, test_file) in tests.iter().enumerate() {
        run_file(test_file, idx as u32 + 100, 4096)?;
    }
    Ok(TestSummary {
        tests_run: tests.len(),
    })
}

pub fn fmt_project(root: &Path, check_only: bool) -> Result<FmtSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let files = gather_project_ocl_files(&layout)?;
    let mut touched = 0usize;
    let mut mismatches = Vec::new();

    for file in files {
        let original = fs::read_to_string(&file)?;
        let formatted = normalize_text(&original);
        if original != formatted {
            touched += 1;
            if check_only {
                mismatches.push(file.to_string_lossy().to_string());
            } else {
                fs::write(&file, formatted)?;
            }
        }
    }

    if !mismatches.is_empty() {
        return Err(SdkError::FmtMismatch(mismatches));
    }

    Ok(FmtSummary {
        files_touched: touched,
    })
}

pub fn build_project(root: &Path) -> Result<BuildSummary, SdkError> {
    build_project_with_lock(root, false)
}

pub fn build_project_with_lock(root: &Path, locked: bool) -> Result<BuildSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let lock_deps = if locked {
        verify_lock_consistency(&layout)?
    } else {
        read_expected_lock(&layout)?
    };
    let files = gather_project_ocl_files(&layout)?;
    fs::create_dir_all(&layout.bundle_dir)?;

    let vendor_root = layout.bundle_dir.join("files");
    fs::create_dir_all(&vendor_root)?;
    let mut manifest_lines = Vec::new();

    for file in &files {
        let rel = file
            .strip_prefix(&layout.root)
            .map_err(|_| SdkError::MissingProject("invalid project path layout".to_string()))?;
        let rel_display = rel.to_string_lossy().replace('\\', "/");
        manifest_lines.push(rel_display.clone());
        let dst = vendor_root.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, dst)?;
    }

    manifest_lines.sort();
    let mut manifest_body = String::new();
    manifest_body.push_str("version=1\n");
    for line in manifest_lines {
        manifest_body.push_str("file=");
        manifest_body.push_str(&line);
        manifest_body.push('\n');
    }
    for dep in lock_deps {
        manifest_body.push_str("dep=");
        manifest_body.push_str(&dep.name);
        manifest_body.push('|');
        manifest_body.push_str(&dep.version);
        manifest_body.push('|');
        manifest_body.push_str(&dep.hash64);
        manifest_body.push('\n');
    }
    fs::write(layout.bundle_dir.join("manifest.txt"), manifest_body)?;

    Ok(BuildSummary {
        files_bundled: files.len(),
    })
}
