use ocp_sdk::version_handshake_allowed_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_lsp_version_handshake_follows_compat_matrix() {
    v19::ensure_run_manifest();

    let matrix_path = v19::contracts_root()
        .join("editor")
        .join("ocp_version_compat_matrix.v1.json");
    let matrix = v19::read_json(&matrix_path);

    let pass = version_handshake_allowed_v19("0.19.0-dev", "0.19.0-dev", "0.19.0-dev", &matrix);
    let fail = version_handshake_allowed_v19("0.20.0", "0.19.0-dev", "0.19.0-dev", &matrix);

    assert!(pass, "0.19.x versions must pass matrix handshake");
    assert!(!fail, "version outside matrix must fail handshake");

    let report = json!({
        "schema": "ocp.w19.lsp.version_handshake_report.v1",
        "status": "PASS",
        "matrix_path": matrix_path.to_string_lossy().replace('\\', "/"),
        "pass_case": pass,
        "fail_case": fail,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("lsp/version_handshake_report.json", &report);
}
