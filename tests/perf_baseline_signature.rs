use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct BaselineFileEntry {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct BaselineManifest {
    schema: String,
    hasher: String,
    baseline_id: String,
    files: Vec<BaselineFileEntry>,
}

#[derive(Debug, Deserialize)]
struct BaselineSignature {
    schema: String,
    hasher: String,
    signature: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read file {}", path.display()));
    sha256_hex_bytes(&bytes)
}

#[test]
fn v16_perf_baseline_bundle_signature_and_hash_verify() {
    let root = repo_root();
    let baseline_dir = root.join("baselines").join("v015");
    let manifest_path = baseline_dir.join("baseline_manifest.json");
    let signature_path = baseline_dir.join("baseline_manifest.sig");

    assert!(
        manifest_path.exists(),
        "missing baseline manifest SoT: {}",
        manifest_path.display()
    );
    assert!(
        signature_path.exists(),
        "missing baseline manifest signature: {}",
        signature_path.display()
    );

    let manifest_raw = fs::read_to_string(&manifest_path).expect("read baseline_manifest.json");
    let manifest: BaselineManifest =
        serde_json::from_str(&manifest_raw).expect("parse baseline_manifest.json");
    assert_eq!(manifest.schema, "ocl.w16.perf.baseline_manifest.v1");
    assert_eq!(manifest.hasher, "sha256-v1");
    assert_eq!(manifest.baseline_id, "v0.15-line");
    assert!(
        !manifest.files.is_empty(),
        "baseline manifest must contain file entries"
    );

    for entry in &manifest.files {
        assert!(
            entry.path.starts_with("baselines/v015/"),
            "baseline SoT must be pinned under baselines/v015, got `{}`",
            entry.path
        );
        let full = root.join(&entry.path);
        assert!(full.exists(), "baseline file missing: {}", full.display());
        let actual = sha256_hex_file(&full);
        assert_eq!(
            actual, entry.sha256,
            "baseline file hash mismatch for {}",
            entry.path
        );
    }

    let signature_raw = fs::read_to_string(&signature_path).expect("read baseline_manifest.sig");
    let signature: BaselineSignature =
        serde_json::from_str(&signature_raw).expect("parse baseline_manifest.sig");
    assert_eq!(signature.schema, "ocl.w16.perf.baseline_manifest.sig.v1");
    assert_eq!(signature.hasher, "sha256-v1");

    let computed_signature = sha256_hex_file(&manifest_path);
    assert_eq!(
        signature.signature, computed_signature,
        "baseline manifest signature must match pinned SoT bytes"
    );

    let out_dir = root.join("target").join("ocl").join("w16").join("perf");
    fs::create_dir_all(&out_dir).expect("create w16 perf output dir");
    let report = json!({
        "schema": "ocl.w16.perf.baseline_verify.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "baseline_id": manifest.baseline_id,
        "manifest_path": "baselines/v015/baseline_manifest.json",
        "signature_path": "baselines/v015/baseline_manifest.sig",
        "hashes_verified": true,
        "signature_verified": true
    });
    fs::write(
        out_dir.join("perf_baseline_verify_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize perf baseline verify report"),
    )
    .expect("write perf_baseline_verify_report.json");
}
