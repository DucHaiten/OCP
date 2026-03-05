use ocp_ocl::ocp_ocl::{execute_program, parse_program, typecheck_program, ExecConfig};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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

#[test]
fn cache_on_off_signature_is_identical_in_locked_lane() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");

    let with_cache = {
        let _cache_guard = EnvVarGuard::set("OCL_CACHE_DISABLE", "0");
        execute_program(
            &program,
            ExecConfig {
                step_cap: 500,
                enable_exec_cache: true,
                ..ExecConfig::default()
            },
        )
        .expect("exec with cache enabled should pass")
    };

    let without_cache = {
        let _cache_guard = EnvVarGuard::set("OCL_CACHE_DISABLE", "1");
        execute_program(
            &program,
            ExecConfig {
                step_cap: 500,
                enable_exec_cache: false,
                ..ExecConfig::default()
            },
        )
        .expect("exec with cache disabled should pass")
    };

    assert_eq!(
        with_cache.signature, without_cache.signature,
        "cache on/off must keep signature invariant in locked lane"
    );
    assert_eq!(
        with_cache.trace.events, without_cache.trace.events,
        "cache on/off must keep trace events invariant in locked lane"
    );
    assert_eq!(
        without_cache.exec_cache.hits, 0,
        "no-cache run must not report exec cache hits"
    );
    assert_eq!(
        without_cache.observe_cache.hits, 0,
        "no-cache run must not report observe cache hits"
    );

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w16")
        .join("determinism");
    fs::create_dir_all(&out_dir).expect("create w16 determinism output dir");
    let report = json!({
        "schema": "ocl.w16.determinism.cache_signature_invariance.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "lane": "locked_v071",
        "signature_equal": with_cache.signature == without_cache.signature,
        "trace_equal": with_cache.trace.events == without_cache.trace.events,
        "with_cache": {
            "signature": with_cache.signature,
            "steps": with_cache.steps,
            "exec_cache_hits": with_cache.exec_cache.hits,
            "exec_cache_misses": with_cache.exec_cache.misses,
            "observe_cache_hits": with_cache.observe_cache.hits,
            "observe_cache_misses": with_cache.observe_cache.misses
        },
        "without_cache": {
            "signature": without_cache.signature,
            "steps": without_cache.steps,
            "exec_cache_hits": without_cache.exec_cache.hits,
            "exec_cache_misses": without_cache.exec_cache.misses,
            "observe_cache_hits": without_cache.observe_cache.hits,
            "observe_cache_misses": without_cache.observe_cache.misses
        }
    });
    fs::write(
        out_dir.join("cache_signature_invariance_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize cache invariance report"),
    )
    .expect("write cache_signature_invariance_report.json");
}
