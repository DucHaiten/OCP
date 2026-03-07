use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_commercial_terms_pointer_repo_path() {
    v100g::ensure_run_manifest();

    let contract = v100g::licensing_model();
    let pointer = contract
        .get("commercial_terms_pointer")
        .and_then(JsonValue::as_object)
        .unwrap_or_else(|| panic!("commercial_terms_pointer must be object"));
    let vi = pointer.get("vi").and_then(JsonValue::as_str).unwrap_or("");
    let en = pointer.get("en").and_then(JsonValue::as_str).unwrap_or("");

    assert_eq!(vi, "docs/vi/legal/commercial.md");
    assert_eq!(en, "docs/en/legal/commercial.md");
    for rel in [vi, en] {
        assert!(
            !rel.starts_with("http://") && !rel.starts_with("https://"),
            "commercial_terms_pointer must stay inside repo"
        );
        assert!(
            v100g::repo_root().join(rel).exists(),
            "commercial_terms_pointer target `{rel}` must exist"
        );
    }

    let report = json!({
        "schema": "ocl.w100.business.commercial_terms_pointer_report.v1",
        "status": "PASS",
        "commercial_terms_pointer": {
            "vi": vi,
            "en": en
        },
        "repo_internal_only": true,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/commercial_terms_pointer_report.json", &report);
}
