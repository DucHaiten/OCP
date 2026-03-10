use ocp::ocp::{execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, Value};

const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const SHA256_EV123: &str = "294022bdce08602a5986d211c9e6fcc08e80c030ae28a97473a5f7fd294f3d19";

#[test]
fn std_hash_sha256_returns_canonical_hex_for_known_vector() {
    let src = r#"
let h = std.hash.sha256("abc");
condition(eq(h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
"#;
    let program = parse_program(src, 91).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = execute_program(&program, ExecConfig::default()).expect("exec should pass");

    assert_eq!(out.env.get("h"), Some(&Value::String(SHA256_ABC.to_string())));
}

#[test]
fn std_hash_sha256_accepts_runtime_string_value() {
    let src = r#"
let seed = {"raw":"abc"};
let h = std.hash.sha256(seed.raw);
condition(eq(h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
"#;
    let program = parse_program(src, 92).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = execute_program(&program, ExecConfig::default()).expect("exec should pass");

    assert_eq!(out.env.get("h"), Some(&Value::String(SHA256_ABC.to_string())));
}

#[test]
fn std_hash_sha256_typecheck_rejects_non_string_argument() {
    let src = r#"
let h = std.hash.sha256(1);
"#;
    let program = parse_program(src, 93).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}

#[test]
fn std_hash_sha256_runtime_rejects_non_string_value() {
    let src = r#"
let seed = {"value":1};
let h = std.hash.sha256(seed.value);
"#;
    let program = parse_program(src, 94).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let err = execute_program(&program, ExecConfig::default()).expect_err("exec should fail");
    assert_eq!(err.code, ErrorCode::XCommitForbidden);
    assert!(
        err.message
            .contains("std.hash.sha256(...) argument must evaluate to string"),
        "unexpected error message: {}",
        err.message
    );
}

#[test]
fn std_hash_sha256_supports_dynamic_state_hash_composition() {
    let src = r#"
let payload = {"events":"E","views":"V","s1":"1","s2":"2","s3":"3"};
let canonical = concat(payload.events, payload.views, payload.s1, payload.s2, payload.s3);
let state_hash = std.hash.sha256(canonical);
condition(eq(state_hash, "294022bdce08602a5986d211c9e6fcc08e80c030ae28a97473a5f7fd294f3d19"));
"#;
    let program = parse_program(src, 95).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = execute_program(&program, ExecConfig::default()).expect("exec should pass");

    assert_eq!(
        out.env.get("state_hash"),
        Some(&Value::String(SHA256_EV123.to_string()))
    );
}
