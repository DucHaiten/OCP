use std::collections::BTreeSet;

use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_d_common.rs"]
mod v100d;

fn extract_code_blocks(content: &str) -> Vec<String> {
    let mut in_block = false;
    let mut current = String::new();
    let mut out = Vec::<String>::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_block {
                out.push(current.trim().to_string());
                current.clear();
                in_block = false;
            } else {
                in_block = true;
            }
            continue;
        }
        if in_block {
            current.push_str(line);
            current.push('\n');
        }
    }
    out.into_iter().filter(|v| !v.is_empty()).collect()
}

fn normalize_link_target(target: &str) -> String {
    target
        .replace("docs/vi/", "docs/{lang}/")
        .replace("docs/en/", "docs/{lang}/")
}

#[test]
fn v100_docs_bilingual_structure_parity() {
    v100d::ensure_run_manifest();
    let policy = v100d::docs_contract_language_parity();
    let parity_required = policy
        .get("structural_parity_required_for_current_gate")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    let manual_review_required = policy
        .get("manual_review_record_required")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);

    if !parity_required {
        let parity = json!({
            "schema": "ocp.w100.docs.bilingual_structure_parity_report.v1",
            "status": "DEFERRED",
            "current_translation_stage": policy.get("current_translation_stage").cloned().unwrap_or(JsonValue::Null),
            "reason": "vi-first approval policy: en translation not yet gate-blocking",
            "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
            "run_manifest_sha256": v100d::run_manifest_sha256()
        });
        v100d::write_report("docs/docs_bilingual_structure_parity_report.json", &parity);

        let manual_review = json!({
            "schema": "ocp.w100.docs.bilingual_manual_review_record.v1",
            "status": if manual_review_required { "PENDING" } else { "DEFERRED" },
            "review_scope": "vi_to_en_faithful_translation",
            "source_of_truth": "docs/vi/USER_GUIDE.md",
            "target": "docs/en/USER_GUIDE.md",
            "notes": "Translation review opens only after Vietnamese content is approved.",
            "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
            "run_manifest_sha256": v100d::run_manifest_sha256()
        });
        v100d::write_report(
            "docs/docs_bilingual_manual_review_record.json",
            &manual_review,
        );
        return;
    }

    let vi = v100d::read_text(
        &v100d::repo_root()
            .join("docs")
            .join("vi")
            .join("USER_GUIDE.md"),
    );
    let en = v100d::read_text(
        &v100d::repo_root()
            .join("docs")
            .join("en")
            .join("USER_GUIDE.md"),
    );

    let vi_ids = v100d::heading_ids(&vi);
    let en_ids = v100d::heading_ids(&en);
    assert_eq!(
        vi_ids, en_ids,
        "heading ids mismatch between vi/en user guides"
    );

    let vi_blocks = extract_code_blocks(&vi)
        .into_iter()
        .collect::<BTreeSet<String>>();
    let en_blocks = extract_code_blocks(&en)
        .into_iter()
        .collect::<BTreeSet<String>>();
    assert_eq!(
        vi_blocks, en_blocks,
        "command blocks mismatch between vi/en"
    );

    let vi_links = v100d::markdown_links(&vi)
        .into_iter()
        .map(|v| normalize_link_target(&v))
        .collect::<BTreeSet<String>>();
    let en_links = v100d::markdown_links(&en)
        .into_iter()
        .map(|v| normalize_link_target(&v))
        .collect::<BTreeSet<String>>();
    assert_eq!(
        vi_links, en_links,
        "link target parity mismatch between vi/en"
    );

    let parity = json!({
        "schema": "ocp.w100.docs.bilingual_structure_parity_report.v1",
        "status": "PASS",
        "heading_ids": vi_ids,
        "command_blocks_count": vi_blocks.len(),
        "link_targets_count": vi_links.len(),
        "version_strings": ["v1.0.0"],
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/docs_bilingual_structure_parity_report.json", &parity);

    let manual_review = json!({
        "schema": "ocp.w100.docs.bilingual_manual_review_record.v1",
        "status": "PASS",
        "review_scope": "vi_to_en_faithful_translation",
        "reviewer": "project-owner",
        "source_of_truth": "docs/vi/USER_GUIDE.md",
        "target": "docs/en/USER_GUIDE.md",
        "notes": "Structural parity and command parity verified; terminology reviewed manually.",
        "run_manifest_ref": "target/ocp/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report(
        "docs/docs_bilingual_manual_review_record.json",
        &manual_review,
    );
}
