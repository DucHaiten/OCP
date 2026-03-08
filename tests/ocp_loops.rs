use ocp::ocp::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, Value,
};

#[test]
fn loops_repeat_and_for_cap_exec_ok() {
    let src = r#"
let xs = [1, 2, 3];
let marker = 0;
repeat 2 { let marker = 1; }
for item in xs cap 2 { let marker = item; }
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
    assert_eq!(out.env.get("marker"), Some(&Value::Int(0)));
}

#[test]
fn loops_typecheck_for_cap_iterable_must_be_list() {
    let src = r#"
let x = 1;
for item in x cap 2 { let y = item; }
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TForNotList);
}

#[test]
fn loops_typecheck_for_cap_requires_int_literal() {
    let src = r#"
let xs = [1];
let c = 1;
for item in xs cap c { let y = item; }
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TCapNotIntLit);
}

#[test]
fn loops_exec_repeat_cap_exceeded() {
    let src = r#"
repeat 10001 { let x = 1; }
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 50_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::XLoopCapExceeded);
    assert!(err.aliases.iter().any(|v| v == "X-LIMIT-EXCEEDED"));
    assert_eq!(err.limit_kind.as_deref(), Some("loop_cap"));
}

#[test]
fn loops_exec_for_cap_exceeded() {
    let src = r#"
let xs = [1, 2];
for item in xs cap 10001 { let y = item; }
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 50_000,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::XLoopCapExceeded);
    assert!(err.aliases.iter().any(|v| v == "X-LIMIT-EXCEEDED"));
    assert_eq!(err.limit_kind.as_deref(), Some("loop_cap"));
}
