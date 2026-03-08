use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};

#[path = "v18_gate_f_common.rs"]
mod common;

fn contains_unqualified_claim(section: &str, phrase: &str) -> bool {
    section.lines().any(|line| {
        let lowered = line.trim().to_lowercase();
        lowered.contains(phrase) && !lowered.contains("khong") && !lowered.contains("không")
    })
}

#[test]
fn v18_release_positioning_guard_keeps_supported_profile_claims_bounded() {
    common::ensure_run_manifest();

    let plan_path = PathBuf::from("docs/plans/history/OCP-MVP-PLAN-v0.18.md");
    let plan_text = std::fs::read_to_string(&plan_path).expect("read v0.18 plan");
    let plan_lower = plan_text.to_lowercase();

    let required_phrases = [
        "wording guard",
        "không claim vượt supported profile",
        "performance positioning",
    ];
    let missing_required = required_phrases
        .iter()
        .filter(|phrase| !plan_lower.contains(&phrase.to_lowercase()))
        .map(|phrase| phrase.to_string())
        .collect::<Vec<String>>();

    let banned_phrases = [
        "native-speed cho mọi workload",
        "thay thế native runtime cho compute-heavy",
    ];
    let unqualified_claims = banned_phrases
        .iter()
        .filter(|phrase| contains_unqualified_claim(&plan_lower, phrase))
        .map(|phrase| phrase.to_string())
        .collect::<Vec<String>>();

    let supported_profile = common::read_json(
        &common::repo_root()
            .join("contracts")
            .join("platform")
            .join("supported_profile.v1.json"),
    );
    let profiles_count = supported_profile
        .get("supported_profiles")
        .and_then(JsonValue::as_array)
        .map(|rows| rows.len())
        .unwrap_or(0);
    let canonicalization = supported_profile
        .get("canonicalization")
        .and_then(JsonValue::as_object)
        .cloned()
        .unwrap_or_default();
    let has_locale = canonicalization
        .get("locale")
        .and_then(JsonValue::as_str)
        .is_some();
    let has_timezone = canonicalization
        .get("timezone")
        .and_then(JsonValue::as_str)
        .is_some();

    let pass = missing_required.is_empty()
        && unqualified_claims.is_empty()
        && profiles_count > 0
        && has_locale
        && has_timezone;

    let report = json!({
        "schema": "ocp.w18.rc.release_positioning_guard_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "required_phrases": required_phrases,
        "missing_required": missing_required,
        "unqualified_claims": unqualified_claims,
        "supported_profile": {
            "path": "contracts/platform/supported_profile.v1.json",
            "profiles_count": profiles_count,
            "has_locale": has_locale,
            "has_timezone": has_timezone
        },
        "status": if pass { "PASS" } else { "BLOCK" }
    });
    common::write_json_pretty(
        &common::w18_rc_dir().join("release_positioning_guard_report.json"),
        &report,
    );

    assert!(pass, "release positioning guard failed");
}
