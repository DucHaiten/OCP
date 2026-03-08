use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{build_project_with_lock, check_project_with_lock, init_project, sync_deps_lock_v1};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_m3_{tag}_{stamp}"))
}

fn write_manifest_with_deps(root: &Path, deps: &[(&str, &str)]) {
    let mut body = String::from(
        "[package]\nname = \"m3_demo\"\nversion = \"0.1.0\"\n\n[targets]\ndefault = \"main\"\n\n[dependencies]\n",
    );
    for (name, version) in deps {
        body.push_str(name);
        body.push_str(" = \"");
        body.push_str(version);
        body.push_str("\"\n");
    }
    body.push_str("\n[permissions.package]\nallow = [\"*\"]\ndeny = [\"std.net.poll\"]\n");
    fs::write(root.join("Ocp.toml"), body).expect("write manifest");
}

#[test]
fn m3_lock_sync_writes_canonical_sorted_entries() {
    let root = temp_project_dir("lock_sync");
    init_project(&root).expect("init");
    write_manifest_with_deps(&root, &[("std", "0.1.0"), ("http", "1.2.3")]);

    let summary = sync_deps_lock_v1(&root).expect("lock sync");
    assert_eq!(summary.deps_synced, 2);

    let raw = fs::read_to_string(root.join("deps.lock")).expect("read lock");
    let lines: Vec<&str> = raw.lines().collect();
    assert_eq!(lines.first().copied(), Some("version=1"));
    assert!(lines[1].starts_with("dep=http|1.2.3|"));
    assert!(lines[2].starts_with("dep=std|0.1.0|"));
    for line in lines.iter().skip(1) {
        let hash = line
            .split('|')
            .next_back()
            .expect("hash segment must exist");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn m3_check_locked_fails_when_lock_missing() {
    let root = temp_project_dir("lock_missing");
    init_project(&root).expect("init");
    write_manifest_with_deps(&root, &[("std", "0.1.0")]);
    fs::remove_file(root.join("deps.lock")).expect("remove lock");

    let err = check_project_with_lock(&root, true).expect_err("locked check must fail");
    let msg = err.to_string();
    assert!(msg.contains("missing deps.lock"));
}

#[test]
fn m3_check_locked_passes_after_sync() {
    let root = temp_project_dir("lock_ok");
    init_project(&root).expect("init");
    write_manifest_with_deps(&root, &[("std", "0.1.0"), ("json", "0.2.0")]);
    sync_deps_lock_v1(&root).expect("sync");

    let summary = check_project_with_lock(&root, true).expect("locked check");
    assert!(summary.files_checked >= 1);
}

#[test]
fn m3_build_locked_manifest_contains_dep_entries() {
    let root = temp_project_dir("build_manifest");
    init_project(&root).expect("init");
    write_manifest_with_deps(&root, &[("std", "0.1.0")]);
    sync_deps_lock_v1(&root).expect("sync");

    let summary = build_project_with_lock(&root, true).expect("build locked");
    assert!(summary.files_bundled >= 1);

    let manifest = fs::read_to_string(root.join(".ocpbundle").join("manifest.txt"))
        .expect("read bundle manifest");
    assert!(manifest.contains("version=1"));
    assert!(manifest.contains("dep=std|0.1.0|"));
}
