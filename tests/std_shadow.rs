use std::sync::{Mutex, OnceLock};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, TraceEvent,
    Value,
};

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
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

#[test]
fn std_shadow_run_is_deterministic_and_emits_trace() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCP_STD_SHADOW_MAX_BRANCHES", "8");
    let _step_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_STEP_CAP", "5000");
    let _budget_cap = EnvVarGuard::set("OCP_STD_SHADOW_BRANCH_BUDGET_CAP", "200000");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}]"), budget(5)) -> s;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(s)) = out1.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Ok);
    match &s.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(branches)) = map.get("branches") else {
                panic!("expected branches list");
            };
            assert_eq!(branches.len(), 3);
            assert_eq!(map.get("truncated"), Some(&Value::Bool(false)));
        }
        _ => panic!("expected map payload"),
    }

    assert!(
        out1.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::ShadowRun {
                key,
                branch_count,
                truncated,
                kind,
                reason,
                ..
            } if key == "std.shadow.run"
                && *branch_count == 3
                && !*truncated
                && *kind == ResultKind::Ok
                && reason.is_none()
        )),
        "expected ShadowRun trace"
    );
}

#[test]
fn std_shadow_run_truncates_when_branch_cap_exceeded() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCP_STD_SHADOW_MAX_BRANCHES", "2");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}]"), budget(5)) -> s;
"#;
    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Degraded);
    assert_eq!(s.reason, Some(ReasonCode::ShadowCapExceeded));
    match &s.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(branches)) = map.get("branches") else {
                panic!("expected branches list");
            };
            assert_eq!(branches.len(), 2);
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected map payload"),
    }
}

#[test]
fn std_shadow_run_blocks_disallowed_effect_keys() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");

    let src = r#"
observe("std.shadow.run", "tier2", ctx("variants_json=[{\"x\":1}];effect_keys=std.fs.write_text|std.ui.present"), budget(5)) -> s;
"#;
    let out = run_program(src);
    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected shadow.run result");
    };
    assert_eq!(s.kind, ResultKind::Insufficient);
    assert_eq!(s.reason, Some(ReasonCode::ShadowEffectDisallowed));
}

#[test]
fn std_shadow_compare_is_bounded_and_deterministic() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_diff = EnvVarGuard::set("OCP_STD_SHADOW_MAX_DIFF_KEYS", "1");
    let _max_report = EnvVarGuard::set("OCP_STD_SHADOW_MAX_REPORT_BYTES", "256");

    let src = r#"
observe("std.shadow.compare", "tier2", ctx("branches_json=[{\"id\":0,\"outcome\":\"OK\",\"signature\":\"a\",\"cost\":{\"steps\":1,\"budget\":2},\"state_summary\":{\"a\":1,\"b\":1,\"c\":1}},{\"id\":1,\"outcome\":\"OK\",\"signature\":\"b\",\"cost\":{\"steps\":2,\"budget\":3},\"state_summary\":{\"a\":1,\"b\":2,\"c\":3}}];baseline_id=0;max_diff_keys=1"), budget(5)) -> c;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(c)) = out1.env.get("c") else {
        panic!("expected shadow.compare result");
    };
    assert_eq!(c.kind, ResultKind::Degraded);
    assert_eq!(c.reason, Some(ReasonCode::ShadowCapExceeded));
    match &c.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(diff_keys)) = map.get("diff_keys") else {
                panic!("expected diff_keys");
            };
            assert_eq!(diff_keys.len(), 1);
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected compare payload"),
    }

    assert!(
        out1.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::ShadowCompare {
                key,
                diff_count,
                kind,
                ..
            } if key == "std.shadow.compare"
                && *diff_count == 1
                && *kind == ResultKind::Degraded
        )),
        "expected ShadowCompare trace"
    );
}

#[test]
fn std_shadow_compare_degraded_when_report_too_large() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_SHADOW_ENABLED", "1");
    let _max_diff = EnvVarGuard::set("OCP_STD_SHADOW_MAX_DIFF_KEYS", "10");
    let _max_report = EnvVarGuard::set("OCP_STD_SHADOW_MAX_REPORT_BYTES", "120");

    let src = r#"
observe("std.shadow.compare", "tier2", ctx("branches_json=[{\"id\":0,\"outcome\":\"OK\",\"signature\":\"aaaaaaaaaaaaaaaa\",\"cost\":{\"steps\":1,\"budget\":2},\"state_summary\":{\"k1\":1,\"k2\":2,\"k3\":3}},{\"id\":1,\"outcome\":\"OK\",\"signature\":\"bbbbbbbbbbbbbbbb\",\"cost\":{\"steps\":20,\"budget\":300},\"state_summary\":{\"k1\":9,\"k2\":8,\"k3\":7}}];baseline_id=0"), budget(5)) -> c;
"#;
    let out = run_program(src);
    let Some(Value::Result4(c)) = out.env.get("c") else {
        panic!("expected shadow.compare result");
    };
    assert_eq!(c.kind, ResultKind::Degraded);
    assert_eq!(c.reason, Some(ReasonCode::ShadowReportTooLarge));
    match &c.payload {
        Some(Value::Map(map)) => {
            let Some(Value::Int(bytes)) = map.get("report_bytes") else {
                panic!("expected report_bytes");
            };
            assert!(*bytes <= 120);
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected compare payload"),
    }
}
