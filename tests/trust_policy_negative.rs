use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{check_project_with_lock, init_project, resolve_deps_v3, sync_deps_lock_v1};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_v16_trust_negative_{tag}_{stamp}"))
}

fn prepare_project_with_registry_dep(root: &Path, lane: &str) {
    init_project(root).expect("init");
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"trust_negative\"\n",
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
    .expect("write package.oclp");
    fs::write(
        root.join("deps")
            .join("widgets")
            .join("src")
            .join("widget.ocl"),
        "let ok = true;\ncondition(ok);\n",
    )
    .expect("write dep source");
    sync_deps_lock_v1(root).expect("sync deps.lock");
    resolve_deps_v3(root, false).expect("resolve deps.lock.v3");
}

#[test]
fn trust_policy_negative_locked_v071_untrusted_dep_must_fail() {
    let root = temp_project_dir("strict_untrusted");
    prepare_project_with_registry_dep(&root, "locked_v071");

    let err = check_project_with_lock(&root, true).expect_err("strict lane must reject untrusted");
    assert!(
        err.to_string().contains("V-DEPS-TRUST-REQUIRED"),
        "unexpected error: {err}"
    );
}
