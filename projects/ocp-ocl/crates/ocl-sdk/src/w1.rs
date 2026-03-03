use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;

use crate::w2::validate_locked_cosmos_bridge_config_v1;
use crate::w5::{parse_cosmos_kits_v1, parse_cosmos_views_v1, ViewKitBindingV1, ViewProfileV1};
use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyLockV1 {
    pub policy_profile_id: String,
    pub canonical_hash256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyLockSyncSummary {
    pub lock_path: PathBuf,
    pub policy_profile_id: String,
    pub canonical_hash256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseProfileV1 {
    pub id: String,
    pub runtime_mode: String,
    pub engine: String,
    pub policy_profile_id: String,
    pub audit: String,
    pub trace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosHiveV1 {
    pub max_swarm_workers: u32,
    pub max_fanout_per_task: u32,
    pub mailbox_max_depth: u32,
    pub max_spawn_per_tick: u32,
    pub max_domains: u32,
    pub max_shadow_worlds: u32,
}

impl Default for CosmosHiveV1 {
    fn default() -> Self {
        Self {
            max_swarm_workers: 16,
            max_fanout_per_task: 8,
            mailbox_max_depth: 64,
            max_spawn_per_tick: 4,
            max_domains: 8,
            max_shadow_worlds: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosSpecV1 {
    pub universes: Vec<UniverseProfileV1>,
    pub hive: CosmosHiveV1,
    pub views: Vec<ViewProfileV1>,
    pub kits: Vec<ViewKitBindingV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosLockV1 {
    pub signer_id: Option<String>,
    pub sign_key_hint: Option<String>,
    pub trust_store_hint: Option<String>,
    pub canonical_hash256: String,
    pub signature_b64: String,
    pub universes: Vec<UniverseProfileV1>,
    pub hive: CosmosHiveV1,
    pub views: Vec<ViewProfileV1>,
    pub kits: Vec<ViewKitBindingV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosInitSummary {
    pub cosmos_path: PathBuf,
    pub preset: String,
    pub universes_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmosLockSyncSummary {
    pub lock_path: PathBuf,
    pub universes_synced: usize,
    pub canonical_hash256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseSelectionV1 {
    pub universe_id: String,
    pub runtime_mode: String,
    pub engine: String,
    pub policy_profile_id: String,
    pub is_legacy: bool,
}

pub fn sync_policy_lock_v1(root: &Path) -> Result<PolicyLockSyncSummary, SdkError> {
    let manifest_path = root.join("Ocl.toml");
    if !manifest_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            manifest_path.display()
        )));
    }
    let manifest = fs::read_to_string(&manifest_path)?;
    let policy_profile_id = parse_manifest_policy_profile_id(&manifest);
    let canonical_hash256 = build_policy_canonical_hash256(&policy_profile_id);
    let lock = PolicyLockV1 {
        policy_profile_id: policy_profile_id.clone(),
        canonical_hash256: canonical_hash256.clone(),
    };
    let lock_path = root.join("policy.lock.v1");
    fs::write(&lock_path, encode_policy_lock_v1(&lock))?;
    Ok(PolicyLockSyncSummary {
        lock_path,
        policy_profile_id,
        canonical_hash256,
    })
}

pub fn init_cosmos_v1(root: &Path, preset: &str) -> Result<CosmosInitSummary, SdkError> {
    let manifest_path = root.join("Ocl.toml");
    if !manifest_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            manifest_path.display()
        )));
    }
    let manifest = fs::read_to_string(&manifest_path)?;
    let policy_profile_id = parse_manifest_policy_profile_id(&manifest);

    let universes = match preset {
        "ci" => vec![
            UniverseProfileV1 {
                id: "default".to_string(),
                runtime_mode: "deterministic".to_string(),
                engine: "dual".to_string(),
                policy_profile_id: policy_profile_id.clone(),
                audit: "hash_only".to_string(),
                trace: "hash_only".to_string(),
            },
            UniverseProfileV1 {
                id: "ci_locked".to_string(),
                runtime_mode: "deterministic".to_string(),
                engine: "dual".to_string(),
                policy_profile_id,
                audit: "hash_only".to_string(),
                trace: "hash_only".to_string(),
            },
        ],
        "default" => vec![UniverseProfileV1 {
            id: "default".to_string(),
            runtime_mode: "deterministic".to_string(),
            engine: "dual".to_string(),
            policy_profile_id,
            audit: "hash_only".to_string(),
            trace: "hash_only".to_string(),
        }],
        _ => {
            return Err(SdkError::MissingProject(format!(
            "V-COSMOS-PRESET-UNKNOWN: unsupported preset `{preset}` (expected `default` or `ci`)"
        )))
        }
    };

    let cosmos = CosmosSpecV1 {
        universes,
        hive: CosmosHiveV1::default(),
        views: Vec::new(),
        kits: Vec::new(),
    };
    let cosmos_path = root.join("cosmos.toml");
    fs::write(&cosmos_path, encode_cosmos_toml_v1(&cosmos))?;
    Ok(CosmosInitSummary {
        cosmos_path,
        preset: preset.to_string(),
        universes_written: cosmos.universes.len(),
    })
}

pub fn sync_cosmos_lock_v1(
    root: &Path,
    locked: bool,
    signer_id: Option<&str>,
    sign_key: Option<&Path>,
    trust_store: Option<&Path>,
) -> Result<CosmosLockSyncSummary, SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Err(SdkError::LockMismatch(format!(
            "V-COSMOS-MISSING: missing cosmos config `{}` (run `ocl cosmos init <project_dir> --preset ci`)",
            cosmos_path.display()
        )));
    }
    let cosmos = parse_cosmos_toml_v1(&cosmos_path)?;

    if locked {
        if signer_id.is_none() {
            return Err(SdkError::LockMismatch(
                "V-COSMOS-SIGNER-REQUIRED: locked cosmos lock sync requires --signer-id"
                    .to_string(),
            ));
        }
        let Some(sign_key) = sign_key else {
            return Err(SdkError::LockMismatch(
                "V-COSMOS-SIGN-KEY-REQUIRED: locked cosmos lock sync requires --sign-key"
                    .to_string(),
            ));
        };
        if !sign_key.exists() {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-SIGN-KEY-MISSING: sign key not found: {}",
                sign_key.display()
            )));
        }
        let Some(trust_store) = trust_store else {
            return Err(SdkError::LockMismatch(
                "V-COSMOS-TRUST-STORE-REQUIRED: locked cosmos lock sync requires --trust-store"
                    .to_string(),
            ));
        };
        if !trust_store.exists() {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-TRUST-STORE-MISSING: trust store not found: {}",
                trust_store.display()
            )));
        }
        validate_locked_cosmos_bridge_config_v1(root)?;
        verify_policy_lock_matches_manifest_v1(root)?;
    }

    let canonical_hash256 =
        canonical_cosmos_hash256(&cosmos.universes, &cosmos.hive, &cosmos.views, &cosmos.kits);
    let sign_key_hint = sign_key.map(normalize_path);
    let trust_store_hint = trust_store.map(normalize_path);
    let signature_b64 = build_cosmos_signature_b64(
        &canonical_hash256,
        signer_id,
        sign_key_hint.as_deref(),
        trust_store_hint.as_deref(),
    );

    let lock = CosmosLockV1 {
        signer_id: signer_id.map(|v| v.to_string()),
        sign_key_hint,
        trust_store_hint,
        canonical_hash256: canonical_hash256.clone(),
        signature_b64,
        universes: cosmos.universes,
        hive: cosmos.hive,
        views: cosmos.views,
        kits: cosmos.kits,
    };
    let lock_path = root.join("cosmos.lock.v1");
    fs::write(&lock_path, encode_cosmos_lock_v1(&lock))?;
    Ok(CosmosLockSyncSummary {
        lock_path,
        universes_synced: lock.universes.len(),
        canonical_hash256,
    })
}

pub fn resolve_universe_v1(
    root: &Path,
    locked: bool,
    universe_id: Option<&str>,
) -> Result<UniverseSelectionV1, SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        if universe_id.is_some() {
            return Err(SdkError::LockMismatch(
                "V-UNIVERSE-NO-COSMOS: `--universe` requires cosmos.toml".to_string(),
            ));
        }
        return Ok(UniverseSelectionV1 {
            universe_id: "__legacy__".to_string(),
            runtime_mode: "deterministic".to_string(),
            engine: "interpreter".to_string(),
            policy_profile_id: "default".to_string(),
            is_legacy: true,
        });
    }

    let cosmos = parse_cosmos_toml_v1(&cosmos_path)?;
    if cosmos.universes.is_empty() {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-EMPTY: cosmos.toml has no [[universe]] entries".to_string(),
        ));
    }

    let active_universes = if locked {
        verify_policy_lock_matches_manifest_v1(root)?;
        let cosmos_lock = verify_cosmos_lock_matches_config_v1(root, &cosmos)?;
        if universe_id.is_none() {
            return Err(SdkError::LockMismatch(
                "V-UNIVERSE-REQUIRED: locked mode with cosmos requires `--universe <id>`"
                    .to_string(),
            ));
        }
        cosmos_lock.universes
    } else {
        cosmos.universes
    };

    let requested = universe_id.unwrap_or("default");
    let selected = active_universes
        .iter()
        .find(|u| u.id == requested)
        .or_else(|| {
            if universe_id.is_none() {
                active_universes.first()
            } else {
                None
            }
        })
        .ok_or_else(|| {
            SdkError::LockMismatch(format!(
                "V-UNIVERSE-NOT-FOUND: universe `{requested}` is not defined"
            ))
        })?;

    Ok(UniverseSelectionV1 {
        universe_id: selected.id.clone(),
        runtime_mode: selected.runtime_mode.clone(),
        engine: selected.engine.clone(),
        policy_profile_id: selected.policy_profile_id.clone(),
        is_legacy: false,
    })
}

pub fn resolve_hive_caps_v1(
    root: &Path,
    locked: bool,
    universe_id: Option<&str>,
) -> Result<CosmosHiveV1, SdkError> {
    let universe = resolve_universe_v1(root, locked, universe_id)?;
    if universe.is_legacy {
        return Ok(CosmosHiveV1::default());
    }
    let cosmos_path = root.join("cosmos.toml");
    let cosmos = parse_cosmos_toml_v1(&cosmos_path)?;
    if locked {
        verify_policy_lock_matches_manifest_v1(root)?;
        let _ = verify_cosmos_lock_matches_config_v1(root, &cosmos)?;
    }
    Ok(cosmos.hive)
}

pub fn enforce_universe_match_v1(
    selection: &UniverseSelectionV1,
    explicit_runtime_mode: Option<&str>,
    explicit_engine: Option<&str>,
    explicit_policy_profile_id: Option<&str>,
    locked: bool,
) -> Result<(), SdkError> {
    if selection.is_legacy || !locked {
        return Ok(());
    }

    if let Some(runtime_mode) = explicit_runtime_mode {
        if !runtime_mode.eq_ignore_ascii_case(&selection.runtime_mode) {
            return Err(SdkError::LockMismatch(format!(
                "V-UNIVERSE-MISMATCH: runtime_mode=`{runtime_mode}` does not match universe `{}` (`{}`)",
                selection.universe_id, selection.runtime_mode
            )));
        }
    }
    if let Some(engine) = explicit_engine {
        if !engine.eq_ignore_ascii_case(&selection.engine) {
            return Err(SdkError::LockMismatch(format!(
                "V-UNIVERSE-MISMATCH: engine=`{engine}` does not match universe `{}` (`{}`)",
                selection.universe_id, selection.engine
            )));
        }
    }
    if let Some(policy_profile_id) = explicit_policy_profile_id {
        if policy_profile_id != selection.policy_profile_id {
            return Err(SdkError::LockMismatch(format!(
                "V-UNIVERSE-MISMATCH: policy_profile_id=`{policy_profile_id}` does not match universe `{}` (`{}`)",
                selection.universe_id, selection.policy_profile_id
            )));
        }
    }

    Ok(())
}

fn parse_manifest_policy_profile_id(manifest: &str) -> String {
    let mut in_policy = false;
    for raw_line in manifest.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_policy = line == "[policy]";
            continue;
        }
        if !in_policy {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() == "budget_profile" {
            let profile = v.trim().trim_matches('"');
            if !profile.is_empty() {
                return profile.to_string();
            }
        }
    }
    "default".to_string()
}

fn build_policy_canonical_hash256(policy_profile_id: &str) -> String {
    let canonical = format!("version=1\npolicy_profile_id={policy_profile_id}\n");
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

fn encode_policy_lock_v1(lock: &PolicyLockV1) -> String {
    format!(
        concat!(
            "version=1\n",
            "policy_profile_id={}\n",
            "canonical_hash256={}\n"
        ),
        lock.policy_profile_id, lock.canonical_hash256
    )
}

fn parse_policy_lock_v1(path: &Path) -> Result<PolicyLockV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut has_version = false;
    let mut policy_profile_id = None::<String>;
    let mut canonical_hash256 = None::<String>;

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "policy_profile_id" => policy_profile_id = Some(value),
            "canonical_hash256" => canonical_hash256 = Some(value),
            _ => {}
        }
    }

    if !has_version {
        return Err(SdkError::LockMismatch(
            "policy.lock.v1 missing `version=1`".to_string(),
        ));
    }
    let Some(policy_profile_id) = policy_profile_id else {
        return Err(SdkError::LockMismatch(
            "policy.lock.v1 missing `policy_profile_id`".to_string(),
        ));
    };
    let Some(canonical_hash256) = canonical_hash256 else {
        return Err(SdkError::LockMismatch(
            "policy.lock.v1 missing `canonical_hash256`".to_string(),
        ));
    };

    Ok(PolicyLockV1 {
        policy_profile_id,
        canonical_hash256,
    })
}

fn verify_policy_lock_matches_manifest_v1(root: &Path) -> Result<(), SdkError> {
    let manifest_path = root.join("Ocl.toml");
    if !manifest_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            manifest_path.display()
        )));
    }
    let policy_lock_path = root.join("policy.lock.v1");
    if !policy_lock_path.exists() {
        return Err(SdkError::LockMismatch(format!(
            "V-POLICY-LOCK-REQUIRED: missing `{}` (run `ocl policy lock sync <project_dir>`)",
            policy_lock_path.display()
        )));
    }
    let manifest = fs::read_to_string(&manifest_path)?;
    let expected_policy_profile = parse_manifest_policy_profile_id(&manifest);
    let expected_hash = build_policy_canonical_hash256(&expected_policy_profile);
    let lock = parse_policy_lock_v1(&policy_lock_path)?;
    if lock.policy_profile_id != expected_policy_profile {
        return Err(SdkError::LockMismatch(format!(
            "V-POLICY-LOCK-MISMATCH: policy_profile_id lock=`{}` manifest=`{}`",
            lock.policy_profile_id, expected_policy_profile
        )));
    }
    if lock.canonical_hash256 != expected_hash {
        return Err(SdkError::LockMismatch(
            "V-POLICY-LOCK-MISMATCH: canonical_hash256 mismatch".to_string(),
        ));
    }
    Ok(())
}

fn encode_cosmos_toml_v1(cosmos: &CosmosSpecV1) -> String {
    let mut out = String::from("version = 1\n\n");
    out.push_str("[hive]\n");
    out.push_str(&format!(
        "max_swarm_workers = {}\n",
        cosmos.hive.max_swarm_workers
    ));
    out.push_str(&format!(
        "max_fanout_per_task = {}\n",
        cosmos.hive.max_fanout_per_task
    ));
    out.push_str(&format!(
        "mailbox_max_depth = {}\n",
        cosmos.hive.mailbox_max_depth
    ));
    out.push_str(&format!(
        "max_spawn_per_tick = {}\n",
        cosmos.hive.max_spawn_per_tick
    ));
    out.push_str(&format!("max_domains = {}\n", cosmos.hive.max_domains));
    out.push_str(&format!(
        "max_shadow_worlds = {}\n\n",
        cosmos.hive.max_shadow_worlds
    ));
    for universe in &cosmos.universes {
        out.push_str("[[universe]]\n");
        out.push_str(&format!("id = \"{}\"\n", universe.id));
        out.push_str(&format!("runtime_mode = \"{}\"\n", universe.runtime_mode));
        out.push_str(&format!("engine = \"{}\"\n", universe.engine));
        out.push_str(&format!(
            "policy_profile_id = \"{}\"\n",
            universe.policy_profile_id
        ));
        out.push_str(&format!("audit = \"{}\"\n", universe.audit));
        out.push_str(&format!("trace = \"{}\"\n\n", universe.trace));
    }
    out
}

fn parse_cosmos_toml_v1(path: &Path) -> Result<CosmosSpecV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut universes = Vec::new();
    let mut current = UniverseBuilder::default();
    let mut in_universe = false;
    let mut in_hive = false;
    let mut hive = CosmosHiveV1::default();

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[universe]]" {
            if in_universe {
                universes.push(current.build()?);
            }
            current = UniverseBuilder::default();
            in_universe = true;
            in_hive = false;
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_hive = line == "[hive]";
            if in_universe {
                universes.push(current.build()?);
                current = UniverseBuilder::default();
                in_universe = false;
            }
            continue;
        }

        if in_hive {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let key = k.trim();
            let value = v.trim().trim_matches('"');
            if let Ok(parsed) = value.parse::<u32>() {
                match key {
                    "max_swarm_workers" => hive.max_swarm_workers = parsed,
                    "max_fanout_per_task" => hive.max_fanout_per_task = parsed,
                    "mailbox_max_depth" => hive.mailbox_max_depth = parsed,
                    "max_spawn_per_tick" => hive.max_spawn_per_tick = parsed,
                    "max_domains" => hive.max_domains = parsed,
                    "max_shadow_worlds" => hive.max_shadow_worlds = parsed,
                    _ => {}
                }
            }
            continue;
        }

        if !in_universe {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "id" => current.id = Some(value),
            "runtime_mode" => current.runtime_mode = Some(value),
            "engine" => current.engine = Some(value),
            "policy_profile_id" => current.policy_profile_id = Some(value),
            "audit" => current.audit = Some(value),
            "trace" => current.trace = Some(value),
            _ => {}
        }
    }
    if in_universe {
        universes.push(current.build()?);
    }
    if universes.is_empty() {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-EMPTY: cosmos.toml has no [[universe]] entries".to_string(),
        ));
    }

    validate_and_sort_universes(&mut universes)?;
    validate_hive_caps(&hive)?;

    let mut views = parse_cosmos_views_v1(path)?;
    let mut kits = parse_cosmos_kits_v1(path)?;
    sort_views_v1(&mut views);
    sort_kits_v1(&mut kits);

    Ok(CosmosSpecV1 {
        universes,
        hive,
        views,
        kits,
    })
}

fn canonical_cosmos_hash256(
    universes: &[UniverseProfileV1],
    hive: &CosmosHiveV1,
    views: &[ViewProfileV1],
    kits: &[ViewKitBindingV1],
) -> String {
    let mut canonical = String::new();
    canonical.push_str(&format!(
        "hive\t{}\t{}\t{}\t{}\t{}\t{}\n",
        hive.max_swarm_workers,
        hive.max_fanout_per_task,
        hive.mailbox_max_depth,
        hive.max_spawn_per_tick,
        hive.max_domains,
        hive.max_shadow_worlds
    ));
    for universe in universes {
        canonical.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            universe.id,
            universe.runtime_mode,
            universe.engine,
            universe.policy_profile_id,
            universe.audit,
            universe.trace
        ));
    }
    for view in views {
        canonical.push_str(&format!(
            "view\t{}\t{}\t{}\t{}\n",
            view.id, view.universe_id, view.domain_id, view.renderer
        ));
    }
    for kit in kits {
        let required = if kit.required_organs.is_empty() {
            "-".to_string()
        } else {
            kit.required_organs.join(",")
        };
        canonical.push_str(&format!(
            "kit\t{}\t{}\t{}\t{}\t{}\n",
            kit.kit_id, kit.universe_id, kit.bind_domain, kit.bind_view, required
        ));
    }
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

fn build_cosmos_signature_b64(
    canonical_hash256: &str,
    signer_id: Option<&str>,
    sign_key_hint: Option<&str>,
    trust_store_hint: Option<&str>,
) -> String {
    let message = format!(
        "{}|{}|{}|{}",
        canonical_hash256,
        signer_id.unwrap_or("-"),
        sign_key_hint.unwrap_or("-"),
        trust_store_hint.unwrap_or("-")
    );
    let digest = blake3::hash(message.as_bytes());
    B64.encode(digest.as_bytes())
}

fn encode_cosmos_lock_v1(lock: &CosmosLockV1) -> String {
    let mut out = String::new();
    out.push_str("version=1\n");
    out.push_str("signer_id=");
    out.push_str(lock.signer_id.as_deref().unwrap_or("-"));
    out.push('\n');
    out.push_str("sign_key_hint=");
    out.push_str(lock.sign_key_hint.as_deref().unwrap_or("-"));
    out.push('\n');
    out.push_str("trust_store_hint=");
    out.push_str(lock.trust_store_hint.as_deref().unwrap_or("-"));
    out.push('\n');
    out.push_str("canonical_hash256=");
    out.push_str(&lock.canonical_hash256);
    out.push('\n');
    out.push_str("signature_b64=");
    out.push_str(&lock.signature_b64);
    out.push('\n');
    out.push_str("hive=");
    out.push_str(&lock.hive.max_swarm_workers.to_string());
    out.push('|');
    out.push_str(&lock.hive.max_fanout_per_task.to_string());
    out.push('|');
    out.push_str(&lock.hive.mailbox_max_depth.to_string());
    out.push('|');
    out.push_str(&lock.hive.max_spawn_per_tick.to_string());
    out.push('|');
    out.push_str(&lock.hive.max_domains.to_string());
    out.push('|');
    out.push_str(&lock.hive.max_shadow_worlds.to_string());
    out.push('\n');
    for universe in &lock.universes {
        out.push_str("universe=");
        out.push_str(&universe.id);
        out.push('|');
        out.push_str(&universe.runtime_mode);
        out.push('|');
        out.push_str(&universe.engine);
        out.push('|');
        out.push_str(&universe.policy_profile_id);
        out.push('|');
        out.push_str(&universe.audit);
        out.push('|');
        out.push_str(&universe.trace);
        out.push('\n');
    }
    for view in &lock.views {
        out.push_str("view=");
        out.push_str(&view.id);
        out.push('|');
        out.push_str(&view.universe_id);
        out.push('|');
        out.push_str(&view.domain_id);
        out.push('|');
        out.push_str(&view.renderer);
        out.push('\n');
    }
    for kit in &lock.kits {
        let required = if kit.required_organs.is_empty() {
            "-".to_string()
        } else {
            kit.required_organs.join(",")
        };
        out.push_str("kit=");
        out.push_str(&kit.kit_id);
        out.push('|');
        out.push_str(&kit.universe_id);
        out.push('|');
        out.push_str(&kit.bind_domain);
        out.push('|');
        out.push_str(&kit.bind_view);
        out.push('|');
        out.push_str(&required);
        out.push('\n');
    }
    out
}

fn parse_cosmos_lock_v1(path: &Path) -> Result<CosmosLockV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut has_version = false;
    let mut signer_id = None::<String>;
    let mut sign_key_hint = None::<String>;
    let mut trust_store_hint = None::<String>;
    let mut canonical_hash256 = None::<String>;
    let mut signature_b64 = None::<String>;
    let mut hive = None::<CosmosHiveV1>;
    let mut universes = Vec::new();
    let mut views = Vec::new();
    let mut kits = Vec::new();

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        if let Some(payload) = line.strip_prefix("universe=") {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() != 6 {
                return Err(SdkError::LockMismatch(
                    "V-COSMOS-LOCK-INVALID: `universe=` row must have 6 columns".to_string(),
                ));
            }
            universes.push(UniverseProfileV1 {
                id: parts[0].to_string(),
                runtime_mode: parts[1].to_string(),
                engine: parts[2].to_string(),
                policy_profile_id: parts[3].to_string(),
                audit: parts[4].to_string(),
                trace: parts[5].to_string(),
            });
            continue;
        }
        if let Some(payload) = line.strip_prefix("hive=") {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() != 6 {
                return Err(SdkError::LockMismatch(
                    "V-HIVE-LOCK-INVALID: `hive=` row must have 6 columns".to_string(),
                ));
            }
            let parse_u32 = |raw: &str| -> Result<u32, SdkError> {
                raw.parse::<u32>().map_err(|_| {
                    SdkError::LockMismatch(
                        "V-HIVE-LOCK-INVALID: hive values must be u32".to_string(),
                    )
                })
            };
            hive = Some(CosmosHiveV1 {
                max_swarm_workers: parse_u32(parts[0])?,
                max_fanout_per_task: parse_u32(parts[1])?,
                mailbox_max_depth: parse_u32(parts[2])?,
                max_spawn_per_tick: parse_u32(parts[3])?,
                max_domains: parse_u32(parts[4])?,
                max_shadow_worlds: parse_u32(parts[5])?,
            });
            continue;
        }
        if let Some(payload) = line.strip_prefix("view=") {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() != 4 {
                return Err(SdkError::LockMismatch(
                    "V-VIEW-LOCK-INVALID: `view=` row must have 4 columns".to_string(),
                ));
            }
            views.push(ViewProfileV1 {
                id: parts[0].to_string(),
                universe_id: parts[1].to_string(),
                domain_id: parts[2].to_string(),
                renderer: parts[3].to_string(),
            });
            continue;
        }
        if let Some(payload) = line.strip_prefix("kit=") {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() != 4 && parts.len() != 5 {
                return Err(SdkError::LockMismatch(
                    "V-KIT-LOCK-INVALID: `kit=` row must have 4 or 5 columns".to_string(),
                ));
            }
            let mut required_organs = if parts.len() == 5 {
                parse_inline_required_organs_v1(parts[4])
            } else {
                Vec::new()
            };
            required_organs.sort();
            required_organs.dedup();
            kits.push(ViewKitBindingV1 {
                kit_id: parts[0].to_string(),
                universe_id: parts[1].to_string(),
                bind_domain: parts[2].to_string(),
                bind_view: parts[3].to_string(),
                required_organs,
            });
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().to_string();
        match key {
            "signer_id" if value != "-" => signer_id = Some(value),
            "sign_key_hint" if value != "-" => sign_key_hint = Some(value),
            "trust_store_hint" if value != "-" => trust_store_hint = Some(value),
            "canonical_hash256" => canonical_hash256 = Some(value),
            "signature_b64" => signature_b64 = Some(value),
            _ => {}
        }
    }

    if !has_version {
        return Err(SdkError::LockMismatch(
            "cosmos.lock.v1 missing `version=1`".to_string(),
        ));
    }
    if universes.is_empty() {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-EMPTY: cosmos.lock.v1 has no `universe=` rows".to_string(),
        ));
    }
    let Some(hive) = hive else {
        return Err(SdkError::LockMismatch(
            "V-HIVE-LOCK-MISSING: cosmos.lock.v1 missing `hive=` row".to_string(),
        ));
    };
    validate_hive_caps(&hive)?;
    validate_and_sort_universes(&mut universes)?;
    sort_views_v1(&mut views);
    sort_kits_v1(&mut kits);

    let Some(canonical_hash256) = canonical_hash256 else {
        return Err(SdkError::LockMismatch(
            "cosmos.lock.v1 missing `canonical_hash256`".to_string(),
        ));
    };
    let Some(signature_b64) = signature_b64 else {
        return Err(SdkError::LockMismatch(
            "cosmos.lock.v1 missing `signature_b64`".to_string(),
        ));
    };

    Ok(CosmosLockV1 {
        signer_id,
        sign_key_hint,
        trust_store_hint,
        canonical_hash256,
        signature_b64,
        universes,
        hive,
        views,
        kits,
    })
}

fn verify_cosmos_lock_matches_config_v1(
    root: &Path,
    cosmos: &CosmosSpecV1,
) -> Result<CosmosLockV1, SdkError> {
    let lock_path = root.join("cosmos.lock.v1");
    if !lock_path.exists() {
        return Err(SdkError::LockMismatch(format!(
            "V-COSMOS-LOCK-REQUIRED: missing `{}` (run `ocl cosmos lock sync <project_dir> --locked ...`)",
            lock_path.display()
        )));
    }
    let lock = parse_cosmos_lock_v1(&lock_path)?;
    if lock.universes != cosmos.universes {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-MISMATCH: cosmos.lock.v1 differs from cosmos.toml".to_string(),
        ));
    }
    if lock.views != cosmos.views {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-MISMATCH: view wiring in cosmos.lock.v1 differs from cosmos.toml"
                .to_string(),
        ));
    }
    if lock.kits != cosmos.kits {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-MISMATCH: kit wiring in cosmos.lock.v1 differs from cosmos.toml"
                .to_string(),
        ));
    }
    let expected_hash256 =
        canonical_cosmos_hash256(&cosmos.universes, &cosmos.hive, &cosmos.views, &cosmos.kits);
    if lock.canonical_hash256 != expected_hash256 {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-MISMATCH: canonical_hash256 mismatch".to_string(),
        ));
    }
    if lock.hive != cosmos.hive {
        return Err(SdkError::LockMismatch(
            "V-HIVE-LOCK-MISMATCH: hive section in cosmos.lock.v1 differs from cosmos.toml"
                .to_string(),
        ));
    }
    let expected_sig = build_cosmos_signature_b64(
        &lock.canonical_hash256,
        lock.signer_id.as_deref(),
        lock.sign_key_hint.as_deref(),
        lock.trust_store_hint.as_deref(),
    );
    if lock.signature_b64 != expected_sig {
        return Err(SdkError::LockMismatch(
            "V-COSMOS-LOCK-SIGNATURE: signature_b64 mismatch".to_string(),
        ));
    }
    Ok(lock)
}

fn validate_and_sort_universes(universes: &mut Vec<UniverseProfileV1>) -> Result<(), SdkError> {
    for universe in universes.iter_mut() {
        if universe.id.is_empty() {
            return Err(SdkError::LockMismatch(
                "V-COSMOS-UNIVERSE-ID-EMPTY: universe id must not be empty".to_string(),
            ));
        }
        if !matches!(
            universe.runtime_mode.as_str(),
            "deterministic" | "throughput"
        ) {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-RUNTIME-INVALID: universe `{}` runtime_mode must be deterministic|throughput",
                universe.id
            )));
        }
        if !matches!(
            universe.engine.as_str(),
            "interpreter" | "bytecode" | "dual"
        ) {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-ENGINE-INVALID: universe `{}` engine must be interpreter|bytecode|dual",
                universe.id
            )));
        }
        if universe.policy_profile_id.is_empty() {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-POLICY-EMPTY: universe `{}` policy_profile_id must not be empty",
                universe.id
            )));
        }
        if universe.audit.is_empty() {
            universe.audit = "hash_only".to_string();
        }
        if universe.trace.is_empty() {
            universe.trace = "hash_only".to_string();
        }
    }

    universes.sort_by(|a, b| a.id.cmp(&b.id));
    let mut seen = HashSet::new();
    for universe in universes {
        if !seen.insert(universe.id.clone()) {
            return Err(SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-DUPLICATE: duplicate universe id `{}`",
                universe.id
            )));
        }
    }
    Ok(())
}

fn validate_hive_caps(hive: &CosmosHiveV1) -> Result<(), SdkError> {
    if hive.max_swarm_workers == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: max_swarm_workers must be > 0".to_string(),
        ));
    }
    if hive.max_fanout_per_task == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: max_fanout_per_task must be > 0".to_string(),
        ));
    }
    if hive.mailbox_max_depth == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: mailbox_max_depth must be > 0".to_string(),
        ));
    }
    if hive.max_spawn_per_tick == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: max_spawn_per_tick must be > 0".to_string(),
        ));
    }
    if hive.max_domains == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: max_domains must be > 0".to_string(),
        ));
    }
    if hive.max_shadow_worlds == 0 {
        return Err(SdkError::LockMismatch(
            "V-HIVE-CAP-INVALID: max_shadow_worlds must be > 0".to_string(),
        ));
    }
    Ok(())
}

fn sort_views_v1(views: &mut [ViewProfileV1]) {
    views.sort_by(|a, b| {
        a.universe_id
            .cmp(&b.universe_id)
            .then(a.domain_id.cmp(&b.domain_id))
            .then(a.id.cmp(&b.id))
            .then(a.renderer.cmp(&b.renderer))
    });
}

fn sort_kits_v1(kits: &mut [ViewKitBindingV1]) {
    for kit in kits.iter_mut() {
        kit.required_organs.sort();
        kit.required_organs.dedup();
    }
    kits.sort_by(|a, b| {
        a.kit_id
            .cmp(&b.kit_id)
            .then(a.universe_id.cmp(&b.universe_id))
            .then(a.bind_domain.cmp(&b.bind_domain))
            .then(a.bind_view.cmp(&b.bind_view))
            .then(
                a.required_organs
                    .join(",")
                    .cmp(&b.required_organs.join(",")),
            )
    });
}

fn parse_inline_required_organs_v1(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() || value == "-" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Default)]
struct UniverseBuilder {
    id: Option<String>,
    runtime_mode: Option<String>,
    engine: Option<String>,
    policy_profile_id: Option<String>,
    audit: Option<String>,
    trace: Option<String>,
}

impl UniverseBuilder {
    fn build(self) -> Result<UniverseProfileV1, SdkError> {
        let id = self.id.ok_or_else(|| {
            SdkError::LockMismatch(
                "V-COSMOS-UNIVERSE-ID-MISSING: universe is missing `id`".to_string(),
            )
        })?;
        let runtime_mode = self.runtime_mode.ok_or_else(|| {
            SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-RUNTIME-MISSING: universe `{id}` is missing `runtime_mode`"
            ))
        })?;
        let engine = self.engine.ok_or_else(|| {
            SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-ENGINE-MISSING: universe `{id}` is missing `engine`"
            ))
        })?;
        let policy_profile_id = self.policy_profile_id.ok_or_else(|| {
            SdkError::LockMismatch(format!(
                "V-COSMOS-UNIVERSE-POLICY-MISSING: universe `{id}` is missing `policy_profile_id`"
            ))
        })?;
        Ok(UniverseProfileV1 {
            id,
            runtime_mode,
            engine,
            policy_profile_id,
            audit: self.audit.unwrap_or_else(|| "hash_only".to_string()),
            trace: self.trace.unwrap_or_else(|| "hash_only".to_string()),
        })
    }
}
