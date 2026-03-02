use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    build_project_with_lock, check_project_with_lock, compose_phenotype, run_project_with_lock,
    run_reactor_ticks_with_lock, sync_deps_lock_v1, test_project_with_lock, verify_assembly,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn apps_root() -> PathBuf {
    workspace_root().join("apps")
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_m5_{tag}_{stamp}"))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination");
    for entry in fs::read_dir(src).expect("read source dir") {
        let entry = entry.expect("entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy file");
        }
    }
}

fn run_app_end_to_end(
    app_name: &str,
    reactor_ticks: Option<u32>,
    with_composer: bool,
) -> (usize, usize, usize) {
    let source_root = apps_root().join(app_name);
    assert!(
        source_root.exists(),
        "missing demo app source: {}",
        source_root.display()
    );

    let temp_root = temp_project_dir(&app_name.replace('-', "_"));
    copy_tree(&source_root, &temp_root);

    let lock = sync_deps_lock_v1(&temp_root).expect("sync lock");
    assert!(lock.deps_synced >= 1);

    if with_composer {
        let phenotype = temp_root.join("phenotype.toml");
        let registry = temp_root.join("registry");
        let compose =
            compose_phenotype(&temp_root, &phenotype, &registry, true).expect("compose phenotype");
        assert!(compose.selected_components >= 1);
        assert!(compose.generated_files >= 2);

        let verify =
            verify_assembly(&temp_root, &phenotype, &registry, true).expect("verify assembly");
        assert!(verify.ok);
    }

    let check = check_project_with_lock(&temp_root, true).expect("check locked");
    assert!(check.files_checked >= 1);

    let run = run_project_with_lock(&temp_root, true).expect("run locked");
    assert!(run.steps > 0);

    if let Some(ticks) = reactor_ticks {
        let reactor = run_reactor_ticks_with_lock(&temp_root, ticks, true).expect("run reactor");
        assert_eq!(reactor.ticks, ticks);
        assert!(reactor.total_steps > 0);
    }

    let tests = test_project_with_lock(&temp_root, true).expect("test locked");
    assert!(tests.tests_run >= 1);

    let build = build_project_with_lock(&temp_root, true).expect("build locked");
    assert!(build.files_bundled >= 1);

    (check.files_checked, tests.tests_run, build.files_bundled)
}

#[test]
fn m5_demo_apps_are_ocl_only() {
    let root = apps_root();
    assert!(root.exists(), "missing demo apps root: {}", root.display());

    let blocked_exts: HashSet<&str> = [
        "rs", "cpp", "cc", "c", "h", "hpp", "py", "js", "ts", "tsx", "java", "cs", "go", "kt",
        "swift",
    ]
    .into_iter()
    .collect();

    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            assert!(
                !blocked_exts.contains(ext.as_str()),
                "demo app contains host-code file: {}",
                path.display()
            );
        }
    }
}

#[test]
fn m5_demo_apps_end_to_end_cli_and_fetch() {
    let hello = run_app_end_to_end("hello-cli", None, false);
    assert!(hello.0 >= 2);
    let fetch = run_app_end_to_end("web-fetch", None, false);
    assert!(fetch.0 >= 2);
}

#[test]
fn m5_demo_apps_end_to_end_reactor() {
    let server = run_app_end_to_end("mini-server", Some(8), false);
    assert!(server.1 >= 1);
    let scheduler = run_app_end_to_end("scheduler", Some(8), false);
    assert!(scheduler.1 >= 1);
}

#[test]
fn m5_demo_apps_end_to_end_composer() {
    let composer = run_app_end_to_end("composer-demo", None, true);
    assert!(composer.2 >= 2);
}
