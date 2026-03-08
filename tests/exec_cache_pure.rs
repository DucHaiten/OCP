use ocp::ocp::{execute_program, parse_program, typecheck_program, ExecConfig, Value};

fn run_program(src: &str, enable_exec_cache: bool) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    execute_program(
        &program,
        ExecConfig {
            step_cap: 20_000,
            enable_exec_cache,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass")
}

#[test]
fn exec_cache_hits_reduce_executed_nodes_without_changing_signature_or_steps() {
    let src = r#"
for i in 0..40 {
  let x = { a: { nested: [1, 2, 3], marker: "ok" }, b: [4, 5, 6] };
}
"#;

    let no_cache = run_program(src, false);
    let with_cache = run_program(src, true);

    assert_eq!(with_cache.signature, no_cache.signature);
    assert_eq!(with_cache.steps, no_cache.steps);
    assert!(with_cache.exec_cache.enabled);
    assert!(
        with_cache.exec_cache.hits > 0,
        "expected exec cache to hit repeated pure subtrees"
    );
    assert!(
        with_cache.exec_cache.node_evals_executed < no_cache.exec_cache.node_evals_executed,
        "cache should reduce executed expression evaluations"
    );
    assert_eq!(
        with_cache.exec_cache.node_evals_charged, no_cache.exec_cache.node_evals_charged,
        "charged node evaluations stay semantics-equivalent"
    );
}

#[test]
fn exec_cache_key_tracks_identifier_bindings_to_avoid_stale_values() {
    let src = r#"
let i = 1;
let a = [i, i];
let i = 2;
let b = [i, i];
"#;

    let out = run_program(src, true);
    assert_eq!(
        out.env.get("a"),
        Some(&Value::List(vec![Value::Int(1), Value::Int(1)]))
    );
    assert_eq!(
        out.env.get("b"),
        Some(&Value::List(vec![Value::Int(2), Value::Int(2)]))
    );
}
