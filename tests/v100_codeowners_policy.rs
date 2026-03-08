use serde_json::json;

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_codeowners_policy() {
    v100f::ensure_run_manifest();

    let codeowners_path = v100f::repo_root().join("CODEOWNERS");
    let lines = v100f::readme_lines_without_comments(&codeowners_path);
    assert!(
        !lines.is_empty(),
        "CODEOWNERS must contain at least one ownership rule"
    );

    let catch_all = lines
        .iter()
        .find(|line| line.starts_with("* "))
        .unwrap_or_else(|| panic!("CODEOWNERS must define a catch-all owner"));
    assert!(
        catch_all.contains("@DucHaiten"),
        "catch-all CODEOWNERS entry must point to project owner"
    );

    let required_paths = [
        "/README.md",
        "/LICENSE",
        "/NOTICE",
        "/AUTHORS",
        "/GOVERNANCE.md",
        "/TRADEMARK.md",
        "/docs/",
        "/contracts/",
        "/projects/",
        "/editor/",
        "/.github/",
    ];
    for path in required_paths {
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with(&format!("{path} "))),
            "CODEOWNERS missing explicit ownership rule for `{path}`"
        );
    }

    let report = json!({
        "schema": "ocp.w100.legal.codeowners_policy_report.v1",
        "status": "PASS",
        "catch_all_owner": catch_all,
        "explicit_paths_verified": required_paths,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/codeowners_policy_report.json", &report);
}
