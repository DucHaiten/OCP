use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    check_project_with_lock, collect_deps_resolved_trace_events_v10, init_project, resolve_deps_v3,
    sync_deps_lock_v1,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v10_trust_lane_{tag}_{stamp}"))
}

fn write_manifest_with_registry_dep(root: &Path, lane: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"trust_lane_demo\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"{}\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n",
            "widgets = {{ name=\"engine-ui-widgets\", version=\"0.3.*\", source=\"registry\" }}\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = [\"std.net.poll\"]\n"
        ),
        lane
    );
    fs::write(root.join("Ocl.toml"), manifest).expect("write Ocl.toml");
}

fn prepare_project_with_non_builtin_dep(root: &Path, lane: &str) -> PathBuf {
    init_project(root).expect("init");
    write_manifest_with_registry_dep(root, lane);
    fs::create_dir_all(root.join("deps").join("widgets").join("src")).expect("create dep src");
    fs::write(
        root.join("deps").join("widgets").join("package.oclp"),
        concat!(
            "[package]\n",
            "name = \"engine-ui-widgets\"\n",
            "version = \"0.3.0\"\n",
            "entry = \"src/widget.ocl\"\n\n",
            "[exports]\n",
            "modules = [\"widget\"]\n"
        ),
    )
    .expect("write dep package.oclp");
    fs::write(
        root.join("deps")
            .join("widgets")
            .join("src")
            .join("widget.ocl"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write dep src");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    let summary = resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
    summary.lock_v3_path
}

fn read_non_builtin_signer(lock_path: &Path) -> String {
    let raw = fs::read_to_string(lock_path).expect("read deps.lock.v3");
    for line in raw.lines() {
        let Some(payload) = line.strip_prefix("dep=") else {
            continue;
        };
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() == 11 && parts[3] != "builtin" {
            return parts[7].to_string();
        }
    }
    panic!("non-builtin signer not found in deps.lock.v3");
}

#[test]
fn v10_locked_lane_rejects_untrusted_non_builtin_dependency() {
    let root = temp_project_dir("reject_untrusted");
    prepare_project_with_non_builtin_dep(&root, "locked_v071");

    let err = check_project_with_lock(&root, true).expect_err("must reject untrusted signer");
    assert!(
        err.to_string().contains("V-DEPS-TRUST-REQUIRED"),
        "unexpected error: {err}"
    );
}

#[test]
fn v10_locked_lane_accepts_trusted_signer_and_emits_deps_resolved_trace() {
    let root = temp_project_dir("accept_trusted");
    let lock_path = prepare_project_with_non_builtin_dep(&root, "locked_v071");
    let signer_pub = read_non_builtin_signer(&lock_path);

    let trust = format!(
        concat!("[trusted_signers.registry]\n", "keys = [\"{}\"]\n"),
        signer_pub
    );
    fs::write(root.join("trust.toml"), trust).expect("write trust.toml");

    check_project_with_lock(&root, true).expect("trusted signer should pass");

    let trace = collect_deps_resolved_trace_events_v10(&root).expect("collect deps trace events");
    assert!(
        trace
            .iter()
            .any(|ev| ev.event == "deps_resolved" && ev.reason.as_deref() == Some("signed-trusted")),
        "missing deps_resolved trace with signed-trusted decision"
    );
}

#[test]
fn v10_locked_v06_allows_untrusted_signer_for_compat() {
    let root = temp_project_dir("compat_allows_untrusted");
    prepare_project_with_non_builtin_dep(&root, "locked_v06");

    check_project_with_lock(&root, true).expect("locked_v06 should allow untrusted signer");
}
