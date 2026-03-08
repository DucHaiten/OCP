use serde_json::json;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_repro_build_evidence() {
    v20g::ensure_run_manifest();
    let protocol = v20g::repro_protocol();
    let clean_build_runs = protocol
        .get("clean_build_runs")
        .and_then(serde_json::Value::as_u64)
        .expect("repro_protocol.clean_build_runs");
    assert!(
        clean_build_runs >= 2,
        "repro protocol requires at least 2 clean runs"
    );

    let mut manifest_bytes = Vec::<Vec<u8>>::new();
    let mut artifact_maps = Vec::<std::collections::BTreeMap<String, String>>::new();

    for _ in 0..clean_build_runs {
        let fixture = v20g::ensure_release_fixture_v20();
        let bytes = std::fs::read(&fixture.manifest_path).expect("read v1_rc_manifest.json");
        manifest_bytes.push(bytes);
        artifact_maps.push(fixture.artifact_hashes);
    }

    let first_manifest = manifest_bytes
        .first()
        .expect("first manifest bytes")
        .clone();
    for (idx, current) in manifest_bytes.iter().enumerate().skip(1) {
        assert_eq!(
            current, &first_manifest,
            "manifest bytes mismatch at clean run index {idx}"
        );
    }

    let first_hashes = artifact_maps
        .first()
        .expect("first artifact hash map")
        .clone();
    for (idx, current) in artifact_maps.iter().enumerate().skip(1) {
        assert_eq!(
            current, &first_hashes,
            "artifact hash map mismatch at clean run index {idx}"
        );
    }

    let report = json!({
        "schema": "ocp.w20.release.reproducibility_report.v1",
        "status": "PASS",
        "clean_build_runs": clean_build_runs,
        "manifest_byte_equal": true,
        "artifact_sha256_equal": true,
        "manifest_sha256": v20g::sha256_hex_file(&v20g::manifest_path()),
        "artifact_hashes": first_hashes,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20g::run_manifest_sha256()
    });
    v20g::write_report("release/reproducibility_report.json", &report);
}
