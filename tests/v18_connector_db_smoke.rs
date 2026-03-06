use serde_json::{json, Value as JsonValue};

use ocl_sdk::check_project_with_lock;

#[path = "v18_gate_e_common.rs"]
mod common;

#[test]
fn v18_connector_db_smoke_matches_connector_set_and_permission_hooks() {
    common::ensure_run_manifest();
    let root = common::temp_project_dir("connector_db_smoke");
    common::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_db_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_db]\n",
            "enabled = true\n",
            "allow_dsn = [\"main\"]\n",
            "allow_modes = [\"read_query\"]\n",
            "max_rows = 100\n",
            "max_bytes = 2048\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.db.query_int\", \"tier2\", ctx(\"dsn=main;sql=select 1\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );
    check_project_with_lock(&root, false).expect("std.db.query_int should pass");

    let denied_root = common::temp_project_dir("connector_db_smoke_denied");
    common::init_connector_project(
        &denied_root,
        concat!(
            "[package]\n",
            "name = \"connector_db_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n"
        ),
        "observe(\"std.db.query_int\", \"tier2\", ctx(\"dsn=main;sql=select 1\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );
    let err = check_project_with_lock(&denied_root, false).expect_err("missing std_db must fail");
    let msg = err.to_string();
    assert!(msg.contains("RC-DB-DSN-DENIED"), "actual={msg}");

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
    let has_db = connectors
        .iter()
        .any(|item| item.get("id").and_then(JsonValue::as_str) == Some("std.db.sql"));
    assert!(has_db, "connector_set.v1 must include std.db.sql");

    common::merge_connector_baseline_section(
        "connector_db_smoke",
        json!({
            "status": "PASS",
            "project_root": root.to_string_lossy().replace('\\', "/"),
            "connector_id": "std.db.sql",
            "in_connector_set": has_db,
            "policy_section": "permissions.std_db",
            "negative_reason": "RC-DB-DSN-DENIED"
        }),
    );
}
