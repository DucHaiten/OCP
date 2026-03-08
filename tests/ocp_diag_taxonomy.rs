use ocp::ocp::{execute_program, parse_program, typecheck_program, ExecConfig};

#[test]
fn taxonomy_parser_is_p_namespace() {
    let err = parse_program("let x = 1", 1).expect_err("parse should fail");
    assert!(err.code.as_str().starts_with("P-"));
}

#[test]
fn taxonomy_typecheck_is_t_namespace() {
    let p = parse_program("condition(1);", 1).expect("parse should pass");
    let err = typecheck_program(&p).expect_err("typecheck should fail");
    assert!(err.code.as_str().starts_with("T-"));
}

#[test]
fn taxonomy_exec_is_x_namespace() {
    let p = parse_program("condition(false);", 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let err = execute_program(
        &p,
        ExecConfig {
            step_cap: 100,
            ..ExecConfig::default()
        },
    )
    .expect_err("exec should fail");
    assert!(err.code.as_str().starts_with("X-"));
}

#[test]
fn taxonomy_runtime_ctx_is_r_namespace() {
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
    assert!(err.code.as_str().starts_with("R-"));
}
