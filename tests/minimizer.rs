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
    std::env::temp_dir().join(format!("ocl_minimizer_v11_{tag}_{stamp}"))
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

fn parse_metric(output: &str, key: &str) -> Option<usize> {
    for line in output.lines() {
        if let Some(raw) = line.strip_prefix(&format!("{key}=")) {
            if let Ok(value) = raw.trim().parse::<usize>() {
                return Some(value);
            }
        }
    }
    None
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

fn error_line(i: u64, code: &str, root_reason: &str) -> String {
    format!(
        concat!(
            "{{\"t\":\"Error\",\"i\":{i},\"tick\":{i},\"seed\":1,\"call_id\":null,\"span\":null,",
            "\"data\":{{\"code\":\"{code}\",\"phase\":\"exec\",\"message\":\"demo\",",
            "\"hint\":null,\"root_reason\":\"{root_reason}\"}}}}"
        ),
        i = i,
        code = code,
        root_reason = root_reason
    )
}

#[test]
fn minimize_error_code_reduces_trace_and_cassette() {
    let root = temp_project_dir("error_code");
    let source = root.join("source");
    let output = root.join("minimized");
    fs::create_dir_all(&source).expect("mkdir source");
    let audit = source.join("audit.jsonl");
    write_program_start(&audit);

    append_line(
        &audit,
        &trace_event_line(1, 1, "stmt_exec", None, Some("ok"), None, None, "h1"),
    );
    append_line(&audit, &error_line(2, "X-READ", "RC-FS-NOT-FOUND"));
    append_line(
        &audit,
        &trace_event_line(
            3,
            3,
            "observe_end",
            Some("std.fs.read_text"),
            Some("ok"),
            None,
            Some(1),
            "h3",
        ),
    );
    append_line(
        &audit,
        &trace_event_line(4, 4, "stmt_exec", None, Some("ok"), None, None, "h4"),
    );
    append_line(
        &audit,
        &trace_event_line(
            5,
            5,
            "observe_end",
            Some("std.kv.get"),
            Some("ok"),
            None,
            Some(2),
            "h5",
        ),
    );
    append_line(
        &audit,
        &trace_event_line(6, 6, "stmt_exec", None, Some("ok"), None, None, "h6"),
    );

    let cassette_dir = source.join("cassette");
    fs::create_dir_all(&cassette_dir).expect("mkdir cassette");
    fs::write(
        cassette_dir.join("cassette_index.json"),
        concat!(
            "{\n",
            "  \"schema_version\": \"v0.8\",\n",
            "  \"mode\": \"record\",\n",
            "  \"call_id_to_entry_id\": {\n",
            "    \"1\": \"entry-1\",\n",
            "    \"2\": \"entry-2\"\n",
            "  },\n",
            "  \"entries\": 2,\n",
            "  \"next_call_id\": 2\n",
            "}\n"
        ),
    )
    .expect("write cassette index");
    fs::write(
        cassette_dir.join("cassette.jsonl"),
        concat!(
            "{\"id\":\"entry-1\",\"cap\":\"std.fs.read_text\"}\n",
            "{\"id\":\"entry-2\",\"cap\":\"std.kv.get\"}\n"
        ),
    )
    .expect("write cassette jsonl");

    let cmd = run_ocl_cli(&[
        "minimize",
        &source.to_string_lossy(),
        "--goal",
        "error_code:X-READ",
        "--out",
        &output.to_string_lossy(),
    ]);
    assert!(
        cmd.status.success(),
        "minimize failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&cmd.stdout),
        String::from_utf8_lossy(&cmd.stderr)
    );
    let rendered = String::from_utf8_lossy(&cmd.stdout);
    assert!(
        rendered.contains("event_count_before=6"),
        "must report original event count"
    );
    let event_after = parse_metric(&rendered, "event_count_after").expect("event_count_after");
    assert!(
        (1..6).contains(&event_after),
        "must reduce trace but preserve goal event"
    );
    assert!(
        rendered.contains("cassette_entries_after=0"),
        "must drop cassette entries not referenced by kept call_id set; output={rendered}"
    );

    let minimized_audit = fs::read_to_string(output.join("audit.jsonl")).expect("read minimized");
    assert!(
        minimized_audit.contains("\"code\":\"X-READ\""),
        "minimized audit must preserve goal error"
    );
    assert_eq!(
        minimized_audit
            .lines()
            .filter(|line| !line.is_empty())
            .count(),
        event_after + 1,
        "expected ProgramStart + minimized event count"
    );
}

#[test]
fn minimize_divergence_keeps_prefix_to_first_divergence() {
    let root = temp_project_dir("divergence");
    let source = root.join("source");
    let against = root.join("against");
    let output = root.join("minimized");
    fs::create_dir_all(&source).expect("mkdir source");
    fs::create_dir_all(&against).expect("mkdir against");

    let source_audit = source.join("audit.jsonl");
    let against_audit = against.join("audit.jsonl");
    write_program_start(&source_audit);
    write_program_start(&against_audit);

    append_line(
        &source_audit,
        &trace_event_line(1, 1, "stmt_exec", None, Some("ok"), None, None, "same-1"),
    );
    append_line(
        &source_audit,
        &trace_event_line(
            2,
            2,
            "observe_end",
            Some("std.kv.get"),
            Some("ok"),
            None,
            Some(11),
            "same-2",
        ),
    );
    append_line(
        &source_audit,
        &trace_event_line(
            3,
            3,
            "observe_end",
            Some("std.kv.put"),
            Some("ok"),
            None,
            Some(12),
            "s3",
        ),
    );
    append_line(
        &source_audit,
        &trace_event_line(4, 4, "stmt_exec", None, Some("ok"), None, None, "same-4"),
    );

    append_line(
        &against_audit,
        &trace_event_line(1, 1, "stmt_exec", None, Some("ok"), None, None, "same-1"),
    );
    append_line(
        &against_audit,
        &trace_event_line(
            2,
            2,
            "observe_end",
            Some("std.kv.get"),
            Some("ok"),
            None,
            Some(11),
            "same-2",
        ),
    );
    append_line(
        &against_audit,
        &trace_event_line(
            3,
            3,
            "observe_end",
            Some("std.kv.put"),
            Some("insufficient"),
            Some("RC-KV-NOT-FOUND"),
            Some(12),
            "a3",
        ),
    );
    append_line(
        &against_audit,
        &trace_event_line(4, 4, "stmt_exec", None, Some("ok"), None, None, "same-4"),
    );

    let cmd = run_ocl_cli(&[
        "minimize",
        &source.to_string_lossy(),
        "--goal",
        "divergence",
        "--against",
        &against.to_string_lossy(),
        "--out",
        &output.to_string_lossy(),
    ]);
    assert!(
        cmd.status.success(),
        "minimize divergence failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&cmd.stdout),
        String::from_utf8_lossy(&cmd.stderr)
    );
    let rendered = String::from_utf8_lossy(&cmd.stdout);
    assert!(
        rendered.contains("event_count_before=4"),
        "must report original event count"
    );
    let event_after = parse_metric(&rendered, "event_count_after").expect("event_count_after");
    assert!(
        (1..4).contains(&event_after),
        "must reduce trace while preserving divergence"
    );

    let minimized_audit = fs::read_to_string(output.join("audit.jsonl")).expect("read minimized");
    assert_eq!(
        minimized_audit
            .lines()
            .filter(|line| !line.is_empty())
            .count(),
        event_after + 1,
        "expected ProgramStart + minimized event count"
    );
}

#[test]
fn minimize_reduces_fixture_manifest_by_trace_references() {
    let root = temp_project_dir("fixtures");
    let source = root.join("source");
    let output = root.join("minimized");
    fs::create_dir_all(source.join("io")).expect("mkdir io");

    let audit = source.join("audit.jsonl");
    write_program_start(&audit);
    append_line(
        &audit,
        &trace_event_line(
            1,
            1,
            "observe_end",
            Some("std.fs.read_text"),
            Some("ok"),
            None,
            Some(1),
            "hp1",
        )
        .replace(
            "\"payload_hash\":\"hp1\"",
            "\"payload_hash\":\"hp1\",\"path\":\"fixtures/in/keep.json\"",
        ),
    );
    append_line(&audit, &error_line(2, "X-READ", "RC-FS-NOT-FOUND"));

    fs::write(
        source.join("io").join("fixtures_manifest.json"),
        concat!(
            "[\n",
            "  {\"path\":\"fixtures/in/keep.json\",\"sha256\":\"a\"},\n",
            "  {\"path\":\"fixtures/in/drop.json\",\"sha256\":\"b\"}\n",
            "]\n"
        ),
    )
    .expect("write fixture manifest");

    let cmd = run_ocl_cli(&[
        "minimize",
        &source.to_string_lossy(),
        "--goal",
        "kind:ok",
        "--key",
        "std.fs.read_text",
        "--out",
        &output.to_string_lossy(),
    ]);
    assert!(
        cmd.status.success(),
        "minimize fixture reduction failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&cmd.stdout),
        String::from_utf8_lossy(&cmd.stderr)
    );

    let rendered = String::from_utf8_lossy(&cmd.stdout);
    assert!(
        rendered.contains("fixture_entries_before=2"),
        "must report fixture entry count before reduction"
    );
    assert!(
        rendered.contains("fixture_entries_after=1"),
        "must keep only referenced fixture entries; output={rendered}"
    );
    let minimized_manifest = fs::read_to_string(output.join("io").join("fixtures_manifest.json"))
        .expect("read manifest");
    assert!(
        minimized_manifest.contains("fixtures/in/keep.json")
            && !minimized_manifest.contains("fixtures/in/drop.json"),
        "manifest must keep referenced fixture only"
    );
}
