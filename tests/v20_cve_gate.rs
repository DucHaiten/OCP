use ocp_sdk::{evaluate_adapter_cve_policy_v17, AdapterCveSeverityV17};
use serde_json::json;

#[path = "v20_gate_f_common.rs"]
mod v20f;

#[test]
fn v20_cve_gate() {
    v20f::ensure_run_manifest();
    let snapshot = v20f::cve_snapshot();
    let source_mode = snapshot
        .get("policy")
        .and_then(|policy| policy.get("source_mode"))
        .and_then(serde_json::Value::as_str)
        .expect("cve_snapshot.policy.source_mode");
    assert_eq!(
        source_mode, "offline_pinned_snapshot",
        "Gate 20-F must use pinned offline CVE snapshot"
    );

    let deny_severities = v20f::string_array_at_path(&snapshot, &["policy", "deny_severities"]);
    assert!(
        deny_severities.iter().any(|item| item == "critical"),
        "deny_severities missing `critical`"
    );
    assert!(
        deny_severities.iter().any(|item| item == "high"),
        "deny_severities missing `high`"
    );

    let strict_critical = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Critical,
        false,
        true,
        true,
    );
    let strict_high = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::High,
        false,
        true,
        true,
    );
    let strict_medium = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Medium,
        false,
        true,
        true,
    );
    let quarantine_high = evaluate_adapter_cve_policy_v17(
        "quarantine",
        AdapterCveSeverityV17::High,
        false,
        true,
        true,
    );

    assert!(
        !strict_critical.allowed,
        "critical CVE must be blocked in strict lane"
    );
    assert!(
        !strict_high.allowed,
        "high CVE must be blocked in strict lane"
    );
    assert!(
        strict_medium.reason_code == "RC-CVE-PATCH-REQUIRED"
            || strict_medium.reason_code == "RC-CVE-STRICT-AUDIT",
        "strict medium severity must follow explicit policy reason code"
    );
    assert!(
        quarantine_high.allowed && quarantine_high.requires_audit_marker,
        "quarantine lane must require audit marker for high severity"
    );

    let report = json!({
        "schema": "ocp.w20.security.cve_gate_report.v1",
        "status": "PASS",
        "snapshot_id": snapshot.get("snapshot_id").and_then(serde_json::Value::as_str).unwrap_or("unknown"),
        "source_mode": source_mode,
        "deny_severities": deny_severities,
        "decisions": {
            "locked_v071_critical": {
                "allowed": strict_critical.allowed,
                "requires_audit_marker": strict_critical.requires_audit_marker,
                "reason_code": strict_critical.reason_code
            },
            "locked_v071_high": {
                "allowed": strict_high.allowed,
                "requires_audit_marker": strict_high.requires_audit_marker,
                "reason_code": strict_high.reason_code
            },
            "locked_v071_medium": {
                "allowed": strict_medium.allowed,
                "requires_audit_marker": strict_medium.requires_audit_marker,
                "reason_code": strict_medium.reason_code
            },
            "quarantine_high": {
                "allowed": quarantine_high.allowed,
                "requires_audit_marker": quarantine_high.requires_audit_marker,
                "reason_code": quarantine_high.reason_code
            }
        },
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20f::run_manifest_sha256()
    });
    v20f::write_report("security/cve_gate_report.json", &report);
}
