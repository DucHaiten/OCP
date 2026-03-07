#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_a_common.rs"]
mod v100a;

#[derive(Debug, Clone)]
pub struct ReleaseFixtureV100 {
    pub release_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_sig_path: PathBuf,
    pub checksums_path: PathBuf,
    pub checksums_sig_path: PathBuf,
    pub artifact_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FileSignatureV100 {
    pub schema: String,
    pub algorithm: String,
    pub signature_schema_version: String,
    pub file_path: String,
    pub file_hash_sha256: String,
    pub pubkey_id: String,
    pub trust_epoch: u64,
    pub signature: String,
}

pub fn repo_root() -> PathBuf {
    v100a::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v100a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v100a::run_manifest_sha256()
}

pub fn read_json(path: &Path) -> JsonValue {
    v100a::read_json(path)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v100a::write_json_pretty(path, value)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("release/") {
        panic!("Gate 1.0-B reports must live under release/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}

pub fn sha256_hex_file(path: &Path) -> String {
    v100a::sha256_hex_file(path)
}

pub fn release_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w100")
        .join("release")
}

pub fn manifest_path() -> PathBuf {
    release_root().join("release_artifact_manifest.json")
}

pub fn checksums_path() -> PathBuf {
    release_root().join("SHA256SUMS")
}

pub fn sig_path_for(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sig", path.to_string_lossy()))
}

pub fn read_manifest() -> JsonValue {
    read_json(&manifest_path())
}

pub fn release_asset_matrix() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("release_asset_matrix.v1.json"),
    )
}

pub fn release_publish_scope() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("release_publish_scope.v1.json"),
    )
}

pub fn release_dependency_policy() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("release_dependency_policy.v1.json"),
    )
}

pub fn signing_trust_root() -> JsonValue {
    read_json(&v100a::signing_trust_root_path())
}

pub fn packaging_toolchain_contract() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("editor")
            .join("editor_packaging_toolchain.v1.json"),
    )
}

pub fn required_assets_for_channel(channel: &str, matrix: &JsonValue) -> Vec<String> {
    let rows = matrix
        .get("release_channels")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    for row in rows {
        let row_channel = row
            .get("channel")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        if row_channel != channel {
            continue;
        }
        let mut out = row
            .get("required_assets")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| item.as_str().map(|v| v.to_string()))
            .collect::<Vec<String>>();
        out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
        out.dedup();
        return out;
    }
    Vec::new()
}

fn release_rel(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .expect("release file under repo root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn trust_key_epoch() -> (String, u64, u64, u64) {
    let trust = signing_trust_root();
    let key = trust
        .get("trust_root_public_keys")
        .and_then(JsonValue::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("key_id"))
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string();
    let epoch = trust
        .get("trust_epoch_policy")
        .and_then(JsonValue::as_object)
        .cloned()
        .unwrap_or_default();
    let min = epoch.get("min").and_then(JsonValue::as_u64).unwrap_or(0);
    let max = epoch.get("max").and_then(JsonValue::as_u64).unwrap_or(0);
    let current = epoch
        .get("current")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    (key, min, max, current)
}

fn file_signature_payload(
    file_path: &str,
    hash: &str,
    pubkey_id: &str,
    trust_epoch: u64,
) -> String {
    format!(
        "ocl-v100-file-sign-v1|file_path={}|file_hash_sha256={}|pubkey_id={}|trust_epoch={}",
        file_path, hash, pubkey_id, trust_epoch
    )
}

fn sign_file(path: &Path, pubkey_id: &str, trust_epoch: u64) -> PathBuf {
    let file_path = release_rel(path);
    let hash = sha256_hex_file(path);
    let signature = v100a::sha256_hex_bytes(
        file_signature_payload(&file_path, &hash, pubkey_id, trust_epoch).as_bytes(),
    );
    let sig = json!({
        "schema": "ocl.release.file.sig.v1",
        "algorithm": "sha256-v1",
        "signature_schema_version": "1",
        "file_path": file_path,
        "file_hash_sha256": hash,
        "pubkey_id": pubkey_id,
        "trust_epoch": trust_epoch,
        "signature": signature
    });
    let sig_path = sig_path_for(path);
    write_json_pretty(&sig_path, &sig);
    sig_path
}

pub fn parse_file_signature(path: &Path) -> FileSignatureV100 {
    let value = read_json(path);
    FileSignatureV100 {
        schema: value
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        algorithm: value
            .get("algorithm")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        signature_schema_version: value
            .get("signature_schema_version")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        file_path: value
            .get("file_path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        file_hash_sha256: value
            .get("file_hash_sha256")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        pubkey_id: value
            .get("pubkey_id")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
        trust_epoch: value
            .get("trust_epoch")
            .and_then(JsonValue::as_u64)
            .unwrap_or_default(),
        signature: value
            .get("signature")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

pub fn verify_file_signature(path: &Path, sig_path: &Path) -> (bool, String) {
    let sig = parse_file_signature(sig_path);
    if sig.schema != "ocl.release.file.sig.v1" {
        return (false, "schema".to_string());
    }
    if sig.algorithm != "sha256-v1" {
        return (false, "algorithm".to_string());
    }
    if sig.signature_schema_version != "1" {
        return (false, "signature_schema_version".to_string());
    }
    let file_rel = release_rel(path);
    if sig.file_path != file_rel {
        return (false, "file_path".to_string());
    }
    let hash = sha256_hex_file(path);
    if sig.file_hash_sha256 != hash {
        return (false, "file_hash_sha256".to_string());
    }
    let (key, min_epoch, max_epoch, _) = trust_key_epoch();
    if sig.pubkey_id != key {
        return (false, "pubkey_id".to_string());
    }
    if sig.trust_epoch < min_epoch || sig.trust_epoch > max_epoch {
        return (false, "trust_epoch".to_string());
    }
    let expected = v100a::sha256_hex_bytes(
        file_signature_payload(&sig.file_path, &hash, &sig.pubkey_id, sig.trust_epoch).as_bytes(),
    );
    if sig.signature != expected {
        return (false, "signature".to_string());
    }
    (true, "ok".to_string())
}

fn assert_magic_prefix(path: &Path, expected: &[u8], label: &str) {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    assert!(
        bytes.len() >= expected.len(),
        "{label} asset too small: {}",
        path.display()
    );
    assert_eq!(
        &bytes[..expected.len()],
        expected,
        "{label} asset has invalid magic bytes: {}",
        path.display()
    );
}

fn assert_release_asset_shape(path: &Path, asset: &str) {
    if asset.ends_with(".zip") || asset.ends_with(".vsix") {
        assert_magic_prefix(path, b"PK", "zip/vsix");
        return;
    }
    if asset.ends_with(".tar.gz") {
        assert_magic_prefix(path, &[0x1f, 0x8b], "tar.gz");
        return;
    }
    if asset.ends_with(".exe") {
        assert_magic_prefix(path, b"MZ", "exe");
    }
}

pub fn verify_download_doc_paths() -> (PathBuf, PathBuf) {
    let vi_path = repo_root()
        .join("docs")
        .join("vi")
        .join("security")
        .join("verify-download.md");
    let en_path = repo_root()
        .join("docs")
        .join("en")
        .join("security")
        .join("verify-download.md");
    assert!(vi_path.exists(), "missing {}", vi_path.display());
    assert!(en_path.exists(), "missing {}", en_path.display());
    (vi_path, en_path)
}

pub fn ensure_release_fixture_v100() -> ReleaseFixtureV100 {
    ensure_run_manifest();
    verify_download_doc_paths();

    let release_root = release_root();
    fs::create_dir_all(&release_root).expect("create w100 release dir");
    let matrix = release_asset_matrix();
    let required_assets = required_assets_for_channel("github_release", &matrix);
    assert!(
        !required_assets.is_empty(),
        "release asset matrix must provide github_release required assets"
    );

    let manifest_path = manifest_path();
    let checksums_path = checksums_path();
    let mut artifact_hashes = BTreeMap::<String, String>::new();
    for asset in &required_assets {
        if asset == "release_artifact_manifest.json"
            || asset == "release_artifact_manifest.sig"
            || asset == "SHA256SUMS"
            || asset == "SHA256SUMS.sig"
        {
            continue;
        }
        let path = release_root.join(asset);
        assert!(path.exists(), "missing release asset `{}`", path.display());
        assert_release_asset_shape(&path, asset);
        let rel = release_rel(&path);
        artifact_hashes.insert(rel, sha256_hex_file(&path));
    }

    let mut checksum_lines = artifact_hashes
        .iter()
        .map(|(path, sha)| format!("{sha}  {path}"))
        .collect::<Vec<String>>();
    checksum_lines.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    fs::write(&checksums_path, format!("{}\n", checksum_lines.join("\n")))
        .expect("write SHA256SUMS");
    artifact_hashes.insert(
        release_rel(&checksums_path),
        sha256_hex_file(&checksums_path),
    );

    let (key_id, _, _, trust_epoch) = trust_key_epoch();
    let checksums_sig = sign_file(&checksums_path, &key_id, trust_epoch);
    artifact_hashes.insert(release_rel(&checksums_sig), sha256_hex_file(&checksums_sig));

    let mut artifacts = artifact_hashes
        .iter()
        .map(|(path, sha)| {
            json!({
                "path": path,
                "sha256": sha,
                "signature_status": if path.ends_with(".sig") { "signed" } else { "unsigned_or_external" }
            })
        })
        .collect::<Vec<JsonValue>>();
    artifacts.sort_by(|lhs, rhs| {
        lhs.get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .as_bytes()
            .cmp(
                rhs.get("path")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .as_bytes(),
            )
    });

    let manifest = json!({
        "schema": "ocl.release.artifact_manifest.v1",
        "contract_id": "v1.release_artifact_manifest",
        "version": "v1",
        "release_version": "v1.0.0",
        "release_channel": "github_release",
        "created_at_utc": "2026-03-07T00:00:00Z",
        "toolchain_digest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "artifacts": artifacts
    });
    write_json_pretty(&manifest_path, &manifest);
    let manifest_sig = sign_file(&manifest_path, &key_id, trust_epoch);

    let summary = json!({
        "schema": "ocl.w100.release.release_asset_matrix_report.v1",
        "status": "PASS",
        "channel": "github_release",
        "required_assets": required_assets,
        "generated_assets": artifact_hashes.keys().cloned().collect::<Vec<String>>(),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("release/release_asset_matrix_report.json", &summary);

    ReleaseFixtureV100 {
        release_root,
        manifest_path,
        manifest_sig_path: manifest_sig,
        checksums_path,
        checksums_sig_path: checksums_sig,
        artifact_hashes,
    }
}
