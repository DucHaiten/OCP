use ocl_sdk::version_rule_matches_v19;
use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

fn normalized(actual: &str) -> String {
    actual.trim().trim_start_matches('v').to_string()
}

#[test]
fn v100_packaging_toolchain_lock() {
    v100b::ensure_release_fixture_v100();
    let run_manifest = v100b::read_json(
        &v100b::repo_root()
            .join("target")
            .join("ocl")
            .join("w100")
            .join("meta")
            .join("run_manifest.json"),
    );
    let contract = v100b::packaging_toolchain_contract();

    let node_rule = contract
        .get("node")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    let pnpm_rule = contract
        .get("pnpm")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    let vsce_rule = contract
        .get("vsce")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    let ovsx_rule = contract
        .get("ovsx")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");

    let node_actual = normalized(
        run_manifest
            .get("node_version")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
    );
    let pnpm_actual = normalized(
        run_manifest
            .get("pnpm_version")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
    );
    let vsce_actual = normalized(
        run_manifest
            .get("vsce_version")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
    );
    let ovsx_actual = normalized(
        run_manifest
            .get("ovsx_version")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
    );

    let node_ok = version_rule_matches_v19(node_rule, &node_actual);
    let pnpm_ok = version_rule_matches_v19(pnpm_rule, &pnpm_actual);
    let vsce_ok = version_rule_matches_v19(vsce_rule, &vsce_actual);
    let ovsx_ok = version_rule_matches_v19(ovsx_rule, &ovsx_actual);
    assert!(
        node_ok,
        "node version `{node_actual}` violates rule `{node_rule}`"
    );
    assert!(
        pnpm_ok,
        "pnpm version `{pnpm_actual}` violates rule `{pnpm_rule}`"
    );
    assert!(
        vsce_ok,
        "vsce version `{vsce_actual}` violates rule `{vsce_rule}`"
    );
    assert!(
        ovsx_ok,
        "ovsx version `{ovsx_actual}` violates rule `{ovsx_rule}`"
    );

    let report = json!({
        "schema": "ocl.w100.release.packaging_toolchain_report.v1",
        "status": "PASS",
        "toolchain": [
            {"tool": "node", "rule": node_rule, "actual": node_actual, "match": node_ok},
            {"tool": "pnpm", "rule": pnpm_rule, "actual": pnpm_actual, "match": pnpm_ok},
            {"tool": "vsce", "rule": vsce_rule, "actual": vsce_actual, "match": vsce_ok},
            {"tool": "ovsx", "rule": ovsx_rule, "actual": ovsx_actual, "match": ovsx_ok}
        ],
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/packaging_toolchain_report.json", &report);
}
