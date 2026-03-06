#[path = "w17_gate_g_common.rs"]
mod w17;

use std::path::PathBuf;

use serde_json::{json, Value as JsonValue};

fn extract_markdown_section<'a>(content: &'a str, heading: &str) -> &'a str {
    let start = content
        .find(heading)
        .unwrap_or_else(|| panic!("missing heading: {heading}"));
    let section = &content[start + heading.len()..];
    if let Some(next) = section.find("\n### ") {
        &section[..next]
    } else {
        section
    }
}

fn has_unqualified_phrase(section: &str, phrase: &str) -> bool {
    section.lines().any(|line| {
        let lowered = line.trim().to_lowercase();
        lowered.contains(phrase) && !lowered.contains("không") && !lowered.contains("cấm")
    })
}

#[test]
fn v017_release_positioning_guard_enforces_locked_claims() {
    let plan_path = PathBuf::from("OCP-OCL-MVP-PLAN-v0.17.md");
    let plan_text = w17::read_utf8(&plan_path);
    let positioning =
        extract_markdown_section(&plan_text, "### 8.3 Performance positioning (LOCKED)");

    let required_phrases = [
        "hoàn hảo theo GGPL",
        "không claim thay thế native runtime cho compute-heavy thuần số học",
        "AOT deterministic subset là hướng nâng cấp trước JIT sâu",
        "Cấm wording mơ hồ trong tài liệu phát hành",
    ];
    let missing_required = required_phrases
        .iter()
        .filter(|phrase| !positioning.contains(**phrase))
        .map(|phrase| phrase.to_string())
        .collect::<Vec<_>>();

    let banned_phrases = [
        "native-speed cho mọi workload",
        "thay thế native runtime cho compute-heavy",
    ];
    let unqualified_claims = banned_phrases
        .iter()
        .filter(|phrase| has_unqualified_phrase(positioning, phrase))
        .map(|phrase| phrase.to_string())
        .collect::<Vec<_>>();

    let supported_profile_path = PathBuf::from("contracts")
        .join("w17")
        .join("supported_platform_profile.v1.json");
    let supported_profile_raw = w17::read_utf8(&supported_profile_path);
    let supported_profile: JsonValue =
        serde_json::from_str(&supported_profile_raw).expect("parse supported_platform_profile");
    let supported_profiles_count = supported_profile
        .get("supported_profiles")
        .and_then(JsonValue::as_array)
        .map(|rows| rows.len())
        .unwrap_or(0);
    let supported_profiles_only = supported_profile
        .get("determinism_scope")
        .and_then(|v| v.get("supported_profiles_only"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    let semantic_path = PathBuf::from("contracts")
        .join("w17")
        .join("semantic_hash_taxonomy.v1.json");
    let semantic_raw = w17::read_utf8(&semantic_path);
    let semantic: JsonValue =
        serde_json::from_str(&semantic_raw).expect("parse semantic hash taxonomy");
    let behavior_claims_only = semantic
        .get("signature_contract")
        .and_then(|v| v.get("semantic_hash_only_for_behavior_claims"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    let pass = missing_required.is_empty()
        && unqualified_claims.is_empty()
        && supported_profiles_count > 0
        && supported_profiles_only
        && behavior_claims_only;

    let report = json!({
        "schema": "ocl.w17.release_positioning_guard_report.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "positioning": {
            "required_phrases": required_phrases,
            "missing_required": missing_required,
            "unqualified_claims": unqualified_claims
        },
        "contracts": {
            "supported_profile": {
                "path": supported_profile_path.to_string_lossy(),
                "supported_profiles_count": supported_profiles_count,
                "supported_profiles_only": supported_profiles_only
            },
            "semantic_hash_taxonomy": {
                "path": semantic_path.to_string_lossy(),
                "semantic_hash_only_for_behavior_claims": behavior_claims_only
            }
        },
        "pass": pass
    });
    w17::write_rc_report("v017_release_positioning_guard_report.json", &report);

    assert!(
        missing_required.is_empty(),
        "missing required locked positioning phrases: {:?}",
        missing_required
    );
    assert!(
        unqualified_claims.is_empty(),
        "found unqualified release claims: {:?}",
        unqualified_claims
    );
    assert!(
        supported_profiles_count > 0 && supported_profiles_only,
        "supported platform profile contract must stay strict"
    );
    assert!(
        behavior_claims_only,
        "semantic hash taxonomy must keep behavior-claims-only policy"
    );
}
