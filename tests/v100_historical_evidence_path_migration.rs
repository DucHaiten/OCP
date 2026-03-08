#![allow(clippy::duplicate_mod)]

use serde_json::json;

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[path = "v16_gate_a_common.rs"]
mod v16;

#[test]
fn v100_historical_evidence_path_migration() {
    v100d::ensure_run_manifest();

    let index_path = v100d::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.json");
    let sig_path = v100d::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.sig");
    assert!(index_path.exists(), "missing {}", index_path.display());
    assert!(sig_path.exists(), "missing {}", sig_path.display());

    let index_raw = std::fs::read_to_string(&index_path).expect("read evidence index");
    let index: v16::HistoryEvidenceIndexV1 =
        serde_json::from_str(&index_raw).expect("parse evidence index");
    let sig_raw = std::fs::read_to_string(&sig_path).expect("read evidence sig");
    let sig: v16::HistoryEvidenceSignatureV1 =
        serde_json::from_str(&sig_raw).expect("parse evidence sig");

    let expected = v16::compute_history_signature(&index);
    assert_eq!(
        sig.signature, expected,
        "historical evidence signature mismatch after path migration"
    );

    let mut verified = Vec::new();
    for entry in &index.entries {
        for file in &entry.evidence_files {
            assert!(
                file.path.starts_with("docs/plans/history/"),
                "historical evidence path must be under docs/plans/history/: {}",
                file.path
            );
            let full = v100d::repo_root().join(&file.path);
            assert!(
                full.exists(),
                "missing migrated evidence file `{}`",
                file.path
            );
            let actual_hash = v16::sha256_hex_file(&full);
            assert_eq!(
                actual_hash, file.sha256,
                "hash mismatch for `{}`",
                file.path
            );
            verified.push(json!({
                "version_id": entry.version_id,
                "path": file.path,
                "sha256": file.sha256
            }));
        }
    }

    let report = json!({
        "schema": "ocp.w100.docs.historical_evidence_path_migration_report.v1",
        "status": "PASS",
        "entries_verified": verified.len(),
        "paths_verified": verified,
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report(
        "docs/historical_evidence_path_migration_report.json",
        &report,
    );
}
