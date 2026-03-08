#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::init_project;
use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_a_common.rs"]
mod gate_a;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_ops_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocp")
        .join("w18")
        .join("ops")
}

pub fn ensure_run_manifest() {
    let _ = gate_a::ensure_run_manifest();
}

pub fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v18_gate_c_{tag}_{stamp}"))
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

pub fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn assert_fail(output: &Output, step: &str) {
    assert!(
        !output.status.success(),
        "{step} unexpectedly succeeded:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn init_demo_project(root: &Path) {
    init_project(root).expect("init project");
}

pub fn write_manifest_with_fs_rule(root: &Path, lane: &str, read_rule: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"perm_v18_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"{}\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.log.info\", \"std.fs.read_text\"]\n",
            "deny = []\n\n",
            "[permissions.std_fs]\n",
            "read = [\"{}\"]\n"
        ),
        lane, read_rule
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");
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

pub fn write_budget_analyze_fixture(artifact: &Path) {
    fs::create_dir_all(artifact).expect("create artifact dir");
    let lines = [
        serde_json::to_string(&json!({
            "t":"ProgramStart","i":0,"tick":0,"seed":0,"call_id": JsonValue::Null,"span": JsonValue::Null,
            "data":{"trace_schema_version":2,"lane":"locked_v071"}
        }))
        .expect("program start"),
        serde_json::to_string(&json!({
            "t":"TraceEvent","i":1,"tick":1,"seed":42,"call_id":1,"span":JsonValue::Null,
            "data":{
                "seq":1,"run_id":"r1","event":"observe_end","key":"std.fs.read_text",
                "callsite_package_id":"pkg.core","kind":"insufficient","reason":"RC-BUDGET-EXCEEDED",
                "origin_id":1,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":3,
                "universe_id":"__legacy__","domain_id":"default","payload_hash":"h1"
            }
        }))
        .expect("event1"),
        serde_json::to_string(&json!({
            "t":"TraceEvent","i":2,"tick":2,"seed":42,"call_id":2,"span":JsonValue::Null,
            "data":{
                "seq":2,"run_id":"r1","event":"observe_end","key":"std.kv.get",
                "callsite_package_id":"pkg.core","kind":"ok","reason":JsonValue::Null,
                "origin_id":2,"allowed":JsonValue::Null,"value":JsonValue::Null,"steps":2,
                "universe_id":"__legacy__","domain_id":"default","payload_hash":"h2"
            }
        }))
        .expect("event2"),
    ];
    fs::write(artifact.join("audit.jsonl"), lines.join("\n")).expect("write audit.jsonl");
}
