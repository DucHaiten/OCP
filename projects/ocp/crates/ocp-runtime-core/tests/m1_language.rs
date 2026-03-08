use ocp_runtime_core::{check_source, run_source, ResultKind, TraceEvent};

#[test]
fn w1_stdnet_poll_forbidden_in_user_ocp() {
    let src = r#"
observe("std.net.poll", "tier2", ctx("conn=local"), budget(1)) -> ev;
"#;
    let err = check_source(src, 1).expect_err("std.net.poll must be forbidden in user path");
    assert!(
        err.message.contains("std.net.poll"),
        "unexpected message: {}",
        err.message
    );
}

#[test]
fn w1_stdnet_event_payload_line_only_contract() {
    let src = r#"
observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19091"), budget(5)) -> listen_res;
match listen_res {
  OK => { let ok = true; condition(ok); }
  DEGRADED => { let ok = true; condition(ok); }
  INSUFFICIENT => { let ok = true; condition(ok); }
  DEFERRED => { let ok = true; condition(ok); }
}
"#;
    let out = run_source(src, 1, 4096).expect("run source");
    assert!(out.steps > 0);

    let mut saw_listen_ok = false;
    for ev in out.trace.events {
        if let TraceEvent::ObserveEnd { key, kind, .. } = ev {
            if key == "std.net.listen" && kind == ResultKind::Ok {
                saw_listen_ok = true;
                break;
            }
        }
    }
    assert!(saw_listen_ok, "missing ObserveEnd OK for std.net.listen");
}
