#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::init_project;
use serde_json::json;

pub fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_v17_connector_{tag}_{stamp}"))
}

pub fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write text");
}

pub fn init_connector_project(root: &Path, manifest: &str, main_source: &str) {
    init_project(root).expect("init project");
    write_text(&root.join("Ocl.toml"), manifest);
    write_text(&root.join("src").join("main.ocl"), main_source);
}

pub fn write_gate_f_report() {
    let report = json!({
        "schema": "ocl.w17.connector_baseline_report.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "connectors": [
            {
                "connector_id": "std.db.sql",
                "keys": ["std.db.query_int", "std.db.exec"],
                "policy_section": "permissions.std_db"
            },
            {
                "connector_id": "std.http.client",
                "keys": ["std.http.client.get", "std.http.client.post"],
                "policy_section": "permissions.std_net_http"
            },
            {
                "connector_id": "std.queue.bus",
                "keys": ["std.queue.bus.publish", "std.queue.bus.consume"],
                "policy_section": "permissions.std_queue"
            }
        ]
    });

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w17")
        .join("connectors");
    fs::create_dir_all(&out_dir).expect("create connectors output dir");
    fs::write(
        out_dir.join("connector_baseline_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize connector report"),
    )
    .expect("write connector_baseline_report.json");
}
