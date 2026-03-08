use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "v16_gate_a_common.rs"]
mod v16;

fn index_path() -> std::path::PathBuf {
    v16::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.json")
}

fn sig_path() -> std::path::PathBuf {
    v16::repo_root()
        .join("contracts")
        .join("history")
        .join("evidence_index.v1.sig")
}

fn build_default_index_v1() -> v16::HistoryEvidenceIndexV1 {
    let versions = [
        ("v0.1", "docs/plans/history/OCP-MVP-PLAN-v0.1.md"),
        ("v0.2", "docs/plans/history/OCP-MVP-PLAN-v0.2.md"),
        ("v0.3", "docs/plans/history/OCP-MVP-PLAN-v0.3.md"),
        ("v0.4", "docs/plans/history/OCP-MVP-PLAN-v0.4.md"),
        ("v0.5", "docs/plans/history/OCP-MVP-PLAN-v0.5.md"),
        ("v0.6", "docs/plans/history/OCP-MVP-PLAN-v0.6.md"),
        ("v0.7", "docs/plans/history/OCP-MVP-PLAN-v0.7.1.md"),
        ("v0.8", "docs/plans/history/OCP-MVP-PLAN-v0.8.md"),
        ("v0.9", "docs/plans/history/OCP-MVP-PLAN-v0.9.md"),
        ("v0.10", "docs/plans/history/OCP-MVP-PLAN-v0.10.md"),
        ("v0.11", "docs/plans/history/OCP-MVP-PLAN-v0.11.md"),
        ("v0.12", "docs/plans/history/OCP-MVP-PLAN-v0.12.md"),
        ("v0.13", "docs/plans/history/OCP-MVP-PLAN-v0.13.md"),
        ("v0.14", "docs/plans/history/OCP-MVP-PLAN-v0.14.md"),
        ("v0.15", "docs/plans/history/OCP-MVP-PLAN-v0.15.md"),
        ("v0.20", "docs/plans/history/OCP-MVP-PLAN-v0.20.md"),
    ];
    let mut entries = Vec::new();
    for (version_id, file) in versions {
        let full = v16::repo_root().join(file);
        let hash = v16::sha256_hex_file(&full);
        entries.push(v16::HistoryEvidenceEntryV1 {
            version_id: version_id.to_string(),
            commit_or_tag: format!("OCP-{version_id}"),
            schema_version: "v1".to_string(),
            evidence_files: vec![v16::EvidenceFileV1 {
                path: file.to_string(),
                sha256: hash,
            }],
        });
    }
    v16::HistoryEvidenceIndexV1 {
        schema: "ocp.history.evidence.index.v1".to_string(),
        hasher: "sha256-v1".to_string(),
        entries,
    }
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ocp_v16_history_sig_{tag}_{stamp}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn historical_evidence_signature_roundtrip_and_tamper_detection() {
    if !index_path().exists() || !sig_path().exists() {
        if let Some(parent) = index_path().parent() {
            fs::create_dir_all(parent).expect("create history contracts dir");
        }
        let index = build_default_index_v1();
        let sig = v16::HistoryEvidenceSignatureV1 {
            schema: "ocp.history.evidence.sig.v1".to_string(),
            hasher: "sha256-v1".to_string(),
            signature: v16::compute_history_signature(&index),
        };
        fs::write(
            index_path(),
            serde_json::to_string_pretty(&index).expect("render index"),
        )
        .expect("write index");
        fs::write(
            sig_path(),
            serde_json::to_string_pretty(&sig).expect("render sig"),
        )
        .expect("write sig");
        panic!(
            "initialized missing historical evidence SoT ({}, {}). Re-run test",
            index_path().display(),
            sig_path().display()
        );
    }

    let index_raw = fs::read_to_string(index_path()).expect("read evidence index");
    let index: v16::HistoryEvidenceIndexV1 =
        serde_json::from_str(&index_raw).expect("parse evidence index");
    let sig_raw = fs::read_to_string(sig_path()).expect("read evidence signature");
    let sig: v16::HistoryEvidenceSignatureV1 =
        serde_json::from_str(&sig_raw).expect("parse evidence signature");

    let computed = v16::compute_history_signature(&index);
    assert_eq!(
        sig.signature, computed,
        "history signature must match canonical index"
    );

    let temp = temp_dir("tamper");
    let mut tampered = index.clone();
    tampered.entries[0].version_id = "v0.1-tampered".to_string();
    let tampered_raw = serde_json::to_string_pretty(&tampered).expect("render tampered");
    fs::write(temp.join("evidence_index.v1.json"), tampered_raw).expect("write tampered");

    let tampered_sig = v16::compute_history_signature(&tampered);
    assert_ne!(
        sig.signature, tampered_sig,
        "tampered historical index must not verify with original signature"
    );
}
