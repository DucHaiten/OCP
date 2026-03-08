use std::collections::BTreeMap;
use std::fs;

use serde_json::json;

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_pack_build_sign_verify_enforces_signature_chain() {
    common::ensure_run_manifest();

    let root = common::temp_project_dir("pack_build_sign_verify");
    let root_s = root.to_string_lossy().to_string();
    let envs = BTreeMap::new();

    let init = common::run_ocp_cli(&["init", &root_s, "--template", "dep-permission"], &envs);
    let _ = common::assert_ok(&init, "init dep-permission");

    let resolve = common::run_ocp_cli(&["deps", "resolve", &root_s], &envs);
    let _ = common::assert_ok(&resolve, "deps resolve");

    let build = common::run_ocp_cli(&["pack", "build", &root_s], &envs);
    let _ = common::assert_ok(&build, "pack build");
    let artifact = common::first_ocppkg(&root);
    let artifact_s = artifact.to_string_lossy().to_string();

    let mut tampered = fs::read_to_string(&artifact).expect("read artifact");
    tampered = tampered.replace("signature_ed25519=", "signature_ed25519=broken-");
    fs::write(&artifact, tampered).expect("write tampered artifact");

    let verify_fail = common::run_ocp_cli(&["pack", "verify", &artifact_s], &envs);
    let verify_fail_stderr = common::assert_fail(&verify_fail, "pack verify tampered");

    let sign = common::run_ocp_cli(&["pack", "sign", &artifact_s], &envs);
    let _ = common::assert_ok(&sign, "pack sign");

    let verify_ok = common::run_ocp_cli(&["pack", "verify", &artifact_s], &envs);
    let _ = common::assert_ok(&verify_ok, "pack verify signed");

    common::merge_pack_shipproof_section(
        "pack_build_sign_verify",
        json!({
            "status": "PASS",
            "artifact": artifact.to_string_lossy().replace('\\', "/"),
            "tamper_verify_failed": verify_fail_stderr.contains("signature"),
            "sign_and_verify_passed": true
        }),
    );
}
