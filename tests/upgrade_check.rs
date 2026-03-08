use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::init_project;
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

fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
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
        .join("upgrade_check")
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

#[test]
fn upgrade_check_core_fixture_passes_with_json_report() {
    let out = run_ocp_cli(&[
        "upgrade-check",
        "projects/ocp/conformance/fixtures/core-exec-pass",
        "--manifest",
        "projects/ocp/conformance/conformance.v5.toml",
        "--json",
    ]);
    let stdout = assert_success(&out);
    let report: JsonValue = serde_json::from_str(&stdout).expect("upgrade-check report json");

    assert_eq!(
        report.get("schema").and_then(JsonValue::as_str),
        Some("ocp.upgrade_check.v1")
    );
    assert_eq!(report.get("ok").and_then(JsonValue::as_bool), Some(true));
    assert_eq!(
        report
            .get("conformance")
            .and_then(|c| c.get("scenarios_failed"))
            .and_then(JsonValue::as_u64),
        Some(0)
    );
    let selected = report
        .get("selected_scenarios")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        selected
            .iter()
            .filter_map(JsonValue::as_str)
            .any(|name| name == "core-exec-pass"),
        "selected subset must include core-exec-pass"
    );
}

#[test]
fn upgrade_check_reports_divergence_for_signature_mismatch() {
    let root = temp_dir_for_test("signature-mismatch");
    let project = root.join("project");
    init_project(&project).expect("init project");
    let manifest_toml = project.join("Ocp.toml");
    let mut manifest_text = fs::read_to_string(&manifest_toml).expect("read Ocp.toml");
    manifest_text.push_str(
        "\n[permissions.package]\nallow = [\"std.log.info\", \"std.fs.read_text\"]\ndeny = []\n",
    );
    fs::write(&manifest_toml, manifest_text).expect("patch Ocp.toml permissions");

    let expected_dir = root.join("expected");
    fs::create_dir_all(&expected_dir).expect("create expected dir");
    fs::write(expected_dir.join("signature.txt"), "deadbeef\n").expect("write expected signature");

    let manifest_path = root.join("conformance.v1.toml");
    let project_path = fs::canonicalize(&project)
        .expect("canonical project")
        .to_string_lossy()
        .replace('\\', "/");
    let manifest = format!(
        concat!(
            "version = 1\n",
            "\n",
            "[[scenario]]\n",
            "name = \"sig-mismatch\"\n",
            "path = \"{}\"\n",
            "lane = \"locked_v071\"\n",
            "expected_status = \"pass\"\n",
            "expected_signature_file = \"../expected/signature.txt\"\n",
            "steps = [\"check\"]\n",
        ),
        project_path
    );
    fs::write(&manifest_path, manifest).expect("write conformance manifest");

    let out = run_ocp_cli(&[
        "upgrade-check",
        project.to_string_lossy().as_ref(),
        "--manifest",
        manifest_path.to_string_lossy().as_ref(),
        "--json",
    ]);
    assert!(
        !out.status.success(),
        "upgrade-check should fail on signature mismatch\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let report: JsonValue = serde_json::from_slice(&out.stdout).expect("upgrade-check json output");
    assert_eq!(report.get("ok").and_then(JsonValue::as_bool), Some(false));
    assert_eq!(
        report.get("error_code").and_then(JsonValue::as_str),
        Some("X-UPGRADE-CHECK-FAILED")
    );
    let reason = report
        .get("first_divergence")
        .and_then(|v| v.get("reason"))
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    assert!(
        reason.contains("X-CONFORMANCE-SIGNATURE-MISMATCH"),
        "upgrade-check must expose signature mismatch reason, got: {reason}"
    );
}
