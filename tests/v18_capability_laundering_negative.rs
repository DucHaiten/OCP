use serde_json::json;

use ocl_sdk::evaluate_capability_edge_v17;

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_capability_laundering_negative_requires_canonical_edge_grant() {
    common::ensure_run_manifest();

    let edge_missing = evaluate_capability_edge_v17(
        "pkg:a",
        "pkg:c",
        "std.fs.read_text",
        "locked_v071",
        true,
        true,
        false,
    );
    assert!(!edge_missing.allowed);
    assert_eq!(edge_missing.reason_code, "RC-LAUNDER-EDGE-DENY");

    let caller_denied = evaluate_capability_edge_v17(
        "pkg:a",
        "pkg:c",
        "std.fs.read_text",
        "locked_v071",
        false,
        true,
        true,
    );
    assert!(!caller_denied.allowed);
    assert_eq!(caller_denied.reason_code, "RC-LAUNDER-CALLER-DENY");

    let callee_denied = evaluate_capability_edge_v17(
        "pkg:a",
        "pkg:c",
        "std.fs.read_text",
        "locked_v071",
        true,
        false,
        true,
    );
    assert!(!callee_denied.allowed);
    assert_eq!(callee_denied.reason_code, "RC-LAUNDER-CALLEE-DENY");

    common::merge_pack_shipproof_section(
        "capability_laundering_negative",
        json!({
            "status": "PASS",
            "edge_id": edge_missing.edge_id,
            "edge_missing_reason": edge_missing.reason_code,
            "caller_denied_reason": caller_denied.reason_code,
            "callee_denied_reason": callee_denied.reason_code
        }),
    );
}
