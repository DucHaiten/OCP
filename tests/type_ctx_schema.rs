use ocp::ocp::{parse_program, typecheck_program, ErrorCode};

#[test]
fn type_ctx_schema_accepts_valid_literal_record() {
    let src = r#"
observe("std.fs.read_text", "tier2", { path: "./README.md", max_bytes: 128 }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
}

#[test]
fn type_ctx_schema_missing_required_field_fails() {
    let src = r#"
observe("std.fs.read_text", "tier2", { max_bytes: 128 }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TCtxMissingField);
}

#[test]
fn type_ctx_schema_unknown_field_fails() {
    let src = r#"
observe("std.fs.read_text", "tier2", { path: "./README.md", typo: true }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TCtxUnknownField);
}

#[test]
fn type_ctx_schema_field_type_mismatch_fails() {
    let src = r#"
observe("std.fs.read_text", "tier2", { path: "./README.md", max_bytes: "128" }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TCtxTypeMismatch);
}

#[test]
fn type_ctx_schema_constraint_violation_fails() {
    let src = r#"
observe("std.fs.read_text", "tier2", { path: "./README.md", max_bytes: 0 }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TCtxConstraintViolation);
}

#[test]
fn type_ctx_schema_dynamic_key_stays_runtime_validated() {
    let src = r#"
let key = "std.fs.read_text";
observe(key, "tier2", { max_bytes: 0 }, budget(10)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass for dynamic-key path");
}
