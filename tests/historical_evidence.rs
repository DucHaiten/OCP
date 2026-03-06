use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::json;

#[path = "v16_gate_a_common.rs"]
mod v16;

fn index_path() -> PathBuf {
    v16::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.json")
}

fn sig_path() -> PathBuf {
    v16::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.sig")
}

fn build_default_index_v1() -> v16::HistoryEvidenceIndexV1 {
    let versions = [
        ("v0.1", "OCP-OCL-MVP-PLAN-v0.1.md"),
        ("v0.2", "OCP-OCL-MVP-PLAN-v0.2.md"),
        ("v0.3", "OCP-OCL-MVP-PLAN-v0.3.md"),
        ("v0.4", "OCP-OCL-MVP-PLAN-v0.4.md"),
        ("v0.5", "OCP-OCL-MVP-PLAN-v0.5.md"),
        ("v0.6", "OCP-OCL-MVP-PLAN-v0.6.md"),
        ("v0.7", "OCP-OCL-MVP-PLAN-v0.7.1.md"),
        ("v0.8", "OCP-OCL-MVP-PLAN-v0.8.md"),
        ("v0.9", "OCP-OCL-MVP-PLAN-v0.9.md"),
        ("v0.10", "OCP-OCL-MVP-PLAN-v0.10.md"),
        ("v0.11", "OCP-OCL-MVP-PLAN-v0.11.md"),
        ("v0.12", "OCP-OCL-MVP-PLAN-v0.12.md"),
        ("v0.13", "OCP-OCL-MVP-PLAN-v0.13.md"),
        ("v0.14", "OCP-OCL-MVP-PLAN-v0.14.md"),
        ("v0.15", "OCP-OCL-MVP-PLAN-v0.15.md"),
        ("v0.20", "OCP-OCL-MVP-PLAN-v0.20.md"),
    ];
    let mut entries = Vec::new();
    for (version_id, file) in versions {
        let full = v16::repo_root().join(file);
        let hash = v16::sha256_hex_file(&full);
        entries.push(v16::HistoryEvidenceEntryV1 {
            version_id: version_id.to_string(),
            commit_or_tag: format!("OCL-{version_id}"),
            schema_version: "v1".to_string(),
            evidence_files: vec![v16::EvidenceFileV1 {
                path: file.to_string(),
                sha256: hash,
            }],
        });
    }
    v16::HistoryEvidenceIndexV1 {
        schema: "ocl.history.evidence.index.v1".to_string(),
        hasher: "sha256-v1".to_string(),
        entries,
    }
}

#[test]
fn historical_evidence_chain_v01_to_v015_is_complete_and_valid() {
    if !index_path().exists() || !sig_path().exists() {
        if let Some(parent) = index_path().parent() {
            fs::create_dir_all(parent).expect("create history contracts dir");
        }
        let index = build_default_index_v1();
        let signature = v16::compute_history_signature(&index);
        let sig = v16::HistoryEvidenceSignatureV1 {
            schema: "ocl.history.evidence.sig.v1".to_string(),
            hasher: "sha256-v1".to_string(),
            signature,
        };
        let index_value = serde_json::to_value(&index).expect("index value");
        let sig_value = serde_json::to_value(&sig).expect("sig value");
        v16::write_json_pretty(&index_path(), &index_value);
        v16::write_json_pretty(&sig_path(), &sig_value);
        panic!(
            "initialized missing historical evidence SoT ({}, {}). Re-run test",
            index_path().display(),
            sig_path().display()
        );
    }

    let index_raw = fs::read_to_string(index_path()).expect("read evidence index");
    let index: v16::HistoryEvidenceIndexV1 =
        serde_json::from_str(&index_raw).expect("parse evidence index");
    assert_eq!(index.schema, "ocl.history.evidence.index.v1");
    assert_eq!(index.hasher, "sha256-v1");

    let sig_raw = fs::read_to_string(sig_path()).expect("read evidence signature");
    let sig: v16::HistoryEvidenceSignatureV1 =
        serde_json::from_str(&sig_raw).expect("parse evidence signature");
    assert_eq!(sig.schema, "ocl.history.evidence.sig.v1");
    assert_eq!(sig.hasher, "sha256-v1");

    let computed_signature = v16::compute_history_signature(&index);
    assert_eq!(
        sig.signature, computed_signature,
        "historical evidence signature mismatch"
    );

    let expected_versions = (1..=15)
        .map(|v| format!("v0.{v}"))
        .collect::<BTreeSet<String>>();
    let actual_versions = index
        .entries
        .iter()
        .map(|entry| entry.version_id.clone())
        .collect::<BTreeSet<String>>();
    assert!(
        expected_versions.is_subset(&actual_versions),
        "historical evidence must include at least full v0.1..v0.15 range"
    );
    assert!(
        actual_versions.contains("v0.20"),
        "historical evidence must include v0.20 signoff continuity entry"
    );

    let mut verified_entries = Vec::new();
    for entry in &index.entries {
        assert!(
            !entry.commit_or_tag.trim().is_empty(),
            "commit_or_tag must be non-empty for {}",
            entry.version_id
        );
        assert!(
            !entry.evidence_files.is_empty(),
            "evidence_files must be non-empty for {}",
            entry.version_id
        );
        for file in &entry.evidence_files {
            let file_path = v16::repo_root().join(&file.path);
            assert!(
                file_path.exists(),
                "missing evidence file `{}` for {}",
                file.path,
                entry.version_id
            );
            let actual_hash = v16::sha256_hex_file(&file_path);
            assert_eq!(
                actual_hash, file.sha256,
                "evidence hash mismatch for `{}` ({})",
                file.path, entry.version_id
            );
            verified_entries.push(json!({
                "version_id": entry.version_id,
                "path": file.path,
                "sha256": file.sha256,
                "verification": "PASS",
            }));
        }
    }

    let report = json!({
        "schema": "ocl.history.evidence.report.v1",
        "status": "PASS",
        "entries_verified": verified_entries.len(),
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "entries": verified_entries,
    });
    let out_path = v16::w16_target_root()
        .join("history")
        .join("historical_evidence_report.json");
    v16::write_json_pretty(&out_path, &report);
}
