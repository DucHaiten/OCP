use ocp_ocl::ocp_ocl::{execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, Value};

#[test]
fn exec_let_and_condition_ok() {
    let src = r#"
let x = 1;
condition(true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig { step_cap: 100 }).expect("exec should pass");
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
    let out = execute_program(&p, ExecConfig { step_cap: 200 }).expect("exec should pass");
    assert_eq!(out.env.get("marker"), Some(&Value::Int(4)));
}

#[test]
fn exec_condition_false_fails() {
    let src = "condition(false);";
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(&p, ExecConfig { step_cap: 100 }).expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::EConditionFalse);
}

#[test]
fn exec_step_cap_exceeded_fails() {
    let src = r#"
let a = 1;
let b = 2;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(&p, ExecConfig { step_cap: 1 }).expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::EBudgetExceeded);
}

#[test]
fn exec_commit_deferred_result_fails() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(&p, ExecConfig { step_cap: 100 }).expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::ECommitForbidden);
}
