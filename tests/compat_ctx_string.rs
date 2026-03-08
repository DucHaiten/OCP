use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    check_project_with_lock, init_project, parse_project_language_config_v071, SdkError,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_compat_ctx_string_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocp.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocp"), body).expect("write main");
}

fn base_manifest_with_compat(compat_block: &str) -> String {
    format!(
        r#"[package]
name = "compat_ctx_string"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.std_fs]
read = ["./**"]

{compat_block}
"#
    )
}

#[test]
fn manifest_parses_ctx_string_and_ctx_extra_fields_modes() {
    let manifest = r#"
[project]
lane = "locked_v06"

[language]
guard_mode = "error"

[compat]
ctx_string = "deny"
ctx_extra_fields = "allow"
"#;
    let cfg = parse_project_language_config_v071(manifest);
    assert_eq!(cfg.lane, "locked_v06");
    assert_eq!(format!("{:?}", cfg.guard_mode), "Error");
    assert_eq!(format!("{:?}", cfg.compat_ctx_string), "Deny");
    assert_eq!(format!("{:?}", cfg.compat_ctx_extra_fields), "Allow");
}

#[test]
fn compat_ctx_string_warn_allows_legacy_ctx_call() {
    let root = temp_project_dir("warn");
    init_project(&root).expect("init");
    write_manifest(
        &root,
        &base_manifest_with_compat(
            r#"[compat]
ctx_string = "warn"
ctx_extra_fields = "warn""#,
        ),
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./README.md;max_bytes=128"), budget(10)) -> r;
"#,
    );

    check_project_with_lock(&root, false).expect("warn mode should keep legacy ctx-string path");
}

#[test]
fn compat_ctx_string_deny_rejects_legacy_ctx_call_for_std_key() {
    let root = temp_project_dir("deny");
    init_project(&root).expect("init");
    write_manifest(
        &root,
        &base_manifest_with_compat(
            r#"[compat]
ctx_string = "deny"
ctx_extra_fields = "warn""#,
        ),
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./README.md;max_bytes=128"), budget(10)) -> r;
"#,
    );

    let err = check_project_with_lock(&root, false).expect_err("deny mode must reject ctx-string");
    let SdkError::Runtime(runtime_err) = err else {
        panic!("expected runtime diagnostic from typecheck");
    };
    let msg = runtime_err.to_string();
    assert!(
        msg.contains("T-CTX-STRING-COMPAT"),
        "actual runtime error: {msg}"
    );
}
