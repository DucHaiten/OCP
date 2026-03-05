use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[path = "v16_gate_a_common.rs"]
mod v16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RequiredContractsV1 {
    schema: String,
    entries: Vec<v16::ContractInventoryEntryV16>,
}

fn required_path() -> PathBuf {
    v16::repo_root()
        .join("contracts")
        .join("required_contracts.v1.json")
}

fn contract_dir() -> PathBuf {
    v16::w16_target_root().join("contracts")
}

#[test]
fn contract_inventory_has_full_required_coverage() {
    let inventory = v16::current_contract_inventory_v16();
    let required_path = required_path();

    if std::env::var("OCL_UPDATE_V16_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1")
    {
        if let Some(parent) = required_path.parent() {
            fs::create_dir_all(parent).expect("create required contracts dir");
        }
        let required = RequiredContractsV1 {
            schema: "ocl.contracts.required.v1".to_string(),
            entries: inventory.clone(),
        };
        let rendered = serde_json::to_string_pretty(&required).expect("render required contracts");
        fs::write(&required_path, rendered).expect("write required contracts");
        panic!(
            "updated {}. Re-run without OCL_UPDATE_V16_REQUIRED_CONTRACTS=1",
            required_path.display()
        );
    }

    if !required_path.exists() {
        if let Some(parent) = required_path.parent() {
            fs::create_dir_all(parent).expect("create required contracts dir");
        }
        let required = RequiredContractsV1 {
            schema: "ocl.contracts.required.v1".to_string(),
            entries: inventory.clone(),
        };
        let rendered = serde_json::to_string_pretty(&required).expect("render required contracts");
        fs::write(&required_path, rendered).expect("write required contracts");
        panic!(
            "initialized missing required contracts {}. Re-run test",
            required_path.display()
        );
    }

    let required_raw = fs::read_to_string(&required_path)
        .unwrap_or_else(|_| panic!("read required contracts file: {}", required_path.display()));
    let required: RequiredContractsV1 =
        serde_json::from_str(&required_raw).expect("parse required contracts");
    assert_eq!(required.schema, "ocl.contracts.required.v1");

    let mut actual_map = BTreeMap::<String, v16::ContractInventoryEntryV16>::new();
    for entry in &inventory {
        assert!(
            actual_map
                .insert(entry.contract_id.clone(), entry.clone())
                .is_none(),
            "duplicate contract_id in current inventory: {}",
            entry.contract_id
        );
    }

    let mut required_map = BTreeMap::<String, v16::ContractInventoryEntryV16>::new();
    for entry in &required.entries {
        assert!(
            required_map
                .insert(entry.contract_id.clone(), entry.clone())
                .is_none(),
            "duplicate contract_id in required set: {}",
            entry.contract_id
        );
    }

    for (contract_id, expected) in &required_map {
        let actual = actual_map
            .get(contract_id)
            .unwrap_or_else(|| panic!("missing required contract_id `{contract_id}`"));
        assert_eq!(
            actual.version, expected.version,
            "version mismatch for contract_id `{contract_id}`"
        );
        assert_eq!(
            actual.schema_hash, expected.schema_hash,
            "schema_hash mismatch for contract_id `{contract_id}`"
        );
    }

    let inventory_value = serde_json::to_value(&inventory).expect("inventory value");
    let inventory_hash = v16::sha256_hex_text(&v16::canonical_json_string(&inventory_value));
    let report = json!({
        "schema": "ocl.contract.inventory.completeness.report.v1",
        "status": "PASS",
        "required_count": required_map.len(),
        "actual_count": actual_map.len(),
        "inventory_hash_sha256": inventory_hash,
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
    });
    let out_dir = contract_dir();
    v16::write_json_pretty(&out_dir.join("contract_inventory.json"), &inventory_value);
    v16::write_json_pretty(
        &out_dir.join("contract_inventory_completeness_report.json"),
        &report,
    );
}
