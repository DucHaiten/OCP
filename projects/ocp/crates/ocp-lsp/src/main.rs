use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, BufReader, Write};

use ocp_sdk::{
    format_source_with_contract_v19, lsp_completion_items_from_source_v19,
    lsp_definition_locations_v19, lsp_diagnostics_from_source_v19, lsp_hover_for_symbol_v19,
    lsp_reference_locations_v19, lsp_rename_preview_v19, lsp_symbols_from_source_v19,
};
use serde_json::{json, Value as JsonValue};

const SERVER_NAME: &str = "ocp-lsp";
const SERVER_VERSION: &str = "1.0.0";
const FORMATTING_CONTRACT_JSON: &str = r#"{
  "engine": "ocp_fmt_shared",
  "options": {
    "indent_size": 2,
    "line_width": 100
  }
}"#;
const SEMANTIC_TOKEN_TYPES: [&str; 22] = [
    "namespace",
    "type",
    "class",
    "enum",
    "interface",
    "struct",
    "typeParameter",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "event",
    "function",
    "method",
    "macro",
    "keyword",
    "modifier",
    "comment",
    "string",
    "number",
    "regexp",
    "operator",
];
const KEYWORDS: [&str; 15] = [
    "module", "import", "export", "fn", "let", "return", "match", "observe", "commit", "guard",
    "try", "else", "repeat", "for", "in",
];

#[derive(Default)]
struct LspState {
    docs: BTreeMap<String, String>,
    shutdown_requested: bool,
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
    let mut writer = stdout.lock();
    let mut state = LspState::default();

    while let Some(message) = read_message(&mut reader)? {
        let id = message.get("id").cloned();
        let method = message
            .get("method")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string();
        let params = message.get("params").cloned().unwrap_or(JsonValue::Null);

        if method.is_empty() {
            continue;
        }

        match method.as_str() {
            "initialize" => {
                send_result(&mut writer, id, initialize_result())?;
            }
            "initialized" => {}
            "shutdown" => {
                state.shutdown_requested = true;
                send_result(&mut writer, id, JsonValue::Null)?;
            }
            "exit" => {
                break;
            }
            "textDocument/didOpen" => {
                if let Some((uri, text)) = open_doc_params(&params) {
                    state.docs.insert(uri.clone(), text.clone());
                    publish_diagnostics(&mut writer, &uri, &text)?;
                }
            }
            "textDocument/didChange" => {
                if let Some((uri, text)) = change_doc_params(&params) {
                    state.docs.insert(uri.clone(), text.clone());
                    publish_diagnostics(&mut writer, &uri, &text)?;
                }
            }
            "textDocument/didSave" => {
                if let Some(uri) = save_doc_uri(&params) {
                    if let Some(text) = load_source(&state, &uri) {
                        state.docs.insert(uri.clone(), text.clone());
                        publish_diagnostics(&mut writer, &uri, &text)?;
                    }
                }
            }
            "workspace/didChangeConfiguration" | "$/cancelRequest" => {}
            "textDocument/hover" => {
                let result = hover_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/completion" => {
                let result = completion_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/definition" => {
                let result = definition_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/references" => {
                let result = references_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/rename" => {
                let result = rename_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/documentSymbol" => {
                let result = document_symbol_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "workspace/symbol" => {
                let result = workspace_symbol_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/formatting" => {
                let result = formatting_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            "textDocument/codeAction" => {
                let result = code_action_result();
                send_result(&mut writer, id, result)?;
            }
            "textDocument/semanticTokens/full" => {
                let result = semantic_tokens_result(&state, &params);
                send_result(&mut writer, id, result)?;
            }
            _ => {
                send_error(
                    &mut writer,
                    id,
                    -32601,
                    format!("method not implemented: {method}"),
                )?;
            }
        }
    }

    Ok(())
}

fn initialize_result() -> JsonValue {
    json!({
        "capabilities": {
            "textDocumentSync": {
                "openClose": true,
                "change": 1,
                "save": { "includeText": true }
            },
            "hoverProvider": true,
            "completionProvider": { "resolveProvider": false },
            "definitionProvider": true,
            "referencesProvider": true,
            "renameProvider": true,
            "documentSymbolProvider": true,
            "workspaceSymbolProvider": true,
            "documentFormattingProvider": true,
            "codeActionProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": SEMANTIC_TOKEN_TYPES,
                    "tokenModifiers": []
                },
                "full": true
            }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    })
}

fn hover_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some((uri, line, column)) = text_doc_position(params) else {
        return JsonValue::Null;
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Null;
    };
    let Some(symbol) = identifier_at_position(&source, line, column) else {
        return JsonValue::Null;
    };
    let Some(hover) = lsp_hover_for_symbol_v19(&source, file_id_from_uri(&uri), &symbol)
        .ok()
        .flatten()
    else {
        return JsonValue::Null;
    };
    json!({
        "contents": {
            "kind": "markdown",
            "value": format!("**{}**\n\n{}\n\n{}", hover.label, hover.kind, hover.detail)
        }
    })
}

fn completion_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some(uri) = text_doc_uri(params) else {
        return json!({ "isIncomplete": false, "items": [] });
    };
    let Some(source) = load_source(state, &uri) else {
        return json!({ "isIncomplete": false, "items": [] });
    };
    let items = lsp_completion_items_from_source_v19(&source, file_id_from_uri(&uri))
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            json!({
                "label": item.label,
                "kind": completion_kind(&item.kind)
            })
        })
        .collect::<Vec<JsonValue>>();
    json!({
        "isIncomplete": false,
        "items": items
    })
}

fn definition_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some((uri, line, column)) = text_doc_position(params) else {
        return JsonValue::Null;
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Null;
    };
    let Some(symbol) = identifier_at_position(&source, line, column) else {
        return JsonValue::Null;
    };
    let locations = lsp_definition_locations_v19(&source, file_id_from_uri(&uri), &symbol)
        .unwrap_or_default()
        .into_iter()
        .map(|loc| lsp_location(&uri, loc.line, loc.column))
        .collect::<Vec<JsonValue>>();
    JsonValue::Array(locations)
}

fn references_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some((uri, line, column)) = text_doc_position(params) else {
        return JsonValue::Array(Vec::new());
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Array(Vec::new());
    };
    let Some(symbol) = identifier_at_position(&source, line, column) else {
        return JsonValue::Array(Vec::new());
    };
    let locations = lsp_reference_locations_v19(&source, file_id_from_uri(&uri), &symbol)
        .unwrap_or_default()
        .into_iter()
        .map(|loc| lsp_location(&uri, loc.line, loc.column))
        .collect::<Vec<JsonValue>>();
    JsonValue::Array(locations)
}

fn rename_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some((uri, line, column)) = text_doc_position(params) else {
        return JsonValue::Null;
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Null;
    };
    let Some(symbol) = identifier_at_position(&source, line, column) else {
        return JsonValue::Null;
    };
    let Some(new_name) = params.get("newName").and_then(JsonValue::as_str) else {
        return JsonValue::Null;
    };
    let preview = lsp_rename_preview_v19(&source, file_id_from_uri(&uri), &symbol, new_name)
        .unwrap_or_else(|_| ocp_sdk::EditorRenamePreviewV19 {
            edits: Vec::new(),
            replaced_count: 0,
        });
    let edits = preview
        .edits
        .into_iter()
        .map(|edit| {
            json!({
                "range": {
                    "start": { "line": edit.line.saturating_sub(1), "character": edit.column.saturating_sub(1) },
                    "end": { "line": edit.line.saturating_sub(1), "character": edit.column.saturating_sub(1) + edit.old_name.len() as u32 }
                },
                "newText": edit.new_name
            })
        })
        .collect::<Vec<JsonValue>>();
    json!({
        "changes": {
            uri: edits
        }
    })
}

fn document_symbol_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some(uri) = text_doc_uri(params) else {
        return JsonValue::Array(Vec::new());
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Array(Vec::new());
    };
    let symbols = lsp_symbols_from_source_v19(&source, file_id_from_uri(&uri))
        .unwrap_or_default()
        .into_iter()
        .map(|symbol| {
            json!({
                "name": symbol.name,
                "kind": symbol_kind(&symbol.kind),
                "location": lsp_location(&uri, symbol.line, symbol.column)
            })
        })
        .collect::<Vec<JsonValue>>();
    JsonValue::Array(symbols)
}

fn workspace_symbol_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let query = params
        .get("query")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut out = Vec::<JsonValue>::new();
    for (uri, source) in &state.docs {
        let symbols =
            lsp_symbols_from_source_v19(source, file_id_from_uri(uri)).unwrap_or_default();
        for symbol in symbols {
            if !query.is_empty() && !symbol.name.to_ascii_lowercase().contains(&query) {
                continue;
            }
            out.push(json!({
                "name": symbol.name,
                "kind": symbol_kind(&symbol.kind),
                "location": lsp_location(uri, symbol.line, symbol.column)
            }));
        }
    }
    JsonValue::Array(out)
}

fn formatting_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some(uri) = text_doc_uri(params) else {
        return JsonValue::Array(Vec::new());
    };
    let Some(source) = load_source(state, &uri) else {
        return JsonValue::Array(Vec::new());
    };
    let contract =
        serde_json::from_str::<JsonValue>(FORMATTING_CONTRACT_JSON).unwrap_or_else(|_| json!({}));
    let Ok(formatted) = format_source_with_contract_v19(&source, &contract) else {
        return JsonValue::Array(Vec::new());
    };
    if formatted == source {
        return JsonValue::Array(Vec::new());
    }
    let last_line = source.lines().count().saturating_sub(1) as u32;
    let last_col = source.lines().last().map(|line| line.len()).unwrap_or(0) as u32;
    json!([
        {
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": last_line, "character": last_col }
            },
            "newText": formatted
        }
    ])
}

fn code_action_result() -> JsonValue {
    JsonValue::Array(vec![
        code_action(
            "Permission Fix Plan",
            "quickfix",
            "ocp.codeAction.permissionFixPlan",
            "ocp.openFixPlan",
        ),
        code_action(
            "Open Diff Report",
            "quickfix",
            "ocp.codeAction.openDiff",
            "ocp.openDoctorReport",
        ),
        code_action(
            "Apply Patch Guidance",
            "quickfix",
            "ocp.codeAction.applyPatch",
            "ocp.openFixPlan",
        ),
        code_action(
            "Open Budget Analyze",
            "quickfix",
            "ocp.codeAction.openBudgetAnalyze",
            "ocp.checkWorkspace",
        ),
        code_action(
            "Open Cassette Report",
            "quickfix",
            "ocp.codeAction.openCassetteReport",
            "ocp.openDebugTrace",
        ),
        code_action(
            "Open Contract Mismatch",
            "quickfix",
            "ocp.codeAction.openContractMismatch",
            "ocp.openDoctorReport",
        ),
    ])
}

fn semantic_tokens_result(state: &LspState, params: &JsonValue) -> JsonValue {
    let Some(uri) = text_doc_uri(params) else {
        return json!({ "data": [] });
    };
    let Some(source) = load_source(state, &uri) else {
        return json!({ "data": [] });
    };
    let data = semantic_token_data(&source);
    json!({ "data": data })
}

fn code_action(title: &str, kind: &str, action_id: &str, command_id: &str) -> JsonValue {
    json!({
        "title": format!("OCP: {title}"),
        "kind": kind,
        "data": { "id": action_id },
        "command": {
            "title": format!("OCP: {title}"),
            "command": command_id
        }
    })
}

fn publish_diagnostics(writer: &mut dyn Write, uri: &str, source: &str) -> io::Result<()> {
    let diagnostics = lsp_diagnostics_from_source_v19(source, file_id_from_uri(uri))
        .into_iter()
        .map(|diag| {
            json!({
                "range": {
                    "start": {
                        "line": diag.line.saturating_sub(1),
                        "character": diag.column.saturating_sub(1)
                    },
                    "end": {
                        "line": diag.line.saturating_sub(1),
                        "character": diag.column.saturating_sub(1) + 1
                    }
                },
                "severity": diagnostic_severity(&diag.severity),
                "code": diag.code,
                "message": diag.message,
                "source": SERVER_NAME
            })
        })
        .collect::<Vec<JsonValue>>();
    send_notification(
        writer,
        "textDocument/publishDiagnostics",
        json!({
            "uri": uri,
            "diagnostics": diagnostics
        }),
    )
}

fn open_doc_params(params: &JsonValue) -> Option<(String, String)> {
    let doc = params.get("textDocument")?;
    Some((
        doc.get("uri")?.as_str()?.to_string(),
        doc.get("text")?.as_str()?.to_string(),
    ))
}

fn change_doc_params(params: &JsonValue) -> Option<(String, String)> {
    let uri = params
        .get("textDocument")?
        .get("uri")?
        .as_str()?
        .to_string();
    let changes = params.get("contentChanges")?.as_array()?;
    let text = changes.last()?.get("text")?.as_str()?.to_string();
    Some((uri, text))
}

fn save_doc_uri(params: &JsonValue) -> Option<String> {
    params
        .get("textDocument")?
        .get("uri")?
        .as_str()
        .map(ToOwned::to_owned)
}

fn text_doc_uri(params: &JsonValue) -> Option<String> {
    params
        .get("textDocument")?
        .get("uri")?
        .as_str()
        .map(ToOwned::to_owned)
}

fn text_doc_position(params: &JsonValue) -> Option<(String, u32, u32)> {
    let uri = text_doc_uri(params)?;
    let position = params.get("position")?;
    Some((
        uri,
        position.get("line")?.as_u64()? as u32,
        position.get("character")?.as_u64()? as u32,
    ))
}

fn load_source(state: &LspState, uri: &str) -> Option<String> {
    if let Some(source) = state.docs.get(uri) {
        return Some(source.clone());
    }
    let path = uri_to_path(uri)?;
    std::fs::read_to_string(path).ok()
}

fn uri_to_path(uri: &str) -> Option<String> {
    let raw = uri.strip_prefix("file://")?;
    let decoded = percent_decode(raw);
    if decoded.len() >= 3 && decoded.as_bytes()[0] == b'/' && decoded.as_bytes()[2] == b':' {
        return Some(decoded[1..].replace('/', "\\"));
    }
    Some(decoded)
}

fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            let hi = (bytes[idx + 1] as char).to_digit(16);
            let lo = (bytes[idx + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push(char::from_u32((hi * 16 + lo) as u32).unwrap_or('%'));
                idx += 3;
                continue;
            }
        }
        out.push(bytes[idx] as char);
        idx += 1;
    }
    out
}

fn file_id_from_uri(uri: &str) -> u32 {
    let mut hash = 2166136261u32;
    for byte in uri.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn identifier_at_position(source: &str, line: u32, character: u32) -> Option<String> {
    let current_line = source.lines().nth(line as usize)?;
    let chars = current_line.chars().collect::<Vec<char>>();
    if chars.is_empty() {
        return None;
    }
    let mut idx = (character as usize).min(chars.len().saturating_sub(1));
    if !is_ident_char(chars[idx]) && idx > 0 && is_ident_char(chars[idx - 1]) {
        idx -= 1;
    }
    if !is_ident_char(chars[idx]) {
        return None;
    }
    let mut start = idx;
    while start > 0 && is_ident_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = idx + 1;
    while end < chars.len() && is_ident_char(chars[end]) {
        end += 1;
    }
    Some(chars[start..end].iter().collect())
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn lsp_location(uri: &str, line: u32, column: u32) -> JsonValue {
    json!({
        "uri": uri,
        "range": {
            "start": {
                "line": line.saturating_sub(1),
                "character": column.saturating_sub(1)
            },
            "end": {
                "line": line.saturating_sub(1),
                "character": column.saturating_sub(1) + 1
            }
        }
    })
}

fn completion_kind(kind: &str) -> u32 {
    match kind {
        "keyword" => 14,
        "function" => 3,
        "struct" | "type" | "enum" => 7,
        _ => 6,
    }
}

fn symbol_kind(kind: &str) -> u32 {
    match kind {
        "function" => 12,
        "struct" | "type" => 23,
        "enum" => 10,
        "variable" => 13,
        _ => 13,
    }
}

fn diagnostic_severity(severity: &str) -> u32 {
    match severity {
        "Error" => 1,
        "Warning" => 2,
        _ => 3,
    }
}

fn semantic_token_data(source: &str) -> Vec<u32> {
    let keyword_set = KEYWORDS
        .iter()
        .map(|item| item.to_string())
        .collect::<BTreeSet<String>>();
    let mut data = Vec::<u32>::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for (line_idx, raw_line) in source.lines().enumerate() {
        let chars = raw_line.chars().collect::<Vec<char>>();
        let mut idx = 0usize;
        while idx < chars.len() {
            if idx + 1 < chars.len() && chars[idx] == '/' && chars[idx + 1] == '/' {
                push_semantic_token(
                    &mut data,
                    line_idx as u32,
                    idx as u32,
                    (chars.len() - idx) as u32,
                    token_index("comment"),
                    &mut prev_line,
                    &mut prev_start,
                );
                break;
            }
            if chars[idx] == '"' {
                let start = idx;
                idx += 1;
                while idx < chars.len() && chars[idx] != '"' {
                    idx += 1;
                }
                if idx < chars.len() {
                    idx += 1;
                }
                push_semantic_token(
                    &mut data,
                    line_idx as u32,
                    start as u32,
                    (idx - start) as u32,
                    token_index("string"),
                    &mut prev_line,
                    &mut prev_start,
                );
                continue;
            }
            if chars[idx].is_ascii_digit() {
                let start = idx;
                idx += 1;
                while idx < chars.len() && chars[idx].is_ascii_digit() {
                    idx += 1;
                }
                push_semantic_token(
                    &mut data,
                    line_idx as u32,
                    start as u32,
                    (idx - start) as u32,
                    token_index("number"),
                    &mut prev_line,
                    &mut prev_start,
                );
                continue;
            }
            if is_ident_char(chars[idx]) {
                let start = idx;
                idx += 1;
                while idx < chars.len() && is_ident_char(chars[idx]) {
                    idx += 1;
                }
                let token = chars[start..idx].iter().collect::<String>();
                let token_type = if keyword_set.contains(&token) {
                    "keyword"
                } else {
                    "variable"
                };
                push_semantic_token(
                    &mut data,
                    line_idx as u32,
                    start as u32,
                    (idx - start) as u32,
                    token_index(token_type),
                    &mut prev_line,
                    &mut prev_start,
                );
                continue;
            }
            idx += 1;
        }
    }

    data
}

fn push_semantic_token(
    data: &mut Vec<u32>,
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    prev_line: &mut u32,
    prev_start: &mut u32,
) {
    let delta_line = line.saturating_sub(*prev_line);
    let delta_start = if delta_line == 0 {
        start.saturating_sub(*prev_start)
    } else {
        start
    };
    data.extend_from_slice(&[delta_line, delta_start, length, token_type, 0]);
    *prev_line = line;
    *prev_start = start;
}

fn token_index(name: &str) -> u32 {
    SEMANTIC_TOKEN_TYPES
        .iter()
        .position(|item| *item == name)
        .unwrap_or(8) as u32
}

fn read_message(reader: &mut dyn BufRead) -> io::Result<Option<JsonValue>> {
    let mut content_length = None::<usize>;
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
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

fn send_result(writer: &mut dyn Write, id: Option<JsonValue>, result: JsonValue) -> io::Result<()> {
    let response = json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(JsonValue::Null),
        "result": result
    });
    write_message(writer, &response)
}

fn send_error(
    writer: &mut dyn Write,
    id: Option<JsonValue>,
    code: i64,
    message: String,
) -> io::Result<()> {
    let response = json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(JsonValue::Null),
        "error": {
            "code": code,
            "message": message
        }
    });
    write_message(writer, &response)
}

fn send_notification(writer: &mut dyn Write, method: &str, params: JsonValue) -> io::Result<()> {
    let message = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });
    write_message(writer, &message)
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
    use super::{identifier_at_position, semantic_token_data};

    #[test]
    fn identifier_lookup_prefers_identifier_under_cursor() {
        let source = "fn helper() {\n  helper();\n}\n";
        assert_eq!(
            identifier_at_position(source, 1, 3).as_deref(),
            Some("helper")
        );
    }

    #[test]
    fn semantic_tokens_emit_non_empty_stream_for_keywords() {
        let source = "fn main() { let n = 1; }\n";
        let data = semantic_token_data(source);
        assert!(!data.is_empty(), "semantic token stream must not be empty");
    }
}
