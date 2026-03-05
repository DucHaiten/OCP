use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, Cacheability, CapabilityRegistry, ExecConfig, Executor,
};

fn run_with_registry(src: &str, registry: CapabilityRegistry) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::with_registry(
        ExecConfig {
            step_cap: 1_200,
            ..ExecConfig::default()
        },
        registry,
    )
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
fn ui_incremental_cache_hits_for_repeated_engine_ui_frame() {
    let src = r#"
observe("engine.ui.run", "tier2", ctx("entry_module=app.main;phase=frame;tick=9;dt_ms=16;w=800;h=600;scale=2;theme=retro;locale=vi-VN;input_cap=4;events=key:W|text:go;draw_cap=4;draw_list=text:1,1,go,12|rect:0,0,10,10,#fff;state_json={\"hp\":10}"), budget(5)) -> a;
observe("engine.ui.run", "tier2", ctx("entry_module=app.main;phase=frame;tick=9;dt_ms=16;w=800;h=600;scale=2;theme=retro;locale=vi-VN;input_cap=4;events=key:W|text:go;draw_cap=4;draw_list=text:1,1,go,12|rect:0,0,10,10,#fff;state_json={\"hp\":10}"), budget(5)) -> b;
"#;

    let out = run_with_registry(src, CapabilityRegistry::v1_baseline());
    assert!(
        out.observe_cache.hits >= 1,
        "expected observe cache hit for repeated engine.ui.run"
    );
    assert!(
        out.observe_cache.misses >= 1,
        "expected initial miss before cache hit"
    );
    assert!(
        out.observe_cache.entries >= 1,
        "expected at least one cached entry"
    );

    let Some(ocp_ocl::ocp_ocl::Value::Result4(a)) = out.env.get("a") else {
        panic!("binding a must be Result4");
    };
    let Some(ocp_ocl::ocp_ocl::Value::Result4(b)) = out.env.get("b") else {
        panic!("binding b must be Result4");
    };
    assert_eq!(a.kind, b.kind, "kind must stay identical");
    assert_eq!(a.reason, b.reason, "reason must stay identical");
    assert_eq!(
        a.payload, b.payload,
        "reused frame payload must match cold frame payload"
    );
}

#[test]
fn game_incremental_cache_keeps_signature_equal_when_cache_disabled() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _enabled = EnvVarGuard::set("OCL_STD_GAME_ENABLED", "1");
    let _seed = EnvVarGuard::set("OCL_STD_GAME_BASE_SEED", "321");
    let _streams = EnvVarGuard::set("OCL_STD_GAME_RNG_STREAMS", "main,loot");
    let _count_cap = EnvVarGuard::set("OCL_STD_GAME_RNG_MAX_COUNT", "8");

    let src = r#"
observe("engine.game.run", "tier2", ctx("entry_module=game.main;tick=4;stream=main;count=3;state_json={\"score\":2};draw_list=text:1,1,score,12"), budget(5)) -> a;
observe("engine.game.run", "tier2", ctx("entry_module=game.main;tick=4;stream=main;count=3;state_json={\"score\":2};draw_list=text:1,1,score,12"), budget(5)) -> b;
"#;

    let cached_out = run_with_registry(src, CapabilityRegistry::v1_baseline());
    assert!(
        cached_out.observe_cache.hits >= 1,
        "expected observe cache hit for repeated engine.game.run"
    );

    let mut no_cache_registry = CapabilityRegistry::v1_baseline();
    no_cache_registry.set_cacheability_for_key("engine.game.run", Cacheability::NoCache);
    let no_cache_out = run_with_registry(src, no_cache_registry);
    assert_eq!(
        no_cache_out.observe_cache.hits, 0,
        "no-cache registry should disable hits for engine.game.run"
    );
    assert_eq!(
        cached_out.signature, no_cache_out.signature,
        "cache-on/off must keep semantic signature identical"
    );

    let cached_b = cached_out
        .env
        .get("b")
        .expect("binding b exists in cached run");
    let no_cache_b = no_cache_out
        .env
        .get("b")
        .expect("binding b exists in no-cache run");
    assert_eq!(
        cached_b, no_cache_b,
        "cache-on/off must keep runtime output identical"
    );
}
