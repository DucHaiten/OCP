use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use ocp_sdk::{
    dap_breakpoint_mapping_v19, dap_launch_summary_v19, dap_step_sequence_v19,
    dap_trace_events_from_source_v19, dap_variables_for_event_v19, EditorDapBreakpointMapEntryV19,
    EditorDapBreakpointV19, EditorDapTraceEventV19,
};
use serde_json::{json, Value as JsonValue};

const SERVER_NAME: &str = "ocp-dap";
const SERVER_VERSION: &str = "1.0.0";
const DEBUG_CONTRACT_JSON: &str = r#"{
  "debug_mode": "dap_replay_backed",
  "trace_acquisition": "generate_on_launch",
  "trace_artifact_path_scheme": ".ocp_artifacts/editor_dbg/<workspace_hash>/<run_id>/",
  "breakpoint_mapping": "(file,span)->breakpoint_id->trace_event_id[]",
  "step_semantics": "trace_event_id_ascending"
}"#;

#[derive(Default)]
struct DapState {
    source_path: Option<String>,
    workspace_folder: Option<String>,
    workspace_hash: String,
    run_id: String,
    events: Vec<EditorDapTraceEventV19>,
    step_ids: Vec<u64>,
    current_step: usize,
    breakpoints: Vec<EditorDapBreakpointV19>,
    breakpoint_map: Vec<EditorDapBreakpointMapEntryV19>,
    trace_path: Option<String>,
    terminated: bool,
}

struct DapWriter<W: Write> {
    writer: W,
    next_seq: i64,
}

impl<W: Write> DapWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            next_seq: 1,
        }
    }

    fn response(
        &mut self,
        request_seq: i64,
        command: &str,
        success: bool,
        body: JsonValue,
        message: Option<String>,
    ) -> io::Result<()> {
        let payload = json!({
            "seq": self.take_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": success,
            "command": command,
            "message": message,
            "body": body
        });
        write_message(&mut self.writer, &payload)
    }

    fn event(&mut self, event: &str, body: JsonValue) -> io::Result<()> {
        let payload = json!({
            "seq": self.take_seq(),
            "type": "event",
            "event": event,
            "body": body
        });
        write_message(&mut self.writer, &payload)
    }

    fn take_seq(&mut self) -> i64 {
        let current = self.next_seq;
        self.next_seq += 1;
        current
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("{SERVER_NAME} v{SERVER_VERSION}");
        return;
    }
    if let Err(err) = run() {
        let _ = writeln!(io::stderr(), "{SERVER_NAME}: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = DapWriter::new(stdout.lock());
    let mut state = DapState::default();

    while let Some(message) = read_message(&mut reader)? {
        let seq = message.get("seq").and_then(JsonValue::as_i64).unwrap_or(0);
        let command = message
            .get("command")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string();
        let arguments = message.get("arguments").cloned().unwrap_or(JsonValue::Null);

        match command.as_str() {
            "initialize" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({
                        "supportsConfigurationDoneRequest": true,
                        "supportsTerminateRequest": true,
                        "supportsSetVariable": false,
                        "supportsStepBack": false,
                        "supportsRestartRequest": false
                    }),
                    None,
                )?;
            }
            "launch" => match handle_launch(&mut state, &arguments) {
                Ok(()) => {
                    writer.response(seq, &command, true, json!({}), None)?;
                    writer.event("initialized", json!({}))?;
                }
                Err(message) => {
                    writer.response(seq, &command, false, json!({}), Some(message))?;
                }
            },
            "setBreakpoints" => {
                let breakpoints = handle_set_breakpoints(&mut state, &arguments);
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({ "breakpoints": breakpoints }),
                    None,
                )?;
            }
            "configurationDone" => {
                writer.response(seq, &command, true, json!({}), None)?;
                writer.event(
                    "stopped",
                    json!({
                        "reason": "entry",
                        "threadId": 1,
                        "allThreadsStopped": true
                    }),
                )?;
            }
            "threads" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({
                        "threads": [
                            { "id": 1, "name": "OCP Replay Thread" }
                        ]
                    }),
                    None,
                )?;
            }
            "stackTrace" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({
                        "stackFrames": current_stack_frames(&state),
                        "totalFrames": 1
                    }),
                    None,
                )?;
            }
            "scopes" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({
                        "scopes": [
                            {
                                "name": "Locals",
                                "presentationHint": "locals",
                                "variablesReference": 1,
                                "expensive": false
                            }
                        ]
                    }),
                    None,
                )?;
            }
            "variables" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({
                        "variables": current_variables(&state)
                    }),
                    None,
                )?;
            }
            "next" => {
                writer.response(seq, &command, true, json!({}), None)?;
                if advance_one_step(&mut state) {
                    writer.event(
                        "stopped",
                        json!({
                            "reason": "step",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }),
                    )?;
                } else {
                    state.terminated = true;
                    writer.event("terminated", json!({ "restart": false }))?;
                }
            }
            "continue" => {
                writer.response(
                    seq,
                    &command,
                    true,
                    json!({ "allThreadsContinued": true }),
                    None,
                )?;
                if advance_to_breakpoint_or_end(&mut state) {
                    writer.event(
                        "stopped",
                        json!({
                            "reason": "breakpoint",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }),
                    )?;
                } else {
                    state.terminated = true;
                    writer.event("terminated", json!({ "restart": false }))?;
                }
            }
            "disconnect" => {
                state.terminated = true;
                writer.response(seq, &command, true, json!({}), None)?;
                break;
            }
            _ => {
                writer.response(
                    seq,
                    &command,
                    false,
                    json!({}),
                    Some(format!("unsupported command: {command}")),
                )?;
            }
        }
    }

    Ok(())
}

fn handle_launch(state: &mut DapState, arguments: &JsonValue) -> Result<(), String> {
    let program = arguments
        .get("program")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "launch arguments.program is required".to_string())?
        .to_string();
    let workspace_folder = arguments
        .get("workspaceFolder")
        .and_then(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            Path::new(&program)
                .parent()
                .map(|path| path.display().to_string())
        });
    let workspace_hash = arguments
        .get("workspaceHash")
        .and_then(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            stable_workspace_hash(workspace_folder.as_deref().unwrap_or("workspace"))
        });
    let run_id = arguments
        .get("runId")
        .and_then(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "editor-run".to_string());
    let file_id = stable_file_id(&program);
    let source =
        std::fs::read_to_string(&program).map_err(|err| format!("read program failed: {err}"))?;
    let contract = serde_json::from_str::<JsonValue>(DEBUG_CONTRACT_JSON)
        .map_err(|err| format!("parse debug contract failed: {err}"))?;
    let summary = dap_launch_summary_v19(&contract, &workspace_hash, &run_id, &source, file_id)
        .map_err(|err| format!("compute launch summary failed: {err}"))?;
    let events = dap_trace_events_from_source_v19(&source, file_id)
        .map_err(|err| format!("build trace events failed: {err:?}"))?;
    let trace_path = if let Some(root) = &workspace_folder {
        Some(write_trace_artifacts(
            root,
            &summary.trace_path,
            &summary,
            &events,
        )?)
    } else {
        None
    };

    state.source_path = Some(program);
    state.workspace_folder = workspace_folder;
    state.workspace_hash = workspace_hash;
    state.run_id = run_id;
    state.step_ids = dap_step_sequence_v19(&events);
    state.current_step = 0;
    state.events = events;
    state.breakpoints.clear();
    state.breakpoint_map.clear();
    state.trace_path = trace_path.or_else(|| Some(summary.trace_path));
    state.terminated = false;

    Ok(())
}

fn handle_set_breakpoints(state: &mut DapState, arguments: &JsonValue) -> Vec<JsonValue> {
    let Some(source_path) = arguments
        .get("source")
        .and_then(|value| value.get("path"))
        .and_then(JsonValue::as_str)
    else {
        return Vec::new();
    };
    let lines = arguments
        .get("lines")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let file_id = stable_file_id(source_path);
    let requested = lines
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let line = value.as_u64()? as u32;
            Some(EditorDapBreakpointV19 {
                breakpoint_id: index as u64 + 1,
                file_id,
                line,
                column: 1,
            })
        })
        .collect::<Vec<EditorDapBreakpointV19>>();
    let contract =
        serde_json::from_str::<JsonValue>(DEBUG_CONTRACT_JSON).unwrap_or_else(|_| json!({}));
    let mapping =
        dap_breakpoint_mapping_v19(&contract, &state.events, &requested).unwrap_or_default();
    state.breakpoints = requested.clone();
    state.breakpoint_map = mapping;

    requested
        .into_iter()
        .map(|bp| {
            json!({
                "id": bp.breakpoint_id,
                "verified": true,
                "line": bp.line,
                "column": bp.column
            })
        })
        .collect()
}

fn current_stack_frames(state: &DapState) -> Vec<JsonValue> {
    let Some(event) = current_event(state) else {
        return Vec::new();
    };
    let Some(path) = &state.source_path else {
        return Vec::new();
    };
    vec![json!({
        "id": 1,
        "name": format!("{}#{}", event.kind, event.trace_event_id),
        "source": {
            "name": Path::new(path).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| "main.ocp".to_string()),
            "path": path
        },
        "line": event.line,
        "column": event.column
    })]
}

fn current_variables(state: &DapState) -> Vec<JsonValue> {
    let Some(event) = current_event(state) else {
        return Vec::new();
    };
    dap_variables_for_event_v19(&state.events, event.trace_event_id)
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            json!({
                "name": item.name,
                "value": item.value,
                "variablesReference": 0
            })
        })
        .collect()
}

fn current_event(state: &DapState) -> Option<&EditorDapTraceEventV19> {
    let event_id = *state.step_ids.get(state.current_step)?;
    state
        .events
        .iter()
        .find(|event| event.trace_event_id == event_id)
}

fn advance_one_step(state: &mut DapState) -> bool {
    if state.current_step + 1 < state.step_ids.len() {
        state.current_step += 1;
        true
    } else {
        false
    }
}

fn advance_to_breakpoint_or_end(state: &mut DapState) -> bool {
    let mut hits = BTreeMap::<u64, bool>::new();
    for entry in &state.breakpoint_map {
        for trace_id in &entry.trace_event_ids {
            hits.insert(*trace_id, true);
        }
    }
    let mut cursor = state.current_step;
    while cursor + 1 < state.step_ids.len() {
        cursor += 1;
        if hits.contains_key(&state.step_ids[cursor]) {
            state.current_step = cursor;
            return true;
        }
    }
    false
}

fn write_trace_artifacts(
    workspace_folder: &str,
    relative_trace_path: &str,
    summary: &ocp_sdk::EditorDapLaunchSummaryV19,
    events: &[EditorDapTraceEventV19],
) -> Result<String, String> {
    let trace_root = Path::new(workspace_folder).join(relative_trace_path);
    std::fs::create_dir_all(&trace_root)
        .map_err(|err| format!("create trace root failed: {err}"))?;
    let summary_path = trace_root.join("launch_summary.json");
    let events_path = trace_root.join("trace_events.json");
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(summary).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("write launch summary failed: {err}"))?;
    std::fs::write(
        &events_path,
        serde_json::to_string_pretty(events).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("write trace events failed: {err}"))?;
    Ok(trace_root.display().to_string())
}

fn stable_workspace_hash(input: &str) -> String {
    let mut hash = 0u64;
    for byte in input.as_bytes() {
        hash = hash.wrapping_mul(131).wrapping_add(*byte as u64);
    }
    format!("ws{:x}", hash)
}

fn stable_file_id(input: &str) -> u32 {
    let mut hash = 2166136261u32;
    for byte in input.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn read_message(reader: &mut dyn BufRead) -> io::Result<Option<JsonValue>> {
    let mut content_length = None::<usize>;
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        if line.trim().is_empty() {
            break;
        }
        if let Some(value) = line.trim().strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let Some(length) = content_length else {
        return Ok(None);
    };
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let message = serde_json::from_slice::<JsonValue>(&body)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(Some(message))
}

fn write_message(writer: &mut dyn Write, payload: &JsonValue) -> io::Result<()> {
    let body = serde_json::to_vec(payload)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    writer.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    writer.write_all(&body)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::{advance_one_step, stable_workspace_hash, DapState};

    #[test]
    fn workspace_hash_is_stable() {
        assert_eq!(
            stable_workspace_hash("E:/demo"),
            stable_workspace_hash("E:/demo")
        );
    }

    #[test]
    fn next_step_stops_at_end() {
        let mut state = DapState {
            step_ids: vec![1, 2],
            ..DapState::default()
        };
        assert!(advance_one_step(&mut state));
        assert!(!advance_one_step(&mut state));
    }
}
