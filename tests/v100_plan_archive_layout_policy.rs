use serde_json::{json, Value as JsonValue};

#[path = "v100_gate_d_common.rs"]
mod v100d;

#[test]
fn v100_plan_archive_layout_policy() {
    v100d::ensure_run_manifest();
    let policy = v100d::docs_contract_link_redirect_policy();
    assert_eq!(
        policy
            .get("contract_id")
            .and_then(JsonValue::as_str)
            .unwrap_or(""),
        "v1.docs_link_redirect_policy"
    );

    let root = v100d::repo_root();
    let allow_root_plan = policy
        .get("allow_root_plan")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    assert!(
        !allow_root_plan.is_empty(),
        "allow_root_plan must not be empty"
    );

    let mut root_plans = Vec::<String>::new();
    for entry in std::fs::read_dir(&root).expect("read root") {
        let entry = entry.expect("root entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("OCP-OCL-MVP-PLAN-v") && name.ends_with(".md") {
            root_plans.push(name);
        }
    }
    root_plans.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    assert_eq!(
        root_plans, allow_root_plan,
        "root plan files must match allow_root_plan contract exactly"
    );

    let history_root = root.join(
        policy
            .get("historical_plans_root")
            .and_then(JsonValue::as_str)
            .unwrap_or("docs/plans/history/"),
    );
    assert!(history_root.exists(), "missing {}", history_root.display());

    let mut history_plans = Vec::<String>::new();
    for entry in std::fs::read_dir(&history_root).expect("read history root") {
        let entry = entry.expect("history entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("OCP-OCL-MVP-PLAN-v0.") && name.ends_with(".md") {
            history_plans.push(name);
        }
    }
    history_plans.sort_by(|lhs, rhs| lhs.as_bytes().cmp(rhs.as_bytes()));
    assert!(
        history_plans.len() >= 20,
        "historical plans archive is too small: {}",
        history_plans.len()
    );

    let index_path = root.join(
        policy
            .get("plans_index")
            .and_then(JsonValue::as_str)
            .unwrap_or("docs/plans/INDEX.md"),
    );
    let index_text = v100d::read_text(&index_path);
    for plan in &history_plans {
        let rel = format!("docs/plans/history/{plan}");
        assert!(
            index_text.contains(&rel),
            "INDEX missing history plan reference `{rel}`"
        );
    }

    let report = json!({
        "schema": "ocl.w100.docs.plan_archive_layout_report.v1",
        "status": "PASS",
        "root_plans": root_plans,
        "history_plan_count": history_plans.len(),
        "plans_index": index_path.to_string_lossy().replace('\\', "/"),
        "run_manifest_ref": "target/ocl/w100/meta/run_manifest.json",
        "run_manifest_sha256": v100d::run_manifest_sha256()
    });
    v100d::write_report("docs/plan_archive_layout_report.json", &report);
}
