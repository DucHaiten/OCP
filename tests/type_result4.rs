use ocp::ocp::{parse_program, typecheck_program, ErrorCode};

#[test]
fn type_try_unwrap_extracts_non_result4_payload() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> rs;
let payload_value = rs?;
commit(payload_value);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn type_try_else_result4_flow_ok() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> rs;
let msg = try rs else { "fallback" };
condition(true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
}

#[test]
fn type_try_operator_requires_result4() {
    let src = r#"
let x = 1;
let y = x?;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
