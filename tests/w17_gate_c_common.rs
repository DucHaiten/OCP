#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use ocp_ocl::ocp_ocl::{
    canonical_round_robin_order, canonical_task_event_order, canonicalize_env_entries,
    canonicalize_iteration_paths, canonicalize_text_boundary, enforce_locale_timezone,
    evaluate_entropy_policy, parse_supported_platform_profile, resolve_task_lifecycle,
    supported_platform_profile_ids, EntropySource, LogicalTaskEvent, TaskLifecycleDecision,
};
use serde_json::json;

pub fn write_gate_c_report() {
    let win = parse_supported_platform_profile("win-x64-ntfs").expect("parse win profile");
    let linux = parse_supported_platform_profile("linux-x64-ext4").expect("parse linux profile");

    let scheduler_events = vec![
        LogicalTaskEvent {
            logical_time: 3,
            task_id: "task-b".to_string(),
            seq: 2,
        },
        LogicalTaskEvent {
            logical_time: 1,
            task_id: "task-a".to_string(),
            seq: 3,
        },
        LogicalTaskEvent {
            logical_time: 1,
            task_id: "task-a".to_string(),
            seq: 1,
        },
        LogicalTaskEvent {
            logical_time: 1,
            task_id: "task-b".to_string(),
            seq: 1,
        },
    ];
    let scheduler_sorted = canonical_task_event_order(&scheduler_events);
    let scheduler_digest = scheduler_sorted
        .iter()
        .map(|event| format!("{}|{}|{}", event.logical_time, event.task_id, event.seq))
        .collect::<Vec<String>>()
        .join("\n");

    let unicode_input = b"\xEF\xBB\xBFCafe\xCC\x81\r\nline\rnext";
    let unicode_output = canonicalize_text_boundary(unicode_input).expect("canonical text");

    let listing = vec![
        ".\\Data\\A.txt".to_string(),
        "./data/../Data/B.txt".to_string(),
        "./Data/A.txt".to_string(),
    ];
    let paths_win = canonicalize_iteration_paths(&listing, win).expect("canonical win paths");
    let paths_linux = canonicalize_iteration_paths(&listing, linux).expect("canonical linux paths");

    let env_entries = canonicalize_env_entries(&[
        ("Z_KEY".to_string(), "2".to_string()),
        ("A_KEY".to_string(), "1".to_string()),
        ("A_KEY".to_string(), "0".to_string()),
    ]);

    let rr = canonical_round_robin_order(&["z".to_string(), "a".to_string(), "b".to_string()], 1);
    let lifecycle = resolve_task_lifecycle(7, Some(5), false);
    let entropy_locked = evaluate_entropy_policy("locked_v071", EntropySource::HostRandom, false);
    let entropy_governed =
        evaluate_entropy_policy("locked_v071", EntropySource::GovernedSeededRng, true);
    let locale_ok = enforce_locale_timezone("C.UTF-8", "UTC").is_ok();

    let report = json!({
        "schema": "ocl.w17.determinism.core.v1",
        "run_manifest_ref": "target/ocl/w17/meta/run_manifest.json",
        "supported_profiles": supported_platform_profile_ids(),
        "scheduler": {
            "ordering_digest": scheduler_digest,
            "round_robin_round1": rr,
            "timeout_decision": format!("{:?}", lifecycle),
            "timeout_expected": format!("{:?}", TaskLifecycleDecision::TimedOut)
        },
        "platform": {
            "win_paths": paths_win,
            "linux_paths": paths_linux,
            "locale_timezone_pinned": locale_ok
        },
        "unicode": {
            "canonical_output": unicode_output
        },
        "iteration": {
            "env_entries_sorted": env_entries
        },
        "entropy": {
            "locked_host_random": {
                "allowed": entropy_locked.allowed,
                "requires_audit_marker": entropy_locked.requires_audit_marker,
                "reason_code": entropy_locked.reason_code
            },
            "locked_governed_rng": {
                "allowed": entropy_governed.allowed,
                "requires_audit_marker": entropy_governed.requires_audit_marker,
                "reason_code": entropy_governed.reason_code
            }
        }
    });

    let out_dir = PathBuf::from("target")
        .join("ocl")
        .join("w17")
        .join("determinism");
    fs::create_dir_all(&out_dir).expect("create deterministic output dir");
    let rendered = serde_json::to_string_pretty(&report).expect("render report");
    fs::write(out_dir.join("determinism_core_report.json"), rendered)
        .expect("write determinism_core_report.json");
}
