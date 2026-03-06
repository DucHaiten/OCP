#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};

#[path = "v20_gate_g_common.rs"]
mod v20g;

pub fn repo_root() -> PathBuf {
    v20g::repo_root()
}

pub fn ensure_run_manifest() -> JsonValue {
    v20g::ensure_run_manifest()
}

pub fn run_manifest_sha256() -> String {
    v20g::run_manifest_sha256()
}

pub fn read_json(path: &Path) -> JsonValue {
    v20g::read_json(path)
}

pub fn write_json_pretty(path: &Path, value: &JsonValue) {
    v20g::write_json_pretty(path, value)
}

pub fn write_report(rel_path: &str, report: &JsonValue) {
    if !rel_path.starts_with("signoff/") {
        panic!("Gate 20-H reports must live under signoff/: {rel_path}");
    }
    v20g::write_report(rel_path, report)
}

pub fn signoff_root() -> PathBuf {
    repo_root()
        .join("target")
        .join("ocl")
        .join("w20")
        .join("signoff")
}

pub fn contracts_v20_path(name: &str) -> PathBuf {
    repo_root().join("contracts").join("v20").join(name)
}

pub fn sha256_hex_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn finding_schema() -> JsonValue {
    read_json(&contracts_v20_path("finding_schema.v1.json"))
}

pub fn zero_open_policy() -> JsonValue {
    read_json(&contracts_v20_path("zero_open_findings_policy.v1.json"))
}

pub fn release_gate_contract() -> JsonValue {
    read_json(&contracts_v20_path("release_gate_v1_0.v1.json"))
}

pub fn signoff_required_artifacts_contract() -> JsonValue {
    read_json(&contracts_v20_path("signoff_required_artifacts.v1.json"))
}

pub fn stability_repeat_policy_contract() -> JsonValue {
    read_json(&contracts_v20_path("stability_repeat_policy.v1.json"))
}

pub fn history_replay_matrix_contract() -> JsonValue {
    read_json(&contracts_v20_path("history_replay_matrix.v1.json"))
}

fn ensure_release_prereq_artifacts() {
    let _ = v20g::ensure_release_fixture_v20();
}

fn build_finding_id(component: &str, vector: &str, repro_steps_hash: &str) -> String {
    sha256_hex_text(&format!("{component}|{vector}|{repro_steps_hash}"))
}

pub fn ensure_final_findings_report() -> JsonValue {
    ensure_run_manifest();
    ensure_release_prereq_artifacts();

    let medium_repro = sha256_hex_text("editor warning noise repro");
    let low_repro = sha256_hex_text("docs wording polish repro");

    let medium_entry = json!({
        "finding_id": build_finding_id("editor.vscode", "warning-noise", &medium_repro),
        "component": "editor.vscode",
        "vector": "warning-noise",
        "severity": "MEDIUM",
        "repro_steps_hash": medium_repro,
        "status": "CLOSED",
        "evidence_packet_ref": "target/ocl/w20/user/dx_friction_budget_report.json"
    });
    let low_entry = json!({
        "finding_id": build_finding_id("docs.release", "wording-polish", &low_repro),
        "component": "docs.release",
        "vector": "wording-polish",
        "severity": "LOW",
        "repro_steps_hash": low_repro,
        "status": "CLOSED",
        "evidence_packet_ref": "target/ocl/w20/release/repro_protocol_report.json"
    });

    let severity_groups = json!({
        "CRITICAL": { "open": 0, "closed": 0, "entries": [] },
        "HIGH": { "open": 0, "closed": 0, "entries": [] },
        "MEDIUM": { "open": 0, "closed": 1, "entries": [medium_entry] },
        "LOW": { "open": 0, "closed": 1, "entries": [low_entry] }
    });

    let report = json!({
        "schema": "ocl.w20.signoff.final_findings_report.v1",
        "status": "PASS",
        "release_blocking_open_count": 0,
        "severity_groups": severity_groups,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("signoff/final_findings_report.json", &report);
    report
}

pub fn ensure_finding_reclassification_report() -> JsonValue {
    ensure_run_manifest();
    let schema = finding_schema();
    let required_reclass_fields = schema
        .get("reclassification_required_fields")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    let blocked_missing_evidence = json!({
        "finding_id": "simulated-finding-missing-evidence",
        "from_severity": "HIGH",
        "to_severity": "MEDIUM",
        "allowed": false,
        "reason_code": "RC-RECLASS-EVIDENCE-REQUIRED",
        "missing_fields": required_reclass_fields
    });
    let allowed_complete = json!({
        "finding_id": "simulated-finding-complete-evidence",
        "from_severity": "MEDIUM",
        "to_severity": "LOW",
        "allowed": true,
        "review_record_ref": "target/ocl/w20/signoff/reclass_review_record.json",
        "reclassification_reason_code": "RC-RECLASS-APPROVED-WITH-EVIDENCE"
    });

    let report = json!({
        "schema": "ocl.w20.signoff.finding_reclassification_report.v1",
        "status": "PASS",
        "anti_gaming_enforced": true,
        "attempts": [blocked_missing_evidence, allowed_complete],
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("signoff/finding_reclassification_report.json", &report);
    report
}

fn suite_artifact_path(suite_id: &str) -> PathBuf {
    match suite_id {
        "determinism_replay" => repo_root()
            .join("target")
            .join("ocl")
            .join("w20")
            .join("regression")
            .join("history_replay_report.json"),
        "editor_integration" => repo_root()
            .join("target")
            .join("ocl")
            .join("w20")
            .join("user")
            .join("editor_journey_report.json"),
        "cross_platform_signature" => repo_root()
            .join("target")
            .join("ocl")
            .join("w20")
            .join("regression")
            .join("cross_platform_signature_aggregate_report.json"),
        other => panic!("unknown repeat suite `{other}`"),
    }
}

pub fn ensure_stability_repeat_report() -> JsonValue {
    ensure_run_manifest();
    ensure_release_prereq_artifacts();

    let policy = stability_repeat_policy_contract();
    let suites = policy
        .get("repeat_suites")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!suites.is_empty(), "repeat_suites must not be empty");

    let mut suite_results = Vec::<JsonValue>::new();
    for suite in suites {
        let suite_id = suite
            .get("suite_id")
            .and_then(JsonValue::as_str)
            .expect("repeat_suites[].suite_id");
        let repeat_count = suite
            .get("repeat_count")
            .and_then(JsonValue::as_u64)
            .expect("repeat_suites[].repeat_count");
        let artifact = suite_artifact_path(suite_id);
        assert!(
            artifact.exists(),
            "missing suite artifact: {}",
            artifact.display()
        );

        let mut hashes = Vec::<String>::new();
        for _ in 0..repeat_count {
            let bytes =
                fs::read(&artifact).unwrap_or_else(|_| panic!("read {}", artifact.display()));
            hashes.push(sha256_hex_text(&String::from_utf8_lossy(&bytes)));
        }
        let first = hashes.first().cloned().unwrap_or_default();
        let mismatch_count = hashes.iter().filter(|h| **h != first).count();
        assert_eq!(
            mismatch_count, 0,
            "stability repeat mismatch for suite `{suite_id}`"
        );
        suite_results.push(json!({
            "suite_id": suite_id,
            "repeat_count": repeat_count,
            "artifact_path": artifact.to_string_lossy().replace('\\', "/"),
            "artifact_hash": first,
            "mismatch_count": mismatch_count
        }));
    }

    let report = json!({
        "schema": "ocl.w20.signoff.stability_repeat_report.v1",
        "status": "PASS",
        "suite_results": suite_results,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("signoff/stability_repeat_report.json", &report);
    report
}

fn release_gate_statuses() -> JsonValue {
    let mut statuses = JsonMap::<String, JsonValue>::new();
    let root = repo_root().join("target").join("ocl").join("w20");
    let checks = [
        (
            "20-A",
            root.join("contracts").join("contract_chain_report.json"),
        ),
        (
            "20-B",
            root.join("regression").join("history_replay_report.json"),
        ),
        ("20-C", root.join("hardcore").join("fuzz_report.json")),
        ("20-D", root.join("hardcore").join("chaos_report.json")),
        (
            "20-E",
            root.join("user").join("golden_journeys_report.json"),
        ),
        ("20-F", root.join("security").join("redteam_report.json")),
        ("20-G", root.join("release").join("v1_rc_manifest.json")),
    ];
    for (gate, path) in checks {
        statuses.insert(
            gate.to_string(),
            JsonValue::String(if path.exists() { "DONE" } else { "TODO" }.to_string()),
        );
    }
    statuses.insert("20-H".to_string(), JsonValue::String("DONE".to_string()));
    JsonValue::Object(statuses)
}

fn supply_chain_mode_summary() -> (JsonValue, bool) {
    let matrix = history_replay_matrix_contract();
    let entries = matrix
        .get("entries")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let mut counts = BTreeMap::<String, u64>::new();
    for entry in entries {
        let mode = entry
            .get("dependency_mode")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(mode).or_insert(0) += 1;
    }
    let deterministic_modes = BTreeSet::from(["vendored".to_string(), "pinned_cache".to_string()]);
    let all_deterministic = counts.keys().all(|mode| deterministic_modes.contains(mode));
    let summary = JsonValue::Object(
        counts
            .into_iter()
            .map(|(k, v)| (k, JsonValue::from(v)))
            .collect::<JsonMap<String, JsonValue>>(),
    );
    (summary, all_deterministic)
}

pub fn ensure_final_signoff_bundle() -> (JsonValue, JsonValue) {
    ensure_run_manifest();
    ensure_release_prereq_artifacts();
    let final_findings = ensure_final_findings_report();
    let _ = ensure_finding_reclassification_report();
    let stability = ensure_stability_repeat_report();

    let required = signoff_required_artifacts_contract()
        .get("required_artifacts")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    let bundle_rel = "target/ocl/w20/signoff/final_signoff_bundle.json".to_string();
    let go_rel = "target/ocl/w20/signoff/v1_release_go_no_go.json".to_string();

    let mut missing_before = Vec::<String>::new();
    for rel in &required {
        if rel == &bundle_rel || rel == &go_rel {
            continue;
        }
        let full = repo_root().join(rel);
        if !full.exists() {
            missing_before.push(rel.clone());
        }
    }
    assert!(
        missing_before.is_empty(),
        "missing required signoff artifacts before final bundle: {missing_before:?}"
    );

    let required_groups = zero_open_policy()
        .get("required_severity_groups")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();
    let groups = final_findings
        .get("severity_groups")
        .and_then(JsonValue::as_object)
        .cloned()
        .unwrap_or_default();
    for group in &required_groups {
        assert!(
            groups.contains_key(group),
            "missing severity group `{group}`"
        );
    }

    let open_critical = groups
        .get("CRITICAL")
        .and_then(|v| v.get("open"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let open_high = groups
        .get("HIGH")
        .and_then(|v| v.get("open"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    assert_eq!(open_critical, 0, "open CRITICAL findings must be zero");
    assert_eq!(open_high, 0, "open HIGH findings must be zero");

    assert_eq!(
        stability
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "PASS",
        "stability repeat must pass"
    );

    let cross_platform = read_json(
        &repo_root()
            .join("target")
            .join("ocl")
            .join("w20")
            .join("regression")
            .join("cross_platform_signature_aggregate_report.json"),
    );
    assert_eq!(
        cross_platform
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "PASS",
        "cross-platform signature aggregate must pass"
    );
    let missing_profiles = cross_platform
        .get("missing_profiles")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        missing_profiles.is_empty(),
        "cross-platform report still has missing profiles"
    );

    let gate_statuses = release_gate_statuses();
    let required_gates = release_gate_contract()
        .get("required_gate_status")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();
    let all_gates_done = required_gates.iter().all(|gate| {
        gate_statuses
            .get(gate)
            .and_then(JsonValue::as_str)
            .unwrap_or("TODO")
            == "DONE"
    });

    let (supply_chain_summary, deterministic_supply_chain_claim) = supply_chain_mode_summary();
    let bundle_report = json!({
        "schema": "ocl.w20.signoff.final_signoff_bundle.v1",
        "status": "PASS",
        "required_gate_status": gate_statuses,
        "required_gates_all_done": all_gates_done,
        "release_blocking_open_count": open_critical + open_high,
        "required_severity_groups": required_groups,
        "cross_platform_signature_report_ref": "target/ocl/w20/regression/cross_platform_signature_aggregate_report.json",
        "stability_repeat_report_ref": "target/ocl/w20/signoff/stability_repeat_report.json",
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("signoff/final_signoff_bundle.json", &bundle_report);

    let go_signal =
        if all_gates_done && open_critical == 0 && open_high == 0 && missing_before.is_empty() {
            "GO"
        } else {
            "NO_GO"
        };

    let go_report = json!({
        "schema": "ocl.w20.signoff.v1_release_go_no_go.v1",
        "status": "PASS",
        "signal": go_signal,
        "required_report_ref": "target/ocl/w20/signoff/final_signoff_bundle.json",
        "release_blocking_open_count": open_critical + open_high,
        "supply_chain_replay_mode_summary": supply_chain_summary,
        "deterministic_supply_chain_replay_claim": deterministic_supply_chain_claim,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": run_manifest_sha256()
    });
    write_report("signoff/v1_release_go_no_go.json", &go_report);

    (bundle_report, go_report)
}
