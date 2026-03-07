use serde_json::json;

#[path = "v100_gate_g_common.rs"]
mod v100g;

#[test]
fn v100_readme_licensing_consistency() {
    v100g::ensure_run_manifest();

    let readme = v100g::read_text(&v100g::repo_root().join("README.md"));
    assert!(
        readme.contains("## License"),
        "README must contain License section"
    );
    assert!(
        readme.contains("AGPL-3.0-only"),
        "README must state OSS license id explicitly"
    );
    assert!(
        readme.contains("docs/en/legal/licensing.md")
            && readme.contains("docs/vi/legal/licensing.md"),
        "README must point to legal licensing docs"
    );
    assert!(
        readme.contains("docs/en/legal/commercial.md")
            && readme.contains("docs/vi/legal/commercial.md"),
        "README must point commercial use to legal/commercial docs"
    );
    assert!(
        !readme.contains("docs/en/business/")
            && !readme.contains("docs/vi/business/")
            && !readme.contains("(GGPL)")
            && !readme.contains("(governed GPL)"),
        "README must not keep ambiguous business links or ambiguous pseudo-license labels"
    );

    let report = json!({
        "schema": "ocl.w100.business.readme_licensing_consistency_report.v1",
        "status": "PASS",
        "readme_verified": "README.md",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100g::run_manifest_sha256()
    });
    v100g::write_report("business/readme_licensing_consistency_report.json", &report);
}
