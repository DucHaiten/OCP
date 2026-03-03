use ocp_ocl::ocp_ocl::{execute_program, parse_program, typecheck_program, ExecConfig, Value};

#[test]
fn toy_program_observe_match_commit_flow() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(8)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");

    let out = execute_program(
        &program,
        ExecConfig {
            step_cap: 300,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
}

#[test]
fn toy_program_entangle_with_condition_pass() {
    let src = r#"
let a = 1;
let b = 2;
entangle(a, b, true);
condition(true);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");

    let out = execute_program(
        &program,
        ExecConfig {
            step_cap: 300,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.env.get("a"), Some(&Value::Int(1)));
    assert_eq!(out.env.get("b"), Some(&Value::Int(2)));
}

#[test]
fn toy_program_failure_honest_on_unknown_condition_payload() {
    let src = "condition(payload().x);";
    let program = parse_program(src, 1).expect("parse should pass");
    let err = execute_program(
        &program,
        ExecConfig {
            step_cap: 300,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), "X-COND-INSUFFICIENT");
}
