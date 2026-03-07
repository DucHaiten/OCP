use serde_json::json;

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_checksums() {
    let fixture = v100b::ensure_release_fixture_v100();
    assert!(fixture.checksums_path.exists(), "missing SHA256SUMS");

    let raw = std::fs::read_to_string(&fixture.checksums_path).expect("read SHA256SUMS");
    let mut entries = Vec::<(String, String)>::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((sha, rel)) = trimmed.split_once("  ") else {
            panic!("invalid SHA256SUMS line format: `{trimmed}`");
        };
        entries.push((sha.to_string(), rel.to_string()));
    }
    assert!(!entries.is_empty(), "SHA256SUMS must have at least one entry");

    for (expected_sha, rel) in &entries {
        let path = v100b::repo_root().join(rel);
        assert!(path.exists(), "checksum entry points to missing file `{rel}`");
        let actual_sha = v100b::sha256_hex_file(&path);
        assert_eq!(
            actual_sha, *expected_sha,
            "checksum mismatch for `{rel}`"
        );
    }

    let report = json!({
        "schema": "ocl.w100.release.release_checksums_report.v1",
        "status": "PASS",
        "entry_count": entries.len(),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100b::run_manifest_sha256()
    });
    v100b::write_report("release/release_checksums_report.json", &report);
}
