use ocp_ocl::ocp_ocl::{execute_program, parse_program, typecheck_program, ExecConfig, Value};

#[test]
fn exec_rejects_invalid_ctx_with_r_ctx_invalid() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab;scene=again"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert_eq!(err.code.as_str(), "R-CTX-INVALID");
    assert_eq!(err.root_reason.map(|r| r.as_str()), Some("RC-CTX-INVALID"));
}

#[test]
fn exec_accepts_valid_ctx_pairs() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab;zone=alpha"), budget(5)) -> r;
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

    let out = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(out.env.get("marker"), Some(&Value::Int(1)));
}
