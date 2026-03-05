use std::sync::{Mutex, OnceLock};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ExecConfig, Executor, ReasonCode, ResultKind, Value,
};

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
fn fs_adapter_denies_parent_traversal_path() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = std::env::temp_dir().join("ocl_fs_boundary_root");
    std::fs::create_dir_all(&root).expect("create root");
    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read = EnvVarGuard::set("OCL_STD_FS_ALLOW_READ", "./**");

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
}
