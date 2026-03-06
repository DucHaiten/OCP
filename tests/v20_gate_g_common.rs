#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ocl_sdk::{sign_contract_json_v20, verify_contract_json_signature_v20};
use serde_json::{json, Value as JsonValue};

#[path = "v20_gate_a_common.rs"]
mod v20a;

#[derive(Debug, Clone)]
pub struct ReleaseFixtureV20 {
    pub release_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_sig_path: PathBuf,
    pub artifact_hashes: BTreeMap<String, String>,
}

pub fn repo_root() -> PathBuf {
    v20a::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v20a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v20a::run_manifest_sha256()
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    v20a::write_report(rel_path, report)
}

pub fn read_json(path: &Path) -> JsonValue {
    v20a::read_json(path)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v20a::write_json_pretty(path, value)
}

pub fn sha256_hex_file(path: &Path) -> String {
    v20a::sha256_hex_file(path)
}

pub fn release_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w20")
        .join("release")
}

pub fn manifest_path() -> PathBuf {
    release_root().join("v1_rc_manifest.json")
}

pub fn manifest_sig_path() -> PathBuf {
    PathBuf::from(format!("{}.sig", manifest_path().to_string_lossy()))
}

pub fn repro_protocol() -> JsonValue {
    let path = repo_root()
        .join("contracts")
        .join("v20")
        .join("repro_protocol.v1.json");
    read_json(&path)
}

pub fn final_audit_scope() -> JsonValue {
    let path = repo_root()
        .join("contracts")
        .join("v20")
        .join("final_audit_scope.v1.json");
    read_json(&path)
}

pub fn release_required_artifacts_for_gate_g() -> Vec<&'static str> {
    vec![
        "target/ocl/w20/meta/run_manifest.json",
        "target/ocl/w20/contracts/contract_chain_report.json",
        "target/ocl/w20/contracts/required_contracts_v20_report.json",
        "target/ocl/w20/regression/history_replay_report.json",
        "target/ocl/w20/regression/cross_platform_signature_aggregate_report.json",
        "target/ocl/w20/hardcore/fuzz_report.json",
        "target/ocl/w20/hardcore/mutation_report.json",
        "target/ocl/w20/hardcore/chaos_report.json",
        "target/ocl/w20/user/golden_journeys_report.json",
        "target/ocl/w20/security/redteam_report.json",
        "target/ocl/w20/security/cve_gate_report.json",
        "target/ocl/w20/security/cve_snapshot_report.json",
    ]
}

pub fn manifest_payload_v20() -> (JsonValue, BTreeMap<String, String>) {
    let root = repo_root();
    let mut artifacts = Vec::<JsonValue>::new();
    let mut hashes = BTreeMap::<String, String>::new();
    for rel in release_required_artifacts_for_gate_g() {
        let full = root.join(rel);
        assert!(
            full.exists(),
            "missing release input artifact: {}",
            full.display()
        );
        let sha = sha256_hex_file(&full);
        hashes.insert(rel.to_string(), sha.clone());
        artifacts.push(json!({
            "path": rel,
            "sha256": sha
        }));
    }
    artifacts.sort_by(|lhs, rhs| {
        let l = lhs
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .as_bytes()
            .to_vec();
        let r = rhs
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .as_bytes()
            .to_vec();
        l.cmp(&r)
    });

    let payload = json!({
        "schema": "ocl.w20.release.v1_rc_manifest.v1",
        "contract_id": "v20.v1_rc_manifest",
        "version": "v1",
        "hasher_version": "sha256-v1",
        "target_release": "v1.0-rc",
        "toolchain_digest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "artifacts": artifacts
    });
    (payload, hashes)
}

pub fn ensure_release_fixture_v20() -> ReleaseFixtureV20 {
    ensure_run_manifest();
    let release_root = release_root();
    fs::create_dir_all(&release_root).expect("create w20 release dir");

    let (manifest, artifact_hashes) = manifest_payload_v20();
    let manifest_path = manifest_path();
    write_json_pretty(&manifest_path, &manifest);

    let signed = sign_contract_json_v20(&manifest_path, "w18-sot-root", 1)
        .expect("sign v1_rc_manifest.json");
    let verified = verify_contract_json_signature_v20(&repo_root(), &manifest_path)
        .expect("verify v1_rc_manifest signature");
    assert!(
        verified.signature_verified,
        "release manifest signature must verify"
    );

    ReleaseFixtureV20 {
        release_root,
        manifest_path,
        manifest_sig_path: signed,
        artifact_hashes,
    }
}
