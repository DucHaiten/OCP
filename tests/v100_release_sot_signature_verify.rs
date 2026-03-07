use std::collections::BTreeSet;

use ocl_sdk::{sign_contract_json_v20, verify_contract_json_signature_v20};
use serde_json::json;

#[path = "v100_gate_a_common.rs"]
mod v100;

#[test]
fn v100_release_sot_signature_verify() {
    v100::ensure_run_manifest();

    let required = v100::parse_release_required_contracts(&v100::release_required_contracts_path());
    let mut files = BTreeSet::<String>::new();
    files.insert("contracts/release/v1.0/release_required_contracts.v1.json".to_string());
    for entry in required.entries {
        files.insert(entry.producer);
    }

    let update = std::env::var("OCL_UPDATE_V100_SOT_SIG").ok().as_deref() == Some("1");
    let mut generated = Vec::<String>::new();
    let mut invalid = Vec::<String>::new();
    for rel in &files {
        let contract_path = v100::repo_root().join(rel);
        let sig_path = ocl_sdk::contract_sig_path_v20(&contract_path);
        if update {
            sign_contract_json_v20(&contract_path, "w18-sot-root", 1)
                .unwrap_or_else(|_| panic!("sign {}", contract_path.display()));
            generated.push(sig_path.to_string_lossy().replace('\\', "/"));
            continue;
        }
        if !sig_path.exists() {
            invalid.push(format!(
                "missing signature: {}",
                sig_path.to_string_lossy().replace('\\', "/")
            ));
            continue;
        }
    }

    if !generated.is_empty() {
        panic!(
            "generated {} signature file(s). Re-run test",
            generated.len(),
        );
    }

    let mut items = Vec::<serde_json::Value>::new();
    for rel in &files {
        let contract_path = v100::repo_root().join(rel);
        let summary = match verify_contract_json_signature_v20(&v100::repo_root(), &contract_path) {
            Ok(summary) => summary,
            Err(err) => {
                invalid.push(format!(
                    "invalid signature: {} ({err})",
                    contract_path.to_string_lossy().replace('\\', "/")
                ));
                continue;
            }
        };
        assert!(
            summary.signature_verified,
            "signature must verify for {}",
            contract_path.display()
        );
        items.push(json!({
            "contract_id": summary.contract_id,
            "version": summary.version,
            "contract_path": summary.contract_path.to_string_lossy().replace('\\', "/"),
            "signature_path": summary.signature_path.to_string_lossy().replace('\\', "/"),
            "contract_hash_sha256": summary.contract_hash_sha256,
            "signature_verified": summary.signature_verified,
            "signer_id": summary.signer_id,
            "trust_epoch": summary.trust_epoch
        }));
    }

    if !invalid.is_empty() {
        panic!(
            "signature verification failed for {} contract(s): {}",
            invalid.len(),
            invalid.join("; ")
        );
    }

    let report = json!({
        "schema": "ocl.w100.release_sot_signature_report.v1",
        "status": "PASS",
        "required_count": files.len(),
        "verified_count": items.len(),
        "items": items,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100::run_manifest_sha256()
    });
    v100::write_report("contracts/release_sot_signature_report.json", &report);
}
