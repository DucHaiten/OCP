#[path = "w17_gate_f_common.rs"]
mod w17;

use ocp_sdk::check_project_with_lock;

#[test]
fn v17_connector_db_denies_without_std_db_policy() {
    let root = w17::temp_project_dir("db_deny_missing_policy");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_db_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n"
        ),
        "observe(\"std.db.query_int\", \"tier2\", ctx(\"dsn=main;sql=select 1\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("missing std_db must fail");
    let msg = err.to_string();
    assert!(msg.contains("RC-DB-DSN-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_db_allows_query_when_mode_granted() {
    let root = w17::temp_project_dir("db_allow_query");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_db_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_db]\n",
            "enabled = true\n",
            "allow_dsn = [\"main\"]\n",
            "allow_modes = [\"read_query\"]\n",
            "max_rows = 100\n",
            "max_bytes = 2048\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.db.query_int\", \"tier2\", ctx(\"dsn=main;sql=select 1\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    check_project_with_lock(&root, false).expect("std.db.query_int should pass with policy");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_db_denies_exec_when_mode_not_granted() {
    let root = w17::temp_project_dir("db_deny_exec_mode");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_db_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_db]\n",
            "enabled = true\n",
            "allow_dsn = [\"main\"]\n",
            "allow_modes = [\"read_query\"]\n",
            "max_rows = 100\n",
            "max_bytes = 2048\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.db.exec\", \"tier2\", ctx(\"dsn=main;sql=update t set v=1\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("exec mode must be denied");
    let msg = err.to_string();
    assert!(msg.contains("RC-DB-MODE-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}
