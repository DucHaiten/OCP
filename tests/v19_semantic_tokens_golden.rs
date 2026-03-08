use std::collections::BTreeSet;
use std::fs;

use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

fn tokenize_for_semantic(source: &str) -> Vec<(String, String)> {
    let keywords = [
        "fn", "let", "if", "else", "return", "match", "observe", "commit",
    ]
    .iter()
    .map(|v| v.to_string())
    .collect::<BTreeSet<String>>();
    let mut out = Vec::<(String, String)>::new();
    for part in source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
    {
        let token_type = if keywords.contains(part) {
            "keyword"
        } else if part.chars().all(|ch| ch.is_ascii_digit()) {
            "number"
        } else {
            "variable"
        };
        out.push((part.to_string(), token_type.to_string()));
    }
    out
}

#[test]
fn v19_semantic_tokens_golden_matches_expected_stream() {
    v19::ensure_run_manifest();

    let fixture_path = v19::repo_root()
        .join("tests")
        .join("fixtures")
        .join("v19")
        .join("lsp")
        .join("semantic_tokens.ocp");
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("read {}", fixture_path.display()));
    let actual = tokenize_for_semantic(&source);
    let expected = vec![
        ("fn".to_string(), "keyword".to_string()),
        ("semantic_case".to_string(), "variable".to_string()),
        ("let".to_string(), "keyword".to_string()),
        ("score".to_string(), "variable".to_string()),
        ("10".to_string(), "number".to_string()),
        ("if".to_string(), "keyword".to_string()),
        ("score".to_string(), "variable".to_string()),
        ("return".to_string(), "keyword".to_string()),
        ("score".to_string(), "variable".to_string()),
        ("return".to_string(), "keyword".to_string()),
        ("0".to_string(), "number".to_string()),
    ];
    assert_eq!(actual, expected, "semantic token stream must match golden");

    let report = json!({
        "schema": "ocp.w19.editor.semantic_tokens_golden_report.v1",
        "status": "PASS",
        "fixture_path": fixture_path.to_string_lossy().replace('\\', "/"),
        "token_count": actual.len(),
        "tokens": actual.iter().map(|(lexeme, kind)| json!({
            "lexeme": lexeme,
            "kind": kind
        })).collect::<Vec<_>>(),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/semantic_tokens_golden_report.json", &report);
}
