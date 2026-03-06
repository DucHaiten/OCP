use std::fs;

use ocl_sdk::evaluate_downgrade_attempt_v17;
use serde_json::json;

#[path = "v18_gate_b_common.rs"]
mod common;

#[test]
fn v18_downgrade_attempts_negative_obey_lane_policy() {
    common::ensure_run_manifest();

    let strict = evaluate_downgrade_attempt_v17("locked_v071", 18, 17);
    assert!(!strict.allowed, "locked_v071 must deny downgrade");
    assert!(strict.requires_audit_marker);
    assert_eq!(strict.reason_code, "RC-DOWNGRADE-BLOCKED-LOCKED");

    let compat = evaluate_downgrade_attempt_v17("locked_v06", 18, 17);
    assert!(compat.allowed, "locked_v06 may allow downgrade with audit");
    assert!(compat.requires_audit_marker);

    let unknown = evaluate_downgrade_attempt_v17("locked", 18, 17);
    assert!(!unknown.allowed);
    assert_eq!(unknown.reason_code, "RC-LANE-UNSUPPORTED");

    let report = json!({
        "schema": "ocl.w18.migration.downgrade_attempts_negative.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "strict_lane": {
            "allowed": strict.allowed,
            "requires_audit_marker": strict.requires_audit_marker,
            "reason_code": strict.reason_code
        },
        "compat_lane": {
            "allowed": compat.allowed,
            "requires_audit_marker": compat.requires_audit_marker,
            "reason_code": compat.reason_code
        },
        "unknown_lane": {
            "allowed": unknown.allowed,
            "requires_audit_marker": unknown.requires_audit_marker,
            "reason_code": unknown.reason_code
        }
    });

    let out_dir = common::w18_migration_dir();
    fs::create_dir_all(&out_dir).expect("create w18 migration dir");
    common::write_json_pretty(
        &out_dir.join("downgrade_attempts_negative_report.json"),
        &report,
    );
}
