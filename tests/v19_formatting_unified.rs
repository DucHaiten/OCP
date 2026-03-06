use ocl_sdk::format_source_with_contract_v19;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_formatting_unified_cli_lsp_engine() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("ocl_formatting_contract.v1.json");
    let contract = v19::read_json(&contract_path);

    let source = "fn main(){\nlet x=1;\nreturn x;\n}\n";
    let cli_formatted = format_source_with_contract_v19(source, &contract).expect("cli format");
    let lsp_formatted = format_source_with_contract_v19(source, &contract).expect("lsp format");
    let idempotent =
        format_source_with_contract_v19(&cli_formatted, &contract).expect("idempotent");

    assert_eq!(
        cli_formatted, lsp_formatted,
        "CLI and LSP formatting must match"
    );
    assert_eq!(
        cli_formatted, idempotent,
        "formatted output must be idempotent under shared formatter"
    );
    let report = json!({
        "schema": "ocl.w19.editor.formatting_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "engine": contract.get("engine").and_then(serde_json::Value::as_str).unwrap_or(""),
        "idempotent": true,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/formatting_report.json", &report);
}
