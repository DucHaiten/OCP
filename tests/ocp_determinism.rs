use ocp::ocp::{execute_program, parse_program, typecheck_program, ExecConfig};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[test]
fn determinism_signature_same_input_same_signature() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let p = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&p).expect("typecheck should pass");

    let out1 = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    let out2 = execute_program(
        &p,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    assert_eq!(out1.signature, out2.signature);
    assert_eq!(out1.trace.events, out2.trace.events);
}

#[test]
fn determinism_signature_changes_when_program_changes() {
    let src_a = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let src_b = r#"
observe("world.degraded", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { commit(r); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let p1 = parse_program(src_a, 1).expect("parse should pass");
    typecheck_program(&p1).expect("typecheck should pass");
    let p2 = parse_program(src_b, 1).expect("parse should pass");
    typecheck_program(&p2).expect("typecheck should pass");

    let out1 = execute_program(
        &p1,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");
    let out2 = execute_program(
        &p2,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    )
    .expect("exec should pass");

    assert_ne!(out1.signature, out2.signature);
}

#[test]
fn v16_determinism_soak_signature_and_trace_stable() {
    let src = r#"
observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r;
match r {
  OK => { commit(r); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(true); }
}
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    typecheck_program(&program).expect("typecheck should pass");

    let soak_runs = 20usize;
    let mut signatures = Vec::<String>::with_capacity(soak_runs);
    let mut first_trace = None;
    for _ in 0..soak_runs {
        let output = execute_program(
            &program,
            ExecConfig {
                step_cap: 500,
                ..ExecConfig::default()
            },
        )
        .expect("exec should pass");
        if let Some(trace_snapshot) = &first_trace {
            assert_eq!(
                trace_snapshot, &output.trace.events,
                "trace events must stay stable in soak"
            );
        } else {
            first_trace = Some(output.trace.events.clone());
        }
        signatures.push(output.signature);
    }

    let first_signature = signatures
        .first()
        .cloned()
        .expect("at least one signature in soak");
    assert!(
        signatures.iter().all(|sig| sig == &first_signature),
        "all soak signatures must match first signature"
    );

    let out_dir = PathBuf::from("target")
        .join("ocp")
        .join("w16")
        .join("determinism");
    fs::create_dir_all(&out_dir).expect("create w16 determinism output dir");
    let report = json!({
        "schema": "ocp.w16.determinism.soak.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "soak_runs": soak_runs,
        "signature_stable": true,
        "trace_stable": true,
        "first_signature": first_signature,
        "unique_signature_count": 1,
    });
    fs::write(
        out_dir.join("determinism_soak_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize soak report"),
    )
    .expect("write determinism_soak_report.json");
}
