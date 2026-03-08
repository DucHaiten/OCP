use ocp_runtime_core::{check_source, normalize_text, run_source};

#[test]
fn m0_runtime_core_check_and_run_pass() {
    let source = r#"
let x = 1;
condition(true);
"#;
    check_source(source, 1).expect("check should pass");
    let out = run_source(source, 1, 128).expect("run should pass");
    assert!(out.steps > 0);
}

#[test]
fn m0_runtime_core_format_normalizes_newlines() {
    let source = "let x = 1;\r\ncondition(true);\r";
    let formatted = normalize_text(source);
    assert_eq!(formatted, "let x = 1;\ncondition(true);\n");
}

#[test]
fn m1_runtime_core_module_fn_for_pass() {
    let source = r#"
module demo.main;
import demo.shared;
struct User { name, age };
enum Mode { Idle, Busy };
fn twice(v) {
  for i in 0..2 {
    condition(true);
  }
  return v;
}
let x = twice(1);
condition(true);
"#;
    check_source(source, 2).expect("m1 check should pass");
    run_source(source, 2, 4096).expect("m1 run should pass");
}

#[test]
fn m1_runtime_core_for_end_unbounded_fail() {
    let source = r#"
let limit = 10;
for i in 0..limit {
  condition(true);
}
"#;
    let err = check_source(source, 3).expect_err("typecheck should fail");
    assert_eq!(err.code.as_str(), "T-TYPE-MISMATCH");
}
