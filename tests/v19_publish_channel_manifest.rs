use ocp_sdk::publish_channels_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_publish_channel_manifest_contract() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let channels_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_publish_channels.v1.json");
    let channels_contract = f19::read_json(&channels_path);
    let (channels, rollback_required) =
        publish_channels_v19(&channels_contract).expect("publish channels contract");
    assert!(
        channels.iter().any(|ch| ch == "vscode_marketplace"),
        "vscode_marketplace channel required"
    );
    assert!(
        channels.iter().any(|ch| ch == "open_vsx"),
        "open_vsx channel required"
    );
    assert!(rollback_required, "rollback rehearsal must be required");

    let manifest = f19::read_json(&fixture.manifest_path);
    let channel = manifest
        .get("channel")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(
        channels.iter().any(|item| item == &channel),
        "release manifest channel must be in publish channel contract"
    );

    let report = json!({
        "schema": "ocp.w19.release.publish_channel_report.v1",
        "status": "PASS",
        "channels_contract_path": channels_path.to_string_lossy().replace('\\', "/"),
        "channels": channels,
        "manifest_channel": channel,
        "rollback_required": rollback_required,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/publish_channel_report.json", &report);
}
