use std::collections::BTreeSet;

use ocp_sdk::required_bundled_binaries_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_bundled_tooling_presence_from_bundle_only() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let contract_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_bundled_binaries.v1.json");
    let contract = f19::read_json(&contract_path);
    let required = required_bundled_binaries_v19(&contract).expect("required bundled binaries");
    let path_dependency_allowed = contract
        .get("path_dependency_allowed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    assert!(
        !path_dependency_allowed,
        "path dependency must remain disabled"
    );

    let manifest = f19::read_json(&fixture.manifest_path);
    let declared = manifest
        .get("binaries")
        .and_then(serde_json::Value::as_array)
        .expect("manifest binaries")
        .iter()
        .filter_map(|item| item.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<String>>();

    for name in &required {
        assert!(
            declared.contains(name),
            "manifest missing bundled binary `{name}`"
        );
        let path = fixture
            .binaries
            .get(name)
            .unwrap_or_else(|| panic!("fixture missing path for {name}"));
        assert!(
            path.exists(),
            "bundled binary file missing: {}",
            path.display()
        );
    }

    let report = json!({
        "schema": "ocp.w19.release.bundled_tooling_presence_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "required_binaries": required,
        "path_dependency_allowed": path_dependency_allowed,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/bundled_tooling_presence_report.json", &report);
}
