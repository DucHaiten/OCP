use std::fs;

use serde_json::json;

#[path = "v18_gate_a_common.rs"]
mod v18;

fn out_dir() -> std::path::PathBuf {
    v18::w18_target_root().join("contracts")
}

#[test]
fn v18_contract_inventory_complete() {
    v18::ensure_run_manifest();
    let required_path = v18::required_contracts_path();
    let required_raw = fs::read_to_string(&required_path)
        .unwrap_or_else(|_| panic!("read {}", required_path.display()));
    let mut required: v18::RequiredContractsV1 =
        serde_json::from_str(&required_raw).expect("parse required contracts");

    let mut actual_entries = Vec::<v18::ContractInventoryEntryV18>::new();
    let mut has_auto_placeholder = false;
    for item in &required.entries {
        let computed_hash = v18::producer_schema_hash(&v18::repo_root(), &item.producer);
        if item.schema_hash == "__AUTO__" {
            has_auto_placeholder = true;
        }
        actual_entries.push(v18::ContractInventoryEntryV18 {
            contract_id: item.contract_id.clone(),
            version: item.version.clone(),
            schema_hash: computed_hash,
            producer: item.producer.clone(),
            consumer: item.consumer.clone(),
            allowed_diffs_v018: item.allowed_diffs_v018.clone(),
        });
    }

    let update = std::env::var("OCP_UPDATE_V18_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if update || has_auto_placeholder {
        required.entries = actual_entries.clone();
        let required_value = serde_json::to_value(&required).expect("required to json");
        v18::write_json_pretty(&required_path, &required_value);
        v18::write_json_pretty(&v18::required_contracts_mirror_path(), &required_value);
        panic!(
            "updated {} and mirror with computed schema_hash. Re-run test",
            required_path.display()
        );
    }

    assert_eq!(
        required.entries.len(),
        actual_entries.len(),
        "required and actual entry counts must match"
    );
    for expected in &required.entries {
        let actual = actual_entries
            .iter()
            .find(|item| item.contract_id == expected.contract_id)
            .unwrap_or_else(|| panic!("missing contract_id {}", expected.contract_id));
        assert_eq!(
            expected.version, actual.version,
            "version mismatch for {}",
            expected.contract_id
        );
        assert_eq!(
            expected.schema_hash, actual.schema_hash,
            "schema_hash mismatch for {}",
            expected.contract_id
        );
        assert_eq!(
            expected.producer, actual.producer,
            "producer mismatch for {}",
            expected.contract_id
        );
    }

    let inventory_value = serde_json::to_value(&actual_entries).expect("inventory to json");
    let inventory_hash =
        v18::sha256_hex_bytes(v18::canonical_json_string(&inventory_value).as_bytes());
    let report = json!({
        "schema": "ocp.w18.contract_inventory_completeness_report.v1",
        "status": "PASS",
        "required_count": required.entries.len(),
        "actual_count": actual_entries.len(),
        "inventory_hash_sha256": inventory_hash,
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json"
    });

    v18::write_json_pretty(&out_dir().join("contract_inventory.json"), &inventory_value);
    v18::write_json_pretty(
        &out_dir().join("contract_inventory_completeness_report.json"),
        &report,
    );
}
