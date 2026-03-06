use std::fs;

use serde_json::json;

#[path = "v18_gate_b_common.rs"]
mod common;

fn has_required_fields(report: &serde_json::Value) -> bool {
    [
        "no_op",
        "input_hash_before",
        "input_hash_after",
        "sot_ref",
        "run_manifest_ref",
    ]
    .iter()
    .all(|key| report.get(key).is_some())
}

#[test]
fn v18_migration_noop_proof_enforces_per_report_schema_lock() {
    let cassette = common::generate_cassette_upgrade_report();
    let manifest = common::generate_manifest_upgrade_report();
    let pack_abi = common::generate_pack_abi_upgrade_report();

    assert!(has_required_fields(&cassette));
    assert!(has_required_fields(&manifest));
    assert!(has_required_fields(&pack_abi));

    if manifest.get("no_op").and_then(serde_json::Value::as_bool) == Some(true) {
        assert_eq!(
            manifest.get("input_hash_before"),
            manifest.get("input_hash_after"),
            "manifest no-op proof must keep input hash identical"
        );
    }
    if pack_abi.get("no_op").and_then(serde_json::Value::as_bool) == Some(true) {
        assert_eq!(
            pack_abi.get("input_hash_before"),
            pack_abi.get("input_hash_after"),
            "pack abi no-op proof must keep input hash identical"
        );
    }

    let report = json!({
        "schema": "ocl.w18.migration.noop_proof_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "reports": [
            {
                "name": "cassette_upgrade_report.json",
                "no_op": cassette.get("no_op"),
                "required_fields_ok": has_required_fields(&cassette)
            },
            {
                "name": "manifest_upgrade_report.json",
                "no_op": manifest.get("no_op"),
                "required_fields_ok": has_required_fields(&manifest)
            },
            {
                "name": "pack_abi_upgrade_report.json",
                "no_op": pack_abi.get("no_op"),
                "required_fields_ok": has_required_fields(&pack_abi)
            }
        ],
        "all_required_fields_ok": true
    });
    let out_dir = common::w18_migration_dir();
    fs::create_dir_all(&out_dir).expect("create w18 migration dir");
    common::write_json_pretty(&out_dir.join("migration_noop_report.json"), &report);
}
