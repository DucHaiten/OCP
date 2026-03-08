use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{
    parse_program, typecheck_program, Diagnostic, ExecConfig, ExecOutput, Executor, ReasonCode,
    ResultKind, Value,
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
    std::env::temp_dir().join(format!("ocp_std_fs_commit_{tag}_{stamp}"))
}

fn write_text(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, content).expect("write content");
}

fn run_program(src: &str) -> Result<ExecOutput, Diagnostic> {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 800,
        ..ExecConfig::default()
    })
    .run(&program)
}

#[test]
fn std_fs_write_text_commit_roundtrip_and_overwrite_policy() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("roundtrip");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_READ", "./out/**");
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./out/**");

    let src = r#"
observe("std.fs.write_text", "tier2", ctx("path=./out/result.txt;text=hello;overwrite=true"), budget(5)) -> w1;
commit(w1);
observe("std.fs.read_text", "tier2", ctx("path=./out/result.txt"), budget(5)) -> r1;
observe("std.fs.write_text", "tier2", ctx("path=./out/result.txt;text=second;overwrite=false"), budget(5)) -> w2;
"#;

    let out = run_program(src).expect("exec should pass");
    assert_eq!(out.commits.len(), 1);
    assert_eq!(out.commits[0].key, "std.fs.write_text");
    assert_eq!(out.commits[0].kind, ResultKind::Ok);

    let Some(Value::Result4(r1)) = out.env.get("r1") else {
        panic!("expected read result");
    };
    assert_eq!(r1.kind, ResultKind::Ok);
    match &r1.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("text"), Some(&Value::String("hello".to_string())));
            assert_eq!(map.get("truncated"), Some(&Value::Bool(false)));
        }
        _ => panic!("expected read payload map"),
    }

    let Some(Value::Result4(w2)) = out.env.get("w2") else {
        panic!("expected write result");
    };
    assert_eq!(w2.kind, ResultKind::Insufficient);
    assert_eq!(w2.reason, Some(ReasonCode::PolicyDenied));
}

#[test]
fn std_fs_mkdir_rename_remove_commit_flow() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("mkdir_rename_remove");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_READ", "./work/**");
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./work/**");
    let _allow_remove_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_REMOVE", "./work/**");
    let _allow_rename_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_RENAME", "./work/**");

    let src = r#"
observe("std.fs.mkdir", "tier2", ctx("path=./work;recursive=true"), budget(5)) -> mk;
commit(mk);
observe("std.fs.write_text", "tier2", ctx("path=./work/a.txt;text=abc;overwrite=true"), budget(5)) -> w;
commit(w);
observe("std.fs.rename", "tier2", ctx("from=./work/a.txt;to=./work/b.txt;overwrite=false"), budget(5)) -> rn;
commit(rn);
observe("std.fs.stat", "tier2", ctx("path=./work/b.txt"), budget(5)) -> s1;
observe("std.fs.remove", "tier2", ctx("path=./work;recursive=true"), budget(5)) -> rm;
commit(rm);
observe("std.fs.stat", "tier2", ctx("path=./work/b.txt"), budget(5)) -> s2;
"#;

    let out = run_program(src).expect("exec should pass");
    assert_eq!(out.commits.len(), 4);
    assert_eq!(out.commits[0].key, "std.fs.mkdir");
    assert_eq!(out.commits[1].key, "std.fs.write_text");
    assert_eq!(out.commits[2].key, "std.fs.rename");
    assert_eq!(out.commits[3].key, "std.fs.remove");

    let Some(Value::Result4(s1)) = out.env.get("s1") else {
        panic!("expected stat before remove");
    };
    assert_eq!(s1.kind, ResultKind::Ok);
    match &s1.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("exists"), Some(&Value::Bool(true)));
        }
        _ => panic!("expected stat payload map"),
    }

    let Some(Value::Result4(s2)) = out.env.get("s2") else {
        panic!("expected stat after remove");
    };
    assert_eq!(s2.kind, ResultKind::Ok);
    match &s2.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("exists"), Some(&Value::Bool(false)));
        }
        _ => panic!("expected stat payload map"),
    }
}

#[test]
fn std_fs_write_text_deferred_when_exceeds_max_write_bytes() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("deferred_cap");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./out/**");
    let _max_write_guard = EnvVarGuard::set("OCP_STD_FS_MAX_WRITE_BYTES", "4");

    let src = r#"
observe("std.fs.write_text", "tier2", ctx("path=./out/large.txt;text=abcdef"), budget(5)) -> w;
"#;
    let out = run_program(src).expect("exec should pass");
    let Some(Value::Result4(w)) = out.env.get("w") else {
        panic!("expected write result");
    };
    assert_eq!(w.kind, ResultKind::Deferred);
    assert_eq!(w.reason, Some(ReasonCode::LimitExceeded));
}

#[test]
fn std_fs_write_text_denied_when_outside_allowlist() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("deny_allowlist");
    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_write_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_WRITE", "./allowed/**");

    let src = r#"
observe("std.fs.write_text", "tier2", ctx("path=./blocked/file.txt;text=x"), budget(5)) -> w;
"#;
    let out = run_program(src).expect("exec should pass");
    let Some(Value::Result4(w)) = out.env.get("w") else {
        panic!("expected write result");
    };
    assert_eq!(w.kind, ResultKind::Insufficient);
    assert_eq!(w.reason, Some(ReasonCode::FsPermissionDenied));
}

#[test]
fn std_fs_rename_commit_denied_when_destination_exists_and_overwrite_false() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("rename_overwrite_policy");
    let src_file = root.join("work/src.txt");
    let dst_file = root.join("work/dst.txt");
    write_text(&src_file, "src");
    write_text(&dst_file, "dst");

    let _root_guard = EnvVarGuard::set("OCP_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_rename_guard = EnvVarGuard::set("OCP_STD_FS_ALLOW_RENAME", "./work/**");

    let src = r#"
observe("std.fs.rename", "tier2", ctx("from=./work/src.txt;to=./work/dst.txt;overwrite=false"), budget(5)) -> rn;
commit(rn);
"#;

    let err = run_program(src).expect_err("commit must be denied");
    assert_eq!(err.code.as_str(), "X-COMMIT-FORBIDDEN");
    assert_eq!(err.root_reason, Some(ReasonCode::PolicyDenied));
}
