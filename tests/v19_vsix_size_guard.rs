use std::fs;

use ocl_sdk::vsix_size_limit_for_channel_v19;
use serde_json::json;

#[path = "v19_gate_f_common.rs"]
mod f19;

#[test]
fn v19_vsix_size_guard_per_channel() {
    f19::ensure_run_manifest();
    let fixture = f19::ensure_release_fixture_v19();

    let limits_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_packaging_limits.v1.json");
    let channels_path = f19::repo_root()
        .join("contracts")
        .join("editor")
        .join("editor_publish_channels.v1.json");
    let limits = f19::read_json(&limits_path);
    let channels = f19::read_json(&channels_path);
    let channel_list = channels
        .get("channels")
        .and_then(serde_json::Value::as_array)
        .expect("channels");

    let size = fs::metadata(&fixture.vsix_path)
        .expect("vsix metadata")
        .len();
    let mut checks = Vec::<serde_json::Value>::new();
    for ch in channel_list {
        let channel = ch.as_str().expect("channel");
        let limit = vsix_size_limit_for_channel_v19(&limits, channel).expect("limit for channel");
        let pass = size <= limit;
        assert!(pass, "vsix size must be <= limit for {channel}");
        checks.push(json!({
            "channel": channel,
            "size_bytes": size,
            "limit_bytes": limit,
            "pass": pass
        }));
    }

    let report = json!({
        "schema": "ocl.w19.release.vsix_size_report.v1",
        "status": "PASS",
        "limits_path": limits_path.to_string_lossy().replace('\\', "/"),
        "checks": checks,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": f19::run_manifest_sha256()
    });
    f19::write_report("release/vsix_size_report.json", &report);
}
