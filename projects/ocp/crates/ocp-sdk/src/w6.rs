use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ocp_runtime_core::{parse_program, Expr, Stmt};

use crate::w5::parse_cosmos_kits_v1;
use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRegistryEntryV1 {
    pub id: String,
    pub key_prefix: String,
    pub platform: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRegistryIndexV1 {
    pub entries: Vec<PluginRegistryEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLockEntryV1 {
    pub id: String,
    pub key_prefix: String,
    pub platform: String,
    pub command: String,
    pub signature_b64: String,
    pub signer_pub_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLockV1 {
    pub entries: Vec<PluginLockEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLockSyncSummary {
    pub plugins_synced: usize,
    pub lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginVerifySummary {
    pub plugins_verified: usize,
    pub lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganRegistryEntryV1 {
    pub pack_id: String,
    pub version: String,
    pub platform: String,
    pub artifact_rel: String,
    pub provides_caps: Vec<String>,
    pub source: String,
    pub abi_version: String,
    pub signer_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganRegistryIndexV1 {
    pub entries: Vec<OrganRegistryEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganLockEntryV1 {
    pub pack_id: String,
    pub version: String,
    pub platform: String,
    pub artifact_rel: String,
    pub artifact_hash256: String,
    pub signer_id: String,
    pub abi_version: String,
    pub signature_b64: String,
    pub signer_pub_b64: String,
    pub provides_caps: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganLockV1 {
    pub entries: Vec<OrganLockEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganLockSyncSummaryV1 {
    pub organs_synced: usize,
    pub lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganVerifySummaryV1 {
    pub organs_verified: usize,
    pub lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganInstallSummaryV1 {
    pub pack_id: String,
    pub version: String,
    pub platform: String,
    pub artifact_hash256: String,
    pub install_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitDoctorSummaryV1 {
    pub kits_checked: usize,
    pub required_organs: Vec<String>,
}

pub fn canonical_plugin_sign_message(entry: &PluginRegistryEntryV1) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        entry.id, entry.key_prefix, entry.platform, entry.command
    )
}

pub fn verify_plugin_signature(
    message: &str,
    signature_b64: &str,
    signer_pub_b64: &str,
) -> Result<(), SdkError> {
    let signature_raw = B64
        .decode(signature_b64)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid plugin signature base64: {e}")))?;
    let signer_pub_raw = B64.decode(signer_pub_b64).map_err(|e| {
        SdkError::SupplyInvalid(format!("invalid plugin signer public key base64: {e}"))
    })?;

    let signer_pub_bytes: [u8; 32] = signer_pub_raw
        .as_slice()
        .try_into()
        .map_err(|_| SdkError::SupplyInvalid("invalid plugin signer key length".to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(&signer_pub_bytes)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid plugin signer key: {e}")))?;
    let signature = Signature::from_slice(&signature_raw)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid plugin signature material: {e}")))?;

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| SdkError::SupplyInvalid("plugin signature verification failed".to_string()))
}

pub fn sync_plugin_lock_v1(root: &Path) -> Result<PluginLockSyncSummary, SdkError> {
    let index_path = root.join("plugins").join("index.toml");
    if !index_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing plugin registry index: {}",
            index_path.display()
        )));
    }
    let index = parse_plugin_registry_index_v1(&index_path)?;
    if index.entries.is_empty() {
        return Err(SdkError::MissingProject(
            "plugin index has no [[plugin]] entries".to_string(),
        ));
    }

    let mut lock_entries = Vec::new();
    for entry in &index.entries {
        let msg = canonical_plugin_sign_message(entry);
        let (signature_b64, signer_pub_b64) = deterministic_sign("plugin-lock-v1", msg.as_bytes());
        lock_entries.push(PluginLockEntryV1 {
            id: entry.id.clone(),
            key_prefix: entry.key_prefix.clone(),
            platform: entry.platform.clone(),
            command: entry.command.clone(),
            signature_b64,
            signer_pub_b64,
        });
    }

    let lock = PluginLockV1 {
        entries: lock_entries,
    };
    let lock_path = root.join("plugins.lock.v1");
    let encoded = encode_plugin_lock_v1(&lock);
    fs::write(&lock_path, encoded)?;
    Ok(PluginLockSyncSummary {
        plugins_synced: lock.entries.len(),
        lock_path,
    })
}

pub fn verify_plugin_lock_v1(root: &Path) -> Result<PluginVerifySummary, SdkError> {
    let lock_path = root.join("plugins.lock.v1");
    if !lock_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing plugin lock: {}",
            lock_path.display()
        )));
    }
    let lock = parse_plugin_lock_v1(&lock_path)?;
    if lock.entries.is_empty() {
        return Err(SdkError::MissingProject(
            "plugin lock has no `plugin=` rows".to_string(),
        ));
    }

    for entry in &lock.entries {
        let msg = format!(
            "{}\t{}\t{}\t{}",
            entry.id, entry.key_prefix, entry.platform, entry.command
        );
        verify_plugin_signature(&msg, &entry.signature_b64, &entry.signer_pub_b64)?;
    }

    Ok(PluginVerifySummary {
        plugins_verified: lock.entries.len(),
        lock_path,
    })
}

pub fn resolve_platform_tag_v1() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x64".to_string(),
        ("linux", "x86_64") => "linux-x64".to_string(),
        ("macos", "aarch64") => "macos-arm64".to_string(),
        (os, arch) => format!("{os}-{arch}"),
    }
}

pub fn canonical_organ_sign_message(
    entry: &OrganRegistryEntryV1,
    artifact_hash256: &str,
) -> String {
    let caps = if entry.provides_caps.is_empty() {
        "-".to_string()
    } else {
        entry.provides_caps.join(",")
    };
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        entry.pack_id,
        entry.version,
        entry.platform,
        entry.artifact_rel,
        artifact_hash256,
        caps,
        entry.source,
        entry.abi_version
    )
}

pub fn verify_organ_entry_signature(
    message: &str,
    signature_b64: &str,
    signer_pub_b64: &str,
) -> Result<(), SdkError> {
    let signature_raw = B64
        .decode(signature_b64)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid organ signature base64: {e}")))?;
    let signer_pub_raw = B64.decode(signer_pub_b64).map_err(|e| {
        SdkError::SupplyInvalid(format!("invalid organ signer public key base64: {e}"))
    })?;

    let signer_pub_bytes: [u8; 32] = signer_pub_raw
        .as_slice()
        .try_into()
        .map_err(|_| SdkError::SupplyInvalid("invalid organ signer key length".to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(&signer_pub_bytes)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid organ signer key: {e}")))?;
    let signature = Signature::from_slice(&signature_raw)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid organ signature material: {e}")))?;

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| SdkError::SupplyInvalid("organ signature verification failed".to_string()))
}

pub fn collect_required_organs_from_cosmos_v1(
    root: &Path,
    locked: bool,
) -> Result<Vec<String>, SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Ok(Vec::new());
    }

    let kits = if locked {
        let lock_path = root.join("cosmos.lock.v1");
        if !lock_path.exists() {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-LOCK-REQUIRED: missing `{}`",
                lock_path.display()
            )));
        }
        parse_kits_from_cosmos_lock_v1(&lock_path)?
    } else {
        parse_cosmos_kits_v1(&cosmos_path)?
    };

    let mut required = kits
        .iter()
        .flat_map(|kit| kit.required_organs.iter().cloned())
        .collect::<Vec<String>>();
    required.sort();
    required.dedup();
    Ok(required)
}

pub fn list_kits_from_cosmos_v1(root: &Path) -> Result<Vec<crate::w5::ViewKitBindingV1>, SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Ok(Vec::new());
    }
    parse_cosmos_kits_v1(&cosmos_path)
}

pub fn sync_organs_lock_v1(
    root: &Path,
    registry_index: &Path,
) -> Result<OrganLockSyncSummaryV1, SdkError> {
    let required = collect_required_organs_from_cosmos_v1(root, false)?;
    let index = parse_organ_registry_index_v1(registry_index)?;
    let platform = resolve_platform_tag_v1();
    let mut entries = Vec::<OrganLockEntryV1>::new();

    for item in required {
        let (pack_id, version) = parse_required_organ_id_v1(&item)?;
        let (selected, exact_platform_match) =
            select_registry_organ_entry_v1(&index.entries, &pack_id, &version, &platform);

        let selected = if let Some(entry) = selected {
            entry
        } else if exact_platform_match {
            return Err(SdkError::SupplyInvalid(format!(
                "V-ORGAN-PLATFORM-MISMATCH: organ `{pack_id}@{version}` has no artifact for platform `{platform}`"
            )));
        } else {
            return Err(SdkError::SupplyInvalid(format!(
                "V-ORGAN-NOT-FOUND: organ `{pack_id}@{version}` is not in registry index"
            )));
        };

        let registry_root = registry_index
            .parent()
            .ok_or_else(|| SdkError::MissingProject("invalid registry index path".to_string()))?;
        let artifact_path = registry_root.join(&selected.artifact_rel);
        if !artifact_path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "V-ORGAN-ARTIFACT-MISSING: artifact not found `{}`",
                artifact_path.display()
            )));
        }
        let artifact_bytes = fs::read(&artifact_path)?;
        let artifact_hash256 = blake3::hash(&artifact_bytes).to_hex().to_string();
        let message = canonical_organ_sign_message(selected, &artifact_hash256);
        let (signature_b64, signer_pub_b64) =
            deterministic_sign("organ-lock-v1", message.as_bytes());
        entries.push(OrganLockEntryV1 {
            pack_id: selected.pack_id.clone(),
            version: selected.version.clone(),
            platform: selected.platform.clone(),
            artifact_rel: selected.artifact_rel.clone(),
            artifact_hash256,
            signer_id: selected.signer_id.clone(),
            abi_version: selected.abi_version.clone(),
            signature_b64,
            signer_pub_b64,
            provides_caps: selected.provides_caps.clone(),
            source: selected.source.clone(),
        });
    }

    entries.sort_by(|a, b| {
        a.pack_id
            .cmp(&b.pack_id)
            .then(a.version.cmp(&b.version))
            .then(a.platform.cmp(&b.platform))
    });
    let lock = OrganLockV1 { entries };
    let lock_path = root.join("organs.lock.v1");
    fs::write(&lock_path, encode_organ_lock_v1(&lock))?;
    Ok(OrganLockSyncSummaryV1 {
        organs_synced: lock.entries.len(),
        lock_path,
    })
}

pub fn verify_organs_lock_v1(root: &Path, locked: bool) -> Result<OrganVerifySummaryV1, SdkError> {
    let lock_path = root.join("organs.lock.v1");
    let required = collect_required_organs_from_cosmos_v1(root, locked)?;
    if !lock_path.exists() {
        if locked && !required.is_empty() {
            return Err(SdkError::SupplyInvalid(format!(
                "V-ORGANS-LOCK-REQUIRED: missing `{}`",
                lock_path.display()
            )));
        }
        return Ok(OrganVerifySummaryV1 {
            organs_verified: 0,
            lock_path,
        });
    }

    let lock = parse_organ_lock_v1(&lock_path)?;
    let platform = resolve_platform_tag_v1();
    let mut covered = HashSet::<String>::new();
    for entry in &lock.entries {
        if locked && entry.platform != platform {
            return Err(SdkError::SupplyInvalid(format!(
                "V-ORGAN-PLATFORM-MISMATCH: `{}` expected `{}` got `{}`",
                entry.pack_id, platform, entry.platform
            )));
        }
        let message = canonical_organ_sign_message(
            &OrganRegistryEntryV1 {
                pack_id: entry.pack_id.clone(),
                version: entry.version.clone(),
                platform: entry.platform.clone(),
                artifact_rel: entry.artifact_rel.clone(),
                provides_caps: entry.provides_caps.clone(),
                source: entry.source.clone(),
                abi_version: entry.abi_version.clone(),
                signer_id: entry.signer_id.clone(),
            },
            &entry.artifact_hash256,
        );
        verify_organ_entry_signature(&message, &entry.signature_b64, &entry.signer_pub_b64)?;
        covered.insert(format!("{}@{}", entry.pack_id, entry.version));
    }

    if locked {
        for needed in required {
            if !covered.contains(&needed) {
                return Err(SdkError::SupplyInvalid(format!(
                    "V-ORGAN-REQUIRED-MISSING: required organ `{needed}` is missing in organs.lock.v1"
                )));
            }
        }
    }

    Ok(OrganVerifySummaryV1 {
        organs_verified: lock.entries.len(),
        lock_path,
    })
}

pub fn install_organs_v1(
    root: &Path,
    registry_index: &Path,
    name: &str,
    version: &str,
    locked: bool,
) -> Result<OrganInstallSummaryV1, SdkError> {
    if locked {
        let _ = verify_organs_lock_v1(root, true)?;
    }
    let index = parse_organ_registry_index_v1(registry_index)?;
    let platform = resolve_platform_tag_v1();
    let (selected, exact_platform_match) =
        select_registry_organ_entry_v1(&index.entries, name, version, &platform);
    let selected = if let Some(entry) = selected {
        entry
    } else if exact_platform_match {
        return Err(SdkError::SupplyInvalid(format!(
            "V-ORGAN-PLATFORM-MISMATCH: organ `{name}@{version}` has no artifact for platform `{platform}`"
        )));
    } else {
        return Err(SdkError::SupplyInvalid(format!(
            "V-ORGAN-NOT-FOUND: organ `{name}@{version}` is not in registry index"
        )));
    };

    let registry_root = registry_index
        .parent()
        .ok_or_else(|| SdkError::MissingProject("invalid registry index path".to_string()))?;
    let src = registry_root.join(&selected.artifact_rel);
    if !src.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "V-ORGAN-ARTIFACT-MISSING: artifact not found `{}`",
            src.display()
        )));
    }
    let bytes = fs::read(&src)?;
    let artifact_hash256 = blake3::hash(&bytes).to_hex().to_string();
    let artifact_name = Path::new(&selected.artifact_rel)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("artifact.bin");
    let install_dir = root
        .join(".ocp")
        .join("organs")
        .join(&selected.pack_id)
        .join(&selected.version)
        .join(&selected.platform);
    fs::create_dir_all(&install_dir)?;
    let install_path = install_dir.join(artifact_name);
    fs::write(&install_path, &bytes)?;
    Ok(OrganInstallSummaryV1 {
        pack_id: selected.pack_id.clone(),
        version: selected.version.clone(),
        platform: selected.platform.clone(),
        artifact_hash256,
        install_path,
    })
}

pub fn run_kit_doctor_v1(root: &Path, locked: bool) -> Result<KitDoctorSummaryV1, SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Ok(KitDoctorSummaryV1 {
            kits_checked: 0,
            required_organs: Vec::new(),
        });
    }
    let kits = parse_cosmos_kits_v1(&cosmos_path)?;
    let required = collect_required_organs_from_cosmos_v1(root, locked)?;
    if !required.is_empty() {
        verify_organs_lock_v1(root, locked)
            .map_err(|e| SdkError::SupplyInvalid(format!("V-KIT-DOCTOR-WIRING: {}", e)))?;
    }
    Ok(KitDoctorSummaryV1 {
        kits_checked: kits.len(),
        required_organs: required,
    })
}

pub fn resolve_plugin_for_custom_key(
    root: &Path,
    key: &str,
    locked: bool,
) -> Result<Option<PluginLockEntryV1>, SdkError> {
    let lock_path = root.join("plugins.lock.v1");
    if !lock_path.exists() {
        return Ok(None);
    }
    if locked {
        verify_plugin_lock_v1(root)?;
    }
    let lock = parse_plugin_lock_v1(&lock_path)?;
    Ok(lock
        .entries
        .into_iter()
        .filter(|entry| key.starts_with(&entry.key_prefix))
        .max_by_key(|entry| entry.key_prefix.len()))
}

pub fn collect_project_custom_keys(root: &Path) -> Result<Vec<String>, SdkError> {
    let mut files = Vec::new();
    collect_ocp_files(&root.join("src"), &mut files)?;
    collect_ocp_files(&root.join("tests"), &mut files)?;
    files.sort();

    let mut out = Vec::new();
    for (idx, file) in files.iter().enumerate() {
        let source = fs::read_to_string(file)?;
        let program =
            parse_program(&source, idx as u32 + 1).map_err(|e| SdkError::Runtime(e.into()))?;
        collect_custom_keys_from_stmts(&program.statements, &mut out);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn collect_ocp_files(base: &Path, out: &mut Vec<PathBuf>) -> Result<(), SdkError> {
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ocp_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("ocp") {
            out.push(path);
        }
    }
    Ok(())
}

fn collect_custom_keys_from_stmts(stmts: &[Stmt], out: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Observe {
                key: Expr::String { value, .. },
                ..
            } => {
                if value.starts_with("custom.") {
                    out.push(value.clone());
                }
            }
            Stmt::Observe { .. } => {}
            Stmt::FnDef { body, .. } | Stmt::ForRange { body, .. } => {
                collect_custom_keys_from_stmts(body, out);
            }
            Stmt::Match(m) => {
                collect_custom_keys_from_stmts(&m.ok_arm, out);
                collect_custom_keys_from_stmts(&m.degraded_arm, out);
                collect_custom_keys_from_stmts(&m.insufficient_arm, out);
                collect_custom_keys_from_stmts(&m.deferred_arm, out);
            }
            _ => {}
        }
    }
}

fn parse_plugin_registry_index_v1(path: &Path) -> Result<PluginRegistryIndexV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut entries = Vec::new();
    let mut current = PluginRegistryEntryV1 {
        id: String::new(),
        key_prefix: String::new(),
        platform: String::new(),
        command: String::new(),
    };
    let mut in_plugin = false;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[plugin]]" {
            if in_plugin && is_valid_registry_entry(&current) {
                entries.push(current.clone());
            }
            current = PluginRegistryEntryV1 {
                id: String::new(),
                key_prefix: String::new(),
                platform: String::new(),
                command: String::new(),
            };
            in_plugin = true;
            continue;
        }
        if !in_plugin {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "id" => current.id = value,
            "key_prefix" => current.key_prefix = value,
            "platform" => current.platform = value,
            "command" => current.command = value,
            _ => {}
        }
    }

    if in_plugin && is_valid_registry_entry(&current) {
        entries.push(current);
    }

    Ok(PluginRegistryIndexV1 { entries })
}

fn is_valid_registry_entry(entry: &PluginRegistryEntryV1) -> bool {
    !entry.id.is_empty()
        && !entry.key_prefix.is_empty()
        && !entry.platform.is_empty()
        && !entry.command.is_empty()
}

fn parse_plugin_lock_v1(path: &Path) -> Result<PluginLockV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut entries = Vec::new();
    let mut has_version = false;

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        let Some(row) = line.strip_prefix("plugin=") else {
            continue;
        };
        let parts: Vec<&str> = row.split('\t').collect();
        if parts.len() != 6 {
            return Err(SdkError::SupplyInvalid(
                "invalid plugin lock row (expected 6 tab-separated columns)".to_string(),
            ));
        }
        entries.push(PluginLockEntryV1 {
            id: parts[0].to_string(),
            key_prefix: parts[1].to_string(),
            platform: parts[2].to_string(),
            command: parts[3].to_string(),
            signature_b64: parts[4].to_string(),
            signer_pub_b64: parts[5].to_string(),
        });
    }

    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "plugins.lock.v1 missing `version=1`".to_string(),
        ));
    }

    Ok(PluginLockV1 { entries })
}

fn encode_plugin_lock_v1(lock: &PluginLockV1) -> String {
    let mut out = String::from("version=1\n");
    for entry in &lock.entries {
        out.push_str("plugin=");
        out.push_str(&entry.id);
        out.push('\t');
        out.push_str(&entry.key_prefix);
        out.push('\t');
        out.push_str(&entry.platform);
        out.push('\t');
        out.push_str(&entry.command);
        out.push('\t');
        out.push_str(&entry.signature_b64);
        out.push('\t');
        out.push_str(&entry.signer_pub_b64);
        out.push('\n');
    }
    out
}

fn parse_kits_from_cosmos_lock_v1(
    path: &Path,
) -> Result<Vec<crate::w5::ViewKitBindingV1>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut kits = Vec::new();
    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(payload) = line.strip_prefix("kit=") else {
            continue;
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 4 && parts.len() != 5 {
            return Err(SdkError::LockMismatch(
                "V-KIT-LOCK-INVALID: `kit=` row must have 4 or 5 columns".to_string(),
            ));
        }
        let mut required_organs = if parts.len() == 5 {
            parse_csv_list_v1(parts[4])
        } else {
            Vec::new()
        };
        required_organs.sort();
        required_organs.dedup();
        kits.push(crate::w5::ViewKitBindingV1 {
            kit_id: parts[0].to_string(),
            universe_id: parts[1].to_string(),
            bind_domain: parts[2].to_string(),
            bind_view: parts[3].to_string(),
            required_organs,
        });
    }
    Ok(kits)
}

fn parse_required_organ_id_v1(raw: &str) -> Result<(String, String), SdkError> {
    let Some((id, version)) = raw.split_once('@') else {
        return Err(SdkError::SupplyInvalid(format!(
            "V-ORGAN-ID-VERSION-INVALID: expected `name@version`, got `{raw}`"
        )));
    };
    let id = id.trim().to_string();
    let version = version.trim().to_string();
    if id.is_empty() || version.is_empty() {
        return Err(SdkError::SupplyInvalid(format!(
            "V-ORGAN-ID-VERSION-INVALID: expected `name@version`, got `{raw}`"
        )));
    }
    Ok((id, version))
}

fn parse_organ_registry_index_v1(path: &Path) -> Result<OrganRegistryIndexV1, SdkError> {
    if !path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing organ registry index: {}",
            path.display()
        )));
    }
    let raw = fs::read_to_string(path)?;
    let mut entries = Vec::new();
    let mut current = OrganRegistryEntryV1 {
        pack_id: String::new(),
        version: String::new(),
        platform: String::new(),
        artifact_rel: String::new(),
        provides_caps: Vec::new(),
        source: String::new(),
        abi_version: "v1".to_string(),
        signer_id: "dev-root-1".to_string(),
    };
    let mut in_organ = false;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[organ]]" {
            if in_organ && is_valid_organ_registry_entry_v1(&current) {
                entries.push(current.clone());
            }
            current = OrganRegistryEntryV1 {
                pack_id: String::new(),
                version: String::new(),
                platform: String::new(),
                artifact_rel: String::new(),
                provides_caps: Vec::new(),
                source: String::new(),
                abi_version: "v1".to_string(),
                signer_id: "dev-root-1".to_string(),
            };
            in_organ = true;
            continue;
        }
        if !in_organ {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim();
        match key {
            "pack_id" => current.pack_id = value.trim_matches('"').to_string(),
            "version" => current.version = value.trim_matches('"').to_string(),
            "platform" => current.platform = value.trim_matches('"').to_string(),
            "artifact_rel" => current.artifact_rel = value.trim_matches('"').to_string(),
            "source" => current.source = value.trim_matches('"').to_string(),
            "abi_version" => current.abi_version = value.trim_matches('"').to_string(),
            "signer_id" => current.signer_id = value.trim_matches('"').to_string(),
            "provides_caps" => current.provides_caps = parse_array_list_v1(value),
            _ => {}
        }
    }

    if in_organ && is_valid_organ_registry_entry_v1(&current) {
        entries.push(current);
    }

    entries.sort_by(|a, b| {
        a.pack_id
            .cmp(&b.pack_id)
            .then(a.version.cmp(&b.version))
            .then(a.platform.cmp(&b.platform))
    });
    Ok(OrganRegistryIndexV1 { entries })
}

fn is_valid_organ_registry_entry_v1(entry: &OrganRegistryEntryV1) -> bool {
    !entry.pack_id.is_empty()
        && !entry.version.is_empty()
        && !entry.platform.is_empty()
        && !entry.artifact_rel.is_empty()
}

fn select_registry_organ_entry_v1<'a>(
    entries: &'a [OrganRegistryEntryV1],
    pack_id: &str,
    version: &str,
    platform: &str,
) -> (Option<&'a OrganRegistryEntryV1>, bool) {
    let mut has_same_id_version = false;
    let mut selected = None;
    for entry in entries {
        if entry.pack_id == pack_id && entry.version == version {
            has_same_id_version = true;
            if entry.platform == platform {
                selected = Some(entry);
                break;
            }
        }
    }
    (selected, has_same_id_version)
}

fn encode_organ_lock_v1(lock: &OrganLockV1) -> String {
    let mut out = String::from("version=1\n");
    for entry in &lock.entries {
        out.push_str("organ=");
        out.push_str(&entry.pack_id);
        out.push('\t');
        out.push_str(&entry.version);
        out.push('\t');
        out.push_str(&entry.platform);
        out.push('\t');
        out.push_str(&entry.artifact_rel);
        out.push('\t');
        out.push_str(&entry.artifact_hash256);
        out.push('\t');
        out.push_str(&entry.signer_id);
        out.push('\t');
        out.push_str(&entry.abi_version);
        out.push('\t');
        out.push_str(&entry.signature_b64);
        out.push('\t');
        out.push_str(&entry.signer_pub_b64);
        out.push('\t');
        out.push_str(&join_csv_list_v1(&entry.provides_caps));
        out.push('\t');
        out.push_str(&entry.source);
        out.push('\n');
    }
    out
}

fn parse_organ_lock_v1(path: &Path) -> Result<OrganLockV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut has_version = false;
    let mut entries = Vec::new();
    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        let Some(row) = line.strip_prefix("organ=") else {
            continue;
        };
        let parts: Vec<&str> = row.split('\t').collect();
        if parts.len() != 11 {
            return Err(SdkError::SupplyInvalid(
                "V-ORGANS-LOCK-INVALID: `organ=` row must have 11 tab-separated columns"
                    .to_string(),
            ));
        }
        entries.push(OrganLockEntryV1 {
            pack_id: parts[0].to_string(),
            version: parts[1].to_string(),
            platform: parts[2].to_string(),
            artifact_rel: parts[3].to_string(),
            artifact_hash256: parts[4].to_string(),
            signer_id: parts[5].to_string(),
            abi_version: parts[6].to_string(),
            signature_b64: parts[7].to_string(),
            signer_pub_b64: parts[8].to_string(),
            provides_caps: parse_csv_list_v1(parts[9]),
            source: parts[10].to_string(),
        });
    }
    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "organs.lock.v1 missing `version=1`".to_string(),
        ));
    }
    Ok(OrganLockV1 { entries })
}

fn parse_csv_list_v1(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "-")
        .collect()
}

fn join_csv_list_v1(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

fn parse_array_list_v1(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Vec::new();
        }
        return inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    let single = value.trim_matches('"').to_string();
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}

fn deterministic_sign(context: &str, message: &[u8]) -> (String, String) {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ocp-sdk-signing-key-v1|");
    hasher.update(context.as_bytes());
    let seed = *hasher.finalize().as_bytes();

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    let signature = signing_key.sign(message);

    (
        B64.encode(signature.to_bytes()),
        B64.encode(verifying_key.to_bytes()),
    )
}
