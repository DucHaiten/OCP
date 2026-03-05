use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_ocl::ocp_ocl::{
    parse_program, typecheck_program, ErrorCode, ExecConfig, Executor, ReasonCode, ResultKind,
    Value,
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

fn temp_fs_root(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_commit_precondition_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dir");
    }
    std::fs::write(path, content).expect("write text");
}

#[test]
fn fs_commit_requires_observe_precondition_snapshot_match() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let root = temp_fs_root("stale");
    let target = root.join("data/out.txt");
    write_text(&target, "seed");

    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_write = EnvVarGuard::set("OCL_STD_FS_ALLOW_WRITE", "./data/**");
    let _allow_read = EnvVarGuard::set("OCL_STD_FS_ALLOW_READ", "./data/**");
    let _max_write = EnvVarGuard::set("OCL_STD_FS_MAX_WRITE_BYTES", "4096");

    let src = r#"
observe("std.fs.write_text", "tier2", ctx("path=./data/out.txt;text=first;overwrite=true"), budget(5)) -> w1;
observe("std.fs.write_text", "tier2", ctx("path=./data/out.txt;text=second;overwrite=true"), budget(5)) -> w2;
commit(w2);
commit(w1);
"#;

    let preflight_src = r#"
observe("std.fs.write_text", "tier2", ctx("path=./data/out.txt;text=first;overwrite=true"), budget(5)) -> w1;
observe("std.fs.write_text", "tier2", ctx("path=./data/out.txt;text=second;overwrite=true"), budget(5)) -> w2;
"#;
    let preflight_program = parse_program(preflight_src, 1).expect("parse preflight");
    typecheck_program(&preflight_program).expect("typecheck preflight");
    let preflight = Executor::new(ExecConfig {
        step_cap: 2_000,
        ..ExecConfig::default()
    })
    .run(&preflight_program)
    .expect("preflight exec should pass");
    let Some(Value::Result4(w1)) = preflight.env.get("w1") else {
        panic!("missing w1 preflight");
    };
    let Some(Value::Result4(w2)) = preflight.env.get("w2") else {
        panic!("missing w2 preflight");
    };
    assert_eq!(
        w1.kind,
        ResultKind::Ok,
        "w1 must be commit-eligible, reason={:?}",
        w1.reason
    );
    assert_eq!(
        w2.kind,
        ResultKind::Ok,
        "w2 must be commit-eligible, reason={:?}",
        w2.reason
    );

    let program = parse_program(src, 1).expect("parse");
    typecheck_program(&program).expect("typecheck");
    let err = Executor::new(ExecConfig {
        step_cap: 2_000,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect_err("stale commit must fail");

    assert_eq!(err.code, ErrorCode::XCommitForbidden);
    assert!(
        err.message.contains("std.fs commit apply failed"),
        "expected fs precondition commit failure, got: {}",
        err.message
    );
    assert_eq!(err.root_reason, Some(ReasonCode::PolicyDenied));
    let final_text = std::fs::read_to_string(&target).expect("read final file");
    assert_eq!(
        final_text, "second",
        "first stale commit must not overwrite state after newer commit"
    );
}
