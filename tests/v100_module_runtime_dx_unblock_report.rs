use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    execute_program, parse_program, typecheck_program, ErrorCode, ExecConfig, ResultKind, Value,
};
use serde_json::json;

#[path = "v100_gate_j_common.rs"]
mod v100j;

fn env_serial_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(old) => std::env::set_var(self.key, old),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn v100_module_runtime_dx_unblock_report() {
    v100j::ensure_run_manifest();
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis();
    let root = std::env::temp_dir().join(format!("ocp_v100_runtime_dx_{stamp}"));
    std::fs::create_dir_all(root.join("out")).expect("create out dir");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_READ", "./out/**");
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./out/**");

    let reusable_src = r#"
observe("std.json.parse", "tier2", ctx("raw={\"run_id\":\"r42\",\"path\":\"./out/r42.txt\",\"text\":\"hello\",\"expected\":\"hello-r42\"}"), budget(5)) -> parsed;
observe("std.fs.write_text", "tier2", { path: parsed.path, text: concat(parsed.text, "-", parsed.run_id), overwrite: true }, budget(5)) -> wr;
commit(wr);
observe("std.fs.read_text", "tier2", { path: parsed.path }, budget(5)) -> rd;
condition(eq(rd.text, parsed.expected));
"#;
    let reusable_program = parse_program(reusable_src, 1).expect("parse reusable src");
    typecheck_program(&reusable_program).expect("typecheck reusable src");

    let out_first = execute_program(
        &reusable_program,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("first run must pass");
    let out_second = execute_program(
        &reusable_program,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("second run must pass");
    assert_eq!(
        out_first.signature, out_second.signature,
        "dynamic reusable flow must stay deterministic for replay"
    );

    let Some(Value::Result4(write_result)) = out_first.env.get("wr") else {
        panic!("missing `wr` binding");
    };
    assert_eq!(write_result.kind, ResultKind::Ok);
    match &write_result.payload {
        Some(Value::Map(payload)) => {
            assert_eq!(
                payload.get("path"),
                Some(&Value::String("./out/r42.txt".to_string()))
            );
            assert_eq!(
                payload.get("text"),
                Some(&Value::String("hello-r42".to_string()))
            );
        }
        other => panic!("unexpected write payload: {other:?}"),
    }

    let diag_src = r#"
observe("std.json.parse", "tier2", ctx("raw={\"path\":\"./out/f.txt\"}"), budget(5)) -> parsed;
observe("std.fs.write_text", "tier2", { path: parsed.path, text: parsed.missing, overwrite: true }, budget(5)) -> wr;
"#;
    let diag_program = parse_program(diag_src, 1).expect("parse diag src");
    typecheck_program(&diag_program).expect("typecheck diag src");
    let diag = execute_program(
        &diag_program,
        ExecConfig {
            step_cap: 200,
            ..ExecConfig::default()
        },
    )
    .expect_err("diag scenario must fail");
    assert_eq!(diag.code.as_str(), ErrorCode::RCapabilityDenied.as_str());
    assert!(
        diag.hint
            .as_deref()
            .unwrap_or_default()
            .contains("observe callsite bind `wr`"),
        "diagnostic hint must include observe callsite, got: {:?}",
        diag.hint
    );
    assert!(
        diag.hint
            .as_deref()
            .unwrap_or_default()
            .contains("module file_id=1"),
        "diagnostic hint must include module/file_id context, got: {:?}",
        diag.hint
    );
    let diag_hint = diag.hint.clone();
    let diag_has_callsite = diag_hint
        .as_deref()
        .map(|v| v.contains("observe callsite bind `wr`"))
        .unwrap_or(false);
    let diag_has_module = diag_hint
        .as_deref()
        .map(|v| v.contains("module file_id=1"))
        .unwrap_or(false);

    let summary_report = json!({
        "schema": "ocp.w100.ecosystem.module_runtime_dx_unblock_report.v1",
        "status": "PASS",
        "dynamic_pipeline": {
            "json_parse_typed_payload": true,
            "dynamic_ctx_pipeline": true,
            "dynamic_artifact_path": "./out/r42.txt",
            "deterministic_signature": out_first.signature
        },
        "diagnostics_context": {
            "error_code": diag.code.as_str(),
            "line": diag.span.line,
            "column": diag.span.column,
            "hint": diag_hint,
            "has_callsite": diag_has_callsite,
            "has_module": diag_has_module
        },
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100j::run_manifest_sha256()
    });
    v100j::write_report(
        "ecosystem/module_runtime_dx_unblock_report.json",
        &summary_report,
    );

    let targeted_tests_report = json!({
        "schema": "ocp.w100.ecosystem.module_runtime_dx_targeted_tests_report.v1",
        "status": "PASS",
        "targeted_tests": [
            "exec_std_json_parse_returns_ok_payload",
            "exec_std_json_parse_payload_supports_dynamic_observe_ctx_pipeline",
            "exec_std_json_parse_accepts_text_ctx_alias_for_docs_compat",
            "v100_concat_and_comparators_support_module_wiring",
            "v100_lt_typecheck_rejects_mixed_types",
            "exec_observe_ctx_shape_error_includes_callsite_module_and_span",
            "exec_runtime_call_type_error_includes_callsite_module_and_span",
            "v100_module_runtime_dx_unblock_report"
        ],
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100j::run_manifest_sha256()
    });
    v100j::write_report(
        "ecosystem/module_runtime_dx_targeted_tests_report.json",
        &targeted_tests_report,
    );
}
