use std::path::Path;

use ocl_sdk::{
    parse_conformance_manifest_v1, run_conformance_v1, ConformanceManifestV1,
    ConformanceRunOptionsV1,
};

fn conformance_v5_path() -> &'static Path {
    Path::new("projects/ocp-ocl/conformance/conformance.v5.toml")
}

fn packs_manifest_subset() -> ConformanceManifestV1 {
    let parsed =
        parse_conformance_manifest_v1(conformance_v5_path()).expect("parse conformance v5");
    let scenarios = parsed
        .scenarios
        .into_iter()
        .filter(|s| {
            s.name.starts_with("packs-")
                || s.name.starts_with("shadow-")
                || s.name.starts_with("quarantine-")
        })
        .collect::<Vec<_>>();
    assert!(
        !scenarios.is_empty(),
        "packs scenarios must exist in conformance.v5"
    );
    ConformanceManifestV1 {
        schema: parsed.schema,
        scenarios,
    }
}

#[test]
fn conformance_v5_includes_packs_scenarios() {
    let manifest = packs_manifest_subset();
    let names = manifest
        .scenarios
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"packs-tool-fs-read"));
    assert!(names.contains(&"packs-consumer-ui"));
    assert!(names.contains(&"shadow-search-rr"));
    assert!(names.contains(&"quarantine-wallclock-replay"));
}

#[test]
fn conformance_packs_subset_passes_and_is_deterministic() {
    let manifest = packs_manifest_subset();
    let first = run_conformance_v1(
        Path::new("."),
        &manifest,
        ConformanceRunOptionsV1::default(),
    );
    assert_eq!(
        first.scenarios_failed, 0,
        "packs subset should pass (first run): {:?}",
        first.results
    );

    let second = run_conformance_v1(
        Path::new("."),
        &manifest,
        ConformanceRunOptionsV1::default(),
    );
    assert_eq!(
        second.scenarios_failed, 0,
        "packs subset should pass (second run): {:?}",
        second.results
    );
    assert_eq!(
        first.required_digest, second.required_digest,
        "conformance packs digest must be deterministic across runs"
    );
}
