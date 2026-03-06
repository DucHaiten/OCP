#[path = "v20_gate_h_common.rs"]
mod v20h;

#[test]
fn v20_signoff_required_artifacts() {
    v20h::ensure_run_manifest();
    let _ = v20h::ensure_final_signoff_bundle();

    let required = v20h::signoff_required_artifacts_contract()
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();
    assert!(
        !required.is_empty(),
        "signoff_required_artifacts contract must not be empty"
    );

    let mut missing = Vec::<String>::new();
    for rel in required {
        let full = v20h::repo_root().join(&rel);
        if !full.exists() {
            missing.push(rel);
        }
    }
    assert!(
        missing.is_empty(),
        "missing required signoff artifacts: {missing:?}"
    );
}
