use ocp::ocp::{
    build_closeout_report, replay_pilot_sequence, run_pilot_sequence, ExecConfig, PilotTurnStatus,
};

#[test]
fn pilot_sequence_collects_turn_results() {
    let turns = [
        r#"observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r; commit(r);"#,
        r#"observe("world.degraded", "tier2", ctx("scene=lab"), budget(5)) -> r; commit(r);"#,
        r#"condition(false);"#,
    ];

    let summary = run_pilot_sequence(
        &turns,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    );
    assert_eq!(summary.turns.len(), 3);
    assert!(summary.ok_count >= 2);
    assert!(summary.error_count >= 1);
    assert!(!summary.aggregate_signature.is_empty());
}

#[test]
fn replay_is_deterministic_for_same_sequence() {
    let turns = [
        r#"observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r; commit(r);"#,
        r#"observe("world.unknown", "tier2", ctx("scene=lab"), budget(5)) -> r;"#,
    ];

    let replay = replay_pilot_sequence(
        &turns,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    );
    assert!(replay.deterministic);
    assert_eq!(
        replay.first.aggregate_signature,
        replay.second.aggregate_signature
    );
}

#[test]
fn closeout_report_contains_sections() {
    let turns = [
        r#"observe("world.ok", "tier2", ctx("scene=lab"), budget(5)) -> r; commit(r);"#,
        r#"condition(false);"#,
    ];
    let summary = run_pilot_sequence(
        &turns,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    );
    let replay = replay_pilot_sequence(
        &turns,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    );
    let report = build_closeout_report(&summary, &replay);

    assert!(report.contains("# OCP v0.1 Pilot Closeout"));
    assert!(report.contains("## Summary"));
    assert!(report.contains("## Turn Details"));
    assert!(report.contains("## Replay Check"));
}

#[test]
fn pilot_turn_status_has_error_for_failing_turn() {
    let turns = [r#"condition(false);"#];
    let summary = run_pilot_sequence(
        &turns,
        ExecConfig {
            step_cap: 500,
            ..ExecConfig::default()
        },
    );
    assert_eq!(summary.turns.len(), 1);
    assert_eq!(summary.turns[0].status, PilotTurnStatus::Error);
    assert!(summary.turns[0].error_code.is_some());
}
