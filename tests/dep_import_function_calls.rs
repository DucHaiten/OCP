use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{check_project_with_lock, init_project, run_project_with_lock};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocp_v10_dep_import_call_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write text");
}

fn setup_provider_consumer(root: &Path) {
    init_project(root).expect("init project");
    write_text(
        &root.join("Ocp.toml"),
        concat!(
            "[package]\n",
            "name = \"consumer.demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n",
            "foo = { name=\"provider.lib\", version=\"0.1.0\", source=\"path\" }\n",
        ),
    );
    write_text(
        &root.join("src").join("main.ocp"),
        r#"
module consumer.main;
import foo.api;
let v = ping("ok");
condition(v == "ok");
"#,
    );

    write_text(
        &root.join("deps").join("foo").join("package.ocpp"),
        concat!(
            "[package]\n",
            "name = \"provider.lib\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"api\"\n\n",
            "[exports]\n",
            "modules = [\"api\"]\n",
        ),
    );
    write_text(
        &root.join("deps").join("foo").join("src").join("api.ocp"),
        r#"
module foo.api;
fn ping(input) {
  return input;
}
condition(true);
"#,
    );
}

#[test]
fn v10_dependency_import_function_call_passes_check_and_run() {
    let root = temp_project_dir("pass");
    setup_provider_consumer(&root);

    check_project_with_lock(&root, false).expect("check should resolve imported dependency fn");
    let run = run_project_with_lock(&root, false)
        .expect("run should resolve imported dependency fn");
    assert!(run.steps > 0, "run should execute at least one step");
}
