use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use crate::w1::resolve_universe_v1;
use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainProfileV1 {
    pub id: String,
    pub universe_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainSelectionV1 {
    pub universe_id: String,
    pub domain_id: String,
    pub is_legacy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRuntimeRuleV1 {
    pub bridge_id: String,
    pub universe_id: String,
    pub from_domain: String,
    pub to_domain: String,
    pub emit_quota_per_tick: u32,
    pub dispatch_max_per_tick: u32,
    pub queue_capacity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRuntimePlanV1 {
    pub universe_id: String,
    pub domain_id: String,
    pub domains: Vec<DomainProfileV1>,
    pub bridges: Vec<BridgeRuntimeRuleV1>,
    pub dispatch_start_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeEnvelopeV1 {
    pub seq: u64,
    pub payload_hash256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeRuntimeStateV1 {
    pub tick_epoch: u64,
    pub admit_count: u32,
    pub dispatch_count: u32,
    pub next_seq: u64,
    pub queue: VecDeque<BridgeEnvelopeV1>,
}

pub fn resolve_domain_selection_v1(
    root: &Path,
    locked: bool,
    universe_id: Option<&str>,
    domain_id: Option<&str>,
) -> Result<DomainSelectionV1, SdkError> {
    let universe = resolve_universe_v1(root, locked, universe_id)?;
    if universe.is_legacy {
        if let Some(requested) = domain_id {
            if requested != "default" {
                return Err(SdkError::LockMismatch(
                    "V-DOMAIN-NO-COSMOS: `--domain` requires cosmos.toml".to_string(),
                ));
            }
        }
        return Ok(DomainSelectionV1 {
            universe_id: "__legacy__".to_string(),
            domain_id: "default".to_string(),
            is_legacy: true,
        });
    }

    let cosmos_path = root.join("cosmos.toml");
    let parsed = parse_cosmos_domain_bridge_v1(&cosmos_path)?;
    let mut domains: Vec<DomainProfileV1> = parsed
        .domains
        .into_iter()
        .filter(|d| d.universe_id == universe.universe_id)
        .collect();
    if domains.is_empty() {
        domains.push(DomainProfileV1 {
            id: "default".to_string(),
            universe_id: universe.universe_id.clone(),
        });
    }
    domains.sort_by(|a, b| a.id.cmp(&b.id));

    let selected_domain = match domain_id {
        Some(requested) => {
            if domains.iter().any(|d| d.id == requested) {
                requested.to_string()
            } else {
                return Err(SdkError::LockMismatch(format!(
                    "V-DOMAIN-NOT-FOUND: domain `{requested}` is not defined for universe `{}`",
                    universe.universe_id
                )));
            }
        }
        None => {
            if domains.iter().any(|d| d.id == "default") {
                "default".to_string()
            } else {
                return Err(SdkError::LockMismatch(
                    "V-DOMAIN-REQUIRED: multiple domains configured without `default`; pass `--domain <id>`".to_string(),
                ));
            }
        }
    };

    Ok(DomainSelectionV1 {
        universe_id: universe.universe_id,
        domain_id: selected_domain,
        is_legacy: false,
    })
}

pub fn resolve_bridge_runtime_plan_v1(
    root: &Path,
    locked: bool,
    universe_id: Option<&str>,
    domain_id: Option<&str>,
    tick: u64,
) -> Result<BridgeRuntimePlanV1, SdkError> {
    let selection = resolve_domain_selection_v1(root, locked, universe_id, domain_id)?;
    if selection.is_legacy {
        return Ok(BridgeRuntimePlanV1 {
            universe_id: selection.universe_id,
            domain_id: selection.domain_id,
            domains: vec![DomainProfileV1 {
                id: "default".to_string(),
                universe_id: "__legacy__".to_string(),
            }],
            bridges: Vec::new(),
            dispatch_start_index: 0,
        });
    }

    let cosmos_path = root.join("cosmos.toml");
    let parsed = parse_cosmos_domain_bridge_v1(&cosmos_path)?;
    let mut domains: Vec<DomainProfileV1> = parsed
        .domains
        .into_iter()
        .filter(|d| d.universe_id == selection.universe_id)
        .collect();
    if domains.is_empty() {
        domains.push(DomainProfileV1 {
            id: "default".to_string(),
            universe_id: selection.universe_id.clone(),
        });
    }
    domains.sort_by(|a, b| a.id.cmp(&b.id));
    let domain_set: HashSet<String> = domains.iter().map(|d| d.id.clone()).collect();

    let mut bridges = Vec::new();
    for bridge in parsed.bridges {
        if bridge.universe_id != selection.universe_id {
            continue;
        }
        if !domain_set.contains(&bridge.from_domain) || !domain_set.contains(&bridge.to_domain) {
            continue;
        }
        if bridge.queue_capacity == 0 {
            if locked {
                return Err(SdkError::LockMismatch(format!(
                    "V-BRIDGE-QUEUE-CAPACITY-REQUIRED: bridge `{}` must declare bridge_queue_capacity in locked mode",
                    bridge.bridge_id
                )));
            }
            continue;
        }
        bridges.push(bridge);
    }
    bridges.sort_by(|a, b| a.bridge_id.cmp(&b.bridge_id));

    let dispatch_start_index = if bridges.is_empty() {
        0
    } else {
        (tick as usize) % bridges.len()
    };
    Ok(BridgeRuntimePlanV1 {
        universe_id: selection.universe_id,
        domain_id: selection.domain_id,
        domains,
        bridges,
        dispatch_start_index,
    })
}

pub fn admit_bridge_emit_v1(
    state: &mut BridgeRuntimeStateV1,
    rule: &BridgeRuntimeRuleV1,
    tick: u64,
    payload_hash256: &str,
) -> Result<u64, SdkError> {
    reset_epoch_if_needed(state, tick);
    if state.admit_count >= rule.emit_quota_per_tick {
        return Err(SdkError::LockMismatch(format!(
            "V-DOMAIN-BRIDGE-CAP: emit quota exceeded for bridge `{}`",
            rule.bridge_id
        )));
    }
    if state.queue.len() >= rule.queue_capacity as usize {
        return Err(SdkError::LockMismatch(format!(
            "V-DOMAIN-BRIDGE-BACKPRESSURE: queue full for bridge `{}`",
            rule.bridge_id
        )));
    }
    state.next_seq = state.next_seq.saturating_add(1);
    state.admit_count = state.admit_count.saturating_add(1);
    state.queue.push_back(BridgeEnvelopeV1 {
        seq: state.next_seq,
        payload_hash256: payload_hash256.to_string(),
    });
    Ok(state.next_seq)
}

pub fn poll_bridge_event_v1(
    state: &mut BridgeRuntimeStateV1,
    rule: &BridgeRuntimeRuleV1,
    tick: u64,
) -> Option<BridgeEnvelopeV1> {
    reset_epoch_if_needed(state, tick);
    if state.dispatch_count >= rule.dispatch_max_per_tick {
        return None;
    }
    let next = state.queue.pop_front();
    if next.is_some() {
        state.dispatch_count = state.dispatch_count.saturating_add(1);
    }
    next
}

pub fn validate_locked_cosmos_bridge_config_v1(root: &Path) -> Result<(), SdkError> {
    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Ok(());
    }
    let parsed = parse_cosmos_domain_bridge_v1(&cosmos_path)?;
    for bridge in parsed.bridges {
        if bridge.queue_capacity == 0 {
            return Err(SdkError::LockMismatch(format!(
                "V-BRIDGE-QUEUE-CAPACITY-REQUIRED: bridge `{}` must declare bridge_queue_capacity in cosmos.toml for locked mode",
                bridge.bridge_id
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedCosmosV1 {
    domains: Vec<DomainProfileV1>,
    bridges: Vec<BridgeRuntimeRuleV1>,
}

#[derive(Debug, Default)]
struct DomainBuilder {
    id: Option<String>,
    universe_id: Option<String>,
}

#[derive(Debug, Default)]
struct BridgeBuilder {
    bridge_id: Option<String>,
    universe_id: Option<String>,
    from_domain: Option<String>,
    to_domain: Option<String>,
    emit_quota_per_tick: Option<u32>,
    dispatch_max_per_tick: Option<u32>,
    queue_capacity: Option<u32>,
}

fn parse_cosmos_domain_bridge_v1(path: &Path) -> Result<ParsedCosmosV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut domains = Vec::new();
    let mut bridges = Vec::new();

    let mut section = String::new();
    let mut domain_builder = DomainBuilder::default();
    let mut bridge_builder = BridgeBuilder::default();

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line == "[[domain]]" {
            if section == "domain" {
                domains.push(build_domain(domain_builder)?);
                domain_builder = DomainBuilder::default();
            } else if section == "bridge" {
                bridges.push(build_bridge(bridge_builder)?);
                bridge_builder = BridgeBuilder::default();
            }
            section = "domain".to_string();
            continue;
        }
        if line == "[[bridge]]" {
            if section == "domain" {
                domains.push(build_domain(domain_builder)?);
                domain_builder = DomainBuilder::default();
            } else if section == "bridge" {
                bridges.push(build_bridge(bridge_builder)?);
                bridge_builder = BridgeBuilder::default();
            }
            section = "bridge".to_string();
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if section == "domain" {
                domains.push(build_domain(domain_builder)?);
                domain_builder = DomainBuilder::default();
            } else if section == "bridge" {
                bridges.push(build_bridge(bridge_builder)?);
                bridge_builder = BridgeBuilder::default();
            }
            section.clear();
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"');

        if section == "domain" {
            match key {
                "id" => domain_builder.id = Some(value.to_string()),
                "universe_id" => domain_builder.universe_id = Some(value.to_string()),
                _ => {}
            }
            continue;
        }
        if section == "bridge" {
            match key {
                "id" => bridge_builder.bridge_id = Some(value.to_string()),
                "universe_id" => bridge_builder.universe_id = Some(value.to_string()),
                "from_domain" => bridge_builder.from_domain = Some(value.to_string()),
                "to_domain" => bridge_builder.to_domain = Some(value.to_string()),
                "emit_quota_per_tick" => {
                    bridge_builder.emit_quota_per_tick = parse_u32(v.trim());
                }
                "dispatch_max_per_tick" => {
                    bridge_builder.dispatch_max_per_tick = parse_u32(v.trim());
                }
                "bridge_queue_capacity" => {
                    bridge_builder.queue_capacity = parse_u32(v.trim());
                }
                _ => {}
            }
        }
    }

    if section == "domain" {
        domains.push(build_domain(domain_builder)?);
    } else if section == "bridge" {
        bridges.push(build_bridge(bridge_builder)?);
    }

    validate_uniqueness(&domains, &bridges)?;
    Ok(ParsedCosmosV1 { domains, bridges })
}

fn build_domain(builder: DomainBuilder) -> Result<DomainProfileV1, SdkError> {
    let id = builder.id.ok_or_else(|| {
        SdkError::LockMismatch("V-DOMAIN-ID-MISSING: domain is missing `id`".to_string())
    })?;
    if id.is_empty() {
        return Err(SdkError::LockMismatch(
            "V-DOMAIN-ID-EMPTY: domain id must not be empty".to_string(),
        ));
    }
    let universe_id = builder.universe_id.unwrap_or_else(|| "default".to_string());
    Ok(DomainProfileV1 { id, universe_id })
}

fn build_bridge(builder: BridgeBuilder) -> Result<BridgeRuntimeRuleV1, SdkError> {
    let bridge_id = builder.bridge_id.ok_or_else(|| {
        SdkError::LockMismatch("V-BRIDGE-ID-MISSING: bridge is missing `id`".to_string())
    })?;
    let from_domain = builder.from_domain.ok_or_else(|| {
        SdkError::LockMismatch(format!(
            "V-BRIDGE-FROM-MISSING: bridge `{bridge_id}` is missing `from_domain`"
        ))
    })?;
    let to_domain = builder.to_domain.ok_or_else(|| {
        SdkError::LockMismatch(format!(
            "V-BRIDGE-TO-MISSING: bridge `{bridge_id}` is missing `to_domain`"
        ))
    })?;
    let emit_quota_per_tick = builder.emit_quota_per_tick.unwrap_or(1).max(1);
    let dispatch_max_per_tick = builder.dispatch_max_per_tick.unwrap_or(1).max(1);
    let queue_capacity = builder.queue_capacity.unwrap_or(0);
    Ok(BridgeRuntimeRuleV1 {
        bridge_id,
        universe_id: builder.universe_id.unwrap_or_else(|| "default".to_string()),
        from_domain,
        to_domain,
        emit_quota_per_tick,
        dispatch_max_per_tick,
        queue_capacity,
    })
}

fn parse_u32(raw: &str) -> Option<u32> {
    raw.trim().trim_matches('"').parse::<u32>().ok()
}

fn validate_uniqueness(
    domains: &[DomainProfileV1],
    bridges: &[BridgeRuntimeRuleV1],
) -> Result<(), SdkError> {
    let mut domain_seen: HashSet<(String, String)> = HashSet::new();
    for d in domains {
        let key = (d.universe_id.clone(), d.id.clone());
        if !domain_seen.insert(key) {
            return Err(SdkError::LockMismatch(format!(
                "V-DOMAIN-DUPLICATE: duplicate domain `{}` in universe `{}`",
                d.id, d.universe_id
            )));
        }
    }

    let mut bridge_seen: HashMap<String, String> = HashMap::new();
    for b in bridges {
        if let Some(prev_universe) = bridge_seen.insert(b.bridge_id.clone(), b.universe_id.clone())
        {
            return Err(SdkError::LockMismatch(format!(
                "V-BRIDGE-DUPLICATE: duplicate bridge id `{}` in universes `{}` and `{}`",
                b.bridge_id, prev_universe, b.universe_id
            )));
        }
    }
    Ok(())
}

fn reset_epoch_if_needed(state: &mut BridgeRuntimeStateV1, tick: u64) {
    if state.tick_epoch != tick {
        state.tick_epoch = tick;
        state.admit_count = 0;
        state.dispatch_count = 0;
    }
}
