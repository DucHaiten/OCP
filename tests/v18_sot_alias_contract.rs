use std::fs;

use ocl_sdk::verify_sot_alias_contract_v18;
use serde_json::json;

#[path = "v18_gate_a_common.rs"]
mod v18;

#[test]
fn v18_sot_alias_contract() {
    v18::ensure_run_manifest();
    let summary = verify_sot_alias_contract_v18(&v18::repo_root()).expect("verify alias contract");
    assert!(
        summary
            .items
            .iter()
            .all(|item| !item.mirror_exists || item.bytes_identical),
        "mirror files must be byte-identical when present"
    );

    let report = json!({
        "schema": "ocl.w18.sot_alias_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "alias_count": summary.alias_count,
        "verified_count": summary.verified_count,
        "items": summary.items.iter().map(|item| json!({
            "canonical_path": item.canonical_path,
            "mirror_path": item.mirror_path,
            "mirror_exists": item.mirror_exists,
            "bytes_identical": item.bytes_identical
        })).collect::<Vec<_>>()
    });

    let out_dir = v18::w18_target_root().join("contracts");
    fs::create_dir_all(&out_dir).expect("create w18 contracts output dir");
    v18::write_json_pretty(&out_dir.join("sot_alias_report.json"), &report);
}
