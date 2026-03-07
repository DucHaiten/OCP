use serde_json::json;

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_troubleshooting_coverage() {
    v100d::ensure_run_manifest();

    let vi = v100d::read_text(
        &v100d::repo_root()
            .join("docs")
            .join("vi")
            .join("USER_GUIDE.md"),
    );
    let policy = v100d::docs_contract_language_parity();
    let en_required = policy
        .get("en_translation_required_for_current_gate")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let en = if en_required {
        Some(v100d::read_text(
            &v100d::repo_root()
                .join("docs")
                .join("en")
                .join("USER_GUIDE.md"),
        ))
    } else {
        None
    };

    let (label, content) = ("vi", &vi);
    assert!(
        content.contains("Troubleshooting"),
        "{label} user guide must contain troubleshooting section"
    );
    assert!(
        content.contains("INSUFFICIENT"),
        "{label} user guide must mention INSUFFICIENT troubleshooting"
    );
    assert!(
        content.to_ascii_lowercase().contains("replay"),
        "{label} user guide must mention replay troubleshooting"
    );
    assert!(
        content.to_ascii_lowercase().contains("editor"),
        "{label} user guide must mention editor troubleshooting"
    );
    if let Some(en) = en.as_ref() {
        let (label, content) = ("en", en);
        assert!(
            content.contains("Troubleshooting"),
            "{label} user guide must contain troubleshooting section"
        );
        assert!(
            content.contains("INSUFFICIENT"),
            "{label} user guide must mention INSUFFICIENT troubleshooting"
        );
        assert!(
            content.to_ascii_lowercase().contains("replay"),
            "{label} user guide must mention replay troubleshooting"
        );
        assert!(
            content.to_ascii_lowercase().contains("editor"),
            "{label} user guide must mention editor troubleshooting"
        );
    }

    let report = json!({
        "schema": "ocl.w100.docs.troubleshooting_coverage_report.v1",
        "status": "PASS",
        "files_verified": [
            "docs/vi/USER_GUIDE.md"
        ],
        "en_translation_required_for_current_gate": en_required,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_troubleshooting_coverage_report.json", &report);
}
