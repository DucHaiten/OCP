use ocp_ocl::ocp_ocl::{parse_program, typecheck_program, ErrorCode};

#[test]
fn type_record_destructure_ok() {
    let src = r#"
let rec = { a: 1, b: "x" };
let {a, b} = rec;
condition(true);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
}

#[test]
fn type_record_destructure_missing_field_fails() {
    let src = r#"
let rec = { a: 1 };
let {a, b} = rec;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn type_record_destructure_non_record_fails() {
    let src = r#"
let x = 1;
let {a} = x;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
