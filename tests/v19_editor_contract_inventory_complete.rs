use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

#[test]
fn v19_editor_contract_inventory_complete() {
    v19::ensure_run_manifest();

    let required_path = v19::required_editor_contracts_path();
    let mut required = v19::parse_required_editor_contracts(&required_path);

    let mut computed = Vec::<v19::ContractInventoryEntryV19>::new();
    let mut has_auto = false;
    for entry in &required.entries {
        let schema_hash = v19::producer_schema_hash(&v19::repo_root(), &entry.producer);
        if entry.schema_hash == "__AUTO__" {
            has_auto = true;
        }
        computed.push(v19::ContractInventoryEntryV19 {
            allowed_diffs_v018: entry.allowed_diffs_v018.clone(),
            contract_id: entry.contract_id.clone(),
            version: entry.version.clone(),
            producer: entry.producer.clone(),
            consumer: entry.consumer.clone(),
            schema_hash,
        });
    }

    let update = std::env::var("OCL_UPDATE_V19_EDITOR_REQUIRED_CONTRACTS")
        .ok()
        .as_deref()
        == Some("1");
    if update || has_auto {
        required.entries = computed.clone();
        let out = serde_json::to_value(&required).expect("required to value");
        v19::write_json_pretty(&required_path, &out);
        panic!(
            "updated {} with computed schema hashes. Re-run test",
            required_path.display()
        );
    }

    let mut mismatch = false;
    if required.entries.len() != computed.len() {
        mismatch = true;
    } else {
        for expected in &required.entries {
            let actual = computed
                .iter()
                .find(|item| item.contract_id == expected.contract_id)
                .unwrap_or_else(|| panic!("missing contract_id {}", expected.contract_id));
            if expected.version != actual.version
                || expected.producer != actual.producer
                || expected.schema_hash != actual.schema_hash
            {
                mismatch = true;
                break;
            }
        }
    }

    if mismatch {
        required.entries = computed.clone();
        let out = serde_json::to_value(&required).expect("required to value");
        v19::write_json_pretty(&required_path, &out);
        panic!(
            "updated {} after detecting inventory mismatch. Re-run test",
            required_path.display()
        );
    }

    let inventory_value = serde_json::to_value(&computed).expect("inventory value");
    let inventory_hash =
        v19::sha256_hex_bytes(v19::canonical_json_string(&inventory_value).as_bytes());
    let report = json!({
        "schema": "ocl.w19.editor_contract_inventory_report.v1",
        "status": "PASS",
        "required_count": required.entries.len(),
        "inventory_hash_sha256": inventory_hash,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("contracts/editor_contract_inventory_report.json", &report);
}
