use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{parse_program, typecheck_program, ExecConfig, Executor, ResultKind, Value};

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

fn as_map<'a>(value: &'a Value, label: &str) -> &'a BTreeMap<String, Value> {
    match value {
        Value::Map(map) => map,
        _ => panic!("expected {label} to be map"),
    }
}

fn as_int(value: Option<&Value>, label: &str) -> i64 {
    match value {
        Some(Value::Int(raw)) => *raw,
        _ => panic!("expected {label} to be int"),
    }
}

#[test]
fn shadow_memo_reuses_deterministic_observe_without_changing_outcome() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "5000");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _checkpoint_every = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_EVERY", "5000");
    let _checkpoint_reuse = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_REUSE", "0");
    let _memo_enabled = EnvVarGuard::set("OCL_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE", "1");
    let _memo_key = EnvVarGuard::set("OCL_STD_SHADOW_MEMO_OBSERVE_KEY", "std.game.tick_info");
    let _memo_cost = EnvVarGuard::set("OCL_STD_SHADOW_MEMO_OBSERVE_COST_STEPS", "64");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":1},{\"x\":2}];checkpoint_every=5000"), budget(5)) -> s;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(s)) = out1.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Ok);
    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let memo = as_map(payload.get("memo").expect("memo section"), "memo");
    assert_eq!(memo.get("enabled"), Some(&Value::Bool(true)));
    assert_eq!(
        memo.get("determinism_class"),
        Some(&Value::String("deterministic".to_string()))
    );
    let hits = as_int(memo.get("hits"), "memo.hits");
    let misses = as_int(memo.get("misses"), "memo.misses");
    let charged = as_int(
        memo.get("observe_calls_charged"),
        "memo.observe_calls_charged",
    );
    let executed = as_int(
        memo.get("observe_calls_executed"),
        "memo.observe_calls_executed",
    );
    let saved = as_int(memo.get("memo_saved_steps"), "memo.memo_saved_steps");
    assert!(hits >= 1, "expected memo hit for duplicate variant");
    assert!(misses >= 1, "expected at least one cold observe");
    assert_eq!(charged, 3);
    assert!(
        executed < charged,
        "memo should reduce executed observe calls"
    );
    assert!(saved > 0, "memo should save executed steps");

    let Some(Value::List(branches)) = payload.get("branches") else {
        panic!("expected branches list");
    };
    for (idx, branch) in branches.iter().enumerate() {
        let branch_map = as_map(branch, "branch");
        assert_eq!(
            branch_map.get("outcome"),
            Some(&Value::String("OK".to_string()))
        );
        let cost = as_map(branch_map.get("cost").expect("branch.cost"), "branch.cost");
        let steps = as_int(cost.get("steps"), "cost.steps");
        let charged_steps = as_int(cost.get("steps_charged"), "cost.steps_charged");
        let executed_steps = as_int(cost.get("steps_executed"), "cost.steps_executed");
        assert_eq!(
            steps, charged_steps,
            "branch {idx} uses charged steps as canonical steps"
        );
        assert!(
            executed_steps <= charged_steps,
            "branch {idx} executed <= charged"
        );
    }
}

#[test]
fn shadow_memo_disabled_keeps_executed_and_charged_observe_calls_equal() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "5000");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _checkpoint_every = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_EVERY", "5000");
    let _checkpoint_reuse = EnvVarGuard::set("OCL_STD_SHADOW_CHECKPOINT_REUSE", "0");
    let _memo_enabled = EnvVarGuard::set("OCL_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE", "0");
    let _memo_key = EnvVarGuard::set("OCL_STD_SHADOW_MEMO_OBSERVE_KEY", "std.game.tick_info");
    let _memo_cost = EnvVarGuard::set("OCL_STD_SHADOW_MEMO_OBSERVE_COST_STEPS", "64");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":1},{\"x\":2}]"), budget(5)) -> s;
"#;

    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Ok);
    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let memo = as_map(payload.get("memo").expect("memo section"), "memo");
    assert_eq!(memo.get("enabled"), Some(&Value::Bool(false)));
    assert_eq!(as_int(memo.get("hits"), "memo.hits"), 0);
    assert_eq!(
        as_int(
            memo.get("observe_calls_charged"),
            "memo.observe_calls_charged"
        ),
        as_int(
            memo.get("observe_calls_executed"),
            "memo.observe_calls_executed"
        )
    );
    assert_eq!(
        as_int(memo.get("memo_saved_steps"), "memo.memo_saved_steps"),
        0
    );
}
