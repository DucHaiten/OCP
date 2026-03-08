use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    check_project_with_lock, init_project, install_organs_v1, resolve_platform_tag_v1,
    run_kit_doctor_v1, sync_cosmos_lock_v1, sync_deps_lock_v1, sync_organs_lock_v1,
    sync_policy_lock_v1, verify_organs_lock_v1,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v5_w6_{tag}_{stamp}"))
}

fn prepare_w6_organ_project(root: &Path) {
    init_project(root).expect("init project");
    let manifest = r#"[package]
name = "v5_w6_organs"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
    fs::write(root.join("Ocp.toml"), manifest).expect("write manifest");
    fs::write(
        root.join("src").join("main.ocp"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write source");
    sync_deps_lock_v1(root).expect("sync deps lock");
    sync_policy_lock_v1(root).expect("sync policy lock");

    let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[view]]
id = "text_default"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[kits]]
kit_id = "std.kit.view_text_basic"
universe_id = "ci_locked"
bind_domain = "default"
bind_view = "text_default"
required_organs = ["noop.organ@0.1.0"]
"#;
    fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    sync_cosmos_lock_v1(root, false, None, None, None).expect("sync cosmos lock");
}

fn write_organ_registry_fixture(root: &Path) -> PathBuf {
    let platform = resolve_platform_tag_v1();
    let registry_root = root.join("registry").join("organs");
    let artifact_rel = format!("artifacts/noop.organ/0.1.0/{platform}/noop.organ.bin");
    let artifact_path = registry_root.join(&artifact_rel);
    fs::create_dir_all(
        artifact_path
            .parent()
            .expect("artifact path must have parent"),
    )
    .expect("mkdir artifact dir");
    fs::write(&artifact_path, b"noop-organ-v5-w6").expect("write organ artifact");

    let index_path = registry_root.join("index.toml");
    fs::create_dir_all(&registry_root).expect("mkdir registry root");
    let index = format!(
        concat!(
            "[[organ]]\n",
            "pack_id = \"noop.organ\"\n",
            "version = \"0.1.0\"\n",
            "platform = \"{}\"\n",
            "artifact_rel = \"{}\"\n",
            "provides_caps = [\"std.io.noop\"]\n",
            "source = \"local\"\n",
            "abi_version = \"v1\"\n",
            "signer_id = \"dev-root-1\"\n"
        ),
        platform, artifact_rel
    );
    fs::write(&index_path, index).expect("write organ index");
    index_path
}

fn tamper_first_organ_signature(lock_path: &Path) {
    let raw = fs::read_to_string(lock_path).expect("read organs lock");
    let mut changed = false;
    let mut out = Vec::new();
    for line in raw.lines() {
        if !changed {
            if let Some(payload) = line.strip_prefix("organ=") {
                let mut parts = payload
                    .split('\t')
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>();
                if parts.len() == 11 {
                    parts[7] = "!".to_string();
                    out.push(format!("organ={}", parts.join("\t")));
                    changed = true;
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    fs::write(lock_path, format!("{}\n", out.join("\n"))).expect("write tampered lock");
}

#[test]
fn v5_w6_organ_lock_verify_install_and_doctor_pass() {
    let root = temp_project_dir("flow_pass");
    prepare_w6_organ_project(&root);
    let index = write_organ_registry_fixture(&root);

    let sync = sync_organs_lock_v1(&root, &index).expect("sync organs lock");
    assert_eq!(sync.organs_synced, 1);
    assert!(sync.lock_path.exists());

    let verify = verify_organs_lock_v1(&root, true).expect("verify organs lock");
    assert_eq!(verify.organs_verified, 1);

    let install =
        install_organs_v1(&root, &index, "noop.organ", "0.1.0", true).expect("install organ");
    assert!(
        install.install_path.exists(),
        "installed artifact must exist"
    );

    let doctor = run_kit_doctor_v1(&root, true).expect("kit doctor");
    assert_eq!(doctor.kits_checked, 1);
    assert_eq!(doctor.required_organs, vec!["noop.organ@0.1.0".to_string()]);
}

#[test]
fn v5_w6_verify_fails_when_signature_tampered() {
    let root = temp_project_dir("tampered_signature");
    prepare_w6_organ_project(&root);
    let index = write_organ_registry_fixture(&root);
    sync_organs_lock_v1(&root, &index).expect("sync organs lock");

    let lock_path = root.join("organs.lock.v1");
    tamper_first_organ_signature(&lock_path);

    let err = verify_organs_lock_v1(&root, true).expect_err("tampered signature must fail");
    assert!(
        err.to_string().contains("invalid organ signature base64"),
        "unexpected error: {err}"
    );
}

#[test]
fn v5_w6_locked_requires_organs_lock_when_required() {
    let root = temp_project_dir("lock_required");
    prepare_w6_organ_project(&root);

    let err = check_project_with_lock(&root, true).expect_err("locked check must fail");
    assert!(
        err.to_string().contains("V-ORGANS-LOCK-REQUIRED"),
        "unexpected error: {err}"
    );
}

#[test]
fn v5_w6_kit_doctor_detects_missing_required_organs_lock() {
    let root = temp_project_dir("doctor_missing_lock");
    prepare_w6_organ_project(&root);

    let err = run_kit_doctor_v1(&root, true).expect_err("kit doctor must fail without lock");
    let text = err.to_string();
    assert!(
        text.contains("V-KIT-DOCTOR-WIRING"),
        "unexpected error: {text}"
    );
    assert!(
        text.contains("V-ORGANS-LOCK-REQUIRED"),
        "unexpected error: {text}"
    );
}
