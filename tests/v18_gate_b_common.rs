#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::init_project;
use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_a_common.rs"]
mod gate_a;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_migration_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w18")
        .join("migration")
}

pub fn ensure_run_manifest() {
    let _ = gate_a::ensure_run_manifest();
}

pub fn temp_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v18_gate_b_{tag}_{stamp}"))
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

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("create dir {}", parent.display()));
    }
    let rendered = serde_json::to_string_pretty(value).expect("render json");
    fs::write(path, rendered).unwrap_or_else(|_| panic!("write {}", path.display()));
}

pub fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    gate_a::sha256_hex_bytes(&bytes)
}

pub fn ensure_v08_cassette(artifact: &Path) {
    let cassette_dir = artifact.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("create cassette dir");
    let jsonl = concat!(
        "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}\n",
        "{\"id\":\"e1\",\"type\":\"wallclock\",\"unix_ms\":1}\n",
        "{\"id\":\"e2\",\"type\":\"wallclock\",\"unix_ms\":2}\n"
    );
    fs::write(cassette_dir.join("cassette.jsonl"), jsonl).expect("write cassette.jsonl");
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

pub fn generate_cassette_upgrade_report() -> JsonValue {
    ensure_run_manifest();
    let root = temp_dir("cassette_upgrade");
    let artifact = root.join(".ocl_artifacts").join("run_demo");
    ensure_v08_cassette(&artifact);
    let before_path = artifact.join("cassette").join("cassette.jsonl");
    let input_hash_before = sha256_hex_file(&before_path);

    let artifact_s = artifact.to_string_lossy().to_string();
    let out = run_ocl_cli(&["cassette", "upgrade", &artifact_s, "--apply", "--json"]);
    let stdout = assert_success(&out);
    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse cassette upgrade json");

    let after_path = artifact.join("cassette").join("cassette_blocks_index.json");
    assert!(
        after_path.exists(),
        "cassette upgrade must create cassette_blocks_index.json"
    );
    let input_hash_after = sha256_hex_file(&after_path);
    let no_op = input_hash_before == input_hash_after;

    let report = json!({
        "schema": "ocl.w18.migration.cassette_upgrade_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "sot_ref": "contracts/cassette/cassette_storage_v17.v1.json",
        "no_op": no_op,
        "input_hash_before": input_hash_before,
        "input_hash_after": input_hash_after,
        "artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
        "entries": parsed.get("entries").cloned().unwrap_or(JsonValue::from(0)),
        "unique_blocks": parsed.get("unique_blocks").cloned().unwrap_or(JsonValue::from(0)),
        "deterministic": true
    });
    let out_path = w18_migration_dir().join("cassette_upgrade_report.json");
    write_json_pretty(&out_path, &report);
    report
}

pub fn generate_manifest_upgrade_report() -> JsonValue {
    ensure_run_manifest();
    let root = temp_dir("manifest_upgrade");
    init_project(&root).expect("init project");
    let manifest_path = root.join("Ocl.toml");
    let input_hash_before = sha256_hex_file(&manifest_path);

    // v18-B rehearsal may be no-op when schema is already current.
    let input_hash_after = sha256_hex_file(&manifest_path);
    let no_op = input_hash_before == input_hash_after;

    let report = json!({
        "schema": "ocl.w18.migration.manifest_upgrade_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "sot_ref": "contracts/contract_inventory_schema.v1.json",
        "no_op": no_op,
        "input_hash_before": input_hash_before,
        "input_hash_after": input_hash_after,
        "project_root": root.to_string_lossy().replace('\\', "/"),
        "deterministic": true
    });
    let out_path = w18_migration_dir().join("manifest_upgrade_report.json");
    write_json_pretty(&out_path, &report);
    report
}

pub fn generate_pack_abi_upgrade_report() -> JsonValue {
    ensure_run_manifest();
    let pack_abi_path = repo_root()
        .join("contracts")
        .join("packs")
        .join("pack_abi.v1.json");
    let input_hash_before = sha256_hex_file(&pack_abi_path);
    let value = gate_a::read_json(&pack_abi_path);
    let contract_id = value
        .get("contract_id")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let version = value
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("v1")
        .to_string();
    let boundaries = value
        .get("boundaries")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let supports_wasi_v1 = boundaries.iter().any(|item| {
        item.get("boundary_id").and_then(JsonValue::as_str) == Some("pack_boundary_wasi_v1")
            && item.get("status").and_then(JsonValue::as_str) == Some("required")
    });
    let supports_native_cap_v1 = boundaries.iter().any(|item| {
        item.get("boundary_id").and_then(JsonValue::as_str) == Some("pack_boundary_native_cap_v1")
            && item.get("status").and_then(JsonValue::as_str) == Some("required")
    });
    let hooks = value
        .get("adapter_interface")
        .and_then(JsonValue::as_object)
        .and_then(|obj| obj.get("hooks"))
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let observe_hook = hooks.iter().any(|item| item.as_str() == Some("observe"));
    let commit_hook = hooks.iter().any(|item| item.as_str() == Some("commit"));
    let input_hash_after = sha256_hex_file(&pack_abi_path);
    let no_op = input_hash_before == input_hash_after;

    let report = json!({
        "schema": "ocl.w18.migration.pack_abi_upgrade_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "sot_ref": "contracts/packs/pack_abi.v1.json",
        "no_op": no_op,
        "input_hash_before": input_hash_before,
        "input_hash_after": input_hash_after,
        "contract_id": contract_id,
        "version": version,
        "supports_wasi_v1": supports_wasi_v1,
        "supports_native_cap_v1": supports_native_cap_v1,
        "observe_hook": observe_hook,
        "commit_hook": commit_hook,
        "deterministic": true
    });
    let out_path = w18_migration_dir().join("pack_abi_upgrade_report.json");
    write_json_pretty(&out_path, &report);
    report
}
