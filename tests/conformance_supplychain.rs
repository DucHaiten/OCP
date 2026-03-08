use std::path::Path;

use ocp_sdk::{
    parse_conformance_manifest_v1, run_conformance_v1, ConformanceManifestV1,
    ConformanceRunOptionsV1,
};

fn conformance_v5_path() -> &'static Path {
    Path::new("projects/ocp/conformance/conformance.v5.toml")
}

fn supplychain_manifest_subset() -> ConformanceManifestV1 {
    let parsed =
        parse_conformance_manifest_v1(conformance_v5_path()).expect("parse conformance v5");
    let scenarios = parsed
        .scenarios
        .into_iter()
        .filter(|s| s.name.starts_with("supplychain-"))
        .collect::<Vec<_>>();
    assert!(
        !scenarios.is_empty(),
        "supplychain scenarios must exist in conformance.v5"
    );
    ConformanceManifestV1 {
        schema: parsed.schema,
        scenarios,
    }
}

#[test]
fn conformance_v5_includes_supplychain_scenarios() {
    let manifest = supplychain_manifest_subset();
    let names = manifest
        .scenarios
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"supplychain-effective-permissions"));
}

#[test]
fn conformance_supplychain_subset_passes_and_is_deterministic() {
    let manifest = supplychain_manifest_subset();
    let first = run_conformance_v1(
        Path::new("."),
        &manifest,
        ConformanceRunOptionsV1::default(),
    );
    assert_eq!(
        first.scenarios_failed, 0,
        "supplychain subset should pass (first run): {:?}",
        first.results
    );

    let second = run_conformance_v1(
        Path::new("."),
        &manifest,
        ConformanceRunOptionsV1::default(),
    );
    assert_eq!(
        second.scenarios_failed, 0,
        "supplychain subset should pass (second run): {:?}",
        second.results
    );
    assert_eq!(
        first.required_digest, second.required_digest,
        "conformance supplychain digest must be deterministic across runs"
    );
}
