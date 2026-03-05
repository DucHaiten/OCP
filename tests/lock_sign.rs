use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    check_project_with_lock, init_project, resolve_deps_v3, sign_deps_lock_v3_v15,
    sync_deps_lock_v1, verify_deps_lock_v3_signature_v15,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v15_lock_sign_{tag}_{stamp}"))
}

fn prepare_basic_project(root: &Path) {
    init_project(root).expect("init");
    let manifest = concat!(
        "[package]\n",
        "name = \"lock_sign_basic\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"*\"]\n",
        "deny = [\"std.net.poll\"]\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
}

fn prepare_locked_project_with_registry_dep(root: &Path) -> PathBuf {
    init_project(root).expect("init");
    let manifest = concat!(
        "[package]\n",
        "name = \"lock_sign_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n",
        "widgets = { name=\"engine-ui-widgets\", version=\"0.3.*\", source=\"registry\" }\n\n",
        "[permissions.package]\n",
        "allow = [\"*\"]\n",
        "deny = [\"std.net.poll\"]\n",
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
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
    .expect("write package.oclp");
    fs::write(
        root.join("deps")
            .join("widgets")
            .join("src")
            .join("widget.ocl"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write dep source");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
    root.join("deps.lock.v3")
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
    panic!("non-builtin signer not found");
}

fn tamper_first_dep_hash(lock_path: &Path) {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    let mut out = Vec::<String>::new();
    let mut tampered = false;
    for line in raw.lines() {
        if !tampered {
            if let Some(payload) = line.strip_prefix("dep=") {
                let mut parts: Vec<String> = payload.split('|').map(|s| s.to_string()).collect();
                if parts.len() == 11 {
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
fn v15_lock_sign_and_verify_roundtrip_passes() {
    let root = temp_project_dir("roundtrip");
    prepare_basic_project(&root);

    let sign = sign_deps_lock_v3_v15(&root, "ci-local-key").expect("sign lock");
    assert!(sign.sig_path.exists(), "deps.lock.v3.sig must be created");

    let verify = verify_deps_lock_v3_signature_v15(&root).expect("verify signature");
    assert_eq!(verify.key_id, "ci-local-key");
    assert_eq!(verify.lock_ast_hash_sha256, sign.lock_ast_hash_sha256);
}

#[test]
fn v15_lock_verify_detects_tampered_lockfile() {
    let root = temp_project_dir("tamper_lock");
    prepare_basic_project(&root);
    sign_deps_lock_v3_v15(&root, "ci-local-key").expect("sign lock");

    let lock_path = root.join("deps.lock.v3");
    tamper_first_dep_hash(&lock_path);

    let err = verify_deps_lock_v3_signature_v15(&root).expect_err("tampered lock must fail");
    assert!(
        err.to_string().contains("X-LOCK-SIGNATURE-MISMATCH"),
        "unexpected error: {err}"
    );
}

#[test]
fn v15_locked_v071_can_require_signed_lock_via_policy_flag() {
    let root = temp_project_dir("require_signed_lock");
    let lock_path = prepare_locked_project_with_registry_dep(&root);
    let signer = read_non_builtin_signer(&lock_path);
    let trust = format!(
        concat!(
            "[policy]\n",
            "mode = \"strict\"\n",
            "require_signed_lock = true\n\n",
            "[trusted_signers.registry]\n",
            "keys = [\"{}\"]\n"
        ),
        signer
    );
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    let err = check_project_with_lock(&root, true)
        .expect_err("locked_v071 with require_signed_lock=true must fail when unsigned lock");
    assert!(
        err.to_string().contains("missing deps.lock.v3.sig"),
        "unexpected error: {err}"
    );

    sign_deps_lock_v3_v15(&root, "ci-local-key").expect("sign lock");
    check_project_with_lock(&root, true).expect("signed lock should pass in locked_v071");
}
