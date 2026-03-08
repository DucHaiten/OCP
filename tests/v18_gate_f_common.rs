#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{contract_sig_path_v18, sign_contract_json_v18};
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v18_gate_a_common.rs"]
mod gate_a;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn w18_rc_dir() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocp")
        .join("w18")
        .join("rc")
}

pub fn ensure_run_manifest() {
    let _ = gate_a::ensure_run_manifest();
}

pub fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let dir = repo_root()
        .join("target")
        .join("tests")
        .join("v18_gate_f")
        .join(format!("{tag}-{stamp}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
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

pub fn list_dirs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let read = fs::read_dir(path).expect("read dir");
    for entry in read {
        let p = entry.expect("entry").path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

pub fn latest_artifact_dir(project_root: &Path) -> PathBuf {
    let mut dirs = list_dirs(&project_root.join(".ocp_artifacts"));
    assert!(!dirs.is_empty(), "missing .ocp_artifacts run dirs");
    dirs.pop().expect("latest artifact dir")
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

pub fn sha256_hex_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

pub fn release_required_artifacts() -> Vec<&'static str> {
    vec![
        "target/ocp/w18/meta/run_manifest.json",
        "target/ocp/w18/contracts/contract_inventory.json",
        "target/ocp/w18/contracts/contract_inventory_completeness_report.json",
        "target/ocp/w18/contracts/sot_signature_report.json",
        "target/ocp/w18/migration/cassette_upgrade_report.json",
        "target/ocp/w18/migration/manifest_upgrade_report.json",
        "target/ocp/w18/migration/pack_abi_upgrade_report.json",
        "target/ocp/w18/migration/migration_noop_report.json",
        "target/ocp/w18/ops/doctor_report.json",
        "target/ocp/w18/ops/fix_plan_report.json",
        "target/ocp/w18/ops/budget_analyze_report.json",
        "target/ocp/w18/ops/doctor_fix_cases_report.json",
        "target/ocp/w18/cassette/cassette_operability_report.json",
        "target/ocp/w18/security/privacy_hygiene_report.json",
        "target/ocp/w18/security/dos_caps_report.json",
        "target/ocp/w18/security/fs_boundary_report.json",
        "target/ocp/w18/packs/pack_shipproof_report.json",
        "target/ocp/w18/packs/connector_baseline_report.json",
    ]
}

pub fn ensure_release_artifact_manifest_signed() -> (PathBuf, PathBuf) {
    ensure_run_manifest();
    let root = repo_root();
    let rc_dir = w18_rc_dir();
    fs::create_dir_all(&rc_dir).expect("create rc dir");

    let required = release_required_artifacts();
    let mut artifacts = Vec::<JsonValue>::new();
    for rel in required {
        let full = root.join(rel);
        assert!(
            full.exists(),
            "required artifact missing: {}",
            full.display()
        );
        let sig_path = contract_sig_path_v18(&full);
        artifacts.push(json!({
            "path": rel,
            "sha256": sha256_hex_file(&full),
            "signature_status": if sig_path.exists() { "signed" } else { "unsigned" }
        }));
    }

    let manifest = json!({
        "schema": "ocp.w18.rc.release_artifact_manifest.v1",
        "contract_id": "v1.release_artifact_manifest",
        "version": "v1",
        "hasher_version": "sha256-v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "toolchain_digest_ref": "target/ocp/w18/meta/run_manifest.json",
        "artifacts": artifacts
    });
    let manifest_path = rc_dir.join("release_artifact_manifest.json");
    write_json_pretty(&manifest_path, &manifest);

    let sig_path = sign_contract_json_v18(&manifest_path, "w18-sot-root", 1)
        .expect("sign release_artifact_manifest.json");
    let compat_sig_path = rc_dir.join("release_artifact_manifest.sig");
    let sig_bytes = fs::read(&sig_path).expect("read canonical release manifest sig");
    fs::write(&compat_sig_path, sig_bytes).expect("write compatibility release manifest sig");
    (manifest_path, sig_path)
}

pub fn patch_manifest_for_permission_delta(project_root: &Path) {
    let manifest_path = project_root.join("Ocp.toml");
    let raw = fs::read_to_string(&manifest_path).expect("read Ocp.toml");
    let from = "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\"]";
    let to = "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\", \"std.proc.*\"]";
    let patched = raw.replace(from, to);
    assert_ne!(
        raw, patched,
        "permission delta patch must update [permissions.package].allow"
    );
    fs::write(manifest_path, patched).expect("write patched Ocp.toml");
}
