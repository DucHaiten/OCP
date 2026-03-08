use ocp::ocp::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, ReasonCode,
    ResultKind, Value,
};

#[test]
fn std_json_parse_valid_returns_ok_result4_value() {
    let src = r#"
let r = std.json.parse("{\"a\":1,\"b\":[true,\"x\"]}");
"#;
    let p = parse_program(src, 81).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    assert_eq!(r.reason, None);
    match &r.payload {
        Some(Value::Map(map)) => {
            assert!(map.contains_key("a"));
            assert!(map.contains_key("b"));
        }
        other => panic!("expected map payload, got {:?}", other),
    }
}

#[test]
fn std_json_parse_invalid_returns_insufficient_rc_json_invalid() {
    let src = r#"
let r = std.json.parse("{\"a\":");
"#;
    let p = parse_program(src, 82).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::JsonInvalid));
}

#[test]
fn std_json_stringify_is_deterministic_and_sorted_for_map_keys() {
    let src = r#"
let m = {"b": 2, "a": 1};
let s1 = std.json.stringify(m);
let s2 = std.json.stringify(m);
"#;
    let p = parse_program(src, 83).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig::default()).expect("exec should pass");

    let Some(Value::String(s1)) = out.env.get("s1") else {
        panic!("expected s1 string");
    };
    let Some(Value::String(s2)) = out.env.get("s2") else {
        panic!("expected s2 string");
    };
    assert_eq!(s1, s2);
    assert_eq!(s1, "{\"a\":1,\"b\":2}");
}

#[test]
fn std_json_typecheck_rejects_parse_non_string_argument() {
    let src = r#"
let r = std.json.parse(1);
"#;
    let p = parse_program(src, 84).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
