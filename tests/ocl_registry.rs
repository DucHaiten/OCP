use ocp_ocl::ocp_ocl::{
    execute_program, parse_key, parse_program, typecheck_program, CapabilityRegistry, ExecConfig,
    Executor, KeyPattern, ReasonCode, ResultKind, Value,
};

#[test]
fn key_parser_accepts_family_name_sub() {
    let k = parse_key("world.exists.node").expect("key should parse");
    assert_eq!(k.family, "world");
    assert_eq!(k.parts, vec!["world", "exists", "node"]);
}

#[test]
fn key_parser_rejects_invalid_shape() {
    assert!(parse_key("world").is_none());
    assert!(parse_key("world..exists").is_none());
    assert!(parse_key("world.exi$ts").is_none());
}

#[test]
fn registry_denies_disabled_family() {
    let mut reg = CapabilityRegistry::v1_baseline();
    reg.add_deny_pattern("world.*");
    let err = reg
        .check_observe("world.exists", "scene=lab")
        .expect_err("must be denied");
    assert_eq!(err.to_reason_code(), ReasonCode::CapabilityDenied);
}

#[test]
fn registry_ctx_required_enforced() {
    let mut reg = CapabilityRegistry::v1_baseline();
    reg.set_ctx_required_for_key("world.exists", vec!["scene".to_string()]);
    let err = reg
        .check_observe("world.exists", "mode=demo")
        .expect_err("ctx should be invalid");
    assert_eq!(err.to_reason_code(), ReasonCode::CtxInvalid);
}

#[test]
fn exec_observe_permission_failure_returns_insufficient() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let mut reg = CapabilityRegistry::v1_baseline();
    reg.add_deny_pattern("world.exists");
    let out = Executor::with_registry(ExecConfig { step_cap: 100 }, reg)
        .run(&p)
        .expect("exec should pass with insufficient result");

    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4 binding");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::CapabilityDenied));
}

#[test]
fn exec_observe_allowed_returns_deferred_stub() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(&p, ExecConfig { step_cap: 100 }).expect("exec should pass");
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4 binding");
    };
    assert_eq!(r.kind, ResultKind::Deferred);
    assert_eq!(r.reason, Some(ReasonCode::NotImplemented));
    assert!(r.origin_id.is_some());
}

#[test]
fn key_pattern_matching_basics() {
    let any = KeyPattern::from_pattern("*");
    let pref = KeyPattern::from_pattern("world.*");
    let ex = KeyPattern::from_pattern("world.exists");
    assert!(any.matches("x.y"));
    assert!(pref.matches("world.exists"));
    assert!(!pref.matches("render.frame"));
    assert!(ex.matches("world.exists"));
    assert!(!ex.matches("world.exists.v2"));
}

#[test]
fn registry_denies_std_net_poll_by_default() {
    let reg = CapabilityRegistry::v1_baseline();
    let err = reg
        .check_observe("std.net.poll", "conn=local")
        .expect_err("std.net.poll must be denied in user path");
    assert_eq!(err.to_reason_code(), ReasonCode::CapabilityDenied);
}
