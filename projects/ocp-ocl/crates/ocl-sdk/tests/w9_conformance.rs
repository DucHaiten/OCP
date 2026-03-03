use std::path::PathBuf;

use ocl_runtime_core::RunEngine;
use ocl_sdk::{
    default_conformance_manifest_path_with_selector, parse_conformance_manifest_v1,
    run_conformance_v1, ConformanceRunOptionsV1, ReactorRuntimeMode,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("..")
}

#[test]
fn w9_manifest_has_10_scenarios_in_order() {
    let root = repo_root();
    let manifest_path = root.join("projects/ocp-ocl/conformance/conformance.v1.toml");
    let manifest = parse_conformance_manifest_v1(&manifest_path).expect("parse conformance");

    assert_eq!(manifest.schema, "ocl.conformance.v1");
    assert_eq!(manifest.scenarios.len(), 10);
    let names: Vec<String> = manifest.scenarios.iter().map(|s| s.name.clone()).collect();
    assert_eq!(
        names,
        vec![
            "hello-cli".to_string(),
            "web-fetch".to_string(),
            "mini-server".to_string(),
            "scheduler".to_string(),
            "composer-demo".to_string(),
            "plugin-demo".to_string(),
            "tls-client".to_string(),
            "sqlite-app".to_string(),
            "tcp-jsonl-server-real".to_string(),
            "ui-demo".to_string()
        ]
    );
}

#[test]
fn w9_conformance_runner_locked_dual_pass() {
    let root = repo_root();
    let manifest_path = root.join("projects/ocp-ocl/conformance/conformance.v1.toml");
    let manifest = parse_conformance_manifest_v1(&manifest_path).expect("parse conformance");

    let report = run_conformance_v1(
        &root,
        &manifest,
        ConformanceRunOptionsV1 {
            locked: true,
            engine: RunEngine::Dual,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            universe_id: None,
        },
    );

    assert_eq!(report.schema, "ocl.conformance.v1");
    assert_eq!(report.scenarios_total, 10);
    assert_eq!(report.scenarios_failed, 0);
    assert_eq!(report.scenarios_passed, 10);
    assert!(!report.required_digest.is_empty());
}

#[test]
fn w9_default_manifest_selector_supports_v1_v5() {
    let root = repo_root();
    let v1 = default_conformance_manifest_path_with_selector(&root, Some("v1"));
    let v5 = default_conformance_manifest_path_with_selector(&root, Some("v5"));
    assert!(v1.ends_with("projects/ocp-ocl/conformance/conformance.v1.toml"));
    assert!(v5.ends_with("projects/ocp-ocl/conformance/conformance.v5.toml"));
}

#[test]
fn w9_v5_manifest_expected_fail_contract_passes() {
    let root = repo_root();
    let manifest_path = root.join("projects/ocp-ocl/conformance/conformance.v5.toml");
    let manifest = parse_conformance_manifest_v1(&manifest_path).expect("parse conformance v5");

    assert_eq!(manifest.schema, "ocl.conformance.manifest.v5");
    assert_eq!(manifest.scenarios.len(), 5);

    let report = run_conformance_v1(
        &root,
        &manifest,
        ConformanceRunOptionsV1 {
            locked: true,
            engine: RunEngine::Dual,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            universe_id: None,
        },
    );
    assert_eq!(report.schema, "ocl.conformance.manifest.v5");
    assert_eq!(report.scenarios_failed, 0);
    assert_eq!(report.scenarios_passed, report.scenarios_total);
}
