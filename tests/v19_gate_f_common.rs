#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ocl_sdk::{sign_contract_json_v19, verify_contract_json_signature_v19};
use serde_json::Value as JsonValue;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[derive(Debug, Clone)]
pub struct ReleaseFixtureV19 {
    pub release_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_sig_path: PathBuf,
    pub vsix_path: PathBuf,
    pub vsix_sha256: String,
    pub binaries: BTreeMap<String, PathBuf>,
    pub binary_hashes: BTreeMap<String, String>,
}

pub fn repo_root() -> PathBuf {
    v19::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v19::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v19::run_manifest_sha256()
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    v19::write_report(rel_path, report);
}

pub fn read_json(path: &Path) -> JsonValue {
    v19::read_json(path)
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    v19::sha256_hex_bytes(bytes)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v19::write_json_pretty(path, value);
}

pub fn release_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w19")
        .join("release")
}

pub fn manifest_path() -> PathBuf {
    release_root().join("editor_release_manifest.json")
}

pub fn manifest_sig_path() -> PathBuf {
    PathBuf::from(format!("{}.sig", manifest_path().to_string_lossy()))
}

fn write_if_needed(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    fs::write(path, bytes).unwrap_or_else(|_| panic!("write {}", path.display()));
}

pub fn ensure_release_fixture_v19() -> ReleaseFixtureV19 {
    ensure_run_manifest();

    let release_root = release_root();
    let vsix_path = release_root.join("ocp-ocl.vsix");
    let bin_dir = release_root.join("bin");
    fs::create_dir_all(&bin_dir).expect("create release bin dir");

    write_if_needed(&vsix_path, b"OCP-OCL VSIX v19 fixture\n");

    let mut binaries = BTreeMap::<String, PathBuf>::new();
    let cli = bin_dir.join("ocl-cli");
    let lsp = bin_dir.join("ocl-lsp");
    let dap = bin_dir.join("ocl-dap");
    write_if_needed(&cli, b"ocl-cli fixture v19\n");
    write_if_needed(&lsp, b"ocl-lsp fixture v19\n");
    write_if_needed(&dap, b"ocl-dap fixture v19\n");
    binaries.insert("ocl-cli".to_string(), cli);
    binaries.insert("ocl-lsp".to_string(), lsp);
    binaries.insert("ocl-dap".to_string(), dap);

    let mut binary_hashes = BTreeMap::<String, String>::new();
    for (name, path) in &binaries {
        let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
        binary_hashes.insert(name.clone(), v19::sha256_hex_bytes(&bytes));
    }

    let vsix_bytes = fs::read(&vsix_path).expect("read vsix fixture");
    let vsix_sha256 = v19::sha256_hex_bytes(&vsix_bytes);

    let contract_path = repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_release_manifest.v1.json");
    let mut manifest = read_json(&contract_path);
    manifest["vsix_path"] = JsonValue::String("target/ocl/w19/release/ocp-ocl.vsix".to_string());
    manifest["vsix_sha256"] = JsonValue::String(vsix_sha256.clone());

    if let Some(items) = manifest
        .get_mut("binaries")
        .and_then(JsonValue::as_array_mut)
    {
        for item in items {
            let Some(name) = item.get("name").and_then(JsonValue::as_str) else {
                continue;
            };
            if let Some(hash) = binary_hashes.get(name) {
                item["sha256"] = JsonValue::String(hash.clone());
            }
        }
    }

    let manifest_path = manifest_path();
    v19::write_json_pretty(&manifest_path, &manifest);

    let manifest_sig_path = sign_contract_json_v19(&manifest_path, "w18-sot-root", 1)
        .expect("sign editor release manifest");
    let _ = verify_contract_json_signature_v19(&repo_root(), &manifest_path)
        .expect("verify editor release manifest signature");

    ReleaseFixtureV19 {
        release_root,
        manifest_path,
        manifest_sig_path,
        vsix_path,
        vsix_sha256,
        binaries,
        binary_hashes,
    }
}
