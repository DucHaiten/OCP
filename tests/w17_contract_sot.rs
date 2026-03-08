use std::fs;
use std::path::PathBuf;

use ocp_sdk::{verify_w17_contract_set_v17, W17_REQUIRED_CONTRACT_FILES};
use serde_json::json;

#[test]
fn v17_contract_sot_verify_all_required_contracts_and_write_report() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join("w17");
    let summary = verify_w17_contract_set_v17(&root).expect("verify w17 contract set");
    assert_eq!(summary.required_count, W17_REQUIRED_CONTRACT_FILES.len());
    assert_eq!(summary.verified_count, W17_REQUIRED_CONTRACT_FILES.len());
    assert_eq!(summary.items.len(), W17_REQUIRED_CONTRACT_FILES.len());
    assert!(
        summary.items.iter().all(|item| item.signature_verified),
        "all w17 contracts must have valid signatures"
    );

    let out_dir = PathBuf::from("target")
        .join("ocp")
        .join("w17")
        .join("contracts");
    fs::create_dir_all(&out_dir).expect("create w17 contracts output dir");
    let report = json!({
        "schema": "ocp.w17.contract_sot_report.v1",
        "run_manifest_ref": "target/ocp/w17/meta/run_manifest.json",
        "contracts_root": summary.root.to_string_lossy().replace('\\', "/"),
        "required_count": summary.required_count,
        "verified_count": summary.verified_count,
        "items": summary.items.iter().map(|item| json!({
            "contract_id": item.contract_id,
            "version": item.version,
            "contract_path": item.contract_path.to_string_lossy().replace('\\', "/"),
            "signature_path": item.signature_path.to_string_lossy().replace('\\', "/"),
            "contract_hash_sha256": item.contract_hash_sha256,
            "signature_verified": item.signature_verified,
            "signer_id": item.signer_id,
            "trust_epoch": item.trust_epoch
        })).collect::<Vec<_>>()
    });
    fs::write(
        out_dir.join("w17_contract_sot_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize w17 contract report"),
    )
    .expect("write w17_contract_sot_report.json");
}
