use ocp::ocp::{parse_program, typecheck_program, ExecConfig, Executor, TraceEvent};

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
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
fn ui_replay_signature_is_stable_for_same_input_stream() {
    let src = r#"
observe("std.ui.input", "tier2", ctx("cap=4;events=key:W|mouse:10,12|text:go"), budget(5)) -> input_r;
observe("std.ui.draw", "tier2", ctx("cap=4;list=text:1,1,go,12"), budget(5)) -> draw_r;
commit(draw_r);
observe("std.ui.present", "tier2", ctx("scope=frame"), budget(5)) -> present_r;
commit(present_r);
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);

    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}

#[test]
fn ui_replay_signature_changes_when_input_order_changes() {
    let src_a = r#"
observe("std.ui.input", "tier2", ctx("cap=4;events=key:W|key:S"), budget(5)) -> input_r;
"#;
    let src_b = r#"
observe("std.ui.input", "tier2", ctx("cap=4;events=key:S|key:W"), budget(5)) -> input_r;
"#;

    let out_a = run_program(src_a);
    let out_b = run_program(src_b);

    assert_ne!(out_a.signature, out_b.signature);
}

#[test]
fn ui_replay_trace_contains_canonical_input_detail() {
    let src = r#"
observe("std.ui.input", "tier2", ctx("cap=3;events=key:W|text:hello|mouse:5,9"), budget(5)) -> input_r;
"#;

    let out = run_program(src);
    assert!(out.trace.events.iter().any(|ev| matches!(
        ev,
        TraceEvent::UiObserve {
            key,
            detail,
            event_count,
            ..
        } if key == "std.ui.input" && *event_count == 3 && detail.contains("\"events\"")
    )));
}
