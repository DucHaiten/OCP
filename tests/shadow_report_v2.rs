use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 1600,
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
fn shadow_report_v2_has_required_sections_and_is_deterministic() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _policy_allow = EnvVarGuard::set("OCL_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "beam");
    let _max_report = EnvVarGuard::set("OCL_STD_SHADOW_MAX_REPORT_BYTES", "4096");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=beam;variants_json=[{\"score\":20,\"tag\":\"a\"},{\"score\":50,\"tag\":\"b\"},{\"score\":30,\"tag\":\"c\"}];max_branches=3;beam_width=3;top_k=3;rounds=2;per_branch_step_cap=80;per_branch_budget_cap=5000;global_step_cap=240;global_budget_cap=15000;state_score_weight=1;outcome_weight=0;cost_budget_weight=0;cost_steps_weight=0;reason_penalty_weight=0;max_diff_keys=8;max_report_bytes=4096"), budget(5)) -> s;
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
        report.get("schema"),
        Some(&Value::String("shadow.report.v2".to_string()))
    );
    assert_eq!(report.get("truncated"), Some(&Value::Bool(false)));
    assert!(as_int(report.get("report_bytes"), "report.report_bytes") <= 4096);

    let _meta = as_map(report.get("meta").expect("meta"), "report.meta");
    let Some(Value::List(ranking)) = report.get("ranking") else {
        panic!("expected report.ranking list");
    };
    assert!(!ranking.is_empty());
    let Some(Value::List(cost_curve)) = report.get("cost_curve") else {
        panic!("expected report.cost_curve list");
    };
    assert!(!cost_curve.is_empty());
    let Some(Value::List(_reason_table)) = report.get("reason_table") else {
        panic!("expected report.reason_table list");
    };
    let Some(Value::List(_diff_summary)) = report.get("diff_summary") else {
        panic!("expected report.diff_summary list");
    };
}

#[test]
fn shadow_report_v2_truncates_deterministically_when_bytes_cap_is_small() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _policy_allow = EnvVarGuard::set("OCL_STD_SHADOW_SCHEDULER_POLICY_ALLOW", "portfolio");
    let _max_report = EnvVarGuard::set("OCL_STD_SHADOW_MAX_REPORT_BYTES", "120");

    let src = r#"
observe("std.shadow.search", "tier2", ctx("policy=portfolio;variants_json=[{\"score\":10,\"a\":1,\"b\":2,\"c\":3},{\"score\":20,\"a\":9,\"b\":8,\"c\":7},{\"score\":30,\"a\":4,\"b\":5,\"c\":6},{\"score\":40,\"a\":7,\"b\":1,\"c\":2}];max_branches=4;rounds=4;top_k=4;per_branch_step_cap=80;per_branch_budget_cap=5000;global_step_cap=320;global_budget_cap=20000;state_score_weight=1;outcome_weight=0;cost_budget_weight=0;cost_steps_weight=0;reason_penalty_weight=0;max_diff_keys=16;max_report_bytes=120"), budget(5)) -> s;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(s)) = out1.env.get("s") else {
        panic!("expected shadow.search result");
    };
    assert_eq!(s.kind, ResultKind::Degraded);
    assert_eq!(s.reason, Some(ReasonCode::ShadowReportTooLarge));

    let payload = as_map(s.payload.as_ref().expect("payload"), "payload");
    assert_eq!(payload.get("truncated"), Some(&Value::Bool(true)));
    let report = as_map(payload.get("report").expect("report"), "report");
    assert_eq!(
        report.get("schema"),
        Some(&Value::String("shadow.report.v2".to_string()))
    );
    assert_eq!(report.get("truncated"), Some(&Value::Bool(true)));
    assert!(as_int(report.get("report_bytes"), "report.report_bytes") <= 120);
}
