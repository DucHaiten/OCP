use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_dbg_v11_{tag}_{stamp}"))
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
    let artifacts_root = project_root.join(".ocp_artifacts");
    let mut dirs = list_dirs(&artifacts_root);
    assert!(!dirs.is_empty(), "missing artifact dir");
    dirs.pop().expect("latest artifact dir")
}

#[test]
fn dbg_script_supports_step_back_break_continue_and_locals() {
    let root = temp_project_dir("script_core");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let run = run_ocp_cli(&["run", &root_s]);
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifact = latest_artifact_dir(&root);
    let replay = run_ocp_cli(&["replay", &artifact.to_string_lossy()]);
    assert!(
        replay.status.success(),
        "replay failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&replay.stdout),
        String::from_utf8_lossy(&replay.stderr)
    );

    let script_path = root.join("dbg.script.txt");
    fs::write(
        &script_path,
        concat!(
            "where\n",
            "step\n",
            "back\n",
            "break on type observe_end\n",
            "continue\n",
            "locals\n",
            "print event.key\n",
            "diffenv\n",
            "last\n"
        ),
    )
    .expect("write script");

    let dbg = run_ocp_cli(&[
        "dbg",
        &artifact.to_string_lossy(),
        "--script",
        &script_path.to_string_lossy(),
    ]);
    assert!(
        dbg.status.success(),
        "dbg failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&dbg.stdout),
        String::from_utf8_lossy(&dbg.stderr)
    );
    let rendered = String::from_utf8_lossy(&dbg.stdout);
    assert!(rendered.contains("dbg v11"), "missing debugger header");
    assert!(
        rendered.contains("cmd[1]=where"),
        "missing scripted command trace"
    );
    assert!(rendered.contains("step cursor="), "missing step output");
    assert!(rendered.contains("back cursor="), "missing back output");
    assert!(
        rendered.contains("continue hit") || rendered.contains("continue reached_end"),
        "missing continue result"
    );
    assert!(
        rendered.contains("locals checkpoint_event_i="),
        "missing locals output"
    );
    assert!(rendered.contains("diffenv left="), "missing diffenv output");
    assert!(rendered.contains("last cursor="), "missing last output");
}

#[test]
fn dbg_jump_first_error_targets_error_event() {
    let artifact = temp_project_dir("jump_first_error");
    fs::create_dir_all(&artifact).expect("create artifact");
    let audit_path = artifact.join("audit.jsonl");
    let audit = concat!(
        "{\"t\":\"ProgramStart\",\"i\":0,\"tick\":0,\"seed\":1,\"call_id\":null,\"span\":null,\"data\":{\"trace_schema_version\":2,\"lane\":\"locked_v071\"}}\n",
        "{\"t\":\"TraceEvent\",\"i\":1,\"tick\":1,\"seed\":1,\"call_id\":null,\"span\":null,\"data\":{\"seq\":1,\"run_id\":\"demo\",\"event\":\"stmt_exec\",\"key\":null,\"callsite_package_id\":null,\"kind\":\"ok\",\"reason\":null,\"origin_id\":null,\"allowed\":null,\"value\":null,\"steps\":1,\"universe_id\":\"u\",\"domain_id\":\"d\",\"payload_hash\":\"abc\"}}\n",
        "{\"t\":\"Error\",\"i\":2,\"tick\":2,\"seed\":1,\"call_id\":null,\"span\":null,\"data\":{\"code\":\"X-DEMO\",\"phase\":\"exec\",\"message\":\"demo error\",\"hint\":null,\"root_reason\":\"RC-DEMO\"}}\n"
    );
    fs::write(&audit_path, audit).expect("write audit");

    let script_path = artifact.join("dbg.script.txt");
    fs::write(
        &script_path,
        concat!(
            "jump first_error\n",
            "where\n",
            "last\n",
            "print event.type\n"
        ),
    )
    .expect("write script");

    let dbg = run_ocp_cli(&[
        "dbg",
        &artifact.to_string_lossy(),
        "--script",
        &script_path.to_string_lossy(),
    ]);
    assert!(
        dbg.status.success(),
        "dbg failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&dbg.stdout),
        String::from_utf8_lossy(&dbg.stderr)
    );
    let rendered = String::from_utf8_lossy(&dbg.stdout);
    assert!(
        rendered.contains("jump first_error cursor="),
        "first_error jump did not land on an error event"
    );
    assert!(
        rendered.contains("event=error"),
        "debugger output must point to the error event"
    );
    assert!(
        rendered.contains("print event.type => error"),
        "print command must expose current error event"
    );
}
