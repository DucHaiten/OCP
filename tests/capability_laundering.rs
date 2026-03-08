#[path = "w17_gate_e_common.rs"]
mod w17;

use ocp_sdk::evaluate_capability_edge_v17;

#[test]
fn v17_capability_laundering_denies_when_edge_grant_missing() {
    let decision = evaluate_capability_edge_v17(
        "pkg:a",
        "pkg:c",
        "std.fs.read_text",
        "locked_v071",
        true,
        true,
        false,
    );
    assert!(!decision.allowed);
    assert_eq!(decision.reason_code, "RC-LAUNDER-EDGE-DENY");
    assert_eq!(decision.edge_id, "pkg:a|pkg:c|std.fs.read_text|locked_v071");
    w17::write_gate_e_report();
}

#[test]
fn v17_capability_laundering_requires_caller_and_callee_grants() {
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
    w17::write_gate_e_report();
}

#[test]
fn v17_capability_laundering_allows_only_canonical_edge_grant() {
    let decision = evaluate_capability_edge_v17(
        "pkg:a",
        "pkg:c",
        "std.fs.read_text",
        "locked_v071",
        true,
        true,
        true,
    );
    assert!(decision.allowed);
    assert_eq!(decision.reason_code, "RC-LAUNDER-OK");
    assert_eq!(decision.edge_id, "pkg:a|pkg:c|std.fs.read_text|locked_v071");
    w17::write_gate_e_report();
}
