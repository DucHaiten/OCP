use std::fs;

use serde_json::json;

#[path = "v18_gate_a_common.rs"]
mod v18;

#[test]
fn v18_platform_profile_contract() {
    v18::ensure_run_manifest();
    let path = v18::contracts_root()
        .join("platform")
        .join("supported_profile.v1.json");
    let value = v18::read_json(&path);

    assert_eq!(
        value
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "v1.supported_profile"
    );
    let profiles = value
        .get("supported_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("supported_profiles array");
    assert!(
        profiles.iter().any(|item| {
            item.get("id").and_then(serde_json::Value::as_str) == Some("linux-x64-ext4")
        }),
        "linux profile must exist"
    );
    assert!(
        profiles.iter().any(|item| {
            item.get("id").and_then(serde_json::Value::as_str) == Some("win-x64-ntfs")
        }),
        "windows profile must exist"
    );
    assert_eq!(
        value
            .get("canonicalization")
            .and_then(serde_json::Value::as_object)
            .and_then(|map| map.get("newline"))
            .and_then(serde_json::Value::as_str),
        Some("LF")
    );

    let report = json!({
        "schema": "ocl.w18.platform_profile_contract_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "contract_path": "contracts/platform/supported_profile.v1.json",
        "status": "PASS",
        "profile_count": profiles.len()
    });
    let out_dir = v18::w18_target_root().join("contracts");
    fs::create_dir_all(&out_dir).expect("create w18 contracts output dir");
    v18::write_json_pretty(
        &out_dir.join("platform_profile_contract_report.json"),
        &report,
    );
}
