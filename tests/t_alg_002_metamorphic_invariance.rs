use ocp_runtime_core::{run_source_with_engine, RunEngine};

fn run_signature(src: &str, file_id: u32) -> String {
    run_source_with_engine(src, file_id, 10_000, RunEngine::Dual)
        .unwrap_or_else(|err| panic!("dual run failed: {err:?}"))
        .signature
}

#[test]
fn t_alg_002_whitespace_and_lineendings_do_not_change_signature() {
    let base = "let ready = true;\ncondition(ready);\n";
    let crlf = "let ready = true;\r\ncondition(ready);\r\n";
    let extra_blank = "\n\nlet ready = true;\n\ncondition(ready);\n\n";
    let spaces = "  let ready = true;   \ncondition(ready);    \n";

    let s_base = run_signature(base, 101);
    let s_crlf = run_signature(crlf, 102);
    let s_blank = run_signature(extra_blank, 103);
    let s_spaces = run_signature(spaces, 104);

    assert_eq!(s_base, s_crlf, "CRLF variant changed signature");
    assert_eq!(s_base, s_blank, "extra blank lines changed signature");
    assert_eq!(s_base, s_spaces, "trailing/leading spaces changed signature");
}

#[test]
fn t_alg_002_format_variants_on_loop_program_keep_signature() {
    let base = r#"
let xs = [1, 2, 3];
for item in xs cap 2 { let seen = item; }
condition(true);
"#;
    let variant = r#"
let xs = [1, 2, 3];
for item in xs cap 2 {
  let seen = item;
}
condition(true);
"#;

    let s_base = run_signature(base, 201);
    let s_variant = run_signature(variant, 202);
    assert_eq!(
        s_base, s_variant,
        "format-only variant changed runtime signature"
    );
}
