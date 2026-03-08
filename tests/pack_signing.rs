use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{init_project, resolve_deps_v3, verify_deps_signing_and_trust_v10};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v10_pack_signing_{tag}_{stamp}"))
}

fn write_manifest_with_registry_dep(root: &Path, lane: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"pack_signing_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"{}\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n",
            "widgets = {{ name=\"engine-ui-widgets\", version=\"0.3.*\", source=\"registry\" }}\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = [\"std.net.poll\"]\n"
        ),
        lane
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");
}

fn tamper_non_builtin_signature(lock_path: &Path) {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    let mut out = String::new();
    for line in raw.lines() {
        if let Some(payload) = line.strip_prefix("dep=") {
            let mut parts: Vec<String> = payload.split('|').map(|v| v.to_string()).collect();
            if parts.len() == 11 && parts[3] != "builtin" {
                parts[6] = "ZmFrZQ==".to_string();
                out.push_str("dep=");
                out.push_str(&parts.join("|"));
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    fs::write(lock_path, out).expect("write tampered deps.lock.v3");
}

#[test]
fn v10_pack_signing_verifier_accepts_resolved_lock() {
    let root = temp_project_dir("ok");
    init_project(&root).expect("init");
    write_manifest_with_registry_dep(&root, "locked_v071");
    resolve_deps_v3(&root, false).expect("resolve deps v3");

    verify_deps_signing_and_trust_v10(&root, false).expect("verify lock signing");
}

#[test]
fn v10_pack_signing_verifier_rejects_tampered_signature() {
    let root = temp_project_dir("tampered");
    init_project(&root).expect("init");
    write_manifest_with_registry_dep(&root, "locked_v071");
    let summary = resolve_deps_v3(&root, false).expect("resolve deps v3");
    tamper_non_builtin_signature(&summary.lock_v3_path);

    let err = verify_deps_signing_and_trust_v10(&root, false).expect_err("must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("signature"),
        "unexpected error (must mention signature): {msg}"
    );
}
