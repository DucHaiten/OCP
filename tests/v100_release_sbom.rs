use serde_json::Value as JsonValue;

#[path = "v100_gate_b_common.rs"]
mod v100b;

#[test]
fn v100_release_sbom() {
    let fixture = v100b::ensure_release_fixture_v100();
    let rust_sbom = fixture.release_root.join("SBOM-rust.spdx.json");
    let vscode_sbom = fixture.release_root.join("SBOM-vscode.spdx.json");
    assert!(rust_sbom.exists(), "missing SBOM-rust.spdx.json");
    assert!(vscode_sbom.exists(), "missing SBOM-vscode.spdx.json");

    for path in [&rust_sbom, &vscode_sbom] {
        let value = v100b::read_json(path);
        let version = value
            .get("spdxVersion")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        assert!(
            version.starts_with("SPDX-"),
            "invalid SPDX version in {}",
            path.display()
        );
        let packages = value
            .get("packages")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(
            !packages.is_empty(),
            "SBOM packages must not be empty in {}",
            path.display()
        );
    }
}
