use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocp_sdk::{
    build_ocppkg_with_lock, init_project, sync_deps_lock_v1, sync_deps_lock_v2,
    verify_supply_artifact,
};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocp_v10_pkg_hash_{tag}_{stamp}"))
}

fn write_ocp_manifest(root: &Path, package_name: &str) {
    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = [\"std.net.poll\"]\n"
        ),
        package_name
    );
    fs::write(root.join("Ocp.toml"), manifest).expect("write Ocp.toml");
}

fn write_package_ocpp(root: &Path, package_name: &str, version: &str, entry: &str) {
    let body = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"{}\"\n",
            "entry = \"{}\"\n\n",
            "[exports]\n",
            "modules = [\"demo.main\"]\n\n",
            "[requested_permissions]\n",
            "std_fs.read = [\"./data/**\"]\n"
        ),
        package_name, version, entry
    );
    fs::write(root.join("package.ocpp"), body).expect("write package.ocpp");
}

fn prepare_project_with_main_bytes(
    root: &Path,
    ocp_package_name: &str,
    package_ocpp_name: &str,
    package_ocpp_version: &str,
    entry: &str,
    main_bytes: &[u8],
) {
    init_project(root).expect("init");
    write_ocp_manifest(root, ocp_package_name);
    write_package_ocpp(root, package_ocpp_name, package_ocpp_version, entry);

    let entry_path = root.join(entry);
    if let Some(parent) = entry_path.parent() {
        fs::create_dir_all(parent).expect("create entry parent");
    }
    fs::write(entry_path, main_bytes).expect("write entry");

    sync_deps_lock_v1(root).expect("sync deps lock v1");
    sync_deps_lock_v2(root).expect("sync deps lock v2");
}

#[test]
fn v10_package_ocpp_parser_overrides_legacy_manifest_identity() {
    let root = temp_project_dir("parser_override");
    prepare_project_with_main_bytes(
        &root,
        "legacy_name",
        "pkg_parser_v10",
        "1.2.3",
        "src/app/main.ocp",
        b"let ready = true;\ncondition(ready);\n",
    );

    let build = build_ocppkg_with_lock(&root, false).expect("build");
    let file_name = build
        .artifact_path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    assert_eq!(file_name, "pkg_parser_v10-1.2.3.ocppkg");
}

#[test]
fn v10_content_hash_text_is_lf_normalized() {
    let root_lf = temp_project_dir("text_lf");
    prepare_project_with_main_bytes(
        &root_lf,
        "legacy_text",
        "pkg_text_norm",
        "0.1.0",
        "src/main.ocp",
        b"let a = true;\ncondition(a);\n",
    );
    let build_lf = build_ocppkg_with_lock(&root_lf, false).expect("build lf");

    let root_crlf = temp_project_dir("text_crlf");
    prepare_project_with_main_bytes(
        &root_crlf,
        "legacy_text",
        "pkg_text_norm",
        "0.1.0",
        "src/main.ocp",
        b"let a = true;\r\ncondition(a);\r\n",
    );
    let build_crlf = build_ocppkg_with_lock(&root_crlf, false).expect("build crlf");

    assert_eq!(
        build_lf.content_hash_sha256, build_crlf.content_hash_sha256,
        "text LF normalization must keep canonical content hash stable"
    );
    assert_ne!(
        build_lf.payload_hash_blake3, build_crlf.payload_hash_blake3,
        "raw payload hash may differ because payload stores original bytes"
    );
}

#[test]
fn v10_content_hash_binary_uses_raw_bytes_without_newline_normalization() {
    let root_a = temp_project_dir("bin_a");
    prepare_project_with_main_bytes(
        &root_a,
        "legacy_bin",
        "pkg_bin_raw",
        "0.1.0",
        "src/main.ocp",
        b"let a = true;\ncondition(a);\n",
    );
    fs::create_dir_all(root_a.join("assets")).expect("create assets");
    fs::write(
        root_a.join("assets").join("blob.bin"),
        [0u8, b'A', b'\r', b'\n', b'B'],
    )
    .expect("write bin a");
    let build_a = build_ocppkg_with_lock(&root_a, false).expect("build a");

    let root_b = temp_project_dir("bin_b");
    prepare_project_with_main_bytes(
        &root_b,
        "legacy_bin",
        "pkg_bin_raw",
        "0.1.0",
        "src/main.ocp",
        b"let a = true;\ncondition(a);\n",
    );
    fs::create_dir_all(root_b.join("assets")).expect("create assets");
    fs::write(
        root_b.join("assets").join("blob.bin"),
        [0u8, b'A', b'\n', b'B'],
    )
    .expect("write bin b");
    let build_b = build_ocppkg_with_lock(&root_b, false).expect("build b");

    assert_ne!(
        build_a.content_hash_sha256, build_b.content_hash_sha256,
        "binary bytes must be hashed raw; CRLF vs LF in binary changes hash"
    );
}

#[test]
fn v10_verify_supply_fails_when_content_hash_is_tampered() {
    let root = temp_project_dir("tamper_content_hash");
    prepare_project_with_main_bytes(
        &root,
        "legacy_tamper",
        "pkg_tamper",
        "0.1.0",
        "src/main.ocp",
        b"let ok = true;\ncondition(ok);\n",
    );
    let build = build_ocppkg_with_lock(&root, false).expect("build");

    let raw = fs::read_to_string(&build.artifact_path).expect("read artifact");
    let mut lines: Vec<String> = raw.lines().map(|v| v.to_string()).collect();
    for line in &mut lines {
        if line.starts_with("content_hash_sha256=") {
            *line = "content_hash_sha256=0000000000000000000000000000000000000000000000000000000000000000"
                .to_string();
        }
    }
    let tampered = lines.join("\n") + "\n";
    fs::write(&build.artifact_path, tampered).expect("write tampered artifact");

    let err = verify_supply_artifact(&build.artifact_path).expect_err("must fail");
    assert!(
        err.to_string().contains("content hash mismatch"),
        "unexpected error: {err}"
    );
}
