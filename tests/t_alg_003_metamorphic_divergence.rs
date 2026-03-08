use ocp_runtime_core::{run_source_with_engine, ErrorCode, RunEngine};

#[test]
fn t_alg_003_condition_flip_changes_outcome() {
    let pass_src = "condition(true);\n";
    let fail_src = "condition(false);\n";

    let pass_out = run_source_with_engine(pass_src, 301, 1000, RunEngine::Dual)
        .expect("condition(true) must pass");
    let fail_err = run_source_with_engine(fail_src, 302, 1000, RunEngine::Dual)
        .expect_err("condition(false) must fail");

    assert!(
        !pass_out.signature.is_empty(),
        "pass case must produce non-empty signature"
    );
    assert_eq!(
        fail_err.code.as_str(),
        ErrorCode::XConditionFalse.as_str(),
        "condition(false) must fail with X-CONDITION-FALSE"
    );
}

#[test]
fn t_alg_003_literal_change_changes_signature() {
    let src_a = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let src_b = r#"
observe("world.degraded", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;

    let out_a = run_source_with_engine(src_a, 401, 1000, RunEngine::Dual)
        .expect("variant A must run");
    let out_b = run_source_with_engine(src_b, 402, 1000, RunEngine::Dual)
        .expect("variant B must run");

    assert_ne!(
        out_a.signature, out_b.signature,
        "semantic literal change must change signature"
    );
}
