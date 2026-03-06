use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ocl_sdk::verify_contract_json_signature_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

fn resolve_repo_relative(path: &str) -> std::path::PathBuf {
    f19::repo_root().join(Path::new(path))
}

#[test]
fn v19_binary_bootstrap_verify_policy_enforced() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let policy_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("ocl_binary_bootstrap_policy.v1.json");
    let policy = f19::read_json(&policy_path);
    assert_eq!(
        policy
            .get("verify_before_execute")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "verify_before_execute must be true"
    );

    let summary = verify_contract_json_signature_v19(&f19::repo_root(), &fixture.manifest_path)
        .expect("manifest signature verify");
    assert!(summary.signature_verified);

    let manifest = f19::read_json(&fixture.manifest_path);
    let binaries = manifest
        .get("binaries")
        .and_then(serde_json::Value::as_array)
        .expect("binaries");
    let mut checked = BTreeMap::<String, bool>::new();
    for item in binaries {
        let name = item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .expect("name");
        let rel = item
            .get("path")
            .and_then(serde_json::Value::as_str)
            .expect("path");
        let expected = item
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .expect("sha");
        let bytes = fs::read(resolve_repo_relative(rel)).expect("read binary");
        let actual = f19::sha256_hex_bytes(&bytes);
        checked.insert(name.to_string(), actual == expected);
    }
    assert!(
        checked.values().all(|ok| *ok),
        "all binaries must pass hash verification before execute"
    );

    let report = json!({
        "schema": "ocl.w19.release.binary_bootstrap_report.v1",
        "status": "PASS",
        "phase": "verify_before_execute",
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "checked": checked,
        "signature_verified": summary.signature_verified,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/binary_bootstrap_report.json", &report);
}
