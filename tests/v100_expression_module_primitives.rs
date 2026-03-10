use ocp::ocp::{parse_program, typecheck_program, ErrorCode, ExecConfig, Executor, Value};

#[test]
fn v100_concat_and_comparators_support_module_wiring() {
    let src = r#"
let run_id = "run42";
let event_id = "ev7";
let artifact_path = "./out/" + run_id + "-" + event_id + ".json";
condition(artifact_path == "./out/run42-ev7.json");
condition(run_id == "run42");
condition(ne(run_id, event_id));
condition(lt(1, 2));
condition(le(2, 2));
condition(gt(3, 2));
condition(ge(3, 3));
observe("std.json.parse", "tier2", { raw: "{\"status\":\"OK\"}" }, budget(5)) -> wr;
condition(eq(wr.status, "OK"));
"#;

    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = Executor::new(ExecConfig {
        step_cap: 300,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass");

    assert_eq!(
        out.env.get("artifact_path"),
        Some(&Value::String("./out/run42-ev7.json".to_string()))
    );
}

#[test]
fn v100_lt_typecheck_rejects_mixed_types() {
    let src = r#"
let bad = lt("1", 2);
condition(bad);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should fail");
    assert_eq!(err.code, ErrorCode::TTypeMismatch);
}
