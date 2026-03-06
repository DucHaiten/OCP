#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map as JsonMap, Value as JsonValue};

#[path = "v18_gate_a_common.rs"]
mod gate_a;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_cassette_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w18")
        .join("cassette")
}

pub fn w18_security_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w18")
        .join("security")
}

pub fn ensure_run_manifest() {
    let _ = gate_a::ensure_run_manifest();
}

pub fn temp_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v18_gate_d_{tag}_{stamp}"))
}

pub fn ensure_artifact_layout(root: &Path) -> PathBuf {
    let artifact = root.join(".ocl_artifacts").join("run_demo");
    fs::create_dir_all(artifact.join("cassette")).expect("create cassette dir");
    artifact
}

pub fn run_ocl_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocl-cli")
}

pub fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn assert_failed_with(output: &Output, needle: &str) {
    assert!(
        !output.status.success(),
        "command should fail:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "expected `{needle}` in stderr, got: {stderr}"
    );
}

pub fn read_json(path: &Path) -> JsonValue {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    let rendered = serde_json::to_string_pretty(value).expect("render json");
    fs::write(path, rendered).unwrap_or_else(|_| panic!("write {}", path.display()));
}

pub fn seed_v08_cassette_lines(artifact: &Path, lines: &[&str]) {
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette");
    let payload = format!("{}\n", lines.join("\n"));
    fs::write(cassette_dir.join("cassette.jsonl"), payload).expect("write cassette.jsonl");
    fs::write(
        cassette_dir.join("cassette_index.json"),
        "{\"call_id_to_entry_id\":{}}",
    )
    .expect("write cassette_index.json");
    fs::write(
        cassette_dir.join("cassette_meta.toml"),
        "mode = \"record\"\n",
    )
    .expect("write cassette_meta.toml");
}

pub fn merge_cassette_operability_section(section: &str, value: JsonValue) {
    let out_path = w18_cassette_dir().join("cassette_operability_report.json");
    let mut report = if out_path.exists() {
        read_json(&out_path)
    } else {
        json!({
            "schema": "ocl.w18.cassette.cassette_operability_report.v1",
            "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
            "sections": JsonValue::Object(JsonMap::new())
        })
    };

    let Some(obj) = report.as_object_mut() else {
        panic!("cassette_operability_report must be object");
    };
    let sections = obj
        .entry("sections")
        .or_insert_with(|| JsonValue::Object(JsonMap::new()));
    let Some(sections_obj) = sections.as_object_mut() else {
        panic!("sections must be object");
    };
    sections_obj.insert(section.to_string(), value);
    write_json_pretty(&out_path, &report);
}
