use serde_json::json;

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_release_required_contracts() {
    v100::ensure_run_manifest();

    let required_path = v100::release_required_contracts_path();
    let mut required = v100::parse_release_required_contracts(&required_path);
    let mut expected_producers = v100::canonical_contract_producers_v100();
    expected_producers.retain(|item| item != "contracts/release/v1.0/release_required_contracts.v1.json");
    expected_producers.push("contracts/sot_aliases.v1.json".to_string());
    expected_producers.sort();

    let mut required_producers = required
        .entries
        .iter()
        .map(|entry| entry.producer.clone())
        .collect::<Vec<String>>();
    required_producers.sort();
    assert_eq!(
        required_producers, expected_producers,
        "release_required_contracts producers must exactly match canonical roots + sot_aliases"
    );

    let mut computed = Vec::<v100::ContractInventoryEntryV100>::new();
    let mut has_auto = false;
    for entry in &required.entries {
        if entry.schema_hash == "__AUTO__" {
            has_auto = true;
        }
        computed.push(v100::ContractInventoryEntryV100 {
            contract_id: entry.contract_id.clone(),
            version: entry.version.clone(),
            producer: entry.producer.clone(),
            consumer: entry.consumer.clone(),
            schema_hash: v100::producer_schema_hash(&v100::repo_root(), &entry.producer),
            allowed_diffs_v100: entry.allowed_diffs_v100.clone(),
        });
    }

    let update = std::env::var("OCL_UPDATE_V100_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if update || has_auto {
        required.entries = computed.clone();
        let out = serde_json::to_value(&required).expect("required to value");
        v100::write_json_pretty(&required_path, &out);
        v100::write_json_pretty(&v100::release_required_contracts_mirror_path(), &out);
        panic!(
            "updated {} and mirror with computed schema hashes. Re-run test",
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
        assert!(
            expected.producer.starts_with("contracts/release/v1.0/")
                || expected.producer.starts_with("contracts/docs/v1.0/")
                || expected.producer.starts_with("contracts/legal/v1.0/")
                || expected.producer.starts_with("contracts/business/v1.0/")
                || expected.producer.starts_with("contracts/security/v1.0/")
                || expected.producer == "contracts/sot_aliases.v1.json",
            "producer `{}` must be in canonical roots",
            expected.producer
        );
    }

    let inventory_value = serde_json::to_value(&computed).expect("inventory value");
    let inventory_hash =
        v100::sha256_hex_bytes(v100::canonical_json_string(&inventory_value).as_bytes());
    let report = json!({
        "schema": "ocl.w100.release_required_contracts_report.v1",
        "status": "PASS",
        "required_count": required.entries.len(),
        "inventory_hash_sha256": inventory_hash,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/release_required_contracts_report.json", &report);
    v100::write_report("contracts/contract_inventory.json", &inventory_value);
}
