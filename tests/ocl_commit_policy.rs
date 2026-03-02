use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, CapabilityRegistry, ExecConfig, Executor, ResultKind, Value,
};

#[test]
fn commit_ok_observe_passes_and_records_commit_event() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "world.ok");
    assert_eq!(out.commits[0].kind, ResultKind::Ok);
}

#[test]
fn commit_degraded_observe_passes_and_records_commit_event() {
    let src = r#"
observe("world.degraded", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "world.degraded");
    assert_eq!(out.commits[0].kind, ResultKind::Degraded);
}

#[test]
fn commit_policy_denied_by_key_fails() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let mut reg = CapabilityRegistry::v1_baseline();
    reg.set_commit_allowed_for_key("world.ok", false);
    let err = Executor::with_registry(ExecConfig { step_cap: 100 }, reg)
        .run(&p)
        .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), "X-COMMIT-FORBIDDEN");
}

#[test]
fn match_selects_ok_arm_for_world_ok_observe() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
let marker = 0;
match r {
  OK => { let marker = 1; }
  DEGRADED => { let marker = 2; }
  INSUFFICIENT => { let marker = 3; }
  DEFERRED => { let marker = 4; }
}
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    assert_eq!(out.env.get("marker"), Some(&Value::Int(1)));
}
