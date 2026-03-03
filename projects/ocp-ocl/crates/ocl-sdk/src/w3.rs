use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ocl_runtime_core::{CommitPolicyMode, ExecConfig, RunEngine};

use crate::{
    run_project_with_trace_engine_config_and_lock, run_reactor_service_with_lock,
    run_reactor_service_with_trace_engine_config_and_lock, ReactorServiceOptions,
    ReactorServiceReport, SdkError, TraceEventV1,
};

pub const SHADOW_EMPTY_COMMIT_HASH256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowPolicyV1 {
    ForbidCommit,
    ShadowCommitLog,
}

impl ShadowPolicyV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ForbidCommit => "forbid_commit",
            Self::ShadowCommitLog => "shadow_commit_log",
        }
    }
}

fn shadow_commit_policy_mode(policy: ShadowPolicyV1) -> CommitPolicyMode {
    match policy {
        ShadowPolicyV1::ForbidCommit => CommitPolicyMode::ForbidCommit,
        ShadowPolicyV1::ShadowCommitLog => CommitPolicyMode::ShadowCommitLog,
    }
}

pub fn parse_shadow_policy_v1(raw: &str) -> Option<ShadowPolicyV1> {
    match raw {
        "forbid_commit" => Some(ShadowPolicyV1::ForbidCommit),
        "shadow_commit_log" => Some(ShadowPolicyV1::ShadowCommitLog),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowOptionsV1 {
    pub shadow_id: String,
    pub policy: ShadowPolicyV1,
    pub report_dir: Option<PathBuf>,
}

impl Default for ShadowOptionsV1 {
    fn default() -> Self {
        Self {
            shadow_id: "parity".to_string(),
            policy: ShadowPolicyV1::ForbidCommit,
            report_dir: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEnvelopeV1 {
    pub universe_id: String,
    pub domain_id: String,
    pub shadow_id: String,
    pub runtime_mode: String,
    pub engine: String,
    pub run_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowTranscriptEventV1 {
    pub logical_tick: u64,
    pub event_id: u64,
    pub actor_id: String,
    pub args_hash256: String,
    pub decision_outcome: String,
    pub commit_intents_hash256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowCompareReportV1 {
    pub shadow_id: String,
    pub policy: ShadowPolicyV1,
    pub main_required_digest: String,
    pub shadow_required_digest: String,
    pub event_count_main: usize,
    pub event_count_shadow: usize,
    pub matched: bool,
    pub unsupported_reason: Option<String>,
    pub mismatch_reason: Option<String>,
    pub main_transcript: Vec<ShadowTranscriptEventV1>,
    pub shadow_transcript: Vec<ShadowTranscriptEventV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowArtifactPathsV1 {
    pub report_path: PathBuf,
    pub main_transcript_path: PathBuf,
    pub shadow_transcript_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowRunSummaryV1 {
    pub steps: u32,
    pub compare: ShadowCompareReportV1,
    pub artifacts: ShadowArtifactPathsV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowReactorRunSummaryV1 {
    pub reactor: ReactorServiceReport,
    pub compare: ShadowCompareReportV1,
    pub artifacts: ShadowArtifactPathsV1,
}

fn hash256_hex(input: &[u8]) -> String {
    blake3::hash(input).to_hex().to_string()
}

fn nondeterministic_reason(events: &[TraceEventV1]) -> Option<String> {
    for event in events {
        let Some(key) = event.key.as_deref() else {
            continue;
        };
        if key.starts_with("std.time.")
            || key.starts_with("std.clock.")
            || key.starts_with("std.rand.")
            || key.starts_with("std.random.")
            || key.starts_with("std.uuid.")
        {
            return Some(format!(
                "V-SHADOW-UNSUPPORTED: nondeterministic source `{key}`"
            ));
        }
    }
    None
}

pub fn build_commit_intent_hash256(events: &[TraceEventV1], policy: ShadowPolicyV1) -> String {
    if matches!(policy, ShadowPolicyV1::ForbidCommit) {
        return SHADOW_EMPTY_COMMIT_HASH256.to_string();
    }

    let mut observe_by_origin: HashMap<u64, (&str, &str)> = HashMap::new();
    for event in events {
        if event.event != "observe_end" {
            continue;
        }
        let Some(origin_id) = event.origin_id else {
            continue;
        };
        let key = event.key.as_deref().unwrap_or("-");
        let payload_hash = event.payload_hash.as_str();
        observe_by_origin.insert(origin_id, (key, payload_hash));
    }

    let mut rows = Vec::new();
    for event in events {
        if event.event != "commit_attempt" {
            continue;
        }
        let seq = event.seq;
        let origin_id = event.origin_id.unwrap_or(0);
        let (effect_key, observed_payload_hash) = observe_by_origin
            .get(&origin_id)
            .copied()
            .unwrap_or(("-", "-"));
        let args_hash256 = hash256_hex(observed_payload_hash.as_bytes());
        let target_hash256 = hash256_hex(origin_id.to_string().as_bytes());
        let kind = event.kind.as_deref().unwrap_or("unknown");
        rows.push(format!(
            "{effect_key}|{args_hash256}|{target_hash256}|{kind}|{seq}"
        ));
    }

    if rows.is_empty() {
        return SHADOW_EMPTY_COMMIT_HASH256.to_string();
    }

    rows.sort();
    let joined = rows.join("\n");
    hash256_hex(joined.as_bytes())
}

fn decision_outcome(event: &TraceEventV1, policy: ShadowPolicyV1) -> String {
    if matches!(policy, ShadowPolicyV1::ForbidCommit) && event.event == "commit_result" {
        return "commit_result|masked|masked|masked|masked".to_string();
    }
    format!(
        "{}|{}|{}|{}|{}",
        event.event,
        event.kind.as_deref().unwrap_or("-"),
        event.reason.as_deref().unwrap_or("-"),
        match event.allowed {
            Some(true) => "allow",
            Some(false) => "deny",
            None => "-",
        },
        match event.value {
            Some(true) => "true",
            Some(false) => "false",
            None => "-",
        }
    )
}

fn transcript_args_hash(event: &TraceEventV1, policy: ShadowPolicyV1) -> String {
    if matches!(policy, ShadowPolicyV1::ForbidCommit) && event.event == "commit_result" {
        return hash256_hex(b"commit_result|masked");
    }
    hash256_hex(event.payload_hash.as_bytes())
}

pub fn build_shadow_transcript_v1(
    events: &[TraceEventV1],
    envelope: &InputEnvelopeV1,
    policy: ShadowPolicyV1,
) -> Vec<ShadowTranscriptEventV1> {
    let commit_hash = build_commit_intent_hash256(events, policy);
    let actor_id = format!(
        "{}:{}:{}",
        envelope.universe_id, envelope.domain_id, envelope.shadow_id
    );

    let mut out = Vec::with_capacity(events.len());
    for event in events {
        out.push(ShadowTranscriptEventV1 {
            logical_tick: event.seq,
            event_id: event.seq,
            actor_id: actor_id.clone(),
            args_hash256: transcript_args_hash(event, policy),
            decision_outcome: decision_outcome(event, policy),
            commit_intents_hash256: commit_hash.clone(),
        });
    }
    out
}

pub fn build_shadow_required_digest(
    envelope: &InputEnvelopeV1,
    transcript: &[ShadowTranscriptEventV1],
) -> String {
    let mut payload = String::new();
    payload.push_str("shadow.v1\n");
    payload.push_str("universe_id=");
    payload.push_str(&envelope.universe_id);
    payload.push('\n');
    payload.push_str("domain_id=");
    payload.push_str(&envelope.domain_id);
    payload.push('\n');
    payload.push_str("shadow_id=");
    payload.push_str(&envelope.shadow_id);
    payload.push('\n');
    payload.push_str("runtime_mode=");
    payload.push_str(&envelope.runtime_mode);
    payload.push('\n');
    payload.push_str("engine=");
    payload.push_str(&envelope.engine);
    payload.push('\n');
    payload.push_str("run_kind=");
    payload.push_str(&envelope.run_kind);
    payload.push('\n');

    for event in transcript {
        payload.push_str(&event.logical_tick.to_string());
        payload.push('|');
        payload.push_str(&event.event_id.to_string());
        payload.push('|');
        payload.push_str(&event.actor_id);
        payload.push('|');
        payload.push_str(&event.args_hash256);
        payload.push('|');
        payload.push_str(&event.decision_outcome);
        payload.push('|');
        payload.push_str(&event.commit_intents_hash256);
        payload.push('\n');
    }

    hash256_hex(payload.as_bytes())
}

pub fn compare_shadow_traces_v1(
    main_events: &[TraceEventV1],
    shadow_events: &[TraceEventV1],
    envelope: &InputEnvelopeV1,
    policy: ShadowPolicyV1,
) -> Result<ShadowCompareReportV1, SdkError> {
    if envelope.runtime_mode != "deterministic" {
        return Err(SdkError::ShadowUnsupported(
            "V-SHADOW-UNSUPPORTED: shadow compare chỉ hỗ trợ runtime deterministic".to_string(),
        ));
    }

    if let Some(reason) = nondeterministic_reason(main_events) {
        return Err(SdkError::ShadowUnsupported(reason));
    }
    if let Some(reason) = nondeterministic_reason(shadow_events) {
        return Err(SdkError::ShadowUnsupported(reason));
    }

    let main_transcript = build_shadow_transcript_v1(main_events, envelope, policy);
    let shadow_transcript = build_shadow_transcript_v1(shadow_events, envelope, policy);
    let main_required_digest = build_shadow_required_digest(envelope, &main_transcript);
    let shadow_required_digest = build_shadow_required_digest(envelope, &shadow_transcript);
    let matched = main_required_digest == shadow_required_digest;

    Ok(ShadowCompareReportV1 {
        shadow_id: envelope.shadow_id.clone(),
        policy,
        main_required_digest: main_required_digest.clone(),
        shadow_required_digest: shadow_required_digest.clone(),
        event_count_main: main_transcript.len(),
        event_count_shadow: shadow_transcript.len(),
        matched,
        unsupported_reason: None,
        mismatch_reason: if matched {
            None
        } else {
            Some(format!(
                "V-SHADOW-MISMATCH: main={} shadow={}",
                main_required_digest, shadow_required_digest
            ))
        },
        main_transcript,
        shadow_transcript,
    })
}

fn next_artifact_run_index(report_dir: &Path, stem: &str) -> Result<u32, SdkError> {
    let mut max_seen = 0u32;
    if !report_dir.exists() {
        return Ok(1);
    }
    for entry in fs::read_dir(report_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(stem) {
            continue;
        }
        let Some(pos) = name.find(".r") else {
            continue;
        };
        let tail = &name[pos + 2..];
        let Some(end) = tail.find('.') else {
            continue;
        };
        if let Ok(value) = tail[..end].parse::<u32>() {
            max_seen = max_seen.max(value);
        }
    }
    Ok(max_seen.saturating_add(1))
}

fn escape_json(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn write_shadow_compare_artifacts_v1(
    root: &Path,
    report_stem: &str,
    report: &ShadowCompareReportV1,
    report_dir_override: Option<&Path>,
) -> Result<ShadowArtifactPathsV1, SdkError> {
    let report_dir = if let Some(dir) = report_dir_override {
        dir.to_path_buf()
    } else {
        root.join("target")
            .join("ocl")
            .join("v5")
            .join("w3")
            .join("reports")
    };
    fs::create_dir_all(&report_dir)?;

    let run_idx = next_artifact_run_index(&report_dir, report_stem)?;
    let report_path = report_dir.join(format!("{report_stem}.r{run_idx}.json"));
    let main_path = report_dir.join(format!("{report_stem}.main.r{run_idx}.jsonl"));
    let shadow_path = report_dir.join(format!("{report_stem}.shadow.r{run_idx}.jsonl"));

    let mut report_json = String::new();
    report_json.push_str("{\n");
    report_json.push_str("  \"shadow_id\": \"");
    report_json.push_str(&escape_json(&report.shadow_id));
    report_json.push_str("\",\n");
    report_json.push_str("  \"policy\": \"");
    report_json.push_str(report.policy.as_str());
    report_json.push_str("\",\n");
    report_json.push_str("  \"main_required_digest\": \"");
    report_json.push_str(&report.main_required_digest);
    report_json.push_str("\",\n");
    report_json.push_str("  \"shadow_required_digest\": \"");
    report_json.push_str(&report.shadow_required_digest);
    report_json.push_str("\",\n");
    report_json.push_str("  \"event_count_main\": ");
    report_json.push_str(&report.event_count_main.to_string());
    report_json.push_str(",\n");
    report_json.push_str("  \"event_count_shadow\": ");
    report_json.push_str(&report.event_count_shadow.to_string());
    report_json.push_str(",\n");
    report_json.push_str("  \"matched\": ");
    report_json.push_str(if report.matched { "true" } else { "false" });
    report_json.push_str(",\n");
    report_json.push_str("  \"unsupported_reason\": ");
    match &report.unsupported_reason {
        Some(value) => {
            report_json.push('"');
            report_json.push_str(&escape_json(value));
            report_json.push('"');
        }
        None => report_json.push_str("null"),
    }
    report_json.push_str(",\n");
    report_json.push_str("  \"mismatch_reason\": ");
    match &report.mismatch_reason {
        Some(value) => {
            report_json.push('"');
            report_json.push_str(&escape_json(value));
            report_json.push('"');
        }
        None => report_json.push_str("null"),
    }
    report_json.push_str("\n}\n");
    fs::write(&report_path, report_json)?;

    let mut main_text = String::new();
    for event in &report.main_transcript {
        main_text.push_str(&format!(
            "{{\"logical_tick\":{},\"event_id\":{},\"actor_id\":\"{}\",\"args_hash256\":\"{}\",\"decision_outcome\":\"{}\",\"commit_intents_hash256\":\"{}\"}}\n",
            event.logical_tick,
            event.event_id,
            escape_json(&event.actor_id),
            event.args_hash256,
            escape_json(&event.decision_outcome),
            event.commit_intents_hash256
        ));
    }
    fs::write(&main_path, main_text)?;

    let mut shadow_text = String::new();
    for event in &report.shadow_transcript {
        shadow_text.push_str(&format!(
            "{{\"logical_tick\":{},\"event_id\":{},\"actor_id\":\"{}\",\"args_hash256\":\"{}\",\"decision_outcome\":\"{}\",\"commit_intents_hash256\":\"{}\"}}\n",
            event.logical_tick,
            event.event_id,
            escape_json(&event.actor_id),
            event.args_hash256,
            escape_json(&event.decision_outcome),
            event.commit_intents_hash256
        ));
    }
    fs::write(&shadow_path, shadow_text)?;

    Ok(ShadowArtifactPathsV1 {
        report_path,
        main_transcript_path: main_path,
        shadow_transcript_path: shadow_path,
    })
}

pub fn run_project_with_shadow_compare(
    root: &Path,
    run_engine: RunEngine,
    locked: bool,
    options: &ShadowOptionsV1,
) -> Result<ShadowRunSummaryV1, SdkError> {
    let main = run_project_with_trace_engine_config_and_lock(
        root,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::Normal,
        },
    )?;
    let shadow = run_project_with_trace_engine_config_and_lock(
        root,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: shadow_commit_policy_mode(options.policy),
        },
    )?;
    let envelope = InputEnvelopeV1 {
        universe_id: "__legacy__".to_string(),
        domain_id: "default".to_string(),
        shadow_id: options.shadow_id.clone(),
        runtime_mode: "deterministic".to_string(),
        engine: run_engine.as_str().to_string(),
        run_kind: "project".to_string(),
    };
    let compare =
        compare_shadow_traces_v1(&main.events, &shadow.events, &envelope, options.policy)?;
    let artifacts = write_shadow_compare_artifacts_v1(
        root,
        "shadow_compare.run",
        &compare,
        options.report_dir.as_deref(),
    )?;
    if !compare.matched {
        return Err(SdkError::ShadowMismatch(
            compare
                .mismatch_reason
                .clone()
                .unwrap_or_else(|| "V-SHADOW-MISMATCH".to_string()),
        ));
    }
    Ok(ShadowRunSummaryV1 {
        steps: main.total_steps,
        compare,
        artifacts,
    })
}

pub fn run_reactor_service_with_shadow_compare(
    root: &Path,
    options: &ReactorServiceOptions,
    run_engine: RunEngine,
    locked: bool,
    shadow_options: &ShadowOptionsV1,
) -> Result<ShadowReactorRunSummaryV1, SdkError> {
    let reactor = run_reactor_service_with_lock(root, options, locked)?;
    let main = run_reactor_service_with_trace_engine_config_and_lock(
        root,
        options,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::Normal,
        },
    )?;
    let shadow = run_reactor_service_with_trace_engine_config_and_lock(
        root,
        options,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: shadow_commit_policy_mode(shadow_options.policy),
        },
    )?;
    let envelope = InputEnvelopeV1 {
        universe_id: options
            .universe_id
            .clone()
            .unwrap_or_else(|| "__legacy__".to_string()),
        domain_id: options
            .domain_id
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        shadow_id: shadow_options.shadow_id.clone(),
        runtime_mode: options.runtime_mode.as_str().to_string(),
        engine: run_engine.as_str().to_string(),
        run_kind: "reactor".to_string(),
    };
    let compare = compare_shadow_traces_v1(
        &main.events,
        &shadow.events,
        &envelope,
        shadow_options.policy,
    )?;
    let artifacts = write_shadow_compare_artifacts_v1(
        root,
        "shadow_compare.reactor",
        &compare,
        shadow_options.report_dir.as_deref(),
    )?;
    if !compare.matched {
        return Err(SdkError::ShadowMismatch(
            compare
                .mismatch_reason
                .clone()
                .unwrap_or_else(|| "V-SHADOW-MISMATCH".to_string()),
        ));
    }
    Ok(ShadowReactorRunSummaryV1 {
        reactor,
        compare,
        artifacts,
    })
}
