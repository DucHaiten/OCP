use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_cli_fs_legacy_{tag}_{stamp}"))
}

fn run_ocp_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo_bin)
        .current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("run ocp-cli")
}

fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn legacy_std_fs_read_list_keys_respect_permissions_std_fs() {
    let root = temp_project_dir("read_list");
    let root_s = root.to_string_lossy().to_string();

    let init = run_ocp_cli(&["init", &root_s, "--template", "mini-game"]);
    assert_success(&init);

    let main = concat!(
        "module compat.fs_legacy.main;\n",
        "observe(\"std.fs.list\", \"tier2\", ctx(\"path=.\"), budget(5)) -> ls;\n",
        "match ls {\n",
        "  OK => { let ok = true; condition(ok); }\n",
        "  DEGRADED => { let ok = true; condition(ok); }\n",
        "  INSUFFICIENT => { let ok = true; condition(ok); }\n",
        "  DEFERRED => { let ok = true; condition(ok); }\n",
        "}\n",
        "observe(\"std.fs.read\", \"tier2\", ctx(\"path=Ocp.toml\"), budget(5)) -> rd;\n",
        "match rd {\n",
        "  OK => { let ok = true; condition(ok); }\n",
        "  DEGRADED => { let ok = true; condition(ok); }\n",
        "  INSUFFICIENT => { let ok = true; condition(ok); }\n",
        "  DEFERRED => { let ok = true; condition(ok); }\n",
        "}\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocp"), main).expect("write main.ocp");

    let mut manifest = fs::read_to_string(root.join("Ocp.toml")).expect("read Ocp.toml");
    manifest.push_str("\n[permissions.package]\n");
    manifest.push_str("allow = [\"std.fs.list\", \"std.fs.read\"]\n");
    manifest.push_str("deny = []\n");
    manifest.push_str("\n[permissions.std_fs]\n");
    manifest.push_str("read = [\"./**\"]\n");
    manifest.push_str("list = [\"./**\"]\n");
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");

    let check = run_ocp_cli(&["check", &root_s]);
    assert_success(&check);

    let run = run_ocp_cli(&["run", &root_s]);
    assert_success(&run);
}
