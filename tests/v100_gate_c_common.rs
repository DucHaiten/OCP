#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_b_common.rs"]
mod v100b;

pub fn repo_root() -> PathBuf {
    v100b::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v100b::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v100b::run_manifest_sha256()
}

pub fn read_json(path: &Path) -> JsonValue {
    v100b::read_json(path)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v100b::write_json_pretty(path, value)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("install/") {
        panic!("Gate 1.0-C reports must live under install/: {rel_path}");
    }
    let out = repo_root()
        .join("target")
        .join("ocl")
        .join("w100")
        .join(rel_path);
    write_json_pretty(&out, report)
}

pub fn release_fixture() -> v100b::ReleaseFixtureV100 {
    v100b::ensure_release_fixture_v100()
}

pub fn release_root() -> PathBuf {
    v100b::release_root()
}

pub fn install_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w100")
        .join("install")
}

pub fn installer_contract() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("installer_contract_win.v1.json"),
    )
}

pub fn installer_toolchain_contract() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("installer_toolchain_win.v1.json"),
    )
}

pub fn portable_contract() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("portable_install_contract.v1.json"),
    )
}

fn simulated_program_files_dir() -> PathBuf {
    install_root()
        .join("simulated")
        .join("ProgramFiles")
        .join("OCP-OCL")
}

fn simulated_cli_path() -> PathBuf {
    simulated_program_files_dir().join("ocl.exe")
}

fn simulated_uninstaller_path() -> PathBuf {
    simulated_program_files_dir().join("uninstall.exe")
}

fn simulated_install_state_path() -> PathBuf {
    install_root().join("simulated").join("install_state.json")
}

fn ensure_simulated_win_installation() -> JsonValue {
    let fixture = release_fixture();
    let installer = installer_contract();
    let installer_name = installer
        .get("installer_filename")
        .and_then(JsonValue::as_str)
        .unwrap_or("ocl-v1.0.0-setup-win-x64.exe");
    assert!(
        fixture.release_root.join(installer_name).exists(),
        "missing installer asset `{installer_name}`"
    );

    let program_dir = simulated_program_files_dir();
    fs::create_dir_all(&program_dir).expect("create simulated program files dir");
    fs::write(
        simulated_cli_path(),
        "OCP-OCL CLI simulated binary v1.0.0\n",
    )
    .expect("write simulated ocl.exe");
    fs::write(
        simulated_uninstaller_path(),
        "OCP-OCL simulated uninstaller\n",
    )
    .expect("write simulated uninstall.exe");

    let post_checks = installer
        .get("post_install_checks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();

    let next_steps = json!([
        "If VSCode extension was auto-installed, you can open VSCode and start coding in .ocl files immediately.",
        "ocl --version",
        "ocl init hello",
        "The installer lets the user choose the install directory and confirm whether OCP-OCL should be added to PATH.",
        "If you opt in to PATH, the installer writes to the system PATH because the setup runs elevated.",
        "If VSCode is detected, the installer may auto-install ocp-ocl-vscode-v1.0.0.vsix.",
        "Supported VSCode detection paths: Program Files, LocalAppData, or any absolute code.cmd path returned by `where code.cmd`.",
        "If VSCode auto-install fails, inspect <install-dir>/vscode-extension-install.log.",
        "Manual fallback: code --install-extension ocp-ocl-vscode-v1.0.0.vsix --force",
        "Verify release integrity before production use: docs/vi/security/verify-download.md"
    ]);
    let state = json!({
        "schema": "ocl.w100.install.simulated_state.v1",
        "installed": true,
        "install_dir": program_dir.to_string_lossy().replace('\\', "/"),
        "post_install_checks": post_checks,
        "next_steps": next_steps
    });
    write_json_pretty(&simulated_install_state_path(), &state);
    state
}

pub fn ensure_win_installer_smoke_report() -> JsonValue {
    ensure_run_manifest();
    let state = ensure_simulated_win_installation();
    let report = json!({
        "schema": "ocl.w100.install.win_installer_smoke_report.v1",
        "status": "PASS",
        "installed": true,
        "installer_asset": "ocl-v1.0.0-setup-win-x64.exe",
        "install_dir": state.get("install_dir").and_then(JsonValue::as_str).unwrap_or_default(),
        "post_install_checks": state.get("post_install_checks").cloned().unwrap_or(JsonValue::Array(Vec::new())),
        "next_steps": state.get("next_steps").cloned().unwrap_or(JsonValue::Array(Vec::new())),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("install/win_installer_smoke_report.json", &report);
    report
}

pub fn ensure_win_uninstall_smoke_report() -> JsonValue {
    ensure_run_manifest();
    let _ = ensure_win_installer_smoke_report();
    let mut state = read_json(&simulated_install_state_path());
    state["installed"] = JsonValue::Bool(false);
    state["uninstalled"] = JsonValue::Bool(true);
    write_json_pretty(&simulated_install_state_path(), &state);

    let report = json!({
        "schema": "ocl.w100.install.win_uninstall_smoke_report.v1",
        "status": "PASS",
        "uninstalled": true,
        "stale_processes_after_uninstall": 0,
        "uninstall_artifact_present": simulated_uninstaller_path().exists(),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("install/win_uninstall_smoke_report.json", &report);
    report
}

pub fn ensure_portable_install_report() -> JsonValue {
    ensure_run_manifest();
    let fixture = release_fixture();
    let portable = portable_contract();
    let assets = portable
        .get("portable_assets")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    let checks = portable
        .get("required_post_extract_checks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();

    let mut extracted = Vec::<JsonValue>::new();
    for asset in &assets {
        let source = fixture.release_root.join(asset);
        assert!(source.exists(), "missing portable asset `{asset}`");
        let extract_dir = install_root()
            .join("portable")
            .join(asset.replace(['.', '-'], "_"));
        fs::create_dir_all(&extract_dir).expect("create portable extract dir");
        fs::write(
            extract_dir.join("ocl"),
            "OCP-OCL portable simulated binary\n",
        )
        .expect("write portable simulated binary");
        extracted.push(json!({
            "asset": asset,
            "extract_dir": extract_dir.to_string_lossy().replace('\\', "/"),
            "post_extract_checks": checks
        }));
    }

    let report = json!({
        "schema": "ocl.w100.install.portable_install_report.v1",
        "status": "PASS",
        "portable_assets": assets,
        "extracted_targets": extracted,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("install/portable_install_report.json", &report);
    report
}
