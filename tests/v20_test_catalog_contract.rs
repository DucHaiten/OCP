use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_test_catalog_contract() {
    v20::ensure_run_manifest();

    let path = v20::contracts_root()
        .join("v20")
        .join("test_catalog.v1.json");
    let value = v20::read_json(&path);

    assert_eq!(
        value.get("contract_id").and_then(serde_json::Value::as_str),
        Some("v20.test_catalog"),
        "test_catalog contract_id must be stable"
    );

    let policy = value
        .get("policy")
        .and_then(serde_json::Value::as_object)
        .expect("policy object");
    for key in [
        "deny_ignored_tests",
        "deny_test_filters",
        "deny_cfg_skips",
        "allow_skip_whitelist",
    ] {
        assert!(policy.contains_key(key), "missing policy key `{key}`");
    }
    assert_eq!(
        policy
            .get("deny_ignored_tests")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_ignored_tests must be true"
    );
    assert_eq!(
        policy
            .get("deny_test_filters")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_test_filters must be true"
    );
    assert_eq!(
        policy
            .get("deny_cfg_skips")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "deny_cfg_skips must be true"
    );

    let suites = value
        .get("suites")
        .and_then(serde_json::Value::as_array)
        .expect("suites");
    assert!(!suites.is_empty(), "test catalog suites must not be empty");

    let mut all_tests = Vec::<String>::new();
    for suite in suites {
        let tests = suite
            .get("tests")
            .and_then(serde_json::Value::as_array)
            .expect("suite.tests");
        for test_name in tests {
            all_tests.push(test_name.as_str().expect("test name string").to_string());
        }
    }
    all_tests.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    all_tests.dedup();
    assert!(
        all_tests.contains(&"v20_contract_chain".to_string()),
        "catalog must include v20_contract_chain"
    );
    assert!(
        all_tests.contains(&"v20_history_toolchain_matrix".to_string()),
        "catalog must include v20_history_toolchain_matrix"
    );

    let report = json!({
        "schema": "ocp.w20.test_catalog_report.v1",
        "status": "PASS",
        "suite_count": suites.len(),
        "test_count_unique": all_tests.len(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/test_catalog_report.json", &report);
}
