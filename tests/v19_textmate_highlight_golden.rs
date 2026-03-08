use std::fs;

use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_textmate_highlight_golden_inputs_are_locked() {
    v19::ensure_run_manifest();

    let grammar_contract_path = v19::contracts_root()
        .join("editor")
        .join("ocp_textmate_grammar.v1.json");
    let grammar_contract = v19::read_json(&grammar_contract_path);
    let grammar_path = v19::repo_root().join(
        grammar_contract
            .get("grammar_path")
            .and_then(serde_json::Value::as_str)
            .expect("grammar_path"),
    );
    assert!(grammar_path.exists(), "grammar file must exist");

    let grammar_json = v19::read_json(&grammar_path);
    let scope_name = grammar_json
        .get("scopeName")
        .and_then(serde_json::Value::as_str)
        .expect("grammar scopeName");
    assert_eq!(scope_name, "source.ocp");

    let golden_path = v19::contracts_root()
        .join("editor")
        .join("golden_vectors.v1.json");
    let golden = v19::read_json(&golden_path);
    let highlight_cases = golden
        .get("highlight_cases")
        .and_then(serde_json::Value::as_array)
        .expect("highlight_cases array");

    let mut fixtures = Vec::<serde_json::Value>::new();
    for item in highlight_cases {
        let rel = item.as_str().expect("highlight case path");
        let full = v19::repo_root().join(rel);
        assert!(full.exists(), "missing highlight case {}", full.display());
        let bytes = fs::read(&full).unwrap_or_else(|_| panic!("read {}", full.display()));
        fixtures.push(json!({
            "path": rel.replace('\\', "/"),
            "sha256": v19::sha256_hex_bytes(&bytes)
        }));
    }

    let report = json!({
        "schema": "ocp.w19.editor.highlight_golden_report.v1",
        "status": "PASS",
        "grammar_path": grammar_path.to_string_lossy().replace('\\', "/"),
        "grammar_sha256": v19::sha256_hex_bytes(&fs::read(&grammar_path).expect("read grammar")),
        "fixtures": fixtures,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/highlight_golden_report.json", &report);
}
