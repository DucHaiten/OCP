use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{check_project_with_lock, init_project, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_compat_ctx_extra_fields_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocl.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocl"), body).expect("write main");
}

fn manifest_with_ctx_extra_mode(mode: Option<&str>) -> String {
    let compat = match mode {
        Some(v) => format!(
            r#"[compat]
ctx_string = "warn"
ctx_extra_fields = "{v}"
"#
        ),
        None => String::new(),
    };
    format!(
        r#"[package]
name = "compat_ctx_extra_fields"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.std_fs]
read = ["./**"]

{compat}"#
    )
}

const SOURCE_WITH_UNKNOWN_CTX_FIELD: &str = r#"observe("std.fs.read_text", "tier2", { path: "./README.md", max_bytes: 128, extra_flag: true }, budget(10)) -> r;
"#;

#[test]
fn compat_ctx_extra_fields_default_warn_allows_unknown_ctx_fields() {
    let root = temp_project_dir("default_warn");
    init_project(&root).expect("init");
    write_manifest(&root, &manifest_with_ctx_extra_mode(None));
    write_main(&root, SOURCE_WITH_UNKNOWN_CTX_FIELD);

    check_project_with_lock(&root, false)
        .expect("default warn mode should allow unknown ctx fields for migration");
}

#[test]
fn compat_ctx_extra_fields_allow_accepts_unknown_ctx_fields() {
    let root = temp_project_dir("allow");
    init_project(&root).expect("init");
    write_manifest(&root, &manifest_with_ctx_extra_mode(Some("allow")));
    write_main(&root, SOURCE_WITH_UNKNOWN_CTX_FIELD);

    check_project_with_lock(&root, false).expect("allow mode should accept unknown ctx fields");
}

#[test]
fn compat_ctx_extra_fields_deny_rejects_unknown_ctx_fields() {
    let root = temp_project_dir("deny");
    init_project(&root).expect("init");
    write_manifest(&root, &manifest_with_ctx_extra_mode(Some("deny")));
    write_main(&root, SOURCE_WITH_UNKNOWN_CTX_FIELD);

    let err =
        check_project_with_lock(&root, false).expect_err("deny mode must reject unknown ctx field");
    let SdkError::Runtime(runtime_err) = err else {
        panic!("expected runtime diagnostic from typecheck");
    };
    let msg = runtime_err.to_string();
    assert!(
        msg.contains("T-CTX-UNKNOWN-FIELD"),
        "actual runtime error: {msg}"
    );
}
