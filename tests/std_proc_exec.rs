use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_ocl::ocp_ocl::{
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
    std::env::temp_dir().join(format!("ocl_cli_std_proc_exec_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str], quarantine_env: Option<&str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .env_remove("OCL_QUARANTINE");
    if let Some(value) = quarantine_env {
        cmd.env("OCL_QUARANTINE", value);
    }
    cmd.output().expect("run ocl-cli")
}

fn configure_manifest_for_proc_quarantine(root: &Path) {
    let manifest = root.join("Ocl.toml");
    let raw = fs::read_to_string(&manifest).expect("read Ocl.toml");
    let mut patched = raw.replace("lane = \"locked_v071\"", "lane = \"quarantine\"");
    patched = patched.replace(
        "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\"]",
        "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\", \"std.proc.*\"]",
    );
    if !patched.contains("[permissions.std_proc]") {
        patched.push('\n');
        patched.push_str("[permissions.std_proc]\n");
        patched.push_str("enabled = true\n");
        patched.push_str("allow_bins = [\"python\"]\n");
        patched.push_str("timeout_ms = 3000\n");
        patched.push_str("max_stdout_bytes = 64\n");
        patched.push_str("max_stderr_bytes = 64\n");
    }
    fs::write(&manifest, patched).expect("write Ocl.toml");
}

fn latest_artifact_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocl_artifacts");
    let read = fs::read_dir(&artifacts_root).expect("read .ocl_artifacts");
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

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
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
fn std_proc_exec_record_and_replay_with_nonzero_and_truncation() {
    let root = temp_project_dir("record_replay");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "tool-cli"], None);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    configure_manifest_for_proc_quarantine(&root);

    let source = r#"observe("std.proc.exec", "tier2", ctx("bin=python;args=-c,raise SystemExit(7);timeout_ms=3000"), budget(5)) -> p1;
observe("std.proc.exec", "tier2", ctx("bin=python;args=-c,print('ABCDEFGHIJ');timeout_ms=3000;max_stdout_bytes=4"), budget(5)) -> p2;
condition(true);
"#;
    fs::write(root.join("src").join("main.ocl"), source).expect("write source");

    let run_without = run_ocl_cli(&["run", &root_s], None);
    assert!(
        !run_without.status.success(),
        "quarantine proc run without env must fail"
    );

    let run_with = run_ocl_cli(&["run", &root_s], Some("1"));
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
        cassette.contains("\"cap\":\"std.proc.exec\""),
        "cassette must include std.proc.exec entries"
    );
    assert!(
        cassette.contains("\"call_id\":0") && cassette.contains("\"call_id\":1"),
        "proc entries must keep deterministic call_id ordering"
    );
    assert!(
        cassette.contains("\"exit_code\":\"7\""),
        "nonzero exit_code must be recorded"
    );
    assert!(
        cassette.contains("\"truncated\":\"true\""),
        "truncated proc output must be recorded as degraded"
    );

    let replay_without = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], None);
    assert!(
        !replay_without.status.success(),
        "replay without quarantine env must fail for lane quarantine"
    );

    let replay_with = run_ocl_cli(&["replay", &run_dir.to_string_lossy()], Some("1"));
    assert!(
        replay_with.status.success(),
        "replay with quarantine env failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay_with.stdout),
        String::from_utf8_lossy(&replay_with.stderr)
    );
}

#[test]
fn std_proc_exec_denies_bin_outside_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _lane = EnvVarGuard::set("OCL_PROJECT_LANE", "quarantine");
    let _mode = EnvVarGuard::set("OCL_QUARANTINE_MODE", "record");
    let _allow = EnvVarGuard::set("OCL_STD_PROC_ALLOW_BINS", "python");
    let _timeout = EnvVarGuard::set("OCL_STD_PROC_TIMEOUT_MS", "2000");
    let _max_out = EnvVarGuard::set("OCL_STD_PROC_MAX_STDOUT_BYTES", "1024");
    let _max_err = EnvVarGuard::set("OCL_STD_PROC_MAX_STDERR_BYTES", "1024");

    let tmp_record = std::env::temp_dir().join("ocl_std_proc_record_guard.tmp");
    let _record = EnvVarGuard::set(
        "OCL_V08_PROC_RECORD_PATH",
        tmp_record.to_string_lossy().as_ref(),
    );

    let src = r#"
observe("std.proc.exec", "tier2", ctx("bin=definitely_not_allowed_bin;args=--version"), budget(5)) -> p;
"#;
    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("p") else {
        panic!("expected proc result binding");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::ProcBinDenied));
}
