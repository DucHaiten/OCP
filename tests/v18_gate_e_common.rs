#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::init_project;
use serde_json::{json, Map as JsonMap, Value as JsonValue};

#[path = "v18_gate_a_common.rs"]
mod gate_a;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_packs_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocp")
        .join("w18")
        .join("packs")
}

pub fn ensure_run_manifest() {
    let _ = gate_a::ensure_run_manifest();
}

pub fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v18_gate_e_{tag}_{stamp}"))
}

pub fn run_ocp_cli(args: &[&str], envs: &BTreeMap<&str, &str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run ocp-cli")
}

pub fn assert_ok(output: &Output, step: &str) -> String {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn assert_fail(output: &Output, step: &str) -> String {
    assert!(
        !output.status.success(),
        "{step} unexpectedly succeeded:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    let rendered = serde_json::to_string_pretty(value).expect("render json");
    fs::write(path, rendered).unwrap_or_else(|_| panic!("write {}", path.display()));
}

pub fn read_json(path: &Path) -> JsonValue {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

pub fn first_ocppkg(root: &Path) -> PathBuf {
    let pkg_dir = root.join(".ocppkg");
    let entries = fs::read_dir(&pkg_dir).expect("read .ocppkg");
    for entry in entries {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("ocppkg") {
            return path;
        }
    }
    panic!("no .ocppkg artifact found in {}", pkg_dir.display());
}

pub fn init_connector_project(root: &Path, manifest: &str, main_source: &str) {
    init_project(root).expect("init project");
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src");
    fs::write(src_dir.join("main.ocp"), main_source).expect("write src/main.ocp");
}

pub fn merge_pack_shipproof_section(section: &str, value: JsonValue) {
    let out_path = w18_packs_dir().join("pack_shipproof_report.json");
    let mut report = if out_path.exists() {
        read_json(&out_path)
    } else {
        json!({
            "schema": "ocp.w18.packs.pack_shipproof_report.v1",
            "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
            "connector_set_ref": "contracts/packs/connector_set.v1.json",
            "sections": JsonValue::Object(JsonMap::new())
        })
    };
    let Some(obj) = report.as_object_mut() else {
        panic!("pack_shipproof_report must be object");
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

pub fn merge_connector_baseline_section(section: &str, value: JsonValue) {
    let out_path = w18_packs_dir().join("connector_baseline_report.json");
    let mut report = if out_path.exists() {
        read_json(&out_path)
    } else {
        json!({
            "schema": "ocp.w18.packs.connector_baseline_report.v1",
            "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
            "connector_set_ref": "contracts/packs/connector_set.v1.json",
            "sections": JsonValue::Object(JsonMap::new())
        })
    };
    let Some(obj) = report.as_object_mut() else {
        panic!("connector_baseline_report must be object");
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
