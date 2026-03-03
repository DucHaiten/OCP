use ocp_ocl::ocp_ocl::{execute_program, parse_program, typecheck_program, ExecConfig};

#[test]
fn determinism_signature_same_input_same_signature() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out1 = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    let out2 = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}

#[test]
fn determinism_signature_changes_when_program_changes() {
    let src_a = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let src_b = r#"
observe("world.degraded", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let p1 = parse_program(src_a, 1).expect("parse should pass");
    typecheck_program(&p1).expect("typecheck should pass");
    let p2 = parse_program(src_b, 1).expect("parse should pass");
    typecheck_program(&p2).expect("typecheck should pass");

    let out1 = execute_program(
        &p1,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    let out2 = execute_program(
        &p2,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    assert_ne!(out1.signature, out2.signature);
}
