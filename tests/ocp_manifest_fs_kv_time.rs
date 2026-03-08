use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{init_project, run_project_with_lock, SdkError};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_manifest_v72_{tag}_{stamp}"))
}

fn write_manifest(root: &Path, body: &str) {
    fs::write(root.join("Ocp.toml"), body).expect("write manifest");
}

fn write_main(root: &Path, body: &str) {
    fs::write(root.join("src").join("main.ocp"), body).expect("write main");
}

fn expect_permission_denied(err: SdkError) -> String {
    let SdkError::PermissionDenied(msg) = err else {
        panic!("expected PermissionDenied, got {err}");
    };
    msg
}

#[test]
fn manifest_fs_missing_block_denies_with_rc_and_hint() {
    let root = temp_project_dir("fs_missing");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "fs_missing"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false)
            .expect_err("std.fs must be denied without std_fs block"),
    );
    assert!(msg.contains("RC-FS-PERMISSION-DENIED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_fs]"), "actual={msg}");
    assert!(msg.contains("read = ["), "actual={msg}");
}

#[test]
fn manifest_fs_present_allows_read_key() {
    let root = temp_project_dir("fs_allowed");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "fs_allowed"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]

[permissions.std_fs]
read = ["./data/**"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("std.fs read should be allowed");
    assert!(summary.steps >= 1);
}

#[test]
fn manifest_package_deny_overrides_std_fs_allow() {
    let root = temp_project_dir("deny_override");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "deny_override"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.fs.*"]
deny = ["std.fs.read_text"]

[permissions.std_fs]
read = ["./data/**"]
"#,
    );
    write_main(
        &root,
        r#"observe("std.fs.read_text", "tier2", ctx("path=./data/sample.txt"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("deny must override std_fs allow"),
    );
    assert!(
        msg.contains("matched deny rule=`std.fs.read_text`"),
        "actual={msg}"
    );
    assert!(msg.contains("[permissions.package]"), "actual={msg}");
}

#[test]
fn manifest_kv_requires_enabled_true() {
    let root = temp_project_dir("kv_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "kv_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.kv.*"]

[permissions.std_kv]
enabled = false
max_keys = 128
"#,
    );
    write_main(
        &root,
        r#"observe("std.kv.get", "tier2", ctx("key=app.flag"), budget(5)) -> r;
"#,
    );

    let msg =
        expect_permission_denied(run_project_with_lock(&root, false).expect_err("kv disabled"));
    assert!(msg.contains("RC-KV-PERMISSION-DENIED"), "actual={msg}");
    assert!(msg.contains("enabled = true"), "actual={msg}");
}

#[test]
fn manifest_time_requires_enabled_true() {
    let root = temp_project_dir("time_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "time_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.time.*"]

[permissions.std_time]
enabled = false
tick_mode = "logical"
dt_ms = 16
"#,
    );
    write_main(
        &root,
        r#"observe("std.time.tick_info", "tier2", ctx("scope=clock"), budget(5)) -> r;
"#,
    );

    let msg =
        expect_permission_denied(run_project_with_lock(&root, false).expect_err("time disabled"));
    assert!(msg.contains("RC-TIME-DISABLED"), "actual={msg}");
    assert!(msg.contains("enabled = true"), "actual={msg}");
}

#[test]
fn manifest_game_requires_enabled_true() {
    let root = temp_project_dir("game_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "game_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.game.*"]

[permissions.std_game]
enabled = false
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 8
state_delta_max_bytes = 128
"#,
    );
    write_main(
        &root,
        r#"observe("std.game.tick_info", "tier2", ctx("tick=1"), budget(5)) -> g;
"#,
    );

    let msg =
        expect_permission_denied(run_project_with_lock(&root, false).expect_err("game disabled"));
    assert!(msg.contains("RC-GAME-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_game]"), "actual={msg}");
}

#[test]
fn manifest_game_present_allows_tick_info_and_rng() {
    let root = temp_project_dir("game_enabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "game_enabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.game.*"]

[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 16
state_delta_max_bytes = 1024
"#,
    );
    write_main(
        &root,
        r#"observe("std.game.tick_info", "tier2", ctx("tick=3"), budget(5)) -> tick;
observe("std.game.rng", "tier2", ctx("stream=main;count=3;tick=3"), budget(5)) -> rng;
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("std.game should be allowed");
    assert!(summary.steps >= 2);
}

#[test]
fn manifest_shadow_requires_enabled_true() {
    let root = temp_project_dir("shadow_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "shadow_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.shadow.*"]

[permissions.std_shadow]
enabled = false
max_branches = 8
branch_step_cap = 5000
branch_budget_cap = 200000
max_diff_keys = 2000
max_report_bytes = 262144
"#,
    );
    write_main(
        &root,
        r#"observe("std.shadow.run", "tier2", ctx("variants_json=[1,2]"), budget(5)) -> s;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("std.shadow disabled must be denied"),
    );
    assert!(msg.contains("RC-SHADOW-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_shadow]"), "actual={msg}");
}

#[test]
fn manifest_shadow_present_allows_run_and_compare() {
    let root = temp_project_dir("shadow_enabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "shadow_enabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.shadow.*"]

[permissions.std_shadow]
enabled = true
max_branches = 8
branch_step_cap = 5000
branch_budget_cap = 200000
max_diff_keys = 2000
max_report_bytes = 262144
"#,
    );
    write_main(
        &root,
        r#"observe("std.shadow.run", "tier2", ctx("variants_json=[1,2]"), budget(5)) -> run;
observe("std.shadow.compare", "tier2", ctx("branches_json=[{\"id\":0,\"outcome\":\"OK\",\"signature\":\"a\",\"cost\":{\"steps\":1,\"budget\":2},\"state_summary\":{\"x\":1}},{\"id\":1,\"outcome\":\"OK\",\"signature\":\"b\",\"cost\":{\"steps\":2,\"budget\":3},\"state_summary\":{\"x\":2}}]"), budget(5)) -> cmp;
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("std.shadow should be allowed");
    assert!(summary.steps >= 2);
}

#[test]
fn manifest_ui_requires_enabled_true() {
    let root = temp_project_dir("ui_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "ui_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.ui.*"]

[permissions.std_ui]
enabled = false
max_draw_cmds = 5000
max_input_events = 500
"#,
    );
    write_main(
        &root,
        r#"observe("std.ui.frame_info", "tier2", ctx("ctx_tick=1"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("std.ui disabled must be denied"),
    );
    assert!(msg.contains("RC-UI-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_ui]"), "actual={msg}");
}

#[test]
fn manifest_ui_present_allows_frame_info_and_input() {
    let root = temp_project_dir("ui_enabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "ui_enabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["std.ui.*"]

[permissions.std_ui]
enabled = true
max_draw_cmds = 200
max_input_events = 50
assets_read = ["./assets/**"]
max_asset_bytes = 2048
"#,
    );
    write_main(
        &root,
        r#"observe("std.ui.frame_info", "tier2", ctx("ctx_tick=3"), budget(5)) -> frame;
observe("std.ui.input", "tier2", ctx("cap=2;events=key:W|quit"), budget(5)) -> input;
"#,
    );

    let summary = run_project_with_lock(&root, false).expect("std.ui should be allowed");
    assert!(summary.steps >= 2);
}

#[test]
fn manifest_engine_ui_requires_std_ui_enabled() {
    let root = temp_project_dir("engine_ui_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "engine_ui_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["engine.ui.*"]

[permissions.std_ui]
enabled = false
max_draw_cmds = 200
max_input_events = 50
"#,
    );
    write_main(
        &root,
        r#"observe("engine.ui.run", "tier2", ctx("entry_module=app.main;tick=1"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("engine.ui should require std_ui"),
    );
    assert!(msg.contains("RC-UI-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_ui]"), "actual={msg}");
}

#[test]
fn manifest_engine_game_requires_std_game_enabled() {
    let root = temp_project_dir("engine_game_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "engine_game_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["engine.game.*"]

[permissions.std_game]
enabled = false
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 8
state_delta_max_bytes = 128
"#,
    );
    write_main(
        &root,
        r#"observe("engine.game.run", "tier2", ctx("entry_module=game.main;tick=1;stream=main;count=1"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("engine.game should require std_game"),
    );
    assert!(msg.contains("RC-GAME-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_game]"), "actual={msg}");
}

#[test]
fn manifest_engine_shadow_requires_std_shadow_enabled() {
    let root = temp_project_dir("engine_shadow_disabled");
    init_project(&root).expect("init");

    write_manifest(
        &root,
        r#"[package]
name = "engine_shadow_disabled"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["engine.shadow.*"]

[permissions.std_shadow]
enabled = false
max_branches = 8
branch_step_cap = 5000
branch_budget_cap = 200000
max_diff_keys = 2000
max_report_bytes = 262144
"#,
    );
    write_main(
        &root,
        r#"observe("engine.shadow.preview", "tier2", ctx("entry_module=game.main;variants_json=[1,2]"), budget(5)) -> r;
"#,
    );

    let msg = expect_permission_denied(
        run_project_with_lock(&root, false).expect_err("engine.shadow should require std_shadow"),
    );
    assert!(msg.contains("RC-SHADOW-DISABLED"), "actual={msg}");
    assert!(msg.contains("[permissions.std_shadow]"), "actual={msg}");
}
