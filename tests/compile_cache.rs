use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_ocl::ocp_ocl::{compile_with_cache, CompileCacheKeyInput, HIR_SCHEMA_VERSION};

fn test_cache_root(name: &str) -> PathBuf {
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    PathBuf::from("target")
        .join("ocl")
        .join("tests")
        .join("compile_cache")
        .join(format!("{name}_{tick}"))
}

fn base_key_input() -> CompileCacheKeyInput {
    CompileCacheKeyInput {
        compiler_version: "compiler-v13".to_string(),
        ocl_version: "ocl-v0.13".to_string(),
        lane_literal: "locked_v071".to_string(),
        lock_hash: "lock_hash_demo".to_string(),
        trust_hash: "trust_hash_demo".to_string(),
        deps_graph_hash: "deps_graph_hash_demo".to_string(),
        manifest_hash: "manifest_hash_demo".to_string(),
        schema_versions_of_packs: "schema_v1".to_string(),
        entry_module_hash: "entry_hash_demo".to_string(),
        source_bundle_hash: "source_bundle_hash_demo".to_string(),
        cache_toggle_inputs: vec!["OCL_CACHE_PROFILE=default".to_string()],
    }
}

#[test]
fn compile_cache_warm_hit_returns_identical_ir() {
    let src = r#"
let payload = { a: 1, z: 2 };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> r;
"#;
    let input = base_key_input();
    let cache_root = test_cache_root("warm_hit");

    let cold = compile_with_cache(src, 1, &input, &cache_root).expect("cold compile should pass");
    assert!(!cold.hit, "first compile must miss cache");
    assert_eq!(cold.ir_hash.len(), 64, "ir hash must be sha256 hex");

    let warm = compile_with_cache(src, 1, &input, &cache_root).expect("warm compile should pass");
    assert!(warm.hit, "second compile should hit cache");
    assert_eq!(
        cold.ir_hash, warm.ir_hash,
        "warm hit must return identical ir hash"
    );
    assert_eq!(
        cold.canonical_hir_bytes, warm.canonical_hir_bytes,
        "warm hit must return identical canonical HIR bytes"
    );
}

#[test]
fn compile_cache_invalidates_on_manifest_hash_change() {
    let src = r#"
let v = [1, 2, 3];
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> r;
"#;
    let cache_root = test_cache_root("manifest_invalidate");
    let mut input_a = base_key_input();
    input_a.manifest_hash = "manifest_hash_A".to_string();
    let mut input_b = input_a.clone();
    input_b.manifest_hash = "manifest_hash_B".to_string();

    let a = compile_with_cache(src, 1, &input_a, &cache_root).expect("compile A should pass");
    assert!(!a.hit, "first compile should miss cache");

    let b = compile_with_cache(src, 1, &input_b, &cache_root).expect("compile B should pass");
    assert!(!b.hit, "manifest hash change must invalidate compile cache");
    assert_ne!(
        a.cache_key, b.cache_key,
        "manifest hash change must produce different cache key"
    );
}

#[test]
fn compile_cache_invalidates_on_compiler_or_deps_change() {
    let src = r#"
let m = { "k": 1 };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> r;
"#;
    let cache_root = test_cache_root("compiler_deps_invalidate");

    let mut base = base_key_input();
    base.compiler_version = "compiler-v13.0".to_string();
    base.deps_graph_hash = "deps_hash_A".to_string();
    let base_artifact =
        compile_with_cache(src, 1, &base, &cache_root).expect("base compile should pass");
    assert!(!base_artifact.hit);

    let mut compiler_changed = base.clone();
    compiler_changed.compiler_version = "compiler-v13.1".to_string();
    let compiler_changed_artifact = compile_with_cache(src, 1, &compiler_changed, &cache_root)
        .expect("compiler changed compile should pass");
    assert!(
        !compiler_changed_artifact.hit,
        "compiler version change must invalidate cache"
    );
    assert_ne!(base_artifact.cache_key, compiler_changed_artifact.cache_key);

    let mut deps_changed = base.clone();
    deps_changed.deps_graph_hash = "deps_hash_B".to_string();
    let deps_changed_artifact = compile_with_cache(src, 1, &deps_changed, &cache_root)
        .expect("deps changed compile should pass");
    assert!(
        !deps_changed_artifact.hit,
        "deps graph hash change must invalidate cache"
    );
    assert_ne!(base_artifact.cache_key, deps_changed_artifact.cache_key);
    assert!(
        deps_changed_artifact
            .canonical_hir_bytes
            .starts_with(format!("hir_schema_version={HIR_SCHEMA_VERSION}").as_bytes()),
        "cached HIR bytes must contain schema header"
    );
}
