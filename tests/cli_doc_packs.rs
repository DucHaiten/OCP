use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_ocp_cli(args: &[&str]) -> Output {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(repo_root())
        .arg("run")
        .arg("-p")
        .arg("ocp-cli")
        .arg("--quiet")
        .arg("--")
        .args(args);
    cmd.output().expect("run ocp-cli")
}

#[test]
fn cli_doc_packs_text_contains_required_sections() {
    let out = run_ocp_cli(&["doc", "packs"]);
    assert!(
        out.status.success(),
        "doc packs failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("key: std.fs.read_text"),
        "expected std.fs.read_text in text output"
    );
    assert!(
        text.contains("permission_class: permissions.std_fs"),
        "expected std_fs permission class in text output"
    );
    assert!(text.contains("ctx_schema:"), "missing ctx_schema section");
    assert!(
        text.contains("payload_schema:"),
        "missing payload_schema section"
    );
    assert!(text.contains("example:"), "missing example section");
}

#[test]
fn cli_doc_packs_json_contains_known_pack_fields() {
    let out = run_ocp_cli(&["doc", "packs", "--json"]);
    assert!(
        out.status.success(),
        "doc packs --json failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with("{\"packs\":["),
        "json output must start with packs array"
    );
    assert!(
        text.contains("\"key\":\"std.net.http.request\""),
        "expected std.net.http.request in json output"
    );
    assert!(
        text.contains("\"permission_class\":\"permissions.std_net_http\""),
        "expected std_net_http permission class in json output"
    );
    assert!(
        text.contains("\"key_kind\":\"observe_only\""),
        "expected key_kind field in json output"
    );
}
