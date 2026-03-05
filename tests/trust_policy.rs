use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{check_project_with_lock, init_project, resolve_deps_v3, sync_deps_lock_v1};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v15_trust_policy_{tag}_{stamp}"))
}

fn write_manifest_with_dep(root: &Path, lane: &str, source: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"trust_policy_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"{}\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n",
            "widgets = {{ name=\"engine-ui-widgets\", version=\"0.3.*\", source=\"{}\" }}\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = [\"std.net.poll\"]\n"
        ),
        lane, source
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

fn prepare_project(root: &Path, lane: &str, source: &str) -> PathBuf {
    init_project(root).expect("init");
    write_manifest_with_dep(root, lane, source);
    fs::create_dir_all(root.join("deps").join("widgets").join("src")).expect("create dep src");
    fs::write(
        root.join("deps").join("widgets").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"engine-ui-widgets\"\n",
            "version = \"0.3.0\"\n",
            "entry = \"src/widget.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"widget\"]\n"
        ),
    )
    .expect("write dep package.oclp");
    fs::write(
        root.join("deps")
            .join("widgets")
            .join("src")
            .join("widget.ocl"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write dep src");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    let summary = resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
    summary.lock_v3_path
}

fn read_non_builtin_signer(lock_path: &Path) -> String {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    for line in raw.lines() {
        let Some(payload) = line.strip_prefix("dep=") else {
            continue;
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() == 11 && parts[3] != "builtin" {
            return parts[7].to_string();
        }
    }
    panic!("non-builtin signer not found in deps.lock.v3");
}

#[test]
fn v15_locked_v071_rejects_warn_mode_in_trust_policy() {
    let root = temp_project_dir("reject_warn_mode");
    let lock_path = prepare_project(&root, "locked_v071", "registry");
    let signer = read_non_builtin_signer(&lock_path);
    let trust = format!(
        concat!(
            "[policy]\n",
            "mode = \"warn\"\n\n",
            "[trusted_signers.registry]\n",
            "keys = [\"{}\"]\n"
        ),
        signer
    );
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    let err = check_project_with_lock(&root, true).expect_err("locked_v071 must reject warn mode");
    assert!(
        err.to_string().contains("X-TRUST-POLICY-MODE-INVALID"),
        "unexpected error: {err}"
    );
}

#[test]
fn v15_locked_v071_accepts_trusted_key_with_scope() {
    let root = temp_project_dir("accept_scoped_trusted_key");
    let lock_path = prepare_project(&root, "locked_v071", "registry");
    let signer = read_non_builtin_signer(&lock_path);
    let trust = format!(
        concat!(
            "[policy]\n",
            "mode = \"strict\"\n\n",
            "[registries]\n",
            "allow = [\"registry\"]\n\n",
            "[[trusted_key]]\n",
            "id = \"publisher:engine-team\"\n",
            "alg = \"ed25519\"\n",
            "public_key = \"{}\"\n",
            "scope = [\"engine-ui-widgets\"]\n"
        ),
        signer
    );
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    check_project_with_lock(&root, true).expect("trusted scoped key should pass");
}

#[test]
fn v15_locked_v071_rejects_dependency_source_outside_allowlist() {
    let root = temp_project_dir("deny_source_outside_allowlist");
    let lock_path = prepare_project(&root, "locked_v071", "path");
    let signer = read_non_builtin_signer(&lock_path);
    let trust = format!(
        concat!(
            "[policy]\n",
            "mode = \"strict\"\n\n",
            "[registries]\n",
            "allow = [\"registry\"]\n\n",
            "[trusted_signers.path]\n",
            "keys = [\"{}\"]\n"
        ),
        signer
    );
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    let err = check_project_with_lock(&root, true)
        .expect_err("locked_v071 must deny source not in registry allowlist");
    assert!(
        err.to_string().contains("X-TRUST-REGISTRY-DENIED"),
        "unexpected error: {err}"
    );
}
