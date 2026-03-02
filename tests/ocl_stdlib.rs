use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, CapabilityRegistry, ExecConfig, Executor, ReasonCode,
    ResultKind, Value,
};

#[test]
fn registry_enforces_std_ctx_required() {
    let reg = CapabilityRegistry::v1_baseline();
    assert!(reg.check_observe("std.fs.read", "path=notes.txt").is_ok());

    let err = reg
        .check_observe("std.fs.read", "mode=demo")
        .expect_err("std.fs.read must require path");
    assert_eq!(err.to_reason_code(), ReasonCode::CtxInvalid);
}

#[test]
fn exec_std_fs_read_returns_ok_payload() {
    let src = r#"
observe("std.fs.read", "tier2", ctx("path=notes.txt"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4 binding");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    assert_eq!(r.reason, None);
    match &r.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("path"), Some(&"notes.txt".to_string()));
            assert_eq!(map.get("content"), Some(&"sample:notes.txt".to_string()));
        }
        _ => panic!("expected payload map"),
    }
}

#[test]
fn exec_std_http_post_returns_degraded() {
    let src = r#"
observe("std.http.post", "tier2", ctx("url=https://example.com;body={}"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4 binding");
    };
    assert_eq!(r.kind, ResultKind::Degraded);
    assert_eq!(r.reason, Some(ReasonCode::AdapterFailed));
}

#[test]
fn exec_std_json_parse_returns_ok_payload() {
    let src = r#"
observe("std.json.parse", "tier2", ctx("raw={\"a\":1}"), budget(5)) -> r;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result4 binding");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    match &r.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("parsed"), Some(&"true".to_string()));
        }
        _ => panic!("expected payload map"),
    }
}

#[test]
fn exec_std_time_sleep_is_deferred_and_commit_forbidden() {
    let src = r#"
observe("std.time.sleep", "tier2", ctx("ms=10"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let err = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect_err("commit on deferred result must fail");
    assert_eq!(err.code.as_str(), "X-COMMIT-FORBIDDEN");
}

#[test]
fn exec_std_log_info_commit_records_event() {
    let src = r#"
observe("std.log.info", "tier2", ctx("message=hello"), budget(5)) -> r;
commit(r);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig { step_cap: 100 })
        .run(&p)
        .expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "std.log.info");
    assert_eq!(out.commits[0].kind, ResultKind::Ok);
}
