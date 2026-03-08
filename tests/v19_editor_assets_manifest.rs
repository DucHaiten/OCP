use std::fs;

use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_assets_manifest_hashes_match() {
    v19::ensure_run_manifest();

    let manifest_path = v19::contracts_root()
        .join("editor")
        .join("editor_assets_manifest.v1.json");
    let mut manifest = v19::read_json(&manifest_path);
    let assets = manifest
        .get_mut("assets")
        .and_then(serde_json::Value::as_array_mut)
        .expect("assets array");

    let update = std::env::var("OCP_UPDATE_V19_ASSETS_MANIFEST")
        .ok()
        .as_deref()
        == Some("1");
    let mut has_auto = false;
    let mut evidence = Vec::<serde_json::Value>::new();
    for asset in assets.iter_mut() {
        let rel = asset
            .get("path")
            .and_then(serde_json::Value::as_str)
            .expect("asset path")
            .to_string();
        let full = v19::repo_root().join(&rel);
        assert!(full.exists(), "missing asset {}", full.display());
        let bytes = fs::read(&full).unwrap_or_else(|_| panic!("read {}", full.display()));
        let hash = v19::sha256_hex_bytes(&bytes);
        let cur = asset
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .expect("asset sha256");
        if cur == "__AUTO__" {
            has_auto = true;
        }
        if update || cur == "__AUTO__" {
            asset["sha256"] = serde_json::Value::String(hash.clone());
        } else {
            assert_eq!(cur, hash, "asset hash mismatch for {rel}");
        }
        evidence.push(json!({
            "path": rel.replace('\\', "/"),
            "sha256": hash
        }));
    }

    if update || has_auto {
        v19::write_json_pretty(&manifest_path, &manifest);
        panic!(
            "updated {} with computed asset hashes. Re-run test",
            manifest_path.display()
        );
    }

    let report = json!({
        "schema": "ocp.w19.editor.assets_manifest_report.v1",
        "status": "PASS",
        "asset_count": evidence.len(),
        "assets": evidence,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/editor_assets_manifest_report.json", &report);
}
