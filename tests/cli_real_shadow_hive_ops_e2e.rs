use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

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
        .join("cli_real_shadow_hive_ops")
        .join(format!("{tag}-{stamp}"));
    fs::create_dir_all(&dir).expect("create temp project dir");
    dir
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

fn assert_ok(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_fail_contains(output: &Output, step: &str, needle: &str) {
    assert!(
        !output.status.success(),
        "{step} unexpectedly passed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "{step} missing `{needle}`:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
}

fn list_dirs(path: &Path) -> Vec<PathBuf> {
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

fn latest_artifact_dir(project_root: &Path) -> PathBuf {
    let mut dirs = list_dirs(&project_root.join(".ocp_artifacts"));
    assert!(!dirs.is_empty(), "missing run artifacts in .ocp_artifacts");
    dirs.pop().expect("latest artifact")
}

fn parse_first_table_id(raw: &str, table: &str) -> String {
    let marker = format!("[[{table}]]");
    let mut inside = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == marker {
            inside = true;
            continue;
        }
        if inside && trimmed.starts_with("[[") {
            break;
        }
        if inside && trimmed.starts_with("id = \"") {
            let value = trimmed
                .split('"')
                .nth(1)
                .expect("id value after quote")
                .to_string();
            return value;
        }
    }
    panic!("unable to find first id in [[{table}]]");
}

fn write_reactor_entrypoint(project_root: &Path) {
    let source = r#"fn on_event(event) {
  let ok = true;
  condition(ok);
  return event;
}

let ready = true;
condition(ready);
"#;
    fs::write(project_root.join("src").join("main.ocp"), source).expect("write reactor source");
}

fn patch_hive_workers_in_cosmos_lock(lock_path: &Path, workers: u32) {
    let raw = fs::read_to_string(lock_path).expect("read cosmos.lock.v1");
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("hive=") {
            let mut cols = rest.split('|').map(|v| v.to_string()).collect::<Vec<_>>();
            assert_eq!(cols.len(), 6, "invalid hive row in cosmos lock: {line}");
            cols[0] = workers.to_string();
            lines.push(format!("hive={}", cols.join("|")));
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    assert!(replaced, "missing hive row in cosmos.lock.v1");
    fs::write(lock_path, format!("{}\n", lines.join("\n"))).expect("rewrite cosmos.lock.v1");
}

fn extract_shadow_digest(stdout: &str) -> String {
    let marker = "shadow_digest=";
    let start = stdout
        .find(marker)
        .unwrap_or_else(|| panic!("missing `{marker}` in stdout: {stdout}"));
    let tail = &stdout[start + marker.len()..];
    let end = tail
        .find(')')
        .unwrap_or_else(|| panic!("missing `)` after shadow digest in stdout: {stdout}"));
    tail[..end].trim().to_string()
}

fn read_dispatch_digest(report_path: &Path) -> String {
    let raw = fs::read_to_string(report_path).expect("read runtime report");
    let parsed: Value = serde_json::from_str(&raw).expect("parse runtime report json");
    parsed
        .get("dispatch_digest256")
        .and_then(Value::as_str)
        .expect("missing dispatch_digest256")
        .to_string()
}

#[test]
fn cli_real_shadow_command_flow_passes() {
    let root = temp_project_dir("shadow");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "shadow-preview"]);
    assert_ok(&init, "init shadow-preview");

    let check = run_ocp_cli(&["check", &root_s]);
    assert_ok(&check, "check");

    let run = run_ocp_cli(&[
        "run",
        &root_s,
        "--engine",
        "dual",
        "--shadow",
        "parity",
        "--shadow-policy",
        "forbid_commit",
    ]);
    assert_ok(&run, "run with shadow compare");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run_stdout.contains("shadow report written"),
        "run stdout must include shadow report evidence:\n{run_stdout}"
    );

    let artifact = latest_artifact_dir(&root);
    let artifact_s = artifact.to_string_lossy().to_string();
    let replay = run_ocp_cli(&["replay", &artifact_s]);
    assert_ok(&replay, "replay");

    let trace_out = root.join("shadow.trace.jsonl");
    let trace_out_s = trace_out.to_string_lossy().to_string();
    let trace = run_ocp_cli(&[
        "trace",
        "run",
        &root_s,
        "--engine",
        "dual",
        "--shadow",
        "parity",
        "--shadow-policy",
        "forbid_commit",
        "--out",
        &trace_out_s,
    ]);
    assert_ok(&trace, "trace run with shadow");
    assert!(trace_out.exists(), "missing trace output");
    let trace_size = fs::metadata(&trace_out).expect("trace metadata").len();
    assert!(trace_size > 0, "trace output must be non-empty");

    let profile_out = root.join("shadow.profile.json");
    let profile_out_s = profile_out.to_string_lossy().to_string();
    let profile = run_ocp_cli(&[
        "profile",
        "run",
        &root_s,
        "--engine",
        "dual",
        "--shadow",
        "parity",
        "--shadow-policy",
        "forbid_commit",
        "--out",
        &profile_out_s,
    ]);
    assert_ok(&profile, "profile run with shadow");
    assert!(profile_out.exists(), "missing profile output");
    let profile_size = fs::metadata(&profile_out).expect("profile metadata").len();
    assert!(profile_size > 0, "profile output must be non-empty");
}

#[test]
fn cli_real_hive_reactor_command_flow_and_locked_guard() {
    let root = temp_project_dir("hive");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--preset", "agent_swarm_basic"]);
    assert_ok(&init, "init agent_swarm_basic");

    let cosmos_raw = fs::read_to_string(root.join("cosmos.toml")).expect("read cosmos.toml");
    assert!(
        cosmos_raw.contains("[hive]"),
        "preset must materialize [hive] in cosmos.toml"
    );
    let universe_id = parse_first_table_id(&cosmos_raw, "universe");
    let domain_id = parse_first_table_id(&cosmos_raw, "domain");

    write_reactor_entrypoint(&root);

    let policy_sync = run_ocp_cli(&["policy", "lock", "sync", &root_s]);
    assert_ok(&policy_sync, "policy lock sync");
    let cosmos_sync = run_ocp_cli(&["cosmos", "lock", "sync", &root_s]);
    assert_ok(&cosmos_sync, "cosmos lock sync");

    let report = root.join("runtime_report.json");
    let replay_audit = root.join("replay_audit.jsonl");
    let report_s = report.to_string_lossy().to_string();
    let replay_audit_s = replay_audit.to_string_lossy().to_string();

    let run = run_ocp_cli(&[
        "run",
        &root_s,
        "--reactor",
        "--ticks",
        "4",
        "--runtime",
        "deterministic",
        "--universe",
        &universe_id,
        "--domain",
        &domain_id,
        "--runtime-report",
        &report_s,
        "--replay-audit",
        &replay_audit_s,
    ]);
    assert_ok(&run, "reactor run with hive-capable cosmos");
    assert!(report.exists(), "missing runtime report");
    assert!(replay_audit.exists(), "missing replay audit");
    let report_text = fs::read_to_string(&report).expect("read runtime report");
    assert!(
        report_text.contains("\"mode\":\"deterministic\""),
        "runtime report must confirm deterministic mode"
    );

    let lock_path = root.join("cosmos.lock.v1");
    patch_hive_workers_in_cosmos_lock(&lock_path, 15);

    let locked_run = run_ocp_cli(&[
        "run",
        &root_s,
        "--reactor",
        "--ticks",
        "4",
        "--runtime",
        "deterministic",
        "--locked",
        "--universe",
        &universe_id,
        "--domain",
        &domain_id,
        "--runtime-report",
        &report_s,
        "--replay-audit",
        &replay_audit_s,
    ]);
    assert_fail_contains(
        &locked_run,
        "locked reactor run with mismatched hive lock",
        "V-HIVE-LOCK-MISMATCH",
    );
}

#[test]
fn cli_real_shadow_digest_stable_across_repeated_runs() {
    let root = temp_project_dir("shadow_stable");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "shadow-preview"]);
    assert_ok(&init, "init shadow-preview");

    let mut digests = Vec::new();
    for _ in 0..5 {
        let run = run_ocp_cli(&[
            "run",
            &root_s,
            "--engine",
            "dual",
            "--shadow",
            "parity",
            "--shadow-policy",
            "forbid_commit",
        ]);
        assert_ok(&run, "shadow run");
        let stdout = String::from_utf8_lossy(&run.stdout);
        digests.push(extract_shadow_digest(&stdout));
    }

    let first = digests.first().expect("at least one digest");
    assert!(
        digests.iter().all(|d| d == first),
        "shadow digest drift detected across repeated runs: {digests:?}"
    );
}

#[test]
fn cli_real_hive_dispatch_digest_stable_across_repeated_runs() {
    let root = temp_project_dir("hive_stable");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--preset", "agent_swarm_basic"]);
    assert_ok(&init, "init agent_swarm_basic");
    write_reactor_entrypoint(&root);

    let cosmos_raw = fs::read_to_string(root.join("cosmos.toml")).expect("read cosmos.toml");
    let universe_id = parse_first_table_id(&cosmos_raw, "universe");
    let domain_id = parse_first_table_id(&cosmos_raw, "domain");

    let policy_sync = run_ocp_cli(&["policy", "lock", "sync", &root_s]);
    assert_ok(&policy_sync, "policy lock sync");
    let cosmos_sync = run_ocp_cli(&["cosmos", "lock", "sync", &root_s]);
    assert_ok(&cosmos_sync, "cosmos lock sync");

    let mut digests = Vec::new();
    for idx in 0..5 {
        let report = root.join(format!("runtime_report_{idx}.json"));
        let replay_audit = root.join(format!("replay_audit_{idx}.jsonl"));
        let report_s = report.to_string_lossy().to_string();
        let replay_audit_s = replay_audit.to_string_lossy().to_string();

        let run = run_ocp_cli(&[
            "run",
            &root_s,
            "--reactor",
            "--ticks",
            "4",
            "--runtime",
            "deterministic",
            "--universe",
            &universe_id,
            "--domain",
            &domain_id,
            "--runtime-report",
            &report_s,
            "--replay-audit",
            &replay_audit_s,
        ]);
        assert_ok(&run, "reactor run");
        assert!(report.exists(), "missing runtime report");
        digests.push(read_dispatch_digest(&report));
    }

    let first = digests.first().expect("at least one digest");
    assert!(
        digests.iter().all(|d| d == first),
        "reactor dispatch digest drift detected: {digests:?}"
    );
}
