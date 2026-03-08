#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ocp::ocp::{pretty_schema, Cacheability, CapabilityRegistry, DeterminismClass};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackContractHashV15 {
    pub ctx_schema_hash: String,
    pub payload_schema_hash: String,
    pub determinism_class: String,
    pub cacheability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StabilityContractSnapshotV15 {
    pub manifest_schema_version: String,
    pub trace_schema_version: u64,
    pub lane_literals: Vec<String>,
    pub lockfile_sot: String,
    pub pack_contracts: BTreeMap<String, PackContractHashV15>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractInventoryEntryV16 {
    pub contract_id: String,
    pub version: String,
    pub schema_hash: String,
    pub producer: String,
    pub consumer: String,
    pub allowed_diffs_v016: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceFileV1 {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEvidenceEntryV1 {
    pub version_id: String,
    pub commit_or_tag: String,
    pub schema_version: String,
    pub evidence_files: Vec<EvidenceFileV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEvidenceIndexV1 {
    pub schema: String,
    pub hasher: String,
    pub entries: Vec<HistoryEvidenceEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEvidenceSignatureV1 {
    pub schema: String,
    pub hasher: String,
    pub signature: String,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w16_target_root() -> PathBuf {
    repo_root().join("target").join("ocp").join("w16")
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_hex_text(text: &str) -> String {
    sha256_hex_bytes(text.as_bytes())
}

pub fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read file {}", path.display()));
    sha256_hex_bytes(&bytes)
}

pub fn canonicalize_json_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = JsonMap::new();
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let item = map.get(&key).expect("object key");
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
    serde_json::to_string(&canonical).expect("render canonical json")
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    let rendered = serde_json::to_string_pretty(value).expect("render json pretty");
    fs::write(path, rendered).unwrap_or_else(|_| panic!("write file {}", path.display()));
}

pub fn read_conformance_schema_version_v15() -> String {
    let path = repo_root()
        .join("projects")
        .join("ocp")
        .join("conformance")
        .join("conformance.v5.toml");
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("schema") {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let value = rhs.trim().trim_matches('"').to_string();
        if !value.is_empty() {
            return value;
        }
    }
    panic!("missing schema in {}", path.display());
}

pub fn read_trace_schema_version_v15() -> u64 {
    let path = repo_root()
        .join("projects")
        .join("ocp")
        .join("crates")
        .join("ocp-cli")
        .join("src")
        .join("main.rs");
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("const TRACE_SCHEMA_VERSION_V11: u64 =") {
            continue;
        }
        let value = trimmed
            .trim_start_matches("const TRACE_SCHEMA_VERSION_V11: u64 =")
            .trim()
            .trim_end_matches(';')
            .trim();
        return value.parse::<u64>().expect("parse trace schema version");
    }
    panic!("missing TRACE_SCHEMA_VERSION_V11 in {}", path.display());
}

fn determinism_class_name(value: DeterminismClass) -> &'static str {
    match value {
        DeterminismClass::Deterministic => "Deterministic",
        DeterminismClass::CassetteBased => "CassetteBased",
        DeterminismClass::NonDeterministic => "NonDeterministicForbidden",
    }
}

fn cacheability_name(value: Cacheability) -> &'static str {
    match value {
        Cacheability::NoCache => "NoCache",
        Cacheability::WithinRun => "WithinRun",
        Cacheability::AcrossRuns => "AcrossRuns",
    }
}

pub fn current_stability_snapshot_v15() -> StabilityContractSnapshotV15 {
    let registry = CapabilityRegistry::v1_baseline();
    let mut pack_contracts = BTreeMap::new();
    for key in registry.documented_keys() {
        let ctx_schema = registry
            .ctx_schema_for_key(&key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let payload_schema = registry
            .payload_schema_for_key(&key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let determinism_class = determinism_class_name(registry.determinism_class_for_key(&key));
        let cacheability = cacheability_name(registry.cacheability_for_key(&key));
        pack_contracts.insert(
            key,
            PackContractHashV15 {
                ctx_schema_hash: sha256_hex_text(&ctx_schema),
                payload_schema_hash: sha256_hex_text(&payload_schema),
                determinism_class: determinism_class.to_string(),
                cacheability: cacheability.to_string(),
            },
        );
    }

    StabilityContractSnapshotV15 {
        manifest_schema_version: read_conformance_schema_version_v15(),
        trace_schema_version: read_trace_schema_version_v15(),
        lane_literals: vec![
            "locked_v06".to_string(),
            "locked_v071".to_string(),
            "quarantine".to_string(),
        ],
        lockfile_sot: "deps.lock.v3".to_string(),
        pack_contracts,
    }
}

pub fn current_contract_inventory_v16() -> Vec<ContractInventoryEntryV16> {
    let snapshot = current_stability_snapshot_v15();
    let snapshot_value = serde_json::to_value(&snapshot).expect("snapshot to json");
    let snapshot_hash = sha256_hex_text(&canonical_json_string(&snapshot_value));

    let history_index_path = repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.json");
    let history_sig_path = repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.sig");
    let history_index_hash = if history_index_path.exists() {
        sha256_hex_file(&history_index_path)
    } else {
        "missing".to_string()
    };
    let history_sig_hash = if history_sig_path.exists() {
        sha256_hex_file(&history_sig_path)
    } else {
        "missing".to_string()
    };

    let mut out = vec![
        ContractInventoryEntryV16 {
            contract_id: "manifest.schema".to_string(),
            version: "v0.15-line".to_string(),
            schema_hash: sha256_hex_text(&snapshot.manifest_schema_version),
            producer: "ocp-sdk:projects/ocp/crates/ocp-sdk/src/lib.rs".to_string(),
            consumer: "ocp-cli+ocp-runtime-core".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "trace.schema".to_string(),
            version: "v0.11-line".to_string(),
            schema_hash: sha256_hex_text(&snapshot.trace_schema_version.to_string()),
            producer: "ocp-cli:projects/ocp/crates/ocp-cli/src/main.rs".to_string(),
            consumer: "ocp-cli+ocp-sdk".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "lane.literals".to_string(),
            version: "v0.8+".to_string(),
            schema_hash: sha256_hex_text(&snapshot.lane_literals.join("|")),
            producer: "ocp-sdk+ocp-cli".to_string(),
            consumer: "ocp-runtime-core+ocp-cli".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "lockfile.sot".to_string(),
            version: "v0.15-line".to_string(),
            schema_hash: sha256_hex_text(&snapshot.lockfile_sot),
            producer: "ocp-sdk".to_string(),
            consumer: "ocp-cli".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "registry.pack_contracts".to_string(),
            version: "v0.15-line".to_string(),
            schema_hash: snapshot_hash,
            producer: "ocp::CapabilityRegistry".to_string(),
            consumer: "ocp-runtime-core+ocp-sdk+ocp-cli".to_string(),
            allowed_diffs_v016: "additive-only".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "history.evidence_index".to_string(),
            version: "v1".to_string(),
            schema_hash: history_index_hash,
            producer: "contracts/history/evidence_index.v1.json".to_string(),
            consumer: "tests/historical_evidence.rs".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
        ContractInventoryEntryV16 {
            contract_id: "history.evidence_index_signature".to_string(),
            version: "v1".to_string(),
            schema_hash: history_sig_hash,
            producer: "contracts/history/evidence_index.v1.sig".to_string(),
            consumer: "tests/historical_evidence_signature.rs".to_string(),
            allowed_diffs_v016: "none".to_string(),
        },
    ];
    out.sort_by(|a, b| a.contract_id.cmp(&b.contract_id));
    out
}

pub fn derive_lane_profile(project_root: &Path) -> String {
    let path = project_root.join("Ocp.toml");
    if !path.exists() {
        return "locked_v071".to_string();
    }
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("lane") {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let value = rhs.trim().trim_matches('"').to_string();
        if !value.is_empty() {
            return value;
        }
    }
    "locked_v071".to_string()
}

fn rustc_version_verbose() -> String {
    let out = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("run rustc -Vv");
    if !out.status.success() {
        return "unknown".to_string();
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .replace("\r\n", "\n")
}

fn cargo_version() -> String {
    let out = Command::new("cargo")
        .arg("-V")
        .output()
        .expect("run cargo -V");
    if !out.status.success() {
        return "unknown".to_string();
    }
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_commit(root: &Path) -> String {
    let mut cur = Some(root.to_path_buf());
    while let Some(path) = cur {
        let dot_git = path.join(".git");
        if dot_git.exists() {
            let head_path = dot_git.join("HEAD");
            let head_raw = fs::read_to_string(&head_path)
                .unwrap_or_else(|_| panic!("read {}", head_path.display()));
            let trimmed = head_raw.trim();
            if let Some(reference) = trimmed.strip_prefix("ref: ") {
                let ref_path = dot_git.join(reference.replace('/', std::path::MAIN_SEPARATOR_STR));
                if ref_path.exists() {
                    let commit = fs::read_to_string(&ref_path)
                        .unwrap_or_else(|_| panic!("read {}", ref_path.display()));
                    return commit.trim().to_string();
                }
                return "unknown".to_string();
            }
            return trimmed.to_string();
        }
        cur = path.parent().map(|v| v.to_path_buf());
    }
    "unknown".to_string()
}

pub fn build_run_manifest(
    project_root: &Path,
    allowed_env_flags: Vec<String>,
    observed_env_flags: Vec<String>,
) -> JsonValue {
    let repo = repo_root();
    let deps_lock_v3 = project_root.join("deps.lock.v3");
    let deps_lock_v3_hash = if deps_lock_v3.exists() {
        sha256_hex_file(&deps_lock_v3)
    } else {
        "missing".to_string()
    };
    let cargo_lock = repo.join("Cargo.lock");
    let cargo_lock_hash = if cargo_lock.exists() {
        Some(sha256_hex_file(&cargo_lock))
    } else {
        None
    };
    let lane_profile = derive_lane_profile(project_root);
    let target_triple = std::env::var("TARGET")
        .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));

    let mut root = JsonMap::new();
    root.insert(
        "git_commit".to_string(),
        JsonValue::String(git_commit(&repo)),
    );
    root.insert(
        "rustc_version_verbose".to_string(),
        JsonValue::String(rustc_version_verbose()),
    );
    root.insert(
        "cargo_version".to_string(),
        JsonValue::String(cargo_version()),
    );
    root.insert(
        "target_triple".to_string(),
        JsonValue::String(target_triple),
    );
    root.insert(
        "os".to_string(),
        JsonValue::String(std::env::consts::OS.to_string()),
    );
    root.insert(
        "arch".to_string(),
        JsonValue::String(std::env::consts::ARCH.to_string()),
    );
    root.insert("lane_profile".to_string(), JsonValue::String(lane_profile));
    root.insert(
        "allowed_env_flags".to_string(),
        JsonValue::Array(
            allowed_env_flags
                .iter()
                .map(|v| JsonValue::String(v.clone()))
                .collect::<Vec<JsonValue>>(),
        ),
    );
    root.insert(
        "observed_env_flags".to_string(),
        JsonValue::Array(
            observed_env_flags
                .iter()
                .map(|v| JsonValue::String(v.clone()))
                .collect::<Vec<JsonValue>>(),
        ),
    );
    root.insert(
        "deps_lock_v3_hash".to_string(),
        JsonValue::String(deps_lock_v3_hash),
    );
    root.insert(
        "cargo_lock_hash".to_string(),
        cargo_lock_hash
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    );
    JsonValue::Object(root)
}

pub fn enforce_run_manifest_allowlist(
    allowed_env_flags: &[String],
    observed_env_flags: &[String],
) -> Result<(), String> {
    let allowed = allowed_env_flags
        .iter()
        .map(|v| v.trim().to_string())
        .collect::<BTreeSet<String>>();
    let mut unknown = observed_env_flags
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && !allowed.contains(v))
        .collect::<Vec<String>>();
    unknown.sort();
    unknown.dedup();
    if unknown.is_empty() {
        return Ok(());
    }
    Err(format!(
        "run_manifest deny-by-default: unknown env flags detected: {}",
        unknown.join(", ")
    ))
}

pub fn compute_history_signature(index: &HistoryEvidenceIndexV1) -> String {
    let value = serde_json::to_value(index).expect("index to value");
    let canonical = canonical_json_string(&value);
    sha256_hex_text(&format!("ocp-history-evidence-v1|{canonical}"))
}
