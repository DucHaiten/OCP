use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 500,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn engine_ui_run_returns_protocol_payload_and_stable_signature() {
    let src = r#"
observe("engine.ui.run", "tier2", ctx("entry_module=app.main;phase=frame;tick=7;dt_ms=16;w=800;h=600;scale=2;theme=retro;locale=vi-VN;input_cap=3;events=key:W|text:go;draw_cap=4;draw_list=text:1,1,go,12|rect:0,0,10,10,#fff;state_json={\"hp\":10}"), budget(5)) -> r;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(result)) = out1.env.get("r") else {
        panic!("expected engine.ui.run result");
    };
    assert_eq!(result.kind, ResultKind::Ok);
    let Some(Value::Map(map)) = &result.payload else {
        panic!("expected map payload");
    };
    assert_eq!(
        map.get("entry_module"),
        Some(&Value::String("app.main".to_string()))
    );
    assert_eq!(map.get("phase"), Some(&Value::String("frame".to_string())));
    assert_eq!(map.get("tick"), Some(&Value::Int(7)));
    assert_eq!(map.get("dt_ms"), Some(&Value::Int(16)));
    assert_eq!(map.get("input_truncated"), Some(&Value::Bool(false)));
    assert_eq!(map.get("draw_truncated"), Some(&Value::Bool(false)));

    let Some(Value::Map(frame_info)) = map.get("frame_info") else {
        panic!("expected frame_info map");
    };
    assert_eq!(frame_info.get("w"), Some(&Value::Int(800)));
    assert_eq!(frame_info.get("h"), Some(&Value::Int(600)));
    assert_eq!(frame_info.get("scale"), Some(&Value::Int(2)));
    assert_eq!(
        frame_info.get("theme"),
        Some(&Value::String("retro".to_string()))
    );
    assert_eq!(
        frame_info.get("locale"),
        Some(&Value::String("vi-VN".to_string()))
    );

    let Some(Value::List(input_events)) = map.get("input_events") else {
        panic!("expected input_events");
    };
    assert_eq!(input_events.len(), 2);

    let Some(Value::List(draw_list)) = map.get("draw") else {
        panic!("expected draw list");
    };
    assert_eq!(draw_list.len(), 2);
}

#[test]
fn engine_ui_run_degrades_when_input_or_draw_exceeds_cap() {
    let src = r#"
observe("engine.ui.run", "tier2", ctx("entry_module=app.main;tick=3;input_cap=1;events=key:W|quit;draw_cap=1;draw_list=text:1,1,go,12|rect:0,0,10,10,#fff"), budget(5)) -> r;
"#;

    let out = run_program(src);
    let Some(Value::Result4(result)) = out.env.get("r") else {
        panic!("expected engine.ui.run result");
    };
    assert_eq!(result.kind, ResultKind::Degraded);
    assert_eq!(result.reason, Some(ReasonCode::UiCapExceeded));

    let Some(Value::Map(map)) = &result.payload else {
        panic!("expected map payload");
    };
    assert_eq!(map.get("input_truncated"), Some(&Value::Bool(true)));
    assert_eq!(map.get("draw_truncated"), Some(&Value::Bool(true)));

    let Some(Value::List(input_events)) = map.get("input_events") else {
        panic!("expected input_events");
    };
    assert_eq!(input_events.len(), 1);

    let Some(Value::List(draw_list)) = map.get("draw") else {
        panic!("expected draw list");
    };
    assert_eq!(draw_list.len(), 1);
}
