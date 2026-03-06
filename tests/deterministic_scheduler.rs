#[path = "w17_gate_c_common.rs"]
mod w17;

use ocp_ocl::ocp_ocl::{
    canonical_round_robin_order, canonical_task_event_order, resolve_task_lifecycle,
    LogicalTaskEvent, TaskLifecycleDecision,
};

#[test]
fn deterministic_scheduler_orders_events_canonically() {
    let events = vec![
        LogicalTaskEvent {
            logical_time: 5,
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
    let canonical = canonical_task_event_order(&events);
    let mut reversed = events.clone();
    reversed.reverse();
    let canonical_reversed = canonical_task_event_order(&reversed);
    assert_eq!(
        canonical, canonical_reversed,
        "ordering must be input-stable"
    );

    let fingerprint = canonical
        .iter()
        .map(|event| format!("{}|{}|{}", event.logical_time, event.task_id, event.seq))
        .collect::<Vec<String>>();
    assert_eq!(
        fingerprint,
        vec![
            "1|task-a|1".to_string(),
            "1|task-a|3".to_string(),
            "1|task-b|1".to_string(),
            "5|task-b|2".to_string(),
        ]
    );

    w17::write_gate_c_report();
}

#[test]
fn deterministic_scheduler_timeout_cancel_is_logical_tick_based() {
    assert_eq!(
        resolve_task_lifecycle(7, Some(5), false),
        TaskLifecycleDecision::TimedOut
    );
    assert_eq!(
        resolve_task_lifecycle(4, Some(5), false),
        TaskLifecycleDecision::Ready
    );
    assert_eq!(
        resolve_task_lifecycle(4, Some(5), true),
        TaskLifecycleDecision::Cancelled
    );
    w17::write_gate_c_report();
}

#[test]
fn deterministic_scheduler_round_robin_is_canonical() {
    let round0 =
        canonical_round_robin_order(&["z".to_string(), "a".to_string(), "b".to_string()], 0);
    let round1 =
        canonical_round_robin_order(&["z".to_string(), "a".to_string(), "b".to_string()], 1);
    let round2 =
        canonical_round_robin_order(&["z".to_string(), "a".to_string(), "b".to_string()], 2);
    assert_eq!(
        round0,
        vec!["a".to_string(), "b".to_string(), "z".to_string()]
    );
    assert_eq!(
        round1,
        vec!["b".to_string(), "z".to_string(), "a".to_string()]
    );
    assert_eq!(
        round2,
        vec!["z".to_string(), "a".to_string(), "b".to_string()]
    );
    w17::write_gate_c_report();
}
