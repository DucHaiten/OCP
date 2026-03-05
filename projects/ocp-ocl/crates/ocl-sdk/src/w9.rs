use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ocl_runtime_core::RunEngine;
use sha2::{Digest, Sha256};

use crate::{
    build_project_with_lock, check_project_with_lock, compose_phenotype,
    compute_effective_permissions_v10, enforce_universe_match_v1, install_organs_v1,
    resolve_domain_selection_v1, resolve_universe_v1, run_kit_doctor_v1,
    run_project_with_engine_and_lock, run_project_with_trace_engine_and_lock,
    run_reactor_service_with_lock, sync_deps_lock_v1, test_project_with_lock,
    trace_required_digest, verify_assembly, verify_dependency_exports_and_collect_provenance_v10,
    verify_organs_lock_v1, ProjectPermissions, ReactorRuntimeMode, ReactorServiceOptions, SdkError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConformanceExpectedStatusV1 {
    #[default]
    Pass,
    Fail,
}

impl ConformanceExpectedStatusV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceStepV1 {
    Composer,
    Check,
    Run,
    ReplayArtifact,
    Reactor,
    Test,
    Build,
    KitDoctor,
    OrganVerify,
    OrganInstall,
}

impl ConformanceStepV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Composer => "composer",
            Self::Check => "check",
            Self::Run => "run",
            Self::ReplayArtifact => "replay_artifact",
            Self::Reactor => "reactor",
            Self::Test => "test",
            Self::Build => "build",
            Self::KitDoctor => "kit_doctor",
            Self::OrganVerify => "organ_verify",
            Self::OrganInstall => "organ_install",
        }
    }

    fn parse(raw: &str) -> Result<Self, SdkError> {
        match raw {
            "composer" => Ok(Self::Composer),
            "check" => Ok(Self::Check),
            "run" => Ok(Self::Run),
            "replay_artifact" => Ok(Self::ReplayArtifact),
            "reactor" => Ok(Self::Reactor),
            "test" => Ok(Self::Test),
            "build" => Ok(Self::Build),
            "kit_doctor" => Ok(Self::KitDoctor),
            "organ_verify" => Ok(Self::OrganVerify),
            "organ_install" => Ok(Self::OrganInstall),
            _ => Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: unknown step `{raw}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceScenarioV1 {
    pub name: String,
    pub path: String,
    pub lane: String,
    pub reactor_ticks: u32,
    pub composer: bool,
    pub expected_status: ConformanceExpectedStatusV1,
    pub expected_error_code: Option<String>,
    pub expected_signature_file: Option<String>,
    pub expected_deps_file: Option<String>,
    pub expected_permissions_effective_file: Option<String>,
    pub steps: Vec<ConformanceStepV1>,
    pub organ_name: Option<String>,
    pub organ_version: Option<String>,
    pub organ_registry: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceManifestV1 {
    pub schema: String,
    pub scenarios: Vec<ConformanceScenarioV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceRunOptionsV1 {
    pub locked: bool,
    pub engine: RunEngine,
    pub runtime_mode: ReactorRuntimeMode,
    pub universe_id: Option<String>,
}

impl Default for ConformanceRunOptionsV1 {
    fn default() -> Self {
        Self {
            locked: true,
            engine: RunEngine::Dual,
            runtime_mode: ReactorRuntimeMode::Deterministic,
            universe_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceScenarioResultV1 {
    pub name: String,
    pub path: String,
    pub ok: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReportV1 {
    pub schema: String,
    pub run_id: String,
    pub engine: String,
    pub runtime_mode: String,
    pub scenarios_total: usize,
    pub scenarios_passed: usize,
    pub scenarios_failed: usize,
    pub required_digest: String,
    pub results: Vec<ConformanceScenarioResultV1>,
}

pub fn default_conformance_manifest_path(workspace_root: &Path) -> PathBuf {
    let selector = std::env::var("OCL_CONFORMANCE_DEFAULT").ok();
    default_conformance_manifest_path_with_selector(workspace_root, selector.as_deref())
}

pub fn default_conformance_manifest_path_with_selector(
    workspace_root: &Path,
    selector: Option<&str>,
) -> PathBuf {
    let v1 = workspace_root.join("projects/ocp-ocl/conformance/conformance.v1.toml");
    let v5 = workspace_root.join("projects/ocp-ocl/conformance/conformance.v5.toml");
    match selector
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "v1" => v1,
        "v5" => v5,
        _ => {
            if v5.exists() {
                v5
            } else {
                v1
            }
        }
    }
}

pub fn parse_conformance_manifest_v1(path: &Path) -> Result<ConformanceManifestV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut schema: Option<String> = None;
    let mut scenarios = Vec::new();
    let mut current = ScenarioBuilder::default();
    let mut in_scenario = false;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[scenario]]" {
            if in_scenario {
                scenarios.push(current.build()?);
            }
            current = ScenarioBuilder::default();
            in_scenario = true;
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim();

        if !in_scenario {
            if key == "version" && strip_quotes(value) == "1" {
                schema = Some("ocl.conformance.v1".to_string());
            } else if key == "schema" {
                schema = Some(strip_quotes(value).to_string());
            }
            continue;
        }

        match key {
            "name" => current.name = Some(strip_quotes(value).to_string()),
            "path" => current.path = Some(strip_quotes(value).to_string()),
            "lane" => current.lane = Some(normalize_lane_v14(strip_quotes(value))),
            "reactor_ticks" => {
                let ticks = strip_quotes(value).parse::<u32>().map_err(|_| {
                    SdkError::MissingProject(
                        "invalid conformance manifest: `reactor_ticks` must be u32".to_string(),
                    )
                })?;
                current.reactor_ticks = Some(ticks);
            }
            "composer" => {
                let flag = match strip_quotes(value) {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(SdkError::MissingProject(
                            "invalid conformance manifest: `composer` must be true|false"
                                .to_string(),
                        ));
                    }
                };
                current.composer = Some(flag);
            }
            "expected_status" => {
                let status = match strip_quotes(value) {
                    "pass" => ConformanceExpectedStatusV1::Pass,
                    "fail" => ConformanceExpectedStatusV1::Fail,
                    other => {
                        return Err(SdkError::MissingProject(format!(
                            "invalid conformance manifest: `expected_status` unsupported `{other}`"
                        )));
                    }
                };
                current.expected_status = Some(status);
            }
            "expected_error_code" => {
                current.expected_error_code = Some(strip_quotes(value).to_string())
            }
            "expected_signature_file" => {
                current.expected_signature_file = Some(strip_quotes(value).to_string())
            }
            "expected_deps_file" => {
                current.expected_deps_file = Some(strip_quotes(value).to_string())
            }
            "expected_permissions_effective_file" => {
                current.expected_permissions_effective_file = Some(strip_quotes(value).to_string())
            }
            "steps" => {
                current.steps = Some(parse_step_list(value)?);
            }
            "organ_name" => current.organ_name = Some(strip_quotes(value).to_string()),
            "organ_version" => current.organ_version = Some(strip_quotes(value).to_string()),
            "organ_registry" => current.organ_registry = Some(strip_quotes(value).to_string()),
            _ => {}
        }
    }

    if in_scenario {
        scenarios.push(current.build()?);
    }

    let schema = schema.ok_or_else(|| {
        SdkError::MissingProject(
            "invalid conformance manifest: missing `version = 1` or `schema = \"ocl.conformance.manifest.v5\"`"
                .to_string(),
        )
    })?;
    if schema != "ocl.conformance.v1" && schema != "ocl.conformance.manifest.v5" {
        return Err(SdkError::MissingProject(format!(
            "invalid conformance manifest: unsupported schema `{schema}`"
        )));
    }
    if scenarios.is_empty() {
        return Err(SdkError::MissingProject(
            "invalid conformance manifest: no `[[scenario]]` entries".to_string(),
        ));
    }

    let mut names = HashSet::new();
    for scenario in &scenarios {
        if !names.insert(scenario.name.clone()) {
            return Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: duplicate scenario name `{}`",
                scenario.name
            )));
        }
    }

    Ok(ConformanceManifestV1 { schema, scenarios })
}

pub fn run_conformance_v1(
    workspace_root: &Path,
    manifest: &ConformanceManifestV1,
    options: ConformanceRunOptionsV1,
) -> ConformanceReportV1 {
    let mut results = Vec::new();
    let runtime_root = conformance_runtime_root_v16(workspace_root);
    let copied_registry_root = copy_runtime_registry(&runtime_root, workspace_root).ok();

    for (idx, scenario) in manifest.scenarios.iter().enumerate() {
        let source_root = resolve_scenario_root(workspace_root, &scenario.path);
        let copied_root =
            match prepare_scenario_workspace_copy(&runtime_root, &source_root, &scenario.name, idx)
            {
                Ok(path) => path,
                Err(err) => {
                    results.push(ConformanceScenarioResultV1 {
                        name: scenario.name.clone(),
                        path: scenario.path.clone(),
                        ok: false,
                        reason: Some(err.to_string()),
                    });
                    continue;
                }
            };

        let outcome = run_scenario(
            workspace_root,
            &source_root,
            &copied_root,
            scenario,
            &options,
            copied_registry_root.as_deref(),
        );
        let (ok, reason) = evaluate_scenario_outcome(scenario, outcome);
        results.push(ConformanceScenarioResultV1 {
            name: scenario.name.clone(),
            path: scenario.path.clone(),
            ok,
            reason,
        });
    }

    let scenarios_total = results.len();
    let scenarios_passed = results.iter().filter(|r| r.ok).count();
    let scenarios_failed = scenarios_total.saturating_sub(scenarios_passed);
    let required_digest = compute_conformance_required_digest(&results);
    let run_id = format!(
        "w9-{}",
        fnv1a64_hex(&format!(
            "{}|{}|{}|{}",
            options.engine.as_str(),
            options.runtime_mode.as_str(),
            manifest.schema,
            required_digest
        ))
    );

    ConformanceReportV1 {
        schema: manifest.schema.clone(),
        run_id,
        engine: options.engine.as_str().to_string(),
        runtime_mode: options.runtime_mode.as_str().to_string(),
        scenarios_total,
        scenarios_passed,
        scenarios_failed,
        required_digest,
        results,
    }
}

pub fn compute_conformance_required_digest(results: &[ConformanceScenarioResultV1]) -> String {
    let mut canonical = String::new();
    for result in results {
        canonical.push_str(&result.name);
        canonical.push('|');
        canonical.push_str(&result.path);
        canonical.push('|');
        canonical.push_str(if result.ok { "ok" } else { "fail" });
        canonical.push('|');
        canonical.push_str(result.reason.as_deref().unwrap_or("-"));
        canonical.push('\n');
    }
    fnv1a64_hex(&canonical)
}

pub fn render_conformance_report_json(report: &ConformanceReportV1) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"");
    out.push_str(&json_escape(&report.schema));
    out.push_str("\",\"run_id\":\"");
    out.push_str(&json_escape(&report.run_id));
    out.push_str("\",\"engine\":\"");
    out.push_str(&json_escape(&report.engine));
    out.push_str("\",\"runtime_mode\":\"");
    out.push_str(&json_escape(&report.runtime_mode));
    out.push_str("\",\"scenarios_total\":");
    out.push_str(&report.scenarios_total.to_string());
    out.push_str(",\"scenarios_passed\":");
    out.push_str(&report.scenarios_passed.to_string());
    out.push_str(",\"scenarios_failed\":");
    out.push_str(&report.scenarios_failed.to_string());
    out.push_str(",\"required_digest\":\"");
    out.push_str(&report.required_digest);
    out.push_str("\",\"results\":[");
    for (idx, result) in report.results.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":\"");
        out.push_str(&json_escape(&result.name));
        out.push_str("\",\"path\":\"");
        out.push_str(&json_escape(&result.path));
        out.push_str("\",\"ok\":");
        out.push_str(if result.ok { "true" } else { "false" });
        out.push_str(",\"reason\":");
        if let Some(reason) = &result.reason {
            out.push('"');
            out.push_str(&json_escape(reason));
            out.push('"');
        } else {
            out.push_str("null");
        }
        out.push('}');
    }
    out.push_str("]}");
    out
}

pub fn write_conformance_report_json(
    path: &Path,
    report: &ConformanceReportV1,
) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_conformance_report_json(report))?;
    Ok(())
}

#[derive(Debug, Default)]
struct ScenarioBuilder {
    name: Option<String>,
    path: Option<String>,
    lane: Option<String>,
    reactor_ticks: Option<u32>,
    composer: Option<bool>,
    expected_status: Option<ConformanceExpectedStatusV1>,
    expected_error_code: Option<String>,
    expected_signature_file: Option<String>,
    expected_deps_file: Option<String>,
    expected_permissions_effective_file: Option<String>,
    steps: Option<Vec<ConformanceStepV1>>,
    organ_name: Option<String>,
    organ_version: Option<String>,
    organ_registry: Option<String>,
}

impl ScenarioBuilder {
    fn build(self) -> Result<ConformanceScenarioV1, SdkError> {
        let name = self.name.ok_or_else(|| {
            SdkError::MissingProject(
                "invalid conformance manifest: scenario missing `name`".to_string(),
            )
        })?;
        let path = self.path.ok_or_else(|| {
            SdkError::MissingProject(
                "invalid conformance manifest: scenario missing `path`".to_string(),
            )
        })?;
        let lane = normalize_lane_v14(self.lane.as_deref().unwrap_or("locked_v071"));
        let steps = self.steps.unwrap_or_default();
        let expected_status = self.expected_status.unwrap_or_default();
        let expected_error_code = self.expected_error_code;
        let expected_signature_file = self.expected_signature_file;
        let expected_deps_file = self.expected_deps_file;
        let expected_permissions_effective_file = self.expected_permissions_effective_file;

        if !matches!(lane.as_str(), "locked_v071" | "locked_v06" | "quarantine") {
            return Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: scenario `{name}` has unsupported `lane={lane}`"
            )));
        }

        if expected_status == ConformanceExpectedStatusV1::Fail && expected_error_code.is_none() {
            return Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: scenario `{name}` expects fail but missing `expected_error_code`"
            )));
        }
        if steps.contains(&ConformanceStepV1::Reactor) && self.reactor_ticks.unwrap_or(0) == 0 {
            return Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: scenario `{name}` step `reactor` requires `reactor_ticks > 0`"
            )));
        }
        if steps.contains(&ConformanceStepV1::OrganInstall)
            && (self.organ_name.is_none() || self.organ_version.is_none())
        {
            return Err(SdkError::MissingProject(format!(
                "invalid conformance manifest: scenario `{name}` step `organ_install` requires `organ_name` and `organ_version`"
            )));
        }

        Ok(ConformanceScenarioV1 {
            name,
            path,
            lane,
            reactor_ticks: self.reactor_ticks.unwrap_or(0),
            composer: self.composer.unwrap_or(false),
            expected_status,
            expected_error_code,
            expected_signature_file,
            expected_deps_file,
            expected_permissions_effective_file,
            steps,
            organ_name: self.organ_name,
            organ_version: self.organ_version,
            organ_registry: self.organ_registry,
        })
    }
}

fn parse_step_list(raw: &str) -> Result<Vec<ConformanceStepV1>, SdkError> {
    let parts = parse_string_list(raw)?;
    let mut steps = Vec::new();
    for part in parts {
        steps.push(ConformanceStepV1::parse(&part)?);
    }
    Ok(steps)
}

fn parse_string_list(raw: &str) -> Result<Vec<String>, SdkError> {
    let text = raw.trim();
    if !text.starts_with('[') || !text.ends_with(']') {
        return Err(SdkError::MissingProject(format!(
            "invalid conformance manifest: list syntax expected `[...]`, got `{raw}`"
        )));
    }
    let body = &text[1..text.len().saturating_sub(1)];
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for chunk in body.split(',') {
        let token = strip_quotes(chunk.trim()).to_string();
        if token.is_empty() {
            return Err(SdkError::MissingProject(
                "invalid conformance manifest: list items must not be empty".to_string(),
            ));
        }
        out.push(token);
    }
    Ok(out)
}

fn resolve_scenario_root(workspace_root: &Path, scenario_path: &str) -> PathBuf {
    let path = PathBuf::from(scenario_path);
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn evaluate_scenario_outcome(
    scenario: &ConformanceScenarioV1,
    outcome: Result<(), SdkError>,
) -> (bool, Option<String>) {
    match scenario.expected_status {
        ConformanceExpectedStatusV1::Pass => match outcome {
            Ok(()) => (true, None),
            Err(err) => (false, Some(err.to_string())),
        },
        ConformanceExpectedStatusV1::Fail => match outcome {
            Err(err) => {
                let text = err.to_string();
                let expected = scenario.expected_error_code.as_deref().unwrap_or_default();
                if expected.is_empty() {
                    return (true, None);
                }
                let actual_code = extract_error_code(&text);
                if actual_code.as_deref() == Some(expected) || text.contains(expected) {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "V-W9-EXPECTED-FAIL-MISMATCH: expected `{expected}`, actual `{}` ({text})",
                            actual_code.unwrap_or_else(|| "unknown".to_string())
                        )),
                    )
                }
            }
            Ok(()) => (
                false,
                Some(format!(
                    "V-W9-EXPECTED-FAIL-NOT-TRIGGERED: expected failure `{}` but scenario passed",
                    scenario.expected_error_code.as_deref().unwrap_or("unknown")
                )),
            ),
        },
    }
}

fn run_scenario(
    workspace_root: &Path,
    source_root: &Path,
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
    options: &ConformanceRunOptionsV1,
    copied_registry_root: Option<&Path>,
) -> Result<(), SdkError> {
    sync_deps_lock_v1(app_root)?;
    let scenario_locked = is_locked_lane_v14(&scenario.lane);

    let steps = if scenario.steps.is_empty() {
        vec![
            ConformanceStepV1::Composer,
            ConformanceStepV1::Check,
            ConformanceStepV1::Run,
            ConformanceStepV1::Reactor,
            ConformanceStepV1::Test,
            ConformanceStepV1::Build,
        ]
    } else {
        scenario.steps.clone()
    };

    let mut runtime_ctx: Option<ScenarioRuntimeContext> = None;
    for step in steps {
        match step {
            ConformanceStepV1::Composer => {
                if scenario.composer {
                    let phenotype_path = app_root.join("phenotype.toml");
                    let registry_root = app_root.join("registry");
                    compose_phenotype(app_root, &phenotype_path, &registry_root, scenario_locked)?;
                    verify_assembly(app_root, &phenotype_path, &registry_root, scenario_locked)?;
                }
            }
            ConformanceStepV1::Check => {
                check_project_with_lock(app_root, scenario_locked)?;
            }
            ConformanceStepV1::Run => {
                if runtime_ctx.is_none() {
                    runtime_ctx = Some(resolve_runtime_context(app_root, options)?);
                }
                let ctx = runtime_ctx.as_ref().expect("runtime context must exist");
                run_project_with_engine_and_lock(app_root, ctx.run_engine, scenario_locked)?;
            }
            ConformanceStepV1::ReplayArtifact => {
                run_replay_artifact_step(workspace_root, source_root, app_root, scenario, options)?;
            }
            ConformanceStepV1::Reactor => {
                if scenario.reactor_ticks == 0 {
                    continue;
                }
                if runtime_ctx.is_none() {
                    runtime_ctx = Some(resolve_runtime_context(app_root, options)?);
                }
                let ctx = runtime_ctx.as_ref().expect("runtime context must exist");
                let reactor_options = ReactorServiceOptions {
                    ticks: scenario.reactor_ticks,
                    runtime_mode: ctx.runtime_mode,
                    socket_listen: None,
                    runtime_report: None,
                    replay_audit: None,
                    io_tape_record_path: None,
                    io_tape_replay_path: None,
                    hive_caps: None,
                    universe_id: ctx.reactor_universe_id.clone(),
                    domain_id: ctx.reactor_domain_id.clone(),
                };
                run_reactor_service_with_lock(app_root, &reactor_options, scenario_locked)?;
            }
            ConformanceStepV1::Test => {
                test_project_with_lock(app_root, scenario_locked)?;
            }
            ConformanceStepV1::Build => {
                build_project_with_lock(app_root, scenario_locked)?;
            }
            ConformanceStepV1::KitDoctor => {
                run_kit_doctor_v1(app_root, scenario_locked)?;
            }
            ConformanceStepV1::OrganVerify => {
                verify_organs_lock_v1(app_root, scenario_locked)?;
            }
            ConformanceStepV1::OrganInstall => {
                let name = scenario.organ_name.as_deref().ok_or_else(|| {
                    SdkError::MissingProject(
                        "invalid conformance scenario: missing `organ_name`".to_string(),
                    )
                })?;
                let version = scenario.organ_version.as_deref().ok_or_else(|| {
                    SdkError::MissingProject(
                        "invalid conformance scenario: missing `organ_version`".to_string(),
                    )
                })?;
                let registry_index = resolve_organ_registry_index_path(
                    workspace_root,
                    app_root,
                    scenario,
                    copied_registry_root,
                );
                install_organs_v1(app_root, &registry_index, name, version, scenario_locked)?;
            }
        }
    }

    if let Some(expected_signature_file) = scenario.expected_signature_file.as_deref() {
        let expected_path = resolve_expected_signature_path_v14(
            workspace_root,
            source_root,
            expected_signature_file,
        );
        if !expected_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-SIGNATURE-MISMATCH: expected signature file missing `{}`",
                expected_path.display()
            )));
        }
        let expected = read_trimmed_file_v14(&expected_path)?;
        let actual = compute_actual_signature_v14(app_root, scenario, options)?;
        if expected != actual {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-SIGNATURE-MISMATCH: expected={} actual={} scenario={}",
                expected, actual, scenario.name
            )));
        }
    }

    if scenario.expected_deps_file.is_some()
        || scenario.expected_permissions_effective_file.is_some()
    {
        assert_supplychain_outputs_v14(workspace_root, source_root, app_root, scenario)?;
    }

    Ok(())
}

fn resolve_organ_registry_index_path(
    workspace_root: &Path,
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
    copied_registry_root: Option<&Path>,
) -> PathBuf {
    if let Some(raw) = scenario.organ_registry.as_deref() {
        let path = PathBuf::from(raw);
        if path.is_absolute() {
            return path;
        }
        return workspace_root.join(path);
    }
    if let Some(root) = copied_registry_root {
        return root.join("organs").join("index.toml");
    }
    let local = app_root.join("registry").join("organs").join("index.toml");
    if local.exists() {
        return local;
    }
    workspace_root.join("projects/ocp-ocl/registry/organs/index.toml")
}

fn is_locked_lane_v14(lane: &str) -> bool {
    matches!(lane, "locked_v071" | "locked_v06")
}

fn resolve_expected_signature_path_v14(
    workspace_root: &Path,
    source_root: &Path,
    raw: &str,
) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return path;
    }
    let candidate = source_root.join(&path);
    if candidate.exists() {
        return candidate;
    }
    if let Some(parent) = source_root.parent() {
        let from_parent = parent.join(&path);
        if from_parent.exists() {
            return from_parent;
        }
    }
    workspace_root.join(path)
}

fn resolve_expected_cassette_dir_v14(source_root: &Path) -> Option<PathBuf> {
    let direct = source_root.join("expected").join("cassette");
    if direct.exists() {
        return Some(direct);
    }
    source_root
        .parent()
        .map(|p| p.join("expected").join("cassette"))
        .filter(|p| p.exists())
}

fn run_replay_artifact_step(
    workspace_root: &Path,
    source_root: &Path,
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
    options: &ConformanceRunOptionsV1,
) -> Result<(), SdkError> {
    if scenario.lane != "quarantine" {
        return Err(SdkError::MissingProject(format!(
            "invalid conformance scenario `{}`: step `replay_artifact` requires lane=quarantine",
            scenario.name
        )));
    }

    let cassette_src = resolve_expected_cassette_dir_v14(source_root).ok_or_else(|| {
        SdkError::MissingProject(format!(
            "V-W14-CASSETTE-MISSING: expected/cassette not found for scenario `{}` (source root `{}`)",
            scenario.name,
            source_root.display()
        ))
    })?;
    let artifact_dir = app_root.join(".ocl_artifacts").join("conformance.replay");
    let cassette_dst = artifact_dir.join("cassette");
    if cassette_dst.exists() {
        fs::remove_dir_all(&cassette_dst)?;
    }
    copy_tree(&cassette_src, &cassette_dst)?;

    let replay_toml = artifact_dir.join("replay.toml");
    let mut replay_signature = None::<String>;
    let mut replay_cassette_hash = None::<String>;
    if replay_toml.exists() {
        replay_signature = parse_replay_signature_v14(&replay_toml)?;
        replay_cassette_hash = parse_replay_cassette_hash_v14(&replay_toml)?;
        if let Some(hash) = replay_cassette_hash.as_deref() {
            let cassette_jsonl_path = cassette_dst.join("cassette.jsonl");
            let cassette_index_path = cassette_dst.join("cassette_index.json");
            if !cassette_jsonl_path.exists() || !cassette_index_path.exists() {
                return Err(SdkError::MissingProject(format!(
                    "V-W14-CASSETTE-MISSING: replay requires cassette.jsonl and cassette_index.json in {}",
                    cassette_dst.display()
                )));
            }
            let cassette_jsonl = fs::read_to_string(cassette_jsonl_path)?;
            let cassette_index = fs::read_to_string(cassette_index_path)?;
            let actual_hash = compute_cassette_hash_v14(&cassette_jsonl, &cassette_index);
            if actual_hash != hash {
                return Err(SdkError::MissingProject(format!(
                    "V-W14-CASSETTE-HASH-MISMATCH: expected={} actual={}",
                    hash, actual_hash
                )));
            }
        }
    }

    let cassette_jsonl_path = cassette_dst.join("cassette.jsonl");
    let cassette_index_path = cassette_dst.join("cassette_index.json");
    if !cassette_jsonl_path.exists() || !cassette_index_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-W14-CASSETTE-MISSING: replay requires cassette.jsonl and cassette_index.json in {}",
            cassette_dst.display()
        )));
    }

    let trace = with_runtime_env_v14(
        vec![
            (ENV_PROJECT_LANE_V08, Some("quarantine".to_string())),
            (
                ENV_QUARANTINE_MODE_V08,
                Some(CASSETTE_MODE_REPLAY_V08.to_string()),
            ),
            (ENV_WALLCLOCK_RECORD_PATH_V08, None),
            (ENV_PROC_RECORD_PATH_V08, None),
            (ENV_HTTP_RECORD_PATH_V08, None),
            (
                ENV_CASSETTE_JSONL_PATH_V08,
                Some(cassette_jsonl_path.to_string_lossy().to_string()),
            ),
            (
                ENV_CASSETTE_INDEX_PATH_V08,
                Some(cassette_index_path.to_string_lossy().to_string()),
            ),
        ],
        || run_project_with_trace_engine_and_lock(app_root, options.engine, false),
    )?;
    let actual_signature = signature_with_lane_v14(
        &trace_required_digest(&trace.events),
        &scenario.lane,
        replay_cassette_hash.as_deref(),
    );
    if let Some(expected) = replay_signature.as_deref() {
        if expected != actual_signature {
            return Err(SdkError::MissingProject(format!(
                "V-REPLAY-SIGNATURE-MISMATCH: expected={} actual={} scenario={}",
                expected, actual_signature, scenario.name
            )));
        }
    }
    fs::write(
        artifact_dir.join("signature.txt"),
        format!("{actual_signature}\n"),
    )?;

    let _ = workspace_root;
    Ok(())
}

fn parse_replay_cassette_hash_v14(path: &Path) -> Result<Option<String>, SdkError> {
    let raw = fs::read_to_string(path)?;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("cassette_hash") {
            if let Some((_, rhs)) = value.split_once('=') {
                let hash = strip_quotes(rhs.trim()).to_string();
                if hash.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(hash));
            }
        }
    }
    Ok(None)
}

fn parse_replay_signature_v14(path: &Path) -> Result<Option<String>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut in_replay = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_replay = trimmed == "[replay]";
            continue;
        }
        if !in_replay || trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("signature") {
            if let Some((_, rhs)) = value.split_once('=') {
                let signature = strip_quotes(rhs.trim()).to_string();
                if signature.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(signature));
            }
        }
    }
    Ok(None)
}

fn compute_cassette_hash_v14(cassette_jsonl: &str, cassette_index_json: &str) -> String {
    let canonical = format!(
        concat!(
            "schema_version={}\n",
            "hasher_version={}\n",
            "lane={}\n",
            "mode={}\n",
            "---cassette.jsonl---\n{}\n",
            "---cassette_index.json---\n{}\n"
        ),
        "ocl.cassette.v08",
        "sha256-v1",
        "quarantine",
        "replay",
        cassette_jsonl,
        cassette_index_json
    );
    sha256_hex_v14(canonical.as_bytes())
}

fn compute_actual_signature_v14(
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
    options: &ConformanceRunOptionsV1,
) -> Result<String, SdkError> {
    if scenario.lane == "quarantine" {
        let signature_path = app_root
            .join(".ocl_artifacts")
            .join("conformance.replay")
            .join("signature.txt");
        if !signature_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "V-W14-QUARANTINE-SIGNATURE-MISSING: {}",
                signature_path.display()
            )));
        }
        return read_trimmed_file_v14(&signature_path);
    }
    let trace = run_project_with_trace_engine_and_lock(app_root, options.engine, true)?;
    let required_digest = trace_required_digest(&trace.events);
    Ok(fnv1a64_hex(&format!(
        "lane={}\npayload={required_digest}",
        scenario.lane
    )))
}

fn assert_supplychain_outputs_v14(
    workspace_root: &Path,
    source_root: &Path,
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
) -> Result<(), SdkError> {
    let output_dir = app_root.join(".ocl_artifacts").join("conformance.outputs");
    fs::create_dir_all(&output_dir)?;

    if let Some(expected_deps_file) = scenario.expected_deps_file.as_deref() {
        let expected_path =
            resolve_expected_signature_path_v14(workspace_root, source_root, expected_deps_file);
        if !expected_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-OUTPUT-MISMATCH: expected deps file missing `{}`",
                expected_path.display()
            )));
        }
        let actual = render_deps_output_v14(app_root)?;
        fs::write(output_dir.join("deps.json"), &actual)?;
        let expected = fs::read_to_string(&expected_path)?;
        if normalize_text_v14(&expected) != normalize_text_v14(&actual) {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-OUTPUT-MISMATCH: deps.json mismatch scenario={}",
                scenario.name
            )));
        }
    }

    if let Some(expected_perm_file) = scenario.expected_permissions_effective_file.as_deref() {
        let expected_path =
            resolve_expected_signature_path_v14(workspace_root, source_root, expected_perm_file);
        if !expected_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-OUTPUT-MISMATCH: expected permissions file missing `{}`",
                expected_path.display()
            )));
        }
        let actual = render_permissions_effective_output_v14(app_root)?;
        fs::write(output_dir.join("permissions_effective.json"), &actual)?;
        let expected = fs::read_to_string(&expected_path)?;
        if normalize_text_v14(&expected) != normalize_text_v14(&actual) {
            return Err(SdkError::MissingProject(format!(
                "X-CONFORMANCE-OUTPUT-MISMATCH: permissions_effective.json mismatch scenario={}",
                scenario.name
            )));
        }
    }

    Ok(())
}

fn render_deps_output_v14(app_root: &Path) -> Result<String, SdkError> {
    let mut provenance = verify_dependency_exports_and_collect_provenance_v10(app_root)?;
    provenance.sort_by(|a, b| {
        a.module_id
            .cmp(&b.module_id)
            .then(a.package_id.cmp(&b.package_id))
    });
    let mut packages = BTreeSet::<String>::new();
    for row in &provenance {
        packages.insert(row.package_id.clone());
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema\":\"ocl.conformance.deps.v14\",\n");
    out.push_str("  \"packages\":[");
    let package_list = packages.into_iter().collect::<Vec<_>>();
    if package_list.is_empty() {
        out.push_str("],\n");
    } else {
        out.push('\n');
        for (idx, package_id) in package_list.iter().enumerate() {
            out.push_str("    \"");
            out.push_str(&json_escape(package_id));
            out.push('"');
            if idx + 1 != package_list.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }
    out.push_str("  \"provenance\":[");
    if provenance.is_empty() {
        out.push_str("]\n");
    } else {
        out.push('\n');
        for (idx, row) in provenance.iter().enumerate() {
            out.push_str("    {\"module_id\":\"");
            out.push_str(&json_escape(&row.module_id));
            out.push_str("\",\"package_id\":\"");
            out.push_str(&json_escape(&row.package_id));
            out.push_str("\"}");
            if idx + 1 != provenance.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n");
    }
    out.push_str("}\n");
    Ok(out)
}

fn render_permissions_effective_output_v14(app_root: &Path) -> Result<String, SdkError> {
    let mut effective = compute_effective_permissions_v10(app_root)?
        .into_iter()
        .collect::<Vec<(String, ProjectPermissions)>>();
    effective.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema\":\"ocl.conformance.permissions_effective.v14\",\n");
    out.push_str("  \"packages\":[");
    if effective.is_empty() {
        out.push_str("]\n");
    } else {
        out.push('\n');
        for (idx, (package_id, permissions)) in effective.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str("      \"package_id\":\"");
            out.push_str(&json_escape(package_id));
            out.push_str("\",\n");
            out.push_str("      \"global_deny\":");
            out.push_str(&render_string_array_v14(&permissions.global_deny));
            out.push_str(",\n");
            if let Some(pkg) = permissions.package.as_ref() {
                out.push_str("      \"package_allow\":");
                out.push_str(&render_string_array_v14(&pkg.allow));
                out.push_str(",\n");
                out.push_str("      \"package_deny\":");
                out.push_str(&render_string_array_v14(&pkg.deny));
                out.push_str(",\n");
            }
            if let Some(proc_cfg) = permissions.std_proc.as_ref() {
                out.push_str("      \"std_proc\":{\n");
                out.push_str("        \"enabled\":");
                out.push_str(if proc_cfg.enabled { "true" } else { "false" });
                out.push_str(",\n");
                out.push_str("        \"allow_bins\":");
                out.push_str(&render_string_array_v14(&proc_cfg.allow_bins));
                out.push_str(",\n");
                out.push_str("        \"timeout_ms\":");
                out.push_str(&proc_cfg.timeout_ms.to_string());
                out.push_str(",\n");
                out.push_str("        \"max_stdout_bytes\":");
                out.push_str(&proc_cfg.max_stdout_bytes.to_string());
                out.push_str(",\n");
                out.push_str("        \"max_stderr_bytes\":");
                out.push_str(&proc_cfg.max_stderr_bytes.to_string());
                out.push('\n');
                out.push_str("      }\n");
            } else {
                out.push_str("      \"std_proc\":null\n");
            }
            out.push_str("    }");
            if idx + 1 != effective.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n");
    }
    out.push_str("}\n");
    Ok(out)
}

fn render_string_array_v14(values: &[String]) -> String {
    let mut sorted = values.to_vec();
    sorted.sort();
    let mut out = String::new();
    out.push('[');
    for (idx, value) in sorted.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(value));
        out.push('"');
    }
    out.push(']');
    out
}

fn normalize_text_v14(raw: &str) -> String {
    raw.replace("\r\n", "\n").trim().to_string()
}

const ENV_PROJECT_LANE_V08: &str = "OCL_PROJECT_LANE";
const ENV_QUARANTINE_MODE_V08: &str = "OCL_QUARANTINE_MODE";
const ENV_WALLCLOCK_RECORD_PATH_V08: &str = "OCL_V08_WALLCLOCK_RECORD_PATH";
const ENV_PROC_RECORD_PATH_V08: &str = "OCL_V08_PROC_RECORD_PATH";
const ENV_HTTP_RECORD_PATH_V08: &str = "OCL_V08_HTTP_RECORD_PATH";
const ENV_CASSETTE_JSONL_PATH_V08: &str = "OCL_V08_CASSETTE_JSONL_PATH";
const ENV_CASSETTE_INDEX_PATH_V08: &str = "OCL_V08_CASSETTE_INDEX_PATH";
const CASSETTE_MODE_REPLAY_V08: &str = "replay";

fn with_runtime_env_v14<R, F>(
    updates: Vec<(&'static str, Option<String>)>,
    op: F,
) -> Result<R, SdkError>
where
    F: FnOnce() -> Result<R, SdkError>,
{
    let mut previous = Vec::<(&'static str, Option<String>)>::with_capacity(updates.len());
    for (key, value) in &updates {
        previous.push((*key, env::var(key).ok()));
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }
    let result = op();
    for (key, value) in previous {
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }
    result
}

fn signature_with_lane_v14(payload: &str, lane: &str, cassette_hash: Option<&str>) -> String {
    if lane == "quarantine" {
        let canonical = format!(
            "lane={lane}\npayload={payload}\ncassette_hash={}\n",
            cassette_hash.unwrap_or_default()
        );
        return sha256_hex_v14(canonical.as_bytes());
    }
    fnv1a64_hex(&format!("lane={lane}\npayload={payload}"))
}

fn read_trimmed_file_v14(path: &Path) -> Result<String, SdkError> {
    Ok(fs::read_to_string(path)?.trim().to_string())
}

#[derive(Debug, Clone)]
struct ScenarioRuntimeContext {
    run_engine: RunEngine,
    runtime_mode: ReactorRuntimeMode,
    reactor_universe_id: Option<String>,
    reactor_domain_id: Option<String>,
}

fn resolve_runtime_context(
    app_root: &Path,
    options: &ConformanceRunOptionsV1,
) -> Result<ScenarioRuntimeContext, SdkError> {
    let universe = resolve_universe_v1(app_root, options.locked, options.universe_id.as_deref())?;
    let domain = resolve_domain_selection_v1(
        app_root,
        options.locked,
        options.universe_id.as_deref(),
        None,
    )?;
    enforce_universe_match_v1(
        &universe,
        Some(options.runtime_mode.as_str()),
        Some(options.engine.as_str()),
        None,
        options.locked,
    )?;

    let run_engine = if universe.is_legacy {
        options.engine
    } else {
        parse_run_engine(universe.engine.as_str())?
    };
    let runtime_mode = if universe.is_legacy {
        options.runtime_mode
    } else {
        parse_runtime_mode(universe.runtime_mode.as_str())?
    };
    let (reactor_universe_id, reactor_domain_id) = if domain.universe_id == "__legacy__" {
        (None, None)
    } else {
        (Some(domain.universe_id), Some(domain.domain_id))
    };

    Ok(ScenarioRuntimeContext {
        run_engine,
        runtime_mode,
        reactor_universe_id,
        reactor_domain_id,
    })
}

fn prepare_scenario_workspace_copy(
    runtime_root: &Path,
    source_root: &Path,
    scenario_name: &str,
    scenario_index: usize,
) -> Result<PathBuf, SdkError> {
    if !source_root.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-W9-SCENARIO-SOURCE-MISSING: source path not found `{}`",
            source_root.display()
        )));
    }
    let target = runtime_root.join("fixtures").join(format!(
        "{}-{:02}",
        sanitize_slug(scenario_name),
        scenario_index
    ));
    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    copy_tree(source_root, &target)?;
    Ok(target)
}

fn copy_runtime_registry(runtime_root: &Path, workspace_root: &Path) -> Result<PathBuf, SdkError> {
    let src = workspace_root.join("projects/ocp-ocl/registry");
    let dst = runtime_root.join("registry");
    if !src.exists() {
        return Ok(dst);
    }
    if dst.exists() {
        fs::remove_dir_all(&dst)?;
    }
    copy_tree(&src, &dst)?;
    Ok(dst)
}

fn conformance_runtime_root_v16(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("ocl")
        .join("w9")
        .join("runtime")
        .join(std::process::id().to_string())
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), SdkError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn sanitize_slug(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "scenario".to_string()
    } else {
        trimmed.to_string()
    }
}

fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}

fn normalize_lane_v14(raw: &str) -> String {
    match raw.trim() {
        "locked" | "locked_v071" => "locked_v071".to_string(),
        "locked_v06" => "locked_v06".to_string(),
        "quarantine" => "quarantine".to_string(),
        other => other.to_string(),
    }
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn sha256_hex_v14(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn json_escape(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn extract_error_code(text: &str) -> Option<String> {
    let line = text.trim();
    if line.starts_with('{') {
        if let Some(idx) = line.find("\"code\"") {
            let tail = &line[idx + 6..];
            if let Some(colon) = tail.find(':') {
                let after = tail[colon + 1..].trim_start();
                if let Some(rest) = after.strip_prefix('"') {
                    if let Some(end) = rest.find('"') {
                        let code = &rest[..end];
                        if looks_like_error_code(code) {
                            return Some(code.to_string());
                        }
                    }
                }
            }
        }
    }
    for token in line.split(|c: char| {
        c.is_whitespace()
            || c == ':'
            || c == ','
            || c == ';'
            || c == '('
            || c == ')'
            || c == '['
            || c == ']'
            || c == '{'
            || c == '}'
            || c == '"'
            || c == '`'
            || c == '|'
    }) {
        if looks_like_error_code(token) {
            return Some(token.to_string());
        }
    }
    None
}

fn looks_like_error_code(raw: &str) -> bool {
    if raw.len() < 3 || !raw.contains('-') {
        return false;
    }
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    raw.chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

fn parse_run_engine(raw: &str) -> Result<RunEngine, SdkError> {
    match raw {
        "interpreter" => Ok(RunEngine::Interpreter),
        "bytecode" => Ok(RunEngine::Bytecode),
        "dual" => Ok(RunEngine::Dual),
        _ => Err(SdkError::LockMismatch(format!(
            "V-UNIVERSE-ENGINE-UNSUPPORTED: unsupported engine `{raw}` in universe profile"
        ))),
    }
}

fn parse_runtime_mode(raw: &str) -> Result<ReactorRuntimeMode, SdkError> {
    match raw {
        "deterministic" => Ok(ReactorRuntimeMode::Deterministic),
        "throughput" => Ok(ReactorRuntimeMode::Throughput),
        _ => Err(SdkError::LockMismatch(format!(
            "V-UNIVERSE-RUNTIME-UNSUPPORTED: unsupported runtime_mode `{raw}` in universe profile"
        ))),
    }
}
