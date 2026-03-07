use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_required_sections() {
    v100d::ensure_run_manifest();

    let contract = v100d::docs_contract_required_sections();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.docs_required_sections"
    );

    let readme_path = v100d::repo_root().join("README.md");
    let readme = v100d::read_text(&readme_path);
    let mut readme_ok = Vec::<String>::new();
    for section in contract
        .get("readme_required_sections")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
    {
        let heading = format!("## {section}");
        assert!(
            readme.contains(&heading),
            "README missing required section `{section}`"
        );
        readme_ok.push(section.to_string());
    }

    let vi_guide = v100d::read_text(&v100d::repo_root().join("docs").join("vi").join("USER_GUIDE.md"));
    let language_policy = v100d::docs_contract_language_parity();
    let en_required = language_policy
        .get("en_translation_required_for_current_gate")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    let en_guide = if en_required {
        Some(v100d::read_text(
            &v100d::repo_root().join("docs").join("en").join("USER_GUIDE.md"),
        ))
    } else {
        None
    };
    let mut guide_ok = Vec::<String>::new();
    for section in contract
        .get("user_guide_required_sections")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(JsonValue::as_str)
    {
        assert!(
            vi_guide.contains(section),
            "docs/vi/USER_GUIDE.md missing section `{section}`"
        );
        if let Some(en_guide) = en_guide.as_ref() {
            assert!(
                en_guide.contains(section),
                "docs/en/USER_GUIDE.md missing section `{section}`"
            );
        }
        guide_ok.push(section.to_string());
    }

    let report = json!({
        "schema": "ocl.w100.docs.docs_coverage_report.v1",
        "status": "PASS",
        "readme_sections_verified": readme_ok,
        "user_guide_sections_verified": guide_ok,
        "en_translation_required_for_current_gate": en_required,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_coverage_report.json", &report);
}
