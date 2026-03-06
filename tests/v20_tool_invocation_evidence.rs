use serde_json::json;

#[path = "v20_gate_c_common.rs"]
mod v20c;

#[test]
fn v20_tool_invocation_evidence() {
    v20c::ensure_run_manifest();

    let tooling = v20c::tooling_versions();
    let external = tooling
        .get("external")
        .and_then(serde_json::Value::as_array)
        .expect("tooling.external");
    assert!(
        !external.is_empty(),
        "external tooling list must not be empty"
    );

    let mut records = Vec::<serde_json::Value>::new();
    for entry in external {
        let tool = entry
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .expect("external.tool");
        let version_rule = entry
            .get("version_rule")
            .and_then(serde_json::Value::as_str)
            .expect("external.version_rule");
        assert!(
            !version_rule.trim().is_empty(),
            "external tool version_rule must not be empty: {tool}"
        );

        let args = match tool {
            "fuzz-harness" => vec!["--protocol", "contracts/v20/hardcore_test_protocol.v1.json"],
            "mutation-runner" => vec!["--scope", "ocl-runtime-core,ocl-sdk,ocl-cli"],
            "chaos-injector" => vec!["--fault-points", "filesystem,network,process,time,cassette"],
            _ => vec!["--help"],
        };

        let mut record = v20c::tool_invocation_record(tool, &args);
        record["tool_version_rule"] = json!(version_rule);
        records.push(record);
    }

    for row in &records {
        let has_binary_hash = row
            .get("tool_binary_sha256")
            .and_then(serde_json::Value::as_str)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        let has_unavailable_reason = row
            .get("tool_binary_sha256_unavailable_reason_code")
            .and_then(serde_json::Value::as_str)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        assert!(
            has_binary_hash || has_unavailable_reason,
            "tool invocation record must contain binary hash or unavailable reason"
        );
        assert!(
            row.get("tool_args_digest")
                .and_then(serde_json::Value::as_str)
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "tool invocation record must contain tool_args_digest"
        );
    }

    let report = json!({
        "schema": "ocl.w20.hardcore_tool_invocation_report.v1",
        "status": "PASS",
        "tool_records": records,
        "tool_count": external.len(),
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20c::run_manifest_sha256()
    });
    v20c::write_report("hardcore/tool_invocation_report.json", &report);
}
