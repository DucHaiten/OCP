use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
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
fn shadow_beam_is_deterministic_and_uses_state_score_priority() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCP_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_STEP_CAP", "80");
    let _budget_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _policy_allow = EnvVarGuard::set("OCP_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "beam");
    let _beam_width_max = EnvVarGuard::set("OCP_STD_SHADOW_BEAM_WIDTH_MAX", "8");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=beam;variants_json=[{\"score\":10},{\"score\":90},{\"score\":80},{\"score\":-10}];max_branches=4;beam_width=3;top_k=3;outcome_weight=0;cost_budget_weight=0;cost_steps_weight=0;reason_penalty_weight=0;state_score_weight=1"), budget(5)) -> s;
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
        Some(&Value::String("beam".to_string()))
    );
    assert_eq!(
        report.get("tie_break"),
        Some(&Value::String(
            "score_desc_steps_charged_asc_branch_id_asc".to_string()
        ))
    );

    let Some(Value::List(topk)) = payload.get("topk") else {
        panic!("expected topk list");
    };
    assert_eq!(topk.len(), 3);
    let top0 = as_map(&topk[0], "topk[0]");
    let top1 = as_map(&topk[1], "topk[1]");
    let top2 = as_map(&topk[2], "topk[2]");
    assert_eq!(top0.get("id"), Some(&Value::Int(1)));
    assert_eq!(top1.get("id"), Some(&Value::Int(2)));
    assert_eq!(top2.get("id"), Some(&Value::Int(0)));
}

#[test]
fn shadow_beam_ranking_is_sorted_by_locked_comparator() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCP_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_STEP_CAP", "90");
    let _budget_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");
    let _policy_allow = EnvVarGuard::set("OCP_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "beam");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=beam;variants_json=[{\"score\":0},{\"score\":0},{\"score\":0},{\"score\":0}];max_branches=4;beam_width=4;top_k=4;outcome_weight=0;state_score_weight=0;reason_penalty_weight=0;cost_budget_weight=0;cost_steps_weight=1"), budget(5)) -> s;
"#;
    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert_eq!(s.kind, ResultKind::Ok);
    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    let Some(Value::List(ranking)) = payload.get("ranking") else {
        panic!("expected ranking");
    };
    assert!(!ranking.is_empty());
    for pair in ranking.windows(2) {
        let left = as_map(&pair[0], "ranking.left");
        let right = as_map(&pair[1], "ranking.right");
        let left_score = as_int(left.get("score"), "left.score");
        let right_score = as_int(right.get("score"), "right.score");
        let left_steps = as_int(left.get("cost_steps"), "left.cost_steps");
        let right_steps = as_int(right.get("cost_steps"), "right.cost_steps");
        let left_id = as_int(left.get("branch_id"), "left.branch_id");
        let right_id = as_int(right.get("branch_id"), "right.branch_id");

        assert!(
            left_score > right_score
                || (left_score == right_score && left_steps < right_steps)
                || (left_score == right_score && left_steps == right_steps && left_id <= right_id),
            "ranking comparator violation: left=({left_score},{left_steps},{left_id}) right=({right_score},{right_steps},{right_id})"
        );
    }
}

#[test]
fn shadow_beam_policy_denied_when_not_in_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _policy_allow = EnvVarGuard::set("OCP_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "round_robin");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=beam;variants_json=[{\"score\":1}]"), budget(5)) -> s;
"#;
    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert_eq!(s.kind, ResultKind::Deferred);
    assert_eq!(s.reason, Some(ReasonCode::ShadowPolicyDenied));
}
