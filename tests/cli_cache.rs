use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_cache_v13_{tag}_{stamp}"))
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

fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn first_artifact_run_dir(root: &Path) -> PathBuf {
    let artifacts_root = root.join(".ocp_artifacts");
    let mut runs = fs::read_dir(&artifacts_root)
        .expect("read .ocp_artifacts")
        .filter_map(|entry| entry.ok().map(|v| v.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    runs.sort();
    runs.into_iter()
        .next()
        .expect("expected at least one run dir in .ocp_artifacts")
}

#[test]
fn cli_cache_stats_reports_perf_counters_json() {
    let root = temp_project_dir("stats");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let run = run_ocp_cli(&["run", &root_s]);
    assert_success(&run);

    let stats = run_ocp_cli(&["cache", "stats", &root_s, "--json"]);
    let stats_json = assert_success(&stats);
    let parsed: JsonValue = serde_json::from_str(&stats_json).expect("stats json");

    let perf_files = parsed
        .get("perf_files")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(
        perf_files >= 1,
        "expected at least one perf_cache file after run"
    );
    assert!(parsed.get("counters").is_some(), "missing counters field");
    assert!(
        parsed.get("compile_cache_entries").is_some(),
        "missing compile_cache_entries field"
    );
}

#[test]
fn cli_cache_clean_requires_yes_flag() {
    let root = temp_project_dir("clean_guard");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let clean = run_ocp_cli(&["cache", "clean", &root_s]);
    assert!(
        !clean.status.success(),
        "cache clean without --yes must fail"
    );
    let stderr = String::from_utf8_lossy(&clean.stderr);
    assert!(
        stderr.contains("--yes"),
        "cache clean guardrail message must mention --yes"
    );
}

#[test]
fn cli_cache_clean_with_yes_removes_cache_dirs() {
    let root = temp_project_dir("clean_yes");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);
    let run = run_ocp_cli(&["run", &root_s]);
    assert_success(&run);

    let artifacts_dir = root.join(".ocp_artifacts");
    assert!(artifacts_dir.exists(), "artifacts must exist before clean");

    let clean = run_ocp_cli(&["cache", "clean", &root_s, "--yes"]);
    assert_success(&clean);

    assert!(
        !artifacts_dir.exists(),
        ".ocp_artifacts must be removed by cache clean --yes"
    );
}

#[test]
fn cli_cache_bench_writes_machine_checkable_report() {
    let root = temp_project_dir("bench");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let bench = run_ocp_cli(&["cache", "bench", &root_s, "--json"]);
    let bench_json = assert_success(&bench);
    let parsed: JsonValue = serde_json::from_str(&bench_json).expect("bench json");

    assert!(
        parsed
            .get("signature_equal")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "cache bench must verify signature equivalence"
    );
    assert!(
        parsed.get("executed_reduction_bps").is_some(),
        "bench report must include deterministic reduction metric"
    );
    assert!(
        parsed.get("no_cache").is_some(),
        "bench report must include explicit no_cache run section"
    );

    let report_path = root
        .join("target")
        .join("ocp")
        .join("v13")
        .join("cache_benchmark.json");
    assert!(
        fs::metadata(report_path).is_ok(),
        "bench command must write cache_benchmark.json"
    );
}

#[test]
fn cli_cache_replay_supports_legacy_main_oc_with_warning() {
    let root = temp_project_dir("legacy_main_oc");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let manifest_path = root.join("Ocp.toml");
    let mut manifest = fs::read_to_string(&manifest_path).expect("read manifest");
    manifest.push_str("\n[project]\nentry = \"src/main.oc\"\n");
    fs::write(&manifest_path, manifest).expect("write manifest entry");

    fs::rename(
        root.join("src").join("main.ocp"),
        root.join("src").join("main.oc"),
    )
    .expect("rename main.ocp -> main.oc");

    let run = run_ocp_cli(&["run", &root_s]);
    assert_success(&run);
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        run_stderr.contains("W-LEGACY-OCP-EXTENSION"),
        "legacy .oc run must emit deprecation warning, got: {run_stderr}"
    );

    let run_dir = first_artifact_run_dir(&root);
    let run_dir_s = run_dir.to_string_lossy().to_string();
    let replay = run_ocp_cli(&["replay", &run_dir_s]);
    assert_success(&replay);

    let stats = run_ocp_cli(&["cache", "stats", &root_s, "--json"]);
    let stats_json = assert_success(&stats);
    let stats_obj: JsonValue = serde_json::from_str(&stats_json).expect("stats json");
    assert!(
        stats_obj.get("compile_cache_entries").is_some(),
        "cache stats must include compile cache counters in legacy mode"
    );

    let clean = run_ocp_cli(&["cache", "clean", &root_s, "--yes"]);
    assert_success(&clean);
    assert!(
        !root.join(".ocp_artifacts").exists(),
        ".ocp_artifacts must be removed by cache clean in legacy mode"
    );
}
