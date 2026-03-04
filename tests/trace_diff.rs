use std::fs;
use std::path::PathBuf;
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
    std::env::temp_dir().join(format!("ocl_trace_diff_v11_{tag}_{stamp}"))
}

fn run_ocl_cli(args: &[&str]) -> Output {
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

fn write_program_start(path: &PathBuf) {
    let content = "{\"t\":\"ProgramStart\",\"i\":0,\"tick\":0,\"seed\":1,\"call_id\":null,\"span\":null,\"data\":{\"trace_schema_version\":2,\"lane\":\"locked_v071\"}}\n";
    fs::write(path, content).expect("write ProgramStart");
}

fn append_line(path: &PathBuf, line: &str) {
    let mut current = fs::read_to_string(path).expect("read audit");
    current.push_str(line);
    current.push('\n');
    fs::write(path, current).expect("append audit line");
}

#[allow(clippy::too_many_arguments)]
fn trace_event_line(
    i: u64,
    seq: u64,
    event: &str,
    key: Option<&str>,
    kind: Option<&str>,
    reason: Option<&str>,
    call_id: Option<u64>,
    payload_hash: &str,
) -> String {
    format!(
        concat!(
            "{{\"t\":\"TraceEvent\",\"i\":{i},\"tick\":{i},\"seed\":1,\"call_id\":{call_id},\"span\":null,",
            "\"data\":{{\"seq\":{seq},\"run_id\":\"demo\",\"event\":\"{event}\",\"key\":{key},",
            "\"callsite_package_id\":null,\"kind\":{kind},\"reason\":{reason},\"origin_id\":null,",
            "\"allowed\":null,\"value\":null,\"steps\":1,\"universe_id\":\"u\",\"domain_id\":\"d\",\"payload_hash\":\"{payload_hash}\"}}}}"
        ),
        i = i,
        seq = seq,
        event = event,
        key = key
            .map(|v| format!("\"{v}\""))
            .unwrap_or_else(|| "null".to_string()),
        kind = kind
            .map(|v| format!("\"{v}\""))
            .unwrap_or_else(|| "null".to_string()),
        reason = reason
            .map(|v| format!("\"{v}\""))
            .unwrap_or_else(|| "null".to_string()),
        call_id = call_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()),
        payload_hash = payload_hash,
    )
}

#[test]
fn trace_diff_strict_reports_first_divergence_and_changed_key() {
    let root = temp_project_dir("strict");
    let left_dir = root.join("left");
    let right_dir = root.join("right");
    let out_dir = root.join("out");
    fs::create_dir_all(&left_dir).expect("mkdir left");
    fs::create_dir_all(&right_dir).expect("mkdir right");
    fs::create_dir_all(&out_dir).expect("mkdir out");

    let left_audit = left_dir.join("audit.jsonl");
    let right_audit = right_dir.join("audit.jsonl");
    write_program_start(&left_audit);
    write_program_start(&right_audit);

    append_line(
        &left_audit,
        &trace_event_line(
            1,
            1,
            "observe_end",
            Some("std.fs.read_text"),
            Some("ok"),
            None,
            Some(7),
            "h-left-1",
        ),
    );
    append_line(
        &left_audit,
        &trace_event_line(2, 2, "stmt_exec", None, Some("ok"), None, None, "h-left-2"),
    );

    append_line(
        &right_audit,
        &trace_event_line(
            1,
            1,
            "observe_end",
            Some("std.fs.read_text"),
            Some("insufficient"),
            Some("RC-FS-NOT-FOUND"),
            Some(7),
            "h-right-1",
        ),
    );
    append_line(
        &right_audit,
        &trace_event_line(2, 2, "stmt_exec", None, Some("ok"), None, None, "h-left-2"),
    );

    let output = run_ocl_cli(&[
        "trace",
        "diff",
        &left_dir.to_string_lossy(),
        &right_dir.to_string_lossy(),
        "--mode",
        "strict",
        "--out",
        &out_dir.to_string_lossy(),
    ]);
    assert!(
        output.status.success(),
        "trace diff failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered = String::from_utf8_lossy(&output.stdout);
    assert!(
        rendered.contains("mode=strict"),
        "missing strict mode output"
    );
    assert!(
        rendered.contains("first_divergence_index=0"),
        "strict diff must detect first divergence at first event"
    );
    assert!(
        rendered.contains("changed_outcomes=1"),
        "strict diff must count changed outcomes"
    );
    assert!(
        rendered.contains("std.fs.read_text"),
        "strict diff must report changed key call"
    );
    assert!(
        out_dir.join("diff_report.txt").exists(),
        "missing diff_report.txt"
    );
    assert!(
        out_dir.join("diff_report.json").exists(),
        "missing diff_report.json"
    );
}

#[test]
fn trace_diff_align_matches_by_call_id_instead_of_position() {
    let root = temp_project_dir("align");
    let left_dir = root.join("left");
    let right_dir = root.join("right");
    fs::create_dir_all(&left_dir).expect("mkdir left");
    fs::create_dir_all(&right_dir).expect("mkdir right");

    let left_audit = left_dir.join("audit.jsonl");
    let right_audit = right_dir.join("audit.jsonl");
    write_program_start(&left_audit);
    write_program_start(&right_audit);

    append_line(
        &left_audit,
        &trace_event_line(
            1,
            1,
            "observe_end",
            Some("std.kv.get"),
            Some("ok"),
            None,
            Some(11),
            "a1",
        ),
    );
    append_line(
        &left_audit,
        &trace_event_line(
            2,
            2,
            "observe_end",
            Some("std.kv.put"),
            Some("ok"),
            None,
            Some(12),
            "a2",
        ),
    );

    append_line(
        &right_audit,
        &trace_event_line(
            1,
            1,
            "observe_end",
            Some("std.kv.put"),
            Some("insufficient"),
            Some("RC-KV-NOT-FOUND"),
            Some(12),
            "b1",
        ),
    );
    append_line(
        &right_audit,
        &trace_event_line(
            2,
            2,
            "observe_end",
            Some("std.kv.get"),
            Some("ok"),
            None,
            Some(11),
            "b2",
        ),
    );

    let output = run_ocl_cli(&[
        "trace",
        "diff",
        &left_dir.to_string_lossy(),
        &right_dir.to_string_lossy(),
        "--mode",
        "align",
    ]);
    assert!(
        output.status.success(),
        "trace diff align failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = String::from_utf8_lossy(&output.stdout);
    assert!(rendered.contains("mode=align"), "missing align mode output");
    assert!(
        rendered.contains("changed_outcomes=1"),
        "align diff must find changed outcome for call_id=12"
    );
    assert!(
        rendered.contains("std.kv.put"),
        "align diff must report changed key for matched call_id"
    );
}
