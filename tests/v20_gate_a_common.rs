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
pub struct ContractInventoryEntryV20 {
    pub contract_id: String,
    pub version: String,
    pub producer: String,
    pub consumer: String,
    pub schema_hash: String,
    pub allowed_diffs_v020: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredContractsV20 {
    pub schema: String,
    pub contract_id: String,
    pub version: String,
    pub entries: Vec<ContractInventoryEntryV20>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalContractEntryV1 {
    pub allowed_diffs_v018: String,
    pub consumer: String,
    pub contract_id: String,
    pub producer: String,
    pub schema_hash: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalRequiredContractsV1 {
    pub schema: String,
    pub contract_id: String,
    pub version: String,
    pub entries: Vec<GlobalContractEntryV1>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w20_target_root() -> PathBuf {
    repo_root().join("target").join("ocp").join("w20")
}

pub fn run_manifest_path() -> PathBuf {
    w20_target_root().join("meta").join("run_manifest.json")
}

pub fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

pub fn required_contracts_v20_path() -> PathBuf {
    contracts_root()
        .join("v20")
        .join("required_contracts_v20.v1.json")
}

pub fn required_global_contracts_path() -> PathBuf {
    contracts_root().join("required_contracts.v1.json")
}

pub fn required_global_contracts_mirror_path() -> PathBuf {
    contracts_root()
        .join("v1")
        .join("required_contracts.v1.json")
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
    let path = w20_target_root().join(rel_path);
    write_json_pretty(&path, report);
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

pub fn parse_required_contracts_v20(path: &Path) -> RequiredContractsV20 {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn parse_required_global_contracts(path: &Path) -> GlobalRequiredContractsV1 {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
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

fn command_version_candidates(candidates: &[(&str, &[&str])]) -> String {
    for (cmd, args) in candidates {
        let value = command_version(cmd, args);
        if value != "unknown" {
            return value;
        }
    }
    "unknown".to_string()
}

fn read_version_rule(path: &Path, field: &str) -> Option<String> {
    let value = read_json(path);
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .map(|v| v.to_string())
}

fn read_required_tool_rule(path: &Path, tool_name: &str) -> Option<String> {
    let value = read_json(path);
    let required = value.get("required").and_then(JsonValue::as_array)?;
    required.iter().find_map(|row| {
        let tool = row.get("tool").and_then(JsonValue::as_str)?;
        if tool != tool_name {
            return None;
        }
        row.get("version_rule")
            .and_then(JsonValue::as_str)
            .map(|v| v.to_string())
    })
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

fn read_string_field(path: &Path, field: &str) -> String {
    read_json(path)
        .get(field)
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn parse_trust_signing_info() -> (String, u64) {
    let trust_path = repo_root().join("trust.toml");
    if !trust_path.exists() {
        return ("unknown".to_string(), 0);
    }
    let raw = fs::read_to_string(&trust_path).expect("read trust.toml");
    let mut trust_epoch = 0u64;
    let mut signer = "unknown".to_string();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("trust_epoch") {
            if let Some((_, rhs)) = trimmed.split_once('=') {
                trust_epoch = rhs.trim().parse::<u64>().unwrap_or(0);
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
    (signer, trust_epoch)
}

pub fn ensure_run_manifest() -> JsonValue {
    let allowed = vec![
        "OCP_RELEASE_GRADE".to_string(),
        "OCP_STRICT_PROFILE".to_string(),
        "OCP_EDITOR_PROFILE".to_string(),
        "OCP_QUARANTINE".to_string(),
        "OCP_V20_AUDIT".to_string(),
        "OCP_ALLOW_NETWORK_REPLAY".to_string(),
    ];
    let observed = std::env::vars()
        .map(|(key, _)| key)
        .filter(|key| key.starts_with("OCP_"))
        .collect::<Vec<String>>();
    if let Err(err) = v16::enforce_run_manifest_allowlist(&allowed, &observed) {
        panic!("{err}");
    }

    let repo = repo_root();
    let mut base = v16::build_run_manifest(&repo, allowed, observed);
    let obj = base.as_object_mut().expect("run manifest object");

    let contracts = contracts_root().join("v20");
    let test_catalog = contracts.join("test_catalog.v1.json");
    let threat_model = contracts.join("threat_model.v1.json");
    let fuzz_seed_suite = contracts.join("fuzz_seed_suite.v1.json");
    let tooling_versions = contracts.join("tooling_versions.v1.json");
    let history_replay_matrix = contracts.join("history_replay_matrix.v1.json");
    let history_toolchain_matrix = contracts.join("history_toolchain_matrix.v1.json");
    let required_contracts = contracts.join("required_contracts_v20.v1.json");
    let editor_packaging_toolchain = contracts_root()
        .join("editor")
        .join("editor_packaging_toolchain.v1.json");

    let dependency_mode = {
        let value = read_json(&history_replay_matrix);
        let mut seen = BTreeSet::<String>::new();
        for item in value
            .get("entries")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default()
        {
            if let Some(mode) = item.get("dependency_mode").and_then(JsonValue::as_str) {
                seen.insert(mode.to_string());
            }
        }
        if seen.len() == 1 {
            seen.into_iter()
                .next()
                .unwrap_or_else(|| "unknown".to_string())
        } else if seen.is_empty() {
            "unknown".to_string()
        } else {
            "mixed".to_string()
        }
    };

    let editor_package = repo
        .join("editor")
        .join("vscode")
        .join("ocp")
        .join("package.json");
    let editor_extension_version = if editor_package.exists() {
        read_json(&editor_package)
            .get("version")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string()
    } else {
        "unknown".to_string()
    };
    let pnpm_lock = repo
        .join("editor")
        .join("vscode")
        .join("ocp")
        .join("pnpm-lock.yaml");
    let pnpm_lock_hash = if pnpm_lock.exists() {
        sha256_hex_file(&pnpm_lock)
    } else {
        "missing".to_string()
    };

    let (signer_id, trust_epoch) = parse_trust_signing_info();
    let cli_version = v19_like_read_cargo_version(
        &repo
            .join("projects")
            .join("ocp")
            .join("crates")
            .join("ocp-cli")
            .join("Cargo.toml"),
    );

    let node_version = command_version("node", &["-v"]);
    let pnpm_version = {
        let shell =
            command_version_candidates(&[("pnpm", &["-v"]), ("corepack", &["pnpm", "--version"])]);
        if shell != "unknown" {
            shell
        } else {
            parse_package_manager_version(&editor_package, "pnpm")
                .or_else(|| read_required_tool_rule(&tooling_versions, "pnpm"))
                .unwrap_or_else(|| "unknown".to_string())
        }
    };
    let vsce_version = {
        let shell = command_version_candidates(&[
            ("vsce", &["--version"]),
            ("corepack", &["pnpm", "exec", "vsce", "--version"]),
        ]);
        if shell != "unknown" {
            shell
        } else {
            read_version_rule(&editor_packaging_toolchain, "vsce")
                .unwrap_or_else(|| "unknown".to_string())
        }
    };
    let ovsx_version = {
        let shell = command_version_candidates(&[
            ("ovsx", &["--version"]),
            ("corepack", &["pnpm", "exec", "ovsx", "--version"]),
        ]);
        if shell != "unknown" {
            shell
        } else {
            read_version_rule(&editor_packaging_toolchain, "ovsx")
                .unwrap_or_else(|| "unknown".to_string())
        }
    };

    obj.insert("node_version".to_string(), JsonValue::String(node_version));
    obj.insert("pnpm_version".to_string(), JsonValue::String(pnpm_version));
    obj.insert(
        "pnpm_lock_hash".to_string(),
        JsonValue::String(pnpm_lock_hash),
    );
    obj.insert("vsce_version".to_string(), JsonValue::String(vsce_version));
    obj.insert("ovsx_version".to_string(), JsonValue::String(ovsx_version));
    obj.insert(
        "required_contracts_hash".to_string(),
        JsonValue::String(sha256_hex_file(&required_contracts)),
    );
    obj.insert(
        "test_catalog_hash".to_string(),
        JsonValue::String(sha256_hex_file(&test_catalog)),
    );
    obj.insert(
        "fuzz_seed_suite_id".to_string(),
        JsonValue::String(read_string_field(&fuzz_seed_suite, "suite_id")),
    );
    obj.insert(
        "tooling_versions_id".to_string(),
        JsonValue::String(read_string_field(&tooling_versions, "tooling_id")),
    );
    obj.insert(
        "toolchain_matrix_hash".to_string(),
        JsonValue::String(sha256_hex_file(&history_toolchain_matrix)),
    );
    obj.insert(
        "threat_model_id".to_string(),
        JsonValue::String(read_string_field(&threat_model, "threat_model_id")),
    );
    obj.insert(
        "dependency_mode".to_string(),
        JsonValue::String(dependency_mode),
    );
    obj.insert(
        "editor_extension_version".to_string(),
        JsonValue::String(editor_extension_version),
    );
    obj.insert(
        "ocp_cli_version".to_string(),
        JsonValue::String(cli_version),
    );
    obj.insert(
        "lsp_server_version".to_string(),
        JsonValue::String("0.19.0-dev".to_string()),
    );
    obj.insert(
        "dap_server_version".to_string(),
        JsonValue::String("0.19.0-dev".to_string()),
    );
    obj.insert(
        "signing_trust_root_id".to_string(),
        JsonValue::String(signer_id),
    );
    obj.insert(
        "signing_trust_epoch".to_string(),
        JsonValue::from(trust_epoch),
    );

    if let Some(parent) = run_manifest_path().parent() {
        fs::create_dir_all(parent).expect("create w20 meta dir");
    }
    write_json_pretty(&run_manifest_path(), &base);
    base
}

fn v19_like_read_cargo_version(path: &Path) -> String {
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

pub fn run_manifest_sha256() -> String {
    let bytes = fs::read(run_manifest_path()).expect("read run manifest");
    sha256_hex_bytes(&bytes)
}

pub fn enforce_run_manifest_allowlist(
    allowed_env_flags: &[String],
    observed_env_flags: &[String],
) -> Result<(), String> {
    v16::enforce_run_manifest_allowlist(allowed_env_flags, observed_env_flags)
}

pub fn merge_v20_into_global(
    global: &GlobalRequiredContractsV1,
    v20_entries: &[ContractInventoryEntryV20],
) -> GlobalRequiredContractsV1 {
    let mut by_id = BTreeMap::<String, GlobalContractEntryV1>::new();
    for entry in &global.entries {
        by_id.insert(entry.contract_id.clone(), entry.clone());
    }
    for entry in v20_entries {
        by_id.insert(
            entry.contract_id.clone(),
            GlobalContractEntryV1 {
                allowed_diffs_v018: "none".to_string(),
                consumer: entry.consumer.clone(),
                contract_id: entry.contract_id.clone(),
                producer: entry.producer.clone(),
                schema_hash: entry.schema_hash.clone(),
                version: entry.version.clone(),
            },
        );
    }
    GlobalRequiredContractsV1 {
        schema: global.schema.clone(),
        contract_id: global.contract_id.clone(),
        version: global.version.clone(),
        entries: by_id.into_values().collect::<Vec<GlobalContractEntryV1>>(),
    }
}
