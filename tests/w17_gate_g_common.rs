#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value as JsonValue};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn read_utf8(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("read utf8 failed: {}", path.display());
    })
}

pub fn write_rc_report(file_name: &str, payload: &JsonValue) {
    let out_dir = PathBuf::from("target").join("ocp").join("w17").join("rc");
    fs::create_dir_all(&out_dir).expect("create w17 rc output dir");
    fs::write(
        out_dir.join(file_name),
        serde_json::to_string_pretty(payload).expect("serialize rc report"),
    )
    .expect("write rc report");
}

pub fn gate_status_in_plan(plan_text: &str, gate: &str) -> Option<String> {
    for line in plan_text.lines() {
        if line.contains(&format!("Gate {gate}")) && line.contains(": `") {
            let start = line.find(": `")? + 3;
            let tail = &line[start..];
            let end = tail.find('`')?;
            return Some(tail[..end].to_string());
        }
    }
    None
}

pub fn required_artifact_presence(items: &[(&str, &str)]) -> Vec<JsonValue> {
    items
        .iter()
        .map(|(id, rel)| {
            let path = PathBuf::from(rel);
            json!({
                "id": id,
                "path": rel,
                "exists": path.exists()
            })
        })
        .collect()
}

pub fn all_present(rows: &[JsonValue]) -> bool {
    rows.iter().all(|row| {
        row.get("exists")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
    })
}
