use ocp_ocl::ocp_ocl::{parse_program, ErrorCode, Stmt};

#[test]
fn parse_core_statements_ok() {
    let src = r#"
let k = "world.exists";
observe(k, "tier2", ctx("scene=lab"), budget(10)) -> r;
commit(r);
condition(true);
entangle(k, r, true);
"#;

    let program = parse_program(src, 1).expect("program should parse");
    assert_eq!(program.statements.len(), 5);
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

#[test]
fn parse_repeat_and_for_cap_ok() {
    let src = r#"
let xs = [1, 2, 3];
repeat 2 { let a = 1; }
for item in xs cap 2 { let b = item; }
"#;
    let program = parse_program(src, 1).expect("loop forms should parse");
    assert_eq!(program.statements.len(), 3);
    match &program.statements[1] {
        Stmt::Repeat { .. } => {}
        other => panic!("expected repeat statement, got {other:?}"),
    }
    match &program.statements[2] {
        Stmt::ForEachCap { .. } => {}
        other => panic!("expected for-cap statement, got {other:?}"),
    }
}

#[test]
fn parse_try_else_and_guard_ok() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> rs;
let msg = try rs else { r.reason_code };
guard rs;
"#;
    let program = parse_program(src, 1).expect("sugar statements should parse");
    assert_eq!(program.statements.len(), 3);
    match &program.statements[1] {
        Stmt::TryLet { .. } => {}
        other => panic!("expected try-let statement, got {other:?}"),
    }
    match &program.statements[2] {
        Stmt::Guard { .. } => {}
        other => panic!("expected guard statement, got {other:?}"),
    }
}
