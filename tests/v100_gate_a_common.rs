#![allow(dead_code)]
#![allow(clippy::duplicate_mod)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v16_gate_a_common.rs"]
mod v16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractInventoryEntryV100 {
    pub contract_id: String,
    pub version: String,
    pub producer: String,
    pub consumer: String,
    pub schema_hash: String,
    pub allowed_diffs_v100: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredContractsV100 {
    pub schema: String,
    pub contract_id: String,
    pub version: String,
    pub entries: Vec<ContractInventoryEntryV100>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w100_target_root() -> PathBuf {
    repo_root().join("target").join("ocl").join("w100")
}

pub fn run_manifest_path() -> PathBuf {
    w100_target_root().join("meta").join("run_manifest.json")
}

pub fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

pub fn canonical_contract_roots_v100() -> Vec<&'static str> {
    vec![
        "contracts/release/v1.0",
        "contracts/docs/v1.0",
        "contracts/legal/v1.0",
        "contracts/business/v1.0",
        "contracts/security/v1.0",
    ]
}

fn collect_json_contracts_under(root: &Path, repo_root: &Path, out: &mut Vec<String>) {
    let entries = fs::read_dir(root).unwrap_or_else(|_| panic!("read dir {}", root.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|_| panic!("read entry {}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_json_contracts_under(&path, repo_root, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or_else(|_| panic!("strip prefix {}", path.display()))
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
}

pub fn canonical_contract_producers_v100() -> Vec<String> {
    let root = repo_root();
    let mut items = Vec::<String>::new();
    for rel_root in canonical_contract_roots_v100() {
        collect_json_contracts_under(&root.join(rel_root), &root, &mut items);
    }
    items.sort();
    items
}

pub fn release_required_contracts_path() -> PathBuf {
    contracts_root()
        .join("release")
        .join("v1.0")
        .join("release_required_contracts.v1.json")
}

pub fn release_required_contracts_mirror_path() -> PathBuf {
    contracts_root()
        .join("v1")
        .join("release_required_contracts.v1.json")
}

pub fn release_asset_matrix_path() -> PathBuf {
    contracts_root()
        .join("release")
        .join("v1.0")
        .join("release_asset_matrix.v1.json")
}

pub fn release_positioning_guard_path() -> PathBuf {
    contracts_root()
        .join("release")
        .join("v1.0")
        .join("release_positioning_guard.v1.json")
}

pub fn release_dependency_policy_path() -> PathBuf {
    contracts_root()
        .join("release")
        .join("v1.0")
        .join("release_dependency_policy.v1.json")
}

pub fn signing_trust_root_path() -> PathBuf {
    contracts_root()
        .join("security")
        .join("v1.0")
        .join("signing_trust_root.v1.json")
}

pub fn read_json(path: &Path) -> JsonValue {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    let rendered = serde_json::to_string_pretty(value).expect("render json");
    fs::write(path, rendered).unwrap_or_else(|_| panic!("write {}", path.display()));
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    let out = w100_target_root().join(rel_path);
    write_json_pretty(&out, report);
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    sha256_hex_bytes(&bytes)
}

pub fn canonicalize_json_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = JsonMap::new();
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
            for key in keys {
                let item = map.get(&key).expect("json key");
                sorted.insert(key, canonicalize_json_value(item));
            }
            JsonValue::Object(sorted)
        }
        JsonValue::Array(items) => JsonValue::Array(
            items
                .iter()
                .map(canonicalize_json_value)
                .collect::<Vec<JsonValue>>(),
        ),
        _ => value.clone(),
    }
}

pub fn canonical_json_string(value: &JsonValue) -> String {
    let canonical = canonicalize_json_value(value);
    serde_json::to_string(&canonical).expect("canonical json string")
}

pub fn producer_schema_hash(repo_root: &Path, producer: &str) -> String {
    let path = repo_root.join(producer);
    if !path.exists() {
        panic!("missing producer {}", path.display());
    }
    let value = read_json(&path);
    let canonical = canonical_json_string(&value);
    sha256_hex_bytes(canonical.as_bytes())
}

pub fn parse_release_required_contracts(path: &Path) -> RequiredContractsV100 {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

fn parse_trust_toml() -> (String, u64) {
    let trust_path = repo_root().join("trust.toml");
    if !trust_path.exists() {
        return ("unknown".to_string(), 0);
    }
    let raw = fs::read_to_string(&trust_path).expect("read trust.toml");
    let mut signer = "unknown".to_string();
    let mut epoch = 0u64;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("trust_epoch") {
            if let Some((_, rhs)) = trimmed.split_once('=') {
                epoch = rhs.trim().parse::<u64>().unwrap_or(0);
            }
            continue;
        }
        if trimmed.starts_with("keys") {
            if let Some((_, rhs)) = trimmed.split_once('=') {
                let cleaned = rhs
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim();
                let first = cleaned
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"');
                if !first.is_empty() {
                    signer = first.to_string();
                }
            }
        }
    }
    (signer, epoch)
}

fn command_version(command: &str, args: &[&str]) -> String {
    let out = match Command::new(command).args(args).output() {
        Ok(out) => out,
        Err(_) => return "unknown".to_string(),
    };
    if !out.status.success() {
        return "unknown".to_string();
    }
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if value.is_empty() {
        "unknown".to_string()
    } else {
        value
    }
}

fn parse_package_manager_version(editor_package_path: &Path, manager_name: &str) -> Option<String> {
    let value = read_json(editor_package_path);
    let package_manager = value.get("packageManager").and_then(JsonValue::as_str)?;
    let (name, version) = package_manager.split_once('@')?;
    if name != manager_name || version.trim().is_empty() {
        return None;
    }
    Some(version.trim().to_string())
}

pub fn ensure_run_manifest() -> JsonValue {
    let allowed = vec![
        "OCL_RELEASE_GRADE".to_string(),
        "OCL_STRICT_PROFILE".to_string(),
        "OCL_RELEASE_CHANNEL".to_string(),
        "OCL_QUARANTINE".to_string(),
        "OCL_UPDATE_V100_REQUIRED_CONTRACTS".to_string(),
        "OCL_UPDATE_V100_SOT_SIG".to_string(),
    ];
    let observed = std::env::vars()
        .map(|(key, _)| key)
        .filter(|key| key.starts_with("OCL_"))
        .collect::<Vec<String>>();
    if let Err(err) = v16::enforce_run_manifest_allowlist(&allowed, &observed) {
        panic!("{err}");
    }

    let mut base = v16::build_run_manifest(&repo_root(), allowed, observed);
    let root = base.as_object_mut().expect("run manifest object");
    let release_channel = std::env::var("OCL_RELEASE_CHANNEL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "github_release".to_string());
    let dependency_mode = read_json(&release_dependency_policy_path())
        .get("release_build_dependency_mode_default")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string();
    let signing = read_json(&signing_trust_root_path());
    let trust_root_id = signing
        .get("trust_root_public_keys")
        .and_then(JsonValue::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("key_id"))
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string();
    let (_, trust_epoch_from_toml) = parse_trust_toml();
    let trust_epoch = if trust_epoch_from_toml == 0 {
        signing
            .get("trust_epoch_policy")
            .and_then(JsonValue::as_object)
            .and_then(|obj| obj.get("current"))
            .and_then(JsonValue::as_u64)
            .unwrap_or(0)
    } else {
        trust_epoch_from_toml
    };
    let editor_package = repo_root()
        .join("editor")
        .join("vscode")
        .join("ocp-ocl")
        .join("package.json");
    let packaging_toolchain = repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_packaging_toolchain.v1.json");
    let node_version = {
        let shell = command_version("node", &["-v"]);
        if shell != "unknown" {
            shell
        } else {
            read_json(&packaging_toolchain)
                .get("node")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string()
        }
    };
    let pnpm_version = {
        let shell = command_version("pnpm", &["-v"]);
        if shell != "unknown" {
            shell
        } else {
            parse_package_manager_version(&editor_package, "pnpm").unwrap_or_else(|| {
                read_json(&packaging_toolchain)
                    .get("pnpm")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("unknown")
                    .to_string()
            })
        }
    };
    let vsce_version = {
        let shell = command_version("vsce", &["--version"]);
        if shell != "unknown" {
            shell
        } else {
            read_json(&packaging_toolchain)
                .get("vsce")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string()
        }
    };
    let ovsx_version = {
        let shell = command_version("ovsx", &["--version"]);
        if shell != "unknown" {
            shell
        } else {
            read_json(&packaging_toolchain)
                .get("ovsx")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string()
        }
    };

    root.insert(
        "release_channel".to_string(),
        JsonValue::String(release_channel),
    );
    root.insert(
        "release_version".to_string(),
        JsonValue::String("v1.0.0".to_string()),
    );
    root.insert(
        "required_contracts_hash".to_string(),
        JsonValue::String(sha256_hex_file(&release_required_contracts_path())),
    );
    root.insert(
        "dependency_mode".to_string(),
        JsonValue::String(dependency_mode),
    );
    root.insert(
        "signing_trust_root_id".to_string(),
        JsonValue::String(trust_root_id),
    );
    root.insert(
        "signing_trust_epoch".to_string(),
        JsonValue::from(trust_epoch),
    );
    root.insert("node_version".to_string(), JsonValue::String(node_version));
    root.insert("pnpm_version".to_string(), JsonValue::String(pnpm_version));
    root.insert("vsce_version".to_string(), JsonValue::String(vsce_version));
    root.insert("ovsx_version".to_string(), JsonValue::String(ovsx_version));

    if let Some(parent) = run_manifest_path().parent() {
        fs::create_dir_all(parent).expect("create w100 meta dir");
    }
    write_json_pretty(&run_manifest_path(), &base);
    base
}

pub fn run_manifest_sha256() -> String {
    let bytes = fs::read(run_manifest_path()).expect("read run manifest");
    sha256_hex_bytes(&bytes)
}
