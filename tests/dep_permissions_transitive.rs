use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{check_project_with_lock, init_project};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_v10_dep_perm_transitive_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write text");
}

fn write_manifest_with_b_and_c(root: &Path) {
    write_text(
        &root.join("Ocl.toml"),
        concat!(
            "[package]\n",
            "name = \"perm_transitive_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "b = { name=\"b-pkg\", version=\"1.0.0\", source=\"path\" }\n",
            "c = { name=\"c-pkg\", version=\"1.0.0\", source=\"path\" }\n\n",
            "[permissions.package]\n",
            "allow = [\"std.*\"]\n\n",
            "[permissions.std_kv]\n",
            "enabled = true\n",
            "max_keys = 10\n",
            "max_value_bytes = 256\n",
            "key_prefix = \"app.\"\n",
        ),
    );
    write_text(
        &root.join("src").join("main.ocl"),
        "import b.bridge;\nlet ok = true;\ncondition(ok);\n",
    );
}

#[test]
fn v10_transitive_permissions_do_not_allow_borrowing_from_other_dependency() {
    let root = temp_project_dir("borrow_blocked");
    init_project(&root).expect("init");
    write_manifest_with_b_and_c(&root);

    write_text(
        &root.join("deps").join("b").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"b-pkg\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/bridge.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"bridge\"]\n\n",
            "[requested_permissions]\n",
            "std_kv.enabled = false\n",
            "std_kv.max_keys = 1\n",
            "std_kv.max_value_bytes = 1\n",
            "std_kv.key_prefix = \"app.\"\n",
        ),
    );
    write_text(
        &root.join("deps").join("b").join("src").join("bridge.ocl"),
        concat!(
            "import c.leaf;\n",
            "observe(\"std.kv.get\", \"tier2\", ctx(\"key=app.user\"), budget(5)) -> b_r;\n",
            "let ok = true;\n",
            "condition(ok);\n",
        ),
    );

    write_text(
        &root.join("deps").join("c").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"c-pkg\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/leaf.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"leaf\"]\n\n",
            "[requested_permissions]\n",
            "std_kv.enabled = true\n",
            "std_kv.max_keys = 10\n",
            "std_kv.max_value_bytes = 128\n",
            "std_kv.key_prefix = \"app.\"\n",
        ),
    );
    write_text(
        &root.join("deps").join("c").join("src").join("leaf.ocl"),
        "observe(\"std.kv.get\", \"tier2\", ctx(\"key=app.user\"), budget(5)) -> c_r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("b package must be blocked");
    let msg = err.to_string();
    assert!(msg.contains("RC-KV-PERMISSION-DENIED"), "actual={msg}");
    assert!(msg.contains("std.kv.get"), "actual={msg}");
}

#[test]
fn v10_transitive_permissions_allow_callsite_with_its_own_grant() {
    let root = temp_project_dir("callsite_ok");
    init_project(&root).expect("init");
    write_manifest_with_b_and_c(&root);

    write_text(
        &root.join("deps").join("b").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"b-pkg\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/bridge.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"bridge\"]\n\n",
            "[requested_permissions]\n",
            "std_kv.enabled = false\n",
            "std_kv.max_keys = 1\n",
            "std_kv.max_value_bytes = 1\n",
            "std_kv.key_prefix = \"app.\"\n",
        ),
    );
    write_text(
        &root.join("deps").join("b").join("src").join("bridge.ocl"),
        "import c.leaf;\nlet ok = true;\ncondition(ok);\n",
    );

    write_text(
        &root.join("deps").join("c").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"c-pkg\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/leaf.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"leaf\"]\n\n",
            "[requested_permissions]\n",
            "std_kv.enabled = true\n",
            "std_kv.max_keys = 5\n",
            "std_kv.max_value_bytes = 64\n",
            "std_kv.key_prefix = \"app.\"\n",
        ),
    );
    write_text(
        &root.join("deps").join("c").join("src").join("leaf.ocl"),
        "observe(\"std.kv.get\", \"tier2\", ctx(\"key=app.user\"), budget(5)) -> c_r;\nlet ok = true;\ncondition(ok);\n",
    );

    check_project_with_lock(&root, false)
        .expect("callsite in c should pass with its own effective grant");
}
