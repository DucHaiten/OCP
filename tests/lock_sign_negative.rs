use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    init_project, resolve_deps_v3, sign_deps_lock_v3_v15, sync_deps_lock_v1,
    verify_deps_lock_v3_signature_v15,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v16_lock_sign_negative_{tag}_{stamp}"))
}

fn prepare_basic_project(root: &Path) {
    init_project(root).expect("init");
    let manifest = concat!(
        "[package]\n",
        "name = \"lock_sign_negative\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n"
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
}

fn tamper_first_dep(lock_path: &Path) {
    let raw = fs::read_to_string(lock_path).expect("read lock");
    let mut out = Vec::<String>::new();
    let mut tampered = false;
    for line in raw.lines() {
        if !tampered {
            if let Some(payload) = line.strip_prefix("dep=") {
                let mut parts = payload
                    .split('|')
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                if parts.len() >= 5 {
                    parts[4].push('x');
                    out.push(format!("dep={}", parts.join("|")));
                    tampered = true;
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    fs::write(lock_path, format!("{}\n", out.join("\n"))).expect("write tampered lock");
}

#[test]
fn lock_sign_negative_tampered_lockfile_must_fail_verify() {
    let root = temp_project_dir("tamper");
    prepare_basic_project(&root);
    sign_deps_lock_v3_v15(&root, "ci-negative").expect("sign deps.lock.v3");

    let lock_path = root.join("deps.lock.v3");
    tamper_first_dep(&lock_path);

    let err = verify_deps_lock_v3_signature_v15(&root).expect_err("tampered lock must fail");
    assert!(
        err.to_string().contains("X-LOCK-SIGNATURE-MISMATCH"),
        "unexpected error: {err}"
    );
}
