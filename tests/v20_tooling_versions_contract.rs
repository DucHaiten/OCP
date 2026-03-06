use std::collections::BTreeSet;

use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_tooling_versions_contract() {
    v20::ensure_run_manifest();

    let path = v20::contracts_root()
        .join("v20")
        .join("tooling_versions.v1.json");
    let value = v20::read_json(&path);

    assert_eq!(
        value.get("contract_id").and_then(serde_json::Value::as_str),
        Some("v20.tooling_versions"),
        "tooling_versions contract_id must be stable"
    );
    let tooling_id = value
        .get("tooling_id")
        .and_then(serde_json::Value::as_str)
        .expect("tooling_id");
    assert!(
        !tooling_id.trim().is_empty(),
        "tooling_id must not be empty"
    );

    let required = value
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("required tooling list");
    assert!(
        !required.is_empty(),
        "required tooling list must not be empty"
    );

    let mut required_tools = BTreeSet::<String>::new();
    for item in required {
        let tool = item
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .expect("required tool");
        let rule = item
            .get("version_rule")
            .and_then(serde_json::Value::as_str)
            .expect("required version_rule");
        assert!(
            !rule.trim().is_empty(),
            "version_rule must not be empty for tool `{tool}`"
        );
        required_tools.insert(tool.to_string());
    }
    for required_tool in ["cargo", "rustc", "node", "pnpm"] {
        assert!(
            required_tools.contains(required_tool),
            "required tooling must include `{required_tool}`"
        );
    }

    let external = value
        .get("external")
        .and_then(serde_json::Value::as_array)
        .expect("external tooling list");
    assert!(
        !external.is_empty(),
        "external tooling list must not be empty"
    );

    let report = json!({
        "schema": "ocl.w20.tooling_versions_report.v1",
        "status": "PASS",
        "tooling_id": tooling_id,
        "required_count": required.len(),
        "external_count": external.len(),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/tooling_versions_report.json", &report);
}
