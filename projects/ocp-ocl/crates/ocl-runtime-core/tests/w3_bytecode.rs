use ocl_runtime_core::{compile_source, run_source, run_source_with_engine, RunEngine, TraceEvent};

#[test]
fn w3_compile_emits_ir_bytecode_and_source_map() {
    let src = r#"
let a = 1;
let b = 2;
condition(true);
"#;
    let compiled = compile_source(src, 1).expect("compile should pass");
    assert!(compiled.ir.op_count() >= 3);
    assert_eq!(compiled.ir.op_count(), compiled.bytecode.op_count());
    assert_eq!(
        compiled.source_map.entries.len(),
        compiled.bytecode.op_count()
    );
    assert!(compiled.source_map.span_for_pc(0).is_some());
}

#[test]
fn w3_bytecode_matches_interpreter_signature() {
    let src = r#"
observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19091"), budget(5)) -> listen_res;
match listen_res {
  OK => { condition(true); }
  DEGRADED => { condition(true); }
  INSUFFICIENT => { condition(true); }
  DEFERRED => { condition(true); }
}
"#;
    let interpreted = run_source(src, 1, 4096).expect("interpreted run");
    let bytecode = run_source_with_engine(src, 1, 4096, RunEngine::Bytecode).expect("bytecode run");
    assert_eq!(interpreted.signature, bytecode.signature);
    assert_eq!(interpreted.commits.len(), bytecode.commits.len());
}

#[test]
fn w3_dual_mode_passes() {
    let src = r#"
let x = 1;
condition(true);
"#;
    let out = run_source_with_engine(src, 1, 4096, RunEngine::Dual).expect("dual run");
    assert!(out.steps > 0);
    let saw_program_end = out
        .trace
        .events
        .iter()
        .any(|ev| matches!(ev, TraceEvent::ProgramEnd { .. }));
    assert!(saw_program_end);
}
