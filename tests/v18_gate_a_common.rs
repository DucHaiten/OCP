#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v16_gate_a_common.rs"]
mod v16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractInventoryEntryV18 {
    pub contract_id: String,
    pub version: String,
    pub schema_hash: String,
    pub producer: String,
    pub consumer: String,
    pub allowed_diffs_v018: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredContractsV1 {
    pub schema: String,
    pub version: Option<String>,
    pub entries: Vec<ContractInventoryEntryV18>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_target_root() -> PathBuf {
    repo_root().join("target").join("ocp").join("w18")
}

pub fn run_manifest_path() -> PathBuf {
    w18_target_root().join("meta").join("run_manifest.json")
}

pub fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

pub fn required_contracts_path() -> PathBuf {
    contracts_root().join("required_contracts.v1.json")
}

pub fn required_contracts_mirror_path() -> PathBuf {
    contracts_root()
        .join("v1")
        .join("required_contracts.v1.json")
}

pub fn ensure_run_manifest() -> JsonValue {
    let allowed = vec![
        "OCP_RELEASE_GRADE".to_string(),
        "OCP_QUARANTINE".to_string(),
        "OCP_STRICT_PROFILE".to_string(),
        "OCP_UPDATE_V18_SOT_SIG".to_string(),
        "OCP_UPDATE_V18_REQUIRED_CONTRACTS".to_string(),
    ];
    let observed = std::env::vars()
        .map(|(key, _)| key)
        .filter(|key| key.starts_with("OCP_"))
        .collect::<Vec<String>>();
    if let Err(err) = v16::enforce_run_manifest_allowlist(&allowed, &observed) {
        panic!("{err}");
    }

    let manifest = v16::build_run_manifest(&repo_root(), allowed, observed);
    if let Some(parent) = run_manifest_path().parent() {
        fs::create_dir_all(parent).expect("create w18 meta dir");
    }
    write_json_pretty(&run_manifest_path(), &manifest);
    manifest
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

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
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
    if producer.ends_with(".json") {
        let value = read_json(&path);
        let canonical = canonical_json_string(&value);
        return sha256_hex_bytes(canonical.as_bytes());
    }
    let bytes = fs::read(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    sha256_hex_bytes(&bytes)
}
