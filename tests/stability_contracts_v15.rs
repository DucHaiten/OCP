use std::fs;
use std::path::PathBuf;

use serde_json::json;

#[path = "v16_gate_a_common.rs"]
mod v16;

fn snapshot_path() -> PathBuf {
    PathBuf::from("projects/ocp/conformance/expected/contracts/stability_contract_v15.json")
}

fn run_manifest_path() -> PathBuf {
    v16::w16_target_root()
        .join("meta")
        .join("run_manifest.json")
}

fn contract_dir() -> PathBuf {
    v16::w16_target_root().join("contracts")
}

#[test]
fn stability_contract_snapshot_v15_additive_only() {
    let current = v16::current_stability_snapshot_v15();
    let path = snapshot_path();

    if std::env::var("OCP_UPDATE_V15_CONTRACT_SNAPSHOT")
        .ok()
        .as_deref()
        == Some("1")
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot dir");
        }
        let rendered = serde_json::to_string_pretty(&current).expect("render snapshot");
        fs::write(&path, rendered).expect("write snapshot");
        panic!(
            "updated {}. Re-run test without OCP_UPDATE_V15_CONTRACT_SNAPSHOT=1",
            path.display()
        );
    }

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot dir");
        }
        let rendered = serde_json::to_string_pretty(&current).expect("render snapshot");
        fs::write(&path, rendered).expect("write initial snapshot");
        panic!(
            "initialized missing snapshot {}. Re-run test",
            path.display()
        );
    }

    let baseline_raw = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read stability snapshot: {}", path.display()));
    let baseline: v16::StabilityContractSnapshotV15 =
        serde_json::from_str(&baseline_raw).expect("parse baseline snapshot");

    assert_eq!(
        current.manifest_schema_version, baseline.manifest_schema_version,
        "manifest schema version changed; requires migration evidence"
    );
    assert_eq!(
        current.trace_schema_version, baseline.trace_schema_version,
        "trace schema version changed; requires migration evidence"
    );
    assert_eq!(
        current.lane_literals, baseline.lane_literals,
        "lane literals changed; requires migration evidence"
    );
    assert_eq!(
        current.lockfile_sot, baseline.lockfile_sot,
        "lockfile SoT changed; requires migration evidence"
    );

    for (key, expected) in &baseline.pack_contracts {
        let actual = current.pack_contracts.get(key).unwrap_or_else(|| {
            panic!("non-additive removal detected: missing pack key `{key}`");
        });
        assert_eq!(
            actual.ctx_schema_hash, expected.ctx_schema_hash,
            "ctx schema hash changed for `{key}`; non-additive drift"
        );
        assert_eq!(
            actual.payload_schema_hash, expected.payload_schema_hash,
            "payload schema hash changed for `{key}`; non-additive drift"
        );
        assert_eq!(
            actual.determinism_class, expected.determinism_class,
            "determinism class changed for `{key}`; non-additive drift"
        );
        assert_eq!(
            actual.cacheability, expected.cacheability,
            "cacheability changed for `{key}`; non-additive drift"
        );
    }

    let allowed_env_flags = vec![
        "OCP_QUARANTINE".to_string(),
        "OCP_ALLOW_OVERRIDES".to_string(),
        "OCP_TRUST_MODE".to_string(),
    ];
    let observed_env_flags = Vec::<String>::new();
    let run_manifest =
        v16::build_run_manifest(&v16::repo_root(), allowed_env_flags, observed_env_flags);
    v16::write_json_pretty(&run_manifest_path(), &run_manifest);

    let current_value = serde_json::to_value(&current).expect("snapshot value");
    let language_snapshot = json!({
        "manifest_schema_version": current.manifest_schema_version,
        "lane_literals": current.lane_literals,
    });
    let runtime_snapshot = json!({
        "lockfile_sot": current.lockfile_sot,
        "pack_contracts_count": current.pack_contracts.len(),
        "pack_contracts_hash": v16::sha256_hex_text(&v16::canonical_json_string(&json!(current.pack_contracts))),
    });
    let trace_snapshot = json!({
        "trace_schema_version": current.trace_schema_version,
    });
    let supplychain_snapshot = json!({
        "lockfile_sot": current.lockfile_sot,
    });
    let report = json!({
        "schema": "ocp.contract.freeze.report.v16",
        "status": "PASS",
        "baseline_path": path.to_string_lossy(),
        "snapshot_hash_sha256": v16::sha256_hex_text(&v16::canonical_json_string(&current_value)),
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
    });

    let contract_dir = contract_dir();
    v16::write_json_pretty(
        &contract_dir.join("language_contract_snapshot.json"),
        &language_snapshot,
    );
    v16::write_json_pretty(
        &contract_dir.join("runtime_contract_snapshot.json"),
        &runtime_snapshot,
    );
    v16::write_json_pretty(
        &contract_dir.join("trace_contract_snapshot.json"),
        &trace_snapshot,
    );
    v16::write_json_pretty(
        &contract_dir.join("supplychain_contract_snapshot.json"),
        &supplychain_snapshot,
    );
    v16::write_json_pretty(&contract_dir.join("contract_freeze_report.json"), &report);
}
