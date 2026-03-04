use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, resolve_deps_v3, verify_deps_lock_v3};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v10_lock_resolve_{tag}_{stamp}"))
}

fn write_manifest_for_lock_v3(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"lock_resolve_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n",
        "engine_ui_widgets = { name=\"engine-ui-widgets\", version=\"0.3.*\", source=\"registry\" }\n",
        "local_tools = { name=\"local-tools\", version=\"1.*\", source=\"path\" }\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

#[test]
fn v10_lock_v3_resolve_is_deterministic_and_exports_ocl_lock() {
    let root = temp_project_dir("det");
    init_project(&root).expect("init");
    write_manifest_for_lock_v3(&root);

    let run1 = resolve_deps_v3(&root, false).expect("resolve run1");
    assert_eq!(run1.deps_resolved, 3);
    assert!(!run1.wrote_legacy_lock_v2);
    assert!(run1.lock_v3_path.exists());
    assert!(run1.ocl_lock_path.exists());

    let lock_text_1 = fs::read_to_string(&run1.lock_v3_path).expect("read deps.lock.v3 run1");
    assert!(lock_text_1.contains("version=3"));
    assert!(lock_text_1.contains("hasher_version=sha256-v1"));
    assert!(lock_text_1.contains("dep=std|std|0.1.0|builtin|"));
    assert!(lock_text_1.contains("dep=engine_ui_widgets|engine-ui-widgets|0.3.0|registry|"));
    assert!(lock_text_1.contains("dep=local_tools|local-tools|1.0.0|path|"));

    let run2 = resolve_deps_v3(&root, false).expect("resolve run2");
    let lock_text_2 = fs::read_to_string(&run2.lock_v3_path).expect("read deps.lock.v3 run2");
    assert_eq!(run1.lock_hash, run2.lock_hash);
    assert_eq!(lock_text_1, lock_text_2);
}

#[test]
fn v10_lock_v3_mismatch_is_fail_honest() {
    let root = temp_project_dir("mismatch");
    init_project(&root).expect("init");
    write_manifest_for_lock_v3(&root);

    let summary = resolve_deps_v3(&root, false).expect("resolve");
    let mut tampered = fs::read_to_string(&summary.lock_v3_path).expect("read lock");
    tampered = tampered.replace(
        "dep=engine_ui_widgets|engine-ui-widgets|0.3.0|registry|",
        "dep=engine_ui_widgets|engine-ui-widgets|9.9.9|registry|",
    );
    fs::write(&summary.lock_v3_path, tampered).expect("write tampered lock");

    let err = verify_deps_lock_v3(&root).expect_err("must fail on mismatch");
    assert!(
        err.to_string().contains("deps.lock.v3 mismatch"),
        "unexpected error: {err}"
    );
}
