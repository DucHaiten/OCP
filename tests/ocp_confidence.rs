use ocp::ocp::{
    execute_program, parse_program, typecheck_program, CommitPolicyMode, ErrorCode, ExecConfig,
    Value,
};

fn deep_payload_field_expr(depth: usize) -> String {
    let mut expr = String::from("payload()");
    for _ in 0..depth {
        expr.push_str(".x");
    }
    expr
}

#[test]
fn confidence_v1_core_flow_is_deterministic_over_many_runs() {
    let src = r#"
let n = 1;
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
let marker = 0;
match r {
  OK => { let marker = 1; }
  DEGRADED => { let marker = 2; }
  INSUFFICIENT => { let marker = 3; }
  DEFERRED => { let marker = 4; }
}
condition(true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let mut baseline_signature = None::<String>;
    let mut baseline_events = None::<usize>;

    for _ in 0..64 {
        let out = execute_program(
            &p,
            ExecConfig {
                step_cap: 200,
                ..ExecConfig::default()
            },
        )
        .expect("exec should pass");

        assert_eq!(out.env.get("n"), Some(&Value::Int(1)));
        assert_eq!(out.env.get("marker"), Some(&Value::Int(4)));

        match (&baseline_signature, &baseline_events) {
            (None, None) => {
                baseline_signature = Some(out.signature.clone());
                baseline_events = Some(out.trace.events.len());
            }
            (Some(sig), Some(evt_len)) => {
                assert_eq!(out.signature, *sig);
                assert_eq!(out.trace.events.len(), *evt_len);
            }
            _ => panic!("baseline state should be initialized consistently"),
        }
    }
}

#[test]
fn confidence_v2_condition_reason_mapping_is_strict() {
    let deferred_src = format!("condition({});", deep_payload_field_expr(80));
    let deferred_program = parse_program(&deferred_src, 1).expect("parse should pass");
    let deferred_err = execute_program(
        &deferred_program,
        ExecConfig {
            step_cap: 2_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(
        deferred_err.code.as_str(),
        ErrorCode::XConditionDeferred.as_str()
    );
    assert_eq!(
        deferred_err.root_reason.map(|r| r.as_str()),
        Some("RC-BUDGET-EXCEEDED")
    );

    let insufficient_src = format!("condition({});", deep_payload_field_expr(300));
    let insufficient_program = parse_program(&insufficient_src, 1).expect("parse should pass");
    let insufficient_err = execute_program(
        &insufficient_program,
        ExecConfig {
            step_cap: 4_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(
        insufficient_err.code.as_str(),
        ErrorCode::XConditionInsufficient.as_str()
    );
    assert_eq!(
        insufficient_err.root_reason.map(|r| r.as_str()),
        Some("RC-NOT-IMPLEMENTED")
    );
}

#[test]
fn confidence_commit_policy_modes_are_explicit() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let normal = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            commit_policy: CommitPolicyMode::Normal,
            ..ExecConfig::default()
        },
    )
    .expect("normal mode should pass");
    assert_eq!(normal.commits.len(), 1);

    let forbid = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            commit_policy: CommitPolicyMode::ForbidCommit,
            ..ExecConfig::default()
        },
    )
    .expect("forbid mode should still return exec success");
    assert_eq!(forbid.commits.len(), 0);

    let shadow = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            commit_policy: CommitPolicyMode::ShadowCommitLog,
            ..ExecConfig::default()
        },
    )
    .expect("shadow mode should pass");
    assert_eq!(shadow.commits.len(), 1);
}

#[test]
fn confidence_runtime_error_surface_is_canonical_not_legacy_e_prefix() {
    let src = "condition(false);";
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");

    assert_eq!(err.code.as_str(), "X-COND-FALSE");
    assert!(!err.code.as_str().starts_with("E-"));
}
