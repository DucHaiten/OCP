use std::fs;

use ocl_sdk::{sign_contract_json_v18, verify_w18_contract_set_v18, W18_REQUIRED_CONTRACT_FILES};
use serde_json::json;

#[path = "v18_gate_a_common.rs"]
mod v18;

fn contracts_root() -> std::path::PathBuf {
    v18::repo_root().join("contracts")
}

#[test]
fn v18_sot_signature_verify() {
    v18::ensure_run_manifest();
    let update = std::env::var("OCL_UPDATE_V18_SOT_SIG").ok().as_deref() == Some("1");
    let mut generated = Vec::<String>::new();
    for rel in W18_REQUIRED_CONTRACT_FILES {
        let contract_path = contracts_root().join(rel);
        let sig_path = ocl_sdk::contract_sig_path_v18(&contract_path);
        if update || !sig_path.exists() {
            sign_contract_json_v18(&contract_path, "w18-sot-root", 1)
                .unwrap_or_else(|_| panic!("sign {}", contract_path.display()));
            generated.push(sig_path.to_string_lossy().replace('\\', "/"));
        }
    }
    if !generated.is_empty() {
        panic!(
            "generated {} signature file(s). Re-run test",
            generated.len()
        );
    }

    let summary = verify_w18_contract_set_v18(&v18::repo_root()).expect("verify w18 contract set");
    assert_eq!(summary.required_count, W18_REQUIRED_CONTRACT_FILES.len());
    assert_eq!(summary.verified_count, W18_REQUIRED_CONTRACT_FILES.len());
    assert!(
        summary.items.iter().all(|item| item.signature_verified),
        "all contracts must verify"
    );

    let report = json!({
        "schema": "ocl.w18.sot_signature_report.v1",
        "run_manifest_ref": "target/ocl/w18/meta/run_manifest.json",
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
        })).collect::<Vec<_>>()
    });

    let out_dir = v18::w18_target_root().join("contracts");
    fs::create_dir_all(&out_dir).expect("create w18 contracts output dir");
    v18::write_json_pretty(&out_dir.join("sot_signature_report.json"), &report);
}
