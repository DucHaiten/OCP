#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v16_gate_a_common.rs"]
mod v16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractInventoryEntryV19 {
    pub allowed_diffs_v018: String,
    pub contract_id: String,
    pub version: String,
    pub producer: String,
    pub consumer: String,
    pub schema_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredContractsEditorV19 {
    pub schema: String,
    pub contract_id: String,
    pub version: String,
    pub entries: Vec<ContractInventoryEntryV19>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredContractsGlobalV1 {
    pub schema: String,
    pub contract_id: String,
    pub version: String,
    pub entries: Vec<ContractInventoryEntryV19>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w19_target_root() -> PathBuf {
    repo_root().join("target").join("ocl").join("w19")
}

pub fn run_manifest_path() -> PathBuf {
    w19_target_root().join("meta").join("run_manifest.json")
}

pub fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

pub fn editor_contracts_root() -> PathBuf {
    contracts_root().join("editor")
}

pub fn required_editor_contracts_path() -> PathBuf {
    editor_contracts_root().join("required_contracts_editor.v1.json")
}

pub fn required_global_contracts_path() -> PathBuf {
    contracts_root().join("required_contracts.v1.json")
}

pub fn required_global_contracts_mirror_path() -> PathBuf {
    contracts_root()
        .join("v1")
        .join("required_contracts.v1.json")
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

pub fn producer_schema_hash(repo_root: &Path, producer: &str) -> String {
    let path = repo_root.join(producer);
    if !path.exists() {
        panic!("missing producer {}", path.display());
    }
    let value = read_json(&path);
    let canonical = canonical_json_string(&value);
    sha256_hex_bytes(canonical.as_bytes())
}

fn command_version(command: &str, args: &[&str]) -> String {
    let out = match Command::new(command).args(args).output() {
        Ok(out) => out,
        Err(_) => return "unknown".to_string(),
    };
    if !out.status.success() {
        return "unknown".to_string();
    }
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn editor_extension_version() -> String {
    let package_path = repo_root()
        .join("editor")
        .join("vscode")
        .join("ocp-ocl")
        .join("package.json");
    if !package_path.exists() {
        return "unknown".to_string();
    }
    let package_json = read_json(&package_path);
    package_json
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn read_local_cargo_package_version(path: &Path) -> String {
    if !path.exists() {
        return "unknown".to_string();
    }
    let raw = fs::read_to_string(path).unwrap_or_default();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("version") {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let value = rhs.trim().trim_matches('"');
        if !value.is_empty() {
            return value.to_string();
        }
    }
    "unknown".to_string()
}

pub fn ensure_run_manifest() -> JsonValue {
    let allowed = vec![
        "OCL_RELEASE_GRADE".to_string(),
        "OCL_STRICT_PROFILE".to_string(),
        "OCL_EDITOR_PROFILE".to_string(),
        "OCL_EDITOR_TRUST_MODE".to_string(),
    ];
    let observed = std::env::vars()
        .map(|(key, _)| key)
        .filter(|key| key.starts_with("OCL_"))
        .collect::<Vec<String>>();
    if let Err(err) = v16::enforce_run_manifest_allowlist(&allowed, &observed) {
        panic!("{err}");
    }

    let repo = repo_root();
    let mut base = v16::build_run_manifest(&repo, allowed, observed);
    let obj = base.as_object_mut().expect("run manifest object");

    obj.insert(
        "node_version".to_string(),
        JsonValue::String(command_version("node", &["-v"])),
    );
    obj.insert(
        "pnpm_version".to_string(),
        JsonValue::String(command_version("pnpm", &["-v"])),
    );
    obj.insert(
        "pnpm_lock_hash".to_string(),
        JsonValue::String("missing".to_string()),
    );
    obj.insert(
        "vsce_version".to_string(),
        JsonValue::String(command_version("vsce", &["--version"])),
    );
    obj.insert(
        "ovsx_version".to_string(),
        JsonValue::String(command_version("ovsx", &["--version"])),
    );
    obj.insert(
        "signing_trust_root_id".to_string(),
        JsonValue::String("editor.editor_signing_trust_root".to_string()),
    );
    obj.insert("signing_trust_epoch".to_string(), JsonValue::from(1u64));
    obj.insert(
        "editor_extension_version".to_string(),
        JsonValue::String(editor_extension_version()),
    );
    obj.insert(
        "ocl_cli_version".to_string(),
        JsonValue::String(read_local_cargo_package_version(
            &repo
                .join("projects")
                .join("ocp-ocl")
                .join("crates")
                .join("ocl-cli")
                .join("Cargo.toml"),
        )),
    );
    obj.insert(
        "lsp_server_version".to_string(),
        JsonValue::String("0.19.0-dev".to_string()),
    );
    obj.insert(
        "dap_server_version".to_string(),
        JsonValue::String("0.19.0-dev".to_string()),
    );

    if let Some(parent) = run_manifest_path().parent() {
        fs::create_dir_all(parent).expect("create w19 meta dir");
    }
    write_json_pretty(&run_manifest_path(), &base);
    base
}

pub fn run_manifest_sha256() -> String {
    let bytes = fs::read(run_manifest_path()).expect("read run manifest");
    sha256_hex_bytes(&bytes)
}

pub fn editor_package_json() -> JsonValue {
    read_json(
        &repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp-ocl")
            .join("package.json"),
    )
}

pub fn sorted_strings_from_json_array(value: &JsonValue) -> Vec<String> {
    let mut out = value
        .as_array()
        .expect("json array")
        .iter()
        .map(|item| item.as_str().expect("array string").to_string())
        .collect::<Vec<String>>();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    out
}

pub fn sorted_strings_from_slice(slice: &[String]) -> Vec<String> {
    let mut out = slice.to_vec();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    out
}

pub fn parse_required_editor_contracts(path: &Path) -> RequiredContractsEditorV19 {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn parse_required_global_contracts(path: &Path) -> RequiredContractsGlobalV1 {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn merge_editor_entries_into_global(
    global: &RequiredContractsGlobalV1,
    editor_entries: &[ContractInventoryEntryV19],
) -> RequiredContractsGlobalV1 {
    let mut by_id = BTreeMap::<String, ContractInventoryEntryV19>::new();
    for entry in &global.entries {
        by_id.insert(entry.contract_id.clone(), entry.clone());
    }
    for entry in editor_entries {
        by_id.insert(entry.contract_id.clone(), entry.clone());
    }
    let entries = by_id
        .into_values()
        .collect::<Vec<ContractInventoryEntryV19>>();
    RequiredContractsGlobalV1 {
        schema: global.schema.clone(),
        contract_id: global.contract_id.clone(),
        version: global.version.clone(),
        entries,
    }
}

pub fn editor_contract_ids_set(required: &RequiredContractsEditorV19) -> BTreeSet<String> {
    required
        .entries
        .iter()
        .map(|item| item.contract_id.clone())
        .collect::<BTreeSet<String>>()
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    let path = w19_target_root().join(rel_path);
    write_json_pretty(&path, report);
}
