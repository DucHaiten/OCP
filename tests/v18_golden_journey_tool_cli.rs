use std::collections::BTreeMap;
use std::fs;

use serde_json::json;

#[path = "v18_gate_f_common.rs"]
mod common;

#[test]
fn v18_golden_journey_tool_cli_runs_end_to_end() {
    common::ensure_run_manifest();
    let root = common::temp_project_dir("tool_cli");
    let root_s = root.to_string_lossy().to_string();
    let empty_env = BTreeMap::new();
    let mut steps = Vec::new();

    let init = common::run_ocl_cli(&["init", &root_s, "--template", "tool-cli"], &empty_env);
    let _ = common::assert_ok(&init, "init tool-cli");
    steps.push(json!({"name":"init","ok":true}));

    let lock_sync = common::run_ocl_cli(&["lock", "sync", &root_s], &empty_env);
    let _ = common::assert_ok(&lock_sync, "lock sync");
    let lock_sign = common::run_ocl_cli(&["lock", "sign", &root_s, "--key", "ci-rc"], &empty_env);
    let _ = common::assert_ok(&lock_sign, "lock sign");
    let lock_verify = common::run_ocl_cli(&["lock", "verify", &root_s], &empty_env);
    let _ = common::assert_ok(&lock_verify, "lock verify");
    steps.push(json!({"name":"lock_chain","ok":true}));

    let perm_old_dir = root.join("perm_old");
    let perm_new_dir = root.join("perm_new");
    fs::create_dir_all(&perm_old_dir).expect("create perm_old");
    fs::create_dir_all(&perm_new_dir).expect("create perm_new");
    let perm_old_s = perm_old_dir.to_string_lossy().to_string();
    let perm_new_s = perm_new_dir.to_string_lossy().to_string();

    let snapshot_old = common::run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &perm_old_s],
        &empty_env,
    );
    let _ = common::assert_ok(&snapshot_old, "perm snapshot old");

    common::patch_manifest_for_permission_delta(&root);

    let snapshot_new = common::run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &perm_new_s],
        &empty_env,
    );
    let _ = common::assert_ok(&snapshot_new, "perm snapshot new");

    let old_snapshot = perm_old_dir.join("permissions.snapshot.json");
    let new_snapshot = perm_new_dir.join("permissions.snapshot.json");
    let diff_report = root.join("permission_diff_report.json");
    let approval_file = root.join("permissions.approval.toml");
    let old_snapshot_s = old_snapshot.to_string_lossy().to_string();
    let new_snapshot_s = new_snapshot.to_string_lossy().to_string();
    let diff_report_s = diff_report.to_string_lossy().to_string();
    let approval_file_s = approval_file.to_string_lossy().to_string();

    let diff_before = common::run_ocl_cli(
        &[
            "perm",
            "diff",
            &old_snapshot_s,
            &new_snapshot_s,
            "--out",
            &diff_report_s,
            "--approval",
            &approval_file_s,
        ],
        &empty_env,
    );
    let diff_before_err = common::assert_fail(&diff_before, "perm diff before approval");
    assert!(
        diff_before_err.contains("RC-PERMISSION-UNAPPROVED"),
        "perm diff before approval must fail with RC-PERMISSION-UNAPPROVED"
    );

    let approve = common::run_ocl_cli(
        &[
            "perm",
            "approve",
            &diff_report_s,
            "--approval",
            &approval_file_s,
            "--by",
            "ci-bot",
            "--date",
            "2026-03-06",
            "--note",
            "approve for v18 gate 18-f golden tool-cli",
        ],
        &empty_env,
    );
    let _ = common::assert_ok(&approve, "perm approve");

    let diff_after = common::run_ocl_cli(
        &[
            "perm",
            "diff",
            &old_snapshot_s,
            &new_snapshot_s,
            "--out",
            &diff_report_s,
            "--approval",
            &approval_file_s,
        ],
        &empty_env,
    );
    let _ = common::assert_ok(&diff_after, "perm diff after approval");
    steps.push(json!({"name":"perm_review","ok":true}));

    let build_attest =
        common::run_ocl_cli(&["build", &root_s, "--source-only", "--attest"], &empty_env);
    let _ = common::assert_ok(&build_attest, "build --attest");
    let attest_dir = root.join("target").join("ocl").join("attestation");
    let attest_dir_s = attest_dir.to_string_lossy().to_string();
    let verify_attest = common::run_ocl_cli(&["verify", "--attest", &attest_dir_s], &empty_env);
    let _ = common::assert_ok(&verify_attest, "verify --attest");
    steps.push(json!({"name":"attestation_chain","ok":true}));

    let run = common::run_ocl_cli(&["run", &root_s], &empty_env);
    let _ = common::assert_ok(&run, "run");
    let artifact = common::latest_artifact_dir(&root);
    let replay = common::run_ocl_cli(&["replay", &artifact.to_string_lossy()], &empty_env);
    let _ = common::assert_ok(&replay, "replay");
    steps.push(json!({"name":"run_replay","ok":true}));

    let dbg_script = root.join("dbg.script.txt");
    fs::write(&dbg_script, "where\nstep\nlocals\nlast\n").expect("write dbg script");
    let dbg = common::run_ocl_cli(
        &[
            "dbg",
            &artifact.to_string_lossy(),
            "--script",
            &dbg_script.to_string_lossy(),
        ],
        &empty_env,
    );
    let dbg_stdout = common::assert_ok(&dbg, "dbg smoke");
    assert!(dbg_stdout.contains("dbg v11"), "missing debugger header");
    steps.push(json!({"name":"dbg_smoke","ok":true}));

    let report = json!({
        "schema": "ocl.w18.rc.golden_journey_tool_cli_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
        "project_root": root.to_string_lossy().replace('\\', "/"),
        "steps": steps,
        "artifacts": {
            "run_artifact_dir": artifact.to_string_lossy().replace('\\', "/"),
            "attestation_dir": attest_dir.to_string_lossy().replace('\\', "/"),
            "permission_diff_report": diff_report.to_string_lossy().replace('\\', "/"),
            "permission_approval_file": approval_file.to_string_lossy().replace('\\', "/")
        },
        "status": "PASS"
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("golden_journey_tool_cli_report.json"),
        &report,
    );
}
