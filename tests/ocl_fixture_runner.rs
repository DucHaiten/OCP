use std::path::PathBuf;

use ocp_ocl::ocp_ocl::{run_fixture_file, run_fixture_source, ExecConfig, FixtureRunnerError};

#[test]
fn fixture_runner_runs_from_source() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
commit(r);
"#;
    let out =
        run_fixture_source(src, 7, ExecConfig { step_cap: 200 }).expect("fixture should pass");
    assert!(!out.signature.is_empty());
    assert_eq!(out.commits.len(), 1);
}

#[test]
fn fixture_runner_runs_from_file() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/v1/exec_ok/determinism_basic.ocl");
    let out =
        run_fixture_file(&p, 9, ExecConfig { step_cap: 200 }).expect("fixture file should pass");
    assert!(!out.signature.is_empty());
}

#[test]
fn fixture_runner_io_error_is_reported() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/v1/exec_ok/does_not_exist.ocl");
    let err = run_fixture_file(&p, 9, ExecConfig { step_cap: 200 }).expect_err("should fail");
    match err {
        FixtureRunnerError::Io(msg) => assert!(msg.contains("failed to read fixture")),
        FixtureRunnerError::Diag(_) => panic!("expected IO error"),
    }
}
