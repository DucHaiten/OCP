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
    std::env::temp_dir().join(format!("ocl_cli_trace_view_v11_{tag}_{stamp}"))
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
    let artifacts_root = project_root.join(".ocl_artifacts");
    let mut dirs = list_dirs(&artifacts_root);
    assert!(!dirs.is_empty(), "missing artifact dir");
    dirs.pop().expect("latest artifact dir")
}

fn output_stdout_text(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn trace_event_lines(text: &str) -> Vec<&str> {
    text.lines().filter(|line| line.starts_with('#')).collect()
}

fn extract_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("{key}=");
    let start = line.find(&marker)?;
    let remain = &line[start + marker.len()..];
    let end = remain.find(' ').unwrap_or(remain.len());
    Some(&remain[..end])
}

#[test]
fn trace_view_json_summary_and_filters_work() {
    let root = temp_project_dir("summary_filters");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocl_cli(&["init", &root_s, "--template", "mini-game"]);
    assert!(
        init.status.success(),
        "init failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let run = run_ocl_cli(&["run", &root_s]);
    assert!(
        run.status.success(),
        "run failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let artifact = latest_artifact_dir(&root);
    let artifact_s = artifact.to_string_lossy().to_string();

    let view_all = run_ocl_cli(&["trace", "view", &artifact_s, "--json"]);
    let json_all = output_stdout_text(&view_all);
    assert!(
        json_all.contains("\"trace_schema_version\": 2"),
        "trace view json must expose schema version"
    );
    assert!(json_all.contains("\"summary\""), "missing summary section");
    assert!(json_all.contains("\"by_type\""), "missing summary.by_type");
    assert!(json_all.contains("\"by_kind\""), "missing summary.by_kind");
    assert!(
        json_all.contains("\"by_reason\""),
        "missing summary.by_reason"
    );

    let view_text = run_ocl_cli(&["trace", "view", &artifact_s]);
    let text_all = output_stdout_text(&view_text);
    let event_lines = trace_event_lines(&text_all);
    assert!(!event_lines.is_empty(), "trace view should return events");
    let selected_event_type = event_lines[0]
        .split_whitespace()
        .nth(1)
        .expect("event type token")
        .to_string();

    let view_type = run_ocl_cli(&["trace", "view", &artifact_s, "--type", &selected_event_type]);
    let text_type = output_stdout_text(&view_type);
    let events_type = trace_event_lines(&text_type);
    assert!(
        !events_type.is_empty(),
        "type filter should keep at least one event"
    );
    for event in events_type {
        assert_eq!(
            event.split_whitespace().nth(1),
            Some(selected_event_type.as_str()),
            "type filter must keep only selected event type"
        );
    }

    if let Some(selected_key) = event_lines
        .iter()
        .find_map(|line| extract_field(line, "key"))
        .filter(|key| *key != "-")
    {
        let view_key = run_ocl_cli(&["trace", "view", &artifact_s, "--key", selected_key]);
        let text_key = output_stdout_text(&view_key);
        let events_key = trace_event_lines(&text_key);
        assert!(!events_key.is_empty(), "key filter should keep events");
        for event in events_key {
            assert_eq!(
                extract_field(event, "key"),
                Some(selected_key),
                "key filter must keep only selected key"
            );
        }
    }
}

#[test]
fn trace_view_reads_span_snippet_and_module_filter() {
    let artifact = temp_project_dir("snippet_module");
    fs::create_dir_all(artifact.join("sources").join("src")).expect("create sources dir");

    let source_text = "let x = 1;\nlet y = x + 2;\n";
    fs::write(
        artifact.join("sources").join("src").join("main.ocl"),
        source_text,
    )
    .expect("write source");

    let program_start =
        "{\"t\":\"ProgramStart\",\"i\":0,\"tick\":0,\"seed\":0,\"call_id\":null,\"span\":null,\"data\":{\"trace_schema_version\":2,\"lane\":\"locked_v071\"}}";
    let trace_event = concat!(
        "{\"t\":\"TraceEvent\",\"i\":1,\"tick\":10,\"seed\":99,\"call_id\":null,",
        "\"span\":{\"module_id\":\"src/main.ocl\",\"start_byte\":0,\"end_byte\":10},",
        "\"data\":{",
        "\"seq\":1,",
        "\"run_id\":\"demo\",",
        "\"event\":\"stmt_exec\",",
        "\"key\":null,",
        "\"callsite_package_id\":null,",
        "\"kind\":\"ok\",",
        "\"reason\":null,",
        "\"origin_id\":null,",
        "\"allowed\":null,",
        "\"value\":null,",
        "\"steps\":1,",
        "\"universe_id\":\"u\",",
        "\"domain_id\":\"d\",",
        "\"payload_hash\":\"abc\"",
        "}}"
    );
    fs::write(
        artifact.join("audit.jsonl"),
        format!("{program_start}\n{trace_event}\n"),
    )
    .expect("write audit");

    let artifact_s = artifact.to_string_lossy().to_string();
    let view = run_ocl_cli(&["trace", "view", &artifact_s, "--module", "src/main.ocl"]);
    let rendered = output_stdout_text(&view);
    let events = trace_event_lines(&rendered);
    assert_eq!(
        events.len(),
        1,
        "module filter should keep exactly one event"
    );

    assert!(
        events[0].contains("snippet=let x = 1;"),
        "snippet must come from sources bundle bytes"
    );
}
