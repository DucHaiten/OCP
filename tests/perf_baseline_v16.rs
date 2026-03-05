use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_ocl::ocp_ocl::{compile_with_cache, CompileCacheKeyInput};
use serde_json::{json, Value as JsonValue};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn compile_cache_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    PathBuf::from("target")
        .join("ocl")
        .join("tests")
        .join("perf_baseline_v16")
        .join(format!("compile_{stamp}"))
}

fn base_key_input() -> CompileCacheKeyInput {
    CompileCacheKeyInput {
        compiler_version: "compiler-v16".to_string(),
        ocl_version: "ocl-v0.16".to_string(),
        lane_literal: "locked_v071".to_string(),
        lock_hash: "lock_hash_v16_perf".to_string(),
        trust_hash: "trust_hash_v16_perf".to_string(),
        deps_graph_hash: "deps_hash_v16_perf".to_string(),
        manifest_hash: "manifest_hash_v16_perf".to_string(),
        schema_versions_of_packs: "schema_v16_perf".to_string(),
        entry_module_hash: "entry_hash_v16_perf".to_string(),
        source_bundle_hash: "source_hash_v16_perf".to_string(),
        cache_toggle_inputs: vec!["OCL_CACHE_PROFILE=default".to_string()],
    }
}

fn json_f64(value: &JsonValue, path: &str) -> f64 {
    value
        .pointer(path)
        .and_then(JsonValue::as_f64)
        .unwrap_or_else(|| panic!("missing or invalid float at json pointer `{path}`"))
}

fn json_u64(value: &JsonValue, path: &str) -> u64 {
    value
        .pointer(path)
        .and_then(JsonValue::as_u64)
        .unwrap_or_else(|| panic!("missing or invalid u64 at json pointer `{path}`"))
}

#[test]
fn v16_perf_baseline_thresholds_hold_against_v015_sot() {
    let root = repo_root();
    let baseline_path = root
        .join("baselines")
        .join("v015")
        .join("perf_budget_report.json");
    let baseline_raw = fs::read_to_string(&baseline_path).expect("read baseline perf report");
    let baseline: JsonValue =
        serde_json::from_str(&baseline_raw).expect("parse baseline perf report json");
    assert_eq!(
        baseline
            .get("baseline_id")
            .and_then(JsonValue::as_str)
            .unwrap_or_default(),
        "v0.15-line",
        "baseline id must be pinned to v0.15-line"
    );

    let bench = run_ocl_cli(&[
        "cache",
        "bench",
        "projects/ocp-ocl/apps/hello-cli",
        "--json",
    ]);
    let bench_json = assert_success(&bench);
    let bench_value: JsonValue = serde_json::from_str(&bench_json).expect("parse cache bench json");
    assert!(
        bench_value
            .get("signature_equal")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "cache bench must keep signature invariant"
    );

    let current_steps_executed_total = json_u64(&bench_value, "/warm/exec_node_evals_executed");
    let warm_exec_hits = json_u64(&bench_value, "/warm/exec_cache_hits");
    let warm_exec_misses = json_u64(&bench_value, "/warm/exec_cache_misses");
    let current_compile_hit_ratio = if warm_exec_hits + warm_exec_misses == 0 {
        0.0
    } else {
        (warm_exec_hits as f64 * 100.0) / (warm_exec_hits + warm_exec_misses) as f64
    };

    let src = r#"
let payload = { "k": 1 };
observe("world.exists", "tier2", ctx("scene=v16"), budget(5)) -> r;
"#;
    let key_input = base_key_input();
    let cache_root = compile_cache_root();
    let cold = compile_with_cache(src, 1, &key_input, &cache_root).expect("cold compile");
    let warm = compile_with_cache(src, 1, &key_input, &cache_root).expect("warm compile");
    let current_modules_compiled_count =
        [cold.hit, warm.hit].into_iter().filter(|hit| !*hit).count() as u64;

    let baseline_steps = json_u64(&baseline, "/warm_cache/steps_executed_total");
    let baseline_modules = json_u64(&baseline, "/warm_cache/modules_compiled_count");
    let baseline_hit_ratio = json_f64(&baseline, "/warm_cache/compile_cache_hit_ratio");

    let steps_limit = baseline_steps as f64 * 1.10;
    let modules_limit = baseline_modules as f64 * 1.10;
    let hit_ratio_floor = baseline_hit_ratio - 10.0;

    assert!(
        (current_steps_executed_total as f64) <= steps_limit,
        "steps_executed_total regression too high: current={} baseline={} limit={}",
        current_steps_executed_total,
        baseline_steps,
        steps_limit
    );
    assert!(
        (current_modules_compiled_count as f64) <= modules_limit,
        "modules_compiled_count regression too high: current={} baseline={} limit={}",
        current_modules_compiled_count,
        baseline_modules,
        modules_limit
    );
    assert!(
        current_compile_hit_ratio >= hit_ratio_floor,
        "compile_cache_hit_ratio regression too high: current={} baseline={} floor={}",
        current_compile_hit_ratio,
        baseline_hit_ratio,
        hit_ratio_floor
    );

    let out_dir = root.join("target").join("ocl").join("w16").join("perf");
    fs::create_dir_all(&out_dir).expect("create w16 perf output dir");

    let perf_budget_report = json!({
        "schema": "ocl.w16.perf.budget.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "baseline_ref": "baselines/v015/perf_budget_report.json",
        "baseline_id": "v0.15-line",
        "mode": "warm_cache",
        "metrics": {
            "steps_executed_total": current_steps_executed_total,
            "modules_compiled_count": current_modules_compiled_count,
            "compile_cache_hit_ratio": current_compile_hit_ratio
        },
        "thresholds": {
            "steps_executed_total_max": steps_limit,
            "modules_compiled_count_max": modules_limit,
            "compile_cache_hit_ratio_min": hit_ratio_floor
        },
        "pass": true
    });
    fs::write(
        out_dir.join("perf_budget_report.json"),
        serde_json::to_string_pretty(&perf_budget_report).expect("serialize perf_budget_report"),
    )
    .expect("write perf_budget_report.json");

    let perf_trend_report = json!({
        "schema": "ocl.w16.perf.trend.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "baseline_ref": "baselines/v015/perf_budget_report.json",
        "baseline_id": "v0.15-line",
        "mode": "warm_cache",
        "deltas": {
            "steps_executed_total_delta": (current_steps_executed_total as i64) - (baseline_steps as i64),
            "steps_executed_total_delta_percent": if baseline_steps == 0 { 0.0 } else { ((current_steps_executed_total as f64 - baseline_steps as f64) * 100.0) / baseline_steps as f64 },
            "modules_compiled_count_delta": (current_modules_compiled_count as i64) - (baseline_modules as i64),
            "modules_compiled_count_delta_percent": if baseline_modules == 0 { 0.0 } else { ((current_modules_compiled_count as f64 - baseline_modules as f64) * 100.0) / baseline_modules as f64 },
            "compile_cache_hit_ratio_delta_points": current_compile_hit_ratio - baseline_hit_ratio
        },
        "status": "PASS"
    });
    fs::write(
        out_dir.join("perf_trend_vs_v015.json"),
        serde_json::to_string_pretty(&perf_trend_report).expect("serialize perf trend report"),
    )
    .expect("write perf_trend_vs_v015.json");
}
