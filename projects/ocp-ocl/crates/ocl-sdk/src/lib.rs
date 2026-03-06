use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

mod attestation_v15;
mod perm_v15;
mod w17;
mod w18;
mod w19;
mod w20;

pub mod m4;
pub mod w1;
pub mod w2;
pub mod w3;
pub mod w5;
pub mod w6;
pub mod w9;

pub use attestation_v15::{
    build_attestation_v15, verify_build_attestation_v15, verify_build_repro_v15,
};
pub use m4::{
    compose_phenotype, load_component_catalog, load_phenotype_spec, verify_assembly,
    AssemblyProofV1, ComponentSpecV1, ComposeSummary, PhenotypeSpecV1, VerifySummary,
};
use ocl_runtime_core::{
    check_file_with_compat, normalize_text, parse_program, run_file_with_engine_config_and_compat,
    run_source_with_engine, CommitPolicyMode, CompatMode, DiagPhase, Diagnostic, ErrorCode,
    ExecCacheStats, ExecConfig, Expr, GuardMode, ObserveCacheStats, RunEngine, RuntimeCoreError,
    Span, Stmt, TraceEvent, TypecheckCompatConfig,
};
pub use perm_v15::{
    apply_permission_fix_plan_v17, approve_permission_diff_v15, write_permission_diff_report_v15,
    write_permission_doctor_report_v17, write_permission_fix_plan_v17,
    write_permission_snapshot_v15,
};
pub use w1::{
    enforce_universe_match_v1, init_cosmos_v1, resolve_hive_caps_v1, resolve_universe_v1,
    sync_cosmos_lock_v1, sync_policy_lock_v1, CosmosHiveV1, CosmosInitSummary,
    CosmosLockSyncSummary, CosmosLockV1, CosmosSpecV1, PolicyLockSyncSummary, PolicyLockV1,
    UniverseProfileV1, UniverseSelectionV1,
};
pub use w17::{
    artifact_hash_from_bytes_v17, build_contract_signature_sha256_v17, canonical_json_string_v17,
    canonical_json_value_v17, contract_sig_path_v17, enforce_build_env_allowlist_v17,
    evaluate_adapter_cve_policy_v17, evaluate_capability_edge_v17, evaluate_downgrade_attempt_v17,
    evaluate_pack_boundary_v17, evaluate_pack_trust_policy_v17, evaluate_trust_lifecycle_v17,
    inspect_contract_json_v17, inspect_pack_abi_spec_v17, semantic_hash_from_bytes_v17,
    sign_contract_json_v17, toolchain_digest_v17, verify_contract_json_signature_v17,
    verify_contract_signature_file_v17, verify_w17_contract_set_v17, AdapterCveDecisionV17,
    AdapterCveSeverityV17, CapabilityEdgeDecisionV17, ContractInspectSummaryV17,
    ContractSetSummaryV17, DowngradeDecisionV17, PackAbiBoundaryRequirementV17,
    PackAbiSpecSummaryV17, PackBoundaryDecisionV17, PackBoundaryV17, PackTrustDecisionV17,
    ToolchainDigestInputV17, TrustKeyRecordV17, TrustLifecycleSummaryV17,
    W17_ARTIFACT_HASH_VERSION, W17_HASHER_VERSION, W17_PACK_ABI_SCHEMA,
    W17_PACK_BOUNDARY_NATIVE_CAP_V1, W17_PACK_BOUNDARY_WASI_V1, W17_REQUIRED_CONTRACT_FILES,
    W17_SEMANTIC_HASH_VERSION, W17_SIGNATURE_SCHEMA, W17_SIGNATURE_SCHEMA_VERSION,
};
pub use w18::{
    build_contract_signature_sha256_v18, canonical_json_string_v18, canonical_json_value_v18,
    contract_sig_path_v18, inspect_contract_json_v18, sign_contract_json_v18,
    verify_contract_json_signature_v18, verify_sot_alias_contract_v18, verify_w18_contract_set_v18,
    ContractInspectSummaryV18, ContractSetSummaryV18, SotAliasItemSummaryV18, SotAliasSummaryV18,
    W18_HASHER_VERSION, W18_REQUIRED_CONTRACT_FILES, W18_SIGNATURE_SCHEMA,
    W18_SIGNATURE_SCHEMA_VERSION,
};
pub use w19::{
    bootstrap_retention_plan_v19, cli_bridge_contract_allows_v19, cli_bridge_output_policy_v19,
    code_action_apply_policy_v19, contract_sig_path_v19, dap_breakpoint_mapping_v19,
    dap_launch_summary_v19, dap_step_sequence_v19, dap_trace_events_from_source_v19,
    dap_trace_path_v19, dap_variables_for_event_v19, debug_contract_profile_v19,
    format_source_with_contract_v19, governed_code_action_ids_for_lane_v19,
    lsp_completion_items_from_source_v19, lsp_definition_locations_v19,
    lsp_diagnostics_from_source_v19, lsp_hover_for_symbol_v19,
    lsp_multiroot_definition_locations_v19, lsp_reference_locations_v19, lsp_rename_preview_v19,
    lsp_symbols_from_source_v19, multiroot_sorted_roots_v19, publish_channels_v19,
    required_bundled_binaries_v19, required_publish_files_v19,
    required_publish_metadata_fields_v19, runtime_resilience_profile_v19, sign_contract_json_v19,
    strict_lane_code_action_allowed_v19, validate_code_action_apply_request_v19,
    verify_contract_json_signature_v19, verify_w19_contract_set_v19, version_handshake_allowed_v19,
    version_rule_matches_v19, vsix_size_limit_for_channel_v19,
    workspace_runtime_features_allowed_v19, workspace_semantic_tokens_allowed_v19,
    ContractInspectSummaryV19, ContractSetSummaryV19, EditorCodeActionApplyPolicyV19,
    EditorCompletionItemV19, EditorDapBreakpointMapEntryV19, EditorDapBreakpointV19,
    EditorDapLaunchSummaryV19, EditorDapTraceEventV19, EditorDapVariableV19, EditorDiagnosticV19,
    EditorHoverV19, EditorLocationV19, EditorRenameEditV19, EditorRenamePreviewV19,
    EditorSymbolV19, W19_HASHER_VERSION, W19_REQUIRED_CONTRACT_FILES, W19_SIGNATURE_SCHEMA,
    W19_SIGNATURE_SCHEMA_VERSION,
};
pub use w2::{
    admit_bridge_emit_v1, poll_bridge_event_v1, resolve_bridge_runtime_plan_v1,
    resolve_domain_selection_v1, validate_locked_cosmos_bridge_config_v1, BridgeEnvelopeV1,
    BridgeRuntimePlanV1, BridgeRuntimeRuleV1, BridgeRuntimeStateV1, DomainProfileV1,
    DomainSelectionV1,
};
pub use w20::{
    contract_sig_path_v20, sign_contract_json_v20, verify_contract_json_signature_v20,
    verify_w20_contract_set_v20, ContractInspectSummaryV20, ContractSetSummaryV20,
    W20_HASHER_VERSION, W20_REQUIRED_CONTRACT_FILES, W20_SIGNATURE_SCHEMA,
    W20_SIGNATURE_SCHEMA_VERSION,
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
    pub global_deny: Vec<String>,
    pub package: Option<PermissionRules>,
    pub modules: HashMap<String, PermissionRules>,
    pub std_fs: Option<StdFsPermissionConfig>,
    pub std_net_http: Option<StdNetHttpPermissionConfig>,
    pub std_db: Option<StdDbPermissionConfig>,
    pub std_kv: Option<StdKvPermissionConfig>,
    pub std_queue: Option<StdQueuePermissionConfig>,
    pub std_time: Option<StdTimePermissionConfig>,
    pub std_proc: Option<StdProcPermissionConfig>,
    pub std_game: Option<StdGamePermissionConfig>,
    pub std_shadow: Option<StdShadowPermissionConfig>,
    pub std_ui: Option<StdUiPermissionConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdFsPermissionConfig {
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub remove: Vec<String>,
    pub rename: Vec<String>,
    pub list: Vec<String>,
    pub max_read_bytes: u64,
    pub max_write_bytes: u64,
    pub max_list_entries: u32,
}

impl Default for StdFsPermissionConfig {
    fn default() -> Self {
        Self {
            read: Vec::new(),
            write: Vec::new(),
            remove: Vec::new(),
            rename: Vec::new(),
            list: Vec::new(),
            max_read_bytes: 1_048_576,
            max_write_bytes: 1_048_576,
            max_list_entries: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdNetHttpPermissionConfig {
    pub enabled: bool,
    pub allow_hosts: Vec<String>,
    pub allow_methods: Vec<String>,
    pub max_body_bytes: u64,
    pub timeout_ms: u32,
}

impl Default for StdNetHttpPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_hosts: Vec::new(),
            allow_methods: Vec::new(),
            max_body_bytes: 1_048_576,
            timeout_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdDbPermissionConfig {
    pub enabled: bool,
    pub allow_dsn: Vec<String>,
    pub allow_modes: Vec<String>,
    pub max_rows: u32,
    pub max_bytes: u64,
    pub timeout_ms: u32,
}

impl Default for StdDbPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_dsn: Vec::new(),
            allow_modes: vec!["read_query".to_string(), "write_exec".to_string()],
            max_rows: 1_000,
            max_bytes: 1_048_576,
            timeout_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdKvPermissionConfig {
    pub enabled: bool,
    pub max_keys: u32,
    pub max_value_bytes: u64,
    pub key_prefix: Option<String>,
}

impl Default for StdKvPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_keys: 5_000,
            max_value_bytes: 65_536,
            key_prefix: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdQueuePermissionConfig {
    pub enabled: bool,
    pub allow_topics: Vec<String>,
    pub max_inflight: u32,
    pub max_payload_bytes: u64,
    pub ack_required: bool,
}

impl Default for StdQueuePermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_topics: Vec::new(),
            max_inflight: 256,
            max_payload_bytes: 65_536,
            ack_required: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdTimePermissionConfig {
    pub enabled: bool,
    pub tick_mode: String,
    pub dt_ms: u32,
}

impl Default for StdTimePermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tick_mode: "logical".to_string(),
            dt_ms: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdProcPermissionConfig {
    pub enabled: bool,
    pub allow_bins: Vec<String>,
    pub timeout_ms: u32,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
}

impl Default for StdProcPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_bins: Vec::new(),
            timeout_ms: 5_000,
            max_stdout_bytes: 1_048_576,
            max_stderr_bytes: 1_048_576,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdGamePermissionConfig {
    pub enabled: bool,
    pub fixed_dt_ms: u32,
    pub rng_streams: Vec<String>,
    pub rng_max_count: u32,
    pub state_delta_max_bytes: u64,
}

impl Default for StdGamePermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fixed_dt_ms: 16,
            rng_streams: vec!["main".to_string(), "loot".to_string()],
            rng_max_count: 1024,
            state_delta_max_bytes: 65_536,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdShadowPermissionConfig {
    pub enabled: bool,
    pub max_branches: u32,
    pub branch_step_cap: u32,
    pub branch_budget_cap: u32,
    pub max_diff_keys: u32,
    pub max_report_bytes: u64,
}

impl Default for StdShadowPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_branches: 8,
            branch_step_cap: 5_000,
            branch_budget_cap: 200_000,
            max_diff_keys: 2_000,
            max_report_bytes: 262_144,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdUiPermissionConfig {
    pub enabled: bool,
    pub max_draw_cmds: u32,
    pub max_input_events: u32,
    pub assets_read: Vec<String>,
    pub max_asset_bytes: u64,
}

impl Default for StdUiPermissionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_draw_cmds: 5_000,
            max_input_events: 500,
            assets_read: Vec::new(),
            max_asset_bytes: 2_097_152,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLanguageConfigV071 {
    pub lane: String,
    pub guard_mode: GuardMode,
    pub compat_ctx_string: CompatMode,
    pub compat_ctx_extra_fields: CompatMode,
}

impl Default for ProjectLanguageConfigV071 {
    fn default() -> Self {
        Self {
            lane: "locked_v071".to_string(),
            guard_mode: GuardMode::Return,
            compat_ctx_string: CompatMode::Warn,
            compat_ctx_extra_fields: CompatMode::Warn,
        }
    }
}

impl ProjectLanguageConfigV071 {
    fn typecheck_compat(&self) -> TypecheckCompatConfig {
        TypecheckCompatConfig {
            ctx_string: self.compat_ctx_string,
            ctx_extra_fields: self.compat_ctx_extra_fields,
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
    pub exec_cache: ExecCacheStats,
    pub observe_cache: ObserveCacheStats,
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
pub struct DepResolveSummaryV3 {
    pub deps_resolved: usize,
    pub lock_hash: String,
    pub lock_v3_path: PathBuf,
    pub ocl_lock_path: PathBuf,
    pub wrote_legacy_lock_v2: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockSignSummaryV15 {
    pub lock_path: PathBuf,
    pub sig_path: PathBuf,
    pub key_id: String,
    pub lock_ast_hash_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockVerifySummaryV15 {
    pub lock_path: PathBuf,
    pub sig_path: PathBuf,
    pub key_id: String,
    pub lock_ast_hash_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionSnapshotSummaryV15 {
    pub requested_path: PathBuf,
    pub granted_path: PathBuf,
    pub effective_path: PathBuf,
    pub snapshot_path: PathBuf,
    pub snapshot_hash_sha256: String,
    pub packages: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionDiffSummaryV15 {
    pub report_path: PathBuf,
    pub permission_diff_hash: String,
    pub has_changes: bool,
    pub introduces_new_permissions: bool,
    pub approval_checked: bool,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionApproveSummaryV15 {
    pub approval_path: PathBuf,
    pub diff_hash: String,
    pub approvals_total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionDoctorFindingV17 {
    pub code: String,
    pub reason: String,
    pub severity: String,
    pub location: String,
    pub value: String,
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionDoctorSummaryV17 {
    pub report_path: PathBuf,
    pub lane: String,
    pub findings_total: usize,
    pub blocking_total: usize,
    pub risk_score: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionFixPlanSummaryV17 {
    pub plan_path: PathBuf,
    pub plan_hash_sha256: String,
    pub lane: String,
    pub findings_total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionFixApplySummaryV17 {
    pub plan_path: PathBuf,
    pub patch_path: PathBuf,
    pub report_path: PathBuf,
    pub approval_path: PathBuf,
    pub plan_hash_sha256: String,
    pub findings_total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PermissionFixApplyOptionsV17 {
    pub plan_path: Option<PathBuf>,
    pub patch_path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub approval_path: Option<PathBuf>,
    pub approved_by: String,
    pub date: String,
    pub justification: String,
    pub ack_risk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildAttestationSummaryV15 {
    pub artifact_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub sig_path: PathBuf,
    pub key_id: String,
    pub manifest_hash_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildVerifyAttestationSummaryV15 {
    pub artifact_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub sig_path: PathBuf,
    pub key_id: String,
    pub manifest_hash_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReproVerifySummaryV15 {
    pub artifact_dir: PathBuf,
    pub baseline_hash_sha256: String,
    pub repro_hash_sha256: String,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDepV3 {
    pub alias: String,
    pub name: String,
    pub version: String,
    pub source: String,
    pub hash64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportProvenanceV10 {
    pub module_id: String,
    pub package_id: String,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOclPkgSummary {
    pub artifact_path: PathBuf,
    pub files_bundled: usize,
    pub payload_hash_blake3: String,
    pub content_hash_sha256: String,
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
    pub content_hash_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignOclPkgSummary {
    pub artifact_path: PathBuf,
    pub package_name: String,
    pub payload_hash_blake3: String,
    pub signer_pub_ed25519_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEventV1 {
    pub seq: u64,
    pub run_id: String,
    pub event: String,
    pub key: Option<String>,
    pub callsite_package_id: Option<String>,
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
    pub exec_cache: ExecCacheStats,
    pub observe_cache: ObserveCacheStats,
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

fn normalize_string_list(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
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
        let scalar_value = value_raw.trim().trim_matches('"');

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

        if current_section == "deny" {
            if key == "patterns" {
                out.global_deny = values;
                normalize_string_list(&mut out.global_deny);
            }
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
            continue;
        }

        if current_section == "permissions.std_fs" {
            let cfg = out
                .std_fs
                .get_or_insert_with(StdFsPermissionConfig::default);
            match key {
                "read" => cfg.read = values,
                "write" => cfg.write = values,
                "remove" => cfg.remove = values,
                "rename" => cfg.rename = values,
                "list" => cfg.list = values,
                "max_read_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_read_bytes = parsed.max(1);
                    }
                }
                "max_write_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_write_bytes = parsed.max(1);
                    }
                }
                "max_list_entries" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_list_entries = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.read);
            normalize_string_list(&mut cfg.write);
            normalize_string_list(&mut cfg.remove);
            normalize_string_list(&mut cfg.rename);
            normalize_string_list(&mut cfg.list);
            continue;
        }

        if current_section == "permissions.std_net_http" {
            let cfg = out
                .std_net_http
                .get_or_insert_with(StdNetHttpPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "allow_hosts" => {
                    cfg.allow_hosts = values.into_iter().map(|v| v.to_ascii_lowercase()).collect();
                }
                "allow_methods" => {
                    cfg.allow_methods =
                        values.into_iter().map(|v| v.to_ascii_uppercase()).collect();
                }
                "max_body_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_body_bytes = parsed.max(1);
                    }
                }
                "timeout_ms" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.timeout_ms = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.allow_hosts);
            normalize_string_list(&mut cfg.allow_methods);
            continue;
        }

        if current_section == "permissions.std_db" {
            let cfg = out
                .std_db
                .get_or_insert_with(StdDbPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "allow_dsn" => {
                    cfg.allow_dsn = values;
                }
                "allow_modes" => {
                    cfg.allow_modes = values.into_iter().map(|v| v.to_ascii_lowercase()).collect();
                }
                "max_rows" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_rows = parsed.max(1);
                    }
                }
                "max_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_bytes = parsed.max(1);
                    }
                }
                "timeout_ms" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.timeout_ms = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.allow_dsn);
            normalize_string_list(&mut cfg.allow_modes);
            continue;
        }

        if current_section == "permissions.std_kv" {
            let cfg = out
                .std_kv
                .get_or_insert_with(StdKvPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "max_keys" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_keys = parsed.max(1);
                    }
                }
                "max_value_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_value_bytes = parsed.max(1);
                    }
                }
                "key_prefix" => {
                    if scalar_value.is_empty() {
                        cfg.key_prefix = None;
                    } else {
                        cfg.key_prefix = Some(scalar_value.to_string());
                    }
                }
                _ => {}
            }
            continue;
        }

        if current_section == "permissions.std_queue" {
            let cfg = out
                .std_queue
                .get_or_insert_with(StdQueuePermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "allow_topics" => {
                    cfg.allow_topics = values;
                }
                "max_inflight" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_inflight = parsed.max(1);
                    }
                }
                "max_payload_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_payload_bytes = parsed.max(1);
                    }
                }
                "ack_required" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.ack_required = parsed;
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.allow_topics);
            continue;
        }

        if current_section == "permissions.std_time" {
            let cfg = out
                .std_time
                .get_or_insert_with(StdTimePermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "tick_mode" => {
                    if scalar_value == "logical" {
                        cfg.tick_mode = scalar_value.to_string();
                    }
                }
                "dt_ms" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.dt_ms = parsed.max(1);
                    }
                }
                _ => {}
            }
        }

        if current_section == "permissions.std_proc" {
            let cfg = out
                .std_proc
                .get_or_insert_with(StdProcPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "allow_bins" => {
                    cfg.allow_bins = values;
                }
                "timeout_ms" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.timeout_ms = parsed.max(1);
                    }
                }
                "max_stdout_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_stdout_bytes = parsed.max(1);
                    }
                }
                "max_stderr_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_stderr_bytes = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.allow_bins);
            continue;
        }

        if current_section == "permissions.std_game" {
            let cfg = out
                .std_game
                .get_or_insert_with(StdGamePermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "fixed_dt_ms" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.fixed_dt_ms = parsed.max(1);
                    }
                }
                "rng_streams" => {
                    cfg.rng_streams = values;
                }
                "rng_max_count" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.rng_max_count = parsed.max(1);
                    }
                }
                "state_delta_max_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.state_delta_max_bytes = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.rng_streams);
            continue;
        }

        if current_section == "permissions.std_shadow" {
            let cfg = out
                .std_shadow
                .get_or_insert_with(StdShadowPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "max_branches" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_branches = parsed.max(1);
                    }
                }
                "branch_step_cap" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.branch_step_cap = parsed.max(1);
                    }
                }
                "branch_budget_cap" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.branch_budget_cap = parsed.max(1);
                    }
                }
                "max_diff_keys" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_diff_keys = parsed.max(1);
                    }
                }
                "max_report_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_report_bytes = parsed.max(1);
                    }
                }
                _ => {}
            }
            continue;
        }

        if current_section == "permissions.std_ui" {
            let cfg = out
                .std_ui
                .get_or_insert_with(StdUiPermissionConfig::default);
            match key {
                "enabled" => {
                    if let Some(parsed) = parse_bool_literal(scalar_value) {
                        cfg.enabled = parsed;
                    }
                }
                "max_draw_cmds" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_draw_cmds = parsed.max(1);
                    }
                }
                "max_input_events" => {
                    if let Ok(parsed) = scalar_value.parse::<u32>() {
                        cfg.max_input_events = parsed.max(1);
                    }
                }
                "assets_read" => {
                    cfg.assets_read = values;
                }
                "max_asset_bytes" => {
                    if let Ok(parsed) = scalar_value.parse::<u64>() {
                        cfg.max_asset_bytes = parsed.max(1);
                    }
                }
                _ => {}
            }
            normalize_string_list(&mut cfg.assets_read);
        }
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedPermissionsV10 {
    permissions: ProjectPermissions,
    has_declared: bool,
}

fn canonicalize_glob_pattern(pattern: &str) -> String {
    let mut out = pattern.trim().replace('\\', "/");
    while out.starts_with("./") {
        out = out[2..].to_string();
    }
    out
}

fn glob_kind(pattern: &str) -> (&'static str, String) {
    let p = canonicalize_glob_pattern(pattern);
    if p == "**" {
        return ("recursive", String::new());
    }
    if p.ends_with("/**") && p.matches('*').count() == 2 {
        return ("recursive", p.trim_end_matches("/**").to_string());
    }
    if !p.contains('*') {
        return ("exact", p);
    }
    ("unsupported", p)
}

fn path_is_under(path: &str, root: &str) -> bool {
    if root.is_empty() {
        return true;
    }
    path == root || path.starts_with(&format!("{root}/"))
}

fn glob_pattern_subset(candidate: &str, container: &str) -> bool {
    let normalized_candidate = canonicalize_glob_pattern(candidate);
    let normalized_container = canonicalize_glob_pattern(container);
    if normalized_candidate == normalized_container {
        return true;
    }

    let (candidate_kind, candidate_value) = glob_kind(&normalized_candidate);
    let (container_kind, container_value) = glob_kind(&normalized_container);

    match (candidate_kind, container_kind) {
        ("exact", "exact") => candidate_value == container_value,
        ("exact", "recursive") => path_is_under(&candidate_value, &container_value),
        ("recursive", "recursive") => path_is_under(&candidate_value, &container_value),
        _ => false,
    }
}

fn key_pattern_subset(candidate: &str, container: &str) -> bool {
    if candidate == container {
        return true;
    }
    if container == "*" {
        return true;
    }
    if let Some(prefix) = container.strip_suffix('*') {
        return candidate.starts_with(prefix);
    }
    false
}

fn intersect_enum_list(granted: &[String], requested: &[String]) -> Vec<String> {
    let requested_set: HashSet<&String> = requested.iter().collect();
    let mut out: Vec<String> = granted
        .iter()
        .filter(|v| requested_set.contains(*v))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

fn intersect_key_patterns(granted: &[String], requested: &[String]) -> Vec<String> {
    let mut granted_norm: Vec<String> = granted
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    let mut requested_norm: Vec<String> = requested
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    granted_norm.sort();
    granted_norm.dedup();
    requested_norm.sort();
    requested_norm.dedup();

    let mut out = Vec::<String>::new();
    for g in &granted_norm {
        if requested_norm.iter().any(|r| key_pattern_subset(g, r)) {
            out.push(g.clone());
        }
    }
    for r in &requested_norm {
        if granted_norm.iter().any(|g| key_pattern_subset(r, g)) {
            out.push(r.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn intersect_glob_patterns(granted: &[String], requested: &[String]) -> Vec<String> {
    let mut granted_norm: Vec<String> = granted
        .iter()
        .map(|v| canonicalize_glob_pattern(v))
        .filter(|v| !v.is_empty())
        .collect();
    let mut requested_norm: Vec<String> = requested
        .iter()
        .map(|v| canonicalize_glob_pattern(v))
        .filter(|v| !v.is_empty())
        .collect();
    granted_norm.sort();
    granted_norm.dedup();
    requested_norm.sort();
    requested_norm.dedup();

    let mut out = Vec::<String>::new();
    for g in &granted_norm {
        if requested_norm.iter().any(|r| glob_pattern_subset(g, r)) {
            out.push(g.clone());
        }
    }
    for r in &requested_norm {
        if granted_norm.iter().any(|g| glob_pattern_subset(r, g)) {
            out.push(r.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn intersect_permission_rules(
    granted: Option<&PermissionRules>,
    requested: Option<&PermissionRules>,
) -> Option<PermissionRules> {
    let (Some(granted), Some(requested)) = (granted, requested) else {
        return None;
    };
    Some(PermissionRules {
        allow: intersect_key_patterns(&granted.allow, &requested.allow),
        deny: {
            let mut out = granted.deny.clone();
            out.extend(requested.deny.clone());
            out.sort();
            out.dedup();
            out
        },
    })
}

fn intersect_prefix(granted: Option<&str>, requested: Option<&str>) -> Option<String> {
    match (granted, requested) {
        (Some(g), Some(r)) if g == r => Some(g.to_string()),
        (Some(g), Some(r)) if g.starts_with(r) => Some(g.to_string()),
        (Some(g), Some(r)) if r.starts_with(g) => Some(r.to_string()),
        (Some(_), Some(_)) => Some("__deny_all__".to_string()),
        _ => None,
    }
}

fn compute_effective_permissions_for_dependency_v10(
    granted: &ProjectPermissions,
    requested: &RequestedPermissionsV10,
) -> ProjectPermissions {
    if !requested.has_declared {
        let mut passthrough = granted.clone();
        passthrough.global_deny.sort();
        passthrough.global_deny.dedup();
        return passthrough;
    }

    let mut out = ProjectPermissions {
        global_deny: granted.global_deny.clone(),
        package: intersect_permission_rules(
            granted.package.as_ref(),
            requested.permissions.package.as_ref(),
        ),
        modules: HashMap::new(),
        std_fs: None,
        std_net_http: None,
        std_db: None,
        std_kv: None,
        std_queue: None,
        std_time: None,
        std_proc: None,
        std_game: None,
        std_shadow: None,
        std_ui: None,
    };

    for (module_name, granted_rules) in &granted.modules {
        if let Some(requested_rules) = requested.permissions.modules.get(module_name) {
            out.modules.insert(
                module_name.clone(),
                PermissionRules {
                    allow: intersect_key_patterns(&granted_rules.allow, &requested_rules.allow),
                    deny: {
                        let mut deny = granted_rules.deny.clone();
                        deny.extend(requested_rules.deny.clone());
                        deny.sort();
                        deny.dedup();
                        deny
                    },
                },
            );
        }
    }

    if let (Some(g), Some(r)) = (
        granted.std_fs.as_ref(),
        requested.permissions.std_fs.as_ref(),
    ) {
        out.std_fs = Some(StdFsPermissionConfig {
            read: intersect_glob_patterns(&g.read, &r.read),
            write: intersect_glob_patterns(&g.write, &r.write),
            remove: intersect_glob_patterns(&g.remove, &r.remove),
            rename: intersect_glob_patterns(&g.rename, &r.rename),
            list: intersect_glob_patterns(&g.list, &r.list),
            max_read_bytes: g.max_read_bytes.min(r.max_read_bytes),
            max_write_bytes: g.max_write_bytes.min(r.max_write_bytes),
            max_list_entries: g.max_list_entries.min(r.max_list_entries),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_net_http.as_ref(),
        requested.permissions.std_net_http.as_ref(),
    ) {
        out.std_net_http = Some(StdNetHttpPermissionConfig {
            enabled: g.enabled && r.enabled,
            allow_hosts: intersect_enum_list(&g.allow_hosts, &r.allow_hosts),
            allow_methods: intersect_enum_list(&g.allow_methods, &r.allow_methods),
            max_body_bytes: g.max_body_bytes.min(r.max_body_bytes),
            timeout_ms: g.timeout_ms.min(r.timeout_ms),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_db.as_ref(),
        requested.permissions.std_db.as_ref(),
    ) {
        out.std_db = Some(StdDbPermissionConfig {
            enabled: g.enabled && r.enabled,
            allow_dsn: intersect_enum_list(&g.allow_dsn, &r.allow_dsn),
            allow_modes: intersect_enum_list(&g.allow_modes, &r.allow_modes),
            max_rows: g.max_rows.min(r.max_rows),
            max_bytes: g.max_bytes.min(r.max_bytes),
            timeout_ms: g.timeout_ms.min(r.timeout_ms),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_kv.as_ref(),
        requested.permissions.std_kv.as_ref(),
    ) {
        out.std_kv = Some(StdKvPermissionConfig {
            enabled: g.enabled && r.enabled,
            max_keys: g.max_keys.min(r.max_keys),
            max_value_bytes: g.max_value_bytes.min(r.max_value_bytes),
            key_prefix: intersect_prefix(g.key_prefix.as_deref(), r.key_prefix.as_deref()),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_queue.as_ref(),
        requested.permissions.std_queue.as_ref(),
    ) {
        out.std_queue = Some(StdQueuePermissionConfig {
            enabled: g.enabled && r.enabled,
            allow_topics: intersect_enum_list(&g.allow_topics, &r.allow_topics),
            max_inflight: g.max_inflight.min(r.max_inflight),
            max_payload_bytes: g.max_payload_bytes.min(r.max_payload_bytes),
            ack_required: g.ack_required && r.ack_required,
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_time.as_ref(),
        requested.permissions.std_time.as_ref(),
    ) {
        out.std_time = Some(StdTimePermissionConfig {
            enabled: g.enabled && r.enabled,
            tick_mode: "logical".to_string(),
            dt_ms: g.dt_ms.min(r.dt_ms),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_proc.as_ref(),
        requested.permissions.std_proc.as_ref(),
    ) {
        out.std_proc = Some(StdProcPermissionConfig {
            enabled: g.enabled && r.enabled,
            allow_bins: intersect_enum_list(&g.allow_bins, &r.allow_bins),
            timeout_ms: g.timeout_ms.min(r.timeout_ms),
            max_stdout_bytes: g.max_stdout_bytes.min(r.max_stdout_bytes),
            max_stderr_bytes: g.max_stderr_bytes.min(r.max_stderr_bytes),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_game.as_ref(),
        requested.permissions.std_game.as_ref(),
    ) {
        out.std_game = Some(StdGamePermissionConfig {
            enabled: g.enabled && r.enabled,
            fixed_dt_ms: g.fixed_dt_ms.min(r.fixed_dt_ms),
            rng_streams: intersect_enum_list(&g.rng_streams, &r.rng_streams),
            rng_max_count: g.rng_max_count.min(r.rng_max_count),
            state_delta_max_bytes: g.state_delta_max_bytes.min(r.state_delta_max_bytes),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_shadow.as_ref(),
        requested.permissions.std_shadow.as_ref(),
    ) {
        out.std_shadow = Some(StdShadowPermissionConfig {
            enabled: g.enabled && r.enabled,
            max_branches: g.max_branches.min(r.max_branches),
            branch_step_cap: g.branch_step_cap.min(r.branch_step_cap),
            branch_budget_cap: g.branch_budget_cap.min(r.branch_budget_cap),
            max_diff_keys: g.max_diff_keys.min(r.max_diff_keys),
            max_report_bytes: g.max_report_bytes.min(r.max_report_bytes),
        });
    }

    if let (Some(g), Some(r)) = (
        granted.std_ui.as_ref(),
        requested.permissions.std_ui.as_ref(),
    ) {
        out.std_ui = Some(StdUiPermissionConfig {
            enabled: g.enabled && r.enabled,
            max_draw_cmds: g.max_draw_cmds.min(r.max_draw_cmds),
            max_input_events: g.max_input_events.min(r.max_input_events),
            assets_read: intersect_glob_patterns(&g.assets_read, &r.assets_read),
            max_asset_bytes: g.max_asset_bytes.min(r.max_asset_bytes),
        });
    }

    out.global_deny.sort();
    out.global_deny.dedup();
    out
}

fn apply_permission_entry(out: &mut ProjectPermissions, section: &str, key: &str, value_raw: &str) {
    let values = parse_string_array_literal(value_raw);
    let scalar_value = value_raw.trim().trim_matches('"');

    if section == "deny" {
        if key == "patterns" {
            out.global_deny = values;
            normalize_string_list(&mut out.global_deny);
        }
        return;
    }

    if section == "package" {
        let rules = out.package.get_or_insert_with(PermissionRules::default);
        match key {
            "allow" => rules.allow = values,
            "deny" => rules.deny = values,
            _ => {}
        }
        normalize_permission_rules(rules);
        return;
    }

    if let Some(module_name) = section.strip_prefix("module.") {
        let module_key = module_name.trim().to_string();
        if module_key.is_empty() {
            return;
        }
        let rules = out.modules.entry(module_key).or_default();
        match key {
            "allow" => rules.allow = values,
            "deny" => rules.deny = values,
            _ => {}
        }
        normalize_permission_rules(rules);
        return;
    }

    if section == "std_fs" {
        let cfg = out
            .std_fs
            .get_or_insert_with(StdFsPermissionConfig::default);
        match key {
            "read" => cfg.read = values,
            "write" => cfg.write = values,
            "remove" => cfg.remove = values,
            "rename" => cfg.rename = values,
            "list" => cfg.list = values,
            "max_read_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_read_bytes = parsed.max(1);
                }
            }
            "max_write_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_write_bytes = parsed.max(1);
                }
            }
            "max_list_entries" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_list_entries = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.read);
        normalize_string_list(&mut cfg.write);
        normalize_string_list(&mut cfg.remove);
        normalize_string_list(&mut cfg.rename);
        normalize_string_list(&mut cfg.list);
        return;
    }

    if section == "std_net_http" {
        let cfg = out
            .std_net_http
            .get_or_insert_with(StdNetHttpPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "allow_hosts" => {
                cfg.allow_hosts = values.into_iter().map(|v| v.to_ascii_lowercase()).collect();
            }
            "allow_methods" => {
                cfg.allow_methods = values.into_iter().map(|v| v.to_ascii_uppercase()).collect();
            }
            "max_body_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_body_bytes = parsed.max(1);
                }
            }
            "timeout_ms" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.timeout_ms = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.allow_hosts);
        normalize_string_list(&mut cfg.allow_methods);
        return;
    }

    if section == "std_db" {
        let cfg = out
            .std_db
            .get_or_insert_with(StdDbPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "allow_dsn" => cfg.allow_dsn = values,
            "allow_modes" => {
                cfg.allow_modes = values.into_iter().map(|v| v.to_ascii_lowercase()).collect();
            }
            "max_rows" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_rows = parsed.max(1);
                }
            }
            "max_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_bytes = parsed.max(1);
                }
            }
            "timeout_ms" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.timeout_ms = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.allow_dsn);
        normalize_string_list(&mut cfg.allow_modes);
        return;
    }

    if section == "std_kv" {
        let cfg = out
            .std_kv
            .get_or_insert_with(StdKvPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "max_keys" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_keys = parsed.max(1);
                }
            }
            "max_value_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_value_bytes = parsed.max(1);
                }
            }
            "key_prefix" => {
                if scalar_value.is_empty() {
                    cfg.key_prefix = None;
                } else {
                    cfg.key_prefix = Some(scalar_value.to_string());
                }
            }
            _ => {}
        }
        return;
    }

    if section == "std_queue" {
        let cfg = out
            .std_queue
            .get_or_insert_with(StdQueuePermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "allow_topics" => cfg.allow_topics = values,
            "max_inflight" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_inflight = parsed.max(1);
                }
            }
            "max_payload_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_payload_bytes = parsed.max(1);
                }
            }
            "ack_required" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.ack_required = parsed;
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.allow_topics);
        return;
    }

    if section == "std_time" {
        let cfg = out
            .std_time
            .get_or_insert_with(StdTimePermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "tick_mode" => {
                if scalar_value == "logical" {
                    cfg.tick_mode = scalar_value.to_string();
                }
            }
            "dt_ms" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.dt_ms = parsed.max(1);
                }
            }
            _ => {}
        }
        return;
    }

    if section == "std_proc" {
        let cfg = out
            .std_proc
            .get_or_insert_with(StdProcPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "allow_bins" => cfg.allow_bins = values,
            "timeout_ms" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.timeout_ms = parsed.max(1);
                }
            }
            "max_stdout_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_stdout_bytes = parsed.max(1);
                }
            }
            "max_stderr_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_stderr_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.allow_bins);
        return;
    }

    if section == "std_game" {
        let cfg = out
            .std_game
            .get_or_insert_with(StdGamePermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "fixed_dt_ms" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.fixed_dt_ms = parsed.max(1);
                }
            }
            "rng_streams" => cfg.rng_streams = values,
            "rng_max_count" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.rng_max_count = parsed.max(1);
                }
            }
            "state_delta_max_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.state_delta_max_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.rng_streams);
        return;
    }

    if section == "std_shadow" {
        let cfg = out
            .std_shadow
            .get_or_insert_with(StdShadowPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "max_branches" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_branches = parsed.max(1);
                }
            }
            "branch_step_cap" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.branch_step_cap = parsed.max(1);
                }
            }
            "branch_budget_cap" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.branch_budget_cap = parsed.max(1);
                }
            }
            "max_diff_keys" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_diff_keys = parsed.max(1);
                }
            }
            "max_report_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_report_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
        return;
    }

    if section == "std_ui" {
        let cfg = out
            .std_ui
            .get_or_insert_with(StdUiPermissionConfig::default);
        match key {
            "enabled" => {
                if let Some(parsed) = parse_bool_literal(scalar_value) {
                    cfg.enabled = parsed;
                }
            }
            "max_draw_cmds" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_draw_cmds = parsed.max(1);
                }
            }
            "max_input_events" => {
                if let Ok(parsed) = scalar_value.parse::<u32>() {
                    cfg.max_input_events = parsed.max(1);
                }
            }
            "assets_read" => cfg.assets_read = values,
            "max_asset_bytes" => {
                if let Ok(parsed) = scalar_value.parse::<u64>() {
                    cfg.max_asset_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
        normalize_string_list(&mut cfg.assets_read);
    }
}

fn parse_requested_permissions_from_package_manifest_v10(raw: &str) -> RequestedPermissionsV10 {
    let mut out = ProjectPermissions::default();
    let mut current_section = String::new();
    let mut has_declared = false;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            if current_section == "requested_permissions"
                || current_section.starts_with("requested_permissions.")
            {
                has_declared = true;
            }
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value_raw = v.trim();

        if current_section == "requested_permissions" {
            if let Some((section, nested_key)) = key.split_once('.') {
                apply_permission_entry(&mut out, section.trim(), nested_key.trim(), value_raw);
            }
            continue;
        }

        if let Some(section) = current_section.strip_prefix("requested_permissions.") {
            apply_permission_entry(&mut out, section.trim(), key, value_raw);
        }
    }

    RequestedPermissionsV10 {
        permissions: out,
        has_declared,
    }
}

fn std_fs_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.fs.read_text" | "std.fs.stat" => Some("read"),
        "std.fs.list_dir" => Some("list"),
        "std.fs.write_text" | "std.fs.mkdir" => Some("write"),
        "std.fs.remove" => Some("remove"),
        "std.fs.rename" => Some("rename"),
        _ if key.starts_with("std.fs.") => Some("unknown"),
        _ => None,
    }
}

fn std_ui_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.ui.frame_info" | "std.ui.input" => Some("observe"),
        "std.ui.draw" => Some("draw"),
        "std.ui.present" => Some("present"),
        _ if key.starts_with("std.ui.") => Some("unknown"),
        _ => None,
    }
}

fn std_game_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.game.tick_info" => Some("tick_info"),
        "std.game.rng" => Some("rng"),
        "std.game.state_delta" => Some("state_delta"),
        _ if key.starts_with("std.game.") => Some("unknown"),
        _ => None,
    }
}

fn std_net_http_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.net.http.request" => Some("request"),
        _ if key.starts_with("std.net.http.") => Some("unknown"),
        _ => None,
    }
}

fn std_http_client_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.http.client.get" => Some("get"),
        "std.http.client.post" => Some("post"),
        _ if key.starts_with("std.http.client.") => Some("unknown"),
        _ => None,
    }
}

fn std_db_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.db.query_int" => Some("read_query"),
        "std.db.exec" => Some("write_exec"),
        _ if key.starts_with("std.db.") => Some("unknown"),
        _ => None,
    }
}

fn std_queue_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.queue.bus.publish" | "std.queue.publish" => Some("publish"),
        "std.queue.bus.consume" | "std.queue.consume" => Some("consume"),
        _ if key.starts_with("std.queue.bus.") || key.starts_with("std.queue.") => Some("unknown"),
        _ => None,
    }
}

fn std_proc_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.proc.exec" => Some("exec"),
        _ if key.starts_with("std.proc.") => Some("unknown"),
        _ => None,
    }
}

fn std_shadow_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "std.shadow.run" => Some("run"),
        "std.shadow.search" => Some("search"),
        "std.shadow.compare" => Some("compare"),
        _ if key.starts_with("std.shadow.") => Some("unknown"),
        _ => None,
    }
}

fn engine_ui_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "engine.ui.run" => Some("run"),
        _ if key.starts_with("engine.ui.") => Some("unknown"),
        _ => None,
    }
}

fn engine_game_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "engine.game.run" => Some("run"),
        _ if key.starts_with("engine.game.") => Some("unknown"),
        _ => None,
    }
}

fn engine_shadow_action_from_key(key: &str) -> Option<&'static str> {
    match key {
        "engine.shadow.preview" => Some("preview"),
        _ if key.starts_with("engine.shadow.") => Some("unknown"),
        _ => None,
    }
}

fn build_permission_hint_for_std_fs(action: &str) -> String {
    match action {
        "read" => {
            "Hint: add `[permissions.std_fs]` with `read = [\"./data/**\"]` in Ocl.toml."
                .to_string()
        }
        "list" => {
            "Hint: add `[permissions.std_fs]` with `list = [\"./data/**\"]` in Ocl.toml."
                .to_string()
        }
        "write" => {
            "Hint: add `[permissions.std_fs]` with `write = [\"./out/**\"]` in Ocl.toml."
                .to_string()
        }
        "remove" => {
            "Hint: add `[permissions.std_fs]` with `remove = [\"./out/**\"]` in Ocl.toml."
                .to_string()
        }
        "rename" => {
            "Hint: add `[permissions.std_fs]` with `rename = [\"./out/**\"]` in Ocl.toml."
                .to_string()
        }
        _ => "Hint: use supported std.fs keys: read_text/stat/list_dir/write_text/mkdir/remove/rename."
            .to_string(),
    }
}

fn verify_pack_permissions_for_key(
    permissions: &ProjectPermissions,
    module_path: Option<&str>,
    file_path: &Path,
    key: &str,
    _callsite_package_id: Option<&str>,
) -> Result<(), SdkError> {
    if let Some(action) = engine_ui_action_from_key(key) {
        let Some(cfg) = permissions.std_ui.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; missing section `[permissions.std_ui]` required by engine.ui. Hint: add `[permissions.std_ui]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; `[permissions.std_ui].enabled = false` while using engine.ui.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if action == "run" {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; unsupported engine.ui action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = engine_game_action_from_key(key) {
        let Some(cfg) = permissions.std_game.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; missing section `[permissions.std_game]` required by engine.game. Hint: add `[permissions.std_game]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; `[permissions.std_game].enabled = false` while using engine.game.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if action == "run" {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; unsupported engine.game action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = engine_shadow_action_from_key(key) {
        let Some(cfg) = permissions.std_shadow.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; missing section `[permissions.std_shadow]` required by engine.shadow. Hint: add `[permissions.std_shadow]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; `[permissions.std_shadow].enabled = false` while using engine.shadow.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if action == "preview" {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; unsupported engine.shadow action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_fs_action_from_key(key) {
        let Some(cfg) = permissions.std_fs.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-FS-PERMISSION-DENIED`; missing section `[permissions.std_fs]`. {}",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
                build_permission_hint_for_std_fs(action),
            )));
        };

        let allowed = match action {
            "read" => !cfg.read.is_empty(),
            "list" => !cfg.list.is_empty(),
            "write" => !cfg.write.is_empty(),
            "remove" => !cfg.remove.is_empty(),
            "rename" => !cfg.rename.is_empty(),
            _ => false,
        };

        if !allowed {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-FS-PERMISSION-DENIED`; action=`{}` has empty allowlist in `[permissions.std_fs]`. {}",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
                action,
                build_permission_hint_for_std_fs(action),
            )));
        }
        return Ok(());
    }

    if key.starts_with("std.kv.") {
        let Some(cfg) = permissions.std_kv.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-KV-PERMISSION-DENIED`; missing section `[permissions.std_kv]`. Hint: add `[permissions.std_kv]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-KV-PERMISSION-DENIED`; `[permissions.std_kv].enabled = false`. Hint: set `enabled = true` in `[permissions.std_kv]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        return Ok(());
    }

    if key.starts_with("std.time.") {
        let Some(cfg) = permissions.std_time.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-TIME-DISABLED`; missing section `[permissions.std_time]`. Hint: add `[permissions.std_time]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-TIME-DISABLED`; `[permissions.std_time].enabled = false`. Hint: set `enabled = true` in `[permissions.std_time]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        return Ok(());
    }

    if let Some(action) = std_net_http_action_from_key(key) {
        let Some(cfg) = permissions.std_net_http.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; missing section `[permissions.std_net_http]`. Hint: add `[permissions.std_net_http]` with `enabled = true`, non-empty `allow_hosts`, and non-empty `allow_methods` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; `[permissions.std_net_http].enabled = false`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_hosts.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; `[permissions.std_net_http].allow_hosts` is empty.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_methods.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-METHOD-DENIED`; `[permissions.std_net_http].allow_methods` is empty.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if action == "request" {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-METHOD-DENIED`; unsupported std.net.http action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_http_client_action_from_key(key) {
        let Some(cfg) = permissions.std_net_http.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; missing section `[permissions.std_net_http]` required by std.http.client.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; `[permissions.std_net_http].enabled = false` while using std.http.client.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_hosts.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-HOST-DENIED`; `[permissions.std_net_http].allow_hosts` is empty for std.http.client.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        let required_method = match action {
            "get" => Some("GET"),
            "post" => Some("POST"),
            _ => None,
        };
        if let Some(method) = required_method {
            if !cfg.allow_methods.iter().any(|m| m == method) {
                return Err(SdkError::PermissionDenied(format!(
                    "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-METHOD-DENIED`; missing method `{}` in `[permissions.std_net_http].allow_methods`.",
                    key,
                    file_path.display(),
                    module_path
                        .map(|m| format!(" (module `{m}`)"))
                        .unwrap_or_default(),
                    method,
                )));
            }
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-NET-METHOD-DENIED`; unsupported std.http.client action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_db_action_from_key(key) {
        let Some(cfg) = permissions.std_db.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-DB-DSN-DENIED`; missing section `[permissions.std_db]`. Hint: add `[permissions.std_db]` with `enabled = true`, `allow_dsn`, and `allow_modes`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-DB-DSN-DENIED`; `[permissions.std_db].enabled = false`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_dsn.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-DB-DSN-DENIED`; `[permissions.std_db].allow_dsn` is empty.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if !cfg.allow_modes.iter().any(|mode| mode == action) {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-DB-MODE-DENIED`; action `{}` not allowed by `[permissions.std_db].allow_modes`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
                action,
            )));
        }
        if matches!(action, "read_query" | "write_exec") {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-DB-MODE-DENIED`; unsupported std.db action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_queue_action_from_key(key) {
        let Some(cfg) = permissions.std_queue.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-QUEUE-TOPIC-DENIED`; missing section `[permissions.std_queue]`. Hint: add `[permissions.std_queue]` with `enabled = true` and `allow_topics`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-QUEUE-TOPIC-DENIED`; `[permissions.std_queue].enabled = false`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_topics.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-QUEUE-TOPIC-DENIED`; `[permissions.std_queue].allow_topics` is empty.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if matches!(action, "publish" | "consume") {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-QUEUE-TOPIC-DENIED`; unsupported std.queue action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_proc_action_from_key(key) {
        let Some(cfg) = permissions.std_proc.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-PROC-BIN-DENIED`; missing section `[permissions.std_proc]`. Hint: add `[permissions.std_proc]` with `enabled = true` and non-empty `allow_bins` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-PROC-BIN-DENIED`; `[permissions.std_proc].enabled = false`. Hint: set `enabled = true` in `[permissions.std_proc]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if cfg.allow_bins.is_empty() {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-PROC-BIN-DENIED`; `[permissions.std_proc].allow_bins` is empty.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if action == "exec" {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-PROC-BIN-DENIED`; unsupported std.proc action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_game_action_from_key(key) {
        let Some(cfg) = permissions.std_game.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; missing section `[permissions.std_game]`. Hint: add `[permissions.std_game]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; `[permissions.std_game].enabled = false`. Hint: set `enabled = true` in `[permissions.std_game]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }
        if matches!(action, "tick_info" | "rng" | "state_delta") {
            return Ok(());
        }
        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-GAME-DISABLED`; unsupported std.game action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_shadow_action_from_key(key) {
        let Some(cfg) = permissions.std_shadow.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; missing section `[permissions.std_shadow]`. Hint: add `[permissions.std_shadow]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; `[permissions.std_shadow].enabled = false`. Hint: set `enabled = true` in `[permissions.std_shadow]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }

        if matches!(action, "run" | "search" | "compare") {
            return Ok(());
        }

        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-SHADOW-DISABLED`; unsupported std.shadow action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    if let Some(action) = std_ui_action_from_key(key) {
        let Some(cfg) = permissions.std_ui.as_ref() else {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; missing section `[permissions.std_ui]`. Hint: add `[permissions.std_ui]` with `enabled = true` in Ocl.toml.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        };
        if !cfg.enabled {
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; `[permissions.std_ui].enabled = false`. Hint: set `enabled = true` in `[permissions.std_ui]`.",
                key,
                file_path.display(),
                module_path
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
            )));
        }

        if matches!(action, "draw" | "present" | "observe") {
            return Ok(());
        }

        return Err(SdkError::PermissionDenied(format!(
            "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}; reason=`RC-UI-DISABLED`; unsupported std.ui action `{}`.",
            key,
            file_path.display(),
            module_path
                .map(|m| format!(" (module `{m}`)"))
                .unwrap_or_default(),
            action,
        )));
    }

    Ok(())
}

pub fn parse_project_language_config_v071(manifest_text: &str) -> ProjectLanguageConfigV071 {
    fn parse_compat_mode(raw: &str) -> CompatMode {
        match raw {
            "allow" => CompatMode::Allow,
            "deny" => CompatMode::Deny,
            _ => CompatMode::Warn,
        }
    }

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
            continue;
        }

        if current_section == "compat" && key == "ctx_string" {
            out.compat_ctx_string = parse_compat_mode(value);
            continue;
        }

        if current_section == "compat" && key == "ctx_extra_fields" {
            out.compat_ctx_extra_fields = parse_compat_mode(value);
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
        return Some((
            PermissionDecision::Deny,
            "<implicit-deny-not-in-allow>".to_string(),
        ));
    }

    None
}

fn permission_decision_for_key(
    permissions: &ProjectPermissions,
    module_path: Option<&str>,
    key: &str,
) -> (PermissionDecision, Option<String>, Option<String>) {
    if let Some(matched) = permissions
        .global_deny
        .iter()
        .find(|pattern| permission_pattern_matches(pattern, key))
    {
        return (
            PermissionDecision::Deny,
            Some(matched.clone()),
            Some("deny".to_string()),
        );
    }

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
    callsite_package_id: Option<&str>,
) -> Result<(), SdkError> {
    if permissions.global_deny.is_empty()
        && permissions.package.is_none()
        && permissions.modules.is_empty()
        && permissions.std_fs.is_none()
        && permissions.std_net_http.is_none()
        && permissions.std_db.is_none()
        && permissions.std_kv.is_none()
        && permissions.std_queue.is_none()
        && permissions.std_time.is_none()
        && permissions.std_proc.is_none()
        && permissions.std_game.is_none()
        && permissions.std_shadow.is_none()
        && permissions.std_ui.is_none()
    {
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
            let hint = if section_name == "deny" {
                "Hint: remove key from `[deny].patterns` or move execution to package with different policy."
                    .to_string()
            } else if section_name.starts_with("permissions.module.") {
                format!(
                    "Hint: add key to `[{}].allow` or remove from `[{}].deny`.",
                    section_name, section_name
                )
            } else {
                "Hint: add key to `[permissions.package].allow` or remove from `[permissions.package].deny`.".to_string()
            };
            return Err(SdkError::PermissionDenied(format!(
                "V-PERMISSION-DENIED: key `{}` is denied for file `{}`{}{}; matched deny rule=`{}` in [{}]. {}",
                key,
                file_path.display(),
                module_path
                    .as_ref()
                    .map(|m| format!(" (module `{m}`)"))
                    .unwrap_or_default(),
                callsite_package_id
                    .map(|pkg| format!(" (callsite_package_id `{pkg}`)"))
                    .unwrap_or_default(),
                matched,
                section_name,
                hint
            )));
        }

        verify_pack_permissions_for_key(
            permissions,
            module_path.as_deref(),
            file_path,
            &key,
            callsite_package_id,
        )?;
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
        enable_exec_cache: true,
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
    alias: String,
    name: String,
    version_req: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockDep {
    name: String,
    version: String,
    hash64: String,
}

fn strip_toml_quotes(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn default_dep_source(dep_name: &str) -> String {
    if dep_name == "std" {
        "builtin".to_string()
    } else {
        "registry".to_string()
    }
}

fn normalize_dep_source(dep_name: &str, source_raw: Option<&str>) -> String {
    let source = source_raw
        .map(|v| strip_toml_quotes(v).to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default_dep_source(dep_name));
    match source.as_str() {
        "builtin" | "registry" | "path" | "git" => source,
        _ => "registry".to_string(),
    }
}

fn resolve_version_req_deterministic(version_req: &str) -> String {
    let trimmed = version_req.trim();
    if trimmed.is_empty() || trimmed == "*" {
        return "0.0.0".to_string();
    }
    if !trimmed.contains('*') {
        return trimmed.to_string();
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    let major = parts
        .first()
        .copied()
        .unwrap_or("0")
        .trim()
        .trim_matches('"');
    let minor = parts
        .get(1)
        .copied()
        .unwrap_or("0")
        .trim()
        .trim_matches('"');
    let patch = parts
        .get(2)
        .copied()
        .unwrap_or("0")
        .trim()
        .trim_matches('"');

    let major_norm = if major == "*" || major.is_empty() {
        "0"
    } else {
        major
    };
    let minor_norm = if minor == "*" || minor.is_empty() {
        "0"
    } else {
        minor
    };
    let patch_norm = if patch == "*" || patch.is_empty() {
        "0"
    } else {
        patch
    };
    format!("{major_norm}.{minor_norm}.{patch_norm}")
}

fn parse_dep_object_fields(raw: &str) -> Result<HashMap<String, String>, SdkError> {
    let value = raw.trim();
    if !(value.starts_with('{') && value.ends_with('}')) {
        return Err(SdkError::MissingProject(
            "invalid dependency object form (expected `{...}`)".to_string(),
        ));
    }
    let inner = &value[1..value.len() - 1];
    let mut out = HashMap::new();
    if inner.trim().is_empty() {
        return Ok(out);
    }
    for segment in inner.split(',') {
        let token = segment.trim();
        if token.is_empty() {
            continue;
        }
        let Some((k, v)) = token.split_once('=') else {
            return Err(SdkError::MissingProject(
                "invalid dependency object field (expected `key=value`)".to_string(),
            ));
        };
        out.insert(k.trim().to_string(), strip_toml_quotes(v));
    }
    Ok(out)
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
        let alias = k.trim();
        if alias.is_empty() {
            return Err(SdkError::MissingProject(
                "dependency name/version must not be empty".to_string(),
            ));
        }
        let value = v.trim();
        if value.is_empty() {
            return Err(SdkError::MissingProject(
                "dependency version must not be empty".to_string(),
            ));
        }

        let (name, version_req, source) = if value.starts_with('{') {
            let fields = parse_dep_object_fields(value)?;
            let resolved_name = fields
                .get("name")
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .unwrap_or(alias);
            let Some(version_raw) = fields.get("version").map(|v| v.trim()) else {
                return Err(SdkError::MissingProject(format!(
                    "dependency `{alias}` object form missing `version` field"
                )));
            };
            if version_raw.is_empty() {
                return Err(SdkError::MissingProject(format!(
                    "dependency `{alias}` has empty `version` field"
                )));
            }
            let source =
                normalize_dep_source(resolved_name, fields.get("source").map(String::as_str));
            (resolved_name.to_string(), version_raw.to_string(), source)
        } else {
            let version_raw = strip_toml_quotes(value);
            if version_raw.is_empty() {
                return Err(SdkError::MissingProject(
                    "dependency version must not be empty".to_string(),
                ));
            }
            let source = normalize_dep_source(alias, None);
            (alias.to_string(), version_raw, source)
        };

        deps.push(ManifestDep {
            alias: alias.to_string(),
            name,
            version_req,
            source,
        });
    }

    deps.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.name.cmp(&b.name))
            .then(a.version_req.cmp(&b.version_req))
            .then(a.source.cmp(&b.source))
    });
    Ok(deps)
}

fn to_lock_deps(deps: &[ManifestDep]) -> Vec<LockDep> {
    let mut out: Vec<LockDep> = deps
        .iter()
        .map(|dep| {
            let resolved_version = resolve_version_req_deterministic(&dep.version_req);
            let digest_input = format!("{}@{}", dep.name, resolved_version);
            LockDep {
                name: dep.name.clone(),
                version: resolved_version,
                hash64: fnv1a64_hex(&digest_input),
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    out
}

fn canonicalize_path(path: &Path, context: &str) -> Result<PathBuf, SdkError> {
    path.canonicalize()
        .map_err(|e| SdkError::MissingProject(format!("{context}: {} ({e})", path.display())))
}

fn canonicalize_path_under(path: &Path, root: &Path, context: &str) -> Result<PathBuf, SdkError> {
    let canonical = canonicalize_path(path, context)?;
    if !canonical.starts_with(root) {
        return Err(SdkError::Runtime(RuntimeCoreError::from(
            Diagnostic::new(
                ErrorCode::TImportNotFound,
                DiagPhase::Typecheck,
                Span::new(0, 0, 0, 0, 0),
                format!(
                    "import path '{}' resolved outside module root '{}'",
                    canonical.display(),
                    root.display()
                ),
            )
            .with_hint("use module path under package src root"),
        )));
    }
    Ok(canonical)
}

fn module_id_from_file_under_root(root: &Path, file: &Path) -> Result<String, SdkError> {
    let rel = file.strip_prefix(root).map_err(|_| {
        SdkError::MissingProject(format!(
            "module file '{}' is outside root '{}'",
            file.display(),
            root.display()
        ))
    })?;
    let mut parts = Vec::new();
    for comp in rel.components() {
        let raw = comp.as_os_str().to_string_lossy().replace('\\', "/");
        parts.push(raw);
    }
    if let Some(last) = parts.last_mut() {
        if let Some(stripped) = last.strip_suffix(".ocl") {
            *last = stripped.to_string();
        }
    }
    Ok(parts.join("."))
}

fn project_package_id_v10(layout: &ProjectLayout) -> Result<String, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    let (name, _) = parse_package_name_version(&manifest)?;
    Ok(format!("project:{name}"))
}

fn dep_package_id_v10(dep: &ManifestDep) -> String {
    let version = resolve_version_req_deterministic(&dep.version_req);
    let digest = sha256_hex(format!("{}@{}|{}", dep.name, version, dep.source).as_bytes());
    let short = &digest[..12];
    format!("dep:{}@{}#{}", dep.name, version, short)
}

fn parse_package_exports_modules_v10(raw: &str) -> Vec<String> {
    let mut in_exports = false;
    let mut modules = Vec::new();
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_exports = line == "[exports]";
            continue;
        }
        if !in_exports {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() == "modules" {
            modules = parse_string_array_literal(v)
                .into_iter()
                .map(|m| m.trim().to_string())
                .filter(|m| !m.is_empty())
                .collect();
        }
    }
    modules.sort();
    modules.dedup();
    modules
}

#[derive(Debug, Clone)]
struct DependencyPackageIndexV10 {
    package_id: String,
    src_root: PathBuf,
    exports: HashSet<String>,
}

fn load_dependency_package_index_v10(
    layout: &ProjectLayout,
    dep: &ManifestDep,
) -> Result<DependencyPackageIndexV10, SdkError> {
    let dep_root = layout.root.join("deps").join(&dep.alias);
    let package_manifest_path = dep_root.join("package.oclp");
    if !package_manifest_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing dependency package manifest for alias `{}`: {}",
            dep.alias,
            package_manifest_path.display()
        )));
    }
    let raw = fs::read_to_string(&package_manifest_path)?;
    let exports = parse_package_exports_modules_v10(&raw);
    let src_root = dep_root.join("src");
    if !src_root.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing dependency src root for alias `{}`: {}",
            dep.alias,
            src_root.display()
        )));
    }
    let src_root_canon = canonicalize_path(&src_root, "failed to canonicalize dependency src")?;
    let exports_set: HashSet<String> = exports.into_iter().collect();
    Ok(DependencyPackageIndexV10 {
        package_id: dep_package_id_v10(dep),
        src_root: src_root_canon,
        exports: exports_set,
    })
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

pub fn verify_dependency_exports_and_collect_provenance_v10(
    root: &Path,
) -> Result<Vec<ImportProvenanceV10>, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;

    let manifest_raw = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest_raw)?;
    let project_src_root =
        canonicalize_path(&layout.root.join("src"), "failed to canonicalize src")?;
    let project_package_id = project_package_id_v10(&layout)?;

    let mut dep_index = HashMap::<String, DependencyPackageIndexV10>::new();
    for dep in deps {
        if dep.source == "builtin" {
            continue;
        }
        dep_index.insert(
            dep.alias.clone(),
            load_dependency_package_index_v10(&layout, &dep)?,
        );
    }

    let mut out = Vec::<ImportProvenanceV10>::new();
    let mut loaded = HashSet::<String>::new();
    let mut visiting = HashSet::<String>::new();

    #[allow(clippy::too_many_arguments)]
    fn visit_module(
        package_id: &str,
        src_root: &Path,
        module_id: String,
        file_path: PathBuf,
        dep_index: &HashMap<String, DependencyPackageIndexV10>,
        loaded: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        out: &mut Vec<ImportProvenanceV10>,
    ) -> Result<(), SdkError> {
        let key = format!("{package_id}::{module_id}");
        if loaded.contains(&key) {
            return Ok(());
        }
        if visiting.contains(&key) {
            return Err(SdkError::Runtime(RuntimeCoreError::from(
                Diagnostic::new(
                    ErrorCode::TImportCycle,
                    DiagPhase::Typecheck,
                    Span::new(0, 0, 0, 0, 0),
                    format!("import cycle detected at `{module_id}`"),
                )
                .with_hint("break circular imports by extracting shared modules"),
            )));
        }
        visiting.insert(key.clone());

        let source = fs::read_to_string(&file_path)?;
        let program = parse_program(&source, 1).map_err(RuntimeCoreError::from)?;
        for stmt in &program.statements {
            let Stmt::ImportDecl { path, .. } = stmt else {
                continue;
            };
            if matches!(path.first().map(String::as_str), Some("std")) {
                continue;
            }
            let Some(first) = path.first() else {
                continue;
            };
            if let Some(dep_pkg) = dep_index.get(first) {
                if path.len() < 2 {
                    return Err(SdkError::Runtime(RuntimeCoreError::from(
                        Diagnostic::new(
                            ErrorCode::TImportNotFound,
                            DiagPhase::Typecheck,
                            Span::new(0, 0, 0, 0, 0),
                            format!("import `{first}` must include module path under dependency"),
                        )
                        .with_hint("use form `import <dep_alias>.<exported_module>;`"),
                    )));
                }
                let target_module = path[1..].join(".");
                if !dep_pkg.exports.contains(&target_module) {
                    return Err(SdkError::Runtime(RuntimeCoreError::from(
                        Diagnostic::new(
                            ErrorCode::TImportNotFound,
                            DiagPhase::Typecheck,
                            Span::new(0, 0, 0, 0, 0),
                            format!(
                                "module `{target_module}` is not exported by dependency alias `{first}`"
                            ),
                        )
                        .with_hint("add module to `[exports].modules` in dependency package.oclp"),
                    )));
                }
                let mut candidate = dep_pkg.src_root.clone();
                for seg in &path[1..] {
                    candidate.push(seg);
                }
                candidate.set_extension("ocl");
                let child_path = canonicalize_path_under(
                    &candidate,
                    &dep_pkg.src_root,
                    "failed to resolve dependency import module",
                )?;
                visit_module(
                    &dep_pkg.package_id,
                    &dep_pkg.src_root,
                    target_module,
                    child_path,
                    dep_index,
                    loaded,
                    visiting,
                    out,
                )?;
            } else {
                let target_module = path.join(".");
                let mut candidate = src_root.to_path_buf();
                for seg in path {
                    candidate.push(seg);
                }
                candidate.set_extension("ocl");
                let child_path = canonicalize_path_under(
                    &candidate,
                    src_root,
                    "failed to resolve local import module",
                )?;
                visit_module(
                    package_id,
                    src_root,
                    target_module,
                    child_path,
                    dep_index,
                    loaded,
                    visiting,
                    out,
                )?;
            }
        }

        out.push(ImportProvenanceV10 {
            module_id: module_id.clone(),
            package_id: package_id.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
        });
        visiting.remove(&key);
        loaded.insert(key);
        Ok(())
    }

    let entry_canon = canonicalize_path_under(
        &layout.src_main,
        &project_src_root,
        "failed to canonicalize entry module",
    )?;
    let entry_module = module_id_from_file_under_root(&project_src_root, &entry_canon)?;
    visit_module(
        &project_package_id,
        &project_src_root,
        entry_module,
        entry_canon,
        &dep_index,
        &mut loaded,
        &mut visiting,
        &mut out,
    )?;

    out.sort_by(|a, b| {
        a.module_id
            .cmp(&b.module_id)
            .then(a.package_id.cmp(&b.package_id))
            .then(a.file_path.cmp(&b.file_path))
    });
    Ok(out)
}

fn compute_effective_permissions_by_package_v10(
    layout: &ProjectLayout,
    project_permissions: &ProjectPermissions,
) -> Result<HashMap<String, ProjectPermissions>, SdkError> {
    let mut out = HashMap::<String, ProjectPermissions>::new();
    let project_package_id = project_package_id_v10(layout)?;
    out.insert(project_package_id, project_permissions.clone());

    let manifest_raw = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest_raw)?;
    for dep in deps {
        if dep.source == "builtin" {
            continue;
        }
        let dep_root = layout.root.join("deps").join(&dep.alias);
        let package_manifest_path = dep_root.join("package.oclp");
        if !package_manifest_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "missing dependency package manifest for permission compute: {}",
                package_manifest_path.display()
            )));
        }
        let raw = fs::read_to_string(&package_manifest_path)?;
        let requested = parse_requested_permissions_from_package_manifest_v10(&raw);
        let dep_package_id = dep_package_id_v10(&dep);
        let effective =
            compute_effective_permissions_for_dependency_v10(project_permissions, &requested);
        out.insert(dep_package_id, effective);
    }

    Ok(out)
}

pub fn compute_effective_permissions_v10(
    root: &Path,
) -> Result<HashMap<String, ProjectPermissions>, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let project_permissions = load_permissions_for_layout(&layout, false)?;
    compute_effective_permissions_by_package_v10(&layout, &project_permissions)
}

fn verify_permissions_with_provenance_v10(
    root: &Path,
    layout: &ProjectLayout,
    project_permissions: &ProjectPermissions,
) -> Result<(), SdkError> {
    let provenance = verify_dependency_exports_and_collect_provenance_v10(root)?;
    let effective_by_package =
        compute_effective_permissions_by_package_v10(layout, project_permissions)?;

    for (idx, entry) in provenance.iter().enumerate() {
        let Some(permissions) = effective_by_package.get(&entry.package_id) else {
            return Err(SdkError::PermissionDenied(format!(
                "X-DEP-PROVENANCE-MISSING: missing effective permissions for `{}` (module `{}`)",
                entry.package_id, entry.module_id
            )));
        };
        verify_permissions_for_file(
            Path::new(&entry.file_path),
            idx as u32 + 1,
            permissions,
            Some(entry.package_id.as_str()),
        )?;
    }

    Ok(())
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
    let lock_v3 = deps_lock_v3_path(layout);
    if lock_v3.exists() {
        let lock_v3_deps = verify_lock_v3_consistency(layout)?;
        let _ = evaluate_lock_v3_decisions_v10(layout, &lock_v3_deps, true)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockDepV3 {
    alias: String,
    name: String,
    version: String,
    source: String,
    hash64: String,
    content_hash_sha256: String,
    signature_b64: String,
    signer_pub_b64: String,
    trust_decision: String,
    requested_permissions_hash: String,
    dependencies: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct TrustStoreV10 {
    mode: Option<TrustPolicyModeV15>,
    require_signed_lock: Option<bool>,
    lane_locked_v071_requires_signed: Option<bool>,
    lane_locked_v06_requires_signed: Option<bool>,
    lane_quarantine_requires_signed: Option<bool>,
    registry_allow: Vec<String>,
    global_keys: HashSet<String>,
    by_source_keys: HashMap<String, HashSet<String>>,
    trusted_keys: Vec<TrustedKeyV15>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrustPolicyModeV15 {
    Strict,
    Warn,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TrustedKeyV15 {
    id: String,
    alg: String,
    public_key: String,
    scope: Vec<String>,
}

fn deps_lock_v3_path(layout: &ProjectLayout) -> PathBuf {
    layout.root.join("deps.lock.v3")
}

fn ocl_lock_export_path(layout: &ProjectLayout) -> PathBuf {
    layout.root.join("ocl.lock")
}

fn empty_requested_permissions_hash() -> String {
    sha256_hex(b"")
}

fn trust_store_path_v10(layout: &ProjectLayout) -> PathBuf {
    layout.root.join("trust.toml")
}

fn normalize_trust_source_v10(raw: &str) -> String {
    let lowered = raw.trim().to_ascii_lowercase().replace('\\', "/");
    let mut out = lowered.trim_end_matches('/').to_string();
    if out.is_empty() {
        out = lowered;
    }
    out
}

fn normalize_public_key_v15(raw: &str) -> String {
    let value = raw.trim().trim_matches('"').trim().to_string();
    if let Some(stripped) = value.strip_prefix("base64:") {
        return stripped.trim().to_string();
    }
    value
}

fn wildcard_no_slash_match_v15(pattern: &str, text: &str) -> bool {
    let p = pattern.as_bytes();
    let t = text.as_bytes();
    let mut pi = 0usize;
    let mut ti = 0usize;
    let mut star = None::<usize>;
    let mut star_ti = 0usize;

    while ti < t.len() {
        if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
            continue;
        }
        if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            star_ti = ti;
            continue;
        }
        if let Some(star_pi) = star {
            if t[star_ti] == b'/' {
                return false;
            }
            pi = star_pi + 1;
            star_ti += 1;
            ti = star_ti;
            continue;
        }
        return false;
    }

    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

fn glob_match_no_cross_slash_v15(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }
    let p_segments: Vec<&str> = pattern.split('/').collect();
    let v_segments: Vec<&str> = value.split('/').collect();
    if p_segments.len() != v_segments.len() {
        return false;
    }
    p_segments
        .iter()
        .zip(v_segments.iter())
        .all(|(p, v)| wildcard_no_slash_match_v15(p, v))
}

fn is_known_registry_source_v15(source: &str) -> bool {
    matches!(source, "registry" | "git" | "path")
}

fn source_matches_registry_pattern_v15(source: &str, pattern: &str) -> bool {
    if glob_match_no_cross_slash_v15(pattern, source) {
        return true;
    }
    match source {
        "registry" => pattern.starts_with("registry:"),
        "git" => pattern.starts_with("git:"),
        "path" => pattern.starts_with("path:"),
        _ => false,
    }
}

fn is_registry_allowed_v15(store: &TrustStoreV10, source: &str) -> bool {
    if source == "builtin" {
        return true;
    }

    if store.registry_allow.is_empty() {
        return is_known_registry_source_v15(source);
    }

    store
        .registry_allow
        .iter()
        .any(|pattern| source_matches_registry_pattern_v15(source, pattern))
}

fn scope_pattern_matches_v15(pattern: &str, package_name: &str) -> bool {
    let p = pattern.trim();
    if p.is_empty() {
        return false;
    }
    if let Some(prefix) = p.strip_suffix('*') {
        return package_name.starts_with(prefix);
    }
    package_name == p
}

fn trusted_key_scope_allows_v15(trusted_key: &TrustedKeyV15, dep: &LockDepV3) -> bool {
    if trusted_key.scope.is_empty() {
        return true;
    }
    trusted_key
        .scope
        .iter()
        .any(|scope| scope_pattern_matches_v15(scope, &dep.name))
        || trusted_key
            .scope
            .iter()
            .any(|scope| scope_pattern_matches_v15(scope, &dep.alias))
}

fn is_signer_trusted_for_dep_v15(store: &TrustStoreV10, dep: &LockDepV3, source: &str) -> bool {
    if dep.signer_pub_b64.trim().is_empty() {
        return false;
    }

    if is_signer_trusted_v10(store, source, &dep.signer_pub_b64) {
        return true;
    }

    store.trusted_keys.iter().any(|trusted| {
        trusted.public_key == dep.signer_pub_b64 && trusted_key_scope_allows_v15(trusted, dep)
    })
}

fn parse_trust_mode_v15(raw: &str) -> Option<TrustPolicyModeV15> {
    match raw.trim().trim_matches('"') {
        "strict" => Some(TrustPolicyModeV15::Strict),
        "warn" => Some(TrustPolicyModeV15::Warn),
        _ => None,
    }
}

fn lane_requires_signed_v15(store: &TrustStoreV10, lane: &str) -> bool {
    match lane {
        "locked_v071" => store.lane_locked_v071_requires_signed.unwrap_or(true),
        "locked_v06" => store.lane_locked_v06_requires_signed.unwrap_or(false),
        "quarantine" => store.lane_quarantine_requires_signed.unwrap_or(false),
        _ => false,
    }
}

fn lane_mode_v15(store: &TrustStoreV10, lane: &str) -> Result<TrustPolicyModeV15, SdkError> {
    if lane == "locked_v071" {
        if matches!(store.mode, Some(TrustPolicyModeV15::Warn)) {
            return Err(SdkError::SupplyInvalid(
                "X-TRUST-POLICY-MODE-INVALID: lane `locked_v071` requires strict mode; `mode=\"warn\"` is not allowed."
                    .to_string(),
            ));
        }
        if matches!(store.lane_locked_v071_requires_signed, Some(false)) {
            return Err(SdkError::SupplyInvalid(
                "X-TRUST-POLICY-INVALID: lane `locked_v071` cannot disable signed dependency enforcement."
                    .to_string(),
            ));
        }
        return Ok(TrustPolicyModeV15::Strict);
    }
    Ok(store.mode.unwrap_or(TrustPolicyModeV15::Warn))
}

fn insert_trusted_keys_v10(store: &mut TrustStoreV10, source: Option<&str>, mut keys: Vec<String>) {
    normalize_string_list(&mut keys);
    if keys.is_empty() {
        return;
    }
    if let Some(source_name) = source {
        let source_key = normalize_trust_source_v10(source_name);
        if source_key.is_empty() {
            return;
        }
        let bucket = store.by_source_keys.entry(source_key).or_default();
        for key in keys {
            bucket.insert(key);
        }
        return;
    }
    for key in keys {
        store.global_keys.insert(key);
    }
}

fn parse_trust_store_v10(path: &Path) -> Result<TrustStoreV10, SdkError> {
    if !path.exists() {
        return Ok(TrustStoreV10::default());
    }

    let raw = fs::read_to_string(path)?;
    let mut out = TrustStoreV10::default();
    let mut current_section = String::new();
    let mut current_trusted_key = None::<TrustedKeyV15>;

    let flush_current_trusted_key =
        |store: &mut TrustStoreV10, current: &mut Option<TrustedKeyV15>| {
            let Some(mut tk) = current.take() else {
                return;
            };
            tk.public_key = normalize_public_key_v15(&tk.public_key);
            normalize_string_list(&mut tk.scope);
            if tk.alg.is_empty() {
                tk.alg = "ed25519".to_string();
            }
            if !tk.public_key.is_empty() {
                store.trusted_keys.push(tk);
            }
        };

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("[[") && line.ends_with("]]") {
            flush_current_trusted_key(&mut out, &mut current_trusted_key);
            current_section = line[2..line.len() - 2].trim().to_string();
            if current_section == "trusted_key" {
                current_trusted_key = Some(TrustedKeyV15::default());
            }
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            flush_current_trusted_key(&mut out, &mut current_trusted_key);
            current_section = line[1..line.len() - 1].trim().to_string();
            continue;
        }

        let Some((key_raw, value_raw)) = line.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        let value = value_raw.trim();
        let keys = parse_string_array_literal(value);

        if current_section == "trusted_signers" {
            if keys.is_empty() {
                continue;
            }
            if key == "keys" || key == "all" {
                insert_trusted_keys_v10(&mut out, None, keys);
                continue;
            }
            insert_trusted_keys_v10(&mut out, Some(key), keys);
            continue;
        }

        if let Some(source) = current_section.strip_prefix("trusted_signers.") {
            if keys.is_empty() {
                continue;
            }
            if key == "keys" || key == "allow" || key == "trusted" {
                insert_trusted_keys_v10(&mut out, Some(source), keys);
            }
            continue;
        }

        if current_section == "policy" {
            match key {
                "mode" => out.mode = parse_trust_mode_v15(value),
                "require_signed_lock" => out.require_signed_lock = parse_bool_literal(value),
                "lane_locked_v071_requires_signed" => {
                    out.lane_locked_v071_requires_signed = parse_bool_literal(value)
                }
                "lane_locked_v06_requires_signed" => {
                    out.lane_locked_v06_requires_signed = parse_bool_literal(value)
                }
                "lane_quarantine_requires_signed" => {
                    out.lane_quarantine_requires_signed = parse_bool_literal(value)
                }
                _ => {}
            }
            continue;
        }

        if current_section == "registries" {
            if key == "allow" && !keys.is_empty() {
                out.registry_allow.extend(
                    keys.into_iter()
                        .map(|item| normalize_trust_source_v10(&item)),
                );
                normalize_string_list(&mut out.registry_allow);
            }
            continue;
        }

        if current_section == "trusted_key" {
            let Some(tk) = current_trusted_key.as_mut() else {
                continue;
            };
            match key {
                "id" => tk.id = strip_toml_quotes(value),
                "alg" => tk.alg = strip_toml_quotes(value).to_ascii_lowercase(),
                "public_key" => tk.public_key = strip_toml_quotes(value),
                "scope" => tk.scope = keys,
                _ => {}
            }
        }
    }

    flush_current_trusted_key(&mut out, &mut current_trusted_key);

    Ok(out)
}

fn is_signer_trusted_v10(store: &TrustStoreV10, source: &str, signer_pub_b64: &str) -> bool {
    if signer_pub_b64.trim().is_empty() {
        return false;
    }
    if store.global_keys.contains(signer_pub_b64) {
        return true;
    }
    let source_key = normalize_trust_source_v10(source);
    store
        .by_source_keys
        .get(&source_key)
        .map(|keys| keys.contains(signer_pub_b64))
        .unwrap_or(false)
}

fn lock_v3_sign_message(dep: &LockDepV3) -> String {
    format!("{}|{}|{}|{}", dep.alias, dep.name, dep.version, dep.source)
}

fn evaluate_lock_dep_trust_decision_v10(
    dep: &LockDepV3,
    trust_store: &TrustStoreV10,
    lane: &str,
) -> Result<String, SdkError> {
    let source = normalize_trust_source_v10(&dep.source);
    let has_sig = !dep.signature_b64.trim().is_empty();
    let has_pub = !dep.signer_pub_b64.trim().is_empty();

    if source == "builtin" && !has_sig && !has_pub {
        return Ok("builtin-unsigned".to_string());
    }

    let mode = lane_mode_v15(trust_store, lane)?;
    if mode == TrustPolicyModeV15::Strict && !is_registry_allowed_v15(trust_store, &source) {
        return Ok("registry-denied".to_string());
    }

    if has_sig ^ has_pub {
        return Err(SdkError::SupplyInvalid(format!(
            "dependency `{}` has partial signing fields in deps.lock.v3 (signature/signer must both exist or both be empty)",
            dep.alias
        )));
    }

    if !has_sig {
        return Ok("unsigned-unverified".to_string());
    }

    let sign_message = lock_v3_sign_message(dep);
    deterministic_verify(
        sign_message.as_bytes(),
        &dep.signature_b64,
        &dep.signer_pub_b64,
    )?;

    if is_signer_trusted_for_dep_v15(trust_store, dep, &source) {
        Ok("signed-trusted".to_string())
    } else {
        Ok("signed-untrusted".to_string())
    }
}

fn evaluate_lock_v3_decisions_v10(
    layout: &ProjectLayout,
    deps: &[LockDepV3],
    enforce_lane_policy: bool,
) -> Result<Vec<String>, SdkError> {
    let language_cfg = load_project_language_config_for_layout(layout)?;
    let lane = language_cfg.lane.trim().to_string();
    let trust_store = parse_trust_store_v10(&trust_store_path_v10(layout))?;
    let mode = lane_mode_v15(&trust_store, &lane)?;
    let requires_signed = lane_requires_signed_v15(&trust_store, &lane);

    let mut decisions = Vec::with_capacity(deps.len());
    for dep in deps {
        let decision = evaluate_lock_dep_trust_decision_v10(dep, &trust_store, &lane)?;
        if enforce_lane_policy && dep.source != "builtin" {
            if decision == "registry-denied" && mode == TrustPolicyModeV15::Strict {
                return Err(SdkError::SupplyInvalid(format!(
                    "X-TRUST-REGISTRY-DENIED: dependency `{}` has source `{}` which is not allowed by trust policy.",
                    dep.alias, dep.source
                )));
            }
            if decision == "unsigned-unverified" && requires_signed {
                return Err(SdkError::SupplyInvalid(format!(
                    "V-DEPS-SIGN-REQUIRED: non-builtin dependency `{}` is unsigned in lane `{}`. Hint: sign dependency and regenerate deps.lock.v3.",
                    dep.alias, lane
                )));
            }
            if decision == "signed-untrusted" && mode == TrustPolicyModeV15::Strict {
                return Err(SdkError::SupplyInvalid(format!(
                    "V-DEPS-TRUST-REQUIRED: signer for dependency `{}` is not trusted in lane `{}`. Hint: add signer key to trust.toml under [trusted_signers], [trusted_signers.{}], or [[trusted_key]].",
                    dep.alias, lane, dep.source
                )));
            }
        }
        decisions.push(decision);
    }

    Ok(decisions)
}

fn to_lock_deps_v3_from_manifest_deps(deps: &[ManifestDep]) -> Vec<LockDepV3> {
    let mut out = Vec::with_capacity(deps.len());
    for dep in deps {
        let version = resolve_version_req_deterministic(&dep.version_req);
        let digest_input = format!("{}@{}|{}", dep.name, version, dep.source);
        let hash64 = fnv1a64_hex(&digest_input);
        let content_hash_sha256 = sha256_hex(digest_input.as_bytes());
        let sign_msg = format!("{}|{}|{}|{}", dep.alias, dep.name, version, dep.source);
        let (signature_b64, signer_pub_b64, trust_decision) = if dep.source == "builtin" {
            (
                "".to_string(),
                "".to_string(),
                "builtin-unsigned".to_string(),
            )
        } else {
            let (sig, pub_key) = deterministic_sign("dep-lock-v3", sign_msg.as_bytes());
            (sig, pub_key, "signed-unverified".to_string())
        };
        out.push(LockDepV3 {
            alias: dep.alias.clone(),
            name: dep.name.clone(),
            version,
            source: dep.source.clone(),
            hash64,
            content_hash_sha256,
            signature_b64,
            signer_pub_b64,
            trust_decision,
            requested_permissions_hash: empty_requested_permissions_hash(),
            dependencies: Vec::new(),
        });
    }
    out.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.name.cmp(&b.name))
            .then(a.version.cmp(&b.version))
            .then(a.source.cmp(&b.source))
    });
    out
}

fn to_lock_deps_v3_from_v2(lock_v2: &[LockDepV2]) -> Vec<LockDepV3> {
    let mut out = Vec::with_capacity(lock_v2.len());
    for dep in lock_v2 {
        let source = if is_builtin_dep(&dep.name) {
            "builtin".to_string()
        } else {
            "registry".to_string()
        };
        let trust_decision = if source == "builtin" {
            "builtin-unsigned".to_string()
        } else if dep.signature_b64.is_empty() || dep.signer_pub_b64.is_empty() {
            "unsigned-unverified".to_string()
        } else {
            "signed-unverified".to_string()
        };
        let digest_input = format!("{}@{}|{}", dep.name, dep.version, source);
        out.push(LockDepV3 {
            alias: dep.name.clone(),
            name: dep.name.clone(),
            version: dep.version.clone(),
            source,
            hash64: dep.hash64.clone(),
            content_hash_sha256: sha256_hex(digest_input.as_bytes()),
            signature_b64: dep.signature_b64.clone(),
            signer_pub_b64: dep.signer_pub_b64.clone(),
            trust_decision,
            requested_permissions_hash: empty_requested_permissions_hash(),
            dependencies: Vec::new(),
        });
    }
    out.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.name.cmp(&b.name))
            .then(a.version.cmp(&b.version))
            .then(a.source.cmp(&b.source))
    });
    out
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

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
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

fn normalize_rel_path_for_hash(path: &Path) -> String {
    let mut rel = path.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = rel.strip_prefix("./") {
        rel = stripped.to_string();
    }
    rel
}

fn is_text_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|v| v.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "ocl" | "md" | "toml" | "json" | "yaml" | "yml" | "txt"
    )
}

fn normalize_lf_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0usize;
    while idx < bytes.len() {
        let b = bytes[idx];
        if b == b'\r' {
            out.push(b'\n');
            if idx + 1 < bytes.len() && bytes[idx + 1] == b'\n' {
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        out.push(b);
        idx += 1;
    }
    out
}

fn canonicalize_bytes_for_hash(path: &Path, bytes: &[u8]) -> Vec<u8> {
    let has_nul = bytes.contains(&0);
    if has_nul || !is_text_extension(path) {
        return bytes.to_vec();
    }
    normalize_lf_bytes(bytes)
}

fn compute_content_hash_sha256_from_raw_files(files: &[(String, Vec<u8>)]) -> String {
    let mut rows = Vec::with_capacity(files.len());
    for (rel, raw_bytes) in files {
        let rel_norm = normalize_rel_path_for_hash(Path::new(rel));
        let canonical = canonicalize_bytes_for_hash(Path::new(&rel_norm), raw_bytes);
        let file_hash = sha256_hex(&canonical);
        rows.push(format!(
            "file={rel_norm}|len={}|sha256={file_hash}",
            canonical.len()
        ));
    }
    rows.sort();
    let canonical_listing = rows.join("\n");
    sha256_hex(canonical_listing.as_bytes())
}

fn decode_packaged_files(files: &[(String, String)]) -> Result<Vec<(String, Vec<u8>)>, SdkError> {
    let mut decoded = Vec::with_capacity(files.len());
    for (rel, content_hex) in files {
        decoded.push((rel.clone(), hex_to_bytes(content_hex)?));
    }
    Ok(decoded)
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

fn encode_lock_v3(lock_deps: &[LockDepV3]) -> String {
    let mut out = String::from("version=3\nhasher_version=sha256-v1\n");
    for dep in lock_deps {
        let mut deps = dep.dependencies.clone();
        deps.sort();
        deps.dedup();
        let deps_joined = deps.join(",");

        out.push_str("dep=");
        out.push_str(&dep.alias);
        out.push('|');
        out.push_str(&dep.name);
        out.push('|');
        out.push_str(&dep.version);
        out.push('|');
        out.push_str(&dep.source);
        out.push('|');
        out.push_str(&dep.hash64);
        out.push('|');
        out.push_str(&dep.content_hash_sha256);
        out.push('|');
        out.push_str(&dep.signature_b64);
        out.push('|');
        out.push_str(&dep.signer_pub_b64);
        out.push('|');
        out.push_str(&dep.trust_decision);
        out.push('|');
        out.push_str(&dep.requested_permissions_hash);
        out.push('|');
        out.push_str(&deps_joined);
        out.push('\n');
    }
    out
}

fn parse_lock_v3(text: &str) -> Result<Vec<LockDepV3>, SdkError> {
    let mut has_version = false;
    let mut has_hasher = false;
    let mut deps = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=3" {
            has_version = true;
            continue;
        }
        if line == "hasher_version=sha256-v1" {
            has_hasher = true;
            continue;
        }
        let Some(payload) = line.strip_prefix("dep=") else {
            return Err(SdkError::SupplyInvalid(
                "invalid deps.lock.v3 line, expected `dep=...`".to_string(),
            ));
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() != 11 {
            return Err(SdkError::SupplyInvalid(
                "invalid dep entry in deps.lock.v3 (expected 11 fields)".to_string(),
            ));
        }
        let mut dependency_list = if parts[10].trim().is_empty() {
            Vec::new()
        } else {
            parts[10]
                .split(',')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect::<Vec<String>>()
        };
        dependency_list.sort();
        dependency_list.dedup();

        deps.push(LockDepV3 {
            alias: parts[0].to_string(),
            name: parts[1].to_string(),
            version: parts[2].to_string(),
            source: parts[3].to_string(),
            hash64: parts[4].to_string(),
            content_hash_sha256: parts[5].to_string(),
            signature_b64: parts[6].to_string(),
            signer_pub_b64: parts[7].to_string(),
            trust_decision: parts[8].to_string(),
            requested_permissions_hash: parts[9].to_string(),
            dependencies: dependency_list,
        });
    }
    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3 missing `version=3` header".to_string(),
        ));
    }
    if !has_hasher {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3 missing `hasher_version=sha256-v1` header".to_string(),
        ));
    }
    deps.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.name.cmp(&b.name))
            .then(a.version.cmp(&b.version))
            .then(a.source.cmp(&b.source))
    });
    Ok(deps)
}

fn lock_v3_sig_path(lock_path: &Path) -> PathBuf {
    lock_path.with_extension("v3.sig")
}

fn append_json_escaped_v15(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
}

fn append_json_string_field_v15(out: &mut String, key: &str, value: &str, trailing_comma: bool) {
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    append_json_escaped_v15(out, value);
    out.push('"');
    if trailing_comma {
        out.push(',');
    }
}

fn canonical_lock_v3_json_ast_v15(lock_deps: &[LockDepV3]) -> String {
    let mut out = String::from("{\"deps\":[");
    for (idx, dep) in lock_deps.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('{');
        append_json_string_field_v15(&mut out, "alias", &dep.alias, true);
        append_json_string_field_v15(
            &mut out,
            "content_hash_sha256",
            &dep.content_hash_sha256,
            true,
        );
        out.push_str("\"dependencies\":[");
        for (dep_idx, dependency) in dep.dependencies.iter().enumerate() {
            if dep_idx > 0 {
                out.push(',');
            }
            out.push('"');
            append_json_escaped_v15(&mut out, dependency);
            out.push('"');
        }
        out.push_str("],");
        append_json_string_field_v15(&mut out, "hash64", &dep.hash64, true);
        append_json_string_field_v15(&mut out, "name", &dep.name, true);
        append_json_string_field_v15(
            &mut out,
            "requested_permissions_hash",
            &dep.requested_permissions_hash,
            true,
        );
        append_json_string_field_v15(&mut out, "signature_b64", &dep.signature_b64, true);
        append_json_string_field_v15(&mut out, "signer_pub_b64", &dep.signer_pub_b64, true);
        append_json_string_field_v15(&mut out, "source", &dep.source, true);
        append_json_string_field_v15(&mut out, "trust_decision", &dep.trust_decision, true);
        append_json_string_field_v15(&mut out, "version", &dep.version, false);
        out.push('}');
    }
    out.push_str("],\"hasher_version\":\"sha256-v1\",\"version\":3}");
    out
}

fn lock_v3_ast_hash_v15(lock_deps: &[LockDepV3]) -> String {
    let canonical = canonical_lock_v3_json_ast_v15(lock_deps);
    sha256_hex(canonical.as_bytes())
}

fn parse_lock_v3_signature_file_v15(path: &Path) -> Result<LockVerifySummaryV15, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut key_id = None::<String>;
    let mut lock_ast_hash_sha256 = None::<String>;
    let mut signature_b64 = None::<String>;
    let mut signer_pub_b64 = None::<String>;
    let mut has_version = false;
    let mut has_hasher = false;
    let mut lock_rel = None::<String>;

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        if line == "hasher_version=sha256-v1" {
            has_hasher = true;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim();
        match key {
            "lock_path" => lock_rel = Some(value.to_string()),
            "key_id" => key_id = Some(value.to_string()),
            "lock_ast_hash_sha256" => lock_ast_hash_sha256 = Some(value.to_string()),
            "signature_b64" => signature_b64 = Some(value.to_string()),
            "signer_pub_b64" => signer_pub_b64 = Some(value.to_string()),
            _ => {}
        }
    }

    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `version=1` header".to_string(),
        ));
    }
    if !has_hasher {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `hasher_version=sha256-v1` header".to_string(),
        ));
    }

    let Some(lock_rel) = lock_rel else {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `lock_path`".to_string(),
        ));
    };
    let Some(key_id) = key_id else {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `key_id`".to_string(),
        ));
    };
    let Some(lock_ast_hash_sha256) = lock_ast_hash_sha256 else {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `lock_ast_hash_sha256`".to_string(),
        ));
    };
    let Some(signature_b64) = signature_b64 else {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `signature_b64`".to_string(),
        ));
    };
    let Some(signer_pub_b64) = signer_pub_b64 else {
        return Err(SdkError::SupplyInvalid(
            "deps.lock.v3.sig missing `signer_pub_b64`".to_string(),
        ));
    };

    let message = format!(
        "lock-sign-v15|key_id={}|lock_path={}|lock_ast_hash_sha256={}",
        key_id, lock_rel, lock_ast_hash_sha256
    );
    deterministic_verify(message.as_bytes(), &signature_b64, &signer_pub_b64)?;

    Ok(LockVerifySummaryV15 {
        lock_path: PathBuf::from(lock_rel),
        sig_path: path.to_path_buf(),
        key_id,
        lock_ast_hash_sha256,
    })
}

pub fn sign_deps_lock_v3_v15(root: &Path, key_id: &str) -> Result<LockSignSummaryV15, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let lock_path = deps_lock_v3_path(&layout);
    if !lock_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing deps.lock.v3: {}",
            lock_path.display()
        )));
    }
    let raw = fs::read_to_string(&lock_path)?;
    let deps = parse_lock_v3(&raw)?;
    let lock_ast_hash_sha256 = lock_v3_ast_hash_v15(&deps);
    let lock_rel = lock_path
        .strip_prefix(&layout.root)
        .unwrap_or(lock_path.as_path())
        .to_string_lossy()
        .replace('\\', "/");
    let message = format!(
        "lock-sign-v15|key_id={}|lock_path={}|lock_ast_hash_sha256={}",
        key_id.trim(),
        lock_rel,
        lock_ast_hash_sha256
    );
    let context = format!("lock-v3-v15|{}", key_id.trim());
    let (signature_b64, signer_pub_b64) = deterministic_sign(&context, message.as_bytes());
    let sig_path = lock_v3_sig_path(&lock_path);
    let sig_raw = format!(
        concat!(
            "version=1\n",
            "hasher_version=sha256-v1\n",
            "lock_path={}\n",
            "key_id={}\n",
            "lock_ast_hash_sha256={}\n",
            "signature_b64={}\n",
            "signer_pub_b64={}\n"
        ),
        lock_rel,
        key_id.trim(),
        lock_ast_hash_sha256,
        signature_b64,
        signer_pub_b64
    );
    fs::write(&sig_path, sig_raw)?;
    Ok(LockSignSummaryV15 {
        lock_path,
        sig_path,
        key_id: key_id.trim().to_string(),
        lock_ast_hash_sha256,
    })
}

pub fn verify_deps_lock_v3_signature_v15(root: &Path) -> Result<LockVerifySummaryV15, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let lock_path = deps_lock_v3_path(&layout);
    if !lock_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing deps.lock.v3: {}",
            lock_path.display()
        )));
    }
    let sig_path = lock_v3_sig_path(&lock_path);
    if !sig_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing deps.lock.v3.sig: {}",
            sig_path.display()
        )));
    }
    let mut summary = parse_lock_v3_signature_file_v15(&sig_path)?;
    let raw = fs::read_to_string(&lock_path)?;
    let deps = parse_lock_v3(&raw)?;
    let actual_hash = lock_v3_ast_hash_v15(&deps);
    if summary.lock_ast_hash_sha256 != actual_hash {
        return Err(SdkError::SupplyInvalid(format!(
            "X-LOCK-SIGNATURE-MISMATCH: lock AST hash mismatch (expected {}, got {}).",
            summary.lock_ast_hash_sha256, actual_hash
        )));
    }
    let expected_lock_rel = lock_path
        .strip_prefix(&layout.root)
        .unwrap_or(lock_path.as_path())
        .to_string_lossy()
        .replace('\\', "/");
    if summary.lock_path.to_string_lossy().replace('\\', "/") != expected_lock_rel {
        return Err(SdkError::SupplyInvalid(format!(
            "X-LOCK-SIGNATURE-MISMATCH: lock path mismatch in deps.lock.v3.sig (expected `{}`, got `{}`).",
            expected_lock_rel,
            summary.lock_path.display()
        )));
    }
    summary.lock_path = lock_path;
    Ok(summary)
}

fn encode_ocl_lock_export_v3(lock_hash: &str, lock_deps: &[LockDepV3]) -> String {
    let mut out = String::from("version=1\nsource=deps.lock.v3\n");
    out.push_str("lock_hash=");
    out.push_str(lock_hash);
    out.push('\n');
    for dep in lock_deps {
        out.push_str("dep=");
        out.push_str(&dep.alias);
        out.push('|');
        out.push_str(&dep.name);
        out.push('|');
        out.push_str(&dep.version);
        out.push('|');
        out.push_str(&dep.source);
        out.push('|');
        out.push_str(&dep.content_hash_sha256);
        out.push('|');
        out.push_str(&dep.trust_decision);
        out.push('\n');
    }
    out
}

fn expected_lock_v3_from_layout(layout: &ProjectLayout) -> Result<Vec<LockDepV3>, SdkError> {
    let manifest = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest)?;
    Ok(to_lock_deps_v3_from_manifest_deps(&deps))
}

fn verify_lock_v3_consistency(layout: &ProjectLayout) -> Result<Vec<LockDepV3>, SdkError> {
    let expected = expected_lock_v3_from_layout(layout)?;
    let path = deps_lock_v3_path(layout);
    if !path.exists() {
        return Err(SdkError::LockMismatch(format!(
            "missing deps.lock.v3: {} (run `ocl deps resolve <project_dir>`)",
            path.display()
        )));
    }
    let raw = fs::read_to_string(&path)?;
    let current = parse_lock_v3(&raw)?;
    if current != expected {
        return Err(SdkError::LockMismatch(
            "deps.lock.v3 mismatch with Ocl.toml dependencies (run `ocl deps resolve <project_dir>`)"
                .to_string(),
        ));
    }
    enforce_signed_lock_policy_v15(layout)?;
    perm_v15::enforce_permission_review_policy_v15(&layout.root)?;
    Ok(current)
}

fn enforce_signed_lock_policy_v15(layout: &ProjectLayout) -> Result<(), SdkError> {
    let language_cfg = load_project_language_config_for_layout(layout)?;
    if language_cfg.lane.trim() != "locked_v071" {
        return Ok(());
    }
    let trust_store = parse_trust_store_v10(&trust_store_path_v10(layout))?;
    if trust_store.require_signed_lock.unwrap_or(false) {
        let _ = verify_deps_lock_v3_signature_v15(&layout.root)?;
    }
    Ok(())
}

fn load_lock_v3_or_v2(layout: &ProjectLayout) -> Result<Vec<LockDepV3>, SdkError> {
    let lock_v3 = deps_lock_v3_path(layout);
    if lock_v3.exists() {
        let raw = fs::read_to_string(lock_v3)?;
        return parse_lock_v3(&raw);
    }
    let lock_v2_path = layout.root.join("deps.lock.v2");
    if lock_v2_path.exists() {
        let raw = fs::read_to_string(lock_v2_path)?;
        let lock_v2 = parse_lock_v2(&raw)?;
        return Ok(to_lock_deps_v3_from_v2(&lock_v2));
    }
    Err(SdkError::LockMismatch(
        "missing deps lock (expected deps.lock.v3 or deps.lock.v2)".to_string(),
    ))
}

fn write_lock_v3_and_export(layout: &ProjectLayout) -> Result<(usize, String), SdkError> {
    let deps_v3 = expected_lock_v3_from_layout(layout)?;
    let lock_text = encode_lock_v3(&deps_v3);
    let lock_hash = sha256_hex(lock_text.as_bytes());
    fs::write(deps_lock_v3_path(layout), lock_text)?;
    fs::write(
        ocl_lock_export_path(layout),
        encode_ocl_lock_export_v3(&lock_hash, &deps_v3),
    )?;
    Ok((deps_v3.len(), lock_hash))
}

pub fn read_resolved_deps_v3(root: &Path) -> Result<Vec<ResolvedDepV3>, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let deps = load_lock_v3_or_v2(&layout)?;
    Ok(deps
        .into_iter()
        .map(|dep| ResolvedDepV3 {
            alias: dep.alias,
            name: dep.name,
            version: dep.version,
            source: dep.source,
            hash64: dep.hash64,
        })
        .collect())
}

pub fn resolve_deps_v3(
    root: &Path,
    write_legacy_lock_v2: bool,
) -> Result<DepResolveSummaryV3, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let (deps_resolved, lock_hash) = write_lock_v3_and_export(&layout)?;
    if write_legacy_lock_v2 {
        let expected = read_expected_lock(&layout)?;
        let lock_v2 = to_lock_deps_v2(&expected);
        fs::write(layout.root.join("deps.lock.v2"), encode_lock_v2(&lock_v2))?;
    }
    Ok(DepResolveSummaryV3 {
        deps_resolved,
        lock_hash,
        lock_v3_path: deps_lock_v3_path(&layout),
        ocl_lock_path: ocl_lock_export_path(&layout),
        wrote_legacy_lock_v2: write_legacy_lock_v2,
    })
}

pub fn verify_deps_lock_v3(root: &Path) -> Result<(), SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let _ = verify_lock_v3_consistency(&layout)?;
    Ok(())
}

pub fn verify_deps_signing_and_trust_v10(
    root: &Path,
    enforce_lane_policy: bool,
) -> Result<(), SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let deps = load_lock_v3_or_v2(&layout)?;
    let _ = evaluate_lock_v3_decisions_v10(&layout, &deps, enforce_lane_policy)?;
    Ok(())
}

pub fn collect_deps_resolved_trace_events_v10(root: &Path) -> Result<Vec<TraceEventV1>, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let callsite_package_id = project_package_id_v10(&layout)?;
    let mut seq = 1u64;
    let mut events = Vec::new();
    append_deps_resolved_trace_events_v10(
        "deps_resolved_audit",
        TRACE_UNIVERSE_SENTINEL,
        TRACE_DOMAIN_SENTINEL,
        Some(&callsite_package_id),
        &layout,
        &mut seq,
        &mut events,
    )?;
    Ok(events)
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

fn collect_all_files(base: &Path, out: &mut Vec<PathBuf>) -> Result<(), SdkError> {
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn gather_package_files_v10(
    layout: &ProjectLayout,
    entry_rel: &str,
) -> Result<Vec<PathBuf>, SdkError> {
    let mut files = Vec::new();
    collect_all_files(&layout.root.join("src"), &mut files)?;
    collect_all_files(&layout.root.join("assets"), &mut files)?;
    collect_all_files(&layout.root.join("docs"), &mut files)?;
    collect_all_files(&layout.tests_dir, &mut files)?;

    let entry_path = layout.root.join(entry_rel);
    if !entry_path.exists() || !entry_path.is_file() {
        return Err(SdkError::MissingProject(format!(
            "package entry file not found: {}",
            entry_path.display()
        )));
    }
    if !files.iter().any(|p| p == &entry_path) {
        files.push(entry_path);
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn verify_permissions_for_file(
    file_path: &Path,
    file_id: u32,
    permissions: &ProjectPermissions,
    callsite_package_id: Option<&str>,
) -> Result<(), SdkError> {
    let source = fs::read_to_string(file_path)?;
    verify_permissions_for_source(
        &source,
        file_id,
        file_path,
        permissions,
        callsite_package_id,
    )
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
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let files = gather_project_ocl_files(&layout)?;
    if files.is_empty() {
        return Err(SdkError::MissingProject(
            "project has no .ocl sources under src/ or tests/".to_string(),
        ));
    }
    for (idx, path) in files.iter().enumerate() {
        verify_permissions_for_file(path, idx as u32 + 1, &permissions, None)?;
        check_file_with_compat(path, idx as u32 + 1, compat)?;
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
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let config = default_exec_config_for_layout(&layout)?;
    let out =
        run_file_with_engine_config_and_compat(&layout.src_main, 1, config, run_engine, compat)?;
    Ok(RunSummary {
        steps: out.steps,
        exec_cache: out.exec_cache,
        observe_cache: out.observe_cache,
    })
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
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let exec_config = default_exec_config_for_layout(&layout)?;

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
                    let out = run_file_with_engine_config_and_compat(
                        &layout.src_main,
                        1,
                        exec_config,
                        RunEngine::Interpreter,
                        compat,
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
                    let out = run_file_with_engine_config_and_compat(
                        &layout.src_main,
                        1,
                        exec_config,
                        RunEngine::Interpreter,
                        compat,
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
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let exec_config = default_exec_config_for_layout(&layout)?;
    let mut tests = Vec::new();
    collect_ocl_files(&layout.tests_dir, &mut tests)?;
    tests.sort();
    for (idx, test_file) in tests.iter().enumerate() {
        verify_permissions_for_file(test_file, idx as u32 + 100, &permissions, None)?;
        run_file_with_engine_config_and_compat(
            test_file,
            idx as u32 + 100,
            exec_config,
            RunEngine::Interpreter,
            compat,
        )?;
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
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
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
struct PackageManifestV10 {
    name: String,
    version: String,
    entry: String,
}

fn parse_package_manifest_v10(raw: &str) -> Result<PackageManifestV10, SdkError> {
    let mut in_package = false;
    let mut name = None::<String>;
    let mut version = None::<String>;
    let mut entry = None::<String>;

    for line_raw in raw.lines() {
        let line = line_raw.split('#').next().unwrap_or_default().trim();
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
            "entry" => entry = Some(value),
            _ => {}
        }
    }

    let Some(name) = name else {
        return Err(SdkError::MissingProject(
            "package.oclp missing [package].name".to_string(),
        ));
    };
    let Some(version) = version else {
        return Err(SdkError::MissingProject(
            "package.oclp missing [package].version".to_string(),
        ));
    };
    let entry = entry.unwrap_or_else(|| "src/main.ocl".to_string());
    if entry.trim().is_empty() {
        return Err(SdkError::MissingProject(
            "package.oclp has empty [package].entry".to_string(),
        ));
    }

    Ok(PackageManifestV10 {
        name,
        version,
        entry,
    })
}

fn load_package_manifest_v10(layout: &ProjectLayout) -> Result<PackageManifestV10, SdkError> {
    let package_oclp = layout.root.join("package.oclp");
    if package_oclp.exists() {
        let raw = fs::read_to_string(&package_oclp)?;
        return parse_package_manifest_v10(&raw);
    }

    let manifest = fs::read_to_string(&layout.manifest)?;
    let (name, version) = parse_package_name_version(&manifest)?;
    Ok(PackageManifestV10 {
        name,
        version,
        entry: "src/main.ocl".to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedOclPkg {
    package_name: String,
    version: String,
    entry: String,
    payload: String,
    payload_hash_blake3: String,
    content_hash_sha256: Option<String>,
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
    let Some(payload_hash_blake3) = hash_line.strip_prefix("payload_hash_blake3=") else {
        return Err(SdkError::SupplyInvalid(
            "invalid payload hash line".to_string(),
        ));
    };
    let Some(third_line) = lines.next() else {
        return Err(SdkError::SupplyInvalid(
            "missing signature line".to_string(),
        ));
    };
    let mut content_hash_sha256 = None::<String>;
    let (sig_line, pub_line) = if let Some(v) = third_line.strip_prefix("content_hash_sha256=") {
        content_hash_sha256 = Some(v.to_string());
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
        (sig_line, pub_line)
    } else {
        let Some(pub_line) = lines.next() else {
            return Err(SdkError::SupplyInvalid(
                "missing signer public key line".to_string(),
            ));
        };
        (third_line, pub_line)
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
        content_hash_sha256,
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
    if let Some(expected_content_hash) = parsed.content_hash_sha256.as_ref() {
        let decoded_files = decode_packaged_files(&parsed.files)?;
        let got_content_hash = compute_content_hash_sha256_from_raw_files(&decoded_files);
        if got_content_hash != *expected_content_hash {
            return Err(SdkError::SupplyInvalid(format!(
                "content hash mismatch: expected {} got {}",
                expected_content_hash, got_content_hash
            )));
        }
    }
    verify_lock_v2_signatures(&parsed.deps)?;
    Ok(SupplyVerifySummary {
        valid: true,
        package_name: parsed.package_name,
        payload_hash_blake3: parsed.payload_hash_blake3,
        content_hash_sha256: parsed.content_hash_sha256,
    })
}

pub fn sign_oclpkg(path: &Path) -> Result<SignOclPkgSummary, SdkError> {
    let parsed = parse_oclpkg(path)?;
    let payload_hash = hash256_hex(parsed.payload.as_bytes());
    if payload_hash != parsed.payload_hash_blake3 {
        return Err(SdkError::SupplyInvalid(format!(
            "payload hash mismatch: expected {} got {}",
            parsed.payload_hash_blake3, payload_hash
        )));
    }

    let (signature_ed25519_b64, signer_pub_ed25519_b64) =
        deterministic_sign("artifact-v1", payload_hash.as_bytes());

    let mut oclpkg = String::new();
    oclpkg.push_str("OCLPKGv1\n");
    oclpkg.push_str("payload_hash_blake3=");
    oclpkg.push_str(&parsed.payload_hash_blake3);
    oclpkg.push('\n');
    if let Some(content_hash_sha256) = parsed.content_hash_sha256.as_ref() {
        oclpkg.push_str("content_hash_sha256=");
        oclpkg.push_str(content_hash_sha256);
        oclpkg.push('\n');
    }
    oclpkg.push_str("signature_ed25519=");
    oclpkg.push_str(&signature_ed25519_b64);
    oclpkg.push('\n');
    oclpkg.push_str("signer_pub_ed25519=");
    oclpkg.push_str(&signer_pub_ed25519_b64);
    oclpkg.push('\n');
    oclpkg.push_str(&parsed.payload);
    fs::write(path, oclpkg)?;

    Ok(SignOclPkgSummary {
        artifact_path: path.to_path_buf(),
        package_name: parsed.package_name,
        payload_hash_blake3: parsed.payload_hash_blake3,
        signer_pub_ed25519_b64,
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

    let package_manifest = load_package_manifest_v10(&layout)?;
    let package_name = package_manifest.name.clone();
    let version = package_manifest.version.clone();
    let entry_rel = normalize_rel_path_for_hash(Path::new(&package_manifest.entry));
    let files = gather_package_files_v10(&layout, &entry_rel)?;

    let mut payload = String::new();
    payload.push_str("name=");
    payload.push_str(&package_name);
    payload.push('\n');
    payload.push_str("version=");
    payload.push_str(&version);
    payload.push('\n');
    payload.push_str("entry=");
    payload.push_str(&entry_rel);
    payload.push('\n');
    payload.push_str("files=");
    payload.push_str(&files.len().to_string());
    payload.push('\n');

    let mut raw_files_for_hash = Vec::with_capacity(files.len());
    for file in &files {
        let rel = file
            .strip_prefix(&layout.root)
            .map_err(|_| SdkError::MissingProject("invalid project path layout".to_string()))?;
        let rel_text = normalize_rel_path_for_hash(rel);
        let content = fs::read(file)?;
        let content_b64 = bytes_to_hex(&content);
        raw_files_for_hash.push((rel_text.clone(), content));
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

    let content_hash_sha256 = compute_content_hash_sha256_from_raw_files(&raw_files_for_hash);
    let payload_hash_blake3 = hash256_hex(payload.as_bytes());
    let (signature_ed25519_b64, signer_pub_ed25519_b64) =
        deterministic_sign("artifact-v1", payload_hash_blake3.as_bytes());

    let mut oclpkg = String::new();
    oclpkg.push_str("OCLPKGv1\n");
    oclpkg.push_str("payload_hash_blake3=");
    oclpkg.push_str(&payload_hash_blake3);
    oclpkg.push('\n');
    oclpkg.push_str("content_hash_sha256=");
    oclpkg.push_str(&content_hash_sha256);
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
        files_bundled: files.len().max(build.files_bundled),
        payload_hash_blake3,
        content_hash_sha256,
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
    Ok(RunSummary {
        steps: out.steps,
        exec_cache: out.exec_cache,
        observe_cache: out.observe_cache,
    })
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
    let callsite_package_id = project_package_id_v10(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let mut runtime_config = config;
    runtime_config.guard_mode = language_cfg.guard_mode;

    let out = run_file_with_engine_config_and_compat(
        &layout.src_main,
        1,
        runtime_config,
        run_engine,
        compat,
    )?;
    let run_id = build_run_id_deterministic("project", root, run_engine, None, None);
    let mut seq = 1u64;
    let mut events = Vec::new();
    if locked {
        append_deps_resolved_trace_events_v10(
            &run_id,
            TRACE_UNIVERSE_SENTINEL,
            TRACE_DOMAIN_SENTINEL,
            Some(&callsite_package_id),
            &layout,
            &mut seq,
            &mut events,
        )?;
    }
    append_trace_events(
        &run_id,
        TRACE_UNIVERSE_SENTINEL,
        TRACE_DOMAIN_SENTINEL,
        Some(&callsite_package_id),
        &mut seq,
        &out.trace.events,
        &mut events,
    );
    Ok(TraceRunSummary {
        run_id,
        total_steps: out.steps,
        events,
        exec_cache: out.exec_cache,
        observe_cache: out.observe_cache,
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
    let callsite_package_id = project_package_id_v10(&layout)?;
    if locked {
        verify_lock_consistency(&layout)?;
    }
    enforce_locked_plugin_contract(&layout, locked)?;
    enforce_locked_organ_contract(&layout, locked)?;
    let permissions = load_permissions_for_layout(&layout, locked)?;
    verify_permissions_with_provenance_v10(root, &layout, &permissions)?;
    let language_cfg = load_project_language_config_for_layout(&layout)?;
    let compat = language_cfg.typecheck_compat();
    let mut runtime_config = config;
    runtime_config.guard_mode = language_cfg.guard_mode;

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
    if locked {
        append_deps_resolved_trace_events_v10(
            &run_id,
            TRACE_UNIVERSE_SENTINEL,
            TRACE_DOMAIN_SENTINEL,
            Some(&callsite_package_id),
            &layout,
            &mut seq,
            &mut events,
        )?;
    }
    let mut total_steps = 0u32;
    let mut exec_cache_enabled = false;
    let mut exec_cache_entries = 0u32;
    let mut exec_cache_hits = 0u32;
    let mut exec_cache_misses = 0u32;
    let mut exec_cache_node_evals_charged = 0u32;
    let mut exec_cache_node_evals_executed = 0u32;
    let mut observe_cache_entries = 0u32;
    let mut observe_cache_hits = 0u32;
    let mut observe_cache_misses = 0u32;
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
                    let out = run_file_with_engine_config_and_compat(
                        &layout.src_main,
                        1,
                        runtime_config,
                        run_engine,
                        compat,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    exec_cache_enabled = exec_cache_enabled || out.exec_cache.enabled;
                    exec_cache_entries = exec_cache_entries.max(out.exec_cache.entries);
                    exec_cache_hits = exec_cache_hits.saturating_add(out.exec_cache.hits);
                    exec_cache_misses = exec_cache_misses.saturating_add(out.exec_cache.misses);
                    exec_cache_node_evals_charged = exec_cache_node_evals_charged
                        .saturating_add(out.exec_cache.node_evals_charged);
                    exec_cache_node_evals_executed = exec_cache_node_evals_executed
                        .saturating_add(out.exec_cache.node_evals_executed);
                    observe_cache_entries = observe_cache_entries.max(out.observe_cache.entries);
                    observe_cache_hits = observe_cache_hits.saturating_add(out.observe_cache.hits);
                    observe_cache_misses =
                        observe_cache_misses.saturating_add(out.observe_cache.misses);
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
                        Some(&callsite_package_id),
                        &mut seq,
                        &out.trace.events,
                        &mut events,
                    );
                    io_tape_replay_cursor = io_tape_replay_cursor.saturating_add(1);
                }
            } else {
                for _ in 0..events_per_tick {
                    let out = run_file_with_engine_config_and_compat(
                        &layout.src_main,
                        1,
                        runtime_config,
                        run_engine,
                        compat,
                    )?;
                    total_steps = total_steps.saturating_add(out.steps);
                    exec_cache_enabled = exec_cache_enabled || out.exec_cache.enabled;
                    exec_cache_entries = exec_cache_entries.max(out.exec_cache.entries);
                    exec_cache_hits = exec_cache_hits.saturating_add(out.exec_cache.hits);
                    exec_cache_misses = exec_cache_misses.saturating_add(out.exec_cache.misses);
                    exec_cache_node_evals_charged = exec_cache_node_evals_charged
                        .saturating_add(out.exec_cache.node_evals_charged);
                    exec_cache_node_evals_executed = exec_cache_node_evals_executed
                        .saturating_add(out.exec_cache.node_evals_executed);
                    observe_cache_entries = observe_cache_entries.max(out.observe_cache.entries);
                    observe_cache_hits = observe_cache_hits.saturating_add(out.observe_cache.hits);
                    observe_cache_misses =
                        observe_cache_misses.saturating_add(out.observe_cache.misses);
                    io_tape_record_entries.push(ReactorIoTapeEntry {
                        tick,
                        domain_id: domain_id.clone(),
                        payload_hash256: fnv1a64_hex(&out.signature),
                    });
                    append_trace_events(
                        &run_id,
                        &plan.universe_id,
                        domain_id,
                        Some(&callsite_package_id),
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
        exec_cache: ExecCacheStats {
            enabled: exec_cache_enabled,
            entries: exec_cache_entries,
            hits: exec_cache_hits,
            misses: exec_cache_misses,
            node_evals_charged: exec_cache_node_evals_charged,
            node_evals_executed: exec_cache_node_evals_executed,
        },
        observe_cache: ObserveCacheStats {
            entries: observe_cache_entries,
            hits: observe_cache_hits,
            misses: observe_cache_misses,
        },
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

fn append_deps_resolved_trace_events_v10(
    run_id: &str,
    universe_id: &str,
    domain_id: &str,
    callsite_package_id: Option<&str>,
    layout: &ProjectLayout,
    seq: &mut u64,
    out: &mut Vec<TraceEventV1>,
) -> Result<(), SdkError> {
    let lock_v3_path = deps_lock_v3_path(layout);
    if !lock_v3_path.exists() {
        return Ok(());
    }

    let raw_lock = fs::read_to_string(&lock_v3_path)?;
    let deps = parse_lock_v3(&raw_lock)?;
    let decisions = evaluate_lock_v3_decisions_v10(layout, &deps, false)?;
    let lock_hash = sha256_hex(raw_lock.as_bytes());
    let callsite = callsite_package_id.map(|v| v.to_string());

    for (idx, dep) in deps.iter().enumerate() {
        let decision = decisions
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let payload = format!(
            "DepsResolved|lock_hash={}|alias={}|name={}|version={}|source={}|content_hash_sha256={}|signature_b64={}|signer_pub_b64={}|trust_decision={}",
            lock_hash,
            dep.alias,
            dep.name,
            dep.version,
            dep.source,
            dep.content_hash_sha256,
            dep.signature_b64,
            dep.signer_pub_b64,
            decision
        );
        out.push(TraceEventV1 {
            seq: *seq,
            run_id: run_id.to_string(),
            event: "deps_resolved".to_string(),
            key: Some(format!("{}@{}|{}", dep.alias, dep.version, dep.source)),
            callsite_package_id: callsite.clone(),
            kind: None,
            reason: Some(decision.clone()),
            origin_id: None,
            allowed: Some(decision == "signed-trusted" || dep.source == "builtin"),
            value: None,
            steps: Some((idx as u32).saturating_add(1)),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash: fnv1a64_hex(&payload),
        });
        *seq = seq.saturating_add(1);
    }

    Ok(())
}

fn append_trace_events(
    run_id: &str,
    universe_id: &str,
    domain_id: &str,
    callsite_package_id: Option<&str>,
    seq: &mut u64,
    input: &[TraceEvent],
    out: &mut Vec<TraceEventV1>,
) {
    for event in input {
        let mapped = map_trace_event(
            *seq,
            run_id,
            universe_id,
            domain_id,
            callsite_package_id,
            event,
        );
        out.push(mapped);
        *seq = seq.saturating_add(1);
    }
}

fn map_trace_event(
    seq: u64,
    run_id: &str,
    universe_id: &str,
    domain_id: &str,
    callsite_package_id: Option<&str>,
    event: &TraceEvent,
) -> TraceEventV1 {
    let payload_hash = fnv1a64_hex(&canonical_trace_event_payload(event));
    let callsite = callsite_package_id.map(|v| v.to_string());
    match event {
        TraceEvent::ObserveStart { key, .. } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "observe_start".to_string(),
            key: Some(key.clone()),
            callsite_package_id: callsite.clone(),
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
            callsite_package_id: callsite.clone(),
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
        TraceEvent::WallclockObserve {
            key,
            call_id,
            unix_ms,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "wallclock_observe".to_string(),
            key: Some(key.clone()),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(unix_ms.is_some()),
            steps: Some((*call_id).min(u32::MAX as u64) as u32),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ProcObserve {
            key,
            call_id,
            exit_code: _,
            truncated,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "proc_observe".to_string(),
            key: Some(key.clone()),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*truncated),
            steps: Some((*call_id).min(u32::MAX as u64) as u32),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::NetHttpObserve {
            key,
            call_id,
            method,
            host,
            status: _,
            truncated,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "net_http_observe".to_string(),
            key: Some(format!("{key}:{method}:{host}")),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*truncated),
            steps: Some((*call_id).min(u32::MAX as u64) as u32),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::UiObserve {
            key,
            event_count,
            truncated,
            kind,
            reason,
            ..
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "ui_observe".to_string(),
            key: Some(key.clone()),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*truncated),
            steps: Some(*event_count),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::GameObserve {
            key,
            stream,
            value_count,
            tick,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "game_observe".to_string(),
            key: Some(format!("{key}:{stream}:{tick}")),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: None,
            steps: Some(*value_count),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ShadowRun {
            key,
            branch_count,
            truncated,
            detail,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "shadow_run".to_string(),
            key: Some(format!("{key}:{detail}")),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*truncated),
            steps: Some(*branch_count),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ShadowCompare {
            key,
            diff_count,
            report_bytes,
            truncated,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "shadow_compare".to_string(),
            key: Some(format!("{key}:{report_bytes}")),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*truncated),
            steps: Some(*diff_count),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::MatchArmSelected { arm } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "match_arm_selected".to_string(),
            key: None,
            callsite_package_id: callsite.clone(),
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
            callsite_package_id: callsite.clone(),
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
            callsite_package_id: callsite.clone(),
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
        TraceEvent::UiCommit {
            key,
            cmd_count,
            present,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "ui_commit".to_string(),
            key: Some(key.clone()),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: Some(*present),
            steps: Some(*cmd_count),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::GameCommit {
            key,
            delta_bytes,
            idempotency_hash,
            kind,
            reason,
        } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "game_commit".to_string(),
            key: Some(format!("{key}:{idempotency_hash}")),
            callsite_package_id: callsite.clone(),
            kind: Some(result_kind_label(*kind)),
            reason: reason.map(|r| r.as_str().to_string()),
            origin_id: None,
            allowed: None,
            value: None,
            steps: Some(*delta_bytes),
            universe_id: universe_id.to_string(),
            domain_id: domain_id.to_string(),
            payload_hash,
        },
        TraceEvent::ConditionCheck { value } => TraceEventV1 {
            seq,
            run_id: run_id.to_string(),
            event: "condition_check".to_string(),
            key: None,
            callsite_package_id: callsite.clone(),
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
            callsite_package_id: callsite.clone(),
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
            callsite_package_id: callsite.clone(),
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
        TraceEvent::WallclockObserve {
            key,
            call_id,
            unix_ms,
            kind,
            reason,
        } => format!(
            "WallclockObserve|{key}|{call_id}|{}|{}|{}",
            unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::ProcObserve {
            key,
            call_id,
            exit_code,
            truncated,
            kind,
            reason,
        } => format!(
            "ProcObserve|{key}|{call_id}|{}|{}|{}|{}",
            exit_code
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            if *truncated { "1" } else { "0" },
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::NetHttpObserve {
            key,
            call_id,
            method,
            host,
            status,
            truncated,
            kind,
            reason,
        } => format!(
            "NetHttpObserve|{key}|{call_id}|{method}|{host}|{}|{}|{}|{}",
            status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            if *truncated { "1" } else { "0" },
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::UiObserve {
            key,
            event_count,
            truncated,
            detail,
            kind,
            reason,
        } => format!(
            "UiObserve|{key}|{event_count}|{}|{}|{}|{detail}",
            if *truncated { "1" } else { "0" },
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::GameObserve {
            key,
            stream,
            value_count,
            tick,
            kind,
            reason,
        } => format!(
            "GameObserve|{key}|{stream}|{value_count}|{tick}|{}|{}",
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::ShadowRun {
            key,
            branch_count,
            truncated,
            detail,
            kind,
            reason,
        } => format!(
            "ShadowRun|{key}|{branch_count}|{}|{detail}|{}|{}",
            if *truncated { "1" } else { "0" },
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::ShadowCompare {
            key,
            diff_count,
            report_bytes,
            truncated,
            kind,
            reason,
        } => format!(
            "ShadowCompare|{key}|{diff_count}|{report_bytes}|{}|{}|{}",
            if *truncated { "1" } else { "0" },
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
        TraceEvent::UiCommit {
            key,
            cmd_count,
            present,
            kind,
            reason,
        } => format!(
            "UiCommit|{key}|{cmd_count}|{}|{}|{}",
            if *present { "1" } else { "0" },
            result_kind_label(*kind),
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::GameCommit {
            key,
            delta_bytes,
            idempotency_hash,
            kind,
            reason,
        } => format!(
            "GameCommit|{key}|{delta_bytes}|{idempotency_hash}|{}|{}",
            result_kind_label(*kind),
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
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.seq,
        event.run_id,
        event.event,
        encode_opt_str(event.key.as_deref()),
        encode_opt_str(event.callsite_package_id.as_deref()),
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
    if parts.len() != 11 && parts.len() != 12 && parts.len() != 13 && parts.len() != 14 {
        return Err(SdkError::MissingProject(
            "invalid trace row (expected 11/12/13/14 columns)".to_string(),
        ));
    }
    let has_callsite = parts.len() == 12 || parts.len() == 14;
    let has_universe_domain = parts.len() == 13 || parts.len() == 14;

    let key_idx = 3usize;
    let callsite_idx = if has_callsite { Some(4usize) } else { None };
    let kind_idx = if has_callsite { 5usize } else { 4usize };
    let reason_idx = if has_callsite { 6usize } else { 5usize };
    let origin_idx = if has_callsite { 7usize } else { 6usize };
    let allowed_idx = if has_callsite { 8usize } else { 7usize };
    let value_idx = if has_callsite { 9usize } else { 8usize };
    let steps_idx = if has_callsite { 10usize } else { 9usize };
    let (universe_id, domain_id, payload_idx) = if has_universe_domain {
        let universe_idx = if has_callsite { 11usize } else { 10usize };
        let domain_idx = if has_callsite { 12usize } else { 11usize };
        let payload_idx = if has_callsite { 13usize } else { 12usize };
        (
            parts[universe_idx].to_string(),
            parts[domain_idx].to_string(),
            payload_idx,
        )
    } else {
        (
            TRACE_UNIVERSE_SENTINEL.to_string(),
            TRACE_DOMAIN_SENTINEL.to_string(),
            if has_callsite { 11usize } else { 10usize },
        )
    };
    Ok(TraceEventV1 {
        seq: parts[0]
            .parse::<u64>()
            .map_err(|_| SdkError::MissingProject("invalid trace seq".to_string()))?,
        run_id: parts[1].to_string(),
        event: parts[2].to_string(),
        key: decode_opt_str(parts[key_idx]),
        callsite_package_id: callsite_idx.and_then(|idx| decode_opt_str(parts[idx])),
        kind: decode_opt_str(parts[kind_idx]),
        reason: decode_opt_str(parts[reason_idx]),
        origin_id: decode_opt_u64(parts[origin_idx])?,
        allowed: decode_opt_bool(parts[allowed_idx])?,
        value: decode_opt_bool(parts[value_idx])?,
        steps: decode_opt_u32(parts[steps_idx])?,
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
            "\"key\":{},\"callsite_package_id\":{},\"kind\":{},\"reason\":{},",
            "\"origin_id\":{},\"allowed\":{},\"value\":{},\"steps\":{},",
            "\"universe_id\":\"{}\",\"domain_id\":\"{}\",",
            "\"payload_hash\":\"{}\"}}"
        ),
        event.seq,
        json_escape(&event.run_id),
        json_escape(&event.event),
        json_opt_str(event.key.as_deref()),
        json_opt_str(event.callsite_package_id.as_deref()),
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
