use ocp_ocl::ocp_ocl::{parse_program, ErrorCode, Stmt};

#[test]
fn parse_core_statements_ok() {
    let src = r#"
let k = "world.exists";
observe(k, "tier2", ctx("scene=lab"), budget(10)) -> r;
commit(r);
condition(true);
"#;

    let program = parse_program(src, 1).expect("program should parse");
    assert_eq!(program.statements.len(), 4);
}

#[test]
fn parse_match_with_all_arms_ok() {
    let src = r#"
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;

    let program = parse_program(src, 1).expect("match should parse");
    assert_eq!(program.statements.len(), 1);
    let Stmt::Match(m) = &program.statements[0] else {
        panic!("expected match statement");
    };
    assert_eq!(m.ok_arm.len(), 1);
    assert_eq!(m.degraded_arm.len(), 1);
    assert_eq!(m.insufficient_arm.len(), 1);
    assert_eq!(m.deferred_arm.len(), 1);
}

#[test]
fn parse_fail_when_missing_semicolon() {
    let src = "let x = 42";
    let err = parse_program(src, 1).expect_err("missing semicolon should fail");
    assert_eq!(err.code, ErrorCode::PUnexpectedToken);
}

#[test]
fn parse_fail_when_match_arms_incomplete() {
    let src = r#"
match r {
  OK => { commit(r); }
}
"#;

    let err = parse_program(src, 1).expect_err("incomplete match arms should fail");
    assert_eq!(err.code, ErrorCode::PUnexpectedToken);
    assert!(
        err.message.contains("all 4 arms"),
        "unexpected message: {}",
        err.message
    );
}
