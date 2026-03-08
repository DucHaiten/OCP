use ocp_runtime_core::{run_source_with_engine, RunEngine};

fn curated_programs() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "core_bool",
            r#"
let ready = true;
condition(ready);
"#,
        ),
        (
            "repeat_loop",
            r#"
repeat 4 { let pulse = true; }
condition(true);
"#,
        ),
        (
            "for_cap_array",
            r#"
let xs = [1, 2, 3];
for item in xs cap 2 { let seen = item; }
condition(true);
"#,
        ),
        (
            "observe_match",
            r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { let state = 1; }
  DEGRADED => { let state = 2; }
  INSUFFICIENT => { let state = 3; }
  DEFERRED => { let state = 4; }
}
condition(true);
"#,
        ),
        (
            "entangle_basic",
            r#"
let a = 1;
let b = 2;
entangle(a, b, true);
condition(true);
"#,
        ),
    ]
}

#[test]
fn t_alg_001_differential_engine_equivalence_curated() {
    let step_cap = 10_000;
    for (idx, (name, src)) in curated_programs().iter().enumerate() {
        let file_id = idx as u32 + 1;
        let interpreter = run_source_with_engine(src, file_id, step_cap, RunEngine::Interpreter)
            .unwrap_or_else(|err| panic!("interpreter failed for `{name}`: {err:?}"));
        let bytecode = run_source_with_engine(src, file_id, step_cap, RunEngine::Bytecode)
            .unwrap_or_else(|err| panic!("bytecode failed for `{name}`: {err:?}"));
        let dual = run_source_with_engine(src, file_id, step_cap, RunEngine::Dual)
            .unwrap_or_else(|err| panic!("dual failed for `{name}`: {err:?}"));

        assert_eq!(
            interpreter.signature, bytecode.signature,
            "signature mismatch interpreter vs bytecode for `{name}`"
        );
        assert_eq!(
            interpreter.signature, dual.signature,
            "signature mismatch interpreter vs dual for `{name}`"
        );
        assert_eq!(
            interpreter.env, bytecode.env,
            "final env mismatch interpreter vs bytecode for `{name}`"
        );
        assert_eq!(
            interpreter.env, dual.env,
            "final env mismatch interpreter vs dual for `{name}`"
        );
    }
}
