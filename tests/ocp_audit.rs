use ocp::ocp::{execute_program, parse_program, typecheck_program, ExecConfig, TraceEvent};

#[test]
fn audit_signature_stable_for_same_program() {
    let src = r#"
let ok = true;
condition(ok);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");

    let out1 = execute_program(
        &program,
        ExecConfig {
            step_cap: 256,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    let out2 = execute_program(
        &program,
        ExecConfig {
            step_cap: 256,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}

#[test]
fn audit_trace_contains_program_end() {
    let src = r#"
let ok = true;
condition(ok);
"#;
    let program = parse_program(src, 2).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = execute_program(
        &program,
        ExecConfig {
            step_cap: 256,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    let has_program_end = out
        .trace
        .events
        .iter()
        .any(|ev| matches!(ev, TraceEvent::ProgramEnd { .. }));
    assert!(has_program_end, "audit trace must contain ProgramEnd");
}
