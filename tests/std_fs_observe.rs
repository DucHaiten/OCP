use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn temp_fs_root(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_std_fs_{tag}_{stamp}"))
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dir");
    }
    std::fs::write(path, content).expect("write file");
}

fn run_program(src: &str) -> ocp_ocl::ocp_ocl::ExecOutput {
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");
    Executor::new(ExecConfig {
        step_cap: 500,
        ..ExecConfig::default()
    })
    .run(&program)
    .expect("exec should pass")
}

#[test]
fn std_fs_read_text_ok_and_stat_not_found_returns_ok_exists_false() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("read_stat");
    write_text(&root.join("data/in.txt"), "hello");

    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCL_STD_FS_ALLOW_READ", "./data/**");

    let src = r#"
observe("std.fs.read_text", "tier2", ctx("path=./data/in.txt"), budget(5)) -> r;
observe("std.fs.stat", "tier2", ctx("path=./data/missing.txt"), budget(5)) -> s;
"#;
    let out = run_program(src);

    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected read_text result");
    };
    assert_eq!(r.kind, ResultKind::Ok);
    match &r.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("text"), Some(&Value::String("hello".to_string())));
            assert_eq!(map.get("truncated"), Some(&Value::Bool(false)));
            assert_eq!(map.get("bytes"), Some(&Value::Int(5)));
        }
        _ => panic!("expected read_text map payload"),
    }

    let Some(Value::Result4(s)) = out.env.get("s") else {
        panic!("expected stat result");
    };
    assert_eq!(s.kind, ResultKind::Ok);
    match &s.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("exists"), Some(&Value::Bool(false)));
            assert_eq!(map.get("is_dir"), Some(&Value::Bool(false)));
            assert_eq!(map.get("size"), Some(&Value::Int(0)));
        }
        _ => panic!("expected stat map payload"),
    }
}

#[test]
fn std_fs_read_text_degraded_when_truncated() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("truncate");
    write_text(&root.join("data/in.txt"), "abcdef");

    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCL_STD_FS_ALLOW_READ", "./data/**");
    let _max_guard = EnvVarGuard::set("OCL_STD_FS_MAX_READ_BYTES", "4");

    let src = r#"
observe("std.fs.read_text", "tier2", ctx("path=./data/in.txt"), budget(5)) -> r;
"#;
    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected read_text result");
    };
    assert_eq!(r.kind, ResultKind::Degraded);
    assert_eq!(r.reason, Some(ReasonCode::FsTooLarge));
    match &r.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("text"), Some(&Value::String("abcd".to_string())));
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
            assert_eq!(map.get("bytes"), Some(&Value::Int(4)));
        }
        _ => panic!("expected read_text payload"),
    }
}

#[test]
fn std_fs_list_dir_sorted_and_truncated() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("list");
    write_text(&root.join("data/b.txt"), "b");
    write_text(&root.join("data/a.txt"), "a");
    write_text(&root.join("data/c.txt"), "c");

    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_list_guard = EnvVarGuard::set("OCL_STD_FS_ALLOW_LIST", "./data/**");

    let src = r#"
observe("std.fs.list_dir", "tier2", ctx("path=./data;cap=2"), budget(5)) -> l;
"#;
    let out = run_program(src);
    let Some(Value::Result4(l)) = out.env.get("l") else {
        panic!("expected list_dir result");
    };
    assert_eq!(l.kind, ResultKind::Degraded);
    assert_eq!(l.reason, Some(ReasonCode::LimitExceeded));
    match &l.payload {
        Some(Value::Map(map)) => {
            assert_eq!(map.get("truncated"), Some(&Value::Bool(true)));
            let Some(Value::List(entries)) = map.get("entries") else {
                panic!("entries must be list");
            };
            let name0 = match &entries[0] {
                Value::Map(row) => row.get("name"),
                _ => None,
            };
            let name1 = match &entries[1] {
                Value::Map(row) => row.get("name"),
                _ => None,
            };
            assert_eq!(name0, Some(&Value::String("a.txt".to_string())));
            assert_eq!(name1, Some(&Value::String("b.txt".to_string())));
        }
        _ => panic!("expected list_dir map payload"),
    }
}

#[test]
fn std_fs_read_text_denies_path_escape() {
    let _guard = env_serial_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_fs_root("escape");
    std::fs::create_dir_all(&root).expect("create root");

    let _root_guard = EnvVarGuard::set("OCL_STD_FS_ROOT", &root.to_string_lossy());
    let _allow_read_guard = EnvVarGuard::set("OCL_STD_FS_ALLOW_READ", "./data/**");

    let src = r#"
observe("std.fs.read_text", "tier2", ctx("path=../secret.txt"), budget(5)) -> r;
"#;
    let out = run_program(src);
    let Some(Value::Result4(r)) = out.env.get("r") else {
        panic!("expected result");
    };
    assert_eq!(r.kind, ResultKind::Insufficient);
    assert_eq!(r.reason, Some(ReasonCode::FsPathOutsideSandbox));
}
