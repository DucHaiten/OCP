use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_required_contracts_complete() {
    v20::ensure_run_manifest();

    let required_path = v20::required_contracts_v20_path();
    let mut required = v20::parse_required_contracts_v20(&required_path);

    let mut computed = Vec::<v20::ContractInventoryEntryV20>::new();
    let mut has_auto = false;
    for entry in &required.entries {
        let schema_hash = v20::producer_schema_hash(&v20::repo_root(), &entry.producer);
        if entry.schema_hash == "__AUTO__" {
            has_auto = true;
        }
        computed.push(v20::ContractInventoryEntryV20 {
            contract_id: entry.contract_id.clone(),
            version: entry.version.clone(),
            producer: entry.producer.clone(),
            consumer: entry.consumer.clone(),
            schema_hash,
            allowed_diffs_v020: entry.allowed_diffs_v020.clone(),
        });
    }

    let update = std::env::var("OCP_UPDATE_V20_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if update || has_auto {
        required.entries = computed.clone();
        let out = serde_json::to_value(&required).expect("required to value");
        v20::write_json_pretty(&required_path, &out);
        panic!(
            "updated {} with computed schema hashes. Re-run test",
            required_path.display()
        );
    }

    assert_eq!(
        required.entries.len(),
        computed.len(),
        "required and computed entries must have same size"
    );
    for expected in &required.entries {
        let actual = computed
            .iter()
            .find(|item| item.contract_id == expected.contract_id)
            .unwrap_or_else(|| panic!("missing contract_id {}", expected.contract_id));
        assert_eq!(
            expected.version, actual.version,
            "version mismatch for {}",
            expected.contract_id
        );
        assert_eq!(
            expected.producer, actual.producer,
            "producer mismatch for {}",
            expected.contract_id
        );
        assert_eq!(
            expected.schema_hash, actual.schema_hash,
            "schema_hash mismatch for {}",
            expected.contract_id
        );
    }

    let global_path = v20::required_global_contracts_path();
    let mirror_path = v20::required_global_contracts_mirror_path();
    let global_required = v20::parse_required_global_contracts(&global_path);
    let mut missing_global = Vec::<String>::new();
    for item in &required.entries {
        let found = global_required.entries.iter().any(|g| {
            g.contract_id == item.contract_id
                && g.version == item.version
                && g.producer == item.producer
                && g.schema_hash == item.schema_hash
        });
        if !found {
            missing_global.push(item.contract_id.clone());
        }
    }
    let sync_global = std::env::var("OCP_UPDATE_V20_GLOBAL_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if sync_global || !missing_global.is_empty() {
        let merged = v20::merge_v20_into_global(&global_required, &required.entries);
        let value = serde_json::to_value(&merged).expect("global required to value");
        v20::write_json_pretty(&global_path, &value);
        v20::write_json_pretty(&mirror_path, &value);
        panic!(
            "updated {} and mirror with v20 entries. Re-run test",
            global_path.display()
        );
    }

    let inventory_value = serde_json::to_value(&computed).expect("inventory value");
    let inventory_hash =
        v20::sha256_hex_bytes(v20::canonical_json_string(&inventory_value).as_bytes());
    let report = json!({
        "schema": "ocp.w20.required_contracts_v20_report.v1",
        "status": "PASS",
        "required_count": required.entries.len(),
        "inventory_hash_sha256": inventory_hash,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    v20::write_report("contracts/required_contracts_v20_report.json", &report);
    v20::write_report("contracts/contract_inventory.json", &inventory_value);
}
