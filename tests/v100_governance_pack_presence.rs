use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_governance_pack_presence() {
    v100f::ensure_run_manifest();

    let contract = v100f::governance_policy();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.governance_policy"
    );

    let governance_file = contract
        .get("governance_file")
        .and_then(JsonValue::as_str)
        .unwrap_or("GOVERNANCE.md");
    let governance_path = v100f::repo_root().join(governance_file);
    assert!(governance_path.exists(), "missing governance file");

    if contract
        .get("codeowners_required")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        assert!(
            v100f::repo_root().join("CODEOWNERS").exists(),
            "missing CODEOWNERS"
        );
    }

    let vi_doc = v100f::repo_root().join("docs/vi/legal/governance.md");
    let en_doc = v100f::repo_root().join("docs/en/legal/governance.md");
    assert!(vi_doc.exists(), "missing docs/vi/legal/governance.md");
    assert!(en_doc.exists(), "missing docs/en/legal/governance.md");

    let governance = v100f::read_text(&governance_path);
    assert!(
        governance.contains("owner-led")
            && governance.contains("Release authority")
            && governance.contains("Official build"),
        "GOVERNANCE.md must describe owner-led governance, release authority, and official build"
    );

    let vi = v100f::read_text(&vi_doc);
    let en = v100f::read_text(&en_doc);
    assert!(
        vi.contains("owner authority") || vi.contains("owner authority rõ"),
        "vi governance explainer must mention owner authority"
    );
    assert!(
        en.contains("owner authority") && en.contains("trust chain"),
        "en governance explainer must mention owner authority and trust chain"
    );

    let report = json!({
        "schema": "ocl.w100.legal.governance_pack_report.v1",
        "status": "PASS",
        "governance_file": governance_file,
        "codeowners_required": true,
        "docs_verified": [
            "docs/vi/legal/governance.md",
            "docs/en/legal/governance.md"
        ],
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/governance_pack_report.json", &report);
}
