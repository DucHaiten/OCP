use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_std_net_http_{tag}_{stamp}"))
}

fn run_ocp_cli(args: &[&str], quarantine_env: Option<&str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .env_remove("OCP_QUARANTINE");
    if let Some(value) = quarantine_env {
        cmd.env("OCP_QUARANTINE", value);
    }
    cmd.output().expect("run ocp-cli")
}

fn configure_manifest_for_net_http_quarantine(root: &Path) {
    let manifest = root.join("Ocp.toml");
    let raw = fs::read_to_string(&manifest).expect("read Ocp.toml");
    let mut patched = raw.replace("lane = \"locked_v071\"", "lane = \"quarantine\"");
    patched = patched.replace(
        "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\"]",
        "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\", \"std.net.http.*\"]",
    );
    if !patched.contains("[permissions.std_net_http]") {
        patched.push('\n');
        patched.push_str("[permissions.std_net_http]\n");
        patched.push_str("enabled = true\n");
        patched.push_str("allow_hosts = [\"mock.local\"]\n");
        patched.push_str("allow_methods = [\"GET\"]\n");
        patched.push_str("timeout_ms = 3000\n");
        patched.push_str("max_body_bytes = 16\n");
    }
    fs::write(&manifest, patched).expect("write Ocp.toml");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocp_artifacts");
    let read = fs::read_dir(&artifacts_root).expect("read .ocp_artifacts");
    let mut dirs = Vec::new();
    for entry in read {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs.pop().expect("missing run artifact dir")
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

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 500,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn std_net_http_record_and_replay_mock_adapter_with_truncation() {
    let root = temp_project_dir("record_replay");
    let root_s = root.to_string_lossy().to_string();
    let init = run_ocp_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    configure_manifest_for_net_http_quarantine(&root);
    let source = "observe(\"std.net.http.request\", \"tier2\", ctx(\"method=GET;url=http://mock.local/demo;timeout_ms=3000;max_body_bytes=5\"), budget(5)) -> r;\ncondition(true);\n".to_string();
    fs::write(root.join("src").join("main.ocp"), source).expect("write source");

    let run_without = run_ocp_cli(&["run", &root_s], None);
    assert!(
        !run_without.status.success(),
        "quarantine http run without env must fail"
    );

    let run_with = run_ocp_cli(&["run", &root_s], Some("1"));
    assert!(
        run_with.status.success(),
        "run with quarantine env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run_with.stdout),
        String::from_utf8_lossy(&run_with.stderr)
    );

    let run_dir = latest_artifact_dir(&root);
    let cassette = fs::read_to_string(run_dir.join("cassette").join("cassette.jsonl"))
        .expect("read cassette.jsonl");
    assert!(
        cassette.contains("\"cap\":\"std.net.http.request\""),
        "cassette must include std.net.http.request entries"
    );
    assert!(
        cassette.contains("\"call_id\":0"),
        "first http entry must keep call_id=0 ordering"
    );
    assert!(
        cassette.contains("\"status\":\"200\""),
        "http status must be recorded"
    );
    assert!(
        cassette.contains("\"truncated\":\"true\""),
        "http body truncation must be recorded as degraded"
    );

    let replay_without = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        !replay_without.status.success(),
        "replay without quarantine env must fail for lane quarantine"
    );

    let replay_with = run_ocp_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_with.status.success(),
        "replay with quarantine env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_with.stdout),
        String::from_utf8_lossy(&replay_with.stderr)
    );
}

#[test]
fn std_net_http_denies_host_outside_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCP_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCP_QUARANTINE_MODE", "record");
    let _hosts = EnvVarGuard::set("OCP_STD_NET_ALLOW_HOSTS", "allowed.example");
    let _methods = EnvVarGuard::set("OCP_STD_NET_ALLOW_METHODS", "GET");
    let _timeout = EnvVarGuard::set("OCP_STD_NET_TIMEOUT_MS", "3000");
    let _max_body = EnvVarGuard::set("OCP_STD_NET_MAX_BODY_BYTES", "1024");
    let _record = EnvVarGuard::set(
        "OCP_V08_HTTP_RECORD_PATH",
        std::env::temp_dir()
            .join("ocp_std_net_http_record_host.tmp")
            .to_string_lossy()
            .as_ref(),
    );

    let src = r#"
observe("std.net.http.request", "tier2", ctx("method=GET;url=http://127.0.0.1:18080/demo"), budget(5)) -> r;
"#;
    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected net http result binding");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::NetHostDenied));
}

#[test]
fn std_net_http_denies_method_outside_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCP_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCP_QUARANTINE_MODE", "record");
    let _hosts = EnvVarGuard::set("OCP_STD_NET_ALLOW_HOSTS", "127.0.0.1");
    let _methods = EnvVarGuard::set("OCP_STD_NET_ALLOW_METHODS", "GET");
    let _timeout = EnvVarGuard::set("OCP_STD_NET_TIMEOUT_MS", "3000");
    let _max_body = EnvVarGuard::set("OCP_STD_NET_MAX_BODY_BYTES", "1024");
    let _record = EnvVarGuard::set(
        "OCP_V08_HTTP_RECORD_PATH",
        std::env::temp_dir()
            .join("ocp_std_net_http_record_method.tmp")
            .to_string_lossy()
            .as_ref(),
    );

    let src = r#"
observe("std.net.http.request", "tier2", ctx("method=POST;url=http://127.0.0.1:18080/demo"), budget(5)) -> r;
"#;
    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected net http result binding");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::NetMethodDenied));
}
