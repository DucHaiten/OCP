use std::collections::BTreeSet;

use serde_json::json;

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_docs_repo_layout_policy() {
    v100d::ensure_run_manifest();

    let allowed_root_md = BTreeSet::from([
        "AGENTS.md".to_string(),
        "README.md".to_string(),
        "CONTRIBUTING.md".to_string(),
        "SECURITY.md".to_string(),
        "GOVERNANCE.md".to_string(),
        "OCP-OCL-MVP-PLAN-v1.0.md".to_string(),
        "TRADEMARK.md".to_string(),
    ]);

    let mut root_md = Vec::<String>::new();
    for entry in std::fs::read_dir(v100d::repo_root()).expect("read repo root") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| v.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        root_md.push(name);
    }
    root_md.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));

    let disallowed = root_md
        .iter()
        .filter(|name| !allowed_root_md.contains(*name))
        .cloned()
        .collect::<Vec<String>>();
    assert!(
        disallowed.is_empty(),
        "root markdown docs outside allowed policy: {}",
        disallowed.join(", ")
    );

    assert!(
        v100d::repo_root().join("docs").join("vi").exists(),
        "missing docs/vi"
    );
    assert!(
        v100d::repo_root().join("docs").join("en").exists(),
        "missing docs/en"
    );
    assert!(
        v100d::repo_root()
            .join("docs")
            .join("plans")
            .join("INDEX.md")
            .exists(),
        "missing docs/plans/INDEX.md"
    );

    let report = json!({
        "schema": "ocl.w100.docs.repo_layout_report.v1",
        "status": "PASS",
        "allowed_root_markdown": allowed_root_md,
        "root_markdown_detected": root_md,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_repo_layout_report.json", &report);
}
