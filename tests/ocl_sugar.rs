use ocp_ocl::ocp_ocl::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, GuardMode, Value,
};

#[test]
fn sugar_try_else_signature_matches_canonical_on_ok_path() {
    let sugar = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> rs;
let x = try rs else { 0 };
condition(true);
"#;
    let canonical = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> rs;
match rs {
  OK => { let x = rs?; }
  DEGRADED => { let x = rs?; }
  INSUFFICIENT => { let x = 0; }
  DEFERRED => { let x = 0; }
}
condition(true);
"#;

    let ps = parse_program(sugar, 1).expect("sugar parse");
    typecheck_program(&ps).expect("sugar typecheck");
    let pc = parse_program(canonical, 1).expect("canonical parse");
    typecheck_program(&pc).expect("canonical typecheck");

    let out_s = execute_program(
        &ps,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("sugar exec");
    let out_c = execute_program(
        &pc,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("canonical exec");
    assert_eq!(out_s.signature, out_c.signature);
}

#[test]
fn sugar_guard_signature_matches_canonical_on_deferred_path() {
    let sugar = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> rs;
guard rs;
condition(true);
"#;
    let canonical = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> rs;
match rs {
  OK => { }
  DEGRADED => { }
  INSUFFICIENT => { return rs; }
  DEFERRED => { return rs; }
}
condition(true);
"#;

    let ps = parse_program(sugar, 1).expect("sugar parse");
    typecheck_program(&ps).expect("sugar typecheck");
    let pc = parse_program(canonical, 1).expect("canonical parse");
    typecheck_program(&pc).expect("canonical typecheck");

    let out_s = execute_program(
        &ps,
        ExecConfig {
            step_cap: 500,
            guard_mode: GuardMode::Return,
            ..ExecConfig::default()
        },
    )
    .expect("sugar exec");
    let out_c = execute_program(
        &pc,
        ExecConfig {
            step_cap: 500,
            guard_mode: GuardMode::Return,
            ..ExecConfig::default()
        },
    )
    .expect("canonical exec");
    assert_eq!(out_s.signature, out_c.signature);
}

#[test]
fn sugar_try_else_implicit_r_reason_code_available() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> rs;
let msg = try rs else { r.reason_code };
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    assert_eq!(
        out.env.get("msg"),
        Some(&Value::String("RC-NOT-IMPLEMENTED".to_string()))
    );
}

#[test]
fn sugar_typecheck_guard_requires_result4() {
    let src = r#"
let x = 1;
guard x;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TGuardNotResult4);
}

#[test]
fn sugar_typecheck_try_else_requires_result4() {
    let src = r#"
let x = 1;
let y = try x else { 0 };
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTryNotResult4);
}

#[test]
fn sugar_guard_mode_error_raises_x_guard_failed() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> rs;
guard rs;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            guard_mode: GuardMode::Error,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail in guard_mode=error");
    assert_eq!(err.code, ErrorCode::XGuardFailed);
}
