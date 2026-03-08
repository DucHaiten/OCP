use std::sync::{Mutex, OnceLock};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, TraceEvent,
    Value,
};

fn run_program(src: &str) -> ocp::ocp::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 400,
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
fn std_game_tick_info_uses_fixed_dt_and_tick() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_GAME_ENABLED", "1");
    let _dt = EnvVarGuard::set("OCP_STD_GAME_FIXED_DT_MS", "20");

    let src = r#"
observe("std.game.tick_info", "tier2", ctx("tick=7"), budget(5)) -> g;
"#;

    let out = run_program(src);
    let Some(Value::Result4(game)) = out.env.get("g") else {
        panic!("expected game result");
    };
    assert_eq!(game.kind, ResultKind::Ok);
    match &game.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("tick"), Some(&Value::Int(7)));
            assert_eq!(map.get("dt_ms"), Some(&Value::Int(20)));
            assert_eq!(map.get("ctx_tick"), Some(&Value::Int(7)));
        }
        _ => panic!("expected map payload"),
    }
}

#[test]
fn std_game_rng_is_deterministic_and_emits_trace() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_GAME_ENABLED", "1");
    let _seed = EnvVarGuard::set("OCP_STD_GAME_BASE_SEED", "99");
    let _streams = EnvVarGuard::set("OCP_STD_GAME_RNG_STREAMS", "main,loot");
    let _count_cap = EnvVarGuard::set("OCP_STD_GAME_RNG_MAX_COUNT", "8");

    let src = r#"
observe("std.game.rng", "tier2", ctx("stream=main;count=4;tick=9"), budget(5)) -> r;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(r1)) = out1.env.get("r") else {
        panic!("expected rng result");
    };
    let Some(Value::Result4(r2)) = out2.env.get("r") else {
        panic!("expected rng result");
    };
    assert_eq!(r1.kind, ResultKind::Ok);
    assert_eq!(r2.kind, ResultKind::Ok);
    assert_eq!(r1.payload, r2.payload);

    match &r1.payload {
        Some(Value::Map(map)) => {
            let Some(Value::List(values)) = map.get("values") else {
                panic!("expected values list");
            };
            assert_eq!(values.len(), 4);
            for value in values {
                let Value::Int(v) = value else {
                    panic!("expected integer rng value");
                };
                assert!((0_i64..=2_147_483_647_i64).contains(v));
            }
        }
        _ => panic!("expected map payload"),
    }

    assert!(
        out1.trace.events.iter().any(|ev| matches!(
            ev,
            TraceEvent::GameObserve {
                key,
                stream,
                value_count,
                tick,
                kind,
                reason,
            } if key == "std.game.rng"
                && stream == "main"
                && *value_count == 4
                && *tick == 9
                && *kind == ResultKind::Ok
                && reason.is_none()
        )),
        "expected GameObserve trace for std.game.rng"
    );
}

#[test]
fn std_game_rng_invalid_stream_returns_insufficient() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_GAME_ENABLED", "1");
    let _streams = EnvVarGuard::set("OCP_STD_GAME_RNG_STREAMS", "main,loot");

    let src = r#"
observe("std.game.rng", "tier2", ctx("stream=bad;count=2;tick=1"), budget(5)) -> r;
"#;

    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected rng result");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::GameRngInvalidStream));
}

#[test]
fn std_game_state_delta_commit_is_idempotent_for_trace_apply() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_GAME_ENABLED", "1");
    let _cap = EnvVarGuard::set("OCP_STD_GAME_STATE_DELTA_MAX_BYTES", "256");

    let src = r#"
observe("std.game.state_delta", "tier2", ctx("idempotency_key=turn-1;delta=hp+1"), budget(5)) -> d;
commit(d);
commit(d);
"#;

    let out = run_program(src);
    let Some(Value::Result4(d)) = out.env.get("d") else {
        panic!("expected state_delta result");
    };
    assert_eq!(d.kind, ResultKind::Ok);

    assert_eq!(out.commits.len(), 2);
    assert_eq!(out.commits[0].key, "std.game.state_delta");
    assert_eq!(out.commits[1].key, "std.game.state_delta");

    let game_commit_events = out
        .trace
        .events
        .iter()
        .filter(
            |ev| matches!(ev, TraceEvent::GameCommit { key, .. } if key == "std.game.state_delta"),
        )
        .count();
    assert_eq!(
        game_commit_events, 1,
        "same state_delta commit should apply once per pending_write_id"
    );
}

#[test]
fn std_game_state_delta_exceed_cap_returns_deferred() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCP_STD_GAME_ENABLED", "1");
    let _cap = EnvVarGuard::set("OCP_STD_GAME_STATE_DELTA_MAX_BYTES", "8");

    let src = r#"
observe("std.game.state_delta", "tier2", ctx("idempotency_key=turn-2;delta=abcdefghijklmnop"), budget(5)) -> d;
"#;

    let out = run_program(src);
    let Some(Value::Result4(d)) = out.env.get("d") else {
        panic!("expected state_delta result");
    };
    assert_eq!(d.kind, ResultKind::Deferred);
    assert_eq!(d.reason, Some(ReasonCode::LimitExceeded));
}
