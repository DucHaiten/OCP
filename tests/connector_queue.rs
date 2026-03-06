#[path = "w17_gate_f_common.rs"]
mod w17;

use ocl_sdk::check_project_with_lock;

#[test]
fn v17_connector_queue_publish_consume_allowed_with_policy() {
    let root = w17::temp_project_dir("queue_allowed");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_queue_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_queue]\n",
            "enabled = true\n",
            "allow_topics = [\"orders\"]\n",
            "max_inflight = 64\n",
            "max_payload_bytes = 4096\n",
            "ack_required = true\n"
        ),
        concat!(
            "observe(\"std.queue.bus.publish\", \"tier2\", ctx(\"topic=orders;payload=ok\"), budget(10)) -> pub_r;\n",
            "observe(\"std.queue.bus.consume\", \"tier2\", ctx(\"topic=orders\"), budget(10)) -> con_r;\n",
            "let ok = true;\n",
            "condition(ok);\n"
        ),
    );

    check_project_with_lock(&root, false).expect("queue publish/consume should pass with policy");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_queue_denied_when_policy_missing() {
    let root = w17::temp_project_dir("queue_policy_missing");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_queue_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n"
        ),
        "observe(\"std.queue.bus.publish\", \"tier2\", ctx(\"topic=orders;payload=ok\"), budget(10)) -> pub_r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("missing std_queue must fail");
    let msg = err.to_string();
    assert!(msg.contains("RC-QUEUE-TOPIC-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_queue_denied_when_allow_topics_empty() {
    let root = w17::temp_project_dir("queue_topics_empty");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_queue_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_queue]\n",
            "enabled = true\n",
            "allow_topics = []\n",
            "max_inflight = 64\n",
            "max_payload_bytes = 4096\n",
            "ack_required = true\n"
        ),
        "observe(\"std.queue.bus.consume\", \"tier2\", ctx(\"topic=orders\"), budget(10)) -> con_r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("empty allow_topics must fail");
    let msg = err.to_string();
    assert!(msg.contains("RC-QUEUE-TOPIC-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}
