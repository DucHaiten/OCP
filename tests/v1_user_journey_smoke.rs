use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let dir = repo_root()
        .join("target")
        .join("tests")
        .join("v1_user_journey_smoke")
        .join(format!("{tag}-{stamp}"));
    fs::create_dir_all(&dir).expect("create v1 journey temp dir");
    dir
}

fn run_ocl_cli(args: &[&str], envs: &BTreeMap<&str, &str>) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocl-cli")
        .arg("--quiet")
        .arg("--")
        .args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run ocl-cli")
}

fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn list_dirs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let read = fs::read_dir(path).expect("read dir");
    for entry in read {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn latest_artifact_dir(project_root: &Path) -> PathBuf {
    let mut dirs = list_dirs(&project_root.join(".ocl_artifacts"));
    assert!(!dirs.is_empty(), "missing run artifacts in .ocl_artifacts");
    dirs.pop().expect("latest artifact")
}

fn patch_manifest_for_permission_delta(project_root: &Path) {
    let manifest_path = project_root.join("Ocl.toml");
    let raw = fs::read_to_string(&manifest_path).expect("read Ocl.toml");
    let from = "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\"]";
    let to = "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\", \"std.proc.*\"]";
    let patched = raw.replace(from, to);
    assert_ne!(
        raw, patched,
        "permission delta patch must update [permissions.package].allow"
    );
    fs::write(manifest_path, patched).expect("write patched Ocl.toml");
}

#[test]
fn v1_user_journey_smoke_runs_end_to_end_and_writes_rc_reports() {
    let root = temp_project_dir("journey");
    let root_s = root.to_string_lossy().to_string();
    let empty_env = BTreeMap::new();
    let mut steps = Vec::new();

    let init = run_ocl_cli(&["init", &root_s, "--template", "tool-cli"], &empty_env);
    assert_ok(&init, "init tool-cli");
    steps.push(json!({
        "name": "init_template",
        "ok": true,
        "evidence": "project initialized with template tool-cli"
    }));

    let lock_sync = run_ocl_cli(&["lock", "sync", &root_s], &empty_env);
    assert_ok(&lock_sync, "lock sync");
    steps.push(json!({
        "name": "lock_sync",
        "ok": true,
        "evidence": "deps.lock.v3 generated"
    }));

    let lock_sign = run_ocl_cli(&["lock", "sign", &root_s, "--key", "ci-rc"], &empty_env);
    assert_ok(&lock_sign, "lock sign");
    steps.push(json!({
        "name": "lock_sign",
        "ok": true,
        "evidence": "deps.lock.v3.sig generated"
    }));

    let lock_verify = run_ocl_cli(&["lock", "verify", &root_s], &empty_env);
    assert_ok(&lock_verify, "lock verify");
    steps.push(json!({
        "name": "lock_verify",
        "ok": true,
        "evidence": "signed lock verified"
    }));

    let perm_old_dir = root.join("perm_old");
    let perm_new_dir = root.join("perm_new");
    fs::create_dir_all(&perm_old_dir).expect("create perm_old");
    fs::create_dir_all(&perm_new_dir).expect("create perm_new");
    let perm_old_s = perm_old_dir.to_string_lossy().to_string();
    let perm_new_s = perm_new_dir.to_string_lossy().to_string();

    let perm_snapshot_old = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &perm_old_s],
        &empty_env,
    );
    assert_ok(&perm_snapshot_old, "perm snapshot old");

    patch_manifest_for_permission_delta(&root);

    let perm_snapshot_new = run_ocl_cli(
        &["perm", "snapshot", &root_s, "--out-dir", &perm_new_s],
        &empty_env,
    );
    assert_ok(&perm_snapshot_new, "perm snapshot new");

    let old_snapshot = perm_old_dir.join("permissions.snapshot.json");
    let new_snapshot = perm_new_dir.join("permissions.snapshot.json");
    let diff_report = root.join("permission_diff_report.json");
    let approval_file = root.join("permissions.approval.toml");
    let old_snapshot_s = old_snapshot.to_string_lossy().to_string();
    let new_snapshot_s = new_snapshot.to_string_lossy().to_string();
    let diff_report_s = diff_report.to_string_lossy().to_string();
    let approval_file_s = approval_file.to_string_lossy().to_string();

    let diff_before = run_ocl_cli(
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
    assert!(
        !diff_before.status.success(),
        "perm diff must fail before approval"
    );
    assert!(
        String::from_utf8_lossy(&diff_before.stderr).contains("RC-PERMISSION-UNAPPROVED"),
        "perm diff before approval must expose RC-PERMISSION-UNAPPROVED"
    );

    let perm_approve = run_ocl_cli(
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
            "approve for v16 gate 16-g smoke",
        ],
        &empty_env,
    );
    assert_ok(&perm_approve, "perm approve");

    let diff_after = run_ocl_cli(
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
    assert_ok(&diff_after, "perm diff after approval");
    steps.push(json!({
        "name": "perm_snapshot_diff_approve",
        "ok": true,
        "evidence": "permission delta reviewed and approved"
    }));

    let build_attest = run_ocl_cli(&["build", &root_s, "--source-only", "--attest"], &empty_env);
    assert_ok(&build_attest, "build --attest");
    let attestation_dir = root.join("target").join("ocl").join("attestation");
    assert!(
        attestation_dir.join("build_manifest.json").exists(),
        "missing build_manifest.json"
    );
    assert!(
        attestation_dir.join("build_manifest.sig").exists(),
        "missing build_manifest.sig"
    );

    let attestation_dir_s = attestation_dir.to_string_lossy().to_string();
    let verify_attest = run_ocl_cli(&["verify", "--attest", &attestation_dir_s], &empty_env);
    assert_ok(&verify_attest, "verify --attest");
    steps.push(json!({
        "name": "build_and_verify_attest",
        "ok": true,
        "evidence": "attestation generated and verified"
    }));

    let run = run_ocl_cli(&["run", &root_s], &empty_env);
    assert_ok(&run, "run");
    let artifact = latest_artifact_dir(&root);

    let replay = run_ocl_cli(&["replay", &artifact.to_string_lossy()], &empty_env);
    assert_ok(&replay, "replay");
    steps.push(json!({
        "name": "run_and_replay",
        "ok": true,
        "evidence": "run artifact replay verified"
    }));

    let dbg_script = root.join("dbg.script.txt");
    fs::write(
        &dbg_script,
        concat!("where\n", "step\n", "locals\n", "last\n"),
    )
    .expect("write dbg script");
    let dbg = run_ocl_cli(
        &[
            "dbg",
            &artifact.to_string_lossy(),
            "--script",
            &dbg_script.to_string_lossy(),
        ],
        &empty_env,
    );
    assert_ok(&dbg, "dbg smoke");
    let dbg_stdout = String::from_utf8_lossy(&dbg.stdout);
    assert!(dbg_stdout.contains("dbg v11"), "missing debugger header");
    steps.push(json!({
        "name": "dbg_smoke",
        "ok": true,
        "evidence": "debugger script executed"
    }));

    let rc_dir = repo_root()
        .join("target")
        .join("ocl")
        .join("w16")
        .join("rc");
    fs::create_dir_all(&rc_dir).expect("create w16 rc output dir");

    let journey_report = json!({
        "schema": "ocl.w16.rc.golden_user_journey.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "project_root": root.to_string_lossy(),
        "steps": steps,
        "artifacts": {
            "attestation_dir": attestation_dir.to_string_lossy(),
            "run_artifact_dir": artifact.to_string_lossy(),
            "permission_diff_report": diff_report.to_string_lossy(),
            "permission_approval_file": approval_file.to_string_lossy()
        },
        "pass": true
    });
    fs::write(
        rc_dir.join("golden_user_journey_report.json"),
        serde_json::to_string_pretty(&journey_report).expect("serialize journey report"),
    )
    .expect("write golden_user_journey_report.json");

    let rc_dryrun_report = json!({
        "schema": "ocl.w16.rc.dryrun.v1",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "status": "PASS",
        "checks": {
            "golden_user_journey": true,
            "attestation_verify": true,
            "lock_signature_verify": true,
            "permission_approval_enforced": true,
            "replay_verified": true,
            "debugger_smoke": true
        },
        "pass": true
    });
    fs::write(
        rc_dir.join("rc_dryrun_report.json"),
        serde_json::to_string_pretty(&rc_dryrun_report).expect("serialize rc dryrun report"),
    )
    .expect("write rc_dryrun_report.json");
}
