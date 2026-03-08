use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_run_manifest_policy() {
    let manifest = v20::ensure_run_manifest();
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
        "required_contracts_hash",
        "test_catalog_hash",
        "fuzz_seed_suite_id",
        "tooling_versions_id",
        "toolchain_matrix_hash",
        "threat_model_id",
        "dependency_mode",
        "node_version",
        "pnpm_version",
        "pnpm_lock_hash",
        "vsce_version",
        "ovsx_version",
        "editor_extension_version",
        "ocp_cli_version",
        "lsp_server_version",
        "dap_server_version",
        "signing_trust_root_id",
        "signing_trust_epoch",
    ] {
        assert!(root.contains_key(key), "missing run-manifest field `{key}`");
    }

    let allowed = root["allowed_env_flags"]
        .as_array()
        .expect("allowed env array")
        .iter()
        .map(|item| item.as_str().expect("allowed env str").to_string())
        .collect::<Vec<String>>();
    let observed = root["observed_env_flags"]
        .as_array()
        .expect("observed env array")
        .iter()
        .map(|item| item.as_str().expect("observed env str").to_string())
        .collect::<Vec<String>>();
    v20::enforce_run_manifest_allowlist(&allowed, &observed)
        .expect("observed env must respect allowlist");

    let report = json!({
        "schema": "ocp.w20.run_manifest_policy_report.v1",
        "status": "PASS",
        "field_count": root.len(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/run_manifest_policy_report.json", &report);
}
