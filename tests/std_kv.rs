use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

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

fn temp_kv_path(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ocp_std_kv_{tag}_{stamp}.json"))
}

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 400,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn std_kv_put_commit_then_get_roundtrip() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let kv_path = temp_kv_path("roundtrip");
    let kv_path_str = kv_path.to_string_lossy().to_string();
    let _kv_path_guard = EnvVarGuard::set("OCP_STD_KV_PATH", &kv_path_str);

    let src = r#"
observe("std.kv.put", "tier2", ctx("key=app.answer;value_json=42;overwrite=true"), budget(5)) -> put_r;
commit(put_r);
observe("std.kv.get", "tier2", ctx("key=app.answer"), budget(5)) -> get_r;
"#;
    let out = run_program(src);

    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "std.kv.put");
    assert_eq!(out.commits[0].kind, ResultKind::Ok);

    let Some(Value::Result4(get_r)) = out.env.get("get_r") else {
        panic!("expected get result");
    };
    assert_eq!(get_r.kind, ResultKind::Ok);
    match &get_r.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("found"), Some(&Value::Bool(true)));
            assert_eq!(map.get("value"), Some(&Value::Int(42)));
        }
        _ => panic!("expected map payload"),
    }
}

#[test]
fn std_kv_keys_returns_sorted_and_degraded_when_truncated() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let kv_path = temp_kv_path("keys");
    let kv_path_str = kv_path.to_string_lossy().to_string();
    let _kv_path_guard = EnvVarGuard::set("OCP_STD_KV_PATH", &kv_path_str);

    let src = r#"
observe("std.kv.put", "tier2", ctx("key=app.c;value=3;overwrite=true"), budget(5)) -> p1;
commit(p1);
observe("std.kv.put", "tier2", ctx("key=app.a;value=1;overwrite=true"), budget(5)) -> p2;
commit(p2);
observe("std.kv.put", "tier2", ctx("key=app.b;value=2;overwrite=true"), budget(5)) -> p3;
commit(p3);
observe("std.kv.keys", "tier2", ctx("cap=2"), budget(5)) -> k;
"#;
    let out = run_program(src);

    let Some(Value::Result4(k)) = out.env.get("k") else {
        panic!("expected keys result");
    };
    assert_eq!(k.kind, ResultKind::Degraded);
    assert_eq!(k.reason, Some(ReasonCode::KvCapExceeded));
    match &k.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(keys)) = map.get("keys") else {
                panic!("expected keys list");
            };
            assert_eq!(
                keys,
                &vec![
                    Value::String("app.a".to_string()),
                    Value::String("app.b".to_string())
                ]
            );
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected keys map payload"),
    }
}

#[test]
fn std_kv_put_deferred_when_value_exceeds_cap() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let kv_path = temp_kv_path("cap");
    let kv_path_str = kv_path.to_string_lossy().to_string();
    let _kv_path_guard = EnvVarGuard::set("OCP_STD_KV_PATH", &kv_path_str);
    let _kv_max_guard = EnvVarGuard::set("OCP_STD_KV_MAX_VALUE_BYTES", "8");

    let src = r#"
observe("std.kv.put", "tier2", ctx("key=app.long;value=0123456789"), budget(5)) -> put_r;
"#;
    let out = run_program(src);
    let Some(Value::Result4(put_r)) = out.env.get("put_r") else {
        panic!("expected put result");
    };
    assert_eq!(put_r.kind, ResultKind::Deferred);
    assert_eq!(put_r.reason, Some(ReasonCode::KvCapExceeded));
}

#[test]
fn std_kv_signature_is_stable_with_same_program_and_state() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let kv_path = temp_kv_path("determinism");
    let kv_path_str = kv_path.to_string_lossy().to_string();
    let _kv_path_guard = EnvVarGuard::set("OCP_STD_KV_PATH", &kv_path_str);

    let src = r#"
observe("std.kv.put", "tier2", ctx("key=app.flag;value=ok;overwrite=true"), budget(5)) -> p;
commit(p);
observe("std.kv.get", "tier2", ctx("key=app.flag"), budget(5)) -> g;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}
