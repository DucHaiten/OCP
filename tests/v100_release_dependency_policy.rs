use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_dependency_policy() {
    v100b::ensure_release_fixture_v100();
    let policy = v100b::release_dependency_policy();
    assert_eq!(
        policy
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.release_dependency_policy"
    );

    let allowed = policy
        .get("release_build_dependency_mode_allowed")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        allowed.iter().any(|v| v == "vendored"),
        "allowed dependency modes must include vendored"
    );
    assert!(
        allowed.iter().any(|v| v == "pinned_cache"),
        "allowed dependency modes must include pinned_cache"
    );

    let deny_channels = policy
        .get("deny_allow_network_channels")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        deny_channels.iter().any(|v| v == "github_release"),
        "deny list must include github_release"
    );
    assert!(
        deny_channels.iter().any(|v| v == "staging_rehearsal"),
        "deny list must include staging_rehearsal"
    );

    let run_manifest = v100b::read_json(
        &v100b::repo_root()
            .join("target")
            .join("ocp")
            .join("w100")
            .join("meta")
            .join("run_manifest.json"),
    );
    let dependency_mode = run_manifest
        .get("dependency_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string();
    assert!(
        allowed.iter().any(|v| v == &dependency_mode),
        "run_manifest dependency_mode `{dependency_mode}` is outside allowed set"
    );
    assert_ne!(
        dependency_mode, "allow_network",
        "release dependency mode must not be allow_network"
    );

    let report = json!({
        "schema": "ocp.w100.release.release_dependency_policy_report.v1",
        "status": "PASS",
        "dependency_mode": dependency_mode,
        "allowed_modes": allowed,
        "deny_allow_network_channels": deny_channels,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_dependency_policy_report.json", &report);
}
