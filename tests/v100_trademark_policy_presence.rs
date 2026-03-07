use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_trademark_policy_presence() {
    v100f::ensure_run_manifest();

    let contract = v100f::trademark_policy();
    assert_eq!(
        contract
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.trademark_policy"
    );

    let trademark_root = v100f::repo_root().join("TRADEMARK.md");
    let vi_doc = v100f::repo_root().join("docs/vi/legal/trademark.md");
    let en_doc = v100f::repo_root().join("docs/en/legal/trademark.md");
    assert!(trademark_root.exists(), "missing TRADEMARK.md");
    assert!(vi_doc.exists(), "missing docs/vi/legal/trademark.md");
    assert!(en_doc.exists(), "missing docs/en/legal/trademark.md");

    let package_json = v100f::read_json(
        &v100f::repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp-ocl")
            .join("package.json"),
    );
    let publisher = package_json
        .get("publisher")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let expected_publisher = contract
        .get("official_vscode_publisher")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert_eq!(
        publisher, expected_publisher,
        "VSCode package publisher must match trademark policy"
    );

    let trademark = v100f::read_text(&trademark_root);
    assert!(
        trademark.contains("Official OCL Build")
            && trademark.contains("based on OCP-OCL")
            && trademark.contains("publisher: `ocp-ocl`"),
        "TRADEMARK.md must define official build, allow based-on wording, and pin official VSCode publisher"
    );

    let report = json!({
        "schema": "ocl.w100.legal.trademark_policy_report.v1",
        "status": "PASS",
        "official_vscode_publisher": expected_publisher,
        "files_verified": [
            "TRADEMARK.md",
            "docs/vi/legal/trademark.md",
            "docs/en/legal/trademark.md"
        ],
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/trademark_policy_report.json", &report);
}
