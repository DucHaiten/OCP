use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    parse_conformance_manifest_v1, run_conformance_v1, ConformanceManifestV1,
    ConformanceRunOptionsV1,
};
use serde_json::Value as JsonValue;

fn conformance_v5_path() -> &'static Path {
    Path::new("projects/ocp-ocl/conformance/conformance.v5.toml")
}

fn cache_manifest_subset() -> ConformanceManifestV1 {
    let parsed =
        parse_conformance_manifest_v1(conformance_v5_path()).expect("parse conformance v5");
    let scenarios = parsed
        .scenarios
        .into_iter()
        .filter(|s| s.name.starts_with("cache-"))
        .collect::<Vec<_>>();
    assert!(
        !scenarios.is_empty(),
        "cache scenarios must exist in conformance.v5"
    );
    ConformanceManifestV1 {
        schema: parsed.schema,
        scenarios,
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

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_conformance_cache_v14_{tag}_{stamp}"))
}

#[test]
fn conformance_v5_includes_cache_scenarios() {
    let manifest = cache_manifest_subset();
    let names = manifest
        .scenarios
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"cache-core-exec-pass"));
}

#[test]
fn conformance_cache_subset_preserves_signature_with_and_without_cache() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let manifest = cache_manifest_subset();
    let with_cache = {
        let _cache_guard = EnvVarGuard::set("OCL_CACHE_DISABLE", "0");
        run_conformance_v1(
            Path::new("."),
            &manifest,
            ConformanceRunOptionsV1::default(),
        )
    };
    assert_eq!(
        with_cache.scenarios_failed, 0,
        "cache subset should pass when cache is enabled: {:?}",
        with_cache.results
    );

    let without_cache = {
        let _cache_guard = EnvVarGuard::set("OCL_CACHE_DISABLE", "1");
        run_conformance_v1(
            Path::new("."),
            &manifest,
            ConformanceRunOptionsV1::default(),
        )
    };
    assert_eq!(
        without_cache.scenarios_failed, 0,
        "cache subset should pass when cache is disabled: {:?}",
        without_cache.results
    );
    assert_eq!(
        with_cache.required_digest, without_cache.required_digest,
        "cache on/off must preserve semantic conformance digest"
    );
}

#[test]
fn conformance_cache_bench_reports_non_semantic_cache_telemetry() {
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
        "cache bench must keep semantic signature invariant"
    );

    let warm = parsed.get("warm").expect("warm section exists");
    let no_cache = parsed.get("no_cache").expect("no_cache section exists");
    assert_eq!(
        warm.get("required_digest"),
        no_cache.get("required_digest"),
        "warm and no_cache runs must keep required digest equal"
    );

    let warm_exec_misses = warm
        .get("exec_cache_misses")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let warm_observe_misses = warm
        .get("observe_cache_misses")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(
        warm_exec_misses > 0 || warm_observe_misses > 0,
        "warm run must expose cache telemetry counters"
    );

    assert_eq!(
        no_cache
            .get("exec_cache_hits")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0),
        0,
        "no_cache run must not report cache hits"
    );

    let report_path = root
        .join("target")
        .join("ocl")
        .join("v13")
        .join("cache_benchmark.json");
    assert!(
        report_path.exists(),
        "cache bench must emit non-semantic telemetry artifact"
    );
}
