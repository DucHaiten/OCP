use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_runtime_core::run_source;
use ocl_sdk::{
    init_project, resolve_view_observe_key_v1, resolve_view_selection_v1, sync_cosmos_lock_v1,
    sync_deps_lock_v1, sync_policy_lock_v1,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v5_w5_{tag}_{stamp}"))
}

fn prepare_project(root: &Path) {
    init_project(root).expect("init project");
    let manifest = r#"[package]
name = "v5_w5_views"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
    fs::write(
        root.join("src").join("main.ocl"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write source");
    sync_deps_lock_v1(root).expect("sync deps lock");
}

#[test]
fn v5_w5_view_selection_domain_ambig_fail_hard() {
    let root = temp_project_dir("domain_ambig");
    prepare_project(&root);

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"

[[view]]
id = "ops"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[view]]
id = "ops"
universe_id = "ci_locked"
domain_id = "side"
renderer = "tree"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let err = resolve_view_selection_v1(&root, false, Some("ci_locked"), None, Some("ops"))
        .expect_err("must fail hard on domain ambiguity for same view id");
    assert!(
        err.to_string().contains("V-VIEW-DOMAIN-AMBIG"),
        "unexpected err: {err}"
    );
}

#[test]
fn v5_w5_view_selection_domain_mismatch_fail_hard() {
    let root = temp_project_dir("domain_mismatch");
    prepare_project(&root);

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"

[[view]]
id = "side_only"
universe_id = "ci_locked"
domain_id = "side"
renderer = "tree"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let err = resolve_view_selection_v1(
        &root,
        false,
        Some("ci_locked"),
        Some("default"),
        Some("side_only"),
    )
    .expect_err("must fail hard on view/domain mismatch");
    assert!(
        err.to_string().contains("V-VIEW-DOMAIN-MISMATCH"),
        "unexpected err: {err}"
    );
}

#[test]
fn v5_w5_view_render_text_differs_by_view_id_same_truth() {
    let src = r#"
observe("std.view.render_text", "tier2", ctx("truth=hello"), budget(3)) -> r;
"#;

    std::env::set_var("OCL_VIEW_ID", "alpha");
    let out_alpha = run_source(src, 1, 128).expect("exec alpha");

    std::env::set_var("OCL_VIEW_ID", "beta");
    let out_beta = run_source(src, 1, 128).expect("exec beta");
    std::env::remove_var("OCL_VIEW_ID");

    let alpha_hash = extract_render_hash(&out_alpha);
    let beta_hash = extract_render_hash(&out_beta);
    assert_ne!(
        alpha_hash, beta_hash,
        "same truth with different view_id must produce different render hash"
    );
}

#[test]
fn v5_w5_kit_wiring_by_view_id_same_truth_distinct_outputs() {
    let root = temp_project_dir("kit_wiring");
    prepare_project(&root);

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[view]]
id = "alpha"
universe_id = "ci_locked"
domain_id = "default"
renderer = "tree"

[[view]]
id = "beta"
universe_id = "ci_locked"
domain_id = "default"
renderer = "tree"

[[kits]]
kit_id = "std.kit.view_text_basic"
universe_id = "ci_locked"
bind_domain = "default"
bind_view = "alpha"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");

    let alpha_selection = resolve_view_selection_v1(
        &root,
        false,
        Some("ci_locked"),
        Some("default"),
        Some("alpha"),
    )
    .expect("resolve alpha view");
    let beta_selection = resolve_view_selection_v1(
        &root,
        false,
        Some("ci_locked"),
        Some("default"),
        Some("beta"),
    )
    .expect("resolve beta view");

    let alpha_key = resolve_view_observe_key_v1(&root, &alpha_selection).expect("alpha key");
    let beta_key = resolve_view_observe_key_v1(&root, &beta_selection).expect("beta key");
    assert_eq!(alpha_key, "std.view.render_text");
    assert_eq!(beta_key, "std.view.render_tree");

    let alpha_src =
        format!("observe(\"{alpha_key}\", \"tier2\", ctx(\"truth=hello\"), budget(3)) -> r;");
    let beta_src =
        format!("observe(\"{beta_key}\", \"tier2\", ctx(\"truth=hello\"), budget(3)) -> r;");

    std::env::set_var("OCL_VIEW_ID", "alpha");
    let out_alpha = run_source(&alpha_src, 1, 128).expect("exec alpha");

    std::env::set_var("OCL_VIEW_ID", "beta");
    let out_beta = run_source(&beta_src, 1, 128).expect("exec beta");
    std::env::remove_var("OCL_VIEW_ID");

    let alpha_hash = extract_render_hash(&out_alpha);
    let beta_hash = extract_render_hash(&out_beta);
    assert_ne!(
        alpha_hash, beta_hash,
        "kit wiring by view_id must produce distinct outputs on same truth"
    );
}

#[test]
fn v5_w5_locked_view_wiring_drift_detected_by_cosmos_lock() {
    let root = temp_project_dir("lock_drift");
    prepare_project(&root);
    sync_policy_lock_v1(&root).expect("sync policy lock");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[view]]
id = "text_default"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[kits]]
kit_id = "std.kit.view_text_basic"
universe_id = "ci_locked"
bind_domain = "default"
bind_view = "text_default"
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    sync_cosmos_lock_v1(&root, false, None, None, None).expect("sync cosmos lock");

    let drifted = cosmos.replace("renderer = \"text\"", "renderer = \"tree\"");
    fs::write(root.join("cosmos.toml"), drifted).expect("mutate cosmos");

    let err = resolve_view_selection_v1(
        &root,
        true,
        Some("ci_locked"),
        Some("default"),
        Some("text_default"),
    )
    .expect_err("locked view drift must fail hard");
    assert!(
        err.to_string().contains("V-COSMOS-LOCK-MISMATCH"),
        "unexpected error: {err}"
    );
}

fn extract_render_hash(out: &ocl_runtime_core::ExecOutput) -> String {
    let value = out.env.get("r").expect("missing bind r");
    format!("{value:?}")
}
