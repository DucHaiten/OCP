use std::path::{Path, PathBuf};

use crate::w18;
use crate::SdkError;

pub const W20_HASHER_VERSION: &str = w18::W18_HASHER_VERSION;
pub const W20_SIGNATURE_SCHEMA_VERSION: &str = w18::W18_SIGNATURE_SCHEMA_VERSION;
pub const W20_SIGNATURE_SCHEMA: &str = w18::W18_SIGNATURE_SCHEMA;

pub const W20_REQUIRED_CONTRACT_FILES: [&str; 19] = [
    "v20/required_contracts_v20.v1.json",
    "v20/final_audit_scope.v1.json",
    "v20/threat_model.v1.json",
    "v20/test_catalog.v1.json",
    "v20/fuzz_seed_suite.v1.json",
    "v20/tooling_versions.v1.json",
    "v20/history_replay_matrix.v1.json",
    "v20/history_toolchain_matrix.v1.json",
    "v20/hardcore_test_protocol.v1.json",
    "v20/repro_protocol.v1.json",
    "v20/dx_friction_budget.v1.json",
    "v20/user_journey_matrix.v1.json",
    "v20/redteam_attack_matrix.v1.json",
    "v20/cve_snapshot.v1.json",
    "v20/finding_schema.v1.json",
    "v20/release_gate_v1_0.v1.json",
    "v20/zero_open_findings_policy.v1.json",
    "v20/signoff_required_artifacts.v1.json",
    "v20/stability_repeat_policy.v1.json",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractInspectSummaryV20 {
    pub contract_id: String,
    pub version: String,
    pub contract_path: PathBuf,
    pub signature_path: PathBuf,
    pub contract_hash_sha256: String,
    pub signature_verified: bool,
    pub signer_id: Option<String>,
    pub trust_epoch: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractSetSummaryV20 {
    pub root: PathBuf,
    pub required_count: usize,
    pub verified_count: usize,
    pub items: Vec<ContractInspectSummaryV20>,
}

pub fn contract_sig_path_v20(contract_path: &Path) -> PathBuf {
    w18::contract_sig_path_v18(contract_path)
}

pub fn sign_contract_json_v20(
    contract_path: &Path,
    pubkey_id: &str,
    trust_epoch: u64,
) -> Result<PathBuf, SdkError> {
    w18::sign_contract_json_v18(contract_path, pubkey_id, trust_epoch)
}

pub fn verify_contract_json_signature_v20(
    repo_root: &Path,
    contract_path: &Path,
) -> Result<ContractInspectSummaryV20, SdkError> {
    let item = w18::verify_contract_json_signature_v18(repo_root, contract_path)?;
    Ok(ContractInspectSummaryV20 {
        contract_id: item.contract_id,
        version: item.version,
        contract_path: item.contract_path,
        signature_path: item.signature_path,
        contract_hash_sha256: item.contract_hash_sha256,
        signature_verified: item.signature_verified,
        signer_id: item.signer_id,
        trust_epoch: item.trust_epoch,
    })
}

pub fn verify_w20_contract_set_v20(repo_root: &Path) -> Result<ContractSetSummaryV20, SdkError> {
    let contracts_root = repo_root.join("contracts");
    let mut items = Vec::<ContractInspectSummaryV20>::new();
    for rel in W20_REQUIRED_CONTRACT_FILES {
        let path = contracts_root.join(rel);
        if !path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "X-V20-CONTRACT-MISSING: {}",
                path.display()
            )));
        }
        let summary = verify_contract_json_signature_v20(repo_root, &path)?;
        items.push(summary);
    }
    items.sort_by(|lhs, rhs| lhs.contract_id.as_bytes().cmp(rhs.contract_id.as_bytes()));
    Ok(ContractSetSummaryV20 {
        root: contracts_root,
        required_count: W20_REQUIRED_CONTRACT_FILES.len(),
        verified_count: items.len(),
        items,
    })
}
