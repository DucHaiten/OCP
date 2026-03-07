use std::path::{Path, PathBuf};

use serde_json::json;

#[path = "v100_gate_d_common.rs"]
mod v100d;

fn is_external_link(link: &str) -> bool {
    let lower = link.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with('#')
}

fn resolve_link(source: &Path, link: &str) -> PathBuf {
    if link.starts_with('/') {
        return v100d::repo_root().join(link.trim_start_matches('/'));
    }
    let link_no_anchor = link.split('#').next().unwrap_or("").trim();
    source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(link_no_anchor)
}

#[test]
fn v100_docs_link_redirect_policy() {
    v100d::ensure_run_manifest();
    let policy = v100d::docs_contract_link_redirect_policy();

    let docs_root = v100d::repo_root().join(
        policy
            .get("docs_root")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("docs/"),
    );
    assert!(docs_root.exists(), "docs root missing");

    let plans_index = v100d::repo_root().join(
        policy
            .get("plans_index")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("docs/plans/INDEX.md"),
    );
    assert!(plans_index.exists(), "plans index missing");

    let historical = v100d::repo_root().join(
        policy
            .get("historical_plans_root")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("docs/plans/history/"),
    );
    assert!(historical.exists(), "historical plans root missing");

    let mut checked_links = Vec::<String>::new();
    let mut broken_links = Vec::<String>::new();
    let mut files = v100d::list_markdown_files(&docs_root);
    files.push(v100d::repo_root().join("README.md"));
    for file in files {
        let content = v100d::read_text(&file);
        for link in v100d::markdown_links(&content) {
            if is_external_link(&link) {
                continue;
            }
            let target_raw = link.split('#').next().unwrap_or("").trim();
            if target_raw.is_empty() {
                continue;
            }
            let resolved = resolve_link(&file, target_raw);
            let key = format!(
                "{} -> {}",
                file.to_string_lossy().replace('\\', "/"),
                target_raw
            );
            checked_links.push(key.clone());
            if !resolved.exists() {
                broken_links.push(key);
            }
        }
    }

    assert!(
        broken_links.is_empty(),
        "broken markdown links found: {}",
        broken_links.join(", ")
    );

    let report = json!({
        "schema": "ocl.w100.docs.link_redirect_policy_report.v1",
        "status": "PASS",
        "links_checked": checked_links.len(),
        "broken_links": broken_links,
        "plans_index": plans_index.to_string_lossy().replace('\\', "/"),
        "historical_plans_root": historical.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_link_redirect_policy_report.json", &report);
}
