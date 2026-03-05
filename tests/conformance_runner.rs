use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    init_project, parse_conformance_manifest_v1, run_conformance_v1, ConformanceRunOptionsV1,
    ConformanceStepV1,
};

fn temp_dir_for_test(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = PathBuf::from("target")
        .join("tests")
        .join("conformance_runner")
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

#[test]
fn parse_manifest_supports_lane_signature_and_replay_artifact_step() {
    let root = temp_dir_for_test("manifest-extended");
    let manifest_path = root.join("conformance.v1.toml");
    let manifest = concat!(
        "version = 1\n",
        "\n",
        "[[scenario]]\n",
        "name = \"q-replay\"\n",
        "path = \"./case_project\"\n",
        "lane = \"quarantine\"\n",
        "expected_status = \"pass\"\n",
        "expected_signature_file = \"expected/signature.txt\"\n",
        "steps = [\"replay_artifact\"]\n",
    );
    fs::write(&manifest_path, manifest).expect("write manifest");

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    assert_eq!(parsed.scenarios.len(), 1);
    let sc = &parsed.scenarios[0];
    assert_eq!(sc.lane, "quarantine");
    assert_eq!(
        sc.expected_signature_file.as_deref(),
        Some("expected/signature.txt")
    );
    assert_eq!(sc.steps, vec![ConformanceStepV1::ReplayArtifact]);
}

#[test]
fn parse_manifest_defaults_lane_to_locked_v071() {
    let root = temp_dir_for_test("manifest-default-lane");
    let manifest_path = root.join("conformance.v1.toml");
    let manifest = concat!(
        "version = 1\n",
        "\n",
        "[[scenario]]\n",
        "name = \"default-lane\"\n",
        "path = \"./project\"\n",
        "expected_status = \"pass\"\n",
        "steps = [\"run\"]\n",
    );
    fs::write(&manifest_path, manifest).expect("write manifest");

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    assert_eq!(parsed.scenarios.len(), 1);
    assert_eq!(parsed.scenarios[0].lane, "locked_v071");
}

#[test]
fn run_conformance_reports_signature_mismatch() {
    let root = temp_dir_for_test("run-signature-mismatch");
    let project = root.join("project");
    init_project(&project).expect("init project");
    let ocl_toml = project.join("Ocl.toml");
    let mut manifest_text = fs::read_to_string(&ocl_toml).expect("read Ocl.toml");
    manifest_text.push_str(
        "\n[permissions.package]\nallow = [\"std.log.info\", \"std.fs.read_text\"]\ndeny = []\n",
    );
    fs::write(&ocl_toml, manifest_text).expect("patch Ocl.toml permissions");

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

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    let report = run_conformance_v1(&root, &parsed, ConformanceRunOptionsV1::default());
    assert_eq!(report.scenarios_total, 1);
    assert_eq!(report.scenarios_failed, 1);
    let reason = report.results[0].reason.clone().unwrap_or_default();
    assert!(
        reason.contains("X-CONFORMANCE-SIGNATURE-MISMATCH"),
        "unexpected reason: {reason}"
    );
}
