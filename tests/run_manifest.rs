use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "v16_gate_a_common.rs"]
mod v16;

fn temp_project(tag: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ocp_v16_run_manifest_{tag}_{stamp}"));
    fs::create_dir_all(&root).expect("create temp project");
    root
}

#[test]
fn run_manifest_contains_required_fields_and_derived_lane() {
    let project = temp_project("lane");
    fs::write(
        project.join("Ocp.toml"),
        "[project]\nname = \"tmp\"\nentry = \"main.ocp\"\nlane = \"quarantine\"\n",
    )
    .expect("write Ocp.toml");
    fs::write(project.join("deps.lock.v3"), "version = 3\n").expect("write deps.lock.v3");

    let allowed_env_flags = vec![
        "OCP_QUARANTINE".to_string(),
        "OCP_ALLOW_OVERRIDES".to_string(),
    ];
    let observed_env_flags = vec!["OCP_QUARANTINE".to_string()];
    let manifest = v16::build_run_manifest(&project, allowed_env_flags.clone(), observed_env_flags);

    let root = manifest.as_object().expect("manifest object");
    for key in [
        "git_commit",
        "rustc_version_verbose",
        "cargo_version",
        "target_triple",
        "os",
        "arch",
        "lane_profile",
        "allowed_env_flags",
        "observed_env_flags",
        "deps_lock_v3_hash",
        "cargo_lock_hash",
    ] {
        assert!(root.contains_key(key), "missing run-manifest field `{key}`");
    }
    assert_eq!(
        root["lane_profile"].as_str(),
        Some("quarantine"),
        "lane_profile must be derived from project manifest"
    );

    v16::enforce_run_manifest_allowlist(&allowed_env_flags, &["OCP_QUARANTINE".to_string()])
        .expect("allowlist check");

    let out_path = v16::w16_target_root()
        .join("meta")
        .join("run_manifest.json");
    v16::write_json_pretty(&out_path, &manifest);
}
