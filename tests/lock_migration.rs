use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{init_project, read_resolved_deps_v3, resolve_deps_v3, sync_deps_lock_v2};
use serde_json::json;

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v10_lock_migration_{tag}_{stamp}"))
}

fn write_manifest_for_migration(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"lock_migration_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n",
        "kit = { name=\"kit-runtime\", version=\"2.4.*\", source=\"registry\" }\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

#[test]
fn v10_reader_supports_legacy_lock_v2_when_v3_missing() {
    let root = temp_project_dir("legacy_reader");
    init_project(&root).expect("init");
    write_manifest_for_migration(&root);

    sync_deps_lock_v2(&root).expect("sync deps.lock.v2");
    let lock_v3_path = root.join("deps.lock.v3");
    if lock_v3_path.exists() {
        fs::remove_file(&lock_v3_path).expect("remove deps.lock.v3");
    }

    let deps = read_resolved_deps_v3(&root).expect("read deps from legacy v2");
    assert_eq!(deps.len(), 2);
    assert!(deps
        .iter()
        .any(|d| d.alias == "std" && d.source == "builtin"));
    assert!(deps
        .iter()
        .any(|d| d.name == "kit-runtime" && d.version == "2.4.0"));
}

#[test]
fn v10_resolve_can_write_v3_and_legacy_v2_together() {
    let root = temp_project_dir("write_legacy");
    init_project(&root).expect("init");
    write_manifest_for_migration(&root);

    let summary = resolve_deps_v3(&root, true).expect("resolve with legacy lock");
    assert_eq!(summary.deps_resolved, 2);
    assert!(summary.wrote_legacy_lock_v2);
    assert!(summary.lock_v3_path.exists());
    assert!(summary.ocl_lock_path.exists());
    assert!(root.join("deps.lock.v2").exists());

    let deps = read_resolved_deps_v3(&root).expect("read deps from v3");
    assert_eq!(deps.len(), 2);
}

#[test]
fn v16_lock_migration_writes_migration_diff_report() {
    let root = temp_project_dir("v16_report");
    init_project(&root).expect("init");
    write_manifest_for_migration(&root);

    sync_deps_lock_v2(&root).expect("sync deps.lock.v2");
    let lock_v3_path = root.join("deps.lock.v3");
    if lock_v3_path.exists() {
        fs::remove_file(&lock_v3_path).expect("remove stale deps.lock.v3");
    }

    let deps_from_v2 = read_resolved_deps_v3(&root).expect("read deps through v2 fallback");
    let summary = resolve_deps_v3(&root, true).expect("resolve deps.lock.v3 + legacy v2");
    let deps_from_v3 = read_resolved_deps_v3(&root).expect("read deps from v3");

    assert_eq!(deps_from_v2.len(), 2);
    assert_eq!(deps_from_v3.len(), 2);
    assert!(summary.wrote_legacy_lock_v2);
    assert!(summary.lock_v3_path.exists());
    assert!(root.join("deps.lock.v2").exists());

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w16")
        .join("compat");
    fs::create_dir_all(&out_dir).expect("create w16 compat output dir");
    let report = json!({
        "schema": "ocl.w16.compat.migration_diff.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "baseline": "v0.15-line",
        "legacy_v2_fallback_read_ok": deps_from_v2.len() == 2,
        "v3_and_v2_dual_write_ok": summary.wrote_legacy_lock_v2
            && summary.lock_v3_path.exists()
            && root.join("deps.lock.v2").exists(),
        "deps_resolved_count": summary.deps_resolved,
        "lock_v3_path": summary.lock_v3_path.to_string_lossy(),
        "ocl_lock_path": summary.ocl_lock_path.to_string_lossy()
    });
    fs::write(
        out_dir.join("migration_diff_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize migration diff report"),
    )
    .expect("write migration_diff_report.json");
}
