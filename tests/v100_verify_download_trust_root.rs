use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_verify_download_trust_root() {
    v100b::ensure_release_fixture_v100();
    let trust = v100b::signing_trust_root();
    let trust_path = "contracts/security/v1.0/signing_trust_root.v1.json";

    let keys = trust
        .get("trust_root_public_keys")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!keys.is_empty(), "trust_root_public_keys must not be empty");
    let key_ids = keys
        .iter()
        .map(|row| {
            row.get("key_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .to_string()
        })
        .collect::<Vec<String>>();
    assert!(
        key_ids.iter().all(|k| !k.is_empty()),
        "all trust root key_ids must be non-empty"
    );

    let vi_doc = std::fs::read_to_string(
        v100b::repo_root()
            .join("docs")
            .join("vi")
            .join("security")
            .join("verify-download.md"),
    )
    .expect("read docs/vi/security/verify-download.md");
    let en_doc = std::fs::read_to_string(
        v100b::repo_root()
            .join("docs")
            .join("en")
            .join("security")
            .join("verify-download.md"),
    )
    .expect("read docs/en/security/verify-download.md");
    assert!(
        vi_doc.contains(trust_path),
        "vi verify guide must reference trust root path"
    );
    assert!(
        en_doc.contains(trust_path),
        "en verify guide must reference trust root path"
    );
    for key in &key_ids {
        assert!(
            vi_doc.contains(key),
            "vi verify guide missing key_id `{key}`"
        );
        assert!(
            en_doc.contains(key),
            "en verify guide missing key_id `{key}`"
        );
    }

    let report = json!({
        "schema": "ocl.w100.release.verify_download_trust_root_report.v1",
        "status": "PASS",
        "trust_root_path": trust_path,
        "key_ids": key_ids,
        "docs": [
            "docs/vi/security/verify-download.md",
            "docs/en/security/verify-download.md"
        ],
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/verify_download_trust_root_report.json", &report);
}
