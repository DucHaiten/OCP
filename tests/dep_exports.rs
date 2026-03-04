use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, verify_dependency_exports_and_collect_provenance_v10, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_v10_dep_exports_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write text");
}

fn setup_dep_project(root: &Path, exported_modules: &str) {
    init_project(root).expect("init project");
    write_text(
        &root.join("Ocl.toml"),
        concat!(
            "[package]\n",
            "name = \"dep_exports_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "toolkit = { name=\"toolkit\", version=\"1.0.0\", source=\"path\" }\n",
        ),
    );
    write_text(
        &root.join("src").join("main.ocl"),
        r#"
import toolkit.api.math;
let ok = dep_ready();
condition(ok);
"#,
    );
    write_text(
        &root.join("deps").join("toolkit").join("package.oclp"),
        &format!(
            "[package]\nname = \"toolkit\"\nversion = \"1.0.0\"\nentry = \"api.math\"\n\n[exports]\nmodules = {exported_modules}\n"
        ),
    );
    write_text(
        &root
            .join("deps")
            .join("toolkit")
            .join("src")
            .join("api")
            .join("math.ocl"),
        r#"
fn dep_ready() {
  return true;
}
"#,
    );
}

#[test]
fn v10_dep_exports_collects_provenance_for_project_and_dependency_modules() {
    let root = temp_project_dir("collect");
    setup_dep_project(&root, "[\"api.math\"]");

    let provenance = verify_dependency_exports_and_collect_provenance_v10(&root)
        .expect("dependency exports should pass");
    assert!(
        provenance
            .iter()
            .any(|p| p.module_id == "main" && p.package_id == "project:dep_exports_demo"),
        "missing project provenance entry: {provenance:#?}"
    );
    assert!(
        provenance.iter().any(|p| p.module_id == "api.math"
            && p.package_id.starts_with("dep:toolkit@1.0.0#")
            && p.file_path
                .replace('\\', "/")
                .ends_with("deps/toolkit/src/api/math.ocl")),
        "missing dependency provenance entry: {provenance:#?}"
    );
}

#[test]
fn v10_dep_exports_rejects_dependency_module_not_in_exports() {
    let root = temp_project_dir("deny");
    setup_dep_project(&root, "[]");

    let err = verify_dependency_exports_and_collect_provenance_v10(&root)
        .expect_err("must fail when dependency module is not exported");
    match err {
        SdkError::Runtime(diag) => {
            let msg = diag.to_string();
            assert!(
                msg.contains("T-IMPORT-NOT-FOUND") || msg.contains("not exported"),
                "unexpected diagnostic: {msg}"
            );
        }
        other => panic!("expected runtime diagnostic, got: {other:?}"),
    }
}
