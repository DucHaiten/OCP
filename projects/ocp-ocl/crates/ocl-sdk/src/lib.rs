use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub mod m4;
pub mod w1;
pub mod w2;
pub mod w3;
pub mod w5;
pub mod w6;
pub mod w9;

pub use m4::{
    compose_phenotype, load_component_catalog, load_phenotype_spec, verify_assembly,
    AssemblyProofV1, ComponentSpecV1, ComposeSummary, PhenotypeSpecV1, VerifySummary,
};
use ocl_runtime_core::{
    check_file, normalize_text, parse_program, run_file_with_engine_config, run_source_with_engine,
    CommitPolicyMode, ExecConfig, Expr, GuardMode, RunEngine, RuntimeCoreError, Stmt, TraceEvent,
};
pub use w1::{
    enforce_universe_match_v1, init_cosmos_v1, resolve_hive_caps_v1, resolve_universe_v1,
    sync_cosmos_lock_v1, sync_policy_lock_v1, CosmosHiveV1, CosmosInitSummary,
    CosmosLockSyncSummary, CosmosLockV1, CosmosSpecV1, PolicyLockSyncSummary, PolicyLockV1,
    UniverseProfileV1, UniverseSelectionV1,
};
pub use w2::{
    admit_bridge_emit_v1, poll_bridge_event_v1, resolve_bridge_runtime_plan_v1,
    resolve_domain_selection_v1, validate_locked_cosmos_bridge_config_v1, BridgeEnvelopeV1,
    BridgeRuntimePlanV1, BridgeRuntimeRuleV1, BridgeRuntimeStateV1, DomainProfileV1,
    DomainSelectionV1,
};
pub use w3::{
    build_commit_intent_hash256, build_shadow_required_digest, build_shadow_transcript_v1,
    compare_shadow_traces_v1, parse_shadow_policy_v1, run_project_with_shadow_compare,
    run_reactor_service_with_shadow_compare, write_shadow_compare_artifacts_v1, InputEnvelopeV1,
    ShadowArtifactPathsV1, ShadowCompareReportV1, ShadowOptionsV1, ShadowPolicyV1,
    ShadowReactorRunSummaryV1, ShadowRunSummaryV1, ShadowTranscriptEventV1,
    SHADOW_EMPTY_COMMIT_HASH256,
};
pub use w5::{
    resolve_view_kit_bindings_v1, resolve_view_observe_key_v1, resolve_view_selection_v1,
    ViewKitBindingV1, ViewProfileV1, ViewSelectionV1,
};
pub use w6::{
    canonical_organ_sign_message, canonical_plugin_sign_message, collect_project_custom_keys,
    collect_required_organs_from_cosmos_v1, install_organs_v1, list_kits_from_cosmos_v1,
    resolve_platform_tag_v1, resolve_plugin_for_custom_key, run_kit_doctor_v1, sync_organs_lock_v1,
    sync_plugin_lock_v1, verify_organ_entry_signature, verify_organs_lock_v1,
    verify_plugin_lock_v1, verify_plugin_signature, KitDoctorSummaryV1, OrganInstallSummaryV1,
    OrganLockEntryV1, OrganLockSyncSummaryV1, OrganRegistryEntryV1, OrganRegistryIndexV1,
    OrganVerifySummaryV1, PluginLockEntryV1, PluginLockSyncSummary, PluginRegistryEntryV1,
    PluginRegistryIndexV1, PluginVerifySummary,
};
pub use w9::{
    compute_conformance_required_digest, default_conformance_manifest_path,
    default_conformance_manifest_path_with_selector, parse_conformance_manifest_v1,
    render_conformance_report_json, run_conformance_v1, write_conformance_report_json,
    ConformanceExpectedStatusV1, ConformanceManifestV1, ConformanceReportV1,
    ConformanceRunOptionsV1, ConformanceScenarioResultV1, ConformanceScenarioV1, ConformanceStepV1,
};

#[derive(Debug)]
pub enum SdkError {
    Io(std::io::Error),
    Runtime(RuntimeCoreError),
    MissingProject(String),
    FmtMismatch(Vec<String>),
    LockMismatch(String),
    PermissionMissing(String),
    PermissionDenied(String),
    AuditChainInvalid(String),
    SupplyInvalid(String),
    ShadowMismatch(String),
    ShadowUnsupported(String),
}

impl Display for SdkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Runtime(err) => write!(f, "{err}"),
            Self::MissingProject(msg) => write!(f, "{msg}"),
            Self::FmtMismatch(paths) => {
                write!(f, "format check failed for {} file(s): ", paths.len())?;
                write!(f, "{}", paths.join(", "))
            }
            Self::LockMismatch(msg) => write!(f, "{msg}"),
            Self::PermissionMissing(msg) => write!(f, "{msg}"),
            Self::PermissionDenied(msg) => write!(f, "{msg}"),
            Self::AuditChainInvalid(msg) => write!(f, "{msg}"),
            Self::SupplyInvalid(msg) => write!(f, "{msg}"),
            Self::ShadowMismatch(msg) => write!(f, "{msg}"),
            Self::ShadowUnsupported(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<RuntimeCoreError> for SdkError {
    fn from(value: RuntimeCoreError) -> Self {
        Self::Runtime(value)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub deps_lock: PathBuf,
    pub src_main: PathBuf,
    pub tests_dir: PathBuf,
    pub bundle_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PermissionRules {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectPermissions {
    pub package: Option<PermissionRules>,
    pub modules: HashMap<String, PermissionRules>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLanguageConfigV071 {
    pub lane: String,
    pub guard_mode: GuardMode,
}

impl Default for ProjectLanguageConfigV071 {
    fn default() -> Self {
        Self {
            lane: "locked_v071".to_string(),
            guard_mode: GuardMode::Return,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermissionDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub seq: u64,
    pub signature: String,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSummary {
    pub files_checked: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSummary {
    pub steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactorSummary {
    pub ticks: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactorRuntimeMode {
    Deterministic,
    Throughput,
}

impl ReactorRuntimeMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Throughput => "throughput",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactorServiceOptions {
    pub ticks: u32,
    pub runtime_mode: ReactorRuntimeMode,
    pub socket_listen: Option<String>,
    pub runtime_report: Option<PathBuf>,
    pub replay_audit: Option<PathBuf>,
    pub io_tape_record_path: Option<PathBuf>,
    pub io_tape_replay_path: Option<PathBuf>,
    pub hive_caps: Option<CosmosHiveV1>,
    pub universe_id: Option<String>,
    pub domain_id: Option<String>,
}

impl Default for ReactorServiceOptions {
    fn default() -> Self {
        Self {
            ticks: 32,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
            io_tape_record_path: None,
            io_tape_replay_path: None,
            hive_caps: None,
            universe_id: None,
            domain_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactorServiceReport {
    pub ticks: u32,
    pub total_steps: u32,
    pub mode: ReactorRuntimeMode,
    pub event_count: u32,
    pub event_id_end: u64,
    pub mailbox_high_water: u32,
    pub backpressure_count: u32,
    pub cancelled_count: u32,
    pub deadline_exceeded_count: u32,
    pub socket_listen: Option<String>,
    pub runtime_report_written: bool,
    pub replay_audit_written: bool,
    pub audit_chain_hash: Option<String>,
    pub universe_id: String,
    pub domain_id: String,
    pub domain_count: u32,
    pub bridge_emit_count: u32,
    pub bridge_dispatch_count: u32,
    pub bridge_backpressure_count: u32,
    pub workers_alive: u32,
    pub workers_spawned_total: u32,
    pub workers_reused_total: u32,
    pub mailbox_max_depth_observed: u32,
    pub alloc_events_total: u32,
    pub fanout_drop_count: u32,
    pub spawn_drop_count: u32,
    pub dispatch_digest256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSummary {
    pub tests_run: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FmtSummary {
    pub files_touched: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildSummary {
    pub files_bundled: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockSyncSummary {
    pub deps_synced: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOclPkgSummary {
    pub artifact_path: PathBuf,
    pub files_bundled: usize,
    pub payload_hash_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishSummary {
    pub artifact_path: PathBuf,
    pub registry_index: PathBuf,
    pub package_name: String,
    pub payload_hash_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchSummary {
    pub artifact_path: PathBuf,
    pub package_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplyVerifySummary {
    pub valid: bool,
    pub package_name: String,
    pub payload_hash_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEventV1 {
    pub seq: u64,
    pub run_id: String,
    pub event: String,
    pub key: Option<String>,
    pub kind: Option<String>,
    pub reason: Option<String>,
    pub origin_id: Option<u64>,
    pub allowed: Option<bool>,
    pub value: Option<bool>,
    pub steps: Option<u32>,
    pub universe_id: String,
    pub domain_id: String,
    pub payload_hash: String,
}

const TRACE_UNIVERSE_SENTINEL: &str = "__legacy__";
const TRACE_DOMAIN_SENTINEL: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceRunSummary {
    pub run_id: String,
    pub total_steps: u32,
    pub events: Vec<TraceEventV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceViewOptions {
    pub tail: Option<usize>,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceViewReport {
    pub event_count: usize,
    pub required_digest: String,
    pub rendered: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfileKeyCostV1 {
    pub key: String,
    pub observe_count: u32,
    pub ok_count: u32,
    pub degraded_count: u32,
    pub insufficient_count: u32,
    pub deferred_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileReportV1 {
    pub run_id: String,
    pub event_count: u32,
    pub observe_count: u32,
    pub eval_steps_est: u32,
    pub alloc_units_est: u32,
    pub total_steps: u32,
    pub required_digest: String,
    pub key_costs: Vec<ProfileKeyCostV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileViewOptions {
    pub top: usize,
    pub json: bool,
}

impl Default for ProfileViewOptions {
    fn default() -> Self {
        Self {
            top: 10,
            json: false,
        }
    }
}

pub fn project_layout(root: &Path) -> ProjectLayout {
    ProjectLayout {
        root: root.to_path_buf(),
        manifest: root.join("Ocl.toml"),
        deps_lock: root.join("deps.lock"),
        src_main: root.join("src").join("main.ocl"),
        tests_dir: root.join("tests"),
        bundle_dir: root.join(".oclbundle"),
    }
}

fn parse_string_array_literal(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Vec::new();
    }

    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Vec::new();
        }
        return inner
            .split(',')
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
            .collect();
    }

    let single = value.trim_matches('"').to_string();
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}

fn normalize_permission_rules(rules: &mut PermissionRules) {
    rules.allow.sort();
    rules.allow.dedup();
    rules.deny.sort();
    rules.deny.dedup();
}

fn parse_permissions_from_manifest(manifest_text: &str) -> ProjectPermissions {
    let mut out = ProjectPermissions::default();
    let mut current_section = String::new();

    for raw in manifest_text.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
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
        let values = parse_string_array_literal(value_raw);

        if current_section == "permissions.package" {
            let rules = out.package.get_or_insert_with(PermissionRules::default);
            match key {
                "allow" => rules.allow = values,
                "deny" => rules.deny = values,
                _ => {}
            }
            normalize_permission_rules(rules);
            continue;
        }

        if let Some(module_name) = current_section.strip_prefix("permissions.module.") {
            let module_key = module_name.trim().to_string();
            if module_key.is_empty() {
                continue;
            }
            let rules = out.modules.entry(module_key).or_default();
            match key {
                "allow" => rules.allow = values,
                "deny" => rules.deny = values,
                _ => {}
            }
            normalize_permission_rules(rules);
        }
    }

    out
}

pub fn parse_project_language_config_v071(manifest_text: &str) -> ProjectLanguageConfigV071 {
    let mut out = ProjectLanguageConfigV071::default();
    let mut current_section = String::new();

    for raw in manifest_text.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
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
        let value = value_raw.trim().trim_matches('"');

        if current_section == "project" && key == "lane" && !value.is_empty() {
            out.lane = value.to_string();
            continue;
        }

        if current_section == "language" && key == "guard_mode" {
            out.guard_mode = match value {
                "error" => GuardMode::Error,
                _ => GuardMode::Return,
            };
        }
    }

    out
}

fn permission_pattern_matches(pattern: &str, key: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return key.starts_with(prefix);
    }
    key == pattern
}

fn permission_rule_decision(
    rules: &PermissionRules,
    key: &str,
) -> Option<(PermissionDecision, String)> {
    if let Some(matched) = rules
        .deny
        .iter()
        .find(|pattern| permission_pattern_matches(pattern, key))
    {
        return Some((PermissionDecision::Deny, matched.clone()));
    }

    if !rules.allow.is_empty() {
        if let Some(matched) = rules
            .allow
            .iter()
            .find(|pattern| permission_pattern_matches(pattern, key))
        {
            return Some((PermissionDecision::Allow, matched.clone()));
        }
        return Some((PermissionDecision::Deny, "<implicit-deny-not-in-allow>".to_string()));
    }

    None
}

fn permission_decision_for_key(
    permissions: &ProjectPermissions,
    module_path: Option<&str>,
    key: &str,
) -> (PermissionDecision, Option<String>, Option<String>) {
    if let Some(module_path) = module_path {
        if let Some(module_rules) = permissions.modules.get(module_path) {
            if let Some((decision, pattern)) = permission_rule_decision(module_rules, key) {
                return (
                    decision,
                    Some(pattern),
                    Some(format!("permissions.module.{module_path}")),
                );
            }
        }
    }

    if let Some(package_rules) = permissions.package.as_ref() {
        if let Some((decision, pattern)) = permission_rule_decision(package_rules, key) {
            return (
                decision,
                Some(pattern),
                Some("permissions.package".to_string()),
            );
        }
    }

    (PermissionDecision::Allow, None, None)
}

fn module_path_from_program(stmts: &[Stmt]) -> Option<String> {
    for stmt in stmts {
        if let Stmt::ModuleDecl { path, .. } = stmt {
            if path.is_empty() {
                return None;
            }
            return Some(path.join("."));
        }
    }
    None
}

fn collect_observe_keys(stmts: &[Stmt], out: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Observe {
                key: Expr::String { value, .. },
                ..
            } => out.push(value.clone()),
            Stmt::Observe { .. } => {}
            Stmt::FnDef { body, .. } | Stmt::ForRange { body, .. } => {
                collect_observe_keys(body, out);
            }
            Stmt::Match(m) => {
                collect_observe_keys(&m.ok_arm, out);
                collect_observe_keys(&m.degraded_arm, out);
                collect_observe_keys(&m.insufficient_arm, out);
                collect_observe_keys(&m.deferred_arm, out);
            }
            _ => {}
        }
    }
}

fn verify_permissions_for_source(
    source: &str,
    file_id: u32,
    file_path: &Path,
    permissions: &ProjectPermissions,
) -> Result<(), SdkError> {
    if permissions.package.is_none() && permissions.modules.is_empty() {
        return Ok(());
    }

    let program = parse_program(source, file_id).map_err(RuntimeCoreError::from)?;
    let module_path = module_path_from_program(&program.statements);
    let mut observe_keys = Vec::new();
    collect_observe_keys(&program.statements, &mut observe_keys);

    for key in observe_keys {
        let (decision, matched_rule, section) =
            permission_decision_for_key(permissions, module_path.as_deref(), &key);
        if decision == PermissionDecision::Deny {
            let section_name = section.unwrap_or_else(|| "permissions.package".to_string());
            let matched = matched_rule.unwrap_or_else(|| "<none>".to_string());
            let hint = if section_name.starts_with("permissions.module.") {
                format!(
                    "Hint: add key to `[{}].allow` or remove from `[{}].deny`.",
                    section_name, section_name
                )
            } else {
                "Hint: add key to `[permissions.package].allow` or remove from `[permissions.package].deny`.".to_string()
            };
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; matched deny rule=`{}` in [{}]. {}",
                key,
                file_path.display(),
                module_path
                    .as_ref()
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
                matched,
                section_name,
                hint
            )));
        }
    }

    Ok(())
}

fn load_permissions_for_layout(
    layout: &ProjectLayout,
    locked: bool,
) -> Result<ProjectPermissions, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    let permissions = parse_permissions_from_manifest(&manifest);

    if locked && permissions.package.is_none() {
        return Err(SdkError::PermissionMissing(
            "V-PERMISSIONS-MISSING: `--locked` requires [permissions.package] in Ocl.toml"
                .to_string(),
        ));
    }

    Ok(permissions)
}

fn load_project_language_config_for_layout(
    layout: &ProjectLayout,
) -> Result<ProjectLanguageConfigV071, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    Ok(parse_project_language_config_v071(&manifest))
}

fn default_exec_config_for_layout(layout: &ProjectLayout) -> Result<ExecConfig, SdkError> {
    let cfg = load_project_language_config_for_layout(layout)?;
    Ok(ExecConfig {
        step_cap: 4096,
        commit_policy: CommitPolicyMode::Normal,
        guard_mode: cfg.guard_mode,
    })
}

pub fn build_audit_entries(signatures: &[String]) -> Vec<AuditEntry> {
    let mut out = Vec::with_capacity(signatures.len());
    let mut prev_hash = "0".to_string();

    for (idx, sig) in signatures.iter().enumerate() {
        let seq = idx as u64 + 1;
        let payload = format!("{seq}|{sig}|{prev_hash}");
        let hash = fnv1a64_hex(&payload);
        out.push(AuditEntry {
            seq,
            signature: sig.clone(),
            prev_hash: prev_hash.clone(),
            hash: hash.clone(),
        });
        prev_hash = hash;
    }

    out
}

pub fn verify_audit_entries(entries: &[AuditEntry]) -> Result<(), SdkError> {
    let mut expected_prev = "0".to_string();
    for (idx, entry) in entries.iter().enumerate() {
        let expected_seq = idx as u64 + 1;
        if entry.seq != expected_seq {
            return Err(SdkError::AuditChainInvalid(format!(
                "V-AUDIT-SEQ: expected seq={}, got seq={}",
                expected_seq, entry.seq
            )));
        }
        if entry.prev_hash != expected_prev {
            return Err(SdkError::AuditChainInvalid(format!(
                "V-AUDIT-PREV-HASH: seq={} expected prev_hash={} got {}",
                entry.seq, expected_prev, entry.prev_hash
            )));
        }
        let payload = format!("{}|{}|{}", entry.seq, entry.signature, entry.prev_hash);
        let expected_hash = fnv1a64_hex(&payload);
        if entry.hash != expected_hash {
            return Err(SdkError::AuditChainInvalid(format!(
                "V-AUDIT-HASH: seq={} expected hash={} got {}",
                entry.seq, expected_hash, entry.hash
            )));
        }
        expected_prev = entry.hash.clone();
    }
    Ok(())
}

fn write_audit_jsonl(path: &Path, entries: &[AuditEntry]) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut content = String::new();
    for entry in entries {
        let line = format!(
            "{{\"seq\":{},\"signature\":\"{}\",\"prev_hash\":\"{}\",\"hash\":\"{}\"}}\n",
            entry.seq, entry.signature, entry.prev_hash, entry.hash
        );
        content.push_str(&line);
    }
    fs::write(path, content)?;
    Ok(())
}

pub fn init_project(root: &Path) -> Result<ProjectLayout, SdkError> {
    let layout = project_layout(root);
    fs::create_dir_all(layout.src_main.parent().expect("src path must have parent"))?;
    fs::create_dir_all(&layout.tests_dir)?;

    if !layout.manifest.exists() {
        let project_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .replace('-', "_");
        let manifest = format!(
            "[package]\nname = \"{project_name}\"\nversion = \"0.1.0\"\n\n[targets]\ndefault = \"main\"\n\n[dependencies]\n"
        );
        fs::write(&layout.manifest, manifest)?;
    }

    if !layout.deps_lock.exists() {
        fs::write(&layout.deps_lock, "version=1\n")?;
    }

    if !layout.src_main.exists() {
        let sample = "let ready = true;\ncondition(ready);\n";
        fs::write(&layout.src_main, sample)?;
    }

    let smoke = layout.tests_dir.join("smoke.ocl");
    if !smoke.exists() {
        fs::write(smoke, "let t = true;\ncondition(t);\n")?;
    }

    Ok(layout)
}

fn verify_project_exists(layout: &ProjectLayout) -> Result<(), SdkError> {
    if !layout.manifest.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            layout.manifest.display()
        )));
    }
    if !layout.src_main.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing entry source: {}",
            layout.src_main.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestDep {
    name: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockDep {
    name: String,
    version: String,
    hash64: String,
}

fn parse_manifest_dependencies(manifest: &str) -> Result<Vec<ManifestDep>, SdkError> {
    let mut deps = Vec::new();
    let mut in_dependencies = false;

    for raw in manifest.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            return Err(SdkError::MissingProject(
                "invalid [dependencies] entry in Ocl.toml".to_string(),
            ));
        };
        let name = k.trim();
        let version = v.trim().trim_matches('"');
        if name.is_empty() || version.is_empty() {
            return Err(SdkError::MissingProject(
                "dependency name/version must not be empty".to_string(),
            ));
        }
        deps.push(ManifestDep {
            name: name.to_string(),
            version: version.to_string(),
        });
    }

    deps.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(deps)
}

fn to_lock_deps(deps: &[ManifestDep]) -> Vec<LockDep> {
    let mut out: Vec<LockDep> = deps
        .iter()
        .map(|dep| {
            let digest_input = format!("{}@{}", dep.name, dep.version);
            LockDep {
                name: dep.name.clone(),
                version: dep.version.clone(),
                hash64: fnv1a64_hex(&digest_input),
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    out
}

fn encode_lock_v1(lock_deps: &[LockDep]) -> String {
    let mut out = String::from("version=1\n");
    for dep in lock_deps {
        out.push_str("dep=");
        out.push_str(&dep.name);
        out.push('|');
        out.push_str(&dep.version);
        out.push('|');
        out.push_str(&dep.hash64);
        out.push('\n');
    }
    out
}

fn parse_lock_v1(text: &str) -> Result<Vec<LockDep>, SdkError> {
    let mut has_version = false;
    let mut deps = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        let Some(payload) = line.strip_prefix("dep=") else {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock line, expected `dep=...`".to_string(),
            ));
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 3 {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock dep format, expected `dep=name|version|hash64`".to_string(),
            ));
        }
        if parts[2].len() != 16 || !parts[2].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SdkError::LockMismatch(
                "invalid deps.lock hash64 format (expected 16 hex chars)".to_string(),
            ));
        }
        deps.push(LockDep {
            name: parts[0].to_string(),
            version: parts[1].to_string(),
            hash64: parts[2].to_ascii_lowercase(),
        });
    }

    if !has_version {
        return Err(SdkError::LockMismatch(
            "deps.lock missing `version=1` header".to_string(),
        ));
    }

    deps.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(deps)
}

fn read_expected_lock(layout: &ProjectLayout) -> Result<Vec<LockDep>, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest)?;
    Ok(to_lock_deps(&deps))
}

fn verify_lock_consistency(layout: &ProjectLayout) -> Result<Vec<LockDep>, SdkError> {
    let expected = read_expected_lock(layout)?;
    if !layout.deps_lock.exists() {
        return Err(SdkError::LockMismatch(format!(
            "missing deps.lock: {} (run `ocl lock sync <project_dir>`)",
            layout.deps_lock.display()
        )));
    }
    let raw = fs::read_to_string(&layout.deps_lock)?;
    let current = parse_lock_v1(&raw)?;
    if current != expected {
        return Err(SdkError::LockMismatch(
            "deps.lock mismatch with Ocl.toml dependencies (run `ocl lock sync <project_dir>`)"
                .to_string(),
        ));
    }
    Ok(current)
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockDepV2 {
    name: String,
    version: String,
    hash64: String,
    signature_b64: String,
    signer_pub_b64: String,
}

fn deterministic_sign(context: &str, message: &[u8]) -> (String, String) {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ocl-sdk-signing-key-v1|");
    hasher.update(context.as_bytes());
    let seed = *hasher.finalize().as_bytes();

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    let signature = signing_key.sign(message);

    (
        B64.encode(signature.to_bytes()),
        B64.encode(verifying_key.to_bytes()),
    )
}

fn deterministic_verify(
    message: &[u8],
    signature_b64: &str,
    signer_pub_b64: &str,
) -> Result<(), SdkError> {
    let signature_raw = B64
        .decode(signature_b64)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid signature base64 encoding: {e}")))?;
    let signer_pub_raw = B64.decode(signer_pub_b64).map_err(|e| {
        SdkError::SupplyInvalid(format!("invalid signer public key base64 encoding: {e}"))
    })?;

    let signer_pub_bytes: [u8; 32] = signer_pub_raw
        .as_slice()
        .try_into()
        .map_err(|_| SdkError::SupplyInvalid("invalid signer public key length".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&signer_pub_bytes)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid signer public key material: {e}")))?;
    let signature = Signature::from_slice(&signature_raw)
        .map_err(|e| SdkError::SupplyInvalid(format!("invalid signature material: {e}")))?;

    verifying_key
        .verify(message, &signature)
        .map_err(|_| SdkError::SupplyInvalid("signature verification failed".to_string()))
}

fn hash256_hex(input: &[u8]) -> String {
    blake3::hash(input).to_hex().to_string()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, SdkError> {
    if !hex.len().is_multiple_of(2) {
        return Err(SdkError::SupplyInvalid(
            "hex payload has odd length".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let hi = bytes[i] as char;
        let lo = bytes[i + 1] as char;
        let pair = [hi, lo].iter().collect::<String>();
        let value = u8::from_str_radix(&pair, 16)
            .map_err(|e| SdkError::SupplyInvalid(format!("invalid hex payload: {e}")))?;
        out.push(value);
        i += 2;
    }
    Ok(out)
}

fn to_lock_deps_v2(lock_deps: &[LockDep]) -> Vec<LockDepV2> {
    let mut out = Vec::new();
    for dep in lock_deps {
        let msg = format!("{}|{}|{}", dep.name, dep.version, dep.hash64);
        let (sig, pub_key) = deterministic_sign("dep-lock-v2", msg.as_bytes());
        out.push(LockDepV2 {
            name: dep.name.clone(),
            version: dep.version.clone(),
            hash64: dep.hash64.clone(),
            signature_b64: sig,
            signer_pub_b64: pub_key,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    out
}

fn encode_lock_v2(lock_deps: &[LockDepV2]) -> String {
    let mut out = String::from("version=2\n");
    for dep in lock_deps {
        out.push_str("dep=");
        out.push_str(&dep.name);
        out.push('|');
        out.push_str(&dep.version);
        out.push('|');
        out.push_str(&dep.hash64);
        out.push('|');
        out.push_str(&dep.signature_b64);
        out.push('|');
        out.push_str(&dep.signer_pub_b64);
        out.push('\n');
    }
    out
}

fn parse_lock_v2(text: &str) -> Result<Vec<LockDepV2>, SdkError> {
    let mut has_version = false;
    let mut deps = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=2" {
            has_version = true;
            continue;
        }
        let Some(payload) = line.strip_prefix("dep=") else {
            return Err(SdkError::SupplyInvalid(
                "invalid deps.lock.v2 line, expected `dep=...`".to_string(),
            ));
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 5 {
            return Err(SdkError::SupplyInvalid(
                "invalid dep entry in deps.lock.v2 (expected 5 fields)".to_string(),
            ));
        }
        deps.push(LockDepV2 {
            name: parts[0].to_string(),
            version: parts[1].to_string(),
            hash64: parts[2].to_string(),
            signature_b64: parts[3].to_string(),
            signer_pub_b64: parts[4].to_string(),
        });
    }
    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v2 missing `version=2` header".to_string(),
        ));
    }
    deps.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(deps)
}

fn is_builtin_dep(name: &str) -> bool {
    name == "std"
}

fn verify_lock_v2_signatures(lock_deps_v2: &[LockDepV2]) -> Result<(), SdkError> {
    for dep in lock_deps_v2 {
        if is_builtin_dep(&dep.name) {
            continue;
        }
        if dep.signature_b64.is_empty() || dep.signer_pub_b64.is_empty() {
            return Err(SdkError::SupplyInvalid(format!(
                "missing signature for non-builtin dep `{}` in deps.lock.v2",
                dep.name
            )));
        }
        let msg = format!("{}|{}|{}", dep.name, dep.version, dep.hash64);
        deterministic_verify(msg.as_bytes(), &dep.signature_b64, &dep.signer_pub_b64)?;
    }
    Ok(())
}

pub fn sync_deps_lock_v1(root: &Path) -> Result<LockSyncSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let expected = read_expected_lock(&layout)?;
    fs::write(&layout.deps_lock, encode_lock_v1(&expected))?;
    let lock_v2 = to_lock_deps_v2(&expected);
    fs::write(layout.root.join("deps.lock.v2"), encode_lock_v2(&lock_v2))?;
    Ok(LockSyncSummary {
        deps_synced: expected.len(),
    })
}

pub fn sync_deps_lock_v2(root: &Path) -> Result<LockSyncSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let expected = read_expected_lock(&layout)?;
    let lock_v2 = to_lock_deps_v2(&expected);
    fs::write(layout.root.join("deps.lock.v2"), encode_lock_v2(&lock_v2))?;
    Ok(LockSyncSummary {
        deps_synced: expected.len(),
    })
}

fn collect_ocl_files(base: &Path, out: &mut Vec<PathBuf>) -> Result<(), SdkError> {
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ocl_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("ocl") {
            out.push(path);
        }
    }
    Ok(())
}

fn gather_project_ocl_files(layout: &ProjectLayout) -> Result<Vec<PathBuf>, SdkError> {
    let mut files = Vec::new();
    collect_ocl_files(&layout.root.join("src"), &mut files)?;
    collect_ocl_files(&layout.tests_dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn verify_permissions_for_file(
    file_path: &Path,
    file_id: u32,
    permissions: &ProjectPermissions,
) -> Result<(), SdkError> {
    let source = fs::read_to_string(file_path)?;
    verify_permissions_for_source(&source, file_id, file_path, permissions)
}

fn enforce_locked_plugin_contract(layout: &ProjectLayout, locked: bool) -> Result<(), SdkError> {
    if !locked {
        return Ok(());
    }
    let custom_keys = collect_project_custom_keys(&layout.root)?;
    if custom_keys.is_empty() {
        return Ok(());
    }

    let verify = verify_plugin_lock_v1(&layout.root)?;
    if verify.plugins_verified == 0 {
        return Err(SdkError::MissingProject(
            "V-PLUGIN-LOCK-EMPTY: plugins.lock.v1 has no plugin entries".to_string(),
        ));
    }

    for key in custom_keys {
        if resolve_plugin_for_custom_key(&layout.root, &key, true)?.is_none() {
            return Err(SdkError::MissingProject(format!(
                "V-PLUGIN-KEY-UNRESOLVED: key `{key}` is not covered by plugins.lock.v1"
            )));
        }
    }

    std::env::set_var(
        "OCL_PLUGIN_LOCK_PATH",
        layout
            .root
            .join("plugins.lock.v1")
            .to_string_lossy()
            .to_string(),
    );
    std::env::set_var("OCL_PLUGIN_ROOT", layout.root.to_string_lossy().to_string());
    Ok(())
}

fn enforce_locked_organ_contract(layout: &ProjectLayout, locked: bool) -> Result<(), SdkError> {
    if !locked {
        return Ok(());
    }
    let doctor = run_kit_doctor_v1(&layout.root, true)?;
    if doctor.required_organs.is_empty() {
        return Ok(());
    }
    let verify = verify_organs_lock_v1(&layout.root, true)?;
    if verify.organs_verified == 0 {
        return Err(SdkError::SupplyInvalid(
            "V-ORGANS-LOCK-EMPTY: organs.lock.v1 has no organ entries".to_string(),
        ));
    }
    std::env::set_var(
        "OCL_ORGANS_LOCK_PATH",
        layout
            .root
            .join("organs.lock.v1")
            .to_string_lossy()
            .to_string(),
    );
    Ok(())
}

pub fn check_project(root: &Path) -> Result<CheckSummary, SdkError> {
    check_project_with_lock(root, false)
}

pub fn check_project_with_lock(root: &Path, locked: bool) -> Result<CheckSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    let files = gather_project_ocl_files(&layout)?;
    if files.is_empty() {
        return Err(SdkError::MissingProject(
            "project has no .ocl sources under src/ or tests/".to_string(),
        ));
    }
    for (idx, path) in files.iter().enumerate() {
        verify_permissions_for_file(path, idx as u32 + 1, &permissions)?;
        check_file(path, idx as u32 + 1)?;
    }
    Ok(CheckSummary {
        files_checked: files.len(),
    })
}

pub fn run_project(root: &Path) -> Result<RunSummary, SdkError> {
    run_project_with_lock(root, false)
}

pub fn run_project_with_lock(root: &Path, locked: bool) -> Result<RunSummary, SdkError> {
    run_project_with_engine_and_lock(root, RunEngine::Interpreter, locked)
}

pub fn run_project_with_engine(root: &Path, run_engine: RunEngine) -> Result<RunSummary, SdkError> {
    run_project_with_engine_and_lock(root, run_engine, false)
}

pub fn run_project_with_engine_and_lock(
    root: &Path,
    run_engine: RunEngine,
    locked: bool,
) -> Result<RunSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_for_file(&layout.src_main, 1, &permissions)?;
    let config = default_exec_config_for_layout(&layout)?;
    let out = run_file_with_engine_config(&layout.src_main, 1, config, run_engine)?;
    Ok(RunSummary { steps: out.steps })
}

pub fn run_reactor_ticks(root: &Path, ticks: u32) -> Result<ReactorSummary, SdkError> {
    run_reactor_ticks_with_lock(root, ticks, false)
}

#[derive(Debug, Clone, Copy)]
struct RuntimeBudgetConfig {
    std_net_enabled: bool,
    mailbox_max_depth: u32,
    io_max_events_per_tick: u32,
    default_deadline_ms: u32,
    actor_max: u32,
}

impl Default for RuntimeBudgetConfig {
    fn default() -> Self {
        Self {
            std_net_enabled: false,
            mailbox_max_depth: 32,
            io_max_events_per_tick: 8,
            default_deadline_ms: 250,
            actor_max: 1,
        }
    }
}

fn parse_bool_literal(raw: &str) -> Option<bool> {
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_runtime_budget_from_manifest(manifest_text: &str) -> RuntimeBudgetConfig {
    let mut out = RuntimeBudgetConfig::default();
    let mut in_runtime = false;

    for raw in manifest_text.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_runtime = line == "[runtime]";
            continue;
        }

        if !in_runtime {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"');

        match key {
            "std_net" => {
                if let Some(flag) = parse_bool_literal(value) {
                    out.std_net_enabled = flag;
                }
            }
            "mailbox_max_depth" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.mailbox_max_depth = parsed.max(1);
                }
            }
            "io_max_events_per_tick" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.io_max_events_per_tick = parsed.max(1);
                }
            }
            "default_deadline_ms" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.default_deadline_ms = parsed.max(1);
                }
            }
            "actor_max" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.actor_max = parsed.max(1);
                }
            }
            _ => {}
        }
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReactorIoTapeEntry {
    tick: u32,
    domain_id: String,
    payload_hash256: String,
}

fn parse_reactor_io_tape_entry(line: &str) -> Result<ReactorIoTapeEntry, SdkError> {
    let mut tick = None::<u32>;
    let mut domain_id = None::<String>;
    let mut payload_hash256 = None::<String>;

    for part in line.split('|') {
        let Some((k, v)) = part.split_once('=') else {
            return Err(SdkError::MissingProject(
                "invalid io tape line: expected key=value fields".to_string(),
            ));
        };
        match k {
            "tick" => {
                let parsed = v.parse::<u32>().map_err(|_| {
                    SdkError::MissingProject("invalid io tape tick value".to_string())
                })?;
                tick = Some(parsed);
            }
            "domain_id" => domain_id = Some(v.to_string()),
            "payload_hash256" => payload_hash256 = Some(v.to_string()),
            _ => {}
        }
    }

    let Some(tick) = tick else {
        return Err(SdkError::MissingProject(
            "invalid io tape line: missing tick".to_string(),
        ));
    };
    let Some(domain_id) = domain_id else {
        return Err(SdkError::MissingProject(
            "invalid io tape line: missing domain_id".to_string(),
        ));
    };
    let Some(payload_hash256) = payload_hash256 else {
        return Err(SdkError::MissingProject(
            "invalid io tape line: missing payload_hash256".to_string(),
        ));
    };

    Ok(ReactorIoTapeEntry {
        tick,
        domain_id,
        payload_hash256,
    })
}

fn read_reactor_io_tape(path: &Path) -> Result<Vec<ReactorIoTapeEntry>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(parse_reactor_io_tape_entry(trimmed)?);
    }
    Ok(out)
}

fn write_reactor_io_tape(path: &Path, entries: &[ReactorIoTapeEntry]) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut out = String::new();
    for entry in entries {
        out.push_str("tick=");
        out.push_str(&entry.tick.to_string());
        out.push_str("|domain_id=");
        out.push_str(&entry.domain_id);
        out.push_str("|payload_hash256=");
        out.push_str(&entry.payload_hash256);
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct HiveDomainRuntimeStateV1 {
    next_worker_id: u32,
    active_workers: u32,
    idle_workers: Vec<u32>,
    mailbox_seq: u64,
}

#[allow(clippy::too_many_arguments)]
fn apply_hive_workload_for_event(
    hive_caps: &CosmosHiveV1,
    domain_state: &mut HiveDomainRuntimeStateV1,
    domain_rank: usize,
    tick: u32,
    event_id: u64,
    payload_hash256: &str,
    spawn_count_tick: &mut u32,
    mailbox_depth: &mut u32,
    backpressure_count: &mut u32,
    workers_spawned_total: &mut u32,
    workers_reused_total: &mut u32,
    alloc_events_total: &mut u32,
    fanout_drop_count: &mut u32,
    spawn_drop_count: &mut u32,
    mailbox_max_depth_observed: &mut u32,
    dispatch_rows: &mut Vec<String>,
) {
    let desired_fanout = 2u32;
    let actual_fanout = desired_fanout.min(hive_caps.max_fanout_per_task);
    *fanout_drop_count = fanout_drop_count.saturating_add(desired_fanout - actual_fanout);

    for task_idx in 0..actual_fanout {
        if *mailbox_depth >= hive_caps.mailbox_max_depth {
            *backpressure_count = backpressure_count.saturating_add(1);
            *spawn_drop_count = spawn_drop_count.saturating_add(1);
            continue;
        }

        *mailbox_depth = mailbox_depth.saturating_add(1);
        *mailbox_max_depth_observed = (*mailbox_max_depth_observed).max(*mailbox_depth);

        let worker_id = if let Some(worker_id) = domain_state.idle_workers.pop() {
            *workers_reused_total = workers_reused_total.saturating_add(1);
            worker_id
        } else if domain_state.active_workers < hive_caps.max_swarm_workers
            && *spawn_count_tick < hive_caps.max_spawn_per_tick
        {
            domain_state.next_worker_id = domain_state.next_worker_id.saturating_add(1);
            domain_state.active_workers = domain_state.active_workers.saturating_add(1);
            *spawn_count_tick = spawn_count_tick.saturating_add(1);
            *workers_spawned_total = workers_spawned_total.saturating_add(1);
            *alloc_events_total = alloc_events_total.saturating_add(1);
            domain_state.next_worker_id
        } else {
            *spawn_drop_count = spawn_drop_count.saturating_add(1);
            *mailbox_depth = mailbox_depth.saturating_sub(1);
            continue;
        };

        domain_state.mailbox_seq = domain_state.mailbox_seq.saturating_add(1);
        dispatch_rows.push(format!(
            "{}|{}|{}|{}|{}|{}|{}",
            tick,
            domain_rank,
            worker_id,
            domain_state.mailbox_seq,
            event_id,
            task_idx,
            payload_hash256
        ));

        domain_state.idle_workers.push(worker_id);
        *mailbox_depth = mailbox_depth.saturating_sub(1);
    }
}

pub fn run_reactor_service_with_lock(
    root: &Path,
    options: &ReactorServiceOptions,
    locked: bool,
) -> Result<ReactorServiceReport, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    let exec_config = default_exec_config_for_layout(&layout)?;
    verify_permissions_for_file(&layout.src_main, 1, &permissions)?;

    let source = fs::read_to_string(&layout.src_main)?;
    if !source.contains("on_event") {
        return Err(SdkError::MissingProject(
            "reactor mode requires `fn on_event(...)` in src/main.ocl".to_string(),
        ));
    }

    let manifest = fs::read_to_string(&layout.manifest)?;
    let runtime_budget = parse_runtime_budget_from_manifest(&manifest);

    if options.socket_listen.is_some() && !runtime_budget.std_net_enabled {
        return Err(SdkError::MissingProject(
            "socket runtime requires `[runtime] std_net = true` in Ocl.toml".to_string(),
        ));
    }

    if runtime_budget.actor_max > 1 {
        return Err(SdkError::MissingProject(
            "W1 only supports single-actor runtime (`actor_max` must be 1)".to_string(),
        ));
    }

    let events_per_tick = match options.runtime_mode {
        ReactorRuntimeMode::Deterministic => 1u32,
        ReactorRuntimeMode::Throughput => runtime_budget.io_max_events_per_tick.max(1),
    };

    let domain_selection = resolve_domain_selection_v1(
        root,
        locked,
        options.universe_id.as_deref(),
        options.domain_id.as_deref(),
    )?;
    let hive_caps = match &options.hive_caps {
        Some(caps) => caps.clone(),
        None => resolve_hive_caps_v1(root, locked, options.universe_id.as_deref())?,
    };
    let mut runtime_universe_id = domain_selection.universe_id.clone();
    let runtime_domain_id = domain_selection.domain_id.clone();
    let mut domain_count = 1u32;
    let mut bridge_states: HashMap<String, BridgeRuntimeStateV1> = HashMap::new();
    let mut hive_domain_states: HashMap<String, HiveDomainRuntimeStateV1> = HashMap::new();

    let mut total_steps = 0u32;
    let mut event_count = 0u32;
    let mut event_id = 0u64;
    let mut mailbox_high_water = 0u32;
    let mut backpressure_count = 0u32;
    let mut bridge_emit_count = 0u32;
    let mut bridge_dispatch_count = 0u32;
    let mut bridge_backpressure_count = 0u32;
    let cancelled_count = 0u32;
    let mut deadline_exceeded_count = 0u32;
    let mut signatures = Vec::new();
    let mut workers_spawned_total = 0u32;
    let mut workers_reused_total = 0u32;
    let mut alloc_events_total = 0u32;
    let mut mailbox_max_depth_observed = 0u32;
    let mut fanout_drop_count = 0u32;
    let mut spawn_drop_count = 0u32;
    let mut dispatch_rows = Vec::<String>::new();
    let io_tape_replay_entries = match &options.io_tape_replay_path {
        Some(path) => Some(read_reactor_io_tape(path)?),
        None => None,
    };
    let mut io_tape_replay_cursor = 0usize;
    let mut io_tape_record_entries = Vec::<ReactorIoTapeEntry>::new();

    for tick in 0..options.ticks {
        let plan = resolve_bridge_runtime_plan_v1(
            root,
            locked,
            options.universe_id.as_deref(),
            options.domain_id.as_deref(),
            u64::from(tick),
        )?;
        runtime_universe_id = plan.universe_id.clone();
        let mut active_domains: Vec<String> = if options.domain_id.is_some() {
            vec![plan.domain_id.clone()]
        } else {
            plan.domains.iter().map(|d| d.id.clone()).collect()
        };
        if active_domains.is_empty() {
            active_domains.push(plan.domain_id.clone());
        }
        active_domains.sort();
        active_domains.dedup();
        domain_count = active_domains.len() as u32;
        if domain_count > hive_caps.max_domains {
            return Err(SdkError::LockMismatch(format!(
                "V-HIVE-DOMAIN-CAP: active domain count {} exceeds max_domains {}",
                domain_count, hive_caps.max_domains
            )));
        }
        let mut spawn_count_tick = 0u32;

        for (domain_rank, domain_id) in active_domains.iter().enumerate() {
            let mut mailbox_depth = 0u32;
            let domain_state = hive_domain_states.entry(domain_id.clone()).or_default();
            if let Some(tape_entries) = io_tape_replay_entries.as_ref() {
                while io_tape_replay_cursor < tape_entries.len() {
                    let tape_entry = &tape_entries[io_tape_replay_cursor];
                    if tape_entry.tick != tick || tape_entry.domain_id != *domain_id {
                        break;
                    }
                    mailbox_depth = mailbox_depth.saturating_add(1);
                    if mailbox_depth > runtime_budget.mailbox_max_depth {
                        backpressure_count = backpressure_count.saturating_add(1);
                        break;
                    }
                    event_count = event_count.saturating_add(1);
                    event_id = event_id.saturating_add(1);
                    let out = run_file_with_engine_config(
                        &layout.src_main,
                        1,
                        exec_config,
                        RunEngine::Interpreter,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    signatures.push(out.signature.clone());

                    let payload_hash256 = fnv1a64_hex(&out.signature);
                    if payload_hash256 != tape_entry.payload_hash256 {
                        return Err(SdkError::MissingProject(format!(
                            "io tape mismatch at tick={} domain={}: expected {} got {}",
                            tick, domain_id, tape_entry.payload_hash256, payload_hash256
                        )));
                    }
                    io_tape_record_entries.push(tape_entry.clone());
                    apply_hive_workload_for_event(
                        &hive_caps,
                        domain_state,
                        domain_rank,
                        tick,
                        event_id,
                        &payload_hash256,
                        &mut spawn_count_tick,
                        &mut mailbox_depth,
                        &mut backpressure_count,
                        &mut workers_spawned_total,
                        &mut workers_reused_total,
                        &mut alloc_events_total,
                        &mut fanout_drop_count,
                        &mut spawn_drop_count,
                        &mut mailbox_max_depth_observed,
                        &mut dispatch_rows,
                    );

                    for bridge in plan
                        .bridges
                        .iter()
                        .filter(|bridge| bridge.from_domain == *domain_id)
                    {
                        let state = bridge_states.entry(bridge.bridge_id.clone()).or_default();
                        match admit_bridge_emit_v1(state, bridge, u64::from(tick), &payload_hash256)
                        {
                            Ok(_) => {
                                bridge_emit_count = bridge_emit_count.saturating_add(1);
                            }
                            Err(err) => {
                                backpressure_count = backpressure_count.saturating_add(1);
                                if err.to_string().contains("V-DOMAIN-BRIDGE-BACKPRESSURE") {
                                    bridge_backpressure_count =
                                        bridge_backpressure_count.saturating_add(1);
                                }
                            }
                        }
                    }

                    io_tape_replay_cursor = io_tape_replay_cursor.saturating_add(1);
                }
            } else {
                for _ in 0..events_per_tick {
                    mailbox_depth = mailbox_depth.saturating_add(1);
                    if mailbox_depth > runtime_budget.mailbox_max_depth {
                        backpressure_count = backpressure_count.saturating_add(1);
                        break;
                    }
                    event_count = event_count.saturating_add(1);
                    event_id = event_id.saturating_add(1);
                    let out = run_file_with_engine_config(
                        &layout.src_main,
                        1,
                        exec_config,
                        RunEngine::Interpreter,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    signatures.push(out.signature.clone());

                    let payload_hash256 = fnv1a64_hex(&out.signature);
                    io_tape_record_entries.push(ReactorIoTapeEntry {
                        tick,
                        domain_id: domain_id.clone(),
                        payload_hash256: payload_hash256.clone(),
                    });
                    apply_hive_workload_for_event(
                        &hive_caps,
                        domain_state,
                        domain_rank,
                        tick,
                        event_id,
                        &payload_hash256,
                        &mut spawn_count_tick,
                        &mut mailbox_depth,
                        &mut backpressure_count,
                        &mut workers_spawned_total,
                        &mut workers_reused_total,
                        &mut alloc_events_total,
                        &mut fanout_drop_count,
                        &mut spawn_drop_count,
                        &mut mailbox_max_depth_observed,
                        &mut dispatch_rows,
                    );

                    for bridge in plan
                        .bridges
                        .iter()
                        .filter(|bridge| bridge.from_domain == *domain_id)
                    {
                        let state = bridge_states.entry(bridge.bridge_id.clone()).or_default();
                        match admit_bridge_emit_v1(state, bridge, u64::from(tick), &payload_hash256)
                        {
                            Ok(_) => {
                                bridge_emit_count = bridge_emit_count.saturating_add(1);
                            }
                            Err(err) => {
                                backpressure_count = backpressure_count.saturating_add(1);
                                if err.to_string().contains("V-DOMAIN-BRIDGE-BACKPRESSURE") {
                                    bridge_backpressure_count =
                                        bridge_backpressure_count.saturating_add(1);
                                }
                            }
                        }
                    }
                }
            }
            mailbox_high_water = mailbox_high_water.max(mailbox_depth);
        }

        if !plan.bridges.is_empty() {
            let start = plan.dispatch_start_index;
            for offset in 0..plan.bridges.len() {
                let idx = (start + offset) % plan.bridges.len();
                let bridge = &plan.bridges[idx];
                let state = bridge_states.entry(bridge.bridge_id.clone()).or_default();
                while poll_bridge_event_v1(state, bridge, u64::from(tick)).is_some() {
                    bridge_dispatch_count = bridge_dispatch_count.saturating_add(1);
                }
            }
        }
    }

    if let Some(tape_entries) = io_tape_replay_entries.as_ref() {
        if io_tape_replay_cursor != tape_entries.len() {
            return Err(SdkError::MissingProject(
                "io tape has unconsumed entries for current reactor run".to_string(),
            ));
        }
    }

    if let Some(record_path) = &options.io_tape_record_path {
        write_reactor_io_tape(record_path, &io_tape_record_entries)?;
    }

    if event_count > runtime_budget.default_deadline_ms {
        deadline_exceeded_count = 1;
    }
    let workers_alive: u32 = hive_domain_states.values().map(|s| s.active_workers).sum();
    let dispatch_digest256 = if dispatch_rows.is_empty() {
        blake3::hash(b"dispatch.empty").to_hex().to_string()
    } else {
        blake3::hash(dispatch_rows.join("\n").as_bytes())
            .to_hex()
            .to_string()
    };

    let mut runtime_report_written = false;
    if let Some(report_path) = &options.runtime_report {
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let report_json = format!(
            concat!(
                "{{",
                "\"mode\":\"{}\",",
                "\"ticks\":{},",
                "\"event_count\":{},",
                "\"event_id_end\":{},",
                "\"total_steps\":{},",
                "\"mailbox_high_water\":{},",
                "\"backpressure_count\":{},",
                "\"cancelled_count\":{},",
                "\"deadline_exceeded_count\":{},",
                "\"universe_id\":\"{}\",",
                "\"domain_id\":\"{}\",",
                "\"domain_count\":{},",
                "\"bridge_emit_count\":{},",
                "\"bridge_dispatch_count\":{},",
                "\"bridge_backpressure_count\":{},",
                "\"workers_alive\":{},",
                "\"workers_spawned_total\":{},",
                "\"workers_reused_total\":{},",
                "\"mailbox_max_depth_observed\":{},",
                "\"alloc_events_total\":{},",
                "\"fanout_drop_count\":{},",
                "\"spawn_drop_count\":{},",
                "\"dispatch_digest256\":\"{}\",",
                "\"socket_listen\":{}",
                "}}"
            ),
            options.runtime_mode.as_str(),
            options.ticks,
            event_count,
            event_id,
            total_steps,
            mailbox_high_water,
            backpressure_count,
            cancelled_count,
            deadline_exceeded_count,
            runtime_universe_id,
            runtime_domain_id,
            domain_count,
            bridge_emit_count,
            bridge_dispatch_count,
            bridge_backpressure_count,
            workers_alive,
            workers_spawned_total,
            workers_reused_total,
            mailbox_max_depth_observed,
            alloc_events_total,
            fanout_drop_count,
            spawn_drop_count,
            dispatch_digest256,
            match &options.socket_listen {
                Some(v) => format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")),
                None => "null".to_string(),
            }
        );
        fs::write(report_path, report_json)?;
        runtime_report_written = true;
    }

    let mut replay_audit_written = false;
    let mut audit_chain_hash = None;
    if let Some(audit_path) = &options.replay_audit {
        let entries = build_audit_entries(&signatures);
        verify_audit_entries(&entries)?;
        write_audit_jsonl(audit_path, &entries)?;
        replay_audit_written = true;
        audit_chain_hash = entries.last().map(|e| e.hash.clone());
    }

    Ok(ReactorServiceReport {
        ticks: options.ticks,
        total_steps,
        mode: options.runtime_mode,
        event_count,
        event_id_end: event_id,
        mailbox_high_water,
        backpressure_count,
        cancelled_count,
        deadline_exceeded_count,
        socket_listen: options.socket_listen.clone(),
        runtime_report_written,
        replay_audit_written,
        audit_chain_hash,
        universe_id: runtime_universe_id,
        domain_id: runtime_domain_id,
        domain_count,
        bridge_emit_count,
        bridge_dispatch_count,
        bridge_backpressure_count,
        workers_alive,
        workers_spawned_total,
        workers_reused_total,
        mailbox_max_depth_observed,
        alloc_events_total,
        fanout_drop_count,
        spawn_drop_count,
        dispatch_digest256,
    })
}

pub fn run_reactor_service(
    root: &Path,
    options: &ReactorServiceOptions,
) -> Result<ReactorServiceReport, SdkError> {
    run_reactor_service_with_lock(root, options, false)
}

pub fn run_reactor_ticks_with_lock(
    root: &Path,
    ticks: u32,
    locked: bool,
) -> Result<ReactorSummary, SdkError> {
    let report = run_reactor_service_with_lock(
        root,
        &ReactorServiceOptions {
            ticks,
            ..ReactorServiceOptions::default()
        },
        locked,
    )?;
    Ok(ReactorSummary {
        ticks: report.ticks,
        total_steps: report.total_steps,
    })
}

pub fn test_project(root: &Path) -> Result<TestSummary, SdkError> {
    test_project_with_lock(root, false)
}

pub fn test_project_with_lock(root: &Path, locked: bool) -> Result<TestSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    let exec_config = default_exec_config_for_layout(&layout)?;
    let mut tests = Vec::new();
    collect_ocl_files(&layout.tests_dir, &mut tests)?;
    tests.sort();
    for (idx, test_file) in tests.iter().enumerate() {
        verify_permissions_for_file(test_file, idx as u32 + 100, &permissions)?;
        run_file_with_engine_config(test_file, idx as u32 + 100, exec_config, RunEngine::Interpreter)?;
    }
    Ok(TestSummary {
        tests_run: tests.len(),
    })
}

pub fn fmt_project(root: &Path, check_only: bool) -> Result<FmtSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let files = gather_project_ocl_files(&layout)?;
    let mut touched = 0usize;
    let mut mismatches = Vec::new();

    for file in files {
        let original = fs::read_to_string(&file)?;
        let formatted = normalize_text(&original);
        if original != formatted {
            touched += 1;
            if check_only {
                mismatches.push(file.to_string_lossy().to_string());
            } else {
                fs::write(&file, formatted)?;
            }
        }
    }

    if !mismatches.is_empty() {
        return Err(SdkError::FmtMismatch(mismatches));
    }

    Ok(FmtSummary {
        files_touched: touched,
    })
}

pub fn build_project(root: &Path) -> Result<BuildSummary, SdkError> {
    build_project_with_lock(root, false)
}

pub fn build_project_with_lock(root: &Path, locked: bool) -> Result<BuildSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let lock_deps = if locked {
        verify_lock_consistency(&layout)?
    } else {
        read_expected_lock(&layout)?
    };
    let files = gather_project_ocl_files(&layout)?;
    fs::create_dir_all(&layout.bundle_dir)?;

    let vendor_root = layout.bundle_dir.join("files");
    fs::create_dir_all(&vendor_root)?;
    let mut manifest_lines = Vec::new();

    for file in &files {
        let rel = file
            .strip_prefix(&layout.root)
            .map_err(|_| SdkError::MissingProject("invalid project path layout".to_string()))?;
        let rel_display = rel.to_string_lossy().replace('\\', "/");
        manifest_lines.push(rel_display.clone());
        let dst = vendor_root.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, dst)?;
    }

    manifest_lines.sort();
    let mut manifest_body = String::new();
    manifest_body.push_str("version=1\n");
    for line in manifest_lines {
        manifest_body.push_str("file=");
        manifest_body.push_str(&line);
        manifest_body.push('\n');
    }
    for dep in lock_deps {
        manifest_body.push_str("dep=");
        manifest_body.push_str(&dep.name);
        manifest_body.push('|');
        manifest_body.push_str(&dep.version);
        manifest_body.push('|');
        manifest_body.push_str(&dep.hash64);
        manifest_body.push('\n');
    }
    fs::write(layout.bundle_dir.join("manifest.txt"), manifest_body)?;

    Ok(BuildSummary {
        files_bundled: files.len(),
    })
}

fn parse_package_name_version(manifest: &str) -> Result<(String, String), SdkError> {
    let mut in_package = false;
    let mut name = None::<String>;
    let mut version = None::<String>;

    for raw in manifest.lines() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "name" => name = Some(value),
            "version" => version = Some(value),
            _ => {}
        }
    }

    let Some(name) = name else {
        return Err(SdkError::MissingProject(
            "Ocl.toml missing [package].name".to_string(),
        ));
    };
    let Some(version) = version else {
        return Err(SdkError::MissingProject(
            "Ocl.toml missing [package].version".to_string(),
        ));
    };

    Ok((name, version))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedOclPkg {
    package_name: String,
    version: String,
    entry: String,
    payload: String,
    payload_hash_blake3: String,
    signature_ed25519_b64: String,
    signer_pub_ed25519_b64: String,
    files: Vec<(String, String)>,
    deps: Vec<LockDepV2>,
}

fn parse_oclpkg(path: &Path) -> Result<ParsedOclPkg, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut lines = raw.lines();
    if lines.next() != Some("OCLPKGv1") {
        return Err(SdkError::SupplyInvalid("invalid .oclpkg magic".to_string()));
    }
    let Some(hash_line) = lines.next() else {
        return Err(SdkError::SupplyInvalid(
            "missing payload hash line".to_string(),
        ));
    };
    let Some(sig_line) = lines.next() else {
        return Err(SdkError::SupplyInvalid(
            "missing signature line".to_string(),
        ));
    };
    let Some(pub_line) = lines.next() else {
        return Err(SdkError::SupplyInvalid(
            "missing signer public key line".to_string(),
        ));
    };

    let Some(payload_hash_blake3) = hash_line.strip_prefix("payload_hash_blake3=") else {
        return Err(SdkError::SupplyInvalid(
            "invalid payload hash line".to_string(),
        ));
    };
    let Some(signature_ed25519_b64) = sig_line.strip_prefix("signature_ed25519=") else {
        return Err(SdkError::SupplyInvalid(
            "invalid signature line".to_string(),
        ));
    };
    let Some(signer_pub_ed25519_b64) = pub_line.strip_prefix("signer_pub_ed25519=") else {
        return Err(SdkError::SupplyInvalid(
            "invalid signer public key line".to_string(),
        ));
    };

    let payload = lines.collect::<Vec<&str>>().join("\n") + "\n";
    let mut package_name = None::<String>;
    let mut version = None::<String>;
    let mut entry = None::<String>;
    let mut files = Vec::new();
    let mut deps = Vec::new();

    for line in payload.lines() {
        if let Some(v) = line.strip_prefix("name=") {
            package_name = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("version=") {
            version = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("entry=") {
            entry = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("file=") {
            let Some((rel, content_b64)) = v.split_once('|') else {
                return Err(SdkError::SupplyInvalid(
                    "invalid file line in .oclpkg".to_string(),
                ));
            };
            files.push((rel.to_string(), content_b64.to_string()));
        } else if let Some(v) = line.strip_prefix("dep=") {
            let parts: Vec<&str> = v.split('|').collect();
            if parts.len() != 5 {
                return Err(SdkError::SupplyInvalid(
                    "invalid dep line in .oclpkg".to_string(),
                ));
            }
            deps.push(LockDepV2 {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                hash64: parts[2].to_string(),
                signature_b64: parts[3].to_string(),
                signer_pub_b64: parts[4].to_string(),
            });
        }
    }

    let Some(package_name) = package_name else {
        return Err(SdkError::SupplyInvalid("missing package name".to_string()));
    };
    let Some(version) = version else {
        return Err(SdkError::SupplyInvalid(
            "missing package version".to_string(),
        ));
    };
    let Some(entry) = entry else {
        return Err(SdkError::SupplyInvalid("missing entry".to_string()));
    };

    Ok(ParsedOclPkg {
        package_name,
        version,
        entry,
        payload,
        payload_hash_blake3: payload_hash_blake3.to_string(),
        signature_ed25519_b64: signature_ed25519_b64.to_string(),
        signer_pub_ed25519_b64: signer_pub_ed25519_b64.to_string(),
        files,
        deps,
    })
}

pub fn verify_supply_artifact(path: &Path) -> Result<SupplyVerifySummary, SdkError> {
    let parsed = parse_oclpkg(path)?;
    let hash = hash256_hex(parsed.payload.as_bytes());
    if hash != parsed.payload_hash_blake3 {
        return Err(SdkError::SupplyInvalid(format!(
            "payload hash mismatch: expected {} got {}",
            parsed.payload_hash_blake3, hash
        )));
    }
    deterministic_verify(
        parsed.payload_hash_blake3.as_bytes(),
        &parsed.signature_ed25519_b64,
        &parsed.signer_pub_ed25519_b64,
    )?;
    verify_lock_v2_signatures(&parsed.deps)?;
    Ok(SupplyVerifySummary {
        valid: true,
        package_name: parsed.package_name,
        payload_hash_blake3: parsed.payload_hash_blake3,
    })
}

pub fn build_oclpkg(root: &Path) -> Result<BuildOclPkgSummary, SdkError> {
    build_oclpkg_with_lock(root, false)
}

pub fn build_oclpkg_with_lock(root: &Path, locked: bool) -> Result<BuildOclPkgSummary, SdkError> {
    let build = build_project_with_lock(root, locked)?;
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let lock_deps = if locked {
        verify_lock_consistency(&layout)?
    } else {
        read_expected_lock(&layout)?
    };

    let lock_v2_path = layout.root.join("deps.lock.v2");
    if !lock_v2_path.exists() {
        let deps_v2 = to_lock_deps_v2(&lock_deps);
        fs::write(&lock_v2_path, encode_lock_v2(&deps_v2))?;
    }
    let lock_v2_raw = fs::read_to_string(&lock_v2_path)?;
    let lock_v2 = parse_lock_v2(&lock_v2_raw)?;
    if locked {
        verify_lock_v2_signatures(&lock_v2)?;
    }

    let manifest = fs::read_to_string(&layout.manifest)?;
    let (package_name, version) = parse_package_name_version(&manifest)?;
    let files = gather_project_ocl_files(&layout)?;

    let mut payload = String::new();
    payload.push_str("name=");
    payload.push_str(&package_name);
    payload.push('\n');
    payload.push_str("version=");
    payload.push_str(&version);
    payload.push('\n');
    payload.push_str("entry=src/main.ocl\n");
    payload.push_str("files=");
    payload.push_str(&files.len().to_string());
    payload.push('\n');

    for file in &files {
        let rel = file
            .strip_prefix(&layout.root)
            .map_err(|_| SdkError::MissingProject("invalid project path layout".to_string()))?;
        let rel_text = rel.to_string_lossy().replace('\\', "/");
        let content = fs::read(file)?;
        let content_b64 = bytes_to_hex(&content);
        payload.push_str("file=");
        payload.push_str(&rel_text);
        payload.push('|');
        payload.push_str(&content_b64);
        payload.push('\n');
    }

    payload.push_str("deps=");
    payload.push_str(&lock_v2.len().to_string());
    payload.push('\n');
    for dep in &lock_v2 {
        payload.push_str("dep=");
        payload.push_str(&dep.name);
        payload.push('|');
        payload.push_str(&dep.version);
        payload.push('|');
        payload.push_str(&dep.hash64);
        payload.push('|');
        payload.push_str(&dep.signature_b64);
        payload.push('|');
        payload.push_str(&dep.signer_pub_b64);
        payload.push('\n');
    }

    let payload_hash_blake3 = hash256_hex(payload.as_bytes());
    let (signature_ed25519_b64, signer_pub_ed25519_b64) =
        deterministic_sign("artifact-v1", payload_hash_blake3.as_bytes());

    let mut oclpkg = String::new();
    oclpkg.push_str("OCLPKGv1\n");
    oclpkg.push_str("payload_hash_blake3=");
    oclpkg.push_str(&payload_hash_blake3);
    oclpkg.push('\n');
    oclpkg.push_str("signature_ed25519=");
    oclpkg.push_str(&signature_ed25519_b64);
    oclpkg.push('\n');
    oclpkg.push_str("signer_pub_ed25519=");
    oclpkg.push_str(&signer_pub_ed25519_b64);
    oclpkg.push('\n');
    oclpkg.push_str(&payload);

    let artifact_dir = layout.root.join(".oclpkg");
    fs::create_dir_all(&artifact_dir)?;
    let artifact_path = artifact_dir.join(format!("{package_name}-{version}.oclpkg"));
    fs::write(&artifact_path, oclpkg)?;

    Ok(BuildOclPkgSummary {
        artifact_path,
        files_bundled: build.files_bundled,
        payload_hash_blake3,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryEntry {
    artifact_file: String,
    package_name: String,
    version: String,
    payload_hash_blake3: String,
    signature_ed25519_b64: String,
    signer_pub_ed25519_b64: String,
}

fn parse_registry_index(raw: &str) -> Result<Vec<RegistryEntry>, SdkError> {
    let mut has_version = false;
    let mut entries = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "version=2" {
            has_version = true;
            continue;
        }
        let Some(v) = trimmed.strip_prefix("artifact=") else {
            return Err(SdkError::SupplyInvalid(
                "invalid registry.index.v2 line".to_string(),
            ));
        };
        let parts: Vec<&str> = v.split('|').collect();
        if parts.len() != 6 {
            return Err(SdkError::SupplyInvalid(
                "invalid artifact entry in registry.index.v2".to_string(),
            ));
        }
        entries.push(RegistryEntry {
            artifact_file: parts[0].to_string(),
            package_name: parts[1].to_string(),
            version: parts[2].to_string(),
            payload_hash_blake3: parts[3].to_string(),
            signature_ed25519_b64: parts[4].to_string(),
            signer_pub_ed25519_b64: parts[5].to_string(),
        });
    }
    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "registry.index.v2 missing `version=2`".to_string(),
        ));
    }
    Ok(entries)
}

fn encode_registry_index(entries: &[RegistryEntry]) -> String {
    let mut out = String::from("version=2\n");
    for entry in entries {
        out.push_str("artifact=");
        out.push_str(&entry.artifact_file);
        out.push('|');
        out.push_str(&entry.package_name);
        out.push('|');
        out.push_str(&entry.version);
        out.push('|');
        out.push_str(&entry.payload_hash_blake3);
        out.push('|');
        out.push_str(&entry.signature_ed25519_b64);
        out.push('|');
        out.push_str(&entry.signer_pub_ed25519_b64);
        out.push('\n');
    }
    out
}

pub fn publish_artifact(
    artifact_path: &Path,
    registry_dir: &Path,
) -> Result<PublishSummary, SdkError> {
    let verify = verify_supply_artifact(artifact_path)?;
    let parsed = parse_oclpkg(artifact_path)?;

    let artifacts_dir = registry_dir.join("artifacts");
    fs::create_dir_all(&artifacts_dir)?;
    let artifact_file = artifact_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| SdkError::SupplyInvalid("invalid artifact file name".to_string()))?
        .to_string();
    let dst = artifacts_dir.join(&artifact_file);
    fs::copy(artifact_path, &dst)?;

    let index_path = registry_dir.join("registry.index.v2");
    let mut entries = if index_path.exists() {
        parse_registry_index(&fs::read_to_string(&index_path)?)?
    } else {
        Vec::new()
    };
    entries.retain(|e| e.artifact_file != artifact_file);
    entries.push(RegistryEntry {
        artifact_file,
        package_name: parsed.package_name.clone(),
        version: parsed.version.clone(),
        payload_hash_blake3: parsed.payload_hash_blake3.clone(),
        signature_ed25519_b64: parsed.signature_ed25519_b64.clone(),
        signer_pub_ed25519_b64: parsed.signer_pub_ed25519_b64.clone(),
    });
    entries.sort_by(|a, b| a.artifact_file.cmp(&b.artifact_file));
    fs::write(&index_path, encode_registry_index(&entries))?;

    Ok(PublishSummary {
        artifact_path: dst,
        registry_index: index_path,
        package_name: verify.package_name,
        payload_hash_blake3: verify.payload_hash_blake3,
    })
}

pub fn fetch_artifact(
    artifact_or_package: &str,
    registry_dir: &Path,
    out_dir: &Path,
) -> Result<FetchSummary, SdkError> {
    let index_path = registry_dir.join("registry.index.v2");
    if !index_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing registry index: {}",
            index_path.display()
        )));
    }
    let entries = parse_registry_index(&fs::read_to_string(&index_path)?)?;
    let target = if artifact_or_package.ends_with(".oclpkg") {
        entries
            .into_iter()
            .find(|e| e.artifact_file == artifact_or_package)
    } else {
        entries
            .into_iter()
            .find(|e| e.package_name == artifact_or_package)
    }
    .ok_or_else(|| {
        SdkError::SupplyInvalid(format!(
            "artifact/package `{artifact_or_package}` not found in registry"
        ))
    })?;

    let src = registry_dir.join("artifacts").join(&target.artifact_file);
    if !src.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing artifact in registry storage: {}",
            src.display()
        )));
    }
    fs::create_dir_all(out_dir)?;
    let dst = out_dir.join(&target.artifact_file);
    fs::copy(&src, &dst)?;

    let verify = verify_supply_artifact(&dst)?;
    if verify.payload_hash_blake3 != target.payload_hash_blake3 {
        return Err(SdkError::SupplyInvalid(
            "fetched artifact hash differs from registry index".to_string(),
        ));
    }

    Ok(FetchSummary {
        artifact_path: dst,
        package_name: target.package_name,
    })
}

pub fn run_artifact(
    artifact_path: &Path,
    run_engine: RunEngine,
    step_cap: u32,
) -> Result<RunSummary, SdkError> {
    verify_supply_artifact(artifact_path)?;
    let parsed = parse_oclpkg(artifact_path)?;
    let entry = parsed.entry;
    let source_b64 = parsed
        .files
        .iter()
        .find_map(|(rel, b64)| (rel == &entry).then_some(b64))
        .ok_or_else(|| SdkError::SupplyInvalid(format!("entry file `{entry}` not found")))?;

    let source_bytes = hex_to_bytes(source_b64)?;
    let source_text = String::from_utf8(source_bytes)
        .map_err(|e| SdkError::SupplyInvalid(format!("entry source is not utf8: {e}")))?;
    let out = run_source_with_engine(&source_text, 1, step_cap, run_engine)
        .map_err(RuntimeCoreError::from)?;
    Ok(RunSummary { steps: out.steps })
}

pub fn build_run_id_deterministic(
    scope: &str,
    root: &Path,
    run_engine: RunEngine,
    runtime_mode: Option<ReactorRuntimeMode>,
    ticks: Option<u32>,
) -> String {
    let mode = runtime_mode.map(|m| m.as_str()).unwrap_or("-");
    let ticks = ticks.unwrap_or(0);
    let seed = format!(
        "scope={scope}|root={}|engine={}|mode={mode}|ticks={ticks}",
        root.to_string_lossy(),
        run_engine.as_str()
    );
    fnv1a64_hex(&seed)
}

pub fn run_project_with_trace_engine_and_lock(
    root: &Path,
    run_engine: RunEngine,
    locked: bool,
) -> Result<TraceRunSummary, SdkError> {
    run_project_with_trace_engine_config_and_lock(
        root,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::Normal,
            ..ExecConfig::default()
        },
    )
}

pub fn run_project_with_trace_engine_config_and_lock(
    root: &Path,
    run_engine: RunEngine,
    locked: bool,
    config: ExecConfig,
) -> Result<TraceRunSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_for_file(&layout.src_main, 1, &permissions)?;
    let mut runtime_config = config;
    runtime_config.guard_mode = load_project_language_config_for_layout(&layout)?.guard_mode;

    let out = run_file_with_engine_config(&layout.src_main, 1, runtime_config, run_engine)?;
    let run_id = build_run_id_deterministic("project", root, run_engine, None, None);
    let mut seq = 1u64;
    let mut events = Vec::new();
    append_trace_events(
        &run_id,
        TRACE_UNIVERSE_SENTINEL,
        TRACE_DOMAIN_SENTINEL,
        &mut seq,
        &out.trace.events,
        &mut events,
    );
    Ok(TraceRunSummary {
        run_id,
        total_steps: out.steps,
        events,
    })
}

pub fn run_reactor_service_with_trace_engine_and_lock(
    root: &Path,
    options: &ReactorServiceOptions,
    run_engine: RunEngine,
    locked: bool,
) -> Result<TraceRunSummary, SdkError> {
    run_reactor_service_with_trace_engine_config_and_lock(
        root,
        options,
        run_engine,
        locked,
        ExecConfig {
            step_cap: 4096,
            commit_policy: CommitPolicyMode::Normal,
            ..ExecConfig::default()
        },
    )
}

pub fn run_reactor_service_with_trace_engine_config_and_lock(
    root: &Path,
    options: &ReactorServiceOptions,
    run_engine: RunEngine,
    locked: bool,
    config: ExecConfig,
) -> Result<TraceRunSummary, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_for_file(&layout.src_main, 1, &permissions)?;
    let mut runtime_config = config;
    runtime_config.guard_mode = load_project_language_config_for_layout(&layout)?.guard_mode;

    let source = fs::read_to_string(&layout.src_main)?;
    if !source.contains("on_event") {
        return Err(SdkError::MissingProject(
            "reactor mode requires `fn on_event(...)` in src/main.ocl".to_string(),
        ));
    }

    let manifest = fs::read_to_string(&layout.manifest)?;
    let runtime_budget = parse_runtime_budget_from_manifest(&manifest);
    if options.socket_listen.is_some() && !runtime_budget.std_net_enabled {
        return Err(SdkError::MissingProject(
            "socket runtime requires `[runtime] std_net = true` in Ocl.toml".to_string(),
        ));
    }
    if runtime_budget.actor_max > 1 {
        return Err(SdkError::MissingProject(
            "W1 only supports single-actor runtime (`actor_max` must be 1)".to_string(),
        ));
    }

    let events_per_tick = match options.runtime_mode {
        ReactorRuntimeMode::Deterministic => 1u32,
        ReactorRuntimeMode::Throughput => runtime_budget.io_max_events_per_tick.max(1),
    };
    let domain_selection = resolve_domain_selection_v1(
        root,
        locked,
        options.universe_id.as_deref(),
        options.domain_id.as_deref(),
    )?;
    let run_id = build_run_id_deterministic(
        "reactor",
        root,
        run_engine,
        Some(options.runtime_mode),
        Some(options.ticks),
    );
    let mut seq = 1u64;
    let mut events = Vec::new();
    let mut total_steps = 0u32;
    let io_tape_replay_entries = match &options.io_tape_replay_path {
        Some(path) => Some(read_reactor_io_tape(path)?),
        None => None,
    };
    let mut io_tape_replay_cursor = 0usize;
    let mut io_tape_record_entries = Vec::<ReactorIoTapeEntry>::new();

    for tick in 0..options.ticks {
        let plan = resolve_bridge_runtime_plan_v1(
            root,
            locked,
            options.universe_id.as_deref(),
            options.domain_id.as_deref(),
            u64::from(tick),
        )?;
        let mut active_domains: Vec<String> = if options.domain_id.is_some() {
            vec![plan.domain_id.clone()]
        } else {
            plan.domains.iter().map(|d| d.id.clone()).collect()
        };
        if active_domains.is_empty() {
            active_domains.push(domain_selection.domain_id.clone());
        }
        active_domains.sort();
        active_domains.dedup();
        for domain_id in &active_domains {
            if let Some(tape_entries) = io_tape_replay_entries.as_ref() {
                while io_tape_replay_cursor < tape_entries.len() {
                    let tape_entry = &tape_entries[io_tape_replay_cursor];
                    if tape_entry.tick != tick || tape_entry.domain_id != *domain_id {
                        break;
                    }
                    let out = run_file_with_engine_config(
                        &layout.src_main,
                        1,
                        runtime_config,
                        run_engine,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    let payload_hash256 = fnv1a64_hex(&out.signature);
                    if payload_hash256 != tape_entry.payload_hash256 {
                        return Err(SdkError::MissingProject(format!(
                            "io tape mismatch at tick={} domain={}: expected {} got {}",
                            tick, domain_id, tape_entry.payload_hash256, payload_hash256
                        )));
                    }
                    io_tape_record_entries.push(tape_entry.clone());
                    append_trace_events(
                        &run_id,
                        &plan.universe_id,
                        domain_id,
                        &mut seq,
                        &out.trace.events,
                        &mut events,
                    );
                    io_tape_replay_cursor = io_tape_replay_cursor.saturating_add(1);
                }
            } else {
                for _ in 0..events_per_tick {
                    let out = run_file_with_engine_config(
                        &layout.src_main,
                        1,
                        runtime_config,
                        run_engine,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    io_tape_record_entries.push(ReactorIoTapeEntry {
                        tick,
                        domain_id: domain_id.clone(),
                        payload_hash256: fnv1a64_hex(&out.signature),
                    });
                    append_trace_events(
                        &run_id,
                        &plan.universe_id,
                        domain_id,
                        &mut seq,
                        &out.trace.events,
                        &mut events,
                    );
                }
            }
        }
    }

    if let Some(tape_entries) = io_tape_replay_entries.as_ref() {
        if io_tape_replay_cursor != tape_entries.len() {
            return Err(SdkError::MissingProject(
                "io tape has unconsumed entries for current trace reactor run".to_string(),
            ));
        }
    }
    if let Some(record_path) = &options.io_tape_record_path {
        write_reactor_io_tape(record_path, &io_tape_record_entries)?;
    }

    Ok(TraceRunSummary {
        run_id,
        total_steps,
        events,
    })
}

pub fn trace_required_digest(events: &[TraceEventV1]) -> String {
    let mut payload = String::new();
    for event in events {
        payload.push_str(&event.seq.to_string());
        payload.push('|');
        payload.push_str(&event.event);
        payload.push('|');
        payload.push_str(&event.payload_hash);
        payload.push('\n');
    }
    fnv1a64_hex(&payload)
}

pub fn write_trace_jsonl(path: &Path, events: &[TraceEventV1]) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut out = String::new();
    for event in events {
        out.push_str(&encode_trace_line(event));
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(())
}

pub fn read_trace_jsonl(path: &Path) -> Result<Vec<TraceEventV1>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(decode_trace_line(trimmed)?);
    }
    Ok(out)
}

pub fn build_profile_from_trace(
    run_id: &str,
    total_steps: u32,
    events: &[TraceEventV1],
) -> ProfileReportV1 {
    let mut by_key: HashMap<String, ProfileKeyCostV1> = HashMap::new();
    let mut observe_count = 0u32;

    for event in events {
        if event.event != "observe_end" {
            continue;
        }
        let Some(key) = event.key.clone() else {
            continue;
        };
        observe_count = observe_count.saturating_add(1);
        let entry = by_key
            .entry(key.clone())
            .or_insert_with(|| ProfileKeyCostV1 {
                key,
                ..ProfileKeyCostV1::default()
            });
        entry.observe_count = entry.observe_count.saturating_add(1);
        match event.kind.as_deref() {
            Some("ok") => entry.ok_count = entry.ok_count.saturating_add(1),
            Some("degraded") => entry.degraded_count = entry.degraded_count.saturating_add(1),
            Some("insufficient") => {
                entry.insufficient_count = entry.insufficient_count.saturating_add(1);
            }
            Some("deferred") => entry.deferred_count = entry.deferred_count.saturating_add(1),
            _ => {}
        }
    }

    let mut key_costs: Vec<ProfileKeyCostV1> = by_key.into_values().collect();
    key_costs.sort_by(|a, b| {
        b.observe_count
            .cmp(&a.observe_count)
            .then_with(|| a.key.cmp(&b.key))
    });

    let event_count = events.len() as u32;
    let eval_steps_est = event_count;
    let alloc_units_est = observe_count.saturating_mul(4);
    let required_digest = trace_required_digest(events);

    ProfileReportV1 {
        run_id: run_id.to_string(),
        event_count,
        observe_count,
        eval_steps_est,
        alloc_units_est,
        total_steps,
        required_digest,
        key_costs,
    }
}

pub fn write_profile_json(path: &Path, report: &ProfileReportV1) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut out = String::new();
    out.push_str("version=1\n");
    out.push_str("run_id=");
    out.push_str(&report.run_id);
    out.push('\n');
    out.push_str("event_count=");
    out.push_str(&report.event_count.to_string());
    out.push('\n');
    out.push_str("observe_count=");
    out.push_str(&report.observe_count.to_string());
    out.push('\n');
    out.push_str("eval_steps_est=");
    out.push_str(&report.eval_steps_est.to_string());
    out.push('\n');
    out.push_str("alloc_units_est=");
    out.push_str(&report.alloc_units_est.to_string());
    out.push('\n');
    out.push_str("total_steps=");
    out.push_str(&report.total_steps.to_string());
    out.push('\n');
    out.push_str("required_digest=");
    out.push_str(&report.required_digest);
    out.push('\n');
    for key_cost in &report.key_costs {
        out.push_str("key_cost=");
        out.push_str(&key_cost.key);
        out.push('|');
        out.push_str(&key_cost.observe_count.to_string());
        out.push('|');
        out.push_str(&key_cost.ok_count.to_string());
        out.push('|');
        out.push_str(&key_cost.degraded_count.to_string());
        out.push('|');
        out.push_str(&key_cost.insufficient_count.to_string());
        out.push('|');
        out.push_str(&key_cost.deferred_count.to_string());
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(())
}

pub fn read_profile_json(path: &Path) -> Result<ProfileReportV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut run_id = None::<String>;
    let mut event_count = 0u32;
    let mut observe_count = 0u32;
    let mut eval_steps_est = 0u32;
    let mut alloc_units_est = 0u32;
    let mut total_steps = 0u32;
    let mut required_digest = None::<String>;
    let mut key_costs = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "version=1" {
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("run_id=") {
            run_id = Some(v.to_string());
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("event_count=") {
            event_count = parse_u32(v, "event_count")?;
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("observe_count=") {
            observe_count = parse_u32(v, "observe_count")?;
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("eval_steps_est=") {
            eval_steps_est = parse_u32(v, "eval_steps_est")?;
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("alloc_units_est=") {
            alloc_units_est = parse_u32(v, "alloc_units_est")?;
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("total_steps=") {
            total_steps = parse_u32(v, "total_steps")?;
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("required_digest=") {
            required_digest = Some(v.to_string());
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("key_cost=") {
            let parts: Vec<&str> = v.split('|').collect();
            if parts.len() != 6 {
                return Err(SdkError::MissingProject(
                    "invalid profile key_cost row".to_string(),
                ));
            }
            key_costs.push(ProfileKeyCostV1 {
                key: parts[0].to_string(),
                observe_count: parse_u32(parts[1], "key.observe_count")?,
                ok_count: parse_u32(parts[2], "key.ok_count")?,
                degraded_count: parse_u32(parts[3], "key.degraded_count")?,
                insufficient_count: parse_u32(parts[4], "key.insufficient_count")?,
                deferred_count: parse_u32(parts[5], "key.deferred_count")?,
            });
        }
    }

    let run_id =
        run_id.ok_or_else(|| SdkError::MissingProject("profile missing `run_id`".to_string()))?;
    let required_digest = required_digest
        .ok_or_else(|| SdkError::MissingProject("profile missing `required_digest`".to_string()))?;

    Ok(ProfileReportV1 {
        run_id,
        event_count,
        observe_count,
        eval_steps_est,
        alloc_units_est,
        total_steps,
        required_digest,
        key_costs,
    })
}

pub fn render_trace_view(events: &[TraceEventV1], options: TraceViewOptions) -> TraceViewReport {
    let selected = if let Some(tail) = options.tail {
        let start = events.len().saturating_sub(tail);
        &events[start..]
    } else {
        events
    };
    let digest = trace_required_digest(selected);

    if options.json {
        let mut rendered = String::new();
        rendered.push_str("{\"event_count\":");
        rendered.push_str(&selected.len().to_string());
        rendered.push_str(",\"required_digest\":\"");
        rendered.push_str(&digest);
        rendered.push_str("\",\"events\":[");
        for (idx, event) in selected.iter().enumerate() {
            if idx > 0 {
                rendered.push(',');
            }
            rendered.push_str(&trace_event_to_json(event));
        }
        rendered.push_str("]}");
        return TraceViewReport {
            event_count: selected.len(),
            required_digest: digest,
            rendered,
        };
    }

    let mut rendered = String::new();
    rendered.push_str("trace view\n");
    rendered.push_str("event_count=");
    rendered.push_str(&selected.len().to_string());
    rendered.push('\n');
    rendered.push_str("required_digest=");
    rendered.push_str(&digest);
    rendered.push('\n');
    for event in selected {
        rendered.push_str(&format!(
            "#{} {} key={} kind={} reason={} origin={} universe={} domain={} hash={}\n",
            event.seq,
            event.event,
            event.key.as_deref().unwrap_or("-"),
            event.kind.as_deref().unwrap_or("-"),
            event.reason.as_deref().unwrap_or("-"),
            event
                .origin_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            event.universe_id,
            event.domain_id,
            event.payload_hash
        ));
    }

    TraceViewReport {
        event_count: selected.len(),
        required_digest: digest,
        rendered,
    }
}

pub fn render_profile_view(report: &ProfileReportV1, options: ProfileViewOptions) -> String {
    let top = if options.top == 0 { 10 } else { options.top };
    let selected = &report.key_costs[..report.key_costs.len().min(top)];

    if options.json {
        let mut out = String::new();
        out.push_str("{\"run_id\":\"");
        out.push_str(&json_escape(&report.run_id));
        out.push_str("\",\"event_count\":");
        out.push_str(&report.event_count.to_string());
        out.push_str(",\"observe_count\":");
        out.push_str(&report.observe_count.to_string());
        out.push_str(",\"eval_steps_est\":");
        out.push_str(&report.eval_steps_est.to_string());
        out.push_str(",\"alloc_units_est\":");
        out.push_str(&report.alloc_units_est.to_string());
        out.push_str(",\"total_steps\":");
        out.push_str(&report.total_steps.to_string());
        out.push_str(",\"required_digest\":\"");
        out.push_str(&report.required_digest);
        out.push_str("\",\"key_costs\":[");
        for (idx, key_cost) in selected.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                concat!(
                    "{{\"key\":\"{}\",",
                    "\"observe_count\":{},",
                    "\"ok_count\":{},",
                    "\"degraded_count\":{},",
                    "\"insufficient_count\":{},",
                    "\"deferred_count\":{}}}"
                ),
                json_escape(&key_cost.key),
                key_cost.observe_count,
                key_cost.ok_count,
                key_cost.degraded_count,
                key_cost.insufficient_count,
                key_cost.deferred_count
            ));
        }
        out.push_str("]}");
        return out;
    }

    let mut out = String::new();
    out.push_str("profile view\n");
    out.push_str("run_id=");
    out.push_str(&report.run_id);
    out.push('\n');
    out.push_str("event_count=");
    out.push_str(&report.event_count.to_string());
    out.push('\n');
    out.push_str("observe_count=");
    out.push_str(&report.observe_count.to_string());
    out.push('\n');
    out.push_str("eval_steps_est=");
    out.push_str(&report.eval_steps_est.to_string());
    out.push('\n');
    out.push_str("alloc_units_est=");
    out.push_str(&report.alloc_units_est.to_string());
    out.push('\n');
    out.push_str("total_steps=");
    out.push_str(&report.total_steps.to_string());
    out.push('\n');
    out.push_str("required_digest=");
    out.push_str(&report.required_digest);
    out.push('\n');
    for key_cost in selected {
        out.push_str(&format!(
            "- key={} observe={} ok={} degraded={} insufficient={} deferred={}\n",
            key_cost.key,
            key_cost.observe_count,
            key_cost.ok_count,
            key_cost.degraded_count,
            key_cost.insufficient_count,
            key_cost.deferred_count
        ));
    }
    out
}

fn append_trace_events(
    run_id: &str,
    universe_id: &str,
    domain_id: &str,
    seq: &mut u64,
    input: &[TraceEvent],
    out: &mut Vec<TraceEventV1>,
) {
    for event in input {
        let mapped = map_trace_event(*seq, run_id, universe_id, domain_id, event);
        out.push(mapped);
        *seq = seq.saturating_add(1);
    }
}

fn map_trace_event(
    seq: u64,
    run_id: &str,
    universe_id: &str,
    domain_id: &str,
    event: &TraceEvent,
) -> TraceEventV1 {
    let payload_hash = fnv1a64_hex(&canonical_trace_event_payload(event));
    match event {
        TraceEvent::ObserveStart { key, .. } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "observe_start".to_string(),
            key: Some(key.clone()),
            kind: None,
            reason: None,
            origin_id: None,
            allowed: None,
            value: None,
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ObserveEnd {
            key,
            kind,
            reason,
            origin_id,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "observe_end".to_string(),
            key: Some(key.clone()),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: Some(*origin_id),
            allowed: None,
            value: None,
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::MatchArmSelected { arm } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "match_arm_selected".to_string(),
            key: None,
            kind: Some(result_kind_label(*arm)),
            reason: None,
            origin_id: None,
            allowed: None,
            value: None,
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::CommitAttempt { origin_id, kind } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "commit_attempt".to_string(),
            key: None,
            kind: kind.as_ref().copied().map(result_kind_label),
            reason: None,
            origin_id: *origin_id,
            allowed: None,
            value: None,
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::CommitResult { allowed, reason } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "commit_result".to_string(),
            key: None,
            kind: None,
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: Some(*allowed),
            value: None,
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ConditionCheck { value } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "condition_check".to_string(),
            key: None,
            kind: None,
            reason: None,
            origin_id: None,
            allowed: None,
            value: Some(*value),
            steps: None,
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::LoopIter {
            loop_kind,
            iter_index,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "loop_iter".to_string(),
            key: Some(loop_kind.clone()),
            kind: None,
            reason: None,
            origin_id: None,
            allowed: None,
            value: None,
            steps: Some(*iter_index),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ProgramEnd { steps } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "program_end".to_string(),
            key: None,
            kind: None,
            reason: None,
            origin_id: None,
            allowed: None,
            value: None,
            steps: Some(*steps),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
    }
}

fn canonical_trace_event_payload(event: &TraceEvent) -> String {
    match event {
        TraceEvent::ObserveStart {
            key,
            tier,
            ctx,
            budget,
        } => format!("ObserveStart|{key}|{tier}|{ctx}|{budget}"),
        TraceEvent::ObserveEnd {
            key,
            kind,
            reason,
            origin_id,
        } => format!(
            "ObserveEnd|{key}|{}|{}|{origin_id}",
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::MatchArmSelected { arm } => {
            format!("MatchArmSelected|{}", result_kind_label(*arm))
        }
        TraceEvent::CommitAttempt { origin_id, kind } => format!(
            "CommitAttempt|{}|{}",
            origin_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            kind.as_ref()
                .copied()
                .map(result_kind_label)
                .unwrap_or_else(|| "-".to_string())
        ),
        TraceEvent::CommitResult { allowed, reason } => format!(
            "CommitResult|{}|{}",
            if *allowed { "allow" } else { "deny" },
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::ConditionCheck { value } => format!("ConditionCheck|{value}"),
        TraceEvent::LoopIter {
            loop_kind,
            iter_index,
        } => format!("LoopIter|{loop_kind}|{iter_index}"),
        TraceEvent::ProgramEnd { steps } => format!("ProgramEnd|{steps}"),
    }
}

fn result_kind_label(kind: ocl_runtime_core::ResultKind) -> String {
    match kind {
        ocl_runtime_core::ResultKind::Ok => "ok".to_string(),
        ocl_runtime_core::ResultKind::Degraded => "degraded".to_string(),
        ocl_runtime_core::ResultKind::Insufficient => "insufficient".to_string(),
        ocl_runtime_core::ResultKind::Deferred => "deferred".to_string(),
    }
}

fn encode_trace_line(event: &TraceEventV1) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.seq,
        event.run_id,
        event.event,
        encode_opt_str(event.key.as_deref()),
        encode_opt_str(event.kind.as_deref()),
        encode_opt_str(event.reason.as_deref()),
        encode_opt_u64(event.origin_id),
        encode_opt_bool(event.allowed),
        encode_opt_bool(event.value),
        encode_opt_u32(event.steps),
        event.universe_id,
        event.domain_id,
        event.payload_hash
    )
}

fn decode_trace_line(line: &str) -> Result<TraceEventV1, SdkError> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() != 11 && parts.len() != 13 {
        return Err(SdkError::MissingProject(
            "invalid trace row (expected 11 or 13 columns)".to_string(),
        ));
    }
    let (universe_id, domain_id, payload_idx) = if parts.len() == 13 {
        (parts[10].to_string(), parts[11].to_string(), 12usize)
    } else {
        (
            TRACE_UNIVERSE_SENTINEL.to_string(),
            TRACE_DOMAIN_SENTINEL.to_string(),
            10usize,
        )
    };
    Ok(TraceEventV1 {
        seq: parts[0]
            .parse::<u64>()
            .map_err(|_| SdkError::MissingProject("invalid trace seq".to_string()))?,
        run_id: parts[1].to_string(),
        event: parts[2].to_string(),
        key: decode_opt_str(parts[3]),
        kind: decode_opt_str(parts[4]),
        reason: decode_opt_str(parts[5]),
        origin_id: decode_opt_u64(parts[6])?,
        allowed: decode_opt_bool(parts[7])?,
        value: decode_opt_bool(parts[8])?,
        steps: decode_opt_u32(parts[9])?,
        universe_id,
        domain_id,
        payload_hash: parts[payload_idx].to_string(),
    })
}

fn encode_opt_str(v: Option<&str>) -> String {
    v.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
}

fn decode_opt_str(raw: &str) -> Option<String> {
    if raw == "-" {
        None
    } else {
        Some(raw.to_string())
    }
}

fn encode_opt_u64(v: Option<u64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string())
}

fn decode_opt_u64(raw: &str) -> Result<Option<u64>, SdkError> {
    if raw == "-" {
        return Ok(None);
    }
    raw.parse::<u64>()
        .map(Some)
        .map_err(|_| SdkError::MissingProject("invalid optional u64 in trace row".to_string()))
}

fn encode_opt_u32(v: Option<u32>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string())
}

fn decode_opt_u32(raw: &str) -> Result<Option<u32>, SdkError> {
    if raw == "-" {
        return Ok(None);
    }
    raw.parse::<u32>()
        .map(Some)
        .map_err(|_| SdkError::MissingProject("invalid optional u32 in trace row".to_string()))
}

fn encode_opt_bool(v: Option<bool>) -> String {
    match v {
        Some(true) => "1".to_string(),
        Some(false) => "0".to_string(),
        None => "-".to_string(),
    }
}

fn decode_opt_bool(raw: &str) -> Result<Option<bool>, SdkError> {
    match raw {
        "-" => Ok(None),
        "1" => Ok(Some(true)),
        "0" => Ok(Some(false)),
        _ => Err(SdkError::MissingProject(
            "invalid optional bool in trace row".to_string(),
        )),
    }
}

fn parse_u32(raw: &str, field: &str) -> Result<u32, SdkError> {
    raw.parse::<u32>()
        .map_err(|_| SdkError::MissingProject(format!("invalid u32 for `{field}`")))
}

fn trace_event_to_json(event: &TraceEventV1) -> String {
    format!(
        concat!(
            "{{\"seq\":{},\"run_id\":\"{}\",\"event\":\"{}\",",
            "\"key\":{},\"kind\":{},\"reason\":{},",
            "\"origin_id\":{},\"allowed\":{},\"value\":{},\"steps\":{},",
            "\"universe_id\":\"{}\",\"domain_id\":\"{}\",",
            "\"payload_hash\":\"{}\"}}"
        ),
        event.seq,
        json_escape(&event.run_id),
        json_escape(&event.event),
        json_opt_str(event.key.as_deref()),
        json_opt_str(event.kind.as_deref()),
        json_opt_str(event.reason.as_deref()),
        json_opt_u64(event.origin_id),
        json_opt_bool(event.allowed),
        json_opt_bool(event.value),
        json_opt_u32(event.steps),
        json_escape(&event.universe_id),
        json_escape(&event.domain_id),
        event.payload_hash
    )
}

fn json_opt_str(v: Option<&str>) -> String {
    match v {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    }
}

fn json_opt_u64(v: Option<u64>) -> String {
    v.map(|n| n.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn json_opt_u32(v: Option<u32>) -> String {
    v.map(|n| n.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn json_opt_bool(v: Option<bool>) -> String {
    match v {
        Some(true) => "true".to_string(),
        Some(false) => "false".to_string(),
        None => "null".to_string(),
    }
}

fn json_escape(input: &str) -> String {
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
