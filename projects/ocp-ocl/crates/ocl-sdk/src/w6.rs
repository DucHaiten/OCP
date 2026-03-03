use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ocl_runtime_core::{parse_program, Expr, Stmt};

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
    collect_ocl_files(&root.join("src"), &mut files)?;
    collect_ocl_files(&root.join("tests"), &mut files)?;
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

fn deterministic_sign(context: &str, message: &[u8]) -> (String, String) {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ocl-sdk-signing-key-v1|");
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
