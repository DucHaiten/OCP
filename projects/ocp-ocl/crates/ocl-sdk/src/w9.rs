use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use ocl_runtime_core::RunEngine;

use crate::{
    build_project_with_lock, check_project_with_lock, compose_phenotype,
    run_project_with_engine_and_lock, run_reactor_service_with_lock, sync_deps_lock_v1,
    test_project_with_lock, verify_assembly, ReactorRuntimeMode, ReactorServiceOptions, SdkError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceScenarioV1 {
    pub name: String,
    pub path: String,
    pub reactor_ticks: u32,
    pub composer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceManifestV1 {
    pub scenarios: Vec<ConformanceScenarioV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConformanceRunOptionsV1 {
    pub locked: bool,
    pub engine: RunEngine,
    pub runtime_mode: ReactorRuntimeMode,
}

impl Default for ConformanceRunOptionsV1 {
    fn default() -> Self {
        Self {
            locked: true,
            engine: RunEngine::Dual,
            runtime_mode: ReactorRuntimeMode::Deterministic,
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

pub fn parse_conformance_manifest_v1(path: &Path) -> Result<ConformanceManifestV1, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut version_ok = false;
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
        let value = strip_quotes(v.trim()).to_string();

        if !in_scenario {
            if key == "version" && value == "1" {
                version_ok = true;
            }
            continue;
        }

        match key {
            "name" => current.name = Some(value),
            "path" => current.path = Some(value),
            "reactor_ticks" => {
                let ticks = value.parse::<u32>().map_err(|_| {
                    SdkError::MissingProject(
                        "invalid conformance manifest: `reactor_ticks` must be u32".to_string(),
                    )
                })?;
                current.reactor_ticks = Some(ticks);
            }
            "composer" => {
                let flag = match value.as_str() {
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
            _ => {}
        }
    }

    if in_scenario {
        scenarios.push(current.build()?);
    }

    if !version_ok {
        return Err(SdkError::MissingProject(
            "invalid conformance manifest: missing `version = 1`".to_string(),
        ));
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

    Ok(ConformanceManifestV1 { scenarios })
}

pub fn run_conformance_v1(
    workspace_root: &Path,
    manifest: &ConformanceManifestV1,
    options: ConformanceRunOptionsV1,
) -> ConformanceReportV1 {
    let mut results = Vec::new();

    for scenario in &manifest.scenarios {
        let app_root = resolve_scenario_root(workspace_root, &scenario.path);
        let outcome = run_scenario(&app_root, scenario, options);
        match outcome {
            Ok(()) => results.push(ConformanceScenarioResultV1 {
                name: scenario.name.clone(),
                path: scenario.path.clone(),
                ok: true,
                reason: None,
            }),
            Err(err) => results.push(ConformanceScenarioResultV1 {
                name: scenario.name.clone(),
                path: scenario.path.clone(),
                ok: false,
                reason: Some(err.to_string()),
            }),
        }
    }

    let scenarios_total = results.len();
    let scenarios_passed = results.iter().filter(|r| r.ok).count();
    let scenarios_failed = scenarios_total.saturating_sub(scenarios_passed);
    let required_digest = compute_conformance_required_digest(&results);
    let run_id = format!(
        "w9-{}",
        fnv1a64_hex(&format!(
            "{}|{}|{}",
            options.engine.as_str(),
            options.runtime_mode.as_str(),
            required_digest
        ))
    );

    ConformanceReportV1 {
        schema: "ocl.conformance.v1".to_string(),
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
    reactor_ticks: Option<u32>,
    composer: Option<bool>,
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
        Ok(ConformanceScenarioV1 {
            name,
            path,
            reactor_ticks: self.reactor_ticks.unwrap_or(0),
            composer: self.composer.unwrap_or(false),
        })
    }
}

fn resolve_scenario_root(workspace_root: &Path, scenario_path: &str) -> PathBuf {
    let path = PathBuf::from(scenario_path);
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn run_scenario(
    app_root: &Path,
    scenario: &ConformanceScenarioV1,
    options: ConformanceRunOptionsV1,
) -> Result<(), SdkError> {
    sync_deps_lock_v1(app_root)?;

    if scenario.composer {
        let phenotype_path = app_root.join("phenotype.toml");
        let registry_root = app_root.join("registry");
        compose_phenotype(app_root, &phenotype_path, &registry_root, options.locked)?;
        verify_assembly(app_root, &phenotype_path, &registry_root, options.locked)?;
    }

    check_project_with_lock(app_root, options.locked)?;
    run_project_with_engine_and_lock(app_root, options.engine, options.locked)?;

    if scenario.reactor_ticks > 0 {
        let reactor_options = ReactorServiceOptions {
            ticks: scenario.reactor_ticks,
            runtime_mode: options.runtime_mode,
            socket_listen: None,
            runtime_report: None,
            replay_audit: None,
        };
        run_reactor_service_with_lock(app_root, &reactor_options, options.locked)?;
    }

    test_project_with_lock(app_root, options.locked)?;
    build_project_with_lock(app_root, options.locked)?;
    Ok(())
}

fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
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
