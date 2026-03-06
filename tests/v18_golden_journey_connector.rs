use std::collections::BTreeMap;
use std::fs;

use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_f_common.rs"]
mod common;

#[test]
fn v18_golden_journey_connector_runs_quarantine_replay_with_perm_and_budget_checks() {
    common::ensure_run_manifest();
    let root = common::temp_project_dir("connector");
    let root_s = root.to_string_lossy().to_string();
    let empty = BTreeMap::new();
    let mut quarantine_env = BTreeMap::new();
    quarantine_env.insert("OCL_QUARANTINE", "1");
    let mut steps = Vec::new();

    let init = common::run_ocl_cli(&["init", &root_s, "--template", "tool-http"], &empty);
    let _ = common::assert_ok(&init, "init tool-http");
    steps.push(json!({"name":"init_connector_template","ok":true}));

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
    let has_db = connectors
        .iter()
        .any(|item| item.get("id").and_then(JsonValue::as_str) == Some("std.db.sql"));
    assert!(
        has_http && has_db,
        "connector_set.v1 must keep std.http.client + std.db.sql"
    );
    steps.push(json!({"name":"connector_set_check","ok":true}));

    let perm_dir = root.join("perm_connector");
    fs::create_dir_all(&perm_dir).expect("create perm dir");
    let perm_out = common::run_ocl_cli(
        &[
            "perm",
            "snapshot",
            &root_s,
            "--out-dir",
            &perm_dir.to_string_lossy(),
        ],
        &empty,
    );
    let _ = common::assert_ok(&perm_out, "perm snapshot");
    steps.push(json!({"name":"perm_snapshot","ok":true}));

    let run_without = common::run_ocl_cli(&["run", &root_s], &empty);
    let _ = common::assert_fail(&run_without, "run without quarantine env");

    let run_with = common::run_ocl_cli(&["run", &root_s], &quarantine_env);
    let _ = common::assert_ok(&run_with, "run with quarantine env");
    let artifact = common::latest_artifact_dir(&root);

    let budget = common::run_ocl_cli(
        &["budget", "analyze", &artifact.to_string_lossy(), "--json"],
        &empty,
    );
    let budget_stdout = common::assert_ok(&budget, "budget analyze");
    let budget_json: JsonValue = serde_json::from_str(&budget_stdout).expect("parse budget json");
    let observe_events = budget_json
        .get("observe_events")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert!(
        observe_events >= 1,
        "connector journey must include observe events"
    );
    steps.push(json!({"name":"budget_analyze","ok":true}));

    let replay_without = common::run_ocl_cli(&["replay", &artifact.to_string_lossy()], &empty);
    let _ = common::assert_fail(&replay_without, "replay without quarantine env");
    let replay_with =
        common::run_ocl_cli(&["replay", &artifact.to_string_lossy()], &quarantine_env);
    let _ = common::assert_ok(&replay_with, "replay with quarantine env");
    steps.push(json!({"name":"run_replay_quarantine","ok":true}));

    let report = json!({
        "schema": "ocl.w18.rc.golden_journey_connector_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "project_root": root.to_string_lossy().replace('\\', "/"),
        "steps": steps,
        "artifacts": {
            "run_artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
            "perm_snapshot_dir": perm_dir.to_string_lossy().replace('\\', "/"),
            "observe_events": observe_events
        },
        "connector_set": {
            "has_http": has_http,
            "has_db": has_db
        },
        "status": "PASS"
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("golden_journey_connector_report.json"),
        &report,
    );
}
