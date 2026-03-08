use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::w18;
use crate::SdkError;
use ocp_runtime_core::{check_source, normalize_text, parse_program, Diagnostic, Expr, Stmt};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub const W19_HASHER_VERSION: &str = w18::W18_HASHER_VERSION;
pub const W19_SIGNATURE_SCHEMA_VERSION: &str = w18::W18_SIGNATURE_SCHEMA_VERSION;
pub const W19_SIGNATURE_SCHEMA: &str = w18::W18_SIGNATURE_SCHEMA;

pub const W19_REQUIRED_CONTRACT_FILES: [&str; 33] = [
    "editor/ocp_language_profile.v1.json",
    "editor/ocp_textmate_grammar.v1.json",
    "editor/ocp_semantic_tokens.v1.json",
    "editor/ocp_lsp_capabilities.v1.json",
    "editor/ocp_diagnostics_mapping.v1.json",
    "editor/ocp_formatting_contract.v1.json",
    "editor/ocp_code_actions_contract.v1.json",
    "editor/ocp_debug_contract.v1.json",
    "editor/ocp_settings_schema.v1.json",
    "editor/ocp_version_compat_matrix.v1.json",
    "editor/ocp_binary_bootstrap_policy.v1.json",
    "editor/editor_runtime_resilience.v1.json",
    "editor/editor_perf_budget.v1.json",
    "editor/editor_perf_protocol.v1.json",
    "editor/editor_publish_channels.v1.json",
    "editor/editor_packaging_limits.v1.json",
    "editor/editor_packaging_toolchain.v1.json",
    "editor/editor_publish_prerequisites.v1.json",
    "editor/editor_cli_bridge.v1.json",
    "editor/editor_code_action_apply_policy.v1.json",
    "editor/editor_bundled_binaries.v1.json",
    "editor/editor_signing_trust_root.v1.json",
    "editor/editor_bootstrap_retention.v1.json",
    "editor/editor_ci_harness.v1.json",
    "editor/editor_privacy_policy.v1.json",
    "editor/editor_workspace_trust_policy.v1.json",
    "editor/editor_multiroot_policy.v1.json",
    "editor/editor_assets_manifest.v1.json",
    "editor/editor_public_surface.v1.json",
    "editor/editor_versioning_policy.v1.json",
    "editor/required_contracts_editor.v1.json",
    "editor/editor_release_manifest.v1.json",
    "editor/golden_vectors.v1.json",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractInspectSummaryV19 {
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
pub struct ContractSetSummaryV19 {
    pub root: PathBuf,
    pub required_count: usize,
    pub verified_count: usize,
    pub items: Vec<ContractInspectSummaryV19>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDiagnosticV19 {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSymbolV19 {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorLocationV19 {
    pub file_id: u32,
    pub line: u32,
    pub column: u32,
    pub symbol: String,
    pub kind: String,
    pub root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCompletionItemV19 {
    pub label: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorHoverV19 {
    pub label: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorRenameEditV19 {
    pub file_id: u32,
    pub line: u32,
    pub column: u32,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorRenamePreviewV19 {
    pub edits: Vec<EditorRenameEditV19>,
    pub replaced_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCodeActionApplyPolicyV19 {
    pub patch_format: String,
    pub apply_mode: String,
    pub record_creation_point: String,
    pub allow_cli_apply_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDapVariableV19 {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDapTraceEventV19 {
    pub trace_event_id: u64,
    pub file_id: u32,
    pub line: u32,
    pub column: u32,
    pub kind: String,
    pub variables: Vec<EditorDapVariableV19>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDapBreakpointV19 {
    pub breakpoint_id: u64,
    pub file_id: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDapBreakpointMapEntryV19 {
    pub breakpoint_id: u64,
    pub trace_event_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDapLaunchSummaryV19 {
    pub debug_mode: String,
    pub trace_acquisition: String,
    pub trace_path: String,
    pub step_semantics: String,
    pub events_count: usize,
}

pub fn contract_sig_path_v19(contract_path: &Path) -> PathBuf {
    w18::contract_sig_path_v18(contract_path)
}

pub fn sign_contract_json_v19(
    contract_path: &Path,
    pubkey_id: &str,
    trust_epoch: u64,
) -> Result<PathBuf, SdkError> {
    w18::sign_contract_json_v18(contract_path, pubkey_id, trust_epoch)
}

pub fn verify_contract_json_signature_v19(
    repo_root: &Path,
    contract_path: &Path,
) -> Result<ContractInspectSummaryV19, SdkError> {
    let item = w18::verify_contract_json_signature_v18(repo_root, contract_path)?;
    Ok(ContractInspectSummaryV19 {
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

pub fn verify_w19_contract_set_v19(repo_root: &Path) -> Result<ContractSetSummaryV19, SdkError> {
    let contracts_root = repo_root.join("contracts");
    let mut items = Vec::<ContractInspectSummaryV19>::new();
    for rel in W19_REQUIRED_CONTRACT_FILES {
        let path = contracts_root.join(rel);
        if !path.exists() {
            return Err(SdkError::SupplyInvalid(format!(
                "X-V19-CONTRACT-MISSING: {}",
                path.display()
            )));
        }
        let summary = verify_contract_json_signature_v19(repo_root, &path)?;
        items.push(summary);
    }
    items.sort_by(|lhs, rhs| lhs.contract_id.as_bytes().cmp(rhs.contract_id.as_bytes()));
    Ok(ContractSetSummaryV19 {
        root: contracts_root,
        required_count: W19_REQUIRED_CONTRACT_FILES.len(),
        verified_count: items.len(),
        items,
    })
}

pub fn lsp_diagnostics_from_source_v19(source: &str, file_id: u32) -> Vec<EditorDiagnosticV19> {
    match check_source(source, file_id) {
        Ok(_) => Vec::new(),
        Err(diag) => vec![EditorDiagnosticV19 {
            code: diag.code.as_str().to_string(),
            severity: diagnostic_severity_from_code(diag.code.as_str()).to_string(),
            message: diag.message,
            line: diag.span.line,
            column: diag.span.column,
        }],
    }
}

pub fn lsp_symbols_from_source_v19(
    source: &str,
    file_id: u32,
) -> Result<Vec<EditorSymbolV19>, Diagnostic> {
    let program = parse_program(source, file_id)?;
    let mut out = Vec::<EditorSymbolV19>::new();
    collect_stmt_symbols(&program.statements, &mut out);
    out.sort_by(|lhs, rhs| {
        lhs.line
            .cmp(&rhs.line)
            .then(lhs.column.cmp(&rhs.column))
            .then(lhs.name.as_bytes().cmp(rhs.name.as_bytes()))
    });
    Ok(out)
}

pub fn version_handshake_allowed_v19(
    extension_version: &str,
    lsp_version: &str,
    dap_version: &str,
    matrix_contract: &JsonValue,
) -> bool {
    let Some(rows) = matrix_contract.get("rows").and_then(JsonValue::as_array) else {
        return false;
    };
    rows.iter().any(|row| {
        row.get("extension")
            .and_then(JsonValue::as_str)
            .map(|v| version_matches(v, extension_version))
            .unwrap_or(false)
            && row
                .get("lsp")
                .and_then(JsonValue::as_str)
                .map(|v| version_matches(v, lsp_version))
                .unwrap_or(false)
            && row
                .get("dap")
                .and_then(JsonValue::as_str)
                .map(|v| version_matches(v, dap_version))
                .unwrap_or(false)
    })
}

pub fn workspace_runtime_features_allowed_v19(workspace_trusted: bool, policy: &JsonValue) -> bool {
    let key = if workspace_trusted {
        "trusted_mode"
    } else {
        "untrusted_mode"
    };
    policy
        .get(key)
        .and_then(|v| v.get("spawn_lsp"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
}

pub fn workspace_semantic_tokens_allowed_v19(workspace_trusted: bool, policy: &JsonValue) -> bool {
    let key = if workspace_trusted {
        "trusted_mode"
    } else {
        "untrusted_mode"
    };
    policy
        .get(key)
        .and_then(|v| v.get("semantic_tokens_provider"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
}

pub fn runtime_resilience_profile_v19(policy: &JsonValue) -> (u64, u64, bool) {
    let retries = policy
        .get("crash_loop")
        .and_then(|v| v.get("max_retries"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let cooldown = policy
        .get("crash_loop")
        .and_then(|v| v.get("cooldown_ms"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let degraded = policy
        .get("degraded_mode")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    (retries, cooldown, degraded)
}

pub fn lsp_completion_items_from_source_v19(
    source: &str,
    file_id: u32,
) -> Result<Vec<EditorCompletionItemV19>, Diagnostic> {
    let program = parse_program(source, file_id)?;
    let mut defs = Vec::<EditorLocationV19>::new();
    collect_stmt_definitions(&program.statements, file_id, None, &mut defs);

    let mut items = BTreeMap::<String, String>::new();
    for keyword in [
        "fn", "let", "return", "match", "observe", "commit", "if", "else", "repeat", "for", "in",
    ] {
        items.insert(keyword.to_string(), "keyword".to_string());
    }
    for def in defs {
        items.insert(def.symbol, def.kind);
    }

    Ok(items
        .into_iter()
        .map(|(label, kind)| EditorCompletionItemV19 { label, kind })
        .collect::<Vec<EditorCompletionItemV19>>())
}

pub fn lsp_hover_for_symbol_v19(
    source: &str,
    file_id: u32,
    symbol: &str,
) -> Result<Option<EditorHoverV19>, Diagnostic> {
    let defs = lsp_definition_locations_v19(source, file_id, symbol)?;
    let Some(def) = defs.first() else {
        return Ok(None);
    };
    let detail = match def.kind.as_str() {
        "function" => format!("fn {}(...)", def.symbol),
        "struct" => format!("struct {}", def.symbol),
        "enum" => format!("enum {}", def.symbol),
        _ => format!("{} {}", def.kind, def.symbol),
    };
    Ok(Some(EditorHoverV19 {
        label: def.symbol.clone(),
        kind: def.kind.clone(),
        detail,
    }))
}

pub fn lsp_definition_locations_v19(
    source: &str,
    file_id: u32,
    symbol: &str,
) -> Result<Vec<EditorLocationV19>, Diagnostic> {
    let program = parse_program(source, file_id)?;
    let mut out = Vec::<EditorLocationV19>::new();
    collect_stmt_definitions(&program.statements, file_id, None, &mut out);
    out.retain(|item| item.symbol == symbol);
    sort_locations(&mut out);
    Ok(out)
}

pub fn lsp_reference_locations_v19(
    source: &str,
    file_id: u32,
    symbol: &str,
) -> Result<Vec<EditorLocationV19>, Diagnostic> {
    let program = parse_program(source, file_id)?;
    let mut out = Vec::<EditorLocationV19>::new();
    collect_stmt_references(&program.statements, symbol, file_id, &mut out);
    sort_locations(&mut out);
    Ok(out)
}

pub fn lsp_rename_preview_v19(
    source: &str,
    file_id: u32,
    old_name: &str,
    new_name: &str,
) -> Result<EditorRenamePreviewV19, Diagnostic> {
    if old_name.is_empty() || new_name.is_empty() || old_name == new_name {
        return Ok(EditorRenamePreviewV19 {
            edits: Vec::new(),
            replaced_count: 0,
        });
    }

    let mut defs = lsp_definition_locations_v19(source, file_id, old_name)?;
    let mut refs = lsp_reference_locations_v19(source, file_id, old_name)?;
    defs.append(&mut refs);
    sort_locations(&mut defs);

    let mut seen = BTreeSet::<(u32, u32, u32)>::new();
    let mut edits = Vec::<EditorRenameEditV19>::new();
    for loc in defs {
        let key = (loc.file_id, loc.line, loc.column);
        if !seen.insert(key) {
            continue;
        }
        edits.push(EditorRenameEditV19 {
            file_id: loc.file_id,
            line: loc.line,
            column: loc.column,
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        });
    }
    edits.sort_by(|lhs, rhs| {
        lhs.file_id
            .cmp(&rhs.file_id)
            .then(lhs.line.cmp(&rhs.line))
            .then(lhs.column.cmp(&rhs.column))
    });

    Ok(EditorRenamePreviewV19 {
        replaced_count: edits.len(),
        edits,
    })
}

pub fn multiroot_sorted_roots_v19(workspace_roots: &[String], policy: &JsonValue) -> Vec<String> {
    let mut roots = workspace_roots.to_vec();
    let policy_sort = policy
        .get("workspace_folder_order")
        .and_then(JsonValue::as_str)
        .unwrap_or("canonical_path_sort");
    if policy_sort == "canonical_path_sort" {
        roots.sort_by(|lhs, rhs| {
            normalize_root_for_sort(lhs)
                .as_bytes()
                .cmp(normalize_root_for_sort(rhs).as_bytes())
        });
    }
    roots.dedup();
    roots
}

pub fn lsp_multiroot_definition_locations_v19(
    workspace_files: &[(String, u32, String)],
    symbol: &str,
    policy: &JsonValue,
) -> Result<Vec<EditorLocationV19>, Diagnostic> {
    let roots = workspace_files
        .iter()
        .map(|item| item.0.clone())
        .collect::<Vec<String>>();
    let ordered_roots = multiroot_sorted_roots_v19(&roots, policy);

    let mut out = Vec::<EditorLocationV19>::new();
    for root in ordered_roots {
        for (file_root, file_id, source) in workspace_files {
            if *file_root != root {
                continue;
            }
            let program = parse_program(source, *file_id)?;
            let mut defs = Vec::<EditorLocationV19>::new();
            collect_stmt_definitions(&program.statements, *file_id, Some(&root), &mut defs);
            out.extend(defs.into_iter().filter(|item| item.symbol == symbol));
        }
    }
    sort_locations(&mut out);
    Ok(out)
}

pub fn format_source_with_contract_v19(
    source: &str,
    formatting_contract: &JsonValue,
) -> Result<String, SdkError> {
    let engine = formatting_contract
        .get("engine")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    if engine != "ocp_fmt_shared" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-FORMAT-ENGINE: unsupported engine `{engine}`"
        )));
    }

    let indent_size = formatting_contract
        .get("options")
        .and_then(|v| v.get("indent_size"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(2) as usize;
    if !(1..=8).contains(&indent_size) {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-FORMAT-INDENT: invalid indent_size {indent_size}"
        )));
    }

    let line_width = formatting_contract
        .get("options")
        .and_then(|v| v.get("line_width"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(100) as usize;

    let normalized = normalize_text(source);
    let mut out = String::new();
    let mut indent_level = 0usize;

    for raw_line in normalized.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            out.push('\n');
            continue;
        }

        if line.starts_with('}') && indent_level > 0 {
            indent_level -= 1;
        }

        let candidate = format!("{}{}", " ".repeat(indent_level * indent_size), line);
        out.push_str(candidate.trim_end());
        out.push('\n');

        if line.ends_with('{') {
            indent_level += 1;
        }
    }

    if out.is_empty() {
        out.push('\n');
    }

    let exceeds_width = out.lines().any(|line| line.len() > line_width);
    if exceeds_width {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-FORMAT-LINE-WIDTH: formatted output exceeds line_width {line_width}"
        )));
    }

    Ok(out)
}

pub fn governed_code_action_ids_for_lane_v19(
    lane: &str,
    code_actions_contract: &JsonValue,
    public_surface_contract: &JsonValue,
) -> Result<Vec<String>, SdkError> {
    let mut ids = code_actions_contract
        .get("code_action_ids")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(
                "X-V19-CODE-ACTIONS-CONTRACT: missing code_action_ids".to_string(),
            )
        })?
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    ids.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    ids.dedup();

    let mut public_ids = public_surface_contract
        .get("code_action_ids")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-PUBLIC-SURFACE: missing code_action_ids".to_string())
        })?
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    public_ids.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    public_ids.dedup();

    if ids != public_ids {
        return Err(SdkError::SupplyInvalid(
            "X-V19-CODE-ACTIONS-SYNC: contract and public surface mismatch".to_string(),
        ));
    }

    let strict_no_bypass = code_actions_contract
        .get("strict_lane_no_bypass")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    if strict_no_bypass && lane == "locked_v071" {
        let has_bypass_marker = ids.iter().any(|id| {
            let lower = id.to_ascii_lowercase();
            lower.contains("bypass") || lower.contains("wildcard") || lower.contains("downgrade")
        });
        if has_bypass_marker {
            return Err(SdkError::SupplyInvalid(
                "X-V19-CODE-ACTIONS-STRICT: bypass-like action detected in strict lane".to_string(),
            ));
        }
    }

    Ok(ids)
}

pub fn strict_lane_code_action_allowed_v19(
    lane: &str,
    action_id: &str,
    request: &JsonValue,
    code_actions_contract: &JsonValue,
) -> bool {
    let Some(ids) = code_actions_contract
        .get("code_action_ids")
        .and_then(JsonValue::as_array)
    else {
        return false;
    };
    let listed = ids.iter().any(|item| item.as_str() == Some(action_id));
    if !listed {
        return false;
    }

    if lane != "locked_v071" {
        return true;
    }

    let strict_no_bypass = code_actions_contract
        .get("strict_lane_no_bypass")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    if !strict_no_bypass {
        return true;
    }

    let bypass = request
        .get("bypass_strict")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let wildcard = request
        .get("wildcard")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let requested_mode = request
        .get("mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("workspace_edit");

    !(bypass || wildcard || requested_mode == "direct_write")
}

pub fn cli_bridge_contract_allows_v19(command: &str, contract: &JsonValue) -> bool {
    let normalized = command.split_whitespace().collect::<Vec<&str>>().join(" ");
    contract
        .get("commands")
        .and_then(JsonValue::as_array)
        .map(|commands| {
            commands
                .iter()
                .filter_map(JsonValue::as_str)
                .map(|item| item.split_whitespace().collect::<Vec<&str>>().join(" "))
                .any(|item| item == normalized)
        })
        .unwrap_or(false)
}

pub fn cli_bridge_output_policy_v19(contract: &JsonValue) -> (String, bool) {
    let output_mode = contract
        .get("output_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("json_only")
        .to_string();
    let text_fallback = contract
        .get("text_fallback_allowed")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    (output_mode, text_fallback)
}

pub fn code_action_apply_policy_v19(
    policy_contract: &JsonValue,
) -> Result<EditorCodeActionApplyPolicyV19, SdkError> {
    let patch_format = policy_contract
        .get("patch_format")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-APPLY-POLICY: missing patch_format".to_string())
        })?
        .to_string();
    let apply_mode = policy_contract
        .get("apply_mode")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-APPLY-POLICY: missing apply_mode".to_string())
        })?
        .to_string();
    let record_creation_point = policy_contract
        .get("record_creation_point")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-APPLY-POLICY: missing record_creation_point".to_string())
        })?
        .to_string();
    let allow_cli_apply_fallback = policy_contract
        .get("allow_cli_apply_fallback")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    Ok(EditorCodeActionApplyPolicyV19 {
        patch_format,
        apply_mode,
        record_creation_point,
        allow_cli_apply_fallback,
    })
}

pub fn validate_code_action_apply_request_v19(
    lane: &str,
    request: &JsonValue,
    policy: &EditorCodeActionApplyPolicyV19,
) -> bool {
    let patch_format = request
        .get("patch_format")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let apply_mode = request
        .get("apply_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let record_created = request
        .get("record_created")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    if patch_format != policy.patch_format || apply_mode != policy.apply_mode {
        return false;
    }

    if policy.record_creation_point == "apply" && !record_created {
        return false;
    }

    if lane == "locked_v071" && !policy.allow_cli_apply_fallback && apply_mode == "cli_apply" {
        return false;
    }

    true
}

pub fn debug_contract_profile_v19(
    contract: &JsonValue,
) -> Result<(String, String, String, String), SdkError> {
    let debug_mode = contract
        .get("debug_mode")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-DAP-CONTRACT: missing debug_mode".to_string())
        })?
        .to_string();
    let trace_acquisition = contract
        .get("trace_acquisition")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-DAP-CONTRACT: missing trace_acquisition".to_string())
        })?
        .to_string();
    let trace_path_scheme = contract
        .get("trace_artifact_path_scheme")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(
                "X-V19-DAP-CONTRACT: missing trace_artifact_path_scheme".to_string(),
            )
        })?
        .to_string();
    let step_semantics = contract
        .get("step_semantics")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-DAP-CONTRACT: missing step_semantics".to_string())
        })?
        .to_string();

    if debug_mode != "dap_replay_backed" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-DAP-CONTRACT: unsupported debug_mode `{debug_mode}`"
        )));
    }
    if trace_acquisition != "generate_on_launch" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-DAP-CONTRACT: unsupported trace_acquisition `{trace_acquisition}`"
        )));
    }
    if step_semantics != "trace_event_id_ascending" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-DAP-CONTRACT: unsupported step_semantics `{step_semantics}`"
        )));
    }
    if !trace_path_scheme.contains("<workspace_hash>") || !trace_path_scheme.contains("<run_id>") {
        return Err(SdkError::SupplyInvalid(
            "X-V19-DAP-CONTRACT: trace_artifact_path_scheme missing placeholders".to_string(),
        ));
    }

    Ok((
        debug_mode,
        trace_acquisition,
        trace_path_scheme,
        step_semantics,
    ))
}

pub fn dap_trace_path_v19(
    contract: &JsonValue,
    workspace_hash: &str,
    run_id: &str,
) -> Result<String, SdkError> {
    let (_, _, scheme, _) = debug_contract_profile_v19(contract)?;
    let path = scheme
        .replace("<workspace_hash>", workspace_hash)
        .replace("<run_id>", run_id);
    Ok(path)
}

pub fn dap_trace_events_from_source_v19(
    source: &str,
    file_id: u32,
) -> Result<Vec<EditorDapTraceEventV19>, Diagnostic> {
    let program = parse_program(source, file_id)?;
    let mut events = Vec::<EditorDapTraceEventV19>::new();
    let mut next_id = 1_u64;
    collect_trace_events(&program.statements, file_id, &mut next_id, &mut events);
    events.sort_by(|lhs, rhs| lhs.trace_event_id.cmp(&rhs.trace_event_id));
    Ok(events)
}

pub fn dap_launch_summary_v19(
    contract: &JsonValue,
    workspace_hash: &str,
    run_id: &str,
    source: &str,
    file_id: u32,
) -> Result<EditorDapLaunchSummaryV19, SdkError> {
    let (debug_mode, trace_acquisition, _scheme, step_semantics) =
        debug_contract_profile_v19(contract)?;
    let trace_path = dap_trace_path_v19(contract, workspace_hash, run_id)?;
    let events = dap_trace_events_from_source_v19(source, file_id)
        .map_err(|diag| SdkError::SupplyInvalid(format!("X-V19-DAP-LAUNCH: {}", diag.message)))?;

    Ok(EditorDapLaunchSummaryV19 {
        debug_mode,
        trace_acquisition,
        trace_path,
        step_semantics,
        events_count: events.len(),
    })
}

pub fn dap_breakpoint_mapping_v19(
    contract: &JsonValue,
    events: &[EditorDapTraceEventV19],
    breakpoints: &[EditorDapBreakpointV19],
) -> Result<Vec<EditorDapBreakpointMapEntryV19>, SdkError> {
    let mapping = contract
        .get("breakpoint_mapping")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    if mapping != "(file,span)->breakpoint_id->trace_event_id[]" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-DAP-MAPPING: unsupported breakpoint_mapping `{mapping}`"
        )));
    }

    let mut out = breakpoints
        .iter()
        .map(|bp| {
            let mut trace_event_ids = events
                .iter()
                .filter(|event| {
                    event.file_id == bp.file_id
                        && event.line == bp.line
                        && event.column >= bp.column
                })
                .map(|event| event.trace_event_id)
                .collect::<Vec<u64>>();
            trace_event_ids.sort_unstable();
            trace_event_ids.dedup();
            EditorDapBreakpointMapEntryV19 {
                breakpoint_id: bp.breakpoint_id,
                trace_event_ids,
            }
        })
        .collect::<Vec<EditorDapBreakpointMapEntryV19>>();
    out.sort_by(|lhs, rhs| lhs.breakpoint_id.cmp(&rhs.breakpoint_id));
    Ok(out)
}

pub fn dap_step_sequence_v19(events: &[EditorDapTraceEventV19]) -> Vec<u64> {
    let mut ids = events
        .iter()
        .map(|event| event.trace_event_id)
        .collect::<Vec<u64>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn dap_variables_for_event_v19(
    events: &[EditorDapTraceEventV19],
    trace_event_id: u64,
) -> Option<Vec<EditorDapVariableV19>> {
    events
        .iter()
        .find(|event| event.trace_event_id == trace_event_id)
        .map(|event| event.variables.clone())
}

pub fn required_bundled_binaries_v19(contract: &JsonValue) -> Result<Vec<String>, SdkError> {
    let Some(items) = contract.get("binaries").and_then(JsonValue::as_array) else {
        return Err(SdkError::SupplyInvalid(
            "X-V19-BUNDLED-BINARIES: missing binaries array".to_string(),
        ));
    };
    let mut out = items
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    out.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    out.dedup();
    Ok(out)
}

pub fn vsix_size_limit_for_channel_v19(contract: &JsonValue, channel: &str) -> Option<u64> {
    contract
        .get("limits_bytes")
        .and_then(JsonValue::as_object)
        .and_then(|map| map.get(channel))
        .and_then(JsonValue::as_u64)
}

pub fn publish_channels_v19(contract: &JsonValue) -> Result<(Vec<String>, bool), SdkError> {
    let channels = contract
        .get("channels")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-PUBLISH-CHANNELS: missing channels".to_string())
        })?
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    let rollback_required = contract
        .get("rollback_rehearsal_required")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    Ok((channels, rollback_required))
}

pub fn required_publish_files_v19(contract: &JsonValue) -> Result<Vec<String>, SdkError> {
    let files = contract
        .get("required_files")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid("X-V19-PUBLISH-REQ-FILES: missing required_files".to_string())
        })?
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    Ok(files)
}

pub fn required_publish_metadata_fields_v19(contract: &JsonValue) -> Result<Vec<String>, SdkError> {
    let fields = contract
        .get("required_metadata_fields")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(
                "X-V19-PUBLISH-REQ-METADATA: missing required_metadata_fields".to_string(),
            )
        })?
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();
    Ok(fields)
}

pub fn version_rule_matches_v19(rule: &str, actual: &str) -> bool {
    if actual == "unknown" {
        return false;
    }
    if let Some(prefix) = rule.strip_suffix(".x") {
        return actual.starts_with(prefix);
    }
    actual == rule
}

pub fn bootstrap_retention_plan_v19(
    policy_contract: &JsonValue,
    versions_oldest_to_newest: &[String],
    active_version: &str,
) -> Result<(Vec<String>, Vec<String>), SdkError> {
    let max_versions = policy_contract
        .get("max_versions_per_platform")
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(
                "X-V19-BOOTSTRAP-RETENTION: missing max_versions_per_platform".to_string(),
            )
        })? as usize;
    let cleanup_order = policy_contract
        .get("cleanup_order")
        .and_then(JsonValue::as_str)
        .unwrap_or("oldest_first");
    if cleanup_order != "oldest_first" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-V19-BOOTSTRAP-RETENTION: unsupported cleanup_order `{cleanup_order}`"
        )));
    }
    let delete_active = policy_contract
        .get("delete_active_version")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    let mut kept = Vec::<String>::new();
    let mut removed = Vec::<String>::new();
    for version in versions_oldest_to_newest {
        kept.push(version.clone());
        while kept.len() > max_versions {
            let candidate = kept.remove(0);
            if !delete_active && candidate == active_version {
                kept.push(candidate);
                break;
            }
            removed.push(candidate);
        }
    }

    let active_missing = !kept.iter().any(|item| item == active_version)
        && !versions_oldest_to_newest.is_empty()
        && !delete_active;
    if active_missing {
        kept.push(active_version.to_string());
    }

    Ok((kept, removed))
}

fn collect_stmt_symbols(stmts: &[Stmt], out: &mut Vec<EditorSymbolV19>) {
    for stmt in stmts {
        match stmt {
            Stmt::FnDef {
                name, span, body, ..
            } => {
                out.push(EditorSymbolV19 {
                    name: name.clone(),
                    kind: "function".to_string(),
                    line: span.line,
                    column: span.column,
                });
                collect_stmt_symbols(body, out);
            }
            Stmt::StructDecl { name, span, .. } => out.push(EditorSymbolV19 {
                name: name.clone(),
                kind: "struct".to_string(),
                line: span.line,
                column: span.column,
            }),
            Stmt::EnumDecl { name, span, .. } => out.push(EditorSymbolV19 {
                name: name.clone(),
                kind: "enum".to_string(),
                line: span.line,
                column: span.column,
            }),
            Stmt::Let { pattern, span, .. } => {
                let label = format!("{pattern:?}");
                out.push(EditorSymbolV19 {
                    name: label,
                    kind: "variable".to_string(),
                    line: span.line,
                    column: span.column,
                });
            }
            Stmt::ForEachCap { body, .. }
            | Stmt::ForRange { body, .. }
            | Stmt::Repeat { body, .. } => {
                collect_stmt_symbols(body, out);
            }
            Stmt::Match(stmt) => {
                collect_stmt_symbols(&stmt.ok_arm, out);
                collect_stmt_symbols(&stmt.degraded_arm, out);
                collect_stmt_symbols(&stmt.insufficient_arm, out);
                collect_stmt_symbols(&stmt.deferred_arm, out);
            }
            _ => {}
        }
    }
}

fn collect_stmt_definitions(
    stmts: &[Stmt],
    file_id: u32,
    root: Option<&str>,
    out: &mut Vec<EditorLocationV19>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::FnDef {
                name, span, body, ..
            } => {
                out.push(EditorLocationV19 {
                    file_id,
                    line: span.line,
                    column: span.column,
                    symbol: name.clone(),
                    kind: "function".to_string(),
                    root: root.map(ToOwned::to_owned),
                });
                collect_stmt_definitions(body, file_id, root, out);
            }
            Stmt::StructDecl { name, span, .. } => out.push(EditorLocationV19 {
                file_id,
                line: span.line,
                column: span.column,
                symbol: name.clone(),
                kind: "struct".to_string(),
                root: root.map(ToOwned::to_owned),
            }),
            Stmt::EnumDecl { name, span, .. } => out.push(EditorLocationV19 {
                file_id,
                line: span.line,
                column: span.column,
                symbol: name.clone(),
                kind: "enum".to_string(),
                root: root.map(ToOwned::to_owned),
            }),
            Stmt::Let { pattern, span, .. } => {
                for name in pattern_names_from_debug(&format!("{pattern:?}")) {
                    out.push(EditorLocationV19 {
                        file_id,
                        line: span.line,
                        column: span.column,
                        symbol: name,
                        kind: "variable".to_string(),
                        root: root.map(ToOwned::to_owned),
                    });
                }
            }
            Stmt::TryLet { name, span, .. } => out.push(EditorLocationV19 {
                file_id,
                line: span.line,
                column: span.column,
                symbol: name.clone(),
                kind: "variable".to_string(),
                root: root.map(ToOwned::to_owned),
            }),
            Stmt::ForEachCap {
                var, span, body, ..
            }
            | Stmt::ForRange {
                var, span, body, ..
            } => {
                out.push(EditorLocationV19 {
                    file_id,
                    line: span.line,
                    column: span.column,
                    symbol: var.clone(),
                    kind: "variable".to_string(),
                    root: root.map(ToOwned::to_owned),
                });
                collect_stmt_definitions(body, file_id, root, out);
            }
            Stmt::Repeat { body, .. } => {
                collect_stmt_definitions(body, file_id, root, out);
            }
            Stmt::Match(stmt) => {
                collect_stmt_definitions(&stmt.ok_arm, file_id, root, out);
                collect_stmt_definitions(&stmt.degraded_arm, file_id, root, out);
                collect_stmt_definitions(&stmt.insufficient_arm, file_id, root, out);
                collect_stmt_definitions(&stmt.deferred_arm, file_id, root, out);
            }
            _ => {}
        }
    }
}

fn collect_stmt_references(
    stmts: &[Stmt],
    symbol: &str,
    file_id: u32,
    out: &mut Vec<EditorLocationV19>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. }
            | Stmt::Return { value, .. }
            | Stmt::Guard { value, .. }
            | Stmt::Condition { value, .. }
            | Stmt::Commit { value, .. } => collect_expr_references(value, symbol, file_id, out),
            Stmt::TryLet {
                value, else_expr, ..
            } => {
                collect_expr_references(value, symbol, file_id, out);
                collect_expr_references(else_expr, symbol, file_id, out);
            }
            Stmt::Repeat { count, body, .. } => {
                collect_expr_references(count, symbol, file_id, out);
                collect_stmt_references(body, symbol, file_id, out);
            }
            Stmt::ForEachCap {
                iter, cap, body, ..
            } => {
                collect_expr_references(iter, symbol, file_id, out);
                collect_expr_references(cap, symbol, file_id, out);
                collect_stmt_references(body, symbol, file_id, out);
            }
            Stmt::ForRange {
                start, end, body, ..
            } => {
                collect_expr_references(start, symbol, file_id, out);
                collect_expr_references(end, symbol, file_id, out);
                collect_stmt_references(body, symbol, file_id, out);
            }
            Stmt::Observe {
                key,
                tier,
                ctx,
                budget,
                bind,
                span,
            } => {
                collect_expr_references(key, symbol, file_id, out);
                collect_expr_references(tier, symbol, file_id, out);
                collect_expr_references(ctx, symbol, file_id, out);
                collect_expr_references(budget, symbol, file_id, out);
                if bind == symbol {
                    out.push(EditorLocationV19 {
                        file_id,
                        line: span.line,
                        column: span.column,
                        symbol: symbol.to_string(),
                        kind: "reference".to_string(),
                        root: None,
                    });
                }
            }
            Stmt::Entangle {
                left,
                right,
                constraint,
                span,
            } => {
                collect_expr_references(constraint, symbol, file_id, out);
                if left == symbol || right == symbol {
                    out.push(EditorLocationV19 {
                        file_id,
                        line: span.line,
                        column: span.column,
                        symbol: symbol.to_string(),
                        kind: "reference".to_string(),
                        root: None,
                    });
                }
            }
            Stmt::FnDef { body, .. } => {
                collect_stmt_references(body, symbol, file_id, out);
            }
            Stmt::Match(stmt) => {
                collect_expr_references(&stmt.value, symbol, file_id, out);
                collect_stmt_references(&stmt.ok_arm, symbol, file_id, out);
                collect_stmt_references(&stmt.degraded_arm, symbol, file_id, out);
                collect_stmt_references(&stmt.insufficient_arm, symbol, file_id, out);
                collect_stmt_references(&stmt.deferred_arm, symbol, file_id, out);
            }
            _ => {}
        }
    }
}

fn collect_expr_references(
    expr: &Expr,
    symbol: &str,
    file_id: u32,
    out: &mut Vec<EditorLocationV19>,
) {
    match expr {
        Expr::Ident { name, span } => {
            if name == symbol {
                out.push(EditorLocationV19 {
                    file_id,
                    line: span.line,
                    column: span.column,
                    symbol: symbol.to_string(),
                    kind: "reference".to_string(),
                    root: None,
                });
            }
        }
        Expr::Call { callee, args, span } => {
            if callee == symbol {
                out.push(EditorLocationV19 {
                    file_id,
                    line: span.line,
                    column: span.column,
                    symbol: symbol.to_string(),
                    kind: "reference".to_string(),
                    root: None,
                });
            }
            for arg in args {
                collect_expr_references(arg, symbol, file_id, out);
            }
        }
        Expr::List { items, .. } => {
            for item in items {
                collect_expr_references(item, symbol, file_id, out);
            }
        }
        Expr::Map { entries, .. } => {
            for (_, value) in entries {
                collect_expr_references(value, symbol, file_id, out);
            }
        }
        Expr::Record { fields, .. } => {
            for (_, value) in fields {
                collect_expr_references(value, symbol, file_id, out);
            }
        }
        Expr::Try { value, .. } => collect_expr_references(value, symbol, file_id, out),
        Expr::FieldAccess { base, .. } => collect_expr_references(base, symbol, file_id, out),
        Expr::Int { .. } | Expr::Bool { .. } | Expr::String { .. } => {}
    }
}

fn pattern_names_from_debug(pattern_debug: &str) -> Vec<String> {
    let names = extract_quoted_segments(pattern_debug);
    if names.is_empty() {
        return vec![pattern_debug.to_string()];
    }
    names
}

fn extract_quoted_segments(raw: &str) -> Vec<String> {
    let mut out = Vec::<String>::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in raw.chars() {
        if in_quote {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                out.push(current.clone());
                current.clear();
                in_quote = false;
                continue;
            }
            current.push(ch);
            continue;
        }
        if ch == '"' {
            in_quote = true;
        }
    }
    out
}

fn normalize_root_for_sort(root: &str) -> String {
    root.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

fn sort_locations(out: &mut Vec<EditorLocationV19>) {
    out.sort_by(|lhs, rhs| {
        lhs.root
            .as_deref()
            .unwrap_or("")
            .as_bytes()
            .cmp(rhs.root.as_deref().unwrap_or("").as_bytes())
            .then(lhs.file_id.cmp(&rhs.file_id))
            .then(lhs.line.cmp(&rhs.line))
            .then(lhs.column.cmp(&rhs.column))
            .then(lhs.symbol.as_bytes().cmp(rhs.symbol.as_bytes()))
    });
    out.dedup_by(|lhs, rhs| {
        lhs.root == rhs.root
            && lhs.file_id == rhs.file_id
            && lhs.line == rhs.line
            && lhs.column == rhs.column
            && lhs.symbol == rhs.symbol
            && lhs.kind == rhs.kind
    });
}

fn collect_trace_events(
    stmts: &[Stmt],
    file_id: u32,
    next_id: &mut u64,
    out: &mut Vec<EditorDapTraceEventV19>,
) {
    for stmt in stmts {
        let (line, column) = stmt_span(stmt);
        out.push(EditorDapTraceEventV19 {
            trace_event_id: *next_id,
            file_id,
            line,
            column,
            kind: stmt_kind(stmt).to_string(),
            variables: stmt_variables(stmt),
        });
        *next_id += 1;

        match stmt {
            Stmt::FnDef { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::ForEachCap { body, .. }
            | Stmt::ForRange { body, .. } => collect_trace_events(body, file_id, next_id, out),
            Stmt::Match(item) => {
                collect_trace_events(&item.ok_arm, file_id, next_id, out);
                collect_trace_events(&item.degraded_arm, file_id, next_id, out);
                collect_trace_events(&item.insufficient_arm, file_id, next_id, out);
                collect_trace_events(&item.deferred_arm, file_id, next_id, out);
            }
            _ => {}
        }
    }
}

fn stmt_span(stmt: &Stmt) -> (u32, u32) {
    match stmt {
        Stmt::ModuleDecl { span, .. }
        | Stmt::ImportDecl { span, .. }
        | Stmt::ExportDecl { span, .. }
        | Stmt::StructDecl { span, .. }
        | Stmt::EnumDecl { span, .. }
        | Stmt::FnDef { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::TryLet { span, .. }
        | Stmt::Guard { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::ForEachCap { span, .. }
        | Stmt::ForRange { span, .. }
        | Stmt::Observe { span, .. }
        | Stmt::Commit { span, .. }
        | Stmt::Condition { span, .. }
        | Stmt::Entangle { span, .. } => (span.line, span.column),
        Stmt::Match(item) => (item.span.line, item.span.column),
    }
}

fn stmt_kind(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::ModuleDecl { .. } => "module_decl",
        Stmt::ImportDecl { .. } => "import_decl",
        Stmt::ExportDecl { .. } => "export_decl",
        Stmt::StructDecl { .. } => "struct_decl",
        Stmt::EnumDecl { .. } => "enum_decl",
        Stmt::FnDef { .. } => "fn_def",
        Stmt::Let { .. } => "let",
        Stmt::Return { .. } => "return",
        Stmt::TryLet { .. } => "try_let",
        Stmt::Guard { .. } => "guard",
        Stmt::Repeat { .. } => "repeat",
        Stmt::ForEachCap { .. } => "for_each_cap",
        Stmt::ForRange { .. } => "for_range",
        Stmt::Observe { .. } => "observe",
        Stmt::Commit { .. } => "commit",
        Stmt::Condition { .. } => "condition",
        Stmt::Entangle { .. } => "entangle",
        Stmt::Match(_) => "match",
    }
}

fn stmt_variables(stmt: &Stmt) -> Vec<EditorDapVariableV19> {
    match stmt {
        Stmt::Let { pattern, .. } => pattern_names_from_debug(&format!("{pattern:?}"))
            .into_iter()
            .map(|name| EditorDapVariableV19 {
                value: "<let>".to_string(),
                name,
            })
            .collect::<Vec<EditorDapVariableV19>>(),
        Stmt::TryLet { name, .. } => vec![EditorDapVariableV19 {
            name: name.clone(),
            value: "<try-let>".to_string(),
        }],
        Stmt::ForEachCap { var, .. } | Stmt::ForRange { var, .. } => vec![EditorDapVariableV19 {
            name: var.clone(),
            value: "<loop>".to_string(),
        }],
        _ => Vec::new(),
    }
}

fn diagnostic_severity_from_code(code: &str) -> &'static str {
    if code.starts_with("X-")
        || code.starts_with("P-")
        || code.starts_with("T-")
        || code.starts_with("R-")
    {
        "Error"
    } else if code.starts_with("RC-") {
        "Warning"
    } else {
        "Error"
    }
}

fn version_matches(rule: &str, actual: &str) -> bool {
    if let Some(prefix) = rule.strip_suffix(".x") {
        return actual.starts_with(prefix);
    }
    rule == actual
}
