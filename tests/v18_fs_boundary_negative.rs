use std::sync::{Mutex, OnceLock};

use ocp::ocp::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};
use serde_json::json;

#[path = "v18_gate_d_common.rs"]
mod common;

fn env_serial_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

#[derive(Debug)]
struct EnvVarGuard {
    key: String,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.prev {
            std::env::set_var(&self.key, prev);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}

#[test]
fn v18_fs_boundary_negative_blocks_parent_traversal() {
    common::ensure_run_manifest();
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = std::env::temp_dir().join("ocp_v18_fs_boundary_root");
    std::fs::create_dir_all(&root).expect("create fs boundary root");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read = EnvVarGuard::set("OCP_STD_FS_ALLOW_READ", "./**");

    let src = r#"
observe("std.fs.read_text", "tier2", ctx("path=../escape.txt"), budget(5)) -> r;
"#;
    let program = parse_program(src, 1).expect("parse");
    typecheck_program(&program).expect("typecheck");
    let out = Executor::new(ExecConfig {
        step_cap: 400,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec");

    let Some(Value::Result4(result)) = out.env.get("r") else {
        panic!("missing result binding");
    };
    assert_eq!(result.kind, ResultKind::Insufficient);
    assert_eq!(result.reason, Some(ReasonCode::FsPathOutsideSandbox));

    let report = json!({
        "schema": "ocp.w18.security.fs_boundary_report.v1",
        "run_manifest_ref": "target/ocp/w18/meta/run_manifest.json",
        "reason_code": "RC-FS-PATH-OUTSIDE-SANDBOX",
        "result_kind": "INSUFFICIENT",
        "status": "PASS"
    });
    common::write_json_pretty(
        &common::w18_security_dir().join("fs_boundary_report.json"),
        &report,
    );
}
