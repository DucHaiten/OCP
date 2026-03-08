use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

use crate::SdkError;

pub const W18_HASHER_VERSION: &str = "sha256-v1";
pub const W18_SIGNATURE_SCHEMA_VERSION: &str = "1";
pub const W18_SIGNATURE_SCHEMA: &str = "ocp.contract.sig.v1";

pub const W18_REQUIRED_CONTRACT_FILES: [&str; 10] = [
    "required_contracts.v1.json",
    "contract_inventory_schema.v1.json",
    "platform/supported_profile.v1.json",
    "serialization/canonical_serializer.v1.json",
    "cassette/cassette_storage_v17.v1.json",
    "packs/pack_abi.v1.json",
    "packs/connector_set.v1.json",
    "policy/risk_locks_rl01_rl17.v1.json",
    "ops/doctor_fix_cases.v1.json",
    "sot_aliases.v1.json",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractInspectSummaryV18 {
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
pub struct ContractSetSummaryV18 {
    pub root: PathBuf,
    pub required_count: usize,
    pub verified_count: usize,
    pub items: Vec<ContractInspectSummaryV18>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SotAliasItemSummaryV18 {
    pub canonical_path: String,
    pub mirror_path: String,
    pub mirror_exists: bool,
    pub bytes_identical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SotAliasSummaryV18 {
    pub alias_count: usize,
    pub verified_count: usize,
    pub items: Vec<SotAliasItemSummaryV18>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignatureRecordV18 {
    schema: String,
    algorithm: String,
    signature_schema_version: String,
    contract_id: String,
    contract_hash_sha256: String,
    pubkey_id: String,
    trust_epoch: u64,
    signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustStoreV18 {
    allowed_keys: BTreeSet<String>,
    current_trust_epoch: u64,
}

pub fn canonical_json_value_v18(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = JsonMap::new();
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
            for key in keys {
                if let Some(item) = map.get(&key) {
                    sorted.insert(key, canonical_json_value_v18(item));
                }
            }
            JsonValue::Object(sorted)
        }
        JsonValue::Array(items) => JsonValue::Array(
            items
                .iter()
                .map(canonical_json_value_v18)
                .collect::<Vec<JsonValue>>(),
        ),
        _ => value.clone(),
    }
}

pub fn canonical_json_string_v18(value: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonical_json_value_v18(value);
    serde_json::to_string(&canonical)
        .map_err(|err| SdkError::SupplyInvalid(format!("X-V18-CANONICAL-JSON: {err}")))
}

pub fn contract_sig_path_v18(contract_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sig", contract_path.to_string_lossy()))
}

pub fn build_contract_signature_sha256_v18(
    contract_id: &str,
    contract_hash_sha256: &str,
    pubkey_id: &str,
    trust_epoch: u64,
) -> String {
    let payload = format!(
        "ocp-v18-contract-sign-v1|contract_id={}|contract_hash_sha256={}|pubkey_id={}|trust_epoch={}",
        contract_id.trim(),
        contract_hash_sha256.trim(),
        pubkey_id.trim(),
        trust_epoch
    );
    sha256_hex(payload.as_bytes())
}

pub fn inspect_contract_json_v18(
    contract_path: &Path,
) -> Result<ContractInspectSummaryV18, SdkError> {
    let value = read_json_file(contract_path)?;
    let contract_id = str_field(&value, "contract_id", contract_path)?;
    let version = value
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("v1")
        .to_string();
    let canonical = canonical_json_string_v18(&value)?;
    let contract_hash_sha256 = sha256_hex(canonical.as_bytes());
    Ok(ContractInspectSummaryV18 {
        contract_id,
        version,
        contract_path: contract_path.to_path_buf(),
        signature_path: contract_sig_path_v18(contract_path),
        contract_hash_sha256,
        signature_verified: false,
        signer_id: None,
        trust_epoch: None,
    })
}

pub fn sign_contract_json_v18(
    contract_path: &Path,
    pubkey_id: &str,
    trust_epoch: u64,
) -> Result<PathBuf, SdkError> {
    let inspect = inspect_contract_json_v18(contract_path)?;
    let signature = build_contract_signature_sha256_v18(
        &inspect.contract_id,
        &inspect.contract_hash_sha256,
        pubkey_id,
        trust_epoch,
    );
    let mut root = JsonMap::new();
    root.insert(
        "schema".to_string(),
        JsonValue::String(W18_SIGNATURE_SCHEMA.to_string()),
    );
    root.insert(
        "algorithm".to_string(),
        JsonValue::String(W18_HASHER_VERSION.to_string()),
    );
    root.insert(
        "signature_schema_version".to_string(),
        JsonValue::String(W18_SIGNATURE_SCHEMA_VERSION.to_string()),
    );
    root.insert(
        "contract_id".to_string(),
        JsonValue::String(inspect.contract_id.clone()),
    );
    root.insert(
        "contract_hash_sha256".to_string(),
        JsonValue::String(inspect.contract_hash_sha256.clone()),
    );
    root.insert(
        "pubkey_id".to_string(),
        JsonValue::String(pubkey_id.trim().to_string()),
    );
    root.insert("trust_epoch".to_string(), JsonValue::from(trust_epoch));
    root.insert("signature".to_string(), JsonValue::String(signature));
    let sig_path = contract_sig_path_v18(contract_path);
    let rendered = serde_json::to_string_pretty(&JsonValue::Object(root))
        .map_err(|err| SdkError::SupplyInvalid(format!("X-V18-SIGN-SERIALIZE: {err}")))?;
    fs::write(&sig_path, format!("{rendered}\n"))?;
    Ok(sig_path)
}

pub fn verify_contract_json_signature_v18(
    repo_root: &Path,
    contract_path: &Path,
) -> Result<ContractInspectSummaryV18, SdkError> {
    let mut inspect = inspect_contract_json_v18(contract_path)?;
    if !inspect.signature_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-MISSING: {}",
            inspect.signature_path.display()
        )));
    }
    let sig = parse_signature_file_v18(&inspect.signature_path)?;
    if sig.schema != W18_SIGNATURE_SCHEMA {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-SCHEMA: expected `{}`, got `{}`",
            W18_SIGNATURE_SCHEMA, sig.schema
        )));
    }
    if sig.algorithm != W18_HASHER_VERSION {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-ALGORITHM: expected `{}`, got `{}`",
            W18_HASHER_VERSION, sig.algorithm
        )));
    }
    if sig.signature_schema_version != W18_SIGNATURE_SCHEMA_VERSION {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-VERSION: expected `{}`, got `{}`",
            W18_SIGNATURE_SCHEMA_VERSION, sig.signature_schema_version
        )));
    }
    if sig.contract_id != inspect.contract_id {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-ID-MISMATCH: `{}` != `{}`",
            sig.contract_id, inspect.contract_id
        )));
    }
    if sig.contract_hash_sha256 != inspect.contract_hash_sha256 {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-HASH-MISMATCH: `{}` != `{}`",
            sig.contract_hash_sha256, inspect.contract_hash_sha256
        )));
    }
    let expected = build_contract_signature_sha256_v18(
        &inspect.contract_id,
        &inspect.contract_hash_sha256,
        &sig.pubkey_id,
        sig.trust_epoch,
    );
    if expected != sig.signature {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-SIGNATURE-MISMATCH: expected `{expected}`, got `{}`",
            sig.signature
        )));
    }

    let trust = parse_trust_store_v18(&repo_root.join("trust.toml"))?;
    if !trust.allowed_keys.contains(sig.pubkey_id.as_str()) {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-TRUST-KEY-NOT-ALLOWED: `{}` not found in trust.toml",
            sig.pubkey_id
        )));
    }
    if sig.trust_epoch > trust.current_trust_epoch {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-TRUST-EPOCH-FUTURE: signature epoch {} > trust epoch {}",
            sig.trust_epoch, trust.current_trust_epoch
        )));
    }

    inspect.signature_verified = true;
    inspect.signer_id = Some(sig.pubkey_id);
    inspect.trust_epoch = Some(sig.trust_epoch);
    Ok(inspect)
}

pub fn verify_w18_contract_set_v18(repo_root: &Path) -> Result<ContractSetSummaryV18, SdkError> {
    let contracts_root = repo_root.join("contracts");
    let mut items = Vec::new();
    for rel in W18_REQUIRED_CONTRACT_FILES {
        let path = contracts_root.join(rel);
        if !path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "X-V18-CONTRACT-MISSING: {}",
                path.display()
            )));
        }
        let summary = verify_contract_json_signature_v18(repo_root, &path)?;
        items.push(summary);
    }
    items.sort_by(|lhs, rhs| lhs.contract_id.as_bytes().cmp(rhs.contract_id.as_bytes()));
    Ok(ContractSetSummaryV18 {
        root: contracts_root,
        required_count: W18_REQUIRED_CONTRACT_FILES.len(),
        verified_count: items.len(),
        items,
    })
}

pub fn verify_sot_alias_contract_v18(repo_root: &Path) -> Result<SotAliasSummaryV18, SdkError> {
    let alias_path = repo_root.join("contracts").join("sot_aliases.v1.json");
    let alias_value = read_json_file(&alias_path)?;
    let aliases = alias_value
        .get("aliases")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-V18-SOT-ALIAS-SCHEMA: missing array `aliases` in {}",
                alias_path.display()
            ))
        })?;
    let mut items = Vec::<SotAliasItemSummaryV18>::new();
    for entry in aliases {
        let canonical_rel = entry
            .get("canonical")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                SdkError::SupplyInvalid(format!(
                    "X-V18-SOT-ALIAS-SCHEMA: missing `canonical` in {}",
                    alias_path.display()
                ))
            })?;
        let mirror_rel = entry
            .get("mirror")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                SdkError::SupplyInvalid(format!(
                    "X-V18-SOT-ALIAS-SCHEMA: missing `mirror` in {}",
                    alias_path.display()
                ))
            })?;
        let canonical_path = repo_root.join(canonical_rel);
        if !canonical_path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "X-V18-SOT-CANONICAL-MISSING: {}",
                canonical_path.display()
            )));
        }
        let mirror_path = repo_root.join(mirror_rel);
        let mirror_exists = mirror_path.exists();
        let bytes_identical = if mirror_exists {
            fs::read(&canonical_path)? == fs::read(&mirror_path)?
        } else {
            true
        };
        if mirror_exists && !bytes_identical {
            return Err(SdkError::SupplyInvalid(format!(
                "X-V18-SOT-ALIAS-MISMATCH: canonical `{}` != mirror `{}`",
                canonical_path.display(),
                mirror_path.display()
            )));
        }
        items.push(SotAliasItemSummaryV18 {
            canonical_path: canonical_rel.replace('\\', "/"),
            mirror_path: mirror_rel.replace('\\', "/"),
            mirror_exists,
            bytes_identical,
        });
    }
    Ok(SotAliasSummaryV18 {
        alias_count: items.len(),
        verified_count: items
            .iter()
            .filter(|item| !item.mirror_exists || item.bytes_identical)
            .count(),
        items,
    })
}

fn parse_trust_store_v18(path: &Path) -> Result<TrustStoreV18, SdkError> {
    if !path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-TRUST-MISSING: missing {}",
            path.display()
        )));
    }
    let raw = fs::read_to_string(path)?;
    let mut current_section = String::new();
    let mut allowed_keys = BTreeSet::<String>::new();
    let mut current_trust_epoch = 0u64;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((key_raw, value_raw)) = line.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        let value = value_raw.trim();

        if current_section == "contract_signing" {
            if key == "trust_epoch" {
                if let Ok(parsed) = value.trim_matches('"').parse::<u64>() {
                    current_trust_epoch = parsed;
                }
            }
            continue;
        }

        if current_section == "trusted_signers" || current_section == "trusted_signers.contracts" {
            if key == "keys" {
                for item in parse_string_array_literal(value) {
                    if !item.is_empty() {
                        allowed_keys.insert(item);
                    }
                }
            }
            continue;
        }

        if current_section == "trusted_key" && key == "id" {
            let id = strip_toml_quotes(value);
            if !id.is_empty() {
                allowed_keys.insert(id);
            }
        }
    }

    if current_trust_epoch == 0 {
        current_trust_epoch = 1;
    }
    if allowed_keys.is_empty() {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V18-TRUST-EMPTY: no trusted keys configured in {}",
            path.display()
        )));
    }
    Ok(TrustStoreV18 {
        allowed_keys,
        current_trust_epoch,
    })
}

fn parse_signature_file_v18(path: &Path) -> Result<SignatureRecordV18, SdkError> {
    let value = read_json_file(path)?;
    let trust_epoch = value
        .get("trust_epoch")
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-V18-SIGNATURE-FIELD: missing `trust_epoch` in {}",
                path.display()
            ))
        })?;
    Ok(SignatureRecordV18 {
        schema: str_field(&value, "schema", path)?,
        algorithm: str_field(&value, "algorithm", path)?,
        signature_schema_version: str_field(&value, "signature_schema_version", path)?,
        contract_id: str_field(&value, "contract_id", path)?,
        contract_hash_sha256: str_field(&value, "contract_hash_sha256", path)?,
        pubkey_id: str_field(&value, "pubkey_id", path)?,
        trust_epoch,
        signature: str_field(&value, "signature", path)?,
    })
}

fn str_field(value: &JsonValue, field: &str, path: &Path) -> Result<String, SdkError> {
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(format!(
                "X-V18-CONTRACT-FIELD: missing `{field}` in {}",
                path.display()
            ))
        })
}

fn read_json_file(path: &Path) -> Result<JsonValue, SdkError> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-V18-CONTRACT-JSON: invalid JSON {} ({err})",
            path.display()
        ))
    })
}

fn parse_string_array_literal(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if !(value.starts_with('[') && value.ends_with(']')) {
        return Vec::new();
    }
    value[1..value.len() - 1]
        .split(',')
        .map(strip_toml_quotes)
        .filter(|item| !item.is_empty())
        .collect::<Vec<String>>()
}

fn strip_toml_quotes(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
