use std::fs;

use ocp_sdk::{
    sign_contract_json_v20, verify_contract_json_signature_v20, verify_w20_contract_set_v20,
    W20_REQUIRED_CONTRACT_FILES,
};
use serde_json::json;

#[path = "v20_gate_a_common.rs"]
mod v20;

#[test]
fn v20_contract_chain() {
    v20::ensure_run_manifest();

    let update = std::env::var("OCP_UPDATE_V20_SOT_SIG").ok().as_deref() == Some("1");
    let mut generated = Vec::<String>::new();
    let mut resigned = Vec::<String>::new();
    for rel in W20_REQUIRED_CONTRACT_FILES {
        let contract_path = v20::contracts_root().join(rel);
        let sig_path = ocp_sdk::contract_sig_path_v20(&contract_path);
        if update || !sig_path.exists() {
            sign_contract_json_v20(&contract_path, "w18-sot-root", 1)
                .unwrap_or_else(|_| panic!("sign {}", contract_path.display()));
            generated.push(sig_path.to_string_lossy().replace('\\', "/"));
        }
        if verify_contract_json_signature_v20(&v20::repo_root(), &contract_path).is_err() {
            sign_contract_json_v20(&contract_path, "w18-sot-root", 1)
                .unwrap_or_else(|_| panic!("re-sign {}", contract_path.display()));
            resigned.push(sig_path.to_string_lossy().replace('\\', "/"));
        }
    }

    if !generated.is_empty() || !resigned.is_empty() {
        panic!(
            "generated {} and re-signed {} signature file(s). Re-run test",
            generated.len(),
            resigned.len()
        );
    }

    let summary = verify_w20_contract_set_v20(&v20::repo_root()).expect("verify w20 contracts");
    assert_eq!(summary.required_count, W20_REQUIRED_CONTRACT_FILES.len());
    assert_eq!(summary.verified_count, W20_REQUIRED_CONTRACT_FILES.len());
    assert!(
        summary.items.iter().all(|item| item.signature_verified),
        "all v20 contracts must verify"
    );

    let report = json!({
        "schema": "ocp.w20.contract_chain_report.v1",
        "status": "PASS",
        "required_count": summary.required_count,
        "verified_count": summary.verified_count,
        "contracts_root": summary.root.to_string_lossy().replace('\\', "/"),
        "items": summary.items.iter().map(|item| json!({
            "contract_id": item.contract_id,
            "version": item.version,
            "contract_path": item.contract_path.to_string_lossy().replace('\\', "/"),
            "signature_path": item.signature_path.to_string_lossy().replace('\\', "/"),
            "contract_hash_sha256": item.contract_hash_sha256,
            "signature_verified": item.signature_verified,
            "signer_id": item.signer_id,
            "trust_epoch": item.trust_epoch
        })).collect::<Vec<_>>(),
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20::run_manifest_sha256()
    });
    let out_dir = v20::w20_target_root().join("contracts");
    fs::create_dir_all(&out_dir).expect("create w20 contracts output");
    v20::write_json_pretty(&out_dir.join("contract_chain_report.json"), &report);
}
