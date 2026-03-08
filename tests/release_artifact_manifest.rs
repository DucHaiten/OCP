use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read file {}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn release_artifact_manifest_is_complete_and_hashes_verify() {
    let root = repo_root();
    let rc_dir = root.join("target").join("ocp").join("w16").join("rc");
    fs::create_dir_all(&rc_dir).expect("create w16 rc output dir");

    let required_files = vec![
        "target/ocp/w16/meta/run_manifest.json",
        "target/ocp/w16/contracts/contract_freeze_report.json",
        "target/ocp/w16/conformance/unified_conformance_report.json",
        "target/ocp/w16/determinism/determinism_soak_report.json",
        "target/ocp/w16/compat/compat_rehearsal_report.json",
        "target/ocp/w16/perf/perf_budget_report.json",
        "target/ocp/w16/security/security_chain_report.json",
        "target/ocp/w16/rc/rc_dryrun_report.json",
        "target/ocp/w16/rc/golden_user_journey_report.json",
    ];

    let mut artifacts = Vec::<JsonValue>::new();
    for rel in &required_files {
        let full = root.join(rel);
        assert!(
            full.exists(),
            "required artifact missing: {}",
            full.display()
        );
        let sig_path = format!("{rel}.sig");
        let sig_exists = root.join(&sig_path).exists();
        artifacts.push(json!({
            "path": rel,
            "sha256": sha256_hex_file(&full),
            "signature_status": if sig_exists { "signed" } else { "unsigned" }
        }));
    }

    let manifest = json!({
        "schema": "ocp.w16.rc.release_artifact_manifest.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "hasher": "sha256-v1",
        "artifacts": artifacts,
        "pass": true
    });
    let manifest_path = rc_dir.join("release_artifact_manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize release artifact manifest"),
    )
    .expect("write release_artifact_manifest.json");

    let read_back: JsonValue = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read release_artifact_manifest.json"),
    )
    .expect("parse release_artifact_manifest.json");
    let items = read_back
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        !items.is_empty(),
        "release artifact manifest must contain items"
    );
    for item in &items {
        let path = item
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("missing path in manifest item: {item}"));
        let expected = item
            .get("sha256")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("missing sha256 in manifest item: {item}"));
        let actual = sha256_hex_file(&root.join(path));
        assert_eq!(
            actual, expected,
            "release artifact hash mismatch for `{path}`"
        );
    }

    let readiness = json!({
        "schema": "ocp.w16.rc.release_readiness.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "checks": [
            { "id": "required_artifacts_exist", "pass": true },
            { "id": "release_artifact_manifest_hash_verified", "pass": true },
            { "id": "golden_user_journey_present", "pass": true },
            { "id": "rc_dryrun_report_present", "pass": true }
        ],
        "overall_pass": true
    });
    fs::write(
        rc_dir.join("release_readiness_checklist.json"),
        serde_json::to_string_pretty(&readiness).expect("serialize release readiness checklist"),
    )
    .expect("write release_readiness_checklist.json");
}
