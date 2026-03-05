use std::path::Path;

use ocl_sdk::{
    parse_conformance_manifest_v1, run_conformance_v1, ConformanceManifestV1,
    ConformanceRunOptionsV1,
};

fn conformance_v5_path() -> &'static Path {
    Path::new("projects/ocp-ocl/conformance/conformance.v5.toml")
}

fn core_manifest_subset() -> ConformanceManifestV1 {
    let parsed =
        parse_conformance_manifest_v1(conformance_v5_path()).expect("parse conformance v5");
    let scenarios = parsed
        .scenarios
        .into_iter()
        .filter(|s| s.name.starts_with("core-"))
        .collect::<Vec<_>>();
    assert!(
        !scenarios.is_empty(),
        "core scenarios must exist in conformance.v5"
    );
    ConformanceManifestV1 {
        schema: parsed.schema,
        scenarios,
    }
}

#[test]
fn conformance_v5_includes_core_scenarios() {
    let manifest = core_manifest_subset();
    let names = manifest
        .scenarios
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"core-exec-pass"));
    assert!(names.contains(&"core-type-fail"));
    assert!(names.contains(&"core-parse-fail"));
    assert!(names.contains(&"core-exec-fail"));
    let exec_pass = manifest
        .scenarios
        .iter()
        .find(|s| s.name == "core-exec-pass")
        .expect("core-exec-pass exists");
    assert_eq!(
        exec_pass.expected_signature_file.as_deref(),
        Some("../expected/core-exec-pass.signature.txt")
    );
}

#[test]
fn conformance_core_subset_passes() {
    let manifest = core_manifest_subset();
    let report = run_conformance_v1(
        Path::new("."),
        &manifest,
        ConformanceRunOptionsV1::default(),
    );
    assert_eq!(
        report.scenarios_failed, 0,
        "core subset should pass: {:?}",
        report.results
    );
    assert_eq!(report.scenarios_passed, report.scenarios_total);
}
