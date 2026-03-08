use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

use crate::SdkError;

pub const W17_HASHER_VERSION: &str = "sha256-v1";
pub const W17_SIGNATURE_SCHEMA_VERSION: &str = "1";
pub const W17_SIGNATURE_SCHEMA: &str = "ocp.w17.contract.sig.v1";
pub const W17_SEMANTIC_HASH_VERSION: &str = "semantic_hash_v1";
pub const W17_ARTIFACT_HASH_VERSION: &str = "artifact_hash_v1";

pub const W17_REQUIRED_CONTRACT_FILES: [&str; 6] = [
    "supported_platform_profile.v1.json",
    "cassette_storage_contract.v1.json",
    "pack_abi_spec.v1.json",
    "risk_lock_policies.v1.json",
    "semantic_hash_taxonomy.v1.json",
    "canonical_serialization_spec.v1.json",
];

pub const W17_PACK_ABI_SCHEMA: &str = "ocp.w17.pack_abi_spec.v1";
pub const W17_PACK_BOUNDARY_WASI_V1: &str = "pack_boundary_wasi_v1";
pub const W17_PACK_BOUNDARY_NATIVE_CAP_V1: &str = "pack_boundary_native_cap_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackBoundaryV17 {
    WasiV1,
    NativeCapV1,
}

impl PackBoundaryV17 {
    pub const fn as_contract_id(self) -> &'static str {
        match self {
            Self::WasiV1 => W17_PACK_BOUNDARY_WASI_V1,
            Self::NativeCapV1 => W17_PACK_BOUNDARY_NATIVE_CAP_V1,
        }
    }

    pub fn from_contract_id(raw: &str) -> Option<Self> {
        match raw.trim() {
            W17_PACK_BOUNDARY_WASI_V1 => Some(Self::WasiV1),
            W17_PACK_BOUNDARY_NATIVE_CAP_V1 => Some(Self::NativeCapV1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackAbiBoundaryRequirementV17 {
    pub boundary_id: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackAbiSpecSummaryV17 {
    pub contract_id: String,
    pub version: String,
    pub contract_path: PathBuf,
    pub boundaries: Vec<PackAbiBoundaryRequirementV17>,
    pub supports_wasi_v1: bool,
    pub supports_native_cap_v1: bool,
    pub observe_hook: bool,
    pub commit_hook: bool,
    pub permission_schema_declaration_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackTrustDecisionV17 {
    pub allowed: bool,
    pub requires_audit_marker: bool,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackBoundaryDecisionV17 {
    pub allowed: bool,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityEdgeDecisionV17 {
    pub allowed: bool,
    pub reason_code: &'static str,
    pub edge_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterCveSeverityV17 {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCveDecisionV17 {
    pub allowed: bool,
    pub requires_audit_marker: bool,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustKeyRecordV17 {
    pub key_id: String,
    pub trust_epoch: u64,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustLifecycleSummaryV17 {
    pub current_epoch: u64,
    pub active_key_id: String,
    pub active_key_epoch: u64,
    pub revoked_key_ids: Vec<String>,
    pub known_key_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DowngradeDecisionV17 {
    pub allowed: bool,
    pub requires_audit_marker: bool,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainDigestInputV17 {
    pub rustc_version: String,
    pub cargo_version: String,
    pub target_triple: String,
    pub build_flags: Vec<String>,
    pub allowed_env: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractInspectSummaryV17 {
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
pub struct ContractSetSummaryV17 {
    pub root: PathBuf,
    pub required_count: usize,
    pub verified_count: usize,
    pub items: Vec<ContractInspectSummaryV17>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContractSignatureRecordV17 {
    schema: String,
    hasher: String,
    signature_schema_version: String,
    contract_id: String,
    trust_epoch: u64,
    signer_id: String,
    contract_hash_sha256: String,
    signature_sha256: String,
}

pub fn semantic_hash_from_bytes_v17(core_semantic_bytes: &[u8], _diagnostic_text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!(
        "semantic_hash_version={}|",
        W17_SEMANTIC_HASH_VERSION
    ));
    hasher.update(core_semantic_bytes);
    hex_digest(hasher.finalize().as_slice())
}

pub fn artifact_hash_from_bytes_v17(artifact_bytes: &[u8], diagnostic_text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!(
        "artifact_hash_version={}|",
        W17_ARTIFACT_HASH_VERSION
    ));
    hasher.update(artifact_bytes);
    hasher.update(b"|diagnostic=");
    hasher.update(diagnostic_text.as_bytes());
    hex_digest(hasher.finalize().as_slice())
}

pub fn canonical_json_value_v17(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = JsonMap::new();
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
            for key in keys {
                if let Some(item) = map.get(&key) {
                    sorted.insert(key, canonical_json_value_v17(item));
                }
            }
            JsonValue::Object(sorted)
        }
        JsonValue::Array(items) => {
            JsonValue::Array(items.iter().map(canonical_json_value_v17).collect())
        }
        _ => value.clone(),
    }
}

pub fn canonical_json_string_v17(value: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonical_json_value_v17(value);
    serde_json::to_string(&canonical)
        .map_err(|err| SdkError::SupplyInvalid(format!("X-SERIALIZATION-CANONICAL: {err}")))
}

pub fn evaluate_trust_lifecycle_v17(
    current_epoch: u64,
    records: &[TrustKeyRecordV17],
) -> Result<TrustLifecycleSummaryV17, SdkError> {
    if records.is_empty() {
        return Err(SdkError::SupplyInvalid(
            "X-TRUST-LIFECYCLE-EMPTY: no trust keys supplied".to_string(),
        ));
    }

    let mut known = BTreeSet::<String>::new();
    let mut revoked = BTreeSet::<String>::new();
    let mut active_candidates = Vec::<(u64, String)>::new();

    for item in records {
        if !known.insert(item.key_id.clone()) {
            return Err(SdkError::SupplyInvalid(format!(
                "X-TRUST-LIFECYCLE-DUPLICATE: duplicate key `{}`",
                item.key_id
            )));
        }
        if item.revoked {
            revoked.insert(item.key_id.clone());
            continue;
        }
        if item.trust_epoch <= current_epoch {
            active_candidates.push((item.trust_epoch, item.key_id.clone()));
        }
    }

    if active_candidates.is_empty() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-TRUST-LIFECYCLE-NO-ACTIVE: no active key for epoch {}",
            current_epoch
        )));
    }

    active_candidates.sort_by(|lhs, rhs| {
        lhs.0
            .cmp(&rhs.0)
            .then_with(|| lhs.1.as_bytes().cmp(rhs.1.as_bytes()))
    });
    let (active_key_epoch, active_key_id) = active_candidates
        .last()
        .expect("active candidates already checked")
        .clone();

    Ok(TrustLifecycleSummaryV17 {
        current_epoch,
        active_key_id,
        active_key_epoch,
        revoked_key_ids: revoked.into_iter().collect(),
        known_key_ids: known.into_iter().collect(),
    })
}

pub fn evaluate_downgrade_attempt_v17(
    lane: &str,
    current_version: u64,
    requested_version: u64,
) -> DowngradeDecisionV17 {
    if requested_version >= current_version {
        return DowngradeDecisionV17 {
            allowed: true,
            requires_audit_marker: false,
            reason_code: "RC-UPGRADE-OK",
        };
    }

    match lane {
        "locked_v071" => DowngradeDecisionV17 {
            allowed: false,
            requires_audit_marker: true,
            reason_code: "RC-DOWNGRADE-BLOCKED-LOCKED",
        },
        "locked_v06" | "quarantine" => DowngradeDecisionV17 {
            allowed: true,
            requires_audit_marker: true,
            reason_code: "RC-DOWNGRADE-AUDIT",
        },
        _ => DowngradeDecisionV17 {
            allowed: false,
            requires_audit_marker: true,
            reason_code: "RC-LANE-UNSUPPORTED",
        },
    }
}

pub fn enforce_build_env_allowlist_v17(
    allowed_keys: &[&str],
    observed_env: &[(String, String)],
) -> Result<Vec<(String, String)>, SdkError> {
    let allow = allowed_keys
        .iter()
        .map(|item| item.trim().to_string())
        .collect::<BTreeSet<String>>();

    let mut unknown = observed_env
        .iter()
        .map(|(key, _)| key.trim().to_string())
        .filter(|key| !key.is_empty() && !allow.contains(key))
        .collect::<Vec<String>>();
    unknown.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    unknown.dedup();
    if !unknown.is_empty() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-TOOLCHAIN-ENV-DENY: unknown env flags: {}",
            unknown.join(", ")
        )));
    }

    let mut out = observed_env
        .iter()
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect::<Vec<(String, String)>>();
    out.sort_by(|lhs, rhs| {
        lhs.0
            .as_bytes()
            .cmp(rhs.0.as_bytes())
            .then_with(|| lhs.1.as_bytes().cmp(rhs.1.as_bytes()))
    });
    Ok(out)
}

pub fn toolchain_digest_v17(input: &ToolchainDigestInputV17) -> String {
    let mut flags = input
        .build_flags
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<String>>();
    flags.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));

    let mut env = input.allowed_env.clone();
    env.sort_by(|lhs, rhs| {
        lhs.0
            .as_bytes()
            .cmp(rhs.0.as_bytes())
            .then_with(|| lhs.1.as_bytes().cmp(rhs.1.as_bytes()))
    });

    let mut canonical = String::new();
    canonical.push_str("w17_toolchain_digest_v1\n");
    canonical.push_str("hasher=sha256-v1\n");
    canonical.push_str(&format!("rustc={}\n", input.rustc_version.trim()));
    canonical.push_str(&format!("cargo={}\n", input.cargo_version.trim()));
    canonical.push_str(&format!("target={}\n", input.target_triple.trim()));
    canonical.push_str("flags=");
    canonical.push_str(&flags.join(","));
    canonical.push('\n');
    canonical.push_str("env=");
    canonical.push_str(
        &env.iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<String>>()
            .join(","),
    );
    canonical.push('\n');
    sha256_hex(canonical.as_bytes())
}

pub fn contract_sig_path_v17(contract_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sig", contract_path.to_string_lossy()))
}

pub fn sign_contract_json_v17(
    contract_path: &Path,
    signer_id: &str,
    trust_epoch: u64,
) -> Result<PathBuf, SdkError> {
    let inspect = inspect_contract_json_v17(contract_path)?;
    let signature_sha256 = build_contract_signature_sha256_v17(
        &inspect.contract_id,
        &inspect.contract_hash_sha256,
        trust_epoch,
        signer_id,
    );
    let payload = JsonValue::Object({
        let mut root = JsonMap::new();
        root.insert(
            "schema".to_string(),
            JsonValue::String(W17_SIGNATURE_SCHEMA.to_string()),
        );
        root.insert(
            "hasher".to_string(),
            JsonValue::String(W17_HASHER_VERSION.to_string()),
        );
        root.insert(
            "signature_schema_version".to_string(),
            JsonValue::String(W17_SIGNATURE_SCHEMA_VERSION.to_string()),
        );
        root.insert(
            "contract_id".to_string(),
            JsonValue::String(inspect.contract_id.clone()),
        );
        root.insert("trust_epoch".to_string(), JsonValue::from(trust_epoch));
        root.insert(
            "signer_id".to_string(),
            JsonValue::String(signer_id.to_string()),
        );
        root.insert(
            "contract_hash_sha256".to_string(),
            JsonValue::String(inspect.contract_hash_sha256.clone()),
        );
        root.insert(
            "signature_sha256".to_string(),
            JsonValue::String(signature_sha256),
        );
        root
    });
    let rendered = serde_json::to_string_pretty(&payload)
        .map_err(|err| SdkError::SupplyInvalid(format!("X-CONTRACT-SIG-SERIALIZE: {err}")))?;
    let sig_path = contract_sig_path_v17(contract_path);
    fs::write(&sig_path, format!("{rendered}\n"))?;
    Ok(sig_path)
}

pub fn inspect_contract_json_v17(
    contract_path: &Path,
) -> Result<ContractInspectSummaryV17, SdkError> {
    let value = read_json_file(contract_path)?;
    let contract_id = value
        .get("contract_id")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-CONTRACT-SOT-ID-MISSING: missing contract_id in {}",
                contract_path.display()
            ))
        })?
        .to_string();
    let version = value
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("-")
        .to_string();
    let canonical = canonical_json_string_v17(&value)?;
    let hash = sha256_hex(canonical.as_bytes());
    Ok(ContractInspectSummaryV17 {
        contract_id,
        version,
        contract_path: contract_path.to_path_buf(),
        signature_path: contract_sig_path_v17(contract_path),
        contract_hash_sha256: hash,
        signature_verified: false,
        signer_id: None,
        trust_epoch: None,
    })
}

pub fn verify_contract_json_signature_v17(
    contract_path: &Path,
) -> Result<ContractInspectSummaryV17, SdkError> {
    let mut inspect = inspect_contract_json_v17(contract_path)?;
    let sig_path = inspect.signature_path.clone();
    if !sig_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-MISSING: missing signature file `{}`",
            sig_path.display()
        )));
    }
    let sig = parse_contract_signature_file_v17(&sig_path)?;
    if sig.schema != W17_SIGNATURE_SCHEMA {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-SCHEMA: expected `{}`, got `{}`",
            W17_SIGNATURE_SCHEMA, sig.schema
        )));
    }
    if sig.hasher != W17_HASHER_VERSION {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-HASHER: expected `{}`, got `{}`",
            W17_HASHER_VERSION, sig.hasher
        )));
    }
    if sig.signature_schema_version != W17_SIGNATURE_SCHEMA_VERSION {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-VERSION: expected `{}`, got `{}`",
            W17_SIGNATURE_SCHEMA_VERSION, sig.signature_schema_version
        )));
    }
    if sig.contract_id != inspect.contract_id {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-ID-MISMATCH: signature contract_id `{}` != actual `{}`",
            sig.contract_id, inspect.contract_id
        )));
    }
    if sig.contract_hash_sha256 != inspect.contract_hash_sha256 {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-HASH-MISMATCH: expected `{}`, got `{}`",
            inspect.contract_hash_sha256, sig.contract_hash_sha256
        )));
    }
    let expected_signature = build_contract_signature_sha256_v17(
        &inspect.contract_id,
        &inspect.contract_hash_sha256,
        sig.trust_epoch,
        &sig.signer_id,
    );
    if sig.signature_sha256 != expected_signature {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-MISMATCH: expected `{}`, got `{}`",
            expected_signature, sig.signature_sha256
        )));
    }

    inspect.signature_verified = true;
    inspect.signer_id = Some(sig.signer_id);
    inspect.trust_epoch = Some(sig.trust_epoch);
    Ok(inspect)
}

pub fn verify_contract_signature_file_v17(
    signature_path: &Path,
) -> Result<ContractInspectSummaryV17, SdkError> {
    let contract_path = derive_contract_path_from_sig(signature_path)?;
    verify_contract_json_signature_v17(&contract_path)
}

pub fn verify_w17_contract_set_v17(
    contracts_root: &Path,
) -> Result<ContractSetSummaryV17, SdkError> {
    let mut items = Vec::new();
    for rel in W17_REQUIRED_CONTRACT_FILES {
        let path = contracts_root.join(rel);
        if !path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "X-CONTRACT-SOT-MISSING: missing `{}`",
                path.display()
            )));
        }
        let summary = verify_contract_json_signature_v17(&path)?;
        items.push(summary);
    }
    items.sort_by(|lhs, rhs| lhs.contract_id.as_bytes().cmp(rhs.contract_id.as_bytes()));
    Ok(ContractSetSummaryV17 {
        root: contracts_root.to_path_buf(),
        required_count: W17_REQUIRED_CONTRACT_FILES.len(),
        verified_count: items.len(),
        items,
    })
}

pub fn inspect_pack_abi_spec_v17(contract_path: &Path) -> Result<PackAbiSpecSummaryV17, SdkError> {
    let value = read_json_file(contract_path)?;
    let schema = value
        .get("schema")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-PACK-ABI-SCHEMA-MISSING: missing schema in {}",
                contract_path.display()
            ))
        })?;
    if schema != W17_PACK_ABI_SCHEMA {
        return Err(SdkError::SupplyInvalid(format!(
            "X-PACK-ABI-SCHEMA-MISMATCH: expected `{W17_PACK_ABI_SCHEMA}`, got `{schema}`"
        )));
    }

    let contract_id = value
        .get("contract_id")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-PACK-ABI-ID-MISSING: missing contract_id in {}",
                contract_path.display()
            ))
        })?
        .to_string();
    let version = value
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("-")
        .to_string();

    let boundaries_value = value.get("boundaries").ok_or_else(|| {
        SdkError::SupplyInvalid(format!(
            "X-PACK-ABI-BOUNDARIES-MISSING: missing boundaries in {}",
            contract_path.display()
        ))
    })?;
    let boundaries_array = boundaries_value.as_array().ok_or_else(|| {
        SdkError::SupplyInvalid(format!(
            "X-PACK-ABI-BOUNDARIES-TYPE: boundaries must be an array in {}",
            contract_path.display()
        ))
    })?;

    let mut boundaries = Vec::<PackAbiBoundaryRequirementV17>::new();
    for item in boundaries_array {
        let boundary_id = item
            .get("boundary_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                SdkError::SupplyInvalid(format!(
                    "X-PACK-ABI-BOUNDARY-ID: missing boundary_id in {}",
                    contract_path.display()
                ))
            })?
            .to_string();
        let status = item
            .get("status")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                SdkError::SupplyInvalid(format!(
                    "X-PACK-ABI-BOUNDARY-STATUS: missing status for boundary `{boundary_id}`"
                ))
            })?;
        boundaries.push(PackAbiBoundaryRequirementV17 {
            boundary_id,
            required: status == "required",
        });
    }
    boundaries.sort_by(|lhs, rhs| lhs.boundary_id.as_bytes().cmp(rhs.boundary_id.as_bytes()));

    let supports_wasi_v1 = boundaries
        .iter()
        .any(|item| item.boundary_id == W17_PACK_BOUNDARY_WASI_V1 && item.required);
    let supports_native_cap_v1 = boundaries
        .iter()
        .any(|item| item.boundary_id == W17_PACK_BOUNDARY_NATIVE_CAP_V1 && item.required);
    if !supports_wasi_v1 || !supports_native_cap_v1 {
        return Err(SdkError::SupplyInvalid(
            "X-PACK-ABI-BOUNDARY-MISSING: both pack_boundary_wasi_v1 and pack_boundary_native_cap_v1 must be required"
                .to_string(),
        ));
    }

    let adapter = value.get("adapter_interface").ok_or_else(|| {
        SdkError::SupplyInvalid(format!(
            "X-PACK-ABI-ADAPTER-MISSING: missing adapter_interface in {}",
            contract_path.display()
        ))
    })?;
    let hooks = adapter
        .get("hooks")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-PACK-ABI-HOOKS-MISSING: missing hooks array in {}",
                contract_path.display()
            ))
        })?;
    let observe_hook = hooks.iter().any(|item| item.as_str() == Some("observe"));
    let commit_hook = hooks.iter().any(|item| item.as_str() == Some("commit"));
    if !observe_hook || !commit_hook {
        return Err(SdkError::SupplyInvalid(
            "X-PACK-ABI-HOOKS-REQUIRED: hooks must include `observe` and `commit`".to_string(),
        ));
    }
    let permission_schema_declaration_required = adapter
        .get("permission_schema_declaration_required")
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-PACK-ABI-PERM-SCHEMA-FIELD: missing permission_schema_declaration_required in {}",
                contract_path.display()
            ))
        })?;
    if !permission_schema_declaration_required {
        return Err(SdkError::SupplyInvalid(
            "X-PACK-ABI-PERM-SCHEMA-REQUIRED: permission schema declaration must be required"
                .to_string(),
        ));
    }

    Ok(PackAbiSpecSummaryV17 {
        contract_id,
        version,
        contract_path: contract_path.to_path_buf(),
        boundaries,
        supports_wasi_v1,
        supports_native_cap_v1,
        observe_hook,
        commit_hook,
        permission_schema_declaration_required,
    })
}

pub fn evaluate_pack_trust_policy_v17(
    lane: &str,
    signed: bool,
    attested: bool,
    trusted: bool,
    sbom_ok: bool,
    fuzz_policy_ok: bool,
) -> PackTrustDecisionV17 {
    if signed && attested && trusted && sbom_ok && fuzz_policy_ok {
        return PackTrustDecisionV17 {
            allowed: true,
            requires_audit_marker: false,
            reason_code: "RC-PACK-TRUST-OK",
        };
    }
    match lane {
        "locked_v071" => {
            if !signed {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-SIGNATURE-REQUIRED",
                };
            }
            if !attested {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-ATTESTATION-REQUIRED",
                };
            }
            if !trusted {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-TRUST-REQUIRED",
                };
            }
            if !sbom_ok {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-SBOM-REQUIRED",
                };
            }
            PackTrustDecisionV17 {
                allowed: false,
                requires_audit_marker: true,
                reason_code: "RC-PACK-FUZZ-POLICY-REQUIRED",
            }
        }
        "locked_v06" => {
            if !signed {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-SIGNATURE-REQUIRED",
                };
            }
            if !trusted {
                return PackTrustDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-PACK-TRUST-REQUIRED",
                };
            }
            PackTrustDecisionV17 {
                allowed: true,
                requires_audit_marker: true,
                reason_code: "RC-PACK-TRUST-COMPAT-AUDIT",
            }
        }
        "quarantine" => PackTrustDecisionV17 {
            allowed: true,
            requires_audit_marker: true,
            reason_code: "RC-PACK-TRUST-QUARANTINE-AUDIT",
        },
        _ => PackTrustDecisionV17 {
            allowed: false,
            requires_audit_marker: true,
            reason_code: "RC-LANE-UNSUPPORTED",
        },
    }
}

pub fn evaluate_pack_boundary_v17(
    boundary_id: &str,
    declared_capabilities: &[String],
    direct_host_io: bool,
) -> PackBoundaryDecisionV17 {
    let Some(boundary) = PackBoundaryV17::from_contract_id(boundary_id) else {
        return PackBoundaryDecisionV17 {
            allowed: false,
            reason_code: "RC-PACK-BOUNDARY-UNKNOWN",
        };
    };
    if direct_host_io {
        return PackBoundaryDecisionV17 {
            allowed: false,
            reason_code: "RC-PACK-BOUNDARY-NO-NAKED-IO",
        };
    }

    for capability in declared_capabilities {
        let cap = capability.trim();
        if cap.is_empty() {
            return PackBoundaryDecisionV17 {
                allowed: false,
                reason_code: "RC-PACK-CAPABILITY-INVALID",
            };
        }
        match boundary {
            PackBoundaryV17::WasiV1 if cap.starts_with("native.") => {
                return PackBoundaryDecisionV17 {
                    allowed: false,
                    reason_code: "RC-PACK-BOUNDARY-WASI-CAP-MISMATCH",
                };
            }
            PackBoundaryV17::NativeCapV1 if cap.starts_with("wasi.") => {
                return PackBoundaryDecisionV17 {
                    allowed: false,
                    reason_code: "RC-PACK-BOUNDARY-NATIVE-CAP-MISMATCH",
                };
            }
            _ => {}
        }
    }

    PackBoundaryDecisionV17 {
        allowed: true,
        reason_code: match boundary {
            PackBoundaryV17::WasiV1 => "RC-PACK-BOUNDARY-WASI-OK",
            PackBoundaryV17::NativeCapV1 => "RC-PACK-BOUNDARY-NATIVE-OK",
        },
    }
}

pub fn evaluate_capability_edge_v17(
    caller_pkg: &str,
    callee_pkg: &str,
    capability_kind: &str,
    lane_profile: &str,
    caller_granted: bool,
    callee_granted: bool,
    edge_granted: bool,
) -> CapabilityEdgeDecisionV17 {
    let edge_id = format!(
        "{}|{}|{}|{}",
        caller_pkg.trim(),
        callee_pkg.trim(),
        capability_kind.trim(),
        lane_profile.trim()
    );
    if !caller_granted {
        return CapabilityEdgeDecisionV17 {
            allowed: false,
            reason_code: "RC-LAUNDER-CALLER-DENY",
            edge_id,
        };
    }
    if !callee_granted {
        return CapabilityEdgeDecisionV17 {
            allowed: false,
            reason_code: "RC-LAUNDER-CALLEE-DENY",
            edge_id,
        };
    }
    if !edge_granted {
        return CapabilityEdgeDecisionV17 {
            allowed: false,
            reason_code: "RC-LAUNDER-EDGE-DENY",
            edge_id,
        };
    }
    CapabilityEdgeDecisionV17 {
        allowed: true,
        reason_code: "RC-LAUNDER-OK",
        edge_id,
    }
}

pub fn evaluate_adapter_cve_policy_v17(
    lane: &str,
    severity: AdapterCveSeverityV17,
    patched: bool,
    fix_available: bool,
    sbom_present: bool,
) -> AdapterCveDecisionV17 {
    if !sbom_present {
        return match lane {
            "locked_v071" => AdapterCveDecisionV17 {
                allowed: false,
                requires_audit_marker: true,
                reason_code: "RC-CVE-SBOM-REQUIRED",
            },
            "locked_v06" | "quarantine" => AdapterCveDecisionV17 {
                allowed: true,
                requires_audit_marker: true,
                reason_code: "RC-CVE-SBOM-MISSING-AUDIT",
            },
            _ => AdapterCveDecisionV17 {
                allowed: false,
                requires_audit_marker: true,
                reason_code: "RC-LANE-UNSUPPORTED",
            },
        };
    }

    if patched {
        return AdapterCveDecisionV17 {
            allowed: true,
            requires_audit_marker: false,
            reason_code: "RC-CVE-OK",
        };
    }

    match lane {
        "locked_v071" => match severity {
            AdapterCveSeverityV17::Critical | AdapterCveSeverityV17::High => {
                AdapterCveDecisionV17 {
                    allowed: false,
                    requires_audit_marker: true,
                    reason_code: "RC-CVE-BLOCK-STRICT",
                }
            }
            AdapterCveSeverityV17::Medium if fix_available => AdapterCveDecisionV17 {
                allowed: false,
                requires_audit_marker: true,
                reason_code: "RC-CVE-PATCH-REQUIRED",
            },
            AdapterCveSeverityV17::Medium | AdapterCveSeverityV17::Low => AdapterCveDecisionV17 {
                allowed: true,
                requires_audit_marker: true,
                reason_code: "RC-CVE-STRICT-AUDIT",
            },
        },
        "locked_v06" => match severity {
            AdapterCveSeverityV17::Critical => AdapterCveDecisionV17 {
                allowed: false,
                requires_audit_marker: true,
                reason_code: "RC-CVE-CRITICAL-DENY",
            },
            _ => AdapterCveDecisionV17 {
                allowed: true,
                requires_audit_marker: true,
                reason_code: "RC-CVE-COMPAT-AUDIT",
            },
        },
        "quarantine" => AdapterCveDecisionV17 {
            allowed: true,
            requires_audit_marker: true,
            reason_code: "RC-CVE-QUARANTINE-AUDIT",
        },
        _ => AdapterCveDecisionV17 {
            allowed: false,
            requires_audit_marker: true,
            reason_code: "RC-LANE-UNSUPPORTED",
        },
    }
}

pub fn build_contract_signature_sha256_v17(
    contract_id: &str,
    contract_hash_sha256: &str,
    trust_epoch: u64,
    signer_id: &str,
) -> String {
    let payload = format!(
        "ocp-w17-contract-sign-v1|contract_id={}|contract_hash_sha256={}|trust_epoch={}|signer_id={}",
        contract_id, contract_hash_sha256, trust_epoch, signer_id
    );
    sha256_hex(payload.as_bytes())
}

fn derive_contract_path_from_sig(signature_path: &Path) -> Result<PathBuf, SdkError> {
    let raw = signature_path.to_string_lossy().to_string();
    if !raw.ends_with(".sig") {
        return Err(SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SIG-PATH: `{}` is not a .sig file",
            signature_path.display()
        )));
    }
    let contract = raw.trim_end_matches(".sig").to_string();
    Ok(PathBuf::from(contract))
}

fn parse_contract_signature_file_v17(path: &Path) -> Result<ContractSignatureRecordV17, SdkError> {
    let value = read_json_file(path)?;
    let schema = json_str_field(&value, "schema", path)?;
    let hasher = json_str_field(&value, "hasher", path)?;
    let signature_schema_version = json_str_field(&value, "signature_schema_version", path)?;
    let contract_id = json_str_field(&value, "contract_id", path)?;
    let trust_epoch = value
        .get("trust_epoch")
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-CONTRACT-SIG-FIELD: missing u64 trust_epoch in {}",
                path.display()
            ))
        })?;
    let signer_id = json_str_field(&value, "signer_id", path)?;
    let contract_hash_sha256 = json_str_field(&value, "contract_hash_sha256", path)?;
    let signature_sha256 = json_str_field(&value, "signature_sha256", path)?;
    Ok(ContractSignatureRecordV17 {
        schema,
        hasher,
        signature_schema_version,
        contract_id,
        trust_epoch,
        signer_id,
        contract_hash_sha256,
        signature_sha256,
    })
}

fn json_str_field(value: &JsonValue, field: &str, path: &Path) -> Result<String, SdkError> {
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-CONTRACT-SIG-FIELD: missing string `{field}` in {}",
                path.display()
            ))
        })
}

fn read_json_file(path: &Path) -> Result<JsonValue, SdkError> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-CONTRACT-SOT-JSON: invalid JSON {} ({err})",
            path.display()
        ))
    })
}

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    hex_digest(&digest)
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
