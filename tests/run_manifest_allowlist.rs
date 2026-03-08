#[path = "v16_gate_a_common.rs"]
mod v16;

#[test]
fn run_manifest_allowlist_is_deny_by_default() {
    let allowed = vec![
        "OCP_QUARANTINE".to_string(),
        "OCP_ALLOW_OVERRIDES".to_string(),
    ];
    v16::enforce_run_manifest_allowlist(
        &allowed,
        &[
            "OCP_QUARANTINE".to_string(),
            "OCP_ALLOW_OVERRIDES".to_string(),
        ],
    )
    .expect("known env flags should pass");

    let err = v16::enforce_run_manifest_allowlist(
        &allowed,
        &[
            "OCP_QUARANTINE".to_string(),
            "OCP_UNDECLARED_FLAG".to_string(),
        ],
    )
    .expect_err("unknown env flag must fail");
    assert!(
        err.contains("OCP_UNDECLARED_FLAG"),
        "error should mention unknown env flag: {err}"
    );
}
