use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_f_common.rs"]
mod v100f;

#[test]
fn v100_legal_consistency() {
    v100f::ensure_run_manifest();

    let licensing_model = v100f::licensing_model();
    let community_policy = v100f::community_policy();

    assert_eq!(
        licensing_model
            .get("oss_license_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only"
    );
    assert_eq!(
        licensing_model
            .get("model")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "dual_license"
    );
    assert_eq!(
        community_policy
            .get("outbound")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only+commercial"
    );

    let license = v100f::read_text(&v100f::repo_root().join("LICENSE"));
    assert!(
        license.contains("GNU AFFERO GENERAL PUBLIC LICENSE")
            && license.contains("Version 3, 19 November 2007"),
        "LICENSE must contain AGPL-3.0-only text"
    );

    let notice = v100f::read_text(&v100f::repo_root().join("NOTICE"));
    assert!(
        notice.contains("OCP") && notice.contains("AGPL-3.0-only"),
        "NOTICE must identify project name and OSS side"
    );

    let authors = v100f::read_text(&v100f::repo_root().join("AUTHORS"));
    assert!(
        authors.contains("DucHaiten") && authors.contains("project owner"),
        "AUTHORS must identify current project owner"
    );

    let vi_licensing = v100f::read_text(&v100f::repo_root().join("docs/vi/legal/licensing.md"));
    let en_licensing = v100f::read_text(&v100f::repo_root().join("docs/en/legal/licensing.md"));
    let vi_commercial = v100f::read_text(&v100f::repo_root().join("docs/vi/legal/commercial.md"));
    let en_commercial = v100f::read_text(&v100f::repo_root().join("docs/en/legal/commercial.md"));
    let cargo_manifests = [
        "Cargo.toml",
        "projects/ocp/crates/ocp-runtime-core/Cargo.toml",
        "projects/ocp/crates/ocp-sdk/Cargo.toml",
        "projects/ocp/crates/ocp-cli/Cargo.toml",
        "projects/ocp/crates/ocp-lsp/Cargo.toml",
        "projects/ocp/crates/ocp-dap/Cargo.toml",
    ];

    for (label, content) in [
        ("vi licensing", &vi_licensing),
        ("en licensing", &en_licensing),
    ] {
        assert!(
            content.contains("AGPL-3.0-only") && content.contains("dual-license"),
            "{label} must explain AGPL-3.0-only and dual-license"
        );
    }

    assert!(
        vi_commercial.contains("không tự phát sinh")
            && vi_commercial.contains("thỏa thuận thương mại riêng"),
        "vi commercial doc must state separate agreement and non-automatic commercial path"
    );
    assert!(
        en_commercial.contains("separate commercial agreement")
            && (en_commercial.contains("does not automatically apply")
                || en_commercial.contains("does not arise automatically")),
        "en commercial doc must state separate agreement and non-automatic commercial path"
    );

    for rel in cargo_manifests {
        let manifest = v100f::read_text(&v100f::repo_root().join(rel));
        assert!(
            manifest.contains("authors = [\"DucHaiten\"]")
                && manifest.contains("license = \"AGPL-3.0-only\""),
            "manifest `{rel}` must pin authors/license metadata"
        );
    }

    let package_json = v100f::read_json(
        &v100f::repo_root()
            .join("editor")
            .join("vscode")
            .join("ocp")
            .join("package.json"),
    );
    assert_eq!(
        package_json
            .get("author")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "DucHaiten"
    );
    assert_eq!(
        package_json
            .get("license")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "AGPL-3.0-only"
    );

    let report = json!({
        "schema": "ocp.w100.legal.legal_consistency_report.v1",
        "status": "PASS",
        "oss_license_id": "AGPL-3.0-only",
        "licensing_model": "dual_license",
        "community_outbound": "AGPL-3.0-only+commercial",
        "root_files_verified": ["LICENSE", "NOTICE", "AUTHORS"],
        "metadata_verified": {
            "cargo_manifests": [
                "Cargo.toml",
                "projects/ocp/crates/ocp-runtime-core/Cargo.toml",
                "projects/ocp/crates/ocp-sdk/Cargo.toml",
                "projects/ocp/crates/ocp-cli/Cargo.toml",
                "projects/ocp/crates/ocp-lsp/Cargo.toml",
                "projects/ocp/crates/ocp-dap/Cargo.toml"
            ],
            "editor_package_json": "editor/vscode/ocp/package.json"
        },
        "docs_verified": [
            "docs/vi/legal/licensing.md",
            "docs/en/legal/licensing.md",
            "docs/vi/legal/commercial.md",
            "docs/en/legal/commercial.md"
        ],
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100f::run_manifest_sha256()
    });
    v100f::write_report("legal/legal_consistency_report.json", &report);
}
