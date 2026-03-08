use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{parse_program, typecheck_program, ExecConfig, Executor, ResultKind, Value};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_shadow_preview_v12_{tag}_{stamp}"))
}

fn run_ocp_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocp-cli")
}

fn list_dirs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let read = fs::read_dir(path).expect("read dir");
    for entry in read {
        let p = entry.expect("entry").path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn count_project_files(root: &Path) -> usize {
    fn walk(path: &Path, count: &mut usize) {
        let Ok(read) = fs::read_dir(path) else {
            return;
        };
        for entry in read {
            let Ok(entry) = entry else {
                continue;
            };
            let p = entry.path();
            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|n| n == ".ocp_artifacts")
                .unwrap_or(false)
            {
                continue;
            }
            if p.is_dir() {
                walk(&p, count);
            } else if p.is_file() {
                *count += 1;
            }
        }
    }
    let mut total = 0usize;
    walk(root, &mut total);
    total
}

fn env_serial_guard() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    old: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let old = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, old }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(old) = &self.old {
            std::env::set_var(self.key, old);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn as_map<'a>(value: &'a Value, label: &str) -> &'a BTreeMap<String, Value> {
    match value {
        Value::Map(map) => map,
        _ => panic!("expected {label} to be map"),
    }
}

fn as_int(value: Option<&Value>, label: &str) -> i64 {
    match value {
        Some(Value::Int(v)) => *v,
        _ => panic!("expected {label} to be int"),
    }
}

fn run_template_program(root: &Path) -> (i64, i64) {
    let src =
        fs::read_to_string(root.join("src").join("main.ocp")).expect("read template main.ocp");
    let program = parse_program(&src, 1).expect("parse template");
    typecheck_program(&program).expect("typecheck template");
    let out = Executor::new(ExecConfig {
        step_cap: 2000,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("run template");

    let Some(Value::Result4(search)) = out.env.get("search") else {
        panic!("expected binding `search` in template");
    };
    assert!(
        matches!(search.kind, ResultKind::Ok | ResultKind::Degraded),
        "unexpected search kind: {:?}",
        search.kind
    );
    let payload = as_map(
        search.payload.as_ref().expect("search payload"),
        "search.payload",
    );
    let report = as_map(payload.get("report").expect("report"), "report");
    assert_eq!(
        report.get("schema"),
        Some(&Value::String("shadow.report.v2".to_string())),
        "report must carry v2 schema marker"
    );
    assert!(report.contains_key("ranking"), "report.ranking missing");
    assert!(
        report.contains_key("diff_summary"),
        "report.diff_summary missing"
    );
    assert!(
        report.contains_key("cost_curve"),
        "report.cost_curve missing"
    );
    assert!(
        report.contains_key("reason_table"),
        "report.reason_table missing"
    );

    let executed = as_int(
        report.get("global_steps_executed"),
        "report.global_steps_executed",
    );
    let charged = as_int(
        report.get("global_steps_charged"),
        "report.global_steps_charged",
    );
    (executed, charged)
}

#[test]
fn cli_shadow_preview_template_v12_run_replay_and_kpi_pass() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_project_dir("run_replay_kpi");
    let root_str = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_str, "--template", "shadow-preview"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let main_src =
        fs::read_to_string(root.join("src").join("main.ocp")).expect("read generated main.ocp");
    assert!(
        main_src.contains("std.shadow.search"),
        "shadow-preview template must use std.shadow.search in v0.12"
    );
    assert!(root.join("Ocp.toml").exists(), "missing Ocp.toml");
    assert!(
        root.join("src").join("main.ocp").exists(),
        "missing src/main.ocp"
    );
    assert!(root.join("README.md").exists(), "missing README.md");
    assert!(
        count_project_files(&root) <= 7,
        "template file count must be <= 7"
    );

    let run = run_ocp_cli(&["run", &root_str]);
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifacts_root = root.join(".ocp_artifacts");
    assert!(artifacts_root.exists(), "missing .ocp_artifacts");
    let mut run_dirs = list_dirs(&artifacts_root);
    assert!(!run_dirs.is_empty(), "missing run artifact dir");
    let run_dir = run_dirs.pop().expect("latest run dir");
    assert!(run_dir.join("audit.jsonl").exists(), "missing audit.jsonl");
    assert!(
        run_dir.join("signature.txt").exists(),
        "missing signature.txt"
    );
    assert!(run_dir.join("replay.toml").exists(), "missing replay.toml");

    let replay = run_ocp_cli(&["replay", &run_dir.to_string_lossy()]);
    assert!(
        replay.status.success(),
        "replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );

    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _memo_key = EnvVarGuard::set("OCP_STD_SHADOW_MEMO_OBSERVE_KEY", "std.game.tick_info");
    let _memo_cost = EnvVarGuard::set("OCP_STD_SHADOW_MEMO_OBSERVE_COST_STEPS", "64");

    let _memo_off = EnvVarGuard::set("OCP_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE", "0");
    let _reuse_off = EnvVarGuard::set("OCP_STD_SHADOW_CHECKPOINT_REUSE", "0");
    let (baseline_steps_executed, baseline_steps_charged) = run_template_program(&root);
    drop(_reuse_off);
    drop(_memo_off);

    let _memo_on = EnvVarGuard::set("OCP_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE", "1");
    let _reuse_on = EnvVarGuard::set("OCP_STD_SHADOW_CHECKPOINT_REUSE", "1");
    let (reuse_steps_executed, reuse_steps_charged) = run_template_program(&root);

    assert_eq!(
        baseline_steps_charged, reuse_steps_charged,
        "charged steps must remain stable for decision invariance"
    );
    assert!(
        baseline_steps_executed > 0,
        "baseline executed steps must be positive"
    );
    assert!(
        reuse_steps_executed <= baseline_steps_executed,
        "reuse executed steps must not exceed baseline"
    );

    let reduction_percent =
        ((baseline_steps_executed - reuse_steps_executed).max(0) * 100) / baseline_steps_executed;
    assert!(
        reduction_percent >= 30,
        "KPI-2 failed: baseline_steps_executed={baseline_steps_executed}, reuse_steps_executed={reuse_steps_executed}, reduction={reduction_percent}%"
    );
}
