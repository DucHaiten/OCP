use ocp::ocp::{parse_program, typecheck_program, ExecConfig, Executor, ResultKind, Value};

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 200,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn std_time_tick_info_and_now_logical_default_to_deterministic_values() {
    let src = r#"
observe("std.time.tick_info", "tier2", ctx("scope=clock"), budget(5)) -> ti;
observe("std.time.now_logical", "tier2", ctx("scope=clock"), budget(5)) -> now;
"#;

    let out = run_program(src);

    let Some(Value::Result4(ti)) = out.env.get("ti") else {
        panic!("expected tick_info result");
    };
    assert_eq!(ti.kind, ResultKind::Ok);
    match &ti.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("tick"), Some(&"0".to_string()));
            assert_eq!(map.get("dt_ms"), Some(&"16".to_string()));
        }
        _ => panic!("expected tick_info payload"),
    }

    let Some(Value::Result4(now)) = out.env.get("now") else {
        panic!("expected now_logical result");
    };
    assert_eq!(now.kind, ResultKind::Ok);
    match &now.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("t"), Some(&"0".to_string()));
        }
        _ => panic!("expected now_logical payload"),
    }
}

#[test]
fn std_time_now_logical_uses_tick_and_dt_ms_from_ctx() {
    let src = r#"
observe("std.time.tick_info", "tier2", ctx("tick=7;dt_ms=20"), budget(5)) -> ti;
observe("std.time.now_logical", "tier2", ctx("tick=7;dt_ms=20"), budget(5)) -> now;
"#;

    let out = run_program(src);

    let Some(Value::Result4(ti)) = out.env.get("ti") else {
        panic!("expected tick_info result");
    };
    match &ti.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("tick"), Some(&"7".to_string()));
            assert_eq!(map.get("dt_ms"), Some(&"20".to_string()));
        }
        _ => panic!("expected tick_info payload"),
    }

    let Some(Value::Result4(now)) = out.env.get("now") else {
        panic!("expected now_logical result");
    };
    match &now.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("t"), Some(&"140".to_string()));
        }
        _ => panic!("expected now_logical payload"),
    }
}

#[test]
fn std_time_signature_is_stable_for_same_program_and_config() {
    let src = r#"
observe("std.time.tick_info", "tier2", ctx("tick=3;dt_ms=10"), budget(5)) -> ti;
observe("std.time.now_logical", "tier2", ctx("tick=3;dt_ms=10"), budget(5)) -> now;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);

    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}
