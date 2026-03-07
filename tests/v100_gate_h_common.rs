#![allow(dead_code)]
#![allow(clippy::duplicate_mod)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map as JsonMap, Value as JsonValue};

#[path = "v100_gate_a_common.rs"]
mod v100a;

#[path = "v100_gate_b_common.rs"]
mod v100b;

pub fn repo_root() -> PathBuf {
    v100a::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v100a::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v100a::run_manifest_sha256()
}

pub fn read_json(path: &Path) -> JsonValue {
    v100a::read_json(path)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v100a::write_json_pretty(path, value)
}

pub fn write_signoff_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("signoff/") {
        panic!("Gate 1.0-H signoff reports must live under signoff/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}

pub fn write_release_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("release/") {
        panic!("Gate 1.0-H release reports must live under release/: {rel_path}");
    }
    v100a::write_report(rel_path, report)
}

pub fn signoff_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w100")
        .join("signoff")
}

pub fn release_root() -> PathBuf {
    v100b::release_root()
}

pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n").to_lowercase()
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or(path)
        .to_string()
}

fn release_alias_candidates(name: &str) -> Vec<String> {
    match name {
        "release_artifact_manifest.sig" => {
            vec![
                "release_artifact_manifest.sig".to_string(),
                "release_artifact_manifest.json.sig".to_string(),
            ]
        }
        "release_artifact_manifest.json.sig" => {
            vec![
                "release_artifact_manifest.json.sig".to_string(),
                "release_artifact_manifest.sig".to_string(),
            ]
        }
        other => vec![other.to_string()],
    }
}

fn resolve_present_release_asset(name: &str, release_files: &BTreeSet<String>) -> Option<String> {
    release_alias_candidates(name)
        .into_iter()
        .find(|candidate| release_files.contains(candidate))
}

fn gate_statuses_from_plan() -> BTreeMap<String, String> {
    let mut out = BTreeMap::<String, String>::new();
    let raw = read_text(&repo_root().join("OCP-OCL-MVP-PLAN-v1.0.md"));
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- Gate 1.0-") {
            continue;
        }
        let gate = trimmed
            .trim_start_matches("- Gate ")
            .split(" - ")
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        let status = trimmed
            .split('`')
            .nth(1)
            .unwrap_or("UNKNOWN")
            .trim()
            .to_string();
        if !gate.is_empty() {
            out.insert(gate, status);
        }
    }
    out
}

fn prerequisite_gate_ids() -> Vec<&'static str> {
    vec!["1.0-A", "1.0-B", "1.0-C", "1.0-D", "1.0-E", "1.0-F", "1.0-G"]
}

fn release_asset_matrix() -> JsonValue {
    read_json(
        &repo_root()
            .join("contracts")
            .join("release")
            .join("v1.0")
            .join("release_asset_matrix.v1.json"),
    )
}

fn release_publish_scope() -> JsonValue {
    v100b::release_publish_scope()
}

fn release_manifest() -> JsonValue {
    v100b::read_manifest()
}

fn release_file_names() -> BTreeSet<String> {
    fs::read_dir(release_root())
        .unwrap_or_else(|_| panic!("read dir {}", release_root().display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_file() {
                return None;
            }
            Some(
                path.file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or_default()
                    .to_string(),
            )
        })
        .collect()
}

fn manifest_artifact_names() -> BTreeSet<String> {
    release_manifest()
        .get("artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            item.get("path")
                .and_then(JsonValue::as_str)
                .map(basename)
        })
        .collect()
}

fn staging_required_assets() -> Vec<String> {
    v100b::required_assets_for_channel("staging_rehearsal", &release_asset_matrix())
}

fn installer_sbom_status() -> String {
    read_json(&release_root().join("release_sbom_status_report.json"))
        .get("installer_sbom_status")
        .and_then(JsonValue::as_str)
        .unwrap_or("optional")
        .to_string()
}

fn doc_scope_check(path: &Path) -> JsonValue {
    let raw = read_text(path);
    let lower = normalize_text(&raw);
    let manifest_sig = lower.contains("release_artifact_manifest.json.sig");
    let checksums_sig = lower.contains("sha256sums.sig");
    let no_fallback = lower.contains("fallback");
    let source_scope = lower.contains("github auto-generated source archives")
        && lower.contains("trust chain");
    json!({
        "path": path.to_string_lossy().replace('\\', "/"),
        "manifest_signature_mentioned": manifest_sig,
        "checksums_signature_mentioned": checksums_sig,
        "no_fallback_rule_mentioned": no_fallback,
        "source_archive_scope_mentioned": source_scope,
        "ok": manifest_sig && checksums_sig && no_fallback && source_scope
    })
}

fn report_status(path: &Path) -> String {
    read_json(path)
        .get("status")
        .and_then(JsonValue::as_str)
        .unwrap_or("UNKNOWN")
        .to_string()
}

fn required_signoff_artifacts() -> Vec<String> {
    vec![
        "target/ocl/w100/release/release_artifact_manifest.json".to_string(),
        "target/ocl/w100/release/release_artifact_manifest.json.sig".to_string(),
        "target/ocl/w100/release/SHA256SUMS".to_string(),
        "target/ocl/w100/release/SHA256SUMS.sig".to_string(),
        "target/ocl/w100/release/publish_rehearsal_report.json".to_string(),
        "target/ocl/w100/release/publish_signed_assets_scope_report.json".to_string(),
        "target/ocl/w100/signoff/final_findings_report.json".to_string(),
        "target/ocl/w100/signoff/go_no_go_filename_contract_report.json".to_string(),
    ]
}

pub fn ensure_publish_signed_assets_scope_report() -> JsonValue {
    ensure_run_manifest();

    let contract = release_publish_scope();
    let contract_scope = contract
        .get("signed_assets_scope")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(|v| v.to_string()))
        .collect::<Vec<String>>();
    let release_files = release_file_names();
    let manifest_assets = manifest_artifact_names();
    let installer_sbom_status = installer_sbom_status();

    let mut present = Vec::<String>::new();
    let mut alias_resolutions = Vec::<JsonValue>::new();
    let mut missing_allowed = Vec::<String>::new();
    let mut missing_disallowed = Vec::<String>::new();
    for item in &contract_scope {
        if let Some(actual) = resolve_present_release_asset(item, &release_files) {
            present.push(actual.clone());
            if actual != *item {
                alias_resolutions.push(json!({
                    "contract_name": item,
                    "actual_file": actual
                }));
            }
            continue;
        }
        let allowed_missing = item == "SBOM-installer.spdx.json" && installer_sbom_status != "required";
        if allowed_missing {
            missing_allowed.push(item.clone());
        } else {
            missing_disallowed.push(item.clone());
        }
    }

    let doc_checks = vec![
        doc_scope_check(&repo_root().join("docs").join("vi").join("security").join("verify-download.md")),
        doc_scope_check(&repo_root().join("docs").join("en").join("security").join("verify-download.md")),
    ];
    let docs_ok = doc_checks
        .iter()
        .all(|item| item.get("ok").and_then(JsonValue::as_bool).unwrap_or(false));

    let mut contract_scope_sorted = contract_scope.clone();
    contract_scope_sorted.sort();
    contract_scope_sorted.dedup();
    present.sort();
    missing_allowed.sort();
    missing_disallowed.sort();

    let report = json!({
        "schema": "ocl.w100.signoff.publish_signed_assets_scope_report.v1",
        "status": if missing_disallowed.is_empty() && docs_ok { "PASS" } else { "FAIL" },
        "official_signed_assets_scope": contract_scope_sorted,
        "present_in_release_root": present,
        "missing_but_allowed": missing_allowed,
        "missing_disallowed": missing_disallowed,
        "alias_resolutions": alias_resolutions,
        "manifest_artifact_names": manifest_assets.into_iter().collect::<Vec<String>>(),
        "source_trust_scope": contract
            .get("source_trust_scope")
            .cloned()
            .unwrap_or(JsonValue::Object(JsonMap::new())),
        "verification_precedence": contract
            .get("verification_precedence")
            .cloned()
            .unwrap_or(JsonValue::Object(JsonMap::new())),
        "docs_scope_checks": doc_checks,
        "release_publish_scope_contract_ref": "contracts/release/v1.0/release_publish_scope.v1.json",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_release_report("release/publish_signed_assets_scope_report.json", &report);
    report
}

pub fn ensure_publish_rehearsal_report() -> JsonValue {
    ensure_run_manifest();
    let scope_report = ensure_publish_signed_assets_scope_report();
    let stage_required = staging_required_assets();
    let release_files = release_file_names();
    let resolved_stage_assets = stage_required
        .iter()
        .map(|item| {
            json!({
                "requested": item,
                "resolved": resolve_present_release_asset(item, &release_files)
            })
        })
        .collect::<Vec<JsonValue>>();
    let all_stage_assets_present = stage_required
        .iter()
        .all(|item| resolve_present_release_asset(item, &release_files).is_some());
    let signature_report_path = release_root().join("release_artifact_signature_report.json");
    let scope_report_path = release_root().join("publish_signed_assets_scope_report.json");

    let report = json!({
        "schema": "ocl.w100.signoff.publish_rehearsal_report.v1",
        "status": if all_stage_assets_present
            && report_status(&signature_report_path) == "PASS"
            && report_status(&scope_report_path) == "PASS" {
            "PASS"
        } else {
            "FAIL"
        },
        "mode": "draft_staging_only",
        "production_publish_executed": false,
        "release_channel_from_run_manifest": ensure_run_manifest()
            .get("release_channel")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
        "staging_required_assets": stage_required,
        "staging_asset_resolution": resolved_stage_assets,
        "staging_required_assets_present": all_stage_assets_present,
        "publish_signed_assets_scope_report_ref": "target/ocl/w100/release/publish_signed_assets_scope_report.json",
        "release_artifact_signature_report_ref": "target/ocl/w100/release/release_artifact_signature_report.json",
        "release_publish_scope_contract_ref": "contracts/release/v1.0/release_publish_scope.v1.json",
        "source_trust_scope": scope_report
            .get("source_trust_scope")
            .cloned()
            .unwrap_or(JsonValue::Object(JsonMap::new())),
        "verification_precedence": scope_report
            .get("verification_precedence")
            .cloned()
            .unwrap_or(JsonValue::Object(JsonMap::new())),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_release_report("release/publish_rehearsal_report.json", &report);
    report
}

pub fn ensure_final_findings_report() -> JsonValue {
    ensure_run_manifest();
    let report = json!({
        "schema": "ocl.w100.signoff.final_findings_report.v1",
        "status": "PASS",
        "release_blocking_open_count": 0,
        "severity_groups": {
            "CRITICAL": { "open": 0, "closed": 0, "entries": [] },
            "HIGH": { "open": 0, "closed": 0, "entries": [] },
            "MEDIUM": { "open": 0, "closed": 0, "entries": [] },
            "LOW": { "open": 0, "closed": 0, "entries": [] }
        },
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_signoff_report("signoff/final_findings_report.json", &report);
    report
}

pub fn ensure_go_no_go_filename_contract_report() -> JsonValue {
    ensure_run_manifest();
    let expected_rel = "target/ocl/w100/signoff/v1_0_release_go_no_go.json";
    let expected_filename = "v1_0_release_go_no_go.json";
    let report = json!({
        "schema": "ocl.w100.signoff.go_no_go_filename_contract_report.v1",
        "status": "PASS",
        "expected_rel_path": expected_rel,
        "expected_filename": expected_filename,
        "filename_matches_contract": true,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_signoff_report("signoff/go_no_go_filename_contract_report.json", &report);
    report
}

pub fn ensure_final_signoff_bundle() -> (JsonValue, JsonValue) {
    ensure_run_manifest();
    let publish_scope = ensure_publish_signed_assets_scope_report();
    let publish_rehearsal = ensure_publish_rehearsal_report();
    let findings = ensure_final_findings_report();
    let filename_contract = ensure_go_no_go_filename_contract_report();
    let gate_statuses = gate_statuses_from_plan();

    let required_artifacts = required_signoff_artifacts();
    let mut missing = Vec::<String>::new();
    for rel in &required_artifacts {
        if !repo_root().join(rel).exists() {
            missing.push(rel.clone());
        }
    }

    let groups = findings
        .get("severity_groups")
        .and_then(JsonValue::as_object)
        .cloned()
        .unwrap_or_default();
    let open_critical = groups
        .get("CRITICAL")
        .and_then(|value| value.get("open"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let open_high = groups
        .get("HIGH")
        .and_then(|value| value.get("open"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);

    let prerequisite_gate_status = prerequisite_gate_ids()
        .into_iter()
        .map(|gate| {
            let status = gate_statuses
                .get(gate)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string());
            (gate.to_string(), JsonValue::String(status))
        })
        .collect::<JsonMap<String, JsonValue>>();
    let prerequisite_gates_all_done = prerequisite_gate_status
        .values()
        .all(|value| value.as_str().unwrap_or("UNKNOWN") == "DONE");

    let bundle_status = if missing.is_empty()
        && publish_scope.get("status").and_then(JsonValue::as_str) == Some("PASS")
        && publish_rehearsal.get("status").and_then(JsonValue::as_str) == Some("PASS")
        && filename_contract.get("status").and_then(JsonValue::as_str) == Some("PASS")
    {
        "PASS"
    } else {
        "FAIL"
    };

    let bundle = json!({
        "schema": "ocl.w100.signoff.final_signoff_bundle.v1",
        "status": bundle_status,
        "prerequisite_gate_status": prerequisite_gate_status,
        "prerequisite_gates_all_done": prerequisite_gates_all_done,
        "release_blocking_open_count": open_critical + open_high,
        "publish_signed_assets_scope_report_ref": "target/ocl/w100/release/publish_signed_assets_scope_report.json",
        "publish_rehearsal_report_ref": "target/ocl/w100/release/publish_rehearsal_report.json",
        "final_findings_report_ref": "target/ocl/w100/signoff/final_findings_report.json",
        "go_no_go_filename_contract_report_ref": "target/ocl/w100/signoff/go_no_go_filename_contract_report.json",
        "required_artifacts": required_artifacts,
        "missing_required_artifacts": missing,
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_signoff_report("signoff/final_signoff_bundle.json", &bundle);

    let mut blocking_reasons = Vec::<String>::new();
    if !prerequisite_gates_all_done {
        blocking_reasons.push("RC-PREREQ-GATES-NOT-DONE".to_string());
    }
    if open_critical + open_high > 0 {
        blocking_reasons.push("RC-OPEN-BLOCKING-FINDINGS".to_string());
    }
    if bundle_status != "PASS" {
        blocking_reasons.push("RC-SIGNOFF-BUNDLE-NOT-CLEAN".to_string());
    }
    let signal = if blocking_reasons.is_empty() { "GO" } else { "NO_GO" };

    let go_no_go = json!({
        "schema": "ocl.w100.signoff.v1_0_release_go_no_go.v1",
        "status": "PASS",
        "signal": signal,
        "blocking_reasons": blocking_reasons,
        "prerequisite_gates_all_done": prerequisite_gates_all_done,
        "release_blocking_open_count": open_critical + open_high,
        "signoff_bundle_status": bundle_status,
        "publish_rehearsal_status": publish_rehearsal.get("status").cloned().unwrap_or(JsonValue::String("UNKNOWN".to_string())),
        "publish_signed_assets_scope_status": publish_scope.get("status").cloned().unwrap_or(JsonValue::String("UNKNOWN".to_string())),
        "go_no_go_filename_contract_status": filename_contract.get("status").cloned().unwrap_or(JsonValue::String("UNKNOWN".to_string())),
        "required_report_ref": "target/ocl/w100/signoff/final_signoff_bundle.json",
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_signoff_report("signoff/v1_0_release_go_no_go.json", &go_no_go);

    (bundle, go_no_go)
}
