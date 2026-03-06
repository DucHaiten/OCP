use ocl_sdk::verify_contract_json_signature_v20;
use serde_json::json;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_repro_protocol_contract() {
    v20g::ensure_run_manifest();
    let contract_path = v20g::repo_root()
        .join("contracts")
        .join("v20")
        .join("repro_protocol.v1.json");
    let summary = verify_contract_json_signature_v20(&v20g::repo_root(), &contract_path)
        .expect("verify repro_protocol signature");
    assert!(
        summary.signature_verified,
        "repro protocol signature must verify"
    );

    let protocol = v20g::repro_protocol();
    let clean_build_runs = protocol
        .get("clean_build_runs")
        .and_then(serde_json::Value::as_u64)
        .expect("clean_build_runs");
    let manifest_byte_equal = protocol
        .get("compare")
        .and_then(|value| value.get("manifest_byte_equal"))
        .and_then(serde_json::Value::as_bool)
        .expect("compare.manifest_byte_equal");
    let artifact_sha256_equal = protocol
        .get("compare")
        .and_then(|value| value.get("artifact_sha256_equal"))
        .and_then(serde_json::Value::as_bool)
        .expect("compare.artifact_sha256_equal");

    assert_eq!(
        clean_build_runs, 2,
        "clean_build_runs must be 2 for Gate 20-G"
    );
    assert!(manifest_byte_equal, "manifest_byte_equal must be true");
    assert!(artifact_sha256_equal, "artifact_sha256_equal must be true");

    let report = json!({
        "schema": "ocl.w20.release.repro_protocol_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "signature_path": summary.signature_path.to_string_lossy().replace('\\', "/"),
        "clean_build_runs": clean_build_runs,
        "compare": {
            "manifest_byte_equal": manifest_byte_equal,
            "artifact_sha256_equal": artifact_sha256_equal
        },
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20g::run_manifest_sha256()
    });
    v20g::write_report("release/repro_protocol_report.json", &report);
}
