use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 1200,
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
        Some(Value::Int(v)) => *v,
        _ => panic!("expected {label} to be int"),
    }
}

#[test]
fn shadow_portfolio_is_deterministic_and_emits_strategy_sequence() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "80");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _policy_allow = EnvVarGuard::set("OCL_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "portfolio");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=portfolio;variants_json=[{\"score\":30},{\"score\":10},{\"score\":60},{\"score\":40}];max_branches=4;rounds=4;top_k=3;per_branch_step_cap=80;per_branch_budget_cap=5000;global_step_cap=320;global_budget_cap=20000;outcome_weight=0;cost_budget_weight=0;cost_steps_weight=0;reason_penalty_weight=0;state_score_weight=1"), budget(5)) -> s;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(s)) = out1.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert_eq!(s.kind, ResultKind::Ok);

    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let report = as_map(payload.get("report").expect("report"), "report");
    assert_eq!(
        report.get("policy"),
        Some(&Value::String("portfolio".to_string()))
    );
    assert_eq!(
        report.get("tie_break"),
        Some(&Value::String(
            "score_desc_steps_charged_asc_branch_id_asc".to_string()
        ))
    );
    let Some(Value::List(strategies)) = report.get("strategies") else {
        panic!("expected report.strategies");
    };
    assert!(!strategies.is_empty());
    assert_eq!(
        strategies[0],
        Value::String("outcome_first".to_string()),
        "first strategy must follow fixed order"
    );

    let best = as_map(payload.get("best").expect("best"), "best");
    assert_eq!(
        best.get("id"),
        Some(&Value::Int(2)),
        "highest state score should win in this setup"
    );
}

#[test]
fn shadow_portfolio_respects_max_rounds_cap() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_STEP_CAP", "80");
    let _budget_cap = EnvVarGuard::set("OCL_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _max_rounds = EnvVarGuard::set("OCL_STD_SHADOW_MAX_ROUNDS", "2");
    let _policy_allow = EnvVarGuard::set("OCL_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "portfolio");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=portfolio;variants_json=[{\"score\":20},{\"score\":30},{\"score\":40}];max_branches=3;rounds=10;top_k=3;per_branch_step_cap=80;per_branch_budget_cap=5000;global_step_cap=240;global_budget_cap=12000"), budget(5)) -> s;
"#;

    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert!(matches!(s.kind, ResultKind::Ok | ResultKind::Degraded));

    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let report = as_map(payload.get("report").expect("report"), "report");
    assert_eq!(as_int(report.get("rounds"), "report.rounds"), 2);
    let Some(Value::List(cost_curve)) = report.get("cost_curve") else {
        panic!("expected report.cost_curve");
    };
    assert!(cost_curve.len() <= 2);
}

#[test]
fn shadow_portfolio_policy_denied_when_not_in_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _policy_allow =
        EnvVarGuard::set("OCL_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "round_robin,beam");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=portfolio;variants_json=[{\"score\":1}]"), budget(5)) -> s;
"#;

    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert_eq!(s.kind, ResultKind::Deferred);
    assert_eq!(s.reason, Some(ReasonCode::ShadowPolicyDenied));
}
