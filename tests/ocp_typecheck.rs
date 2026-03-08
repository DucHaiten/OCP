use ocp::ocp::{parse_program, typecheck_program, ErrorCode};

#[test]
fn typecheck_core_flow_ok() {
    let src = r#"
let k = "world.exists";
observe(k, "tier2", ctx("scene=lab"), budget(10)) -> r;
entangle(k, r, true);
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
}

#[test]
fn typecheck_fails_entangle_constraint_non_bool() {
    let src = r#"
let a = 1;
let b = 2;
entangle(a, b, 1);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn typecheck_fails_condition_non_bool() {
    let src = "condition(1);";
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn typecheck_fails_commit_non_observe_binding() {
    let src = r#"
let x = 1;
commit(x);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn typecheck_fails_unknown_identifier() {
    let src = "commit(r);";
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TUnknownIdentifier);
}

#[test]
fn typecheck_fails_observe_wrong_arg_types() {
    let src = r#"
observe(1, "tier2", ctx("scene=lab"), budget(10)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
