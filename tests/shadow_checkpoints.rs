use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 800,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
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

fn as_map<'a>(value: &'a Value, label: &str) -> &'a std::collections::BTreeMap<String, Value> {
    match value {
        Value::Map(map) => map,
        _ => panic!("expected {label} to be map"),
    }
}

#[test]
fn shadow_checkpoint_resume_digest_matches_full_digest() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "5000");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _checkpoint_every = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_EVERY", "5000");
    let _checkpoint_cache_cap =
        EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_CACHE_MAX_ENTRIES", "64");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":1}];checkpoint_every=5000"), budget(5)) -> s;
"#;

    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Ok);

    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let Some(Value::String(prefix_key)) = payload.get("prefix_key") else {
        panic!("expected prefix_key");
    };
    assert!(!prefix_key.is_empty());
    assert_eq!(payload.get("checkpoint_every"), Some(&Value::Int(5000)));

    let checkpoint_cache = as_map(
        payload
            .get("checkpoint_cache")
            .expect("checkpoint_cache field"),
        "checkpoint_cache",
    );
    let Some(Value::Int(hits)) = checkpoint_cache.get("hits") else {
        panic!("expected checkpoint_cache.hits");
    };
    assert!(
        *hits >= 1,
        "expected at least one checkpoint hit for duplicate variant"
    );

    let Some(Value::List(branches)) = payload.get("branches") else {
        panic!("expected branches");
    };
    assert_eq!(branches.len(), 2);

    let second_branch = as_map(&branches[1], "branch[1]");
    let state_summary = as_map(
        second_branch
            .get("state_summary")
            .expect("state_summary in branch"),
        "state_summary",
    );
    let Some(Value::String(full_digest)) = state_summary.get("state_digest_full") else {
        panic!("expected state_digest_full");
    };
    let Some(Value::String(resumed_digest)) = state_summary.get("state_digest_resumed") else {
        panic!("expected state_digest_resumed");
    };
    assert_eq!(full_digest, resumed_digest);

    let Some(Value::Int(resume_index)) = state_summary.get("checkpoint_resume_index") else {
        panic!("expected checkpoint_resume_index");
    };
    assert!(
        *resume_index > 0,
        "duplicate variant should resume from existing checkpoint"
    );
}

#[test]
fn shadow_checkpoint_cache_is_bounded_and_truncated() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "5000");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _checkpoint_every = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_EVERY", "1");
    let _checkpoint_cache_cap =
        EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_CACHE_MAX_ENTRIES", "1");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}];checkpoint_every=1"), budget(5)) -> s;
"#;
    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Degraded);
    assert_eq!(s.reason, Some(ReasonCode::ShadowCapExceeded));

    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    assert_eq!(payload.get("truncated"), Some(&Value::Bool(true)));
    let checkpoint_cache = as_map(
        payload
            .get("checkpoint_cache")
            .expect("checkpoint_cache field"),
        "checkpoint_cache",
    );
    assert_eq!(checkpoint_cache.get("truncated"), Some(&Value::Bool(true)));
    let Some(Value::Int(entries)) = checkpoint_cache.get("entries") else {
        panic!("expected checkpoint_cache.entries");
    };
    let Some(Value::Int(cap)) = checkpoint_cache.get("cap") else {
        panic!("expected checkpoint_cache.cap");
    };
    assert!(*entries <= *cap);
}
