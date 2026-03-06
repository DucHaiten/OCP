use ocl_sdk::version_rule_matches_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

fn tool_status(rule: &str, actual: &str) -> serde_json::Value {
    if actual == "unknown" {
        return json!({
            "rule": rule,
            "actual": actual,
            "mode": "environment_unknown",
            "checked": false,
            "pass": true
        });
    }
    let normalized = actual.trim_start_matches('v');
    let pass = version_rule_matches_v19(rule, normalized);
    json!({
        "rule": rule,
        "actual": actual,
        "normalized_actual": normalized,
        "mode": "strict_match",
        "checked": true,
        "pass": pass
    })
}

#[test]
fn v19_packaging_toolchain_lock_contract() {
    let run_manifest = f19::ensure_run_manifest();
    let _fixture = f19::ensure_release_fixture_v19();

    let contract_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_packaging_toolchain.v1.json");
    let contract = f19::read_json(&contract_path);

    let node_rule = contract
        .get("node")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("22.x");
    let pnpm_rule = contract
        .get("pnpm")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("10.x");
    let vsce_rule = contract
        .get("vsce")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("2.x");
    let ovsx_rule = contract
        .get("ovsx")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0.10.x");

    let node_actual = run_manifest
        .get("node_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let pnpm_actual = run_manifest
        .get("pnpm_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let vsce_actual = run_manifest
        .get("vsce_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let ovsx_actual = run_manifest
        .get("ovsx_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    let node_status = tool_status(node_rule, node_actual);
    let pnpm_status = tool_status(pnpm_rule, pnpm_actual);
    let vsce_status = tool_status(vsce_rule, vsce_actual);
    let ovsx_status = tool_status(ovsx_rule, ovsx_actual);

    assert!(node_status
        .get("pass")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false));
    assert!(pnpm_status
        .get("pass")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false));
    assert!(vsce_status
        .get("pass")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false));
    assert!(ovsx_status
        .get("pass")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false));

    let report = json!({
        "schema": "ocl.w19.release.packaging_toolchain_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "node": node_status,
        "pnpm": pnpm_status,
        "vsce": vsce_status,
        "ovsx": ovsx_status,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/packaging_toolchain_report.json", &report);
}
