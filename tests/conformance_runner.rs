use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{
    init_project, parse_conformance_manifest_v1, run_conformance_v1, ConformanceManifestV1,
    ConformanceRunOptionsV1, ConformanceStepV1,
};
use serde_json::{json, Map, Value as JsonValue};

#[path = "v16_gate_a_common.rs"]
mod v16;

fn temp_dir_for_test(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = PathBuf::from("target")
        .join("tests")
        .join("conformance_runner")
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

fn conformance_v5_path() -> &'static Path {
    Path::new("projects/ocp-ocl/conformance/conformance.v5.toml")
}

fn subset_manifest<F>(manifest: &ConformanceManifestV1, predicate: F) -> ConformanceManifestV1
where
    F: Fn(&str) -> bool,
{
    let scenarios = manifest
        .scenarios
        .iter()
        .filter(|scenario| predicate(&scenario.name))
        .cloned()
        .collect::<Vec<_>>();
    ConformanceManifestV1 {
        schema: manifest.schema.clone(),
        scenarios,
    }
}

fn domain_from_scenario_name(name: &str) -> &'static str {
    if name.starts_with("core-") {
        "core"
    } else if name.starts_with("packs-") || name.starts_with("shadow-") {
        "packs"
    } else if name.starts_with("quarantine-") {
        "quarantine_replay"
    } else if name.starts_with("supplychain-") {
        "supplychain"
    } else if name.starts_with("cache-") {
        "cache"
    } else if name.starts_with("foundation-") {
        "foundation"
    } else {
        "misc"
    }
}

#[test]
fn parse_manifest_supports_lane_signature_and_replay_artifact_step() {
    let root = temp_dir_for_test("manifest-extended");
    let manifest_path = root.join("conformance.v1.toml");
    let manifest = concat!(
        "version = 1\n",
        "\n",
        "[[scenario]]\n",
        "name = \"q-replay\"\n",
        "path = \"./case_project\"\n",
        "lane = \"quarantine\"\n",
        "expected_status = \"pass\"\n",
        "expected_signature_file = \"expected/signature.txt\"\n",
        "steps = [\"replay_artifact\"]\n",
    );
    fs::write(&manifest_path, manifest).expect("write manifest");

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    assert_eq!(parsed.scenarios.len(), 1);
    let sc = &parsed.scenarios[0];
    assert_eq!(sc.lane, "quarantine");
    assert_eq!(
        sc.expected_signature_file.as_deref(),
        Some("expected/signature.txt")
    );
    assert_eq!(sc.steps, vec![ConformanceStepV1::ReplayArtifact]);
}

#[test]
fn parse_manifest_defaults_lane_to_locked_v071() {
    let root = temp_dir_for_test("manifest-default-lane");
    let manifest_path = root.join("conformance.v1.toml");
    let manifest = concat!(
        "version = 1\n",
        "\n",
        "[[scenario]]\n",
        "name = \"default-lane\"\n",
        "path = \"./project\"\n",
        "expected_status = \"pass\"\n",
        "steps = [\"run\"]\n",
    );
    fs::write(&manifest_path, manifest).expect("write manifest");

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    assert_eq!(parsed.scenarios.len(), 1);
    assert_eq!(parsed.scenarios[0].lane, "locked_v071");
}

#[test]
fn run_conformance_reports_signature_mismatch() {
    let root = temp_dir_for_test("run-signature-mismatch");
    let project = root.join("project");
    init_project(&project).expect("init project");
    let ocl_toml = project.join("Ocl.toml");
    let mut manifest_text = fs::read_to_string(&ocl_toml).expect("read Ocl.toml");
    manifest_text.push_str(
        "\n[permissions.package]\nallow = [\"std.log.info\", \"std.fs.read_text\"]\ndeny = []\n",
    );
    fs::write(&ocl_toml, manifest_text).expect("patch Ocl.toml permissions");

    let expected_dir = root.join("expected");
    fs::create_dir_all(&expected_dir).expect("create expected dir");
    fs::write(expected_dir.join("signature.txt"), "deadbeef\n").expect("write expected signature");

    let manifest_path = root.join("conformance.v1.toml");
    let project_path = fs::canonicalize(&project)
        .expect("canonical project")
        .to_string_lossy()
        .replace('\\', "/");
    let manifest = format!(
        concat!(
            "version = 1\n",
            "\n",
            "[[scenario]]\n",
            "name = \"sig-mismatch\"\n",
            "path = \"{}\"\n",
            "lane = \"locked_v071\"\n",
            "expected_status = \"pass\"\n",
            "expected_signature_file = \"../expected/signature.txt\"\n",
            "steps = [\"check\"]\n",
        ),
        project_path
    );
    fs::write(&manifest_path, manifest).expect("write conformance manifest");

    let parsed = parse_conformance_manifest_v1(&manifest_path).expect("parse manifest");
    let report = run_conformance_v1(&root, &parsed, ConformanceRunOptionsV1::default());
    assert_eq!(report.scenarios_total, 1);
    assert_eq!(report.scenarios_failed, 1);
    let reason = report.results[0].reason.clone().unwrap_or_default();
    assert!(
        reason.contains("X-CONFORMANCE-SIGNATURE-MISMATCH"),
        "unexpected reason: {reason}"
    );
}

#[test]
fn v16_unified_conformance_matrix_generates_report_artifacts() {
    let manifest =
        parse_conformance_manifest_v1(conformance_v5_path()).expect("parse conformance v5");

    assert!(
        manifest
            .scenarios
            .iter()
            .any(|scenario| scenario.name.starts_with("core-")),
        "conformance.v5 missing required domain scenarios: core"
    );
    assert!(
        manifest.scenarios.iter().any(|scenario| {
            scenario.name.starts_with("packs-") || scenario.name.starts_with("shadow-")
        }),
        "conformance.v5 missing required domain scenarios: packs"
    );
    assert!(
        manifest
            .scenarios
            .iter()
            .any(|scenario| scenario.name.starts_with("quarantine-")),
        "conformance.v5 missing required domain scenarios: quarantine_replay"
    );
    assert!(
        manifest
            .scenarios
            .iter()
            .any(|scenario| scenario.name.starts_with("supplychain-")),
        "conformance.v5 missing required domain scenarios: supplychain"
    );
    assert!(
        manifest
            .scenarios
            .iter()
            .any(|scenario| scenario.name.starts_with("cache-")),
        "conformance.v5 missing required domain scenarios: cache"
    );

    let options = ConformanceRunOptionsV1::default();
    let full_first = run_conformance_v1(Path::new("."), &manifest, options.clone());
    assert_eq!(
        full_first.scenarios_failed, 0,
        "full unified conformance must pass: {:?}",
        full_first.results
    );
    let full_second = run_conformance_v1(Path::new("."), &manifest, options.clone());
    assert_eq!(
        full_second.scenarios_failed, 0,
        "full unified conformance second run must pass: {:?}",
        full_second.results
    );
    assert_eq!(
        full_first.required_digest, full_second.required_digest,
        "unified conformance digest must be deterministic across runs"
    );

    let suite_core = subset_manifest(&manifest, |name| name.starts_with("core-"));
    let suite_packs = subset_manifest(&manifest, |name| {
        name.starts_with("packs-") || name.starts_with("shadow-")
    });
    let suite_quarantine = subset_manifest(&manifest, |name| name.starts_with("quarantine-"));
    let suite_supplychain = subset_manifest(&manifest, |name| name.starts_with("supplychain-"));
    let suite_cache = subset_manifest(&manifest, |name| name.starts_with("cache-"));

    let report_core = run_conformance_v1(Path::new("."), &suite_core, options.clone());
    let report_packs = run_conformance_v1(Path::new("."), &suite_packs, options.clone());
    let report_quarantine = run_conformance_v1(Path::new("."), &suite_quarantine, options.clone());
    let report_supplychain =
        run_conformance_v1(Path::new("."), &suite_supplychain, options.clone());
    let report_cache = run_conformance_v1(Path::new("."), &suite_cache, options);

    for (suite_name, report) in [
        ("core", &report_core),
        ("packs", &report_packs),
        ("quarantine_replay", &report_quarantine),
        ("supplychain", &report_supplychain),
        ("cache", &report_cache),
    ] {
        assert_eq!(
            report.scenarios_failed, 0,
            "conformance suite `{suite_name}` must pass: {:?}",
            report.results
        );
    }

    let mut lane_counts = BTreeMap::<String, u64>::new();
    for scenario in &manifest.scenarios {
        *lane_counts.entry(scenario.lane.clone()).or_default() += 1;
    }

    let mut domain_pass = BTreeMap::<String, u64>::new();
    let mut domain_fail = BTreeMap::<String, u64>::new();
    for result in &full_first.results {
        let key = domain_from_scenario_name(&result.name).to_string();
        if result.ok {
            *domain_pass.entry(key).or_default() += 1;
        } else {
            *domain_fail.entry(key).or_default() += 1;
        }
    }

    let run_manifest_path = v16::w16_target_root()
        .join("meta")
        .join("run_manifest.json");
    if !run_manifest_path.exists() {
        let allowed_env_flags = [
            "OCL_PROFILE",
            "OCL_CONFORMANCE_DEFAULT",
            "OCL_CACHE_DISABLE",
            "OCL_ALLOW_OVERRIDES",
            "OCL_QUARANTINE",
        ];
        let observed_env_flags = std::env::vars()
            .filter(|(key, _)| key.starts_with("OCL_"))
            .map(|(key, _)| key)
            .collect::<Vec<_>>();
        let run_manifest = v16::build_run_manifest(
            &v16::repo_root(),
            allowed_env_flags.iter().map(|v| v.to_string()).collect(),
            observed_env_flags,
        );
        v16::write_json_pretty(&run_manifest_path, &run_manifest);
    }
    let run_manifest = serde_json::from_str::<JsonValue>(
        &fs::read_to_string(&run_manifest_path).expect("read run_manifest"),
    )
    .expect("parse run_manifest json");

    let conformance_dir = v16::w16_target_root().join("conformance");
    fs::create_dir_all(&conformance_dir).expect("create w16 conformance dir");
    let json_path = conformance_dir.join("unified_conformance_report.json");
    let txt_path = conformance_dir.join("unified_conformance_report.txt");

    let mut suites = Map::<String, JsonValue>::new();
    for (name, report) in [
        ("core", &report_core),
        ("packs", &report_packs),
        ("quarantine_replay", &report_quarantine),
        ("supplychain", &report_supplychain),
        ("cache", &report_cache),
    ] {
        suites.insert(
            name.to_string(),
            json!({
                "scenarios_total": report.scenarios_total,
                "scenarios_passed": report.scenarios_passed,
                "scenarios_failed": report.scenarios_failed,
                "required_digest": report.required_digest,
            }),
        );
    }

    let unified = json!({
        "schema": "ocl.w16.conformance.unified.v1",
        "manifest_path": "projects/ocp-ocl/conformance/conformance.v5.toml",
        "run_manifest_ref": "target/ocl/w16/meta/run_manifest.json",
        "run_manifest": run_manifest,
        "engine": full_first.engine,
        "runtime_mode": full_first.runtime_mode,
        "scenarios_total": full_first.scenarios_total,
        "scenarios_passed": full_first.scenarios_passed,
        "scenarios_failed": full_first.scenarios_failed,
        "required_digest": full_first.required_digest,
        "determinism_check": {
            "second_run_required_digest": full_second.required_digest,
            "stable": full_first.required_digest == full_second.required_digest
        },
        "lane_matrix": lane_counts,
        "domain_pass": domain_pass,
        "domain_fail": domain_fail,
        "suites": suites
    });

    v16::write_json_pretty(&json_path, &unified);
    let txt = format!(
        concat!(
            "Unified Conformance Report (v0.16)\n",
            "manifest: projects/ocp-ocl/conformance/conformance.v5.toml\n",
            "run_manifest: target/ocl/w16/meta/run_manifest.json\n",
            "engine: {}\n",
            "runtime_mode: {}\n",
            "scenarios_total: {}\n",
            "scenarios_passed: {}\n",
            "scenarios_failed: {}\n",
            "required_digest: {}\n",
            "determinism_stable: {}\n",
            "suites:\n",
            "  - core: total={} pass={} fail={} digest={}\n",
            "  - packs: total={} pass={} fail={} digest={}\n",
            "  - quarantine_replay: total={} pass={} fail={} digest={}\n",
            "  - supplychain: total={} pass={} fail={} digest={}\n",
            "  - cache: total={} pass={} fail={} digest={}\n"
        ),
        full_first.engine,
        full_first.runtime_mode,
        full_first.scenarios_total,
        full_first.scenarios_passed,
        full_first.scenarios_failed,
        full_first.required_digest,
        full_first.required_digest == full_second.required_digest,
        report_core.scenarios_total,
        report_core.scenarios_passed,
        report_core.scenarios_failed,
        report_core.required_digest,
        report_packs.scenarios_total,
        report_packs.scenarios_passed,
        report_packs.scenarios_failed,
        report_packs.required_digest,
        report_quarantine.scenarios_total,
        report_quarantine.scenarios_passed,
        report_quarantine.scenarios_failed,
        report_quarantine.required_digest,
        report_supplychain.scenarios_total,
        report_supplychain.scenarios_passed,
        report_supplychain.scenarios_failed,
        report_supplychain.required_digest,
        report_cache.scenarios_total,
        report_cache.scenarios_passed,
        report_cache.scenarios_failed,
        report_cache.required_digest,
    );
    fs::write(&txt_path, txt).expect("write unified_conformance_report.txt");

    let persisted = serde_json::from_str::<JsonValue>(
        &fs::read_to_string(&json_path).expect("read unified report"),
    )
    .expect("parse unified report");
    assert_eq!(
        persisted
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or_default(),
        "ocl.w16.conformance.unified.v1"
    );
    assert_eq!(
        persisted
            .get("run_manifest_ref")
            .and_then(JsonValue::as_str)
            .unwrap_or_default(),
        "target/ocl/w16/meta/run_manifest.json"
    );
    assert!(
        persisted
            .get("determinism_check")
            .and_then(JsonValue::as_object)
            .and_then(|obj| obj.get("stable"))
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        "unified conformance determinism check must be stable"
    );
}
