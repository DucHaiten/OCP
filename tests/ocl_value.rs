use ocp_ocl::ocp_ocl::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, ReasonCode, Value,
};

#[test]
fn value_len_supports_list_map_string_and_payload() {
    let src = r#"
let xs = [1, 2, 3];
let n1 = len(xs);
let m = {"a": 1, "b": 2};
let n2 = len(m);
let s = "abc";
let n3 = len(s);
let p = payload();
let n4 = len(p);
"#;
    let p = parse_program(src, 71).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    assert_eq!(out.env.get("n1"), Some(&Value::Int(3)));
    assert_eq!(out.env.get("n2"), Some(&Value::Int(2)));
    assert_eq!(out.env.get("n3"), Some(&Value::Int(3)));
    assert_eq!(out.env.get("n4"), Some(&Value::Int(0)));
}

#[test]
fn value_keys_returns_sorted_and_capped_keys() {
    let src = r#"
let m = {"b": 2, "a": 1, "c": 3};
let ks = keys(m, 2);
"#;
    let p = parse_program(src, 72).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    assert_eq!(
        out.env.get("ks"),
        Some(&Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]))
    );
}

#[test]
fn value_merge_merges_deterministically_and_overwrites_right() {
    let src = r#"
let a = {"a": 1, "b": 2};
let b = {"b": 9, "c": 3};
let m = merge(a, b, 3);
let b2 = m.b;
let n = len(m);
"#;
    let p = parse_program(src, 73).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    assert_eq!(out.env.get("b2"), Some(&Value::Int(9)));
    assert_eq!(out.env.get("n"), Some(&Value::Int(3)));
}

#[test]
fn value_keys_cap_exceeded_surfaces_canonical_code_and_alias_metadata() {
    let src = r#"
let m = {"a": 1};
let ks = keys(m, 2001);
"#;
    let p = parse_program(src, 74).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(&p, ExecConfig::default()).expect_err("exec should fail");

    assert_eq!(err.code, ErrorCode::XKeysCapExceeded);
    assert!(err.aliases.iter().any(|v| v == "X-LIMIT-EXCEEDED"));
    assert_eq!(err.limit_kind.as_deref(), Some("value_keys_cap"));
    assert_eq!(err.root_reason, Some(ReasonCode::PolicyDenied));
}

#[test]
fn value_merge_result_cap_exceeded_surfaces_canonical_code_and_alias_metadata() {
    let src = r#"
let a = {"a": 1, "b": 2};
let b = {"c": 3};
let m = merge(a, b, 2);
"#;
    let p = parse_program(src, 75).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(&p, ExecConfig::default()).expect_err("exec should fail");

    assert_eq!(err.code, ErrorCode::XKeysCapExceeded);
    assert!(err.aliases.iter().any(|v| v == "X-LIMIT-EXCEEDED"));
    assert_eq!(err.limit_kind.as_deref(), Some("value_keys_cap"));
    assert_eq!(err.root_reason, Some(ReasonCode::PolicyDenied));
}

#[test]
fn value_typecheck_rejects_invalid_keys_argument_type() {
    let src = r#"
let xs = [1, 2];
let ks = keys(xs, 1);
"#;
    let p = parse_program(src, 76).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
