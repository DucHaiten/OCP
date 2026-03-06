use std::fs;
use std::path::PathBuf;

use ocl_sdk::{evaluate_trust_lifecycle_v17, TrustKeyRecordV17};
use serde_json::json;

#[test]
fn v17_trust_lifecycle_selects_latest_active_key_and_writes_report() {
    let summary = evaluate_trust_lifecycle_v17(
        7,
        &[
            TrustKeyRecordV17 {
                key_id: "k-epoch-1".to_string(),
                trust_epoch: 1,
                revoked: false,
            },
            TrustKeyRecordV17 {
                key_id: "k-epoch-5".to_string(),
                trust_epoch: 5,
                revoked: false,
            },
            TrustKeyRecordV17 {
                key_id: "k-epoch-6-revoked".to_string(),
                trust_epoch: 6,
                revoked: true,
            },
        ],
    )
    .expect("trust lifecycle summary");

    assert_eq!(summary.current_epoch, 7);
    assert_eq!(summary.active_key_id, "k-epoch-5");
    assert_eq!(summary.active_key_epoch, 5);
    assert_eq!(
        summary.revoked_key_ids,
        vec!["k-epoch-6-revoked".to_string()]
    );

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w17")
        .join("contracts");
    fs::create_dir_all(&out_dir).expect("create w17 contracts output dir");
    let report = json!({
        "schema": "ocl.w17.trust_lifecycle_report.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "current_epoch": summary.current_epoch,
        "active_key_id": summary.active_key_id,
        "active_key_epoch": summary.active_key_epoch,
        "revoked_key_ids": summary.revoked_key_ids,
        "known_key_ids": summary.known_key_ids
    });
    fs::write(
        out_dir.join("trust_lifecycle_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize trust lifecycle report"),
    )
    .expect("write trust_lifecycle_report.json");
}

#[test]
fn v17_trust_lifecycle_rejects_duplicate_key_id() {
    let err = evaluate_trust_lifecycle_v17(
        3,
        &[
            TrustKeyRecordV17 {
                key_id: "dup".to_string(),
                trust_epoch: 1,
                revoked: false,
            },
            TrustKeyRecordV17 {
                key_id: "dup".to_string(),
                trust_epoch: 2,
                revoked: false,
            },
        ],
    )
    .expect_err("duplicate key ids must be rejected");
    assert!(
        err.to_string().contains("X-TRUST-LIFECYCLE-DUPLICATE"),
        "unexpected error: {err}"
    );
}
