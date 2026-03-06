use std::fs;

use serde_json::json;

#[path = "v19_gate_g_common.rs"]
mod v19g;

fn quantile_index(sample_count: usize) -> usize {
    ((0.95f64 * sample_count as f64).ceil() as usize).saturating_sub(1)
}

#[test]
fn v19_editor_perf_budget_protocol() {
    v19g::ensure_run_manifest();

    let protocol = v19g::read_json("contracts/editor/editor_perf_protocol.v1.json");
    let dataset_id = protocol
        .get("dataset_id")
        .and_then(|v| v.as_str())
        .expect("dataset_id");
    let dataset_path = protocol
        .get("dataset_path")
        .and_then(|v| v.as_str())
        .expect("dataset_path");
    let sample_count = protocol
        .get("p95_protocol")
        .and_then(|v| v.get("sample_count"))
        .and_then(|v| v.as_u64())
        .expect("sample_count") as usize;
    let sort_order = protocol
        .get("p95_protocol")
        .and_then(|v| v.get("sort_order"))
        .and_then(|v| v.as_str())
        .expect("sort_order");
    let quantile_rule = protocol
        .get("p95_protocol")
        .and_then(|v| v.get("quantile_index_rule"))
        .and_then(|v| v.as_str())
        .expect("quantile_index_rule");

    assert_eq!(sort_order, "ascending");
    assert_eq!(quantile_rule, "ceil(0.95*n)-1");
    assert_eq!(sample_count, 50usize);

    let counters = protocol
        .get("counters")
        .and_then(|v| v.as_array())
        .expect("counters");
    let expected_counters = [
        "lsp_keystroke_work_units",
        "workspace_initial_index_work_units",
        "lsp_cache_bytes",
        "cancelled_requests_work_units",
        "cancelled_requests_poll_iterations",
    ];
    for expected in expected_counters {
        let found = counters.iter().any(|item| item.as_str() == Some(expected));
        assert!(found, "missing protocol counter {expected}");
    }

    let dataset = v19g::repo_root().join(dataset_path);
    assert!(dataset.exists(), "missing dataset {}", dataset.display());
    let sample_files = fs::read_dir(&dataset)
        .unwrap_or_else(|_| panic!("read dir {}", dataset.display()))
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some("ocl"))
        .count();
    assert!(
        sample_files >= 2,
        "dataset requires at least two .ocl fixtures"
    );

    let report = json!({
        "schema": "ocl.w19.perf.editor_perf_protocol_report.v1",
        "status": "PASS",
        "dataset_id": dataset_id,
        "dataset_path": dataset_path,
        "sample_count": sample_count,
        "quantile_index": quantile_index(sample_count),
        "quantile_index_rule": quantile_rule,
        "counters": counters,
        "fixture_count": sample_files,
        "run_manifest_ref": "target/ocl/w19/meta/run_manifest.json",
        "run_manifest_sha256": v19g::run_manifest_sha256()
    });
    v19g::write_report("perf/editor_perf_protocol_report.json", &report);
}
