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

#[test]
fn engine_game_run_is_deterministic_and_returns_rng_state_payload() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_GAME_ENABLED", "1");
    let _seed = EnvVarGuard::set("OCL_STD_GAME_BASE_SEED", "123");
    let _streams = EnvVarGuard::set("OCL_STD_GAME_RNG_STREAMS", "main,loot");
    let _count_cap = EnvVarGuard::set("OCL_STD_GAME_RNG_MAX_COUNT", "8");

    let src = r#"
observe("engine.game.run", "tier2", ctx("entry_module=game.main;tick=4;stream=main;count=3;state_json={\"score\":2};draw_list=text:1,1,score,12"), budget(5)) -> r;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(result)) = out1.env.get("r") else {
        panic!("expected engine.game.run result");
    };
    assert_eq!(result.kind, ResultKind::Ok);
    let Some(Value::Map(map)) = &result.payload else {
        panic!("expected map payload");
    };
    assert_eq!(
        map.get("entry_module"),
        Some(&Value::String("game.main".to_string()))
    );
    assert_eq!(map.get("tick"), Some(&Value::Int(4)));
    assert_eq!(map.get("stream"), Some(&Value::String("main".to_string())));
    assert_eq!(map.get("count"), Some(&Value::Int(3)));

    let Some(Value::List(rng_values)) = map.get("rng_values") else {
        panic!("expected rng_values");
    };
    assert_eq!(rng_values.len(), 3);
    for value in rng_values {
        let Value::Int(v) = value else {
            panic!("rng value must be Int");
        };
        assert!((0_i64..=2_147_483_647_i64).contains(v));
    }
}

#[test]
fn engine_shadow_preview_wraps_run_and_compare_deterministically() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");
    let _max_branches = EnvVarGuard::set("OCL_STD_SHADOW_MAX_BRANCHES", "8");
    let _max_diff = EnvVarGuard::set("OCL_STD_SHADOW_MAX_DIFF_KEYS", "8");

    let src = r#"
observe("engine.shadow.preview", "tier2", ctx("entry_module=game.main;variants_json=[{\"x\":1},{\"x\":2}];baseline_id=0;max_diff_keys=4"), budget(5)) -> p;
"#;

    let out1 = run_program(src);
    let out2 = run_program(src);
    assert_eq!(out1.signature, out2.signature);

    let Some(Value::Result4(result)) = out1.env.get("p") else {
        panic!("expected engine.shadow.preview result");
    };
    assert!(matches!(result.kind, ResultKind::Ok | ResultKind::Degraded));
    let Some(Value::Map(map)) = &result.payload else {
        panic!("expected map payload");
    };
    assert_eq!(
        map.get("entry_module"),
        Some(&Value::String("game.main".to_string()))
    );
    let Some(Value::Map(run_map)) = map.get("run") else {
        panic!("expected run payload");
    };
    let Some(Value::List(branches)) = run_map.get("branches") else {
        panic!("expected branches");
    };
    assert_eq!(branches.len(), 2);

    let Some(Value::Map(compare_map)) = map.get("compare") else {
        panic!("expected compare payload");
    };
    assert!(compare_map.contains_key("cost_table"));
    assert!(compare_map.contains_key("diff_keys"));
}

#[test]
fn engine_shadow_preview_blocks_disallowed_effect_keys() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_SHADOW_ENABLED", "1");

    let src = r#"
observe("engine.shadow.preview", "tier2", ctx("entry_module=game.main;variants_json=[{\"x\":1}];effect_keys=std.fs.write_text"), budget(5)) -> p;
"#;

    let out = run_program(src);
    let Some(Value::Result4(result)) = out.env.get("p") else {
        panic!("expected engine.shadow.preview result");
    };
    assert_eq!(result.kind, ResultKind::Insufficient);
    assert_eq!(result.reason, Some(ReasonCode::ShadowEffectDisallowed));
}
