use ocp::ocp::{
    parse_program, typecheck_program, CapabilityRegistry, ErrorCode, KeyCapabilityKind, TypeChecker,
};

#[test]
fn commit_for_observe_only_literal_key_fails_at_typecheck() {
    let src = r#"
observe("std.fs.read_text", "tier2", { path: "./README.md", max_bytes: 64 }, budget(10)) -> r;
commit(r);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    let err = typecheck_program(&program).expect_err("typecheck should reject observe-only key");
    assert_eq!(err.code, ErrorCode::TCommitForbiddenKey);
}

#[test]
fn commit_for_observe_and_commit_key_passes_typecheck() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(10)) -> r;
commit(r);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass for commit-capable key");
}

#[test]
fn dynamic_key_keeps_runtime_only_commit_validation() {
    let src = r#"
let k = "std.fs.read_text";
observe(k, "tier2", { path: "./README.md", max_bytes: 64 }, budget(10)) -> r;
commit(r);
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("dynamic key should remain runtime-validated");
}

#[test]
fn registry_key_kind_override_is_enforced() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(10)) -> r;
commit(r);
"#;
    let program = parse_program(src, 1).expect("parse should pass");

    let mut registry = CapabilityRegistry::v1_baseline();
    registry.set_key_kind_for_key("world.ok", KeyCapabilityKind::ObserveOnly);

    let err = TypeChecker::with_registry(registry)
        .check_program(&program)
        .expect_err("typecheck should reject overridden observe-only key");
    assert_eq!(err.code, ErrorCode::TCommitForbiddenKey);
}
