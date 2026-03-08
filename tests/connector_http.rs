#[path = "w17_gate_f_common.rs"]
mod w17;

use ocp_sdk::check_project_with_lock;

#[test]
fn v17_connector_http_get_allowed_when_method_granted() {
    let root = w17::temp_project_dir("http_get_allowed");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_http_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_net_http]\n",
            "enabled = true\n",
            "allow_hosts = [\"api.example.com\"]\n",
            "allow_methods = [\"GET\"]\n",
            "max_body_bytes = 4096\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.http.client.get\", \"tier2\", ctx(\"url=https://api.example.com/ping\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    check_project_with_lock(&root, false).expect("std.http.client.get should pass");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_http_post_denied_when_method_missing() {
    let root = w17::temp_project_dir("http_post_denied");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_http_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n\n",
            "[permissions.std_net_http]\n",
            "enabled = true\n",
            "allow_hosts = [\"api.example.com\"]\n",
            "allow_methods = [\"GET\"]\n",
            "max_body_bytes = 4096\n",
            "timeout_ms = 1000\n"
        ),
        "observe(\"std.http.client.post\", \"tier2\", ctx(\"url=https://api.example.com/ping;body=ok\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("POST should be denied");
    let msg = err.to_string();
    assert!(msg.contains("RC-NET-METHOD-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}

#[test]
fn v17_connector_http_denied_when_policy_missing() {
    let root = w17::temp_project_dir("http_policy_missing");
    w17::init_connector_project(
        &root,
        concat!(
            "[package]\n",
            "name = \"connector_http_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n"
        ),
        "observe(\"std.http.client.get\", \"tier2\", ctx(\"url=https://api.example.com/ping\"), budget(10)) -> r;\nlet ok = true;\ncondition(ok);\n",
    );

    let err = check_project_with_lock(&root, false).expect_err("missing std_net_http must fail");
    let msg = err.to_string();
    assert!(msg.contains("RC-NET-HOST-DENIED"), "actual={msg}");
    w17::write_gate_f_report();
}
