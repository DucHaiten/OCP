use std::fs;
use std::path::PathBuf;

use ocl_sdk::evaluate_downgrade_attempt_v17;
use serde_json::json;

#[test]
fn v17_downgrade_guard_enforces_lane_policy_and_writes_compat_report() {
    let strict = evaluate_downgrade_attempt_v17("locked_v071", 17, 16);
    assert!(!strict.allowed, "locked_v071 must block downgrade");
    assert!(strict.requires_audit_marker);
    assert_eq!(strict.reason_code, "RC-DOWNGRADE-BLOCKED-LOCKED");

    let compat = evaluate_downgrade_attempt_v17("locked_v06", 17, 16);
    assert!(compat.allowed, "locked_v06 may allow downgrade with audit");
    assert!(compat.requires_audit_marker);
    assert_eq!(compat.reason_code, "RC-DOWNGRADE-AUDIT");

    let quarantine = evaluate_downgrade_attempt_v17("quarantine", 17, 16);
    assert!(
        quarantine.allowed,
        "quarantine may allow downgrade with audit"
    );
    assert!(quarantine.requires_audit_marker);
    assert_eq!(quarantine.reason_code, "RC-DOWNGRADE-AUDIT");

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w17")
        .join("compat");
    fs::create_dir_all(&out_dir).expect("create w17 compat output dir");
    let report = json!({
        "schema": "ocl.w17.v016_line_compat_report.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "lane_literals": ["locked_v071", "locked_v06", "quarantine"],
        "downgrade_policy": {
            "locked_v071": {
                "allowed": strict.allowed,
                "requires_audit_marker": strict.requires_audit_marker,
                "reason_code": strict.reason_code
            },
            "locked_v06": {
                "allowed": compat.allowed,
                "requires_audit_marker": compat.requires_audit_marker,
                "reason_code": compat.reason_code
            },
            "quarantine": {
                "allowed": quarantine.allowed,
                "requires_audit_marker": quarantine.requires_audit_marker,
                "reason_code": quarantine.reason_code
            }
        },
        "contract_line": "v0.8..v0.16"
    });
    fs::write(
        out_dir.join("v016_line_compat_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize compat report"),
    )
    .expect("write v016_line_compat_report.json");
}

#[test]
fn v17_downgrade_guard_rejects_unknown_lane() {
    let decision = evaluate_downgrade_attempt_v17("locked", 17, 16);
    assert!(!decision.allowed);
    assert!(decision.requires_audit_marker);
    assert_eq!(decision.reason_code, "RC-LANE-UNSUPPORTED");
}
