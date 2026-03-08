use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    init_project, resolve_deps_v3, sign_deps_lock_v3_v15, sync_deps_lock_v1,
    write_permission_snapshot_v15,
};
use serde_json::Value as JsonValue;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_ocp_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocp-cli")
}

fn assert_success(output: &Output, step: &str) -> String {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn temp_dir_for_test(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = PathBuf::from("target")
        .join("tests")
        .join("lts_check")
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

fn canonical_slash_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_non_builtin_signer(lock_path: &Path) -> Option<String> {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    for line in raw.lines() {
        let Some(payload) = line.strip_prefix("dep=") else {
            continue;
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() >= 8 && parts.get(3) != Some(&"builtin") {
            return Some(parts[7].to_string());
        }
    }
    None
}

fn write_trust_toml_for_lts(root: &Path, non_builtin_signer: Option<&str>) {
    let mut trust = String::from(concat!(
        "[policy]\n",
        "mode = \"strict\"\n",
        "lane_locked_v071_requires_signed = true\n",
        "lane_locked_v06_requires_signed = false\n",
        "lane_quarantine_requires_signed = false\n"
    ));
    if let Some(signer) = non_builtin_signer {
        trust.push_str("\n[trusted_signers.registry]\nkeys = [\"");
        trust.push_str(signer);
        trust.push_str("\"]\n");
    }
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");
}

fn write_manifest_for_lts(root: &Path) {
    let manifest = concat!(
        "[package]\n",
        "name = \"lts_check_demo\"\n",
        "version = \"0.1.0\"\n\n",
        "[project]\n",
        "lane = \"locked_v071\"\n",
        "entry = \"src/main.ocp\"\n\n",
        "[dependencies]\n",
        "std = \"0.1.0\"\n\n",
        "[permissions.package]\n",
        "allow = [\"std.log.info\"]\n",
        "deny = []\n"
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");
}

fn write_conformance_manifest(manifest_path: &Path, project_root: &Path) {
    let project = canonical_slash_path(project_root);
    let manifest = format!(
        concat!(
            "version = 1\n",
            "\n",
            "[[scenario]]\n",
            "name = \"core-lts-check\"\n",
            "path = \"{}\"\n",
            "lane = \"locked_v071\"\n",
            "steps = [\"check\"]\n"
        ),
        project
    );
    fs::write(manifest_path, manifest).expect("write conformance manifest");
}

fn write_w16_security_reports_from_lts(report: &JsonValue) {
    let out_dir = repo_root()
        .join("target")
        .join("ocp")
        .join("w16")
        .join("security");
    fs::create_dir_all(&out_dir).expect("create w16 security output dir");

    let lts_report = serde_json::json!({
        "schema": "ocp.w16.security.lts_strict.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "source_schema": report.get("schema").cloned().unwrap_or(JsonValue::Null),
        "ok": report.get("ok").cloned().unwrap_or(JsonValue::Bool(false)),
        "gates": report.get("gates").cloned().unwrap_or(JsonValue::Null),
        "risks": report.get("risks").cloned().unwrap_or(JsonValue::Null)
    });
    fs::write(
        out_dir.join("lts_strict_report.json"),
        serde_json::to_string_pretty(&lts_report).expect("serialize lts_strict_report"),
    )
    .expect("write lts_strict_report.json");

    let security_chain = serde_json::json!({
        "schema": "ocp.w16.security.chain.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "lock_signature_ok": report
            .pointer("/gates/lock_signature/ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "trust_ok": report
            .pointer("/gates/trust/ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "permission_review_ok": report
            .pointer("/gates/permission_review/ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "upgrade_check_ok": report
            .pointer("/gates/upgrade_check/ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "conformance_ok": report
            .pointer("/gates/conformance/ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "overall_ok": report.get("ok").and_then(JsonValue::as_bool).unwrap_or(false)
    });
    fs::write(
        out_dir.join("security_chain_report.json"),
        serde_json::to_string_pretty(&security_chain).expect("serialize security_chain_report"),
    )
    .expect("write security_chain_report.json");
}

fn prepare_lts_project(root: &Path, with_permission_baseline: bool) -> (PathBuf, PathBuf) {
    let project = root.join("project");
    init_project(&project).expect("init project");
    write_manifest_for_lts(&project);
    sync_deps_lock_v1(&project).expect("sync deps.lock");
    let resolve_summary = resolve_deps_v3(&project, false).expect("resolve deps.lock.v3");
    sign_deps_lock_v3_v15(&project, "ci-lts").expect("sign deps.lock.v3");
    let signer = read_non_builtin_signer(&resolve_summary.lock_v3_path);
    write_trust_toml_for_lts(&project, signer.as_deref());
    if with_permission_baseline {
        write_permission_snapshot_v15(&project, None).expect("write permission baseline");
    }

    let manifest_path = root.join("conformance.v1.toml");
    write_conformance_manifest(&manifest_path, &project);
    (project, manifest_path)
}

#[test]
fn lts_check_passes_and_report_is_actionable() {
    let root = temp_dir_for_test("pass");
    let (project, manifest_path) = prepare_lts_project(&root, true);
    let report_path = root.join("lts_report.json");
    let project_s = canonical_slash_path(&project);
    let manifest_s = canonical_slash_path(&manifest_path);
    let report_s = canonical_slash_path(&report_path);

    let out = run_ocp_cli(&[
        "lts",
        "check",
        &project_s,
        "--manifest",
        &manifest_s,
        "--out",
        &report_s,
        "--json",
    ]);
    let stdout = assert_success(&out, "lts check");
    let report: JsonValue = serde_json::from_str(&stdout).expect("parse lts report");
    assert_eq!(
        report.get("schema").and_then(JsonValue::as_str),
        Some("ocp.lts_check.v1")
    );
    assert_eq!(report.get("ok").and_then(JsonValue::as_bool), Some(true));
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("lock_signature"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("trust"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("permission_review"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("upgrade_check"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("conformance"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert!(
        report
            .get("risks")
            .and_then(JsonValue::as_array)
            .map(|v| v.is_empty())
            .unwrap_or(false),
        "lts report should not contain risks when all gates pass"
    );
    assert!(report_path.exists(), "lts report file missing");
    write_w16_security_reports_from_lts(&report);

    let report_view = run_ocp_cli(&["lts", "report", &report_s, "--json"]);
    let report_stdout = assert_success(&report_view, "lts report");
    let report_view_json: JsonValue =
        serde_json::from_str(&report_stdout).expect("parse lts report view");
    assert_eq!(report_view_json.get("ok"), report.get("ok"));
}

#[test]
fn lts_check_fails_when_permission_baseline_missing() {
    let root = temp_dir_for_test("missing-baseline");
    let (project, manifest_path) = prepare_lts_project(&root, false);
    let report_path = root.join("lts_report_missing_baseline.json");
    let project_s = canonical_slash_path(&project);
    let manifest_s = canonical_slash_path(&manifest_path);
    let report_s = canonical_slash_path(&report_path);

    let out = run_ocp_cli(&[
        "lts",
        "check",
        &project_s,
        "--manifest",
        &manifest_s,
        "--out",
        &report_s,
        "--json",
    ]);
    assert!(
        !out.status.success(),
        "lts check should fail when permissions.snapshot.json is missing\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let report: JsonValue = serde_json::from_str(&stdout).expect("parse failed lts report");
    assert_eq!(report.get("ok").and_then(JsonValue::as_bool), Some(false));
    assert_eq!(
        report.get("error_code").and_then(JsonValue::as_str),
        Some("X-LTS-CHECK-FAILED")
    );
    assert_eq!(
        report
            .get("gates")
            .and_then(|g| g.get("permission_review"))
            .and_then(|v| v.get("ok"))
            .and_then(JsonValue::as_bool),
        Some(false)
    );
    let risk_codes: Vec<String> = report
        .get("risks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            v.get("code")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string())
        })
        .collect();
    assert!(
        risk_codes
            .iter()
            .any(|code| code == "X-PERMISSION-REVIEW-REQUIRED"),
        "risk codes should include X-PERMISSION-REVIEW-REQUIRED, got: {:?}",
        risk_codes
    );
}
