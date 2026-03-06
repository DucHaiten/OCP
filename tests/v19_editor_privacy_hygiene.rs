use serde_json::json;

#[path = "v19_gate_a_common.rs"]
mod v19;

fn violates_privacy(line: &str, forbidden_tokens: &[String]) -> bool {
    let lower = line.to_ascii_lowercase();
    forbidden_tokens
        .iter()
        .any(|token| lower.contains(&token.to_ascii_lowercase()))
}

#[test]
fn v19_editor_privacy_hygiene_policy_blocks_sensitive_tokens() {
    v19::ensure_run_manifest();

    let policy_path = v19::contracts_root()
        .join("editor")
        .join("editor_privacy_policy.v1.json");
    let policy = v19::read_json(&policy_path);
    let forbidden = policy
        .get("forbidden_log_fields")
        .and_then(serde_json::Value::as_array)
        .expect("forbidden_log_fields")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect::<Vec<String>>();
    let env_dump_allowed = policy
        .get("env_dump_allowed")
        .and_then(serde_json::Value::as_bool)
        .expect("env_dump_allowed");

    assert!(!forbidden.is_empty(), "forbidden fields must not be empty");
    assert!(!env_dump_allowed, "env_dump_allowed must be false");

    let safe_line = "OCL diagnostics ready for workspace";
    let unsafe_line = "received auth_token from request";
    assert!(!violates_privacy(safe_line, &forbidden));
    assert!(violates_privacy(unsafe_line, &forbidden));

    let report = json!({
        "schema": "ocl.w19.security.editor_privacy_hygiene_report.v1",
        "status": "PASS",
        "forbidden_fields_count": forbidden.len(),
        "env_dump_allowed": env_dump_allowed,
        "policy_path": policy_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19::run_manifest_sha256()
    });
    v19::write_report("security/editor_privacy_hygiene_report.json", &report);
}
