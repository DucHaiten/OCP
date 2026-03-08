use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    build_project_with_lock, check_project_with_lock, init_project, run_project_with_lock,
    sync_deps_lock_v1, sync_deps_lock_v2, test_project_with_lock,
};
use sha2::{Digest, Sha256};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_t_wf_001_{tag}_{stamp}"))
}

fn file_hash_hex(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

fn state_fingerprint(root: &Path) -> String {
    let mut entries = Vec::<String>::new();
    let tracked = [
        root.join("Ocp.toml"),
        root.join("deps.lock"),
        root.join("deps.lock.v2"),
        root.join("src").join("main.ocp"),
        root.join("tests").join("smoke.ocp"),
        root.join(".ocpbundle").join("manifest.txt"),
        root.join(".ocpbundle").join("files").join("src").join("main.ocp"),
        root.join(".ocpbundle").join("files").join("tests").join("smoke.ocp"),
    ];
    for path in tracked {
        if path.exists() {
            let rel = path
                .strip_prefix(root)
                .expect("tracked path must be under root")
                .to_string_lossy()
                .replace('\\', "/");
            let digest = file_hash_hex(&path);
            entries.push(format!("{rel}:{digest}"));
        }
    }
    entries.sort();
    let mut hasher = Sha256::new();
    for line in entries {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

#[test]
fn t_wf_001_workflow_three_rounds_is_idempotent() {
    let root = temp_project_dir("idempotent");
    init_project(&root).expect("init project");
    fs::write(
        root.join("Ocp.toml"),
        concat!(
            "[package]\n",
            "name = \"t_wf_001\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = [\"std.net.poll\"]\n"
        ),
    )
    .expect("write locked-compatible manifest");
    sync_deps_lock_v1(&root).expect("sync lock v1");
    sync_deps_lock_v2(&root).expect("sync lock v2");

    let mut fingerprints = Vec::<String>::new();
    for round in 1..=3 {
        let check = check_project_with_lock(&root, true).expect("check locked");
        assert!(check.files_checked >= 1, "round {round}: check files_checked");

        let run = run_project_with_lock(&root, true).expect("run locked");
        assert!(run.steps >= 1, "round {round}: run steps");

        let tests = test_project_with_lock(&root, true).expect("test locked");
        assert!(tests.tests_run >= 1, "round {round}: tests_run");

        let build = build_project_with_lock(&root, true).expect("build locked");
        assert!(build.files_bundled >= 1, "round {round}: files_bundled");

        fingerprints.push(state_fingerprint(&root));
    }

    assert_eq!(
        fingerprints[0], fingerprints[1],
        "state drift detected between round1 and round2"
    );
    assert_eq!(
        fingerprints[1], fingerprints[2],
        "state drift detected between round2 and round3"
    );
}
