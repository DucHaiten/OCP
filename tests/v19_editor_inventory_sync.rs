use std::collections::BTreeMap;

use ocp_sdk::sign_contract_json_v18;
use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_inventory_sync() {
    v19::ensure_run_manifest();

    let editor_path = v19::required_editor_contracts_path();
    let global_path = v19::required_global_contracts_path();
    let mirror_path = v19::required_global_contracts_mirror_path();

    let editor_required = v19::parse_required_editor_contracts(&editor_path);
    let mut global_required = v19::parse_required_global_contracts(&global_path);

    let mut global_by_id = BTreeMap::<String, v19::ContractInventoryEntryV19>::new();
    for entry in &global_required.entries {
        global_by_id.insert(entry.contract_id.clone(), entry.clone());
    }

    let mut missing = Vec::<String>::new();
    let mut mismatch = Vec::<serde_json::Value>::new();
    for editor_entry in &editor_required.entries {
        match global_by_id.get(&editor_entry.contract_id) {
            Some(global_entry) => {
                if global_entry.version != editor_entry.version
                    || global_entry.producer != editor_entry.producer
                    || global_entry.schema_hash != editor_entry.schema_hash
                {
                    mismatch.push(json!({
                        "contract_id": editor_entry.contract_id,
                        "global": global_entry,
                        "editor": editor_entry
                    }));
                }
            }
            None => missing.push(editor_entry.contract_id.clone()),
        }
    }

    let update = std::env::var("OCP_UPDATE_V19_GLOBAL_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if update || !missing.is_empty() || !mismatch.is_empty() {
        global_required =
            v19::merge_editor_entries_into_global(&global_required, &editor_required.entries);
        let value = serde_json::to_value(&global_required).expect("global required to json");
        v19::write_json_pretty(&global_path, &value);
        v19::write_json_pretty(&mirror_path, &value);
        panic!(
            "updated {} and mirror with synced editor entries. Re-run test",
            global_path.display()
        );
    }

    assert!(
        missing.is_empty(),
        "missing editor entries in global required: {}",
        missing.join(", ")
    );
    assert!(
        mismatch.is_empty(),
        "mismatched entries found between editor and global required"
    );

    let sig_path = sign_contract_json_v18(&global_path, "w18-sot-root", 1)
        .unwrap_or_else(|_| panic!("sign {}", global_path.display()));

    let report = json!({
        "schema": "ocp.w19.editor_inventory_sync_report.v1",
        "status": "PASS",
        "editor_required_count": editor_required.entries.len(),
        "global_required_count": global_required.entries.len(),
        "global_required_signature_path": sig_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("contracts/editor_inventory_sync_report.json", &report);
}
