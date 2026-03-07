use serde_json::json;

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_quickstart_flow() {
    v100d::ensure_run_manifest();

    let readme = v100d::read_text(&v100d::repo_root().join("README.md"));
    assert!(readme.contains("## Install"));
    assert!(readme.contains("## Quickstart"));
    assert!(readme.contains("docs/vi/USER_GUIDE.md"));

    let vi_guide = v100d::read_text(&v100d::repo_root().join("docs").join("vi").join("USER_GUIDE.md"));
    let policy = v100d::docs_contract_language_parity();
    let en_required = policy
        .get("en_translation_required_for_current_gate")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let en_guide = if en_required {
        assert!(readme.contains("docs/en/USER_GUIDE.md"));
        Some(v100d::read_text(
            &v100d::repo_root().join("docs").join("en").join("USER_GUIDE.md"),
        ))
    } else {
        None
    };

    let (label, content) = ("vi", &vi_guide);
    assert!(
        content.contains("ocl init"),
        "{label} user guide must include init in quickstart flow"
    );
    assert!(
        content.contains("ocl lock sync"),
        "{label} user guide must include lock sync in quickstart flow"
    );
    assert!(
        content.contains("ocl run"),
        "{label} user guide must include run in quickstart flow"
    );
    assert!(
        content.contains("ocl replay"),
        "{label} user guide must include replay in quickstart flow"
    );
    if let Some(en_guide) = en_guide.as_ref() {
        let (label, content) = ("en", en_guide);
        assert!(
            content.contains("ocl init"),
            "{label} user guide must include init in quickstart flow"
        );
        assert!(
            content.contains("ocl lock sync"),
            "{label} user guide must include lock sync in quickstart flow"
        );
        assert!(
            content.contains("ocl run"),
            "{label} user guide must include run in quickstart flow"
        );
        assert!(
            content.contains("ocl replay"),
            "{label} user guide must include replay in quickstart flow"
        );
    }

    let report = json!({
        "schema": "ocl.w100.docs.quickstart_report.v1",
        "status": "PASS",
        "readme": "README.md",
        "vi_user_guide": "docs/vi/USER_GUIDE.md",
        "en_user_guide": if en_required { json!("docs/en/USER_GUIDE.md") } else { serde_json::Value::Null },
        "en_translation_required_for_current_gate": en_required,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_quickstart_report.json", &report);
}
