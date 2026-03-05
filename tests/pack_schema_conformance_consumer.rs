use std::collections::BTreeMap;

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, validate_schema_value, CapabilityRegistry, ExecConfig,
    Executor, ReasonCode, ResultKind, Value,
};

fn run_with_registry(src: &str, registry: CapabilityRegistry) -> (CapabilityRegistry, Value) {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = Executor::with_registry(
        ExecConfig {
            step_cap: 400,
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
fn registry_has_ctx_and_payload_schema_for_consumer_pack_keys() {
    let reg = CapabilityRegistry::v1_baseline();
    let keys_with_payload = [
        "std.ui.frame_info",
        "std.ui.input",
        "std.ui.draw",
        "std.ui.present",
        "std.game.tick_info",
        "std.game.rng",
        "std.game.state_delta",
        "std.shadow.run",
        "std.shadow.search",
        "std.shadow.compare",
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
}

#[test]
fn runtime_ctx_schema_rejects_invalid_dynamic_key_ctx_for_consumer_pack() {
    let src = r#"
let k = "std.ui.input";
observe(k, "tier2", ctx("cap=bad-int;events=key:W"), budget(5)) -> r;
"#;
    let (_reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::CtxInvalid));
}

#[test]
fn runtime_payload_schema_matches_std_ui_frame_info_payload() {
    let src = r#"
observe("std.ui.frame_info", "tier2", ctx("w=800;h=600;scale=2;theme=light;locale=vi-VN;ctx_tick=5"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.ui.frame_info")
        .expect("payload schema");
    let value = payload_to_schema_value(payload);
    validate_schema_value(payload_schema, &value).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_game_tick_info_payload() {
    let src = r#"
observe("std.game.tick_info", "tier2", ctx("tick=7"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.game.tick_info")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_shadow_compare_payload() {
    let src = r#"
observe("std.shadow.compare", "tier2", ctx("branches_json=[{\"id\":0,\"outcome\":\"OK\",\"signature\":\"a\",\"cost\":{\"steps\":1,\"budget\":2},\"state_summary\":{\"x\":1}},{\"id\":1,\"outcome\":\"OK\",\"signature\":\"b\",\"cost\":{\"steps\":2,\"budget\":3},\"state_summary\":{\"x\":2}}];baseline_id=0;max_diff_keys=2"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert!(matches!(r.kind, ResultKind::Ok | ResultKind::Degraded));
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.shadow.compare")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_shadow_search_payload() {
    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=round_robin;variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}];max_branches=3;per_branch_step_cap=80;per_branch_budget_cap=2000;global_step_cap=240;global_budget_cap=12000;rounds=2;top_k=3"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert!(matches!(r.kind, ResultKind::Ok | ResultKind::Degraded));
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.shadow.search")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}
