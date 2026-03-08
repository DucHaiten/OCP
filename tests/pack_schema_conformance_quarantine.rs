use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    parse_program, typecheck_program, validate_schema_value, CapabilityRegistry, ExecConfig,
    Executor, ReasonCode, ResultKind, Value,
};

fn run_with_registry(src: &str, registry: CapabilityRegistry) -> (CapabilityRegistry, Value) {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    let out = Executor::with_registry(
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
        registry.clone(),
    )
    .run(&program)
    .expect("exec should pass");
    let result = out.env.get("r").cloned().expect("binding r must exist");
    (registry, result)
}

fn payload_to_schema_value(payload: &Value) -> Value {
    match payload {
        Value::Payload(map) => {
            let mut out = BTreeMap::new();
            for (k, v) in map {
                out.insert(k.clone(), Value::String(v.clone()));
            }
            Value::Map(out)
        }
        other => other.clone(),
    }
}

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

fn temp_record_file(tag: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir()
        .join(format!("ocp_schema_quarantine_{tag}_{stamp}.jsonl"))
        .to_string_lossy()
        .to_string()
}

#[test]
fn registry_has_ctx_and_payload_schema_for_quarantine_pack_keys() {
    let reg = CapabilityRegistry::v1_baseline();
    let keys_with_payload = [
        "std.time.wallclock.now",
        "std.proc.exec",
        "std.net.http.request",
    ];

    for key in keys_with_payload {
        assert!(
            reg.ctx_schema_for_key(key).is_some(),
            "missing ctx schema for {key}"
        );
        assert!(
            reg.payload_schema_for_key(key).is_some(),
            "missing payload schema for {key}"
        );
    }
}

#[test]
fn runtime_ctx_schema_rejects_invalid_dynamic_key_ctx_for_quarantine_pack() {
    let src = r#"
let k = "std.net.http.request";
observe(k, "tier2", ctx("method=GET;url=http://mock.local/demo;timeout_ms=bad-int"), budget(5)) -> r;
"#;
    let (_reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::CtxInvalid));
}

#[test]
fn runtime_payload_schema_matches_std_time_wallclock_payload() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCP_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCP_QUARANTINE_MODE", "record");
    let record_path = temp_record_file("wallclock");
    let _record = EnvVarGuard::set("OCP_V08_WALLCLOCK_RECORD_PATH", &record_path);

    let src = r#"
observe("std.time.wallclock.now", "tier2", ctx("scope=tool"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.time.wallclock.now")
        .expect("payload schema");
    let value = payload_to_schema_value(payload);
    validate_schema_value(payload_schema, &value).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_proc_exec_payload() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCP_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCP_QUARANTINE_MODE", "record");
    let _allow = EnvVarGuard::set("OCP_STD_PROC_ALLOW_BINS", "mock.proc");
    let _timeout = EnvVarGuard::set("OCP_STD_PROC_TIMEOUT_MS", "3000");
    let _max_out = EnvVarGuard::set("OCP_STD_PROC_MAX_STDOUT_BYTES", "1024");
    let _max_err = EnvVarGuard::set("OCP_STD_PROC_MAX_STDERR_BYTES", "1024");
    let record_path = temp_record_file("proc");
    let _record = EnvVarGuard::set("OCP_V08_PROC_RECORD_PATH", &record_path);

    let src = r#"
observe("std.proc.exec", "tier2", ctx("bin=mock.proc;args=--stdout=HELLO"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert!(matches!(r.kind, ResultKind::Ok | ResultKind::Degraded));
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.proc.exec")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}

#[test]
fn runtime_payload_schema_matches_std_net_http_payload() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCP_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCP_QUARANTINE_MODE", "record");
    let _allow_hosts = EnvVarGuard::set("OCP_STD_NET_ALLOW_HOSTS", "mock.local");
    let _allow_methods = EnvVarGuard::set("OCP_STD_NET_ALLOW_METHODS", "GET");
    let _timeout = EnvVarGuard::set("OCP_STD_NET_TIMEOUT_MS", "3000");
    let _body_cap = EnvVarGuard::set("OCP_STD_NET_MAX_BODY_BYTES", "256");
    let record_path = temp_record_file("net");
    let _record = EnvVarGuard::set("OCP_V08_NET_HTTP_RECORD_PATH", &record_path);

    let src = r#"
observe("std.net.http.request", "tier2", ctx("method=GET;url=http://mock.local/demo;max_body_bytes=16"), budget(5)) -> r;
"#;
    let (reg, result) = run_with_registry(src, CapabilityRegistry::v1_baseline());
    let Value::Result4(r) = result else {
        panic!("expected Result4");
    };
    assert!(matches!(r.kind, ResultKind::Ok | ResultKind::Degraded));
    let payload = r.payload.as_ref().expect("payload");
    let payload_schema = reg
        .payload_schema_for_key("std.net.http.request")
        .expect("payload schema");
    validate_schema_value(payload_schema, payload).expect("payload should satisfy schema");
}
