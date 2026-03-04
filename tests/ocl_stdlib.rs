use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, CapabilityRegistry, ExecConfig, Executor, ReasonCode,
    ResultKind, Value,
};
use std::net::TcpListener;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn env_serial_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

#[derive(Debug)]
struct EnvVarGuard {
    key: String,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key: key.to_string(),
            prev,
        }
    }

    fn remove(key: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::remove_var(key);
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.prev {
            std::env::set_var(&self.key, prev);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}

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

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
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

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
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

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
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

    let err = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
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

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "std.log.info");
    assert_eq!(out.commits[0].kind, ResultKind::Ok);
}

#[test]
fn registry_enforces_w7_ctx_required() {
    let reg = CapabilityRegistry::v1_baseline();
    assert!(reg
        .check_observe("std.tls.connect", "host=example.com;port=443")
        .is_ok());

    let err = reg
        .check_observe("std.db.exec", "sql=UPDATE t SET n=1")
        .expect_err("std.db.exec must require dsn");
    assert_eq!(err.to_reason_code(), ReasonCode::CtxInvalid);
}

#[test]
fn exec_std_tls_handshake_returns_ok_payload() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _unset_local_real = EnvVarGuard::remove("OCL_W7_TLS_LOCAL_REAL");
    let _unset_timeout = EnvVarGuard::remove("OCL_W7_TLS_TIMEOUT_MS");

    let src = r#"
observe("std.tls.connect", "tier2", ctx("host=example.com;port=443"), budget(5)) -> conn;
observe("std.tls.handshake", "tier2", ctx("conn=example.com:443"), budget(5)) -> hs;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");
    let Some(Value::Result4(r)) = out.env.get("hs") else {
        panic!("expected handshake result");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    match &r.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("protocol"), Some(&"TLS1.3".to_string()));
            assert!(map.contains_key("transcript_hash256"));
        }
        _ => panic!("expected handshake payload"),
    }
}

#[test]
fn exec_std_db_query_is_read_only_and_exec_is_commit_gated() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _unset_db_local_real = EnvVarGuard::remove("OCL_W7_DB_LOCAL_REAL");

    let src = r#"
observe("std.db.query_int", "tier2", ctx("dsn=file:demo.db;sql=SELECT count(*) FROM counters"), budget(5)) -> q;
observe("std.db.exec", "tier2", ctx("dsn=file:demo.db;sql=UPDATE counters SET n=1"), budget(5)) -> w;
commit(w);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");

    let Some(Value::Result4(query_res)) = out.env.get("q") else {
        panic!("expected query result");
    };
    assert_eq!(query_res.kind, ResultKind::Ok);
    match &query_res.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("value"), Some(&"1".to_string()));
        }
        _ => panic!("expected query payload"),
    }

    let Some(Value::Result4(write_res)) = out.env.get("w") else {
        panic!("expected write result");
    };
    assert_eq!(write_res.kind, ResultKind::Ok);
    match &write_res.payload {
        Some(Value::Payload(map)) => {
            assert!(map.contains_key("pending_write_id"));
            assert_eq!(map.get("applied"), Some(&"false".to_string()));
        }
        _ => panic!("expected write payload"),
    }

    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "std.db.exec");
}

#[test]
fn exec_commit_std_db_query_int_is_forbidden() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _unset_db_local_real = EnvVarGuard::remove("OCL_W7_DB_LOCAL_REAL");

    let src = r#"
observe("std.db.query_int", "tier2", ctx("dsn=file:demo.db;sql=SELECT 1"), budget(5)) -> q;
commit(q);
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let err = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect_err("commit on std.db.query_int must be denied by policy");
    assert_eq!(err.code.as_str(), "X-COMMIT-FORBIDDEN");
}

#[test]
fn exec_std_ui_and_game_tick_info_return_ctx_tick() {
    let src = r#"
observe("std.ui.frame_info", "tier2", ctx("ctx_tick=42"), budget(5)) -> ui;
observe("std.game.tick_info", "tier2", ctx("ctx_tick=42"), budget(5)) -> game;
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out = Executor::new(ExecConfig {
        step_cap: 100,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");

    let Some(Value::Result4(ui_res)) = out.env.get("ui") else {
        panic!("expected ui result");
    };
    let Some(Value::Result4(game_res)) = out.env.get("game") else {
        panic!("expected game result");
    };

    match &ui_res.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("ctx_tick"), Some(&"42".to_string()));
        }
        _ => panic!("expected ui payload"),
    }
    match &game_res.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("ctx_tick"), Some(&"42".to_string()));
        }
        Some(Value::Map(map)) => {
            assert_eq!(map.get("ctx_tick"), Some(&Value::Int(42)));
        }
        _ => panic!("expected game payload"),
    }
}

#[test]
fn exec_w7_local_real_sqlite_apply_once() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _db_local_real = EnvVarGuard::set("OCL_W7_DB_LOCAL_REAL", "1");

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis();
    let dsn = format!("file:target/ocl/w7/local_real_{stamp}.db");
    let src = format!(
        concat!(
            "observe(\"std.db.exec\", \"tier2\", ctx(\"dsn={dsn};sql=CREATE TABLE IF NOT EXISTS counters(n INTEGER)\"), budget(5)) -> w0;\n",
            "commit(w0);\n",
            "observe(\"std.db.exec\", \"tier2\", ctx(\"dsn={dsn};sql=INSERT INTO counters(n) VALUES (1)\"), budget(5)) -> w1;\n",
            "commit(w1);\n",
            "commit(w1);\n",
            "observe(\"std.db.query_int\", \"tier2\", ctx(\"dsn={dsn};sql=SELECT count(*) FROM counters\"), budget(5)) -> q;\n"
        ),
        dsn = dsn
    );

    let p = parse_program(&src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = Executor::new(ExecConfig {
        step_cap: 1000,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");

    let Some(Value::Result4(r)) = out.env.get("q") else {
        panic!("expected query result");
    };
    match &r.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("value"), Some(&"1".to_string()));
            assert_eq!(map.get("mode"), Some(&"local_real".to_string()));
        }
        _ => panic!("expected query payload"),
    }
}

#[test]
fn exec_w7_local_real_tls_probe() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _tls_local_real = EnvVarGuard::set("OCL_W7_TLS_LOCAL_REAL", "1");
    let _tls_timeout = EnvVarGuard::set("OCL_W7_TLS_TIMEOUT_MS", "2000");

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let port = listener
        .local_addr()
        .expect("local addr")
        .port()
        .to_string();
    listener.set_nonblocking(false).expect("set blocking mode");

    let handler = thread::spawn(move || {
        for _ in 0..2 {
            let _ = listener.accept();
        }
    });

    let src = format!(
        concat!(
            "observe(\"std.tls.connect\", \"tier2\", ctx(\"host=127.0.0.1;port={port}\"), budget(5)) -> c;\n",
            "observe(\"std.tls.handshake\", \"tier2\", ctx(\"conn=127.0.0.1:{port}\"), budget(5)) -> h;\n"
        ),
        port = port
    );

    let p = parse_program(&src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");
    let out = Executor::new(ExecConfig {
        step_cap: 1000,
        ..ExecConfig::default()
    })
    .run(&p)
    .expect("exec should pass");

    let Some(Value::Result4(h)) = out.env.get("h") else {
        panic!("expected handshake result");
    };
    match &h.payload {
        Some(Value::Payload(map)) => {
            assert_eq!(map.get("mode"), Some(&"local_real".to_string()));
            assert_eq!(map.get("protocol"), Some(&"TLS1.3".to_string()));
            assert!(map.contains_key("transcript_hash256"));
        }
        _ => panic!("expected handshake payload"),
    }

    let _ = handler.join();
    thread::sleep(Duration::from_millis(20));
}
