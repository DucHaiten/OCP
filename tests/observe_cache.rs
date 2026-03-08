use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    parse_program, typecheck_program, Cacheability, CapabilityRegistry, DeterminismClass,
    ExecConfig, Executor, Value,
};

fn run_program_with_registry(src: &str, registry: CapabilityRegistry) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::with_registry(
        ExecConfig {
            step_cap: 2_000,
            ..ExecConfig::default()
        },
        registry,
    )
    .run(&program)
    .expect("exec should pass")
}

fn as_payload_map<'a>(value: &'a Value, label: &str) -> &'a BTreeMap<String, Value> {
    match value {
        Value::Map(map) => map,
        _ => panic!("expected {label} map payload"),
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

fn temp_fs_root(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ocp_observe_cache_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dir");
    }
    std::fs::write(path, content).expect("write file");
}

#[test]
fn observe_cache_only_memoizes_keys_marked_within_run() {
    let src = r#"
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> a;
observe("world.exists", "tier2", ctx("scene=lab"), budget(5)) -> b;
"#;

    let mut cacheable_registry = CapabilityRegistry::v1_baseline();
    cacheable_registry
        .set_determinism_class_for_key("world.exists", DeterminismClass::Deterministic);
    cacheable_registry.set_cacheability_for_key("world.exists", Cacheability::WithinRun);
    let cached = run_program_with_registry(src, cacheable_registry);
    assert_eq!(cached.observe_cache.hits, 1);
    assert_eq!(cached.observe_cache.misses, 1);
    assert_eq!(cached.observe_cache.entries, 1);

    let mut no_cache_registry = CapabilityRegistry::v1_baseline();
    no_cache_registry
        .set_determinism_class_for_key("world.exists", DeterminismClass::Deterministic);
    no_cache_registry.set_cacheability_for_key("world.exists", Cacheability::NoCache);
    let uncached = run_program_with_registry(src, no_cache_registry);
    assert_eq!(uncached.observe_cache.hits, 0);
    assert_eq!(uncached.observe_cache.entries, 0);
}

#[test]
fn observe_cache_fs_generation_invalidation_prevents_stale_read() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let root = temp_fs_root("fs_generation");
    write_text(&root.join("data/in.txt"), "one");

    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_READ", "./data/**");
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./data/**");

    let src = r#"
observe("std.fs.read_text", "tier2", ctx("path=./data/in.txt"), budget(5)) -> r1;
observe("std.fs.write_text", "tier2", ctx("path=./data/in.txt;text=two;overwrite=true"), budget(5)) -> w;
commit(w);
observe("std.fs.read_text", "tier2", ctx("path=./data/in.txt"), budget(5)) -> r2;
"#;

    let out = run_program_with_registry(src, CapabilityRegistry::v1_baseline());
    let Some(Value::Result4(r1)) = out.env.get("r1") else {
        panic!("expected r1");
    };
    let Some(Value::Result4(r2)) = out.env.get("r2") else {
        panic!("expected r2");
    };
    let p1 = as_payload_map(r1.payload.as_ref().expect("r1 payload"), "r1");
    let p2 = as_payload_map(r2.payload.as_ref().expect("r2 payload"), "r2");
    assert_eq!(p1.get("text"), Some(&Value::String("one".to_string())));
    assert_eq!(p2.get("text"), Some(&Value::String("two".to_string())));
    assert_eq!(
        out.observe_cache.hits, 0,
        "fs generation change should prevent stale memo hit"
    );
    assert_eq!(
        out.observe_cache.misses, 2,
        "both read_text observes should be evaluated on their own generation"
    );
}
