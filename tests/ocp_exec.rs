use ocp::ocp::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, Value,
};

fn deep_payload_field_expr(depth: usize) -> String {
    let mut expr = String::from("payload()");
    for _ in 0..depth {
        expr.push_str(".x");
    }
    expr
}

#[test]
fn exec_let_and_condition_ok() {
    let src = r#"
let x = 1;
condition(true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.env.get("x"), Some(&Value::Int(1)));
}

#[test]
fn exec_match_selects_deferred_arm_from_observe_stub() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
let marker = 0;
match r {
  OK => { let marker = 1; }
  DEGRADED => { let marker = 2; }
  INSUFFICIENT => { let marker = 3; }
  DEFERRED => { let marker = 4; }
}
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(
        &p,
        ExecConfig {
            step_cap: 200,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.env.get("marker"), Some(&Value::Int(4)));
}

#[test]
fn exec_condition_false_fails() {
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
    assert_eq!(err.code.as_str(), ErrorCode::XConditionFalse.as_str());
}

#[test]
fn exec_step_cap_exceeded_fails() {
    let src = r#"
let a = 1;
let b = 2;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 1,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), ErrorCode::XBudgetExceeded.as_str());
}

#[test]
fn exec_commit_deferred_result_fails() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
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
    assert_eq!(err.code.as_str(), ErrorCode::XCommitForbidden.as_str());
}

#[test]
fn exec_condition_deferred_when_condition_budget_exceeded() {
    let src = format!("condition({});", deep_payload_field_expr(80));
    let p = parse_program(&src, 1).expect("parse should pass");

    // V2-C: runtime bounded evaluator may defer before bool proof, independent from typecheck.
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 2_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), ErrorCode::XConditionDeferred.as_str());
    assert_eq!(
        err.root_reason.map(|r| r.as_str()),
        Some("RC-BUDGET-EXCEEDED")
    );
}

#[test]
fn exec_condition_insufficient_when_constraint_cap_exceeded() {
    let src = format!("condition({});", deep_payload_field_expr(300));
    let p = parse_program(&src, 1).expect("parse should pass");

    // V2-C: oversized condition graph becomes insufficient with explicit root reason.
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 4_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(
        err.code.as_str(),
        ErrorCode::XConditionInsufficient.as_str()
    );
    assert_eq!(
        err.root_reason.map(|r| r.as_str()),
        Some("RC-NOT-IMPLEMENTED")
    );
}

#[test]
fn exec_entangle_session_local_ok() {
    let src = r#"
let a = 1;
let b = 2;
entangle(a, b, true);
entangle(a, b, true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.env.get("a"), Some(&Value::Int(1)));
}

#[test]
fn exec_entangle_degree_cap_fails() {
    let mut src = String::from("let a = 1; let b = 2;");
    for _ in 0..17 {
        src.push_str("entangle(a, b, true);");
    }

    let p = parse_program(&src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 5000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), "X-ENTANGLE-DEGREE-CAP");
    assert_eq!(
        err.root_reason.map(|r| r.as_str()),
        Some("RC-POLICY-DENIED")
    );
}
