use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{read_trace_jsonl, write_trace_jsonl, TraceEventV1};

fn temp_trace_file(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("ocl_v10_dep_provenance_{tag}_{stamp}.jsonl"))
}

#[test]
fn v10_trace_roundtrip_preserves_callsite_package_id() {
    let path = temp_trace_file("roundtrip");
    let expected = TraceEventV1 {
        seq: 1,
        run_id: "run_1".to_string(),
        event: "observe_end".to_string(),
        key: Some("std.fs.read_text".to_string()),
        callsite_package_id: Some("project:demo".to_string()),
        kind: Some("ok".to_string()),
        reason: None,
        origin_id: Some(7),
        allowed: Some(true),
        value: None,
        steps: Some(12),
        universe_id: "u-main".to_string(),
        domain_id: "d-main".to_string(),
        payload_hash: "abcd1234".to_string(),
    };

    write_trace_jsonl(&path, std::slice::from_ref(&expected)).expect("write trace");
    let read_back = read_trace_jsonl(&path).expect("read trace");
    assert_eq!(read_back, vec![expected]);
}

#[test]
fn v10_trace_decode_keeps_legacy_rows_compatible() {
    let path = temp_trace_file("legacy");
    let line_11 = "1|run_legacy|observe_end|std.fs.read_text|ok|-|9|1|0|5|p11";
    let line_13 = "2|run_legacy|observe_end|std.fs.read_text|ok|-|9|1|0|6|u13|d13|p13";
    fs::write(&path, format!("{line_11}\n{line_13}\n")).expect("write legacy rows");

    let rows = read_trace_jsonl(&path).expect("decode legacy rows");
    assert_eq!(rows.len(), 2);

    assert_eq!(rows[0].callsite_package_id, None);
    assert_eq!(rows[0].universe_id, "__legacy__");
    assert_eq!(rows[0].domain_id, "default");
    assert_eq!(rows[0].payload_hash, "p11");

    assert_eq!(rows[1].callsite_package_id, None);
    assert_eq!(rows[1].universe_id, "u13");
    assert_eq!(rows[1].domain_id, "d13");
    assert_eq!(rows[1].payload_hash, "p13");
}
