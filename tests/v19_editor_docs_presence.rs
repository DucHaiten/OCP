use std::fs;

use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_docs_presence() {
    v19::ensure_run_manifest();

    let required = [
        "CONTRIBUTING.md",
        "SECURITY.md",
        "docs/editor/DEVELOPMENT.md",
        "docs/editor/ARCHITECTURE.md",
    ];
    let mut items = Vec::<serde_json::Value>::new();
    for rel in required {
        let full = v19::repo_root().join(rel);
        assert!(full.exists(), "missing required doc {}", full.display());
        let raw = fs::read_to_string(&full).unwrap_or_else(|_| panic!("read {}", full.display()));
        assert!(
            raw.trim().len() >= 40,
            "required doc {} is too short",
            full.display()
        );
        items.push(json!({
            "path": rel,
            "bytes": raw.len()
        }));
    }

    let report = json!({
        "schema": "ocl.w19.community.editor_docs_readiness_report.v1",
        "status": "PASS",
        "documents": items,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("community/editor_docs_readiness_report.json", &report);
}
