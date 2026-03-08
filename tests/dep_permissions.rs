use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{check_project_with_lock, compute_effective_permissions_v10, init_project};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocp_v10_dep_permissions_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write text");
}

#[test]
fn v10_effective_permissions_apply_bool_number_enum_and_glob_algebra() {
    let root = temp_project_dir("algebra");
    init_project(&root).expect("init");

    write_text(
        &root.join("Ocp.toml"),
        concat!(
            "[package]\n",
            "name = \"perm_algebra_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "toolkit = { name=\"toolkit\", version=\"1.0.0\", source=\"path\" }\n\n",
            "[permissions.package]\n",
            "allow = [\"std.*\"]\n\n",
            "[deny]\n",
            "patterns = [\"std.net.*\"]\n\n",
            "[permissions.std_proc]\n",
            "enabled = true\n",
            "allow_bins = [\"python\", \"git\"]\n",
            "timeout_ms = 4000\n",
            "max_stdout_bytes = 900\n",
            "max_stderr_bytes = 700\n\n",
            "[permissions.std_ui]\n",
            "enabled = true\n",
            "max_draw_cmds = 1000\n",
            "max_input_events = 200\n",
            "assets_read = [\"./assets/**\", \"./themes/base.css\"]\n",
            "max_asset_bytes = 10000\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./data/**\", \"./shared/config.json\"]\n",
            "list = [\"./data/**\"]\n",
            "max_read_bytes = 500\n",
            "max_list_entries = 30\n",
        ),
    );

    write_text(
        &root.join("deps").join("toolkit").join("package.ocpp"),
        concat!(
            "[package]\n",
            "name = \"toolkit\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/worker.ocp\"\n\n",
            "[exports]\n",
            "modules = [\"worker\"]\n\n",
            "[requested_permissions]\n",
            "std_proc.enabled = true\n",
            "std_proc.allow_bins = [\"python\", \"node\"]\n",
            "std_proc.timeout_ms = 1500\n",
            "std_proc.max_stdout_bytes = 1200\n",
            "std_proc.max_stderr_bytes = 300\n",
            "std_ui.enabled = true\n",
            "std_ui.max_draw_cmds = 3000\n",
            "std_ui.max_input_events = 120\n",
            "std_ui.assets_read = [\"./assets/icons/**\", \"./themes/base.css\", \"./themes/*.css\"]\n",
            "std_ui.max_asset_bytes = 8000\n",
            "std_fs.read = [\"./data/reports/**\", \"./shared/config.json\", \"./shared/*.json\"]\n",
            "std_fs.list = [\"./data/reports/**\"]\n",
            "std_fs.max_read_bytes = 200\n",
            "std_fs.max_list_entries = 10\n",
        ),
    );

    let effective = compute_effective_permissions_v10(&root).expect("compute effective perms");
    let dep_entry = effective
        .iter()
        .find(|(pkg, _)| pkg.starts_with("dep:toolkit@1.0.0#"))
        .expect("dep effective entry must exist")
        .1;

    let proc_cfg = dep_entry.std_proc.as_ref().expect("std_proc must exist");
    assert!(proc_cfg.enabled);
    assert_eq!(proc_cfg.allow_bins, vec!["python".to_string()]);
    assert_eq!(proc_cfg.timeout_ms, 1500);
    assert_eq!(proc_cfg.max_stdout_bytes, 900);
    assert_eq!(proc_cfg.max_stderr_bytes, 300);

    let ui_cfg = dep_entry.std_ui.as_ref().expect("std_ui must exist");
    assert!(ui_cfg.enabled);
    assert_eq!(ui_cfg.max_draw_cmds, 1000);
    assert_eq!(ui_cfg.max_input_events, 120);
    assert_eq!(ui_cfg.max_asset_bytes, 8000);
    assert!(ui_cfg.assets_read.contains(&"assets/icons/**".to_string()));
    assert!(ui_cfg.assets_read.contains(&"themes/base.css".to_string()));
    assert!(!ui_cfg.assets_read.contains(&"themes/*.css".to_string()));

    let fs_cfg = dep_entry.std_fs.as_ref().expect("std_fs must exist");
    assert_eq!(fs_cfg.max_read_bytes, 200);
    assert_eq!(fs_cfg.max_list_entries, 10);
    assert!(fs_cfg.read.contains(&"data/reports/**".to_string()));
    assert!(fs_cfg.read.contains(&"shared/config.json".to_string()));
    assert!(!fs_cfg.read.contains(&"shared/*.json".to_string()));

    assert!(dep_entry.global_deny.contains(&"std.net.*".to_string()));
}

#[test]
fn v10_dependency_requested_permissions_can_block_even_if_project_grants() {
    let root = temp_project_dir("deny_by_requested");
    init_project(&root).expect("init");

    write_text(
        &root.join("Ocp.toml"),
        concat!(
            "[package]\n",
            "name = \"perm_runtime_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "toolkit = { name=\"toolkit\", version=\"1.0.0\", source=\"path\" }\n\n",
            "[permissions.package]\n",
            "allow = [\"std.*\"]\n\n",
            "[permissions.std_proc]\n",
            "enabled = true\n",
            "allow_bins = [\"git\"]\n",
            "timeout_ms = 5000\n",
            "max_stdout_bytes = 1024\n",
            "max_stderr_bytes = 1024\n",
        ),
    );
    write_text(
        &root.join("src").join("main.ocp"),
        "import toolkit.worker;\nlet ready = true;\ncondition(ready);\n",
    );

    write_text(
        &root.join("deps").join("toolkit").join("package.ocpp"),
        concat!(
            "[package]\n",
            "name = \"toolkit\"\n",
            "version = \"1.0.0\"\n",
            "entry = \"src/worker.ocp\"\n\n",
            "[exports]\n",
            "modules = [\"worker\"]\n\n",
            "[requested_permissions]\n",
            "std_proc.enabled = false\n",
            "std_proc.allow_bins = []\n",
        ),
    );
    write_text(
        &root.join("deps").join("toolkit").join("src").join("worker.ocp"),
        "observe(\"std.proc.exec\", \"tier2\", ctx(\"bin=git;args=status\"), budget(5)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("must fail by effective permission");
    let msg = err.to_string();
    assert!(msg.contains("RC-PROC-BIN-DENIED"), "actual={msg}");
    assert!(msg.contains("std.proc.exec"), "actual={msg}");
}
