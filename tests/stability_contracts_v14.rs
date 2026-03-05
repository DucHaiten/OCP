use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ocp_ocl::ocp_ocl::{pretty_schema, CapabilityRegistry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PackContractHashV14 {
    ctx_schema_hash: String,
    payload_schema_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StabilityContractSnapshotV14 {
    manifest_schema_version: String,
    trace_schema_version: u64,
    pack_contracts: BTreeMap<String, PackContractHashV14>,
}

fn snapshot_path() -> PathBuf {
    PathBuf::from("projects/ocp-ocl/conformance/expected/contracts/stability_contract_v14.json")
}

fn conformance_manifest_path() -> PathBuf {
    PathBuf::from("projects/ocp-ocl/conformance/conformance.v5.toml")
}

fn cli_main_path() -> PathBuf {
    PathBuf::from("projects/ocp-ocl/crates/ocl-cli/src/main.rs")
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn read_conformance_schema_version_v14(path: &Path) -> String {
    let raw = fs::read_to_string(path).expect("read conformance manifest");
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

fn read_trace_schema_version_v14(path: &Path) -> u64 {
    let raw = fs::read_to_string(path).expect("read cli main.rs");
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

fn current_snapshot_v14() -> StabilityContractSnapshotV14 {
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
        pack_contracts.insert(
            key,
            PackContractHashV14 {
                ctx_schema_hash: sha256_hex(&ctx_schema),
                payload_schema_hash: sha256_hex(&payload_schema),
            },
        );
    }
    StabilityContractSnapshotV14 {
        manifest_schema_version: read_conformance_schema_version_v14(&conformance_manifest_path()),
        trace_schema_version: read_trace_schema_version_v14(&cli_main_path()),
        pack_contracts,
    }
}

#[test]
fn stability_contract_snapshot_v14_additive_only() {
    let current = current_snapshot_v14();
    let path = snapshot_path();

    if std::env::var("OCL_UPDATE_V14_CONTRACT_SNAPSHOT")
        .ok()
        .as_deref()
        == Some("1")
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot dir");
        }
        let rendered = serde_json::to_string_pretty(&current).expect("render snapshot");
        fs::write(&path, rendered).expect("write snapshot");
        // Snapshot update is an explicit maintainer action; force rerun without update flag.
        panic!(
            "updated {}. Re-run test without OCL_UPDATE_V14_CONTRACT_SNAPSHOT=1",
            path.display()
        );
    }

    let baseline_raw = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing stability snapshot: {}", path.display()));
    let baseline: StabilityContractSnapshotV14 =
        serde_json::from_str(&baseline_raw).expect("parse baseline snapshot");

    assert_eq!(
        current.manifest_schema_version, baseline.manifest_schema_version,
        "manifest schema version changed; requires migration evidence"
    );
    assert_eq!(
        current.trace_schema_version, baseline.trace_schema_version,
        "trace schema version changed; requires migration evidence"
    );

    for (key, expected) in &baseline.pack_contracts {
        let actual = current.pack_contracts.get(key).unwrap_or_else(|| {
            panic!("non-additive removal detected: missing pack key `{key}`");
        });
        assert_eq!(
            actual.ctx_schema_hash, expected.ctx_schema_hash,
            "ctx schema hash changed for `{key}`; non-additive contract drift"
        );
        assert_eq!(
            actual.payload_schema_hash, expected.payload_schema_hash,
            "payload schema hash changed for `{key}`; non-additive contract drift"
        );
    }
}
