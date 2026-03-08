use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

fn resolve_repo_relative(path: &str) -> std::path::PathBuf {
    f19::repo_root().join(Path::new(path))
}

#[test]
fn v19_bundled_binary_hashes_match_manifest() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();
    let manifest = f19::read_json(&fixture.manifest_path);

    let binaries = manifest
        .get("binaries")
        .and_then(serde_json::Value::as_array)
        .expect("manifest binaries array");
    let mut compared = BTreeMap::<String, bool>::new();
    for item in binaries {
        let name = item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .expect("binary name");
        let rel = item
            .get("path")
            .and_then(serde_json::Value::as_str)
            .expect("binary path");
        let expected = item
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .expect("binary sha256");
        let bytes = fs::read(resolve_repo_relative(rel)).expect("read binary");
        let actual_hash = f19::sha256_hex_bytes(&bytes);
        compared.insert(name.to_string(), actual_hash == expected);
    }
    assert!(
        compared.values().all(|matched| *matched),
        "all bundled binary hashes must match manifest"
    );

    let report = json!({
        "schema": "ocp.w19.release.binary_bootstrap_report.v1",
        "status": "PASS",
        "phase": "bundled_binary_hashes",
        "compared": compared,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/binary_bootstrap_report.json", &report);
}
