use ocl_sdk::{cli_bridge_contract_allows_v19, cli_bridge_output_policy_v19};
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_cli_bridge_contract_is_machine_readable() {
    v19::ensure_run_manifest();

    let contract_path = v19::contracts_root()
        .join("editor")
        .join("editor_cli_bridge.v1.json");
    let contract = v19::read_json(&contract_path);

    assert!(
        cli_bridge_contract_allows_v19("ocl fmt", &contract),
        "ocl fmt must be allowed"
    );
    assert!(
        cli_bridge_contract_allows_v19("ocl perm fix --plan", &contract),
        "ocl perm fix --plan must be allowed"
    );
    assert!(
        !cli_bridge_contract_allows_v19("ocl perm fix --all", &contract),
        "unknown bridge command must be denied"
    );

    let (output_mode, text_fallback_allowed) = cli_bridge_output_policy_v19(&contract);
    assert_eq!(output_mode, "json_only");
    assert!(!text_fallback_allowed, "text fallback must stay disabled");

    let report = json!({
        "schema": "ocl.w19.editor.cli_bridge_report.v1",
        "status": "PASS",
        "contract_path": contract_path.to_string_lossy().replace('\\', "/"),
        "output_mode": output_mode,
        "text_fallback_allowed": text_fallback_allowed,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("editor/cli_bridge_report.json", &report);
}
