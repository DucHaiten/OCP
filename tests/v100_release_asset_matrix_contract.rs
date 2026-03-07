use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_release_asset_matrix_contract() {
    v100::ensure_run_manifest();

    let matrix = v100::read_json(&v100::release_asset_matrix_path());
    assert_eq!(
        matrix
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.release_asset_matrix"
    );
    assert_eq!(
        matrix
            .get("distribution_model")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "bundle"
    );

    let bundled = matrix
        .get("bundled_binaries")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let bundled_names = bundled
        .iter()
        .filter_map(JsonValue::as_str)
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    for required in ["ocl", "ocl-lsp", "ocl-dap"] {
        assert!(
            bundled_names.iter().any(|item| item == required),
            "missing bundled binary `{required}`"
        );
    }

    let channels = matrix
        .get("release_channels")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!channels.is_empty(), "release_channels must not be empty");
    let mut channel_names = Vec::<String>::new();
    let mut has_installer = false;
    for row in &channels {
        let channel = row
            .get("channel")
            .and_then(JsonValue::as_str)
            .unwrap_or("")
            .to_string();
        assert!(!channel.is_empty(), "channel must be non-empty");
        channel_names.push(channel.clone());
        let assets = row
            .get("required_assets")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(
            !assets.is_empty(),
            "channel `{channel}` has empty required_assets"
        );
        for asset in assets.iter().filter_map(JsonValue::as_str) {
            if asset == "ocl-v1.0.0-setup-win-x64.exe" {
                has_installer = true;
            }
        }
    }
    assert!(
        channel_names.iter().any(|item| item == "github_release"),
        "missing github_release channel"
    );
    assert!(
        channel_names.iter().any(|item| item == "staging_rehearsal"),
        "missing staging_rehearsal channel"
    );
    assert!(has_installer, "matrix must include Windows installer asset");

    let installer_sbom = matrix
        .get("installer_sbom")
        .and_then(JsonValue::as_object)
        .expect("installer_sbom object");
    let status = installer_sbom
        .get("status")
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    assert!(
        status == "required" || status == "optional" || status == "unavailable",
        "installer_sbom.status must be required|optional|unavailable"
    );
    if status == "unavailable" {
        let reason = installer_sbom
            .get("unavailable_reason_code")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        assert!(
            !reason.trim().is_empty(),
            "unavailable installer SBOM must include reason code"
        );
    }

    let report = json!({
        "schema": "ocl.w100.release_asset_matrix_report.v1",
        "status": "PASS",
        "distribution_model": "bundle",
        "channel_count": channels.len(),
        "channels": channel_names,
        "installer_sbom_status": status,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/release_asset_matrix_report.json", &report);
}
