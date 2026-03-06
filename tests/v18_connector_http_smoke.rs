use serde_json::{json, Value as JsonValue};

use ocl_sdk::check_project_with_lock;

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_connector_http_smoke_matches_connector_set_and_permission_hooks() {
    common::ensure_run_manifest();
    let root = common::temp_project_dir("connector_http_smoke");
    common::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_http_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_net_http]\n",
            "enabled = true\n",
            "allow_hosts = [\"api.example.com\"]\n",
            "allow_methods = [\"GET\"]\n",
            "max_body_bytes = 4096\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.http.client.get\", \"tier2\", ctx(\"url=https://api.example.com/ping\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );
    check_project_with_lock(&root, false).expect("std.http.client.get should pass");

    let connector_set = common::read_json(
        &common::repo_root()
            .join("contracts")
            .join("packs")
            .join("connector_set.v1.json"),
    );
    let connectors = connector_set
        .get("connectors")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let has_http = connectors
        .iter()
        .any(|item| item.get("id").and_then(JsonValue::as_str) == Some("std.http.client"));
    assert!(has_http, "connector_set.v1 must include std.http.client");

    common::merge_connector_baseline_section(
        "connector_http_smoke",
        json!({
            "status": "PASS",
            "project_root": root.to_string_lossy().replace('\\', "/"),
            "connector_id": "std.http.client",
            "in_connector_set": has_http,
            "policy_section": "permissions.std_net_http"
        }),
    );
}
