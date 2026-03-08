use std::collections::BTreeMap;

use ocp::ocp::{
    parse_program, typecheck_program, validate_schema_value, CapabilityRegistry, ExecConfig,
    Executor, ReasonCode, ResultKind, Value,
};

fn run_with_registry(src: &str, registry: CapabilityRegistry) -> (CapabilityRegistry, Value) {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = Executor::with_registry(
        ExecConfig {
            step_cap: 200,
            ..ExecConfig::default()
        },
        registry.clone(),
    )
    .run(&program)
    .expect("exec should pass");
    let result = out.env.get("r").cloned().expect("binding r must exist");
    (registry, result)
}

fn payload_to_schema_value(payload: &Value) -> Value {
    match payload {
        Value::Payload(map) => {
            let mut out = BTreeMap::new();
            for (k, v) in map {
                out.insert(k.clone(), Value::String(v.clone()));
            }
            Value::Map(out)
        }
        other => other.clone(),
    }
}

#[test]
fn registry_has_ctx_and_payload_schema_for_core_pack_keys() {
    let reg = CapabilityRegistry::v1_baseline();
    let keys_with_payload = [
        "std.fs.read_text",
        "std.fs.list_dir",
        "std.fs.stat",
        "std.fs.write_text",
        "std.fs.mkdir",
        "std.fs.remove",
        "std.fs.rename",
        "std.fs.read",
        "std.fs.write",
        "std.fs.list",
        "std.kv.get",
        "std.kv.keys",
        "std.kv.put",
        "std.kv.del",
        "std.kv.clear",
        "std.time.now",
        "std.time.tick_info",
        "std.time.now_logical",
    ];

    for key in keys_with_payload {
        assert!(
            reg.ctx_schema_for_key(key).is_some(),
            "missing ctx schema for {key}"
        );
        assert!(
            reg.payload_schema_for_key(key).is_some(),
            "missing payload schema for {key}"
        );
    }

    assert!(reg.ctx_schema_for_key("std.time.sleep").is_some());
}

#[test]
fn runtime_ctx_schema_rejects_invalid_dynamic_key_ctx_for_core_pack() {
    let src = r#"
let k = "std.kv.keys";
observe(k, "tier2", ctx("cap=bad-int"), budget(5)) -> r;
"#;
    let (_reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::CtxInvalid));
}

#[test]
fn runtime_payload_schema_matches_std_time_tick_info_payload() {
    let src = r#"
observe("std.time.tick_info", "tier2", ctx("tick=7;dt_ms=20"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.time.tick_info")
        .expect("payload schema");
    let value = payload_to_schema_value(payload);
    validate_schema_value(payload_schema, &value).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_kv_keys_payload() {
    let src = r#"
observe("std.kv.keys", "tier2", ctx("cap=4"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert!(matches!(r.kind, ResultKind::Ok | ResultKind::Degraded));
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.kv.keys")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}
