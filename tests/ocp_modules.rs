use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp::ocp::{run_fixture_file, ErrorCode, ExecConfig, FixtureRunnerError, Value};

fn unique_temp_dir(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("ocp_modules_{name}_{now}"));
    p
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent directory");
    }
    fs::write(path, content).expect("write file");
}

#[test]
fn modules_import_local_file_passes() {
    let root = unique_temp_dir("pass");
    let entry = root.join("main.ocp");
    let mod_file = root.join("util").join("math.ocp");

    write_text(
        &entry,
        r#"
import util.math;
let ok = always_true();
condition(ok);
"#,
    );
    write_text(
        &mod_file,
        r#"
module util.math;
fn always_true() {
  return true;
}
"#,
    );

    let out = run_fixture_file(
        &entry,
        901,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("module import should run");
    assert_eq!(out.env.get("ok"), Some(&Value::Bool(true)));
}

#[test]
fn modules_missing_import_returns_t_import_not_found_with_hint() {
    let root = unique_temp_dir("missing");
    let entry = root.join("main.ocp");
    write_text(
        &entry,
        r#"
import util.missing;
condition(true);
"#,
    );

    let err = run_fixture_file(
        &entry,
        902,
        ExecConfig {
            step_cap: 200,
            ..ExecConfig::default()
        },
    )
    .expect_err("missing import should fail");

    match err {
        FixtureRunnerError::Diag(diag) => {
            assert_eq!(diag.code, ErrorCode::TImportNotFound);
            assert!(diag.hint.is_some());
        }
        other => panic!("expected diag error, got {other:?}"),
    }
}

#[test]
fn modules_import_cycle_returns_t_import_cycle_with_hint() {
    let root = unique_temp_dir("cycle");
    let entry = root.join("main.ocp");
    let a = root.join("a.ocp");
    let b = root.join("b.ocp");

    write_text(
        &entry,
        r#"
import a;
let ok = ping();
condition(ok);
"#,
    );
    write_text(
        &a,
        r#"
import b;
fn ping() {
  return pong();
}
"#,
    );
    write_text(
        &b,
        r#"
import a;
fn pong() {
  return true;
}
"#,
    );

    let err = run_fixture_file(
        &entry,
        903,
        ExecConfig {
            step_cap: 300,
            ..ExecConfig::default()
        },
    )
    .expect_err("cycle should fail");

    match err {
        FixtureRunnerError::Diag(diag) => {
            assert_eq!(diag.code, ErrorCode::TImportCycle);
            assert!(diag.hint.is_some());
        }
        other => panic!("expected diag error, got {other:?}"),
    }
}
