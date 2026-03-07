use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_language_policy() {
    v100d::ensure_run_manifest();

    let policy = v100d::docs_contract_language_parity();
    assert_eq!(
        policy
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.docs_language_parity_policy"
    );
    assert_eq!(
        policy
            .get("readme_mode")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "bilingual_single_file"
    );
    assert_eq!(
        policy
            .get("docs_mode")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "split_vi_en"
    );

    let readme = v100d::read_text(&v100d::repo_root().join("README.md"));
    assert!(readme.contains("**English:**"));
    assert!(readme.contains("**Tiếng Việt:**"));

    let vi = v100d::repo_root().join("docs").join("vi").join("USER_GUIDE.md");
    let en = v100d::repo_root().join("docs").join("en").join("USER_GUIDE.md");
    assert!(vi.exists(), "missing {}", vi.display());
    let en_required = policy
        .get("en_translation_required_for_current_gate")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    if en_required {
        assert!(en.exists(), "missing {}", en.display());
    }
    assert!(
        policy
            .get("vi_is_source_of_truth")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
    );

    let report = json!({
        "schema": "ocl.w100.docs.language_policy_report.v1",
        "status": "PASS",
        "readme_mode": "bilingual_single_file",
        "docs_mode": "split_vi_en",
        "current_translation_stage": policy.get("current_translation_stage").cloned().unwrap_or(JsonValue::Null),
        "en_translation_required_for_current_gate": en_required,
        "vi_user_guide": "docs/vi/USER_GUIDE.md",
        "en_user_guide": if en.exists() { JsonValue::String("docs/en/USER_GUIDE.md".to_string()) } else { JsonValue::Null },
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_language_policy_report.json", &report);
}
