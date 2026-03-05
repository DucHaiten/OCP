use std::fs;
use std::path::PathBuf;
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
    std::env::temp_dir().join(format!("ocl_cli_cache_v13_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocl-cli")
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

#[test]
fn cli_cache_stats_reports_perf_counters_json() {
    let root = temp_project_dir("stats");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let run = run_ocl_cli(&["run", &root_s]);
    assert_success(&run);

    let stats = run_ocl_cli(&["cache", "stats", &root_s, "--json"]);
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

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let clean = run_ocl_cli(&["cache", "clean", &root_s]);
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

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);
    let run = run_ocl_cli(&["run", &root_s]);
    assert_success(&run);

    let artifacts_dir = root.join(".ocl_artifacts");
    assert!(artifacts_dir.exists(), "artifacts must exist before clean");

    let clean = run_ocl_cli(&["cache", "clean", &root_s, "--yes"]);
    assert_success(&clean);

    assert!(
        !artifacts_dir.exists(),
        ".ocl_artifacts must be removed by cache clean --yes"
    );
}

#[test]
fn cli_cache_bench_writes_machine_checkable_report() {
    let root = temp_project_dir("bench");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let bench = run_ocl_cli(&["cache", "bench", &root_s, "--json"]);
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
        .join("ocl")
        .join("v13")
        .join("cache_benchmark.json");
    assert!(
        fs::metadata(report_path).is_ok(),
        "bench command must write cache_benchmark.json"
    );
}
