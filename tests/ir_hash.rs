use ocp::ocp::{
    build_hir_and_hash, canonical_hir_bytes, parse_program, HIR_HASHER_VERSION_V1,
    HIR_SCHEMA_VERSION,
};

fn hash_source(src: &str) -> String {
    let program = parse_program(src, 1).expect("source should parse");
    let (_, hash) = build_hir_and_hash(&program).expect("source should build HIR");
    hash
}

#[test]
fn ir_hash_is_stable_for_same_source_across_runs() {
    let src = r#"
let xs = [1, 2, 3];
let meta = { name: "demo", enabled: true };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> r;
"#;

    let h1 = hash_source(src);
    let h2 = hash_source(src);
    assert_eq!(h1, h2, "same source must produce same HIR hash");
    assert_eq!(h1.len(), 64, "sha256 hex must be 64 chars");
}

#[test]
fn ir_hash_ignores_whitespace_only_changes() {
    let src_compact = r#"let v={a:1,b:2};observe("world.exists","tier2",{path:"./a.txt",quota:2},budget(10))->r;"#;
    let src_spaced = r#"
let v = { a: 1, b: 2 };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> r;
"#;

    let h1 = hash_source(src_compact);
    let h2 = hash_source(src_spaced);
    assert_eq!(h1, h2, "whitespace-only edits must not change HIR hash");
}

#[test]
fn ir_hash_uses_canonical_key_order_for_map_and_record_literals() {
    let src_a = r#"
let m = { "b": 2, "a": 1 };
let r = { z: 3, a: 1 };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> out;
"#;
    let src_b = r#"
let m = { "a": 1, "b": 2 };
let r = { a: 1, z: 3 };
observe("world.exists", "tier2", { path: "./a.txt", quota: 2 }, budget(10)) -> out;
"#;

    let program_a = parse_program(src_a, 1).expect("source A should parse");
    let (hir_a, hash_a) = build_hir_and_hash(&program_a).expect("source A should build HIR");
    let bytes_a = canonical_hir_bytes(&hir_a);
    let header = format!("hir_schema_version={}", HIR_SCHEMA_VERSION);
    assert!(
        String::from_utf8_lossy(&bytes_a).starts_with(&header),
        "canonical bytes must start with schema header"
    );
    assert_eq!(HIR_HASHER_VERSION_V1, "sha256-v1");

    let hash_b = hash_source(src_b);
    assert_eq!(
        hash_a, hash_b,
        "record/map key reordering must not change canonical HIR hash"
    );
}
