use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, TraceEvent,
    Value,
};

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 400,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn std_ui_frame_info_is_deterministic_and_keeps_ctx_tick_compat() {
    let src = r#"
observe("std.ui.frame_info", "tier2", ctx("ctx_tick=42;w=800;h=600;scale=2;theme=retro;locale=vi-VN"), budget(5)) -> f;
"#;

    let out = run_program(src);
    let Some(Value::Result4(frame)) = out.env.get("f") else {
        panic!("expected frame_info result");
    };
    assert_eq!(frame.kind, ResultKind::Ok);
    match &frame.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("ctx_tick"), Some(&"42".to_string()));
            assert_eq!(map.get("frame"), Some(&"42".to_string()));
            assert_eq!(map.get("w"), Some(&"800".to_string()));
            assert_eq!(map.get("h"), Some(&"600".to_string()));
            assert_eq!(map.get("scale"), Some(&"2".to_string()));
            assert_eq!(map.get("theme"), Some(&"retro".to_string()));
            assert_eq!(map.get("locale"), Some(&"vi-VN".to_string()));
        }
        _ => panic!("expected payload map"),
    }
}

#[test]
fn std_ui_input_truncates_with_degraded_and_ui_observe_trace() {
    let src = r#"
observe("std.ui.input", "tier2", ctx("cap=2;events=key:Space|text:hello|quit"), budget(5)) -> input;
"#;

    let out = run_program(src);
    let Some(Value::Result4(input)) = out.env.get("input") else {
        panic!("expected input result");
    };
    assert_eq!(input.kind, ResultKind::Degraded);
    assert_eq!(input.reason, Some(ReasonCode::UiCapExceeded));
    match &input.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(events)) = map.get("events") else {
                panic!("expected events list");
            };
            assert_eq!(events.len(), 2);
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected map payload"),
    }

    assert!(
        out.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::UiObserve {
                key,
                event_count,
                truncated,
                kind,
                reason,
                ..
            } if key == "std.ui.input"
                && *event_count == 2
                && *truncated
                && *kind == ResultKind::Degraded
                && *reason == Some(ReasonCode::UiCapExceeded)
        )),
        "expected UiObserve trace for input truncation"
    );
}

#[test]
fn std_ui_draw_and_present_commit_emit_ui_commit_trace() {
    let src = r#"
observe("std.ui.draw", "tier2", ctx("cap=3;list=rect:0,0,10,10,#fff|text:1,1,hi,12"), budget(5)) -> draw_r;
commit(draw_r);
observe("std.ui.present", "tier2", ctx("scope=frame"), budget(5)) -> present_r;
commit(present_r);
"#;

    let out = run_program(src);
    assert_eq!(out.commits.len(), 2);
    assert_eq!(out.commits[0].key, "std.ui.draw");
    assert_eq!(out.commits[1].key, "std.ui.present");

    assert!(
        out.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::UiCommit {
                key,
                cmd_count,
                present,
                kind,
                ..
            } if key == "std.ui.draw" && *cmd_count == 2 && !*present && *kind == ResultKind::Ok
        )),
        "expected UiCommit draw trace"
    );
    assert!(
        out.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::UiCommit {
                key,
                cmd_count,
                present,
                kind,
                ..
            } if key == "std.ui.present" && *cmd_count == 0 && *present && *kind == ResultKind::Ok
        )),
        "expected UiCommit present trace"
    );
}

#[test]
fn std_ui_draw_deferred_when_cmd_count_exceeds_cap() {
    let src = r#"
observe("std.ui.draw", "tier2", ctx("cap=1;list=rect:0,0,10,10,#fff|text:1,1,hi,12"), budget(5)) -> draw_r;
"#;

    let out = run_program(src);
    let Some(Value::Result4(draw_r)) = out.env.get("draw_r") else {
        panic!("expected draw result");
    };
    assert_eq!(draw_r.kind, ResultKind::Deferred);
    assert_eq!(draw_r.reason, Some(ReasonCode::UiCapExceeded));
}
