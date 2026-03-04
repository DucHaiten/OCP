use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use ocl_runtime_core::{
    pretty_schema, schema_skeleton, CapabilityRegistry, KeyCapabilityKind, RunEngine,
    RuntimeCoreError,
};
use ocl_sdk::{
    build_oclpkg_with_lock, build_profile_from_trace, build_project_with_lock,
    build_run_id_deterministic, check_project_with_lock, compare_shadow_traces_v1,
    compose_phenotype, default_conformance_manifest_path, enforce_universe_match_v1,
    fetch_artifact, fmt_project, init_cosmos_v1, init_project, install_organs_v1,
    list_kits_from_cosmos_v1, parse_conformance_manifest_v1, parse_shadow_policy_v1,
    publish_artifact, read_profile_json, read_trace_jsonl, render_conformance_report_json,
    render_profile_view, render_trace_view, resolve_domain_selection_v1, resolve_universe_v1,
    resolve_view_selection_v1, run_artifact, run_conformance_v1, run_kit_doctor_v1,
    run_project_with_engine_and_lock, run_project_with_shadow_compare,
    run_project_with_trace_engine_and_lock, run_reactor_service_with_lock,
    run_reactor_service_with_shadow_compare, run_reactor_service_with_trace_engine_and_lock,
    sync_cosmos_lock_v1, sync_deps_lock_v1, sync_organs_lock_v1, sync_plugin_lock_v1,
    sync_policy_lock_v1, test_project_with_lock, trace_required_digest, verify_assembly,
    verify_organs_lock_v1, verify_plugin_lock_v1, verify_supply_artifact,
    write_conformance_report_json, write_profile_json, write_shadow_compare_artifacts_v1,
    write_trace_jsonl, ConformanceRunOptionsV1, InputEnvelopeV1, ProfileViewOptions,
    ReactorRuntimeMode, ReactorServiceOptions, SdkError, ShadowOptionsV1, TraceEventV1,
    TraceViewOptions,
};
use sha2::{Digest, Sha256};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = run_cli(&args);
    std::process::exit(code);
}

fn run_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_help();
        return 2;
    }

    match args[0].as_str() {
        "init" => {
            let Some(path) = args.get(1) else {
                eprintln!(
                    "usage: ocl init <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview] [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]"
                );
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            let template = parse_string_flag(args, "--template");
            let preset = parse_string_flag(args, "--preset");
            if template.is_some() && preset.is_some() {
                eprintln!("`--template` cannot be combined with `--preset`");
                return 2;
            }
            let locked = args.iter().any(|a| a == "--locked");
            let signer_id = parse_string_flag(args, "--signer-id");
            let sign_key = parse_string_flag(args, "--sign-key").map(PathBuf::from);
            let trust_store = parse_string_flag(args, "--trust-store").map(PathBuf::from);
            let registry = parse_string_flag(args, "--registry").map(PathBuf::from);
            let root = Path::new(path);

            match init_project(root) {
                Ok(layout) => {
                    if let Some(ref template_id) = template {
                        if let Err(err) = apply_init_template_v072(root, template_id) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                        if let Err(err) = sync_deps_lock_v1(root) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                    }
                    if let Some(ref preset_id) = preset {
                        if let Err(err) = apply_foundation_preset_v5(root, preset_id) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                        if let Err(err) = sync_deps_lock_v1(root) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                        if let Err(err) = sync_policy_lock_v1(root) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                        if let Err(err) = sync_cosmos_lock_v1(
                            root,
                            locked,
                            signer_id.as_deref(),
                            sign_key.as_deref(),
                            trust_store.as_deref(),
                        ) {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            return 1;
                        }
                        match list_kits_from_cosmos_v1(root) {
                            Ok(kits) => {
                                let requires_organs =
                                    kits.iter().any(|k| !k.required_organs.is_empty());
                                if requires_organs {
                                    let registry_path = registry.unwrap_or_else(|| {
                                        root.join("registry").join("organs").join("index.toml")
                                    });
                                    if let Err(err) = sync_organs_lock_v1(root, &registry_path) {
                                        if json_mode {
                                            println!("{}", error_to_json(&err));
                                        } else {
                                            eprintln!("{err}");
                                        }
                                        return 1;
                                    }
                                    if locked {
                                        if let Err(err) = verify_organs_lock_v1(root, true) {
                                            if json_mode {
                                                println!("{}", error_to_json(&err));
                                            } else {
                                                eprintln!("{err}");
                                            }
                                            return 1;
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                if json_mode {
                                    println!("{}", error_to_json(&err));
                                } else {
                                    eprintln!("{err}");
                                }
                                return 1;
                            }
                        }
                    }
                    if json_mode {
                        println!(
                            "{{\"ok\":true,\"root\":\"{}\",\"template\":{},\"preset\":{},\"locked\":{}}}",
                            json_escape(&layout.root.to_string_lossy()),
                            template
                                .as_ref()
                                .map(|v| format!("\"{}\"", json_escape(v)))
                                .unwrap_or_else(|| "null".to_string()),
                            preset
                                .as_ref()
                                .map(|v| format!("\"{}\"", json_escape(v)))
                                .unwrap_or_else(|| "null".to_string()),
                            if locked { "true" } else { "false" }
                        );
                    } else if let Some(template_id) = template {
                        println!(
                            "initialized {} (template={}, locked={})",
                            layout.root.display(),
                            template_id,
                            locked
                        );
                    } else if let Some(preset_id) = preset {
                        println!(
                            "initialized {} (preset={}, locked={})",
                            layout.root.display(),
                            preset_id,
                            locked
                        );
                    } else {
                        println!("initialized {}", layout.root.display());
                    }
                    0
                }
                Err(err) => {
                    if json_mode {
                        println!("{}", error_to_json(&err));
                    } else {
                        eprintln!("{err}");
                    }
                    1
                }
            }
        }
        "check" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl check <project_dir> [--json] [--locked] [--universe <id>]");
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            let locked = args.iter().any(|a| a == "--locked");
            let universe_id = parse_string_flag(args, "--universe");
            if let Err(err) = resolve_universe_v1(Path::new(path), locked, universe_id.as_deref()) {
                if json_mode {
                    println!("{}", error_to_json(&err));
                } else {
                    eprintln!("{err}");
                }
                return 1;
            }
            match check_project_with_lock(Path::new(path), locked) {
                Ok(summary) => {
                    if json_mode {
                        println!(
                            "{{\"ok\":true,\"files_checked\":{}}}",
                            summary.files_checked
                        );
                    } else {
                        println!("check ok (files_checked={})", summary.files_checked);
                    }
                    0
                }
                Err(err) => {
                    if json_mode {
                        println!("{}", error_to_json(&err));
                    } else {
                        eprintln!("{err}");
                    }
                    1
                }
            }
        }
        "run" => {
            let Some(path) = args.get(1) else {
                eprintln!(
                    "usage: ocl run <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
                );
                return 2;
            };
            let reactor_mode = args.iter().any(|a| a == "--reactor");
            let force_locked = args.iter().any(|a| a == "--locked");
            let shadow_options = match parse_shadow_args(args) {
                Ok(value) => value,
                Err(code) => return code,
            };
            let path_ref = Path::new(path);
            let path_is_artifact = path_ref
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| v.eq_ignore_ascii_case("oclpkg"))
                .unwrap_or(false);
            let lane = if path_is_artifact {
                "locked_v071".to_string()
            } else {
                let (lane, _) = read_lane_and_entry_for_v071(path_ref);
                lane
            };
            if !path_is_artifact {
                if let Err(err) = parse_lane_mode_v071(&lane) {
                    eprintln!("{err}");
                    return 1;
                }
                if let Err(err) = enforce_quarantine_lane_gate_v073(&lane) {
                    eprintln!("{err}");
                    return 1;
                }
            }
            let locked = if force_locked {
                true
            } else if path_is_artifact {
                false
            } else {
                locked_from_lane_v071(&lane)
            };
            let universe_id = parse_string_flag(args, "--universe");
            let domain_id = parse_string_flag(args, "--domain");
            let view_id = parse_string_flag(args, "--view");
            let explicit_engine = parse_string_flag(args, "--engine");
            let explicit_runtime_mode = parse_string_flag(args, "--runtime");
            if path_is_artifact && universe_id.is_some() {
                eprintln!("`--universe` is only supported when running a project directory");
                return 2;
            }
            if path_is_artifact && domain_id.is_some() {
                eprintln!("`--domain` is only supported when running a project directory");
                return 2;
            }
            if path_is_artifact && view_id.is_some() {
                eprintln!("`--view` is only supported when running a project directory");
                return 2;
            }
            if path_is_artifact && shadow_options.is_some() {
                eprintln!("`--shadow` is only supported when running a project directory");
                return 2;
            }
            let universe = if path_is_artifact {
                None
            } else {
                match resolve_universe_v1(path_ref, locked, universe_id.as_deref()) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        eprintln!("{err}");
                        return 1;
                    }
                }
            };
            let domain = if path_is_artifact {
                None
            } else {
                match resolve_domain_selection_v1(
                    path_ref,
                    locked,
                    universe_id.as_deref(),
                    domain_id.as_deref(),
                ) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        eprintln!("{err}");
                        return 1;
                    }
                }
            };
            let view = if path_is_artifact {
                None
            } else {
                match resolve_view_selection_v1(
                    path_ref,
                    locked,
                    universe_id.as_deref(),
                    domain_id.as_deref(),
                    view_id.as_deref(),
                ) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        eprintln!("{err}");
                        return 1;
                    }
                }
            };
            if let Some(universe) = &universe {
                if let Err(err) = enforce_universe_match_v1(
                    universe,
                    explicit_runtime_mode.as_deref(),
                    explicit_engine.as_deref(),
                    None,
                    locked,
                ) {
                    eprintln!("{err}");
                    return 1;
                }
            }

            let engine_raw = if reactor_mode {
                if shadow_options.is_some() {
                    explicit_engine
                        .clone()
                        .unwrap_or_else(|| "dual".to_string())
                } else {
                    explicit_engine
                        .clone()
                        .unwrap_or_else(|| "interpreter".to_string())
                }
            } else if explicit_engine.is_none() {
                if let Some(universe) = &universe {
                    if !universe.is_legacy {
                        universe.engine.clone()
                    } else {
                        "interpreter".to_string()
                    }
                } else {
                    "interpreter".to_string()
                }
            } else {
                explicit_engine
                    .clone()
                    .unwrap_or_else(|| "interpreter".to_string())
            };
            let run_engine = match engine_raw.as_str() {
                "interpreter" => RunEngine::Interpreter,
                "bytecode" => RunEngine::Bytecode,
                "dual" => RunEngine::Dual,
                _ => {
                    eprintln!(
                        "invalid engine mode: `{engine_raw}` (expected interpreter|bytecode|dual)"
                    );
                    return 2;
                }
            };
            let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
            let runtime_mode_raw = if explicit_runtime_mode.is_none() && universe.is_some() {
                universe
                    .as_ref()
                    .map(|v| v.runtime_mode.clone())
                    .unwrap_or_else(|| "deterministic".to_string())
            } else {
                explicit_runtime_mode
                    .clone()
                    .unwrap_or_else(|| "deterministic".to_string())
            };
            let runtime_mode = match runtime_mode_raw.as_str() {
                "deterministic" => ReactorRuntimeMode::Deterministic,
                "throughput" => ReactorRuntimeMode::Throughput,
                _ => {
                    eprintln!("invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)");
                    return 2;
                }
            };
            let socket_listen = parse_string_flag(args, "--socket-listen");
            let runtime_report = parse_string_flag(args, "--runtime-report").map(PathBuf::from);
            let replay_audit = parse_string_flag(args, "--replay-audit").map(PathBuf::from);

            if !reactor_mode
                && (socket_listen.is_some() || runtime_report.is_some() || replay_audit.is_some())
            {
                eprintln!(
                    "`--socket-listen`, `--runtime-report`, and `--replay-audit` require `--reactor`"
                );
                return 2;
            }
            if reactor_mode && explicit_engine.is_some() && shadow_options.is_none() {
                eprintln!("`--engine` currently supports non-reactor `run` only");
                return 2;
            }
            with_view_env(view.as_ref().map(|v| v.view_id.as_str()), || {
                let mut runtime_updates = vec![
                    (ENV_PROJECT_LANE_V08, Some(lane.clone())),
                    (
                        ENV_QUARANTINE_MODE_V08,
                        if is_quarantine_lane_v08(&lane) {
                            Some(CASSETTE_MODE_RECORD_V08.to_string())
                        } else {
                            None
                        },
                    ),
                    (ENV_WALLCLOCK_RECORD_PATH_V08, None),
                    (ENV_PROC_RECORD_PATH_V08, None),
                    (ENV_HTTP_RECORD_PATH_V08, None),
                    (ENV_CASSETTE_JSONL_PATH_V08, None),
                    (ENV_CASSETTE_INDEX_PATH_V08, None),
                ];
                runtime_updates.extend(fs_runtime_env_updates_v08(path_ref));
                runtime_updates.extend(proc_runtime_env_updates_v08(path_ref));
                runtime_updates.extend(net_http_runtime_env_updates_v08(path_ref));
                with_runtime_env_v08(runtime_updates, || {
                    if reactor_mode {
                        let options = ReactorServiceOptions {
                            ticks,
                            runtime_mode,
                            socket_listen: socket_listen.clone(),
                            runtime_report: runtime_report.clone(),
                            replay_audit: replay_audit.clone(),
                            hive_caps: None,
                            io_tape_record_path: None,
                            io_tape_replay_path: None,
                            universe_id: domain.as_ref().and_then(|v| {
                                if v.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(v.universe_id.clone())
                                }
                            }),
                            domain_id: domain.as_ref().and_then(|v| {
                                if v.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(v.domain_id.clone())
                                }
                            }),
                        };
                        if let Some(shadow_cfg) = shadow_options.as_ref() {
                            match run_reactor_service_with_shadow_compare(
                                Path::new(path),
                                &options,
                                run_engine,
                                locked,
                                shadow_cfg,
                            ) {
                                Ok(summary) => {
                                    println!(
                                    "reactor run ok (mode={}, ticks={}, events={}, total_steps={}, backpressure={}, universe={}, domain={}, shadow={}, shadow_digest={})",
                                    summary.reactor.mode.as_str(),
                                    summary.reactor.ticks,
                                    summary.reactor.event_count,
                                    summary.reactor.total_steps,
                                    summary.reactor.backpressure_count,
                                    domain
                                        .as_ref()
                                        .map(|v| v.universe_id.as_str())
                                        .unwrap_or("__legacy__"),
                                    domain
                                        .as_ref()
                                        .map(|v| v.domain_id.as_str())
                                        .unwrap_or("default"),
                                    summary.compare.shadow_id,
                                    summary.compare.main_required_digest
                                );
                                    println!(
                                        "shadow report written: {}",
                                        summary.artifacts.report_path.display()
                                    );
                                    if let Some(path) = &runtime_report {
                                        println!("runtime report written: {}", path.display());
                                    }
                                    if let Some(path) = &replay_audit {
                                        println!("replay audit written: {}", path.display());
                                    }
                                    0
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    exit_code_for_sdk_error(&err)
                                }
                            }
                        } else {
                            match run_reactor_service_with_lock(Path::new(path), &options, locked) {
                                Ok(summary) => {
                                    println!(
                                    "reactor run ok (mode={}, ticks={}, events={}, total_steps={}, backpressure={}, universe={}, domain={})",
                                    summary.mode.as_str(),
                                    summary.ticks,
                                    summary.event_count,
                                    summary.total_steps,
                                    summary.backpressure_count,
                                    domain
                                        .as_ref()
                                        .map(|v| v.universe_id.as_str())
                                        .unwrap_or("__legacy__"),
                                    domain
                                        .as_ref()
                                        .map(|v| v.domain_id.as_str())
                                        .unwrap_or("default")
                                );
                                    if let Some(path) = &runtime_report {
                                        println!("runtime report written: {}", path.display());
                                    }
                                    if let Some(path) = &replay_audit {
                                        println!("replay audit written: {}", path.display());
                                    }
                                    0
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    1
                                }
                            }
                        }
                    } else if path_ref
                        .extension()
                        .and_then(|v| v.to_str())
                        .map(|v| v.eq_ignore_ascii_case("oclpkg"))
                        .unwrap_or(false)
                    {
                        match run_artifact(path_ref, run_engine, 4096) {
                            Ok(summary) => {
                                println!(
                                    "run artifact ok (engine={}, steps={})",
                                    run_engine.as_str(),
                                    summary.steps
                                );
                                0
                            }
                            Err(err) => {
                                eprintln!("{err}");
                                1
                            }
                        }
                    } else if let Some(shadow_cfg) = shadow_options.as_ref() {
                        match run_project_with_shadow_compare(
                            path_ref, run_engine, locked, shadow_cfg,
                        ) {
                            Ok(summary) => {
                                let artifact_dir = match emit_v071_artifacts_for_project_run(
                                    path_ref, run_engine, locked, None,
                                ) {
                                    Ok(path) => path,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        return 1;
                                    }
                                };
                                println!(
                                "run ok (engine={}, steps={}, universe={}, domain={}, shadow={}, shadow_digest={})",
                                run_engine.as_str(),
                                summary.steps,
                                domain
                                    .as_ref()
                                    .map(|v| v.universe_id.as_str())
                                    .unwrap_or("__legacy__"),
                                domain
                                    .as_ref()
                                    .map(|v| v.domain_id.as_str())
                                    .unwrap_or("default"),
                                summary.compare.shadow_id,
                                summary.compare.main_required_digest
                            );
                                println!(
                                    "shadow report written: {}",
                                    summary.artifacts.report_path.display()
                                );
                                println!("v0.7.1 artifacts written: {}", artifact_dir.display());
                                0
                            }
                            Err(err) => {
                                eprintln!("{err}");
                                if let Ok(path) = emit_v071_artifacts_for_project_run(
                                    path_ref,
                                    run_engine,
                                    locked,
                                    Some(&err),
                                ) {
                                    println!("v0.7.1 artifacts written: {}", path.display());
                                }
                                exit_code_for_sdk_error(&err)
                            }
                        }
                    } else {
                        match run_project_with_engine_and_lock(path_ref, run_engine, locked) {
                            Ok(summary) => {
                                let artifact_dir = match emit_v071_artifacts_for_project_run(
                                    path_ref, run_engine, locked, None,
                                ) {
                                    Ok(path) => path,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        return 1;
                                    }
                                };
                                println!(
                                    "run ok (engine={}, steps={}, universe={}, domain={})",
                                    run_engine.as_str(),
                                    summary.steps,
                                    domain
                                        .as_ref()
                                        .map(|v| v.universe_id.as_str())
                                        .unwrap_or("__legacy__"),
                                    domain
                                        .as_ref()
                                        .map(|v| v.domain_id.as_str())
                                        .unwrap_or("default")
                                );
                                println!("v0.7.1 artifacts written: {}", artifact_dir.display());
                                0
                            }
                            Err(err) => {
                                eprintln!("{err}");
                                if let Ok(path) = emit_v071_artifacts_for_project_run(
                                    path_ref,
                                    run_engine,
                                    locked,
                                    Some(&err),
                                ) {
                                    println!("v0.7.1 artifacts written: {}", path.display());
                                }
                                1
                            }
                        }
                    }
                })
            })
        }
        "replay" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl replay <artifact_dir>");
                return 2;
            };
            match replay_v071(Path::new(path)) {
                Ok(signature) => {
                    println!("replay ok (signature match: {signature})");
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "trace" => {
            let Some(subcmd) = args.get(1).map(String::as_str) else {
                eprintln!(
                    "usage: ocl trace <run|view> ...\n  run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]\n  view <trace_file> [--tail N] [--json]"
                );
                return 2;
            };
            match subcmd {
                "run" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl trace run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let reactor_mode = args.iter().any(|a| a == "--reactor");
                    let shadow_options = match parse_shadow_args(args) {
                        Ok(value) => value,
                        Err(code) => return code,
                    };
                    let universe_id = parse_string_flag(args, "--universe");
                    let domain_id = parse_string_flag(args, "--domain");
                    let view_id = parse_string_flag(args, "--view");
                    let out_path = parse_string_flag(args, "--out")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("target/ocl/w5/trace.latest.jsonl"));
                    let engine_raw = parse_string_flag(args, "--engine")
                        .unwrap_or_else(|| "interpreter".to_string());
                    let run_engine = match engine_raw.as_str() {
                        "interpreter" => RunEngine::Interpreter,
                        "bytecode" => RunEngine::Bytecode,
                        "dual" => RunEngine::Dual,
                        _ => {
                            eprintln!(
                                "invalid engine mode: `{engine_raw}` (expected interpreter|bytecode|dual)"
                            );
                            return 2;
                        }
                    };
                    let domain_selection = match resolve_domain_selection_v1(
                        Path::new(path),
                        locked,
                        universe_id.as_deref(),
                        domain_id.as_deref(),
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("{err}");
                            return 1;
                        }
                    };
                    let view_selection = match resolve_view_selection_v1(
                        Path::new(path),
                        locked,
                        universe_id.as_deref(),
                        domain_id.as_deref(),
                        view_id.as_deref(),
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("{err}");
                            return 1;
                        }
                    };

                    with_view_env(Some(view_selection.view_id.as_str()), || {
                        let (runtime_mode, result) = if reactor_mode {
                            let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
                            let runtime_mode_raw = parse_string_flag(args, "--runtime")
                                .unwrap_or_else(|| "deterministic".to_string());
                            let runtime_mode = match runtime_mode_raw.as_str() {
                                "deterministic" => ReactorRuntimeMode::Deterministic,
                                "throughput" => ReactorRuntimeMode::Throughput,
                                _ => {
                                    eprintln!("invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)");
                                    return 2;
                                }
                            };
                            let options = ReactorServiceOptions {
                                ticks,
                                runtime_mode,
                                socket_listen: None,
                                runtime_report: None,
                                replay_audit: None,
                                hive_caps: None,
                                io_tape_record_path: None,
                                io_tape_replay_path: None,
                                universe_id: if domain_selection.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(domain_selection.universe_id.clone())
                                },
                                domain_id: if domain_selection.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(domain_selection.domain_id.clone())
                                },
                            };
                            (
                                runtime_mode,
                                run_reactor_service_with_trace_engine_and_lock(
                                    Path::new(path),
                                    &options,
                                    run_engine,
                                    locked,
                                ),
                            )
                        } else {
                            (
                                ReactorRuntimeMode::Deterministic,
                                run_project_with_trace_engine_and_lock(
                                    Path::new(path),
                                    run_engine,
                                    locked,
                                ),
                            )
                        };

                        match result {
                            Ok(mut summary) => {
                                if let Some(shadow_cfg) = shadow_options.as_ref() {
                                    let shadow_result = if reactor_mode {
                                        let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
                                        let options = ReactorServiceOptions {
                                            ticks,
                                            runtime_mode,
                                            socket_listen: None,
                                            runtime_report: None,
                                            replay_audit: None,
                                            hive_caps: None,
                                            io_tape_record_path: None,
                                            io_tape_replay_path: None,
                                            universe_id: if domain_selection.universe_id
                                                == "__legacy__"
                                            {
                                                None
                                            } else {
                                                Some(domain_selection.universe_id.clone())
                                            },
                                            domain_id: if domain_selection.universe_id
                                                == "__legacy__"
                                            {
                                                None
                                            } else {
                                                Some(domain_selection.domain_id.clone())
                                            },
                                        };
                                        run_reactor_service_with_trace_engine_and_lock(
                                            Path::new(path),
                                            &options,
                                            run_engine,
                                            locked,
                                        )
                                    } else {
                                        run_project_with_trace_engine_and_lock(
                                            Path::new(path),
                                            run_engine,
                                            locked,
                                        )
                                    };
                                    let shadow_summary = match shadow_result {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };

                                    let envelope = InputEnvelopeV1 {
                                        universe_id: domain_selection.universe_id.clone(),
                                        domain_id: domain_selection.domain_id.clone(),
                                        shadow_id: shadow_cfg.shadow_id.clone(),
                                        runtime_mode: runtime_mode.as_str().to_string(),
                                        engine: run_engine.as_str().to_string(),
                                        run_kind: "trace".to_string(),
                                    };
                                    let report = match compare_shadow_traces_v1(
                                        &summary.events,
                                        &shadow_summary.events,
                                        &envelope,
                                        shadow_cfg.policy,
                                    ) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };
                                    let artifacts = match write_shadow_compare_artifacts_v1(
                                        Path::new(path),
                                        "shadow_compare.trace",
                                        &report,
                                        shadow_cfg.report_dir.as_deref(),
                                    ) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };
                                    if !report.matched {
                                        eprintln!(
                                            "{}",
                                            report
                                                .mismatch_reason
                                                .as_deref()
                                                .unwrap_or("V-SHADOW-MISMATCH")
                                        );
                                        return 5;
                                    }
                                    println!(
                                        "shadow trace compare ok (shadow={}, digest={}, report={})",
                                        report.shadow_id,
                                        report.main_required_digest,
                                        artifacts.report_path.display()
                                    );
                                }

                                for event in &mut summary.events {
                                    event.universe_id = domain_selection.universe_id.clone();
                                    event.domain_id = domain_selection.domain_id.clone();
                                }
                                if let Err(err) = write_trace_jsonl(&out_path, &summary.events) {
                                    eprintln!("{err}");
                                    return 1;
                                }
                                println!(
                                    "trace run ok (events={}, steps={}, run_id={}, universe={}, domain={}, out={})",
                                    summary.events.len(),
                                    summary.total_steps,
                                    summary.run_id,
                                    domain_selection.universe_id,
                                    domain_selection.domain_id,
                                    out_path.display()
                                );
                                0
                            }
                            Err(err) => {
                                eprintln!("{err}");
                                1
                            }
                        }
                    })
                }
                "view" => {
                    let Some(trace_path) = args.get(2) else {
                        eprintln!("usage: ocl trace view <trace_file> [--tail N] [--json]");
                        return 2;
                    };
                    let tail = parse_u32_flag(args, "--tail").map(|v| v as usize);
                    let json_mode = args.iter().any(|a| a == "--json");
                    match read_trace_jsonl(Path::new(trace_path)) {
                        Ok(events) => {
                            let report = render_trace_view(
                                &events,
                                TraceViewOptions {
                                    tail,
                                    json: json_mode,
                                },
                            );
                            println!("{}", report.rendered);
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl trace <run|view> ...");
                    2
                }
            }
        }
        "profile" => {
            let Some(subcmd) = args.get(1).map(String::as_str) else {
                eprintln!(
                    "usage: ocl profile <run|view> ...\n  run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]\n  view <profile_file> [--top N] [--json]"
                );
                return 2;
            };
            match subcmd {
                "run" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl profile run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let reactor_mode = args.iter().any(|a| a == "--reactor");
                    let shadow_options = match parse_shadow_args(args) {
                        Ok(value) => value,
                        Err(code) => return code,
                    };
                    let universe_id = parse_string_flag(args, "--universe");
                    let domain_id = parse_string_flag(args, "--domain");
                    let view_id = parse_string_flag(args, "--view");
                    let out_path = parse_string_flag(args, "--out")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("target/ocl/w5/profile.latest.json"));
                    let engine_raw = parse_string_flag(args, "--engine")
                        .unwrap_or_else(|| "interpreter".to_string());
                    let run_engine = match engine_raw.as_str() {
                        "interpreter" => RunEngine::Interpreter,
                        "bytecode" => RunEngine::Bytecode,
                        "dual" => RunEngine::Dual,
                        _ => {
                            eprintln!(
                                "invalid engine mode: `{engine_raw}` (expected interpreter|bytecode|dual)"
                            );
                            return 2;
                        }
                    };
                    let domain_selection = match resolve_domain_selection_v1(
                        Path::new(path),
                        locked,
                        universe_id.as_deref(),
                        domain_id.as_deref(),
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("{err}");
                            return 1;
                        }
                    };
                    let view_selection = match resolve_view_selection_v1(
                        Path::new(path),
                        locked,
                        universe_id.as_deref(),
                        domain_id.as_deref(),
                        view_id.as_deref(),
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("{err}");
                            return 1;
                        }
                    };

                    with_view_env(Some(view_selection.view_id.as_str()), || {
                        let (runtime_mode, trace_result) = if reactor_mode {
                            let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
                            let runtime_mode_raw = parse_string_flag(args, "--runtime")
                                .unwrap_or_else(|| "deterministic".to_string());
                            let runtime_mode = match runtime_mode_raw.as_str() {
                                "deterministic" => ReactorRuntimeMode::Deterministic,
                                "throughput" => ReactorRuntimeMode::Throughput,
                                _ => {
                                    eprintln!("invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)");
                                    return 2;
                                }
                            };
                            let options = ReactorServiceOptions {
                                ticks,
                                runtime_mode,
                                socket_listen: None,
                                runtime_report: None,
                                replay_audit: None,
                                hive_caps: None,
                                io_tape_record_path: None,
                                io_tape_replay_path: None,
                                universe_id: if domain_selection.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(domain_selection.universe_id.clone())
                                },
                                domain_id: if domain_selection.universe_id == "__legacy__" {
                                    None
                                } else {
                                    Some(domain_selection.domain_id.clone())
                                },
                            };
                            (
                                runtime_mode,
                                run_reactor_service_with_trace_engine_and_lock(
                                    Path::new(path),
                                    &options,
                                    run_engine,
                                    locked,
                                ),
                            )
                        } else {
                            (
                                ReactorRuntimeMode::Deterministic,
                                run_project_with_trace_engine_and_lock(
                                    Path::new(path),
                                    run_engine,
                                    locked,
                                ),
                            )
                        };

                        match trace_result {
                            Ok(mut trace_summary) => {
                                if let Some(shadow_cfg) = shadow_options.as_ref() {
                                    let shadow_result = if reactor_mode {
                                        let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
                                        let options = ReactorServiceOptions {
                                            ticks,
                                            runtime_mode,
                                            socket_listen: None,
                                            runtime_report: None,
                                            replay_audit: None,
                                            hive_caps: None,
                                            io_tape_record_path: None,
                                            io_tape_replay_path: None,
                                            universe_id: if domain_selection.universe_id
                                                == "__legacy__"
                                            {
                                                None
                                            } else {
                                                Some(domain_selection.universe_id.clone())
                                            },
                                            domain_id: if domain_selection.universe_id
                                                == "__legacy__"
                                            {
                                                None
                                            } else {
                                                Some(domain_selection.domain_id.clone())
                                            },
                                        };
                                        run_reactor_service_with_trace_engine_and_lock(
                                            Path::new(path),
                                            &options,
                                            run_engine,
                                            locked,
                                        )
                                    } else {
                                        run_project_with_trace_engine_and_lock(
                                            Path::new(path),
                                            run_engine,
                                            locked,
                                        )
                                    };
                                    let shadow_summary = match shadow_result {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };

                                    let envelope = InputEnvelopeV1 {
                                        universe_id: domain_selection.universe_id.clone(),
                                        domain_id: domain_selection.domain_id.clone(),
                                        shadow_id: shadow_cfg.shadow_id.clone(),
                                        runtime_mode: runtime_mode.as_str().to_string(),
                                        engine: run_engine.as_str().to_string(),
                                        run_kind: "profile".to_string(),
                                    };
                                    let report = match compare_shadow_traces_v1(
                                        &trace_summary.events,
                                        &shadow_summary.events,
                                        &envelope,
                                        shadow_cfg.policy,
                                    ) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };
                                    let artifacts = match write_shadow_compare_artifacts_v1(
                                        Path::new(path),
                                        "shadow_compare.profile",
                                        &report,
                                        shadow_cfg.report_dir.as_deref(),
                                    ) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            return exit_code_for_sdk_error(&err);
                                        }
                                    };
                                    if !report.matched {
                                        eprintln!(
                                            "{}",
                                            report
                                                .mismatch_reason
                                                .as_deref()
                                                .unwrap_or("V-SHADOW-MISMATCH")
                                        );
                                        return 5;
                                    }
                                    println!(
                                        "shadow profile compare ok (shadow={}, digest={}, report={})",
                                        report.shadow_id,
                                        report.main_required_digest,
                                        artifacts.report_path.display()
                                    );
                                }

                                for event in &mut trace_summary.events {
                                    event.universe_id = domain_selection.universe_id.clone();
                                    event.domain_id = domain_selection.domain_id.clone();
                                }
                                let report = build_profile_from_trace(
                                    &trace_summary.run_id,
                                    trace_summary.total_steps,
                                    &trace_summary.events,
                                );
                                if let Err(err) = write_profile_json(&out_path, &report) {
                                    eprintln!("{err}");
                                    return 1;
                                }
                                println!(
                                    "profile run ok (events={}, observe={}, digest={}, universe={}, domain={}, out={})",
                                    report.event_count,
                                    report.observe_count,
                                    report.required_digest,
                                    domain_selection.universe_id,
                                    domain_selection.domain_id,
                                    out_path.display()
                                );
                                0
                            }
                            Err(err) => {
                                eprintln!("{err}");
                                1
                            }
                        }
                    })
                }
                "view" => {
                    let Some(profile_path) = args.get(2) else {
                        eprintln!("usage: ocl profile view <profile_file> [--top N] [--json]");
                        return 2;
                    };
                    let top = parse_u32_flag(args, "--top").unwrap_or(10) as usize;
                    let json_mode = args.iter().any(|a| a == "--json");
                    match read_profile_json(Path::new(profile_path)) {
                        Ok(report) => {
                            let rendered = render_profile_view(
                                &report,
                                ProfileViewOptions {
                                    top,
                                    json: json_mode,
                                },
                            );
                            println!("{rendered}");
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl profile <run|view> ...");
                    2
                }
            }
        }
        "fmt" => {
            let (path, check_only) = parse_path_with_flag(args, "--check");
            let Some(path) = path else {
                eprintln!("usage: ocl fmt <project_dir> [--check]");
                return 2;
            };
            match fmt_project(&path, check_only) {
                Ok(summary) => {
                    if check_only {
                        println!("fmt check ok (files_checked)");
                    } else {
                        println!("fmt ok (files_touched={})", summary.files_touched);
                    }
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "test" => {
            if args.iter().any(|a| a == "--conformance") {
                let locked = args.iter().any(|a| a == "--locked");
                let json_mode = args.iter().any(|a| a == "--json");
                let universe_id = parse_string_flag(args, "--universe");
                let manifest_path = parse_string_flag(args, "--manifest")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| default_conformance_manifest_path(Path::new(".")));
                let out_path = parse_string_flag(args, "--out")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        PathBuf::from("target/ocl/w9/reports/conformance_report.json")
                    });
                let runtime_mode_raw = parse_string_flag(args, "--runtime")
                    .unwrap_or_else(|| "deterministic".to_string());
                let runtime_mode = match runtime_mode_raw.as_str() {
                    "deterministic" => ReactorRuntimeMode::Deterministic,
                    "throughput" => ReactorRuntimeMode::Throughput,
                    _ => {
                        eprintln!("invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)");
                        return 2;
                    }
                };
                let engine_raw =
                    parse_string_flag(args, "--engine").unwrap_or_else(|| "dual".to_string());
                let run_engine = match engine_raw.as_str() {
                    "interpreter" => RunEngine::Interpreter,
                    "bytecode" => RunEngine::Bytecode,
                    "dual" => RunEngine::Dual,
                    _ => {
                        eprintln!(
                            "invalid engine mode: `{engine_raw}` (expected interpreter|bytecode|dual)"
                        );
                        return 2;
                    }
                };

                let trust_store = parse_string_flag(args, "--trust-store").map(PathBuf::from);
                let signer_id = parse_string_flag(args, "--signer-id");
                let sign_key = parse_string_flag(args, "--sign-key")
                    .map(PathBuf::from)
                    .or_else(|| std::env::var("OCL_SIGN_KEY_PATH").ok().map(PathBuf::from));

                if locked {
                    if sign_key.is_none() {
                        eprintln!(
                            "W9-CONFORMANCE-SIGN-REQUIRED: locked conformance requires --sign-key or OCL_SIGN_KEY_PATH"
                        );
                        return 12;
                    }
                    if let Some(path) = &sign_key {
                        if !path.exists() {
                            eprintln!(
                                "W9-CONFORMANCE-SIGN-MISSING: sign key not found: {}",
                                path.display()
                            );
                            return 12;
                        }
                    }
                    if let Some(path) = &trust_store {
                        if !path.exists() {
                            eprintln!(
                                "W9-CONFORMANCE-TRUST-STORE-MISSING: trust store not found: {}",
                                path.display()
                            );
                            return 12;
                        }
                    }
                    if signer_id.is_none() {
                        eprintln!(
                            "W9-CONFORMANCE-SIGNER-REQUIRED: locked conformance requires --signer-id"
                        );
                        return 12;
                    }
                }

                let manifest = match parse_conformance_manifest_v1(&manifest_path) {
                    Ok(value) => value,
                    Err(err) => {
                        if json_mode {
                            println!("{}", error_to_json(&err));
                        } else {
                            eprintln!("{err}");
                        }
                        return 9;
                    }
                };

                let report = run_conformance_v1(
                    Path::new("."),
                    &manifest,
                    ConformanceRunOptionsV1 {
                        locked,
                        engine: run_engine,
                        runtime_mode,
                        universe_id,
                    },
                );

                if let Err(err) = write_conformance_report_json(&out_path, &report) {
                    if json_mode {
                        println!("{}", error_to_json(&err));
                    } else {
                        eprintln!("{err}");
                    }
                    return 11;
                }

                if json_mode {
                    println!("{}", render_conformance_report_json(&report));
                } else {
                    println!(
                        "conformance done (run_id={}, total={}, pass={}, fail={}, digest={}, out={})",
                        report.run_id,
                        report.scenarios_total,
                        report.scenarios_passed,
                        report.scenarios_failed,
                        report.required_digest,
                        out_path.display()
                    );
                }

                if report.scenarios_failed > 0 {
                    10
                } else {
                    0
                }
            } else {
                let Some(path) = args.get(1) else {
                    eprintln!(
                        "usage: ocl test <project_dir> [--locked] [--universe <id>] [--domain <id>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--golden <dir>] [--clean]"
                    );
                    return 2;
                };
                let locked = args.iter().any(|a| a == "--locked");
                let clean = args.iter().any(|a| a == "--clean");
                let golden = parse_string_flag(args, "--golden");
                let shadow_options = match parse_shadow_args(args) {
                    Ok(value) => value,
                    Err(code) => return code,
                };
                let universe_id = parse_string_flag(args, "--universe");
                let domain_id = parse_string_flag(args, "--domain");
                if clean {
                    if let Err(err) = clean_v072_test_outputs(Path::new(path)) {
                        eprintln!("{err}");
                        return 1;
                    }
                }
                if let Err(err) =
                    resolve_universe_v1(Path::new(path), locked, universe_id.as_deref())
                {
                    eprintln!("{err}");
                    return 1;
                }
                if let Err(err) = resolve_domain_selection_v1(
                    Path::new(path),
                    locked,
                    universe_id.as_deref(),
                    domain_id.as_deref(),
                ) {
                    eprintln!("{err}");
                    return 1;
                }
                match test_project_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        let artifact_dir = match emit_v071_artifacts_for_project_run(
                            Path::new(path),
                            RunEngine::Dual,
                            locked,
                            None,
                        ) {
                            Ok(value) => value,
                            Err(err) => {
                                eprintln!("{err}");
                                return exit_code_for_sdk_error(&err);
                            }
                        };
                        if let Err(err) =
                            emit_v072_io_replay_metadata(Path::new(path), &artifact_dir)
                        {
                            eprintln!("{err}");
                            return 1;
                        }
                        if let Err(err) = materialize_v072_fixture_output(Path::new(path)) {
                            eprintln!("{err}");
                            return 1;
                        }
                        if let Some(golden_dir) = golden.as_deref() {
                            if let Err(msg) =
                                compare_v072_golden_outputs(Path::new(path), golden_dir)
                            {
                                eprintln!("{msg}");
                                return 1;
                            }
                        }
                        if let Some(shadow_cfg) = shadow_options.as_ref() {
                            let shadow_run = run_project_with_shadow_compare(
                                Path::new(path),
                                RunEngine::Dual,
                                locked,
                                shadow_cfg,
                            );
                            match shadow_run {
                                Ok(shadow_summary) => {
                                    println!(
                                        "shadow test compare ok (shadow={}, digest={}, report={})",
                                        shadow_summary.compare.shadow_id,
                                        shadow_summary.compare.main_required_digest,
                                        shadow_summary.artifacts.report_path.display()
                                    );
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    return exit_code_for_sdk_error(&err);
                                }
                            }
                        }
                        println!(
                            "test ok (tests_run={}, artifacts={})",
                            summary.tests_run,
                            artifact_dir.display()
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            }
        }
        "build" => {
            let Some(path) = args.get(1) else {
                eprintln!(
                    "usage: ocl build <project_dir> [--locked] [--source-only] [--universe <id>]"
                );
                return 2;
            };
            let locked = args.iter().any(|a| a == "--locked");
            let source_only = args.iter().any(|a| a == "--source-only");
            let universe_id = parse_string_flag(args, "--universe");
            if let Err(err) = resolve_universe_v1(Path::new(path), locked, universe_id.as_deref()) {
                eprintln!("{err}");
                return 1;
            }
            if source_only {
                match build_project_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        println!(
                            "build source-only ok (files_bundled={})",
                            summary.files_bundled
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                match build_oclpkg_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        println!(
                            "build ok (.oclpkg={}, files_bundled={}, payload_hash_blake3={})",
                            summary.artifact_path.display(),
                            summary.files_bundled,
                            summary.payload_hash_blake3
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            }
        }
        "publish" => {
            let Some(artifact) = args.get(1) else {
                eprintln!("usage: ocl publish <artifact.oclpkg> [--registry <dir>]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            match publish_artifact(Path::new(artifact), Path::new(&registry)) {
                Ok(summary) => {
                    println!(
                        "publish ok (artifact={}, package={}, hash={})",
                        summary.artifact_path.display(),
                        summary.package_name,
                        summary.payload_hash_blake3
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "fetch" => {
            let Some(artifact_or_package) = args.get(1) else {
                eprintln!("usage: ocl fetch <artifact|package> [--registry <dir>] [--out <dir>]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            let out = parse_string_flag(args, "--out").unwrap_or_else(|| "fetched".to_string());
            match fetch_artifact(artifact_or_package, Path::new(&registry), Path::new(&out)) {
                Ok(summary) => {
                    println!(
                        "fetch ok (artifact={}, package={})",
                        summary.artifact_path.display(),
                        summary.package_name
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "verify-supply" => {
            let Some(artifact) = args.get(1) else {
                eprintln!("usage: ocl verify-supply <artifact.oclpkg>");
                return 2;
            };
            match verify_supply_artifact(Path::new(artifact)) {
                Ok(summary) => {
                    println!(
                        "verify-supply ok (package={}, hash={})",
                        summary.package_name, summary.payload_hash_blake3
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "lock" => {
            if args.get(1).map(String::as_str) != Some("sync") {
                eprintln!("usage: ocl lock sync <project_dir>");
                return 2;
            }
            let Some(path) = args.get(2) else {
                eprintln!("usage: ocl lock sync <project_dir>");
                return 2;
            };
            match sync_deps_lock_v1(Path::new(path)) {
                Ok(summary) => {
                    println!("lock sync ok (deps_synced={})", summary.deps_synced);
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "policy" => {
            if args.get(1).map(String::as_str) != Some("lock")
                || args.get(2).map(String::as_str) != Some("sync")
            {
                eprintln!("usage: ocl policy lock sync <project_dir>");
                return 2;
            }
            let Some(path) = args.get(3) else {
                eprintln!("usage: ocl policy lock sync <project_dir>");
                return 2;
            };
            match sync_policy_lock_v1(Path::new(path)) {
                Ok(summary) => {
                    println!(
                        "policy lock sync ok (profile={}, hash={}, lock={})",
                        summary.policy_profile_id,
                        summary.canonical_hash256,
                        summary.lock_path.display()
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "cosmos" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl cosmos <init|lock> ...");
                return 2;
            };
            match scope {
                "init" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl cosmos init <project_dir> [--preset default|ci]");
                        return 2;
                    };
                    let preset = parse_string_flag(args, "--preset")
                        .unwrap_or_else(|| "default".to_string());
                    match init_cosmos_v1(Path::new(path), &preset) {
                        Ok(summary) => {
                            println!(
                                "cosmos init ok (preset={}, universes={}, file={})",
                                summary.preset,
                                summary.universes_written,
                                summary.cosmos_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "lock" => {
                    if args.get(2).map(String::as_str) != Some("sync") {
                        eprintln!("usage: ocl cosmos lock sync <project_dir> [--locked] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>]");
                        return 2;
                    }
                    let Some(path) = args.get(3) else {
                        eprintln!("usage: ocl cosmos lock sync <project_dir> [--locked] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>]");
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let signer_id = parse_string_flag(args, "--signer-id");
                    let sign_key = parse_string_flag(args, "--sign-key")
                        .map(PathBuf::from)
                        .or_else(|| std::env::var("OCL_SIGN_KEY_PATH").ok().map(PathBuf::from));
                    let trust_store = parse_string_flag(args, "--trust-store").map(PathBuf::from);
                    match sync_cosmos_lock_v1(
                        Path::new(path),
                        locked,
                        signer_id.as_deref(),
                        sign_key.as_deref(),
                        trust_store.as_deref(),
                    ) {
                        Ok(summary) => {
                            println!(
                                "cosmos lock sync ok (universes={}, hash={}, lock={})",
                                summary.universes_synced,
                                summary.canonical_hash256,
                                summary.lock_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl cosmos <init|lock> ...");
                    2
                }
            }
        }
        "organ" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl organ <lock|verify|install> ...");
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            match scope {
                "lock" => {
                    if args.get(2).map(String::as_str) != Some("sync") {
                        eprintln!("usage: ocl organ lock sync <project_dir> [--registry <index.toml>] [--json]");
                        return 2;
                    }
                    let Some(path) = args.get(3) else {
                        eprintln!("usage: ocl organ lock sync <project_dir> [--registry <index.toml>] [--json]");
                        return 2;
                    };
                    let project = Path::new(path);
                    let registry = parse_string_flag(args, "--registry")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| {
                            project.join("registry").join("organs").join("index.toml")
                        });
                    match sync_organs_lock_v1(project, &registry) {
                        Ok(summary) => {
                            if json_mode {
                                println!(
                                    "{{\"ok\":true,\"organs_synced\":{},\"lock\":\"{}\"}}",
                                    summary.organs_synced,
                                    json_escape(&summary.lock_path.to_string_lossy())
                                );
                            } else {
                                println!(
                                    "organ lock sync ok (organs_synced={}, lock={})",
                                    summary.organs_synced,
                                    summary.lock_path.display()
                                );
                            }
                            0
                        }
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            1
                        }
                    }
                }
                "verify" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl organ verify <project_dir> [--locked|--unlocked] [--json]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    match verify_organs_lock_v1(Path::new(path), locked) {
                        Ok(summary) => {
                            if json_mode {
                                println!(
                                    "{{\"ok\":true,\"organs_verified\":{},\"lock\":\"{}\"}}",
                                    summary.organs_verified,
                                    json_escape(&summary.lock_path.to_string_lossy())
                                );
                            } else {
                                println!(
                                    "organ verify ok (organs_verified={}, lock={})",
                                    summary.organs_verified,
                                    summary.lock_path.display()
                                );
                            }
                            0
                        }
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            1
                        }
                    }
                }
                "install" => {
                    let Some(name) = args.get(2) else {
                        eprintln!("usage: ocl organ install <name> <version> [--project <dir>] [--registry <index.toml>] [--locked] [--json]");
                        return 2;
                    };
                    let Some(version) = args.get(3) else {
                        eprintln!("usage: ocl organ install <name> <version> [--project <dir>] [--registry <index.toml>] [--locked] [--json]");
                        return 2;
                    };
                    let project = parse_string_flag(args, "--project")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    let registry = parse_string_flag(args, "--registry")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| {
                            project.join("registry").join("organs").join("index.toml")
                        });
                    let locked = args.iter().any(|a| a == "--locked");
                    match install_organs_v1(&project, &registry, name, version, locked) {
                        Ok(summary) => {
                            if json_mode {
                                println!(
                                    concat!(
                                        "{{\"ok\":true,",
                                        "\"pack_id\":\"{}\",",
                                        "\"version\":\"{}\",",
                                        "\"platform\":\"{}\",",
                                        "\"artifact_hash256\":\"{}\",",
                                        "\"install_path\":\"{}\"}}"
                                    ),
                                    json_escape(&summary.pack_id),
                                    json_escape(&summary.version),
                                    json_escape(&summary.platform),
                                    summary.artifact_hash256,
                                    json_escape(&summary.install_path.to_string_lossy())
                                );
                            } else {
                                println!(
                                    "organ install ok ({}@{} platform={} hash={} path={})",
                                    summary.pack_id,
                                    summary.version,
                                    summary.platform,
                                    summary.artifact_hash256,
                                    summary.install_path.display()
                                );
                            }
                            0
                        }
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl organ <lock|verify|install> ...");
                    2
                }
            }
        }
        "kit" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl kit <list|doctor> ...");
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            match scope {
                "list" => {
                    let project = if let Some(v) = args.get(2) {
                        if v.starts_with("--") {
                            PathBuf::from(".")
                        } else {
                            PathBuf::from(v)
                        }
                    } else {
                        PathBuf::from(".")
                    };
                    match list_kits_from_cosmos_v1(&project) {
                        Ok(kits) => {
                            if json_mode {
                                let mut rendered = String::from("{\"ok\":true,\"kits\":[");
                                for (idx, kit) in kits.iter().enumerate() {
                                    if idx > 0 {
                                        rendered.push(',');
                                    }
                                    rendered.push_str(&format!(
                                        "{{\"kit_id\":\"{}\",\"universe_id\":\"{}\",\"bind_domain\":\"{}\",\"bind_view\":\"{}\",\"required_organs\":[{}]}}",
                                        json_escape(&kit.kit_id),
                                        json_escape(&kit.universe_id),
                                        json_escape(&kit.bind_domain),
                                        json_escape(&kit.bind_view),
                                        kit.required_organs
                                            .iter()
                                            .map(|v| format!("\"{}\"", json_escape(v)))
                                            .collect::<Vec<String>>()
                                            .join(",")
                                    ));
                                }
                                rendered.push_str("]}");
                                println!("{rendered}");
                            } else {
                                println!("kit list (count={})", kits.len());
                                for kit in kits {
                                    println!(
                                        "- {} universe={} domain={} view={} required_organs={}",
                                        kit.kit_id,
                                        kit.universe_id,
                                        kit.bind_domain,
                                        kit.bind_view,
                                        if kit.required_organs.is_empty() {
                                            "-".to_string()
                                        } else {
                                            kit.required_organs.join(",")
                                        }
                                    );
                                }
                            }
                            0
                        }
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            1
                        }
                    }
                }
                "doctor" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl kit doctor <project_dir> [--locked|--json]");
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    match run_kit_doctor_v1(Path::new(path), locked) {
                        Ok(summary) => {
                            if json_mode {
                                println!(
                                    "{{\"ok\":true,\"kits_checked\":{},\"required_organs\":[{}]}}",
                                    summary.kits_checked,
                                    summary
                                        .required_organs
                                        .iter()
                                        .map(|v| format!("\"{}\"", json_escape(v)))
                                        .collect::<Vec<String>>()
                                        .join(",")
                                );
                            } else {
                                println!(
                                    "kit doctor ok (kits_checked={}, required_organs={})",
                                    summary.kits_checked,
                                    if summary.required_organs.is_empty() {
                                        "-".to_string()
                                    } else {
                                        summary.required_organs.join(",")
                                    }
                                );
                            }
                            0
                        }
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&err));
                            } else {
                                eprintln!("{err}");
                            }
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl kit <list|doctor> ...");
                    2
                }
            }
        }
        "plugin" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl plugin <lock|verify> ...");
                return 2;
            };
            match scope {
                "lock" => {
                    if args.get(2).map(String::as_str) != Some("sync") {
                        eprintln!("usage: ocl plugin lock sync <project_dir>");
                        return 2;
                    }
                    let Some(path) = args.get(3) else {
                        eprintln!("usage: ocl plugin lock sync <project_dir>");
                        return 2;
                    };
                    match sync_plugin_lock_v1(Path::new(path)) {
                        Ok(summary) => {
                            println!(
                                "plugin lock sync ok (plugins_synced={}, lock={})",
                                summary.plugins_synced,
                                summary.lock_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "verify" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl plugin verify <project_dir>");
                        return 2;
                    };
                    match verify_plugin_lock_v1(Path::new(path)) {
                        Ok(summary) => {
                            println!(
                                "plugin verify ok (plugins_verified={}, lock={})",
                                summary.plugins_verified,
                                summary.lock_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("usage: ocl plugin <lock|verify> ...");
                    2
                }
            }
        }
        "compose" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl compose <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                return 2;
            };
            let Some(phenotype) = parse_string_flag(args, "--phenotype") else {
                eprintln!("usage: ocl compose <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            let locked = args.iter().any(|a| a == "--locked");
            let universe_id = parse_string_flag(args, "--universe");
            if let Err(err) = resolve_universe_v1(Path::new(path), locked, universe_id.as_deref()) {
                eprintln!("{err}");
                return 1;
            }
            match compose_phenotype(
                Path::new(path),
                Path::new(&phenotype),
                Path::new(&registry),
                locked,
            ) {
                Ok(summary) => {
                    println!(
                        "compose ok (selected_components={}, generated_files={})",
                        summary.selected_components, summary.generated_files
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "doc" => {
            let Some(topic) = args.get(1) else {
                eprintln!("usage: ocl doc packs [--json]");
                return 2;
            };
            match topic.as_str() {
                "packs" => {
                    let json_mode = args.iter().any(|a| a == "--json");
                    let rendered = if json_mode {
                        render_doc_packs_json_v09()
                    } else {
                        render_doc_packs_text_v09()
                    };
                    println!("{rendered}");
                    0
                }
                _ => {
                    eprintln!("usage: ocl doc packs [--json]");
                    2
                }
            }
        }
        "verify" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                return 2;
            };
            let Some(phenotype) = parse_string_flag(args, "--phenotype") else {
                eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            let locked = args.iter().any(|a| a == "--locked");
            let universe_id = parse_string_flag(args, "--universe");
            if let Err(err) = resolve_universe_v1(Path::new(path), locked, universe_id.as_deref()) {
                eprintln!("{err}");
                return 1;
            }
            match verify_assembly(
                Path::new(path),
                Path::new(&phenotype),
                Path::new(&registry),
                locked,
            ) {
                Ok(summary) => {
                    println!(
                        "verify ok (selected_components={})",
                        summary.selected_components
                    );
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        _ => {
            print_help();
            2
        }
    }
}

fn render_doc_packs_text_v09() -> String {
    let registry = CapabilityRegistry::v1_baseline();
    let mut out = String::new();
    out.push_str("# OCL capability docs (v0.9)\n\n");
    out.push_str("Generated from CapabilityRegistry::v1_baseline()\n\n");
    for key in registry.documented_keys() {
        let key_kind = key_kind_label_v09(registry.key_kind_for_key(&key));
        let permission_class = permission_class_for_key_v09(&key);
        let ctx_required = registry.ctx_required_for_key(&key);
        let ctx_schema = registry
            .ctx_schema_for_key(&key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let payload_schema = registry
            .payload_schema_for_key(&key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let ctx_skeleton = registry
            .ctx_schema_for_key(&key)
            .map(schema_skeleton)
            .unwrap_or_else(|| "{}".to_string());
        out.push_str("- key: ");
        out.push_str(&key);
        out.push('\n');
        out.push_str("  key_kind: ");
        out.push_str(key_kind);
        out.push('\n');
        out.push_str("  permission_class: ");
        out.push_str(permission_class);
        out.push('\n');
        if ctx_required.is_empty() {
            out.push_str("  ctx_required: []\n");
        } else {
            out.push_str("  ctx_required: [");
            for (idx, field) in ctx_required.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                out.push_str(field);
            }
            out.push_str("]\n");
        }
        out.push_str("  ctx_schema:\n");
        out.push_str(&indent_block_v09(&ctx_schema, "    "));
        out.push_str("  payload_schema:\n");
        out.push_str(&indent_block_v09(&payload_schema, "    "));
        out.push_str("  example:\n");
        out.push_str("    observe(\"");
        out.push_str(&key);
        out.push_str("\", \"tier2\", <ctx>, budget(10)) -> r;\n");
        out.push_str("    ctx_skeleton:\n");
        out.push_str(&indent_block_v09(&ctx_skeleton, "      "));
        out.push('\n');
    }
    out
}

fn render_doc_packs_json_v09() -> String {
    let registry = CapabilityRegistry::v1_baseline();
    let mut out = String::from("{\"packs\":[");
    let keys = registry.documented_keys();
    for (idx, key) in keys.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let key_kind = key_kind_label_v09(registry.key_kind_for_key(key));
        let permission_class = permission_class_for_key_v09(key);
        let ctx_required = registry.ctx_required_for_key(key);
        let ctx_schema = registry
            .ctx_schema_for_key(key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let payload_schema = registry
            .payload_schema_for_key(key)
            .map(pretty_schema)
            .unwrap_or_else(|| "n/a".to_string());
        let ctx_skeleton = registry
            .ctx_schema_for_key(key)
            .map(schema_skeleton)
            .unwrap_or_else(|| "{}".to_string());
        out.push_str("{\"key\":\"");
        out.push_str(&json_escape(key));
        out.push_str("\",\"key_kind\":\"");
        out.push_str(key_kind);
        out.push_str("\",\"permission_class\":\"");
        out.push_str(permission_class);
        out.push_str("\",\"ctx_required\":[");
        for (field_idx, field) in ctx_required.iter().enumerate() {
            if field_idx > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(&json_escape(field));
            out.push('"');
        }
        out.push_str("],\"ctx_schema\":\"");
        out.push_str(&json_escape(&ctx_schema));
        out.push_str("\",\"payload_schema\":\"");
        out.push_str(&json_escape(&payload_schema));
        out.push_str("\",\"example\":\"");
        out.push_str(&json_escape(&format!(
            "observe(\"{}\", \"tier2\", <ctx>, budget(10)) -> r;",
            key
        )));
        out.push_str("\",\"ctx_skeleton\":\"");
        out.push_str(&json_escape(&ctx_skeleton));
        out.push_str("\"}");
    }
    out.push_str("]}");
    out
}

fn key_kind_label_v09(kind: KeyCapabilityKind) -> &'static str {
    match kind {
        KeyCapabilityKind::ObserveOnly => "observe_only",
        KeyCapabilityKind::ObserveAndCommit => "observe_and_commit",
    }
}

fn permission_class_for_key_v09(key: &str) -> &'static str {
    if key.starts_with("std.fs.") {
        "permissions.std_fs"
    } else if key.starts_with("std.kv.") {
        "permissions.std_kv"
    } else if key.starts_with("std.time.wallclock.") {
        "permissions.std_time_wallclock"
    } else if key.starts_with("std.time.") {
        "permissions.std_time"
    } else if key.starts_with("std.net.http.") {
        "permissions.std_net_http"
    } else if key.starts_with("std.proc.") {
        "permissions.std_proc"
    } else if key.starts_with("std.ui.") {
        "permissions.std_ui"
    } else if key.starts_with("std.game.") {
        "permissions.std_game"
    } else if key.starts_with("std.shadow.") {
        "permissions.std_shadow"
    } else if key.starts_with("engine.ui.") {
        "permissions.std_ui"
    } else if key.starts_with("engine.game.") {
        "permissions.std_game"
    } else if key.starts_with("engine.shadow.") {
        "permissions.std_shadow"
    } else {
        "permissions.package"
    }
}

fn indent_block_v09(input: &str, prefix: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        out.push_str(prefix);
        out.push_str(line);
        out.push('\n');
    }
    if input.is_empty() {
        out.push_str(prefix);
        out.push('\n');
    }
    out
}

fn with_view_env<T, F>(view_id: Option<&str>, run: F) -> T
where
    F: FnOnce() -> T,
{
    let prev = std::env::var("OCL_VIEW_ID").ok();
    match view_id {
        Some(value) => std::env::set_var("OCL_VIEW_ID", value),
        None => std::env::remove_var("OCL_VIEW_ID"),
    }
    let out = run();
    match prev {
        Some(value) => std::env::set_var("OCL_VIEW_ID", value),
        None => std::env::remove_var("OCL_VIEW_ID"),
    }
    out
}

fn with_runtime_env_v08<T, F>(updates: Vec<(&'static str, Option<String>)>, run: F) -> T
where
    F: FnOnce() -> T,
{
    let mut prev = Vec::<(&'static str, Option<String>)>::new();
    for (key, value) in &updates {
        prev.push((*key, std::env::var(key).ok()));
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    let out = run();
    for (key, value) in prev {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    out
}

fn parse_path_with_flag(args: &[String], flag: &str) -> (Option<PathBuf>, bool) {
    let mut path = None;
    let mut has_flag = false;
    for arg in args.iter().skip(1) {
        if arg == flag {
            has_flag = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        }
    }
    (path, has_flag)
}

fn parse_u32_flag(args: &[String], flag: &str) -> Option<u32> {
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == flag {
            if let Some(next) = args.get(idx + 1) {
                if let Ok(v) = next.parse::<u32>() {
                    return Some(v);
                }
            }
        }
        idx += 1;
    }
    None
}

fn parse_string_flag(args: &[String], flag: &str) -> Option<String> {
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == flag {
            return args.get(idx + 1).cloned();
        }
        idx += 1;
    }
    None
}

fn parse_shadow_args(args: &[String]) -> Result<Option<ShadowOptionsV1>, i32> {
    let Some(shadow_id) = parse_string_flag(args, "--shadow") else {
        return Ok(None);
    };
    let policy_raw =
        parse_string_flag(args, "--shadow-policy").unwrap_or_else(|| "forbid_commit".to_string());
    let Some(policy) = parse_shadow_policy_v1(&policy_raw) else {
        eprintln!(
            "invalid shadow policy: `{policy_raw}` (expected forbid_commit|shadow_commit_log)"
        );
        return Err(2);
    };
    Ok(Some(ShadowOptionsV1 {
        shadow_id,
        policy,
        report_dir: None,
    }))
}

fn apply_init_template_v072(root: &Path, template: &str) -> Result<(), SdkError> {
    match template {
        "tool-cli" => apply_tool_cli_template_v072(root),
        "tool-http" => apply_tool_http_template_v08(root),
        "tool-proc" => apply_tool_proc_template_v08(root),
        "tool-wallclock" => apply_tool_wallclock_template_v08(root),
        "mini-game" => apply_mini_game_template_v073(root),
        "shadow-preview" => apply_shadow_preview_template_v073(root),
        _ => Err(SdkError::MissingProject(format!(
            "V-INIT-TEMPLATE-UNKNOWN: unsupported template `{template}` (expected `tool-cli`, `tool-http`, `tool-proc`, `tool-wallclock`, `mini-game`, or `shadow-preview`)"
        ))),
    }
}

fn apply_tool_cli_template_v072(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tool_cli")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.fs.*\", \"std.kv.*\", \"std.time.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./fixtures/in/**\", \"./out/**\"]\n",
            "write = [\"./out/**\"]\n",
            "remove = [\"./out/**\"]\n",
            "rename = [\"./out/**\"]\n",
            "list = [\"./fixtures/in/**\", \"./out/**\"]\n",
            "max_read_bytes = 1048576\n",
            "max_write_bytes = 1048576\n",
            "max_list_entries = 500\n\n",
            "[permissions.std_kv]\n",
            "enabled = true\n",
            "max_keys = 512\n",
            "max_value_bytes = 65536\n",
            "key_prefix = \"tool.\"\n\n",
            "[permissions.std_time]\n",
            "enabled = true\n",
            "tick_mode = \"logical\"\n",
            "dt_ms = 16\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.tool_cli;\n\n",
        "observe(\"std.fs.mkdir\", \"tier2\", ctx(\"path=./out;recursive=true\"), budget(5)) -> mk;\n",
        "commit(mk);\n\n",
        "observe(\"std.fs.write_text\", \"tier2\", ctx(\"path=./out/out.json;text=OK;overwrite=true\"), budget(5)) -> wr;\n",
        "commit(wr);\n\n",
        "observe(\"std.time.tick_info\", \"tier2\", ctx(\"scope=tool\"), budget(5)) -> ti;\n",
        "observe(\"std.kv.put\", \"tier2\", ctx(\"key=tool.last_run;value=ok;overwrite=true\"), budget(5)) -> kvp;\n",
        "commit(kvp);\n\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# tool-cli template\n\n",
        "- Input fixtures: `fixtures/in/*.json`\n",
        "- Expected outputs: `fixtures/expected/*.json`\n",
        "- Runtime outputs: `out/`\n\n",
        "Run:\n",
        "- `ocl test . --golden fixtures/expected --clean`\n"
    );
    fs::write(root.join("README.md"), readme)?;

    let fixtures_in = root.join("fixtures").join("in");
    let fixtures_expected = root.join("fixtures").join("expected");
    fs::create_dir_all(&fixtures_in)?;
    fs::create_dir_all(&fixtures_expected)?;
    fs::write(
        fixtures_in.join("sample.json"),
        "{\n  \"input\": \"sample\"\n}\n",
    )?;
    fs::write(
        fixtures_expected.join("out.json"),
        "{\n  \"input\": \"sample\"\n}\n",
    )?;
    Ok(())
}

fn apply_tool_http_template_v08(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tool_http")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"quarantine\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[quarantine]\n",
            "mode = \"record\"\n",
            "max_cassette_bytes = 10485760\n",
            "max_entries = 2000\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[compat]\n",
            "ctx_string = \"deny\"\n",
            "ctx_extra_fields = \"warn\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.net.http.*\", \"std.fs.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_net_http]\n",
            "enabled = true\n",
            "allow_hosts = [\"mock.local\"]\n",
            "allow_methods = [\"GET\"]\n",
            "timeout_ms = 3000\n",
            "max_body_bytes = 64\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./out/**\"]\n",
            "write = [\"./out/**\"]\n",
            "remove = [\"./out/**\"]\n",
            "rename = [\"./out/**\"]\n",
            "list = [\"./out/**\"]\n",
            "max_read_bytes = 1048576\n",
            "max_write_bytes = 1048576\n",
            "max_list_entries = 500\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.tool_http;\n\n",
        "observe(\"std.net.http.request\", \"tier2\", { method: \"GET\", url: \"http://mock.local/template-http\", timeout_ms: 3000, max_body_bytes: 32 }, budget(8)) -> net;\n",
        "observe(\"std.fs.mkdir\", \"tier2\", { path: \"./out\", recursive: true }, budget(5)) -> mk;\n",
        "commit(mk);\n",
        "observe(\"std.fs.write_text\", \"tier2\", { path: \"./out/http.txt\", text: \"HTTP_TOOL_OK\", overwrite: true }, budget(5)) -> wr;\n",
        "commit(wr);\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# tool-http template\n\n",
        "- Lane: `quarantine`\n",
        "- Capability chính: `std.net.http.request` (record/replay qua cassette)\n\n",
        "Run record:\n",
        "- `OCL_QUARANTINE=1 ocl run .`\n\n",
        "Replay:\n",
        "- `OCL_QUARANTINE=1 ocl replay ./.ocl_artifacts/<run_id>`\n"
    );
    fs::write(root.join("README.md"), readme)?;
    Ok(())
}

fn apply_tool_proc_template_v08(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tool_proc")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"quarantine\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[quarantine]\n",
            "mode = \"record\"\n",
            "max_cassette_bytes = 10485760\n",
            "max_entries = 2000\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[compat]\n",
            "ctx_string = \"deny\"\n",
            "ctx_extra_fields = \"warn\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.proc.*\", \"std.fs.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_proc]\n",
            "enabled = true\n",
            "allow_bins = [\"mock.proc\"]\n",
            "timeout_ms = 3000\n",
            "max_stdout_bytes = 128\n",
            "max_stderr_bytes = 128\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./out/**\"]\n",
            "write = [\"./out/**\"]\n",
            "remove = [\"./out/**\"]\n",
            "rename = [\"./out/**\"]\n",
            "list = [\"./out/**\"]\n",
            "max_read_bytes = 1048576\n",
            "max_write_bytes = 1048576\n",
            "max_list_entries = 500\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.tool_proc;\n\n",
        "observe(\"std.proc.exec\", \"tier2\", { bin: \"mock.proc\", args: \"--template\", timeout_ms: 3000, max_stdout_bytes: 32 }, budget(8)) -> proc_res;\n",
        "observe(\"std.fs.mkdir\", \"tier2\", { path: \"./out\", recursive: true }, budget(5)) -> mk;\n",
        "commit(mk);\n",
        "observe(\"std.fs.write_text\", \"tier2\", { path: \"./out/proc.txt\", text: \"PROC_TOOL_OK\", overwrite: true }, budget(5)) -> wr;\n",
        "commit(wr);\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# tool-proc template\n\n",
        "- Lane: `quarantine`\n",
        "- Capability chính: `std.proc.exec` (record/replay qua cassette)\n\n",
        "Run record:\n",
        "- `OCL_QUARANTINE=1 ocl run .`\n\n",
        "Replay:\n",
        "- `OCL_QUARANTINE=1 ocl replay ./.ocl_artifacts/<run_id>`\n"
    );
    fs::write(root.join("README.md"), readme)?;
    Ok(())
}

fn apply_tool_wallclock_template_v08(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tool_wallclock")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"quarantine\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[quarantine]\n",
            "mode = \"record\"\n",
            "max_cassette_bytes = 10485760\n",
            "max_entries = 2000\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[compat]\n",
            "ctx_string = \"deny\"\n",
            "ctx_extra_fields = \"warn\"\n\n",
            "[permissions.package]\n",
            "allow = [\"std.time.*\", \"std.fs.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_time]\n",
            "enabled = true\n",
            "tick_mode = \"logical\"\n",
            "dt_ms = 16\n\n",
            "[permissions.std_fs]\n",
            "read = [\"./out/**\"]\n",
            "write = [\"./out/**\"]\n",
            "remove = [\"./out/**\"]\n",
            "rename = [\"./out/**\"]\n",
            "list = [\"./out/**\"]\n",
            "max_read_bytes = 1048576\n",
            "max_write_bytes = 1048576\n",
            "max_list_entries = 500\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.tool_wallclock;\n\n",
        "observe(\"std.time.wallclock.now\", \"tier2\", { scope: \"tool\" }, budget(8)) -> now;\n",
        "observe(\"std.fs.mkdir\", \"tier2\", { path: \"./out\", recursive: true }, budget(5)) -> mk;\n",
        "commit(mk);\n",
        "observe(\"std.fs.write_text\", \"tier2\", { path: \"./out/wallclock.txt\", text: \"WALLCLOCK_TOOL_OK\", overwrite: true }, budget(5)) -> wr;\n",
        "commit(wr);\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# tool-wallclock template\n\n",
        "- Lane: `quarantine`\n",
        "- Capability chính: `std.time.wallclock.now` (record/replay qua cassette)\n\n",
        "Run record:\n",
        "- `OCL_QUARANTINE=1 ocl run .`\n\n",
        "Replay:\n",
        "- `OCL_QUARANTINE=1 ocl replay ./.ocl_artifacts/<run_id>`\n"
    );
    fs::write(root.join("README.md"), readme)?;
    Ok(())
}

fn apply_mini_game_template_v073(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("mini_game")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[permissions.package]\n",
            "allow = [\"engine.game.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_game]\n",
            "enabled = true\n",
            "fixed_dt_ms = 16\n",
            "rng_streams = [\"main\", \"loot\"]\n",
            "rng_max_count = 32\n",
            "state_delta_max_bytes = 65536\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.mini_game;\n\n",
        "observe(\"engine.game.run\", \"tier2\", ctx(\"entry_module=app.mini_game;phase=frame;tick=1;stream=main;count=4;input_cap=8;events=key:Space|text:start;draw_cap=8;draw_list=text:1,1,mini-game,12;state_json={\\\"score\\\":0}\"), budget(10)) -> frame;\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# mini-game template\n\n",
        "- Run: `ocl run .`\n",
        "- Replay: `ocl replay ./.ocl_artifacts/<run_id>`\n"
    );
    fs::write(root.join("README.md"), readme)?;
    Ok(())
}

fn apply_shadow_preview_template_v073(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("shadow_preview")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[project]\n",
            "lane = \"locked_v071\"\n",
            "entry = \"src/main.ocl\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[language]\n",
            "guard_mode = \"return\"\n\n",
            "[permissions.package]\n",
            "allow = [\"engine.shadow.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_shadow]\n",
            "enabled = true\n",
            "max_branches = 8\n",
            "branch_step_cap = 5000\n",
            "branch_budget_cap = 200000\n",
            "max_diff_keys = 2000\n",
            "max_report_bytes = 262144\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.shadow_preview;\n\n",
        "observe(\"engine.shadow.preview\", \"tier2\", ctx(\"entry_module=app.shadow_preview;tick=1;variants_json=[{\\\"move\\\":\\\"left\\\"},{\\\"move\\\":\\\"right\\\"}];baseline_id=0;max_diff_keys=8\"), budget(20)) -> preview;\n",
        "condition(true);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let readme = concat!(
        "# shadow-preview template\n\n",
        "- Run: `ocl run .`\n",
        "- Replay: `ocl replay ./.ocl_artifacts/<run_id>`\n"
    );
    fs::write(root.join("README.md"), readme)?;
    Ok(())
}

fn apply_foundation_preset_v5(root: &Path, preset: &str) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("app")
        .replace('-', "_");

    let manifest = format!(
        concat!(
            "[package]\n",
            "name = \"{}\"\n",
            "version = \"0.1.0\"\n\n",
            "[targets]\n",
            "default = \"main\"\n\n",
            "[dependencies]\n",
            "std = \"0.1.0\"\n\n",
            "[policy]\n",
            "budget_profile = \"ci_default\"\n\n",
            "[permissions.package]\n",
            "allow = [\"*\"]\n",
            "deny = []\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = match preset {
        "workflow_basic" => {
            "module app.workflow_basic;\n\nlet ready = true;\ncondition(ready);\n".to_string()
        }
        "agent_swarm_basic" => concat!(
            "module app.agent_swarm_basic;\n\n",
            "let overmind_ready = true;\n",
            "condition(overmind_ready);\n"
        )
        .to_string(),
        _ => {
            return Err(SdkError::MissingProject(format!(
                "V-INIT-PRESET-UNKNOWN: unsupported preset `{preset}` (expected `workflow_basic` or `agent_swarm_basic`)"
            )));
        }
    };
    fs::write(root.join("src").join("main.ocl"), source)?;

    let cosmos = match preset {
        "workflow_basic" => concat!(
            "version = 1\n\n",
            "[[universe]]\n",
            "id = \"ci_locked\"\n",
            "runtime_mode = \"deterministic\"\n",
            "engine = \"dual\"\n",
            "policy_profile_id = \"ci_default\"\n",
            "audit = \"hash_only\"\n",
            "trace = \"hash_only\"\n\n",
            "[[domain]]\n",
            "id = \"default\"\n",
            "universe_id = \"ci_locked\"\n\n",
            "[[view]]\n",
            "id = \"text_default\"\n",
            "universe_id = \"ci_locked\"\n",
            "domain_id = \"default\"\n",
            "renderer = \"text\"\n\n",
            "[[kits]]\n",
            "kit_id = \"std.kit.view_text_basic\"\n",
            "universe_id = \"ci_locked\"\n",
            "bind_domain = \"default\"\n",
            "bind_view = \"text_default\"\n",
            "required_organs = []\n"
        )
        .to_string(),
        "agent_swarm_basic" => concat!(
            "version = 1\n\n",
            "[[universe]]\n",
            "id = \"ci_locked\"\n",
            "runtime_mode = \"deterministic\"\n",
            "engine = \"dual\"\n",
            "policy_profile_id = \"ci_default\"\n",
            "audit = \"hash_only\"\n",
            "trace = \"hash_only\"\n\n",
            "[[domain]]\n",
            "id = \"default\"\n",
            "universe_id = \"ci_locked\"\n\n",
            "[[domain]]\n",
            "id = \"side\"\n",
            "universe_id = \"ci_locked\"\n\n",
            "[[bridge]]\n",
            "id = \"default_to_side\"\n",
            "universe_id = \"ci_locked\"\n",
            "from_domain = \"default\"\n",
            "to_domain = \"side\"\n",
            "emit_quota_per_tick = 4\n",
            "dispatch_max_per_tick = 2\n",
            "bridge_queue_capacity = 8\n\n",
            "[hive]\n",
            "max_swarm_workers = 16\n",
            "max_fanout_per_task = 8\n",
            "mailbox_max_depth = 64\n",
            "max_spawn_per_tick = 4\n",
            "max_domains = 8\n",
            "max_shadow_worlds = 8\n\n",
            "[[view]]\n",
            "id = \"ops\"\n",
            "universe_id = \"ci_locked\"\n",
            "domain_id = \"default\"\n",
            "renderer = \"text\"\n\n",
            "[[kits]]\n",
            "kit_id = \"std.kit.view_text_basic\"\n",
            "universe_id = \"ci_locked\"\n",
            "bind_domain = \"default\"\n",
            "bind_view = \"ops\"\n",
            "required_organs = []\n"
        )
        .to_string(),
        _ => {
            return Err(SdkError::MissingProject(format!(
                "V-INIT-PRESET-UNKNOWN: unsupported preset `{preset}` (expected `workflow_basic` or `agent_swarm_basic`)"
            )));
        }
    };
    fs::write(root.join("cosmos.toml"), cosmos)?;
    Ok(())
}

fn manifest_path_for_v071(root: &Path) -> PathBuf {
    let lower = root.join("ocl.toml");
    if lower.exists() {
        return lower;
    }
    root.join("Ocl.toml")
}

fn read_lane_and_entry_for_v071(root: &Path) -> (String, String) {
    let manifest_path = manifest_path_for_v071(root);
    let mut lane = "locked_v071".to_string();
    let mut entry = "src/main.ocl".to_string();
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return (lane, entry);
    };

    let mut section = String::new();
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"');
        if section == "project" {
            if key == "lane" && !value.is_empty() {
                lane = value.to_string();
            } else if key == "entry" && !value.is_empty() {
                entry = value.to_string();
            }
        } else if section == "targets"
            && key == "default"
            && !value.is_empty()
            && entry == "src/main.ocl"
        {
            entry = format!("src/{value}.ocl");
        }
    }

    (lane, entry)
}

#[derive(Debug, Clone)]
struct StdFsRuntimeConfigV08 {
    read: Vec<String>,
    write: Vec<String>,
    remove: Vec<String>,
    rename: Vec<String>,
    list: Vec<String>,
    max_read_bytes: u64,
    max_write_bytes: u64,
    max_list_entries: u32,
}

impl Default for StdFsRuntimeConfigV08 {
    fn default() -> Self {
        Self {
            read: Vec::new(),
            write: Vec::new(),
            remove: Vec::new(),
            rename: Vec::new(),
            list: Vec::new(),
            max_read_bytes: 1_048_576,
            max_write_bytes: 1_048_576,
            max_list_entries: 1_000,
        }
    }
}

#[derive(Debug, Clone)]
struct StdProcRuntimeConfigV08 {
    enabled: bool,
    allow_bins: Vec<String>,
    timeout_ms: u32,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
}

impl Default for StdProcRuntimeConfigV08 {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_bins: Vec::new(),
            timeout_ms: 5_000,
            max_stdout_bytes: 1_048_576,
            max_stderr_bytes: 1_048_576,
        }
    }
}

fn parse_string_array_literal_v08(value_raw: &str) -> Vec<String> {
    let value = value_raw.trim();
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Vec::new();
        }
        return inner
            .split(',')
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
            .collect();
    }
    let single = value.trim_matches('"').to_string();
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}

fn read_std_fs_runtime_config_v08(root: &Path) -> Option<StdFsRuntimeConfigV08> {
    let manifest_path = manifest_path_for_v071(root);
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return None;
    };

    let mut out = StdFsRuntimeConfigV08::default();
    let mut section = String::new();
    let mut section_seen = false;
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if section != "permissions.std_fs" {
            continue;
        }
        section_seen = true;
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"');
        match key {
            "read" => out.read = parse_string_array_literal_v08(v_raw),
            "write" => out.write = parse_string_array_literal_v08(v_raw),
            "remove" => out.remove = parse_string_array_literal_v08(v_raw),
            "rename" => out.rename = parse_string_array_literal_v08(v_raw),
            "list" => out.list = parse_string_array_literal_v08(v_raw),
            "max_read_bytes" => {
                if let Ok(parsed) = value.parse::<u64>() {
                    out.max_read_bytes = parsed.max(1);
                }
            }
            "max_write_bytes" => {
                if let Ok(parsed) = value.parse::<u64>() {
                    out.max_write_bytes = parsed.max(1);
                }
            }
            "max_list_entries" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.max_list_entries = parsed.max(1);
                }
            }
            _ => {}
        }
    }

    if !section_seen {
        return None;
    }

    out.read.sort();
    out.read.dedup();
    out.write.sort();
    out.write.dedup();
    out.remove.sort();
    out.remove.dedup();
    out.rename.sort();
    out.rename.dedup();
    out.list.sort();
    out.list.dedup();
    Some(out)
}

fn fs_runtime_env_updates_v08(root: &Path) -> Vec<(&'static str, Option<String>)> {
    let mut updates = vec![
        (ENV_STD_FS_ROOT_V08, None),
        (ENV_STD_FS_ALLOW_READ_V08, None),
        (ENV_STD_FS_ALLOW_WRITE_V08, None),
        (ENV_STD_FS_ALLOW_REMOVE_V08, None),
        (ENV_STD_FS_ALLOW_RENAME_V08, None),
        (ENV_STD_FS_ALLOW_LIST_V08, None),
        (ENV_STD_FS_MAX_READ_BYTES_V08, None),
        (ENV_STD_FS_MAX_WRITE_BYTES_V08, None),
        (ENV_STD_FS_MAX_LIST_ENTRIES_V08, None),
    ];
    let Some(cfg) = read_std_fs_runtime_config_v08(root) else {
        return updates;
    };

    let root_path = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .to_string();
    updates[0] = (ENV_STD_FS_ROOT_V08, Some(root_path));
    updates[1] = (ENV_STD_FS_ALLOW_READ_V08, Some(cfg.read.join(",")));
    updates[2] = (ENV_STD_FS_ALLOW_WRITE_V08, Some(cfg.write.join(",")));
    updates[3] = (ENV_STD_FS_ALLOW_REMOVE_V08, Some(cfg.remove.join(",")));
    updates[4] = (ENV_STD_FS_ALLOW_RENAME_V08, Some(cfg.rename.join(",")));
    updates[5] = (ENV_STD_FS_ALLOW_LIST_V08, Some(cfg.list.join(",")));
    updates[6] = (
        ENV_STD_FS_MAX_READ_BYTES_V08,
        Some(cfg.max_read_bytes.to_string()),
    );
    updates[7] = (
        ENV_STD_FS_MAX_WRITE_BYTES_V08,
        Some(cfg.max_write_bytes.to_string()),
    );
    updates[8] = (
        ENV_STD_FS_MAX_LIST_ENTRIES_V08,
        Some(cfg.max_list_entries.to_string()),
    );
    updates
}

fn read_std_proc_runtime_config_v08(root: &Path) -> StdProcRuntimeConfigV08 {
    let manifest_path = manifest_path_for_v071(root);
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return StdProcRuntimeConfigV08::default();
    };

    let mut out = StdProcRuntimeConfigV08::default();
    let mut section = String::new();
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if section != "permissions.std_proc" {
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"');
        match key {
            "enabled" => {
                out.enabled = matches!(value, "true");
            }
            "allow_bins" => {
                out.allow_bins = parse_string_array_literal_v08(v_raw);
                out.allow_bins.sort();
                out.allow_bins.dedup();
            }
            "timeout_ms" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.timeout_ms = parsed.max(1);
                }
            }
            "max_stdout_bytes" => {
                if let Ok(parsed) = value.parse::<u64>() {
                    out.max_stdout_bytes = parsed.max(1);
                }
            }
            "max_stderr_bytes" => {
                if let Ok(parsed) = value.parse::<u64>() {
                    out.max_stderr_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
    }
    out
}

fn proc_runtime_env_updates_v08(root: &Path) -> Vec<(&'static str, Option<String>)> {
    let mut updates = vec![
        (ENV_STD_PROC_ALLOW_BINS_V08, None),
        (ENV_STD_PROC_TIMEOUT_MS_V08, None),
        (ENV_STD_PROC_MAX_STDOUT_BYTES_V08, None),
        (ENV_STD_PROC_MAX_STDERR_BYTES_V08, None),
    ];
    let cfg = read_std_proc_runtime_config_v08(root);
    if !cfg.enabled {
        return updates;
    }
    updates[0] = (ENV_STD_PROC_ALLOW_BINS_V08, Some(cfg.allow_bins.join(",")));
    updates[1] = (
        ENV_STD_PROC_TIMEOUT_MS_V08,
        Some(cfg.timeout_ms.to_string()),
    );
    updates[2] = (
        ENV_STD_PROC_MAX_STDOUT_BYTES_V08,
        Some(cfg.max_stdout_bytes.to_string()),
    );
    updates[3] = (
        ENV_STD_PROC_MAX_STDERR_BYTES_V08,
        Some(cfg.max_stderr_bytes.to_string()),
    );
    updates
}

#[derive(Debug, Clone)]
struct StdNetHttpRuntimeConfigV08 {
    enabled: bool,
    allow_hosts: Vec<String>,
    allow_methods: Vec<String>,
    timeout_ms: u32,
    max_body_bytes: u64,
}

impl Default for StdNetHttpRuntimeConfigV08 {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_hosts: Vec::new(),
            allow_methods: Vec::new(),
            timeout_ms: 5_000,
            max_body_bytes: 1_048_576,
        }
    }
}

fn read_std_net_http_runtime_config_v08(root: &Path) -> StdNetHttpRuntimeConfigV08 {
    let manifest_path = manifest_path_for_v071(root);
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return StdNetHttpRuntimeConfigV08::default();
    };

    let mut out = StdNetHttpRuntimeConfigV08::default();
    let mut section = String::new();
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if section != "permissions.std_net_http" {
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"');
        match key {
            "enabled" => {
                out.enabled = matches!(value, "true");
            }
            "allow_hosts" => {
                out.allow_hosts = parse_string_array_literal_v08(v_raw)
                    .into_iter()
                    .map(|v| v.to_ascii_lowercase())
                    .collect();
                out.allow_hosts.sort();
                out.allow_hosts.dedup();
            }
            "allow_methods" => {
                out.allow_methods = parse_string_array_literal_v08(v_raw)
                    .into_iter()
                    .map(|v| v.to_ascii_uppercase())
                    .collect();
                out.allow_methods.sort();
                out.allow_methods.dedup();
            }
            "timeout_ms" => {
                if let Ok(parsed) = value.parse::<u32>() {
                    out.timeout_ms = parsed.max(1);
                }
            }
            "max_body_bytes" => {
                if let Ok(parsed) = value.parse::<u64>() {
                    out.max_body_bytes = parsed.max(1);
                }
            }
            _ => {}
        }
    }
    out
}

fn net_http_runtime_env_updates_v08(root: &Path) -> Vec<(&'static str, Option<String>)> {
    let mut updates = vec![
        (ENV_STD_NET_ALLOW_HOSTS_V08, None),
        (ENV_STD_NET_ALLOW_METHODS_V08, None),
        (ENV_STD_NET_TIMEOUT_MS_V08, None),
        (ENV_STD_NET_MAX_BODY_BYTES_V08, None),
    ];
    let cfg = read_std_net_http_runtime_config_v08(root);
    if !cfg.enabled {
        return updates;
    }
    updates[0] = (ENV_STD_NET_ALLOW_HOSTS_V08, Some(cfg.allow_hosts.join(",")));
    updates[1] = (
        ENV_STD_NET_ALLOW_METHODS_V08,
        Some(cfg.allow_methods.join(",")),
    );
    updates[2] = (ENV_STD_NET_TIMEOUT_MS_V08, Some(cfg.timeout_ms.to_string()));
    updates[3] = (
        ENV_STD_NET_MAX_BODY_BYTES_V08,
        Some(cfg.max_body_bytes.to_string()),
    );
    updates
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaneModeV071 {
    LockedV06,
    LockedV071,
    Quarantine,
}

fn parse_lane_mode_v071(lane: &str) -> Result<LaneModeV071, SdkError> {
    match lane {
        "locked_v06" => Ok(LaneModeV071::LockedV06),
        "locked_v071" => Ok(LaneModeV071::LockedV071),
        "quarantine" => Ok(LaneModeV071::Quarantine),
        other => Err(SdkError::MissingProject(format!(
            "V-LANE-INVALID: unsupported lane `{other}` (expected locked_v06|locked_v071|quarantine)"
        ))),
    }
}

fn quarantine_env_enabled_v073() -> bool {
    matches!(std::env::var("OCL_QUARANTINE"), Ok(v) if v.trim() == "1")
}

fn enforce_quarantine_lane_gate_v073(lane: &str) -> Result<(), SdkError> {
    let mode = parse_lane_mode_v071(lane)?;
    if mode == LaneModeV071::Quarantine && !quarantine_env_enabled_v073() {
        return Err(SdkError::MissingProject(
            "V-QUARANTINE-DISABLED: lane `quarantine` requires env `OCL_QUARANTINE=1`".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ReplaySpecV071 {
    root: PathBuf,
    lane: String,
    engine: RunEngine,
    signature: String,
    mode: String,
    cassette_hash: Option<String>,
    hasher_version: Option<String>,
}

const CASSETTE_SCHEMA_VERSION_V08: &str = "v0.8";
const CASSETTE_HASHER_VERSION_V08: &str = "sha256-v1";
const CASSETTE_MODE_RECORD_V08: &str = "record";
const CASSETTE_MODE_REPLAY_V08: &str = "replay";
const ENV_PROJECT_LANE_V08: &str = "OCL_PROJECT_LANE";
const ENV_QUARANTINE_MODE_V08: &str = "OCL_QUARANTINE_MODE";
const ENV_WALLCLOCK_RECORD_PATH_V08: &str = "OCL_V08_WALLCLOCK_RECORD_PATH";
const ENV_PROC_RECORD_PATH_V08: &str = "OCL_V08_PROC_RECORD_PATH";
const ENV_HTTP_RECORD_PATH_V08: &str = "OCL_V08_HTTP_RECORD_PATH";
const ENV_CASSETTE_JSONL_PATH_V08: &str = "OCL_V08_CASSETTE_JSONL_PATH";
const ENV_CASSETTE_INDEX_PATH_V08: &str = "OCL_V08_CASSETTE_INDEX_PATH";
const ENV_STD_FS_ROOT_V08: &str = "OCL_STD_FS_ROOT";
const ENV_STD_FS_ALLOW_READ_V08: &str = "OCL_STD_FS_ALLOW_READ";
const ENV_STD_FS_ALLOW_WRITE_V08: &str = "OCL_STD_FS_ALLOW_WRITE";
const ENV_STD_FS_ALLOW_REMOVE_V08: &str = "OCL_STD_FS_ALLOW_REMOVE";
const ENV_STD_FS_ALLOW_RENAME_V08: &str = "OCL_STD_FS_ALLOW_RENAME";
const ENV_STD_FS_ALLOW_LIST_V08: &str = "OCL_STD_FS_ALLOW_LIST";
const ENV_STD_FS_MAX_READ_BYTES_V08: &str = "OCL_STD_FS_MAX_READ_BYTES";
const ENV_STD_FS_MAX_WRITE_BYTES_V08: &str = "OCL_STD_FS_MAX_WRITE_BYTES";
const ENV_STD_FS_MAX_LIST_ENTRIES_V08: &str = "OCL_STD_FS_MAX_LIST_ENTRIES";
const ENV_STD_PROC_ALLOW_BINS_V08: &str = "OCL_STD_PROC_ALLOW_BINS";
const ENV_STD_PROC_TIMEOUT_MS_V08: &str = "OCL_STD_PROC_TIMEOUT_MS";
const ENV_STD_PROC_MAX_STDOUT_BYTES_V08: &str = "OCL_STD_PROC_MAX_STDOUT_BYTES";
const ENV_STD_PROC_MAX_STDERR_BYTES_V08: &str = "OCL_STD_PROC_MAX_STDERR_BYTES";
const ENV_STD_NET_ALLOW_HOSTS_V08: &str = "OCL_STD_NET_ALLOW_HOSTS";
const ENV_STD_NET_ALLOW_METHODS_V08: &str = "OCL_STD_NET_ALLOW_METHODS";
const ENV_STD_NET_TIMEOUT_MS_V08: &str = "OCL_STD_NET_TIMEOUT_MS";
const ENV_STD_NET_MAX_BODY_BYTES_V08: &str = "OCL_STD_NET_MAX_BODY_BYTES";

fn parse_run_engine_literal(raw: &str) -> Result<RunEngine, SdkError> {
    match raw {
        "interpreter" => Ok(RunEngine::Interpreter),
        "bytecode" => Ok(RunEngine::Bytecode),
        "dual" => Ok(RunEngine::Dual),
        other => Err(SdkError::MissingProject(format!(
            "V-REPLAY-ENGINE-INVALID: unsupported engine `{other}` in replay.toml"
        ))),
    }
}

fn parse_replay_toml_v071(path: &Path) -> Result<ReplaySpecV071, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut section = String::new();
    let mut root = None::<PathBuf>;
    let mut lane = "locked_v071".to_string();
    let mut engine = None::<RunEngine>;
    let mut signature = None::<String>;
    let mut mode = CASSETTE_MODE_RECORD_V08.to_string();
    let mut cassette_hash = None::<String>;
    let mut hasher_version = None::<String>;

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if section != "replay" {
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"');
        match key {
            "root" if !value.is_empty() => {
                root = Some(PathBuf::from(value));
            }
            "lane" if !value.is_empty() => {
                lane = value.to_string();
            }
            "engine" if !value.is_empty() => {
                engine = Some(parse_run_engine_literal(value)?);
            }
            "signature" if !value.is_empty() => {
                signature = Some(value.to_string());
            }
            "mode" if !value.is_empty() => {
                mode = value.to_string();
            }
            "cassette_hash" if !value.is_empty() => {
                cassette_hash = Some(value.to_string());
            }
            "hasher_version" if !value.is_empty() => {
                hasher_version = Some(value.to_string());
            }
            _ => {}
        }
    }

    let Some(root) = root else {
        return Err(SdkError::MissingProject(
            "V-REPLAY-MISSING-ROOT: replay.toml missing `[replay].root`".to_string(),
        ));
    };
    let Some(engine) = engine else {
        return Err(SdkError::MissingProject(
            "V-REPLAY-MISSING-ENGINE: replay.toml missing `[replay].engine`".to_string(),
        ));
    };
    let Some(signature) = signature else {
        return Err(SdkError::MissingProject(
            "V-REPLAY-MISSING-SIGNATURE: replay.toml missing `[replay].signature`".to_string(),
        ));
    };
    if mode != CASSETTE_MODE_RECORD_V08 && mode != CASSETTE_MODE_REPLAY_V08 {
        return Err(SdkError::MissingProject(format!(
            "V-REPLAY-MODE-INVALID: unsupported mode `{mode}` in replay.toml"
        )));
    }
    if lane == "quarantine" {
        if cassette_hash.is_none() {
            return Err(SdkError::MissingProject(
                "V-CASSETTE-MISSING: replay.toml missing `[replay].cassette_hash` for quarantine lane (INSUFFICIENT(RC-CASSETTE-MISSING))".to_string(),
            ));
        }
        if hasher_version.is_none() {
            return Err(SdkError::MissingProject(
                "V-CASSETTE-HASHER-MISSING: replay.toml missing `[replay].hasher_version` for quarantine lane".to_string(),
            ));
        }
    }

    Ok(ReplaySpecV071 {
        root,
        lane,
        engine,
        signature,
        mode,
        cassette_hash,
        hasher_version,
    })
}

fn locked_from_lane_v071(lane: &str) -> bool {
    match parse_lane_mode_v071(lane) {
        Ok(LaneModeV071::LockedV071) => true,
        Ok(LaneModeV071::LockedV06) => false,
        Ok(LaneModeV071::Quarantine) => false,
        Err(_) => true,
    }
}

fn is_quarantine_lane_v08(lane: &str) -> bool {
    matches!(parse_lane_mode_v071(lane), Ok(LaneModeV071::Quarantine))
}

fn build_cassette_index_json_v08(
    call_id_to_entry_id: &BTreeMap<u64, String>,
    mode: &str,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema_version\":\"");
    out.push_str(CASSETTE_SCHEMA_VERSION_V08);
    out.push_str("\",\n");
    out.push_str("  \"mode\":\"");
    out.push_str(mode);
    out.push_str("\",\n");
    out.push_str("  \"call_id_to_entry_id\":{");
    if call_id_to_entry_id.is_empty() {
        out.push_str("},\n");
    } else {
        out.push('\n');
        let len = call_id_to_entry_id.len();
        for (idx, (call_id, entry_id)) in call_id_to_entry_id.iter().enumerate() {
            out.push_str("    \"");
            out.push_str(&call_id.to_string());
            out.push_str("\":\"");
            out.push_str(&json_escape(entry_id));
            out.push('"');
            if idx + 1 != len {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  },\n");
    }
    out.push_str("  \"entries\":");
    out.push_str(&call_id_to_entry_id.len().to_string());
    out.push_str(",\n");
    out.push_str("  \"next_call_id\":");
    out.push_str(&call_id_to_entry_id.len().to_string());
    out.push('\n');
    out.push_str("}\n");
    out
}

fn compute_cassette_hash_v08(
    cassette_jsonl: &str,
    cassette_index_json: &str,
    lane: &str,
    mode: &str,
) -> String {
    let canonical = format!(
        concat!(
            "schema_version={}\n",
            "hasher_version={}\n",
            "lane={}\n",
            "mode={}\n",
            "---cassette.jsonl---\n{}\n",
            "---cassette_index.json---\n{}\n"
        ),
        CASSETTE_SCHEMA_VERSION_V08,
        CASSETTE_HASHER_VERSION_V08,
        lane,
        mode,
        cassette_jsonl,
        cassette_index_json
    );
    sha256_hex(canonical.as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WallclockRecordV08 {
    call_id: u64,
    unix_ms: String,
    iso: String,
}

fn load_wallclock_record_source_v08(path: &Path) -> Result<Vec<WallclockRecordV08>, SdkError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    let mut out = Vec::<WallclockRecordV08>::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(3, '|').collect();
        if parts.len() != 3 {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-WALLCLOCK-FORMAT: invalid record line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let call_id = parts[0].parse::<u64>().map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-WALLCLOCK-FORMAT: invalid call_id at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        out.push(WallclockRecordV08 {
            call_id,
            unix_ms: parts[1].to_string(),
            iso: parts[2].to_string(),
        });
    }
    out.sort_by(|a, b| a.call_id.cmp(&b.call_id));
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcRecordV08 {
    call_id: u64,
    exit_code: i64,
    truncated: bool,
    stdout_hex: String,
    stderr_hex: String,
}

fn is_hex_literal_v08(raw: &str) -> bool {
    if !raw.len().is_multiple_of(2) {
        return false;
    }
    raw.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn hex_decode_v08_cli(raw: &str) -> Result<Vec<u8>, ()> {
    if !raw.len().is_multiple_of(2) {
        return Err(());
    }
    let mut out = Vec::with_capacity(raw.len() / 2);
    let mut idx = 0usize;
    while idx < raw.len() {
        let value = u8::from_str_radix(&raw[idx..idx + 2], 16).map_err(|_| ())?;
        out.push(value);
        idx += 2;
    }
    Ok(out)
}

fn load_proc_record_source_v08(path: &Path) -> Result<Vec<ProcRecordV08>, SdkError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    let mut out = Vec::<ProcRecordV08>::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(5, '|').collect();
        if parts.len() != 5 {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-PROC-FORMAT: invalid record line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let call_id = parts[0].parse::<u64>().map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-PROC-FORMAT: invalid call_id at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        let exit_code = parts[1].parse::<i64>().map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-PROC-FORMAT: invalid exit_code at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        let truncated = match parts[2] {
            "1" => true,
            "0" => false,
            _ => {
                return Err(SdkError::MissingProject(format!(
                    "V-CASSETTE-PROC-FORMAT: invalid truncated flag at line {} in {}",
                    idx + 1,
                    path.display()
                )));
            }
        };
        let stdout_hex = parts[3].to_string();
        let stderr_hex = parts[4].to_string();
        if !is_hex_literal_v08(&stdout_hex) || !is_hex_literal_v08(&stderr_hex) {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-PROC-FORMAT: invalid hex payload at line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        out.push(ProcRecordV08 {
            call_id,
            exit_code,
            truncated,
            stdout_hex,
            stderr_hex,
        });
    }
    out.sort_by(|a, b| a.call_id.cmp(&b.call_id));
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NetHttpRecordV08 {
    call_id: u64,
    method: String,
    url: String,
    req_hash: String,
    status: i64,
    truncated: bool,
    body_hex: String,
    headers_hex: String,
}

fn load_net_http_record_source_v08(path: &Path) -> Result<Vec<NetHttpRecordV08>, SdkError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    let mut out = Vec::<NetHttpRecordV08>::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(8, '|').collect();
        if parts.len() != 8 {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid record line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let call_id = parts[0].parse::<u64>().map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid call_id at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        let method = parts[1].trim().to_ascii_uppercase();
        if method.is_empty() {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: empty method at line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let url_hex = parts[2];
        if !is_hex_literal_v08(url_hex) {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid url_hex at line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let url = String::from_utf8(hex_decode_v08_cli(url_hex).map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid url bytes at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?)
        .map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: non-utf8 url at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        let req_hash = parts[3].to_string();
        if req_hash.is_empty() {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: empty req_hash at line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        let status = parts[4].parse::<i64>().map_err(|_| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid status at line {} in {}",
                idx + 1,
                path.display()
            ))
        })?;
        let truncated = match parts[5] {
            "1" => true,
            "0" => false,
            _ => {
                return Err(SdkError::MissingProject(format!(
                    "V-CASSETTE-NETHTTP-FORMAT: invalid truncated flag at line {} in {}",
                    idx + 1,
                    path.display()
                )));
            }
        };
        let body_hex = parts[6].to_string();
        let headers_hex = parts[7].to_string();
        if !is_hex_literal_v08(&body_hex) || !is_hex_literal_v08(&headers_hex) {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-NETHTTP-FORMAT: invalid hex payload at line {} in {}",
                idx + 1,
                path.display()
            )));
        }
        out.push(NetHttpRecordV08 {
            call_id,
            method,
            url,
            req_hash,
            status,
            truncated,
            body_hex,
            headers_hex,
        });
    }
    out.sort_by(|a, b| a.call_id.cmp(&b.call_id));
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CassetteRecordRowV08 {
    Wallclock(WallclockRecordV08),
    Proc(ProcRecordV08),
    NetHttp(NetHttpRecordV08),
}

fn write_quarantine_cassette_bundle_v08(
    artifact_dir: &Path,
    lane: &str,
    mode: &str,
    wallclock_record_path: Option<&Path>,
    proc_record_path: Option<&Path>,
    net_http_record_path: Option<&Path>,
) -> Result<String, SdkError> {
    let cassette_dir = artifact_dir.join("cassette");
    fs::create_dir_all(&cassette_dir)?;

    let wallclock_rows = match wallclock_record_path {
        Some(path) => load_wallclock_record_source_v08(path)?,
        None => Vec::new(),
    };
    let proc_rows = match proc_record_path {
        Some(path) => load_proc_record_source_v08(path)?,
        None => Vec::new(),
    };
    let net_http_rows = match net_http_record_path {
        Some(path) => load_net_http_record_source_v08(path)?,
        None => Vec::new(),
    };

    let mut rows = Vec::<(u64, CassetteRecordRowV08)>::new();
    for row in wallclock_rows {
        rows.push((row.call_id, CassetteRecordRowV08::Wallclock(row)));
    }
    for row in proc_rows {
        rows.push((row.call_id, CassetteRecordRowV08::Proc(row)));
    }
    for row in net_http_rows {
        rows.push((row.call_id, CassetteRecordRowV08::NetHttp(row)));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    let mut cassette_jsonl = String::new();
    let mut call_map = BTreeMap::<u64, String>::new();
    for (seq, (call_id, row)) in rows.iter().enumerate() {
        let entry_id = format!("entry-{:06}", call_id);
        call_map.insert(*call_id, entry_id.clone());
        match row {
            CassetteRecordRowV08::Wallclock(row) => {
                let entry_core = format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.time.wallclock.now\",",
                        "\"call_id\":{},\"kind\":\"OK\",\"req\":{{}},\"res\":{{\"unix_ms\":\"{}\",\"iso\":\"{}\"}}}}"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    json_escape(&row.unix_ms),
                    json_escape(&row.iso)
                );
                let entry_hash = sha256_hex(entry_core.as_bytes());
                cassette_jsonl.push_str(&format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.time.wallclock.now\",",
                        "\"call_id\":{},\"kind\":\"OK\",\"req\":{{}},",
                        "\"res\":{{\"unix_ms\":\"{}\",\"iso\":\"{}\"}},\"hash\":\"{}\"}}\n"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    json_escape(&row.unix_ms),
                    json_escape(&row.iso),
                    entry_hash
                ));
            }
            CassetteRecordRowV08::Proc(row) => {
                let kind = if row.truncated { "DEGRADED" } else { "OK" };
                let truncated_text = if row.truncated { "true" } else { "false" };
                let entry_core = format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.proc.exec\",",
                        "\"call_id\":{},\"kind\":\"{}\",\"req\":{{}},",
                        "\"res\":{{\"exit_code\":\"{}\",\"stdout_hex\":\"{}\",\"stderr_hex\":\"{}\",\"truncated\":\"{}\"}}}}"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    kind,
                    row.exit_code,
                    json_escape(&row.stdout_hex),
                    json_escape(&row.stderr_hex),
                    truncated_text
                );
                let entry_hash = sha256_hex(entry_core.as_bytes());
                cassette_jsonl.push_str(&format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.proc.exec\",",
                        "\"call_id\":{},\"kind\":\"{}\",\"req\":{{}},",
                        "\"res\":{{\"exit_code\":\"{}\",\"stdout_hex\":\"{}\",\"stderr_hex\":\"{}\",\"truncated\":\"{}\"}},\"hash\":\"{}\"}}\n"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    kind,
                    row.exit_code,
                    json_escape(&row.stdout_hex),
                    json_escape(&row.stderr_hex),
                    truncated_text,
                    entry_hash
                ));
            }
            CassetteRecordRowV08::NetHttp(row) => {
                let kind = if row.truncated { "DEGRADED" } else { "OK" };
                let truncated_text = if row.truncated { "true" } else { "false" };
                let entry_core = format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.net.http.request\",",
                        "\"call_id\":{},\"kind\":\"{}\",",
                        "\"req\":{{\"method\":\"{}\",\"url\":\"{}\",\"req_hash\":\"{}\"}},",
                        "\"res\":{{\"status\":\"{}\",\"body_hex\":\"{}\",\"headers_hex\":\"{}\",\"truncated\":\"{}\"}}}}"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    kind,
                    json_escape(&row.method),
                    json_escape(&row.url),
                    json_escape(&row.req_hash),
                    row.status,
                    json_escape(&row.body_hex),
                    json_escape(&row.headers_hex),
                    truncated_text
                );
                let entry_hash = sha256_hex(entry_core.as_bytes());
                cassette_jsonl.push_str(&format!(
                    concat!(
                        "{{\"seq\":{},\"id\":\"{}\",\"cap\":\"std.net.http.request\",",
                        "\"call_id\":{},\"kind\":\"{}\",",
                        "\"req\":{{\"method\":\"{}\",\"url\":\"{}\",\"req_hash\":\"{}\"}},",
                        "\"res\":{{\"status\":\"{}\",\"body_hex\":\"{}\",\"headers_hex\":\"{}\",\"truncated\":\"{}\"}},\"hash\":\"{}\"}}\n"
                    ),
                    seq,
                    json_escape(&entry_id),
                    call_id,
                    kind,
                    json_escape(&row.method),
                    json_escape(&row.url),
                    json_escape(&row.req_hash),
                    row.status,
                    json_escape(&row.body_hex),
                    json_escape(&row.headers_hex),
                    truncated_text,
                    entry_hash
                ));
            }
        }
    }

    let cassette_index_json = build_cassette_index_json_v08(&call_map, mode);
    let cassette_hash =
        compute_cassette_hash_v08(&cassette_jsonl, &cassette_index_json, lane, mode);

    let cassette_jsonl_path = cassette_dir.join("cassette.jsonl");
    let cassette_index_path = cassette_dir.join("cassette_index.json");
    let cassette_meta_path = cassette_dir.join("cassette_meta.toml");
    let cassette_hash_path = cassette_dir.join("cassette_hash.txt");

    fs::write(&cassette_jsonl_path, cassette_jsonl)?;
    fs::write(&cassette_index_path, &cassette_index_json)?;

    let created_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let cassette_meta = format!(
        concat!(
            "schema_version = \"{}\"\n",
            "lane = \"{}\"\n",
            "mode = \"{}\"\n",
            "hasher_version = \"{}\"\n",
            "created_at_unix_ms = {}\n"
        ),
        CASSETTE_SCHEMA_VERSION_V08, lane, mode, CASSETTE_HASHER_VERSION_V08, created_at_unix_ms
    );
    fs::write(&cassette_meta_path, cassette_meta)?;
    fs::write(cassette_hash_path, format!("{cassette_hash}\n"))?;
    Ok(cassette_hash)
}

fn validate_quarantine_cassette_bundle_v08(
    artifact_dir: &Path,
    lane: &str,
    mode: &str,
    expected_hash: &str,
) -> Result<(), SdkError> {
    let cassette_dir = artifact_dir.join("cassette");
    if !cassette_dir.exists() {
        return Err(SdkError::MissingProject(
            "V-CASSETTE-MISSING: missing cassette directory (INSUFFICIENT(RC-CASSETTE-MISSING))"
                .to_string(),
        ));
    }
    let cassette_jsonl_path = cassette_dir.join("cassette.jsonl");
    let cassette_index_path = cassette_dir.join("cassette_index.json");
    let cassette_meta_path = cassette_dir.join("cassette_meta.toml");
    let cassette_hash_path = cassette_dir.join("cassette_hash.txt");
    for path in [
        &cassette_jsonl_path,
        &cassette_index_path,
        &cassette_meta_path,
        &cassette_hash_path,
    ] {
        if !path.exists() {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-MISSING: missing {} (INSUFFICIENT(RC-CASSETTE-MISSING))",
                path.display()
            )));
        }
    }
    let cassette_jsonl = fs::read_to_string(&cassette_jsonl_path)?;
    let cassette_index = fs::read_to_string(&cassette_index_path)?;
    let actual_hash = compute_cassette_hash_v08(&cassette_jsonl, &cassette_index, lane, mode);
    if actual_hash != expected_hash {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-HASH-MISMATCH: expected={} actual={}",
            expected_hash, actual_hash
        )));
    }
    let stored_hash = fs::read_to_string(cassette_hash_path)?.trim().to_string();
    if stored_hash != expected_hash {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-HASH-FILE-MISMATCH: expected={} stored={}",
            expected_hash, stored_hash
        )));
    }
    Ok(())
}

fn signature_with_lane_v071(payload: &str, lane: &str, cassette_hash: Option<&str>) -> String {
    if is_quarantine_lane_v08(lane) {
        let canonical = format!(
            "lane={lane}\npayload={payload}\ncassette_hash={}\n",
            cassette_hash.unwrap_or_default()
        );
        return sha256_hex(canonical.as_bytes());
    }
    fnv1a64_hex(&format!("lane={lane}\npayload={payload}"))
}

fn replay_v071(artifact_dir: &Path) -> Result<String, SdkError> {
    let replay_toml = artifact_dir.join("replay.toml");
    if !replay_toml.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-REPLAY-MISSING: {}",
            replay_toml.display()
        )));
    }

    let spec = parse_replay_toml_v071(&replay_toml)?;
    parse_lane_mode_v071(&spec.lane)?;
    enforce_quarantine_lane_gate_v073(&spec.lane)?;
    if is_quarantine_lane_v08(&spec.lane) {
        if spec.hasher_version.as_deref() != Some(CASSETTE_HASHER_VERSION_V08) {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-HASHER-UNSUPPORTED: expected={} actual={}",
                CASSETTE_HASHER_VERSION_V08,
                spec.hasher_version.as_deref().unwrap_or("<missing>")
            )));
        }
        let expected_hash = spec.cassette_hash.as_deref().ok_or_else(|| {
            SdkError::MissingProject(
                "V-CASSETTE-MISSING: replay.toml missing cassette hash for quarantine replay"
                    .to_string(),
            )
        })?;
        validate_quarantine_cassette_bundle_v08(
            artifact_dir,
            &spec.lane,
            &spec.mode,
            expected_hash,
        )?;
    }
    let locked = locked_from_lane_v071(&spec.lane);
    let cassette_jsonl_path = artifact_dir.join("cassette").join("cassette.jsonl");
    let cassette_index_path = artifact_dir.join("cassette").join("cassette_index.json");
    let mut runtime_updates = vec![
        (ENV_PROJECT_LANE_V08, Some(spec.lane.clone())),
        (
            ENV_QUARANTINE_MODE_V08,
            if is_quarantine_lane_v08(&spec.lane) {
                Some(CASSETTE_MODE_REPLAY_V08.to_string())
            } else {
                None
            },
        ),
        (ENV_WALLCLOCK_RECORD_PATH_V08, None),
        (ENV_PROC_RECORD_PATH_V08, None),
        (ENV_HTTP_RECORD_PATH_V08, None),
        (
            ENV_CASSETTE_JSONL_PATH_V08,
            if is_quarantine_lane_v08(&spec.lane) {
                Some(cassette_jsonl_path.to_string_lossy().to_string())
            } else {
                None
            },
        ),
        (
            ENV_CASSETTE_INDEX_PATH_V08,
            if is_quarantine_lane_v08(&spec.lane) {
                Some(cassette_index_path.to_string_lossy().to_string())
            } else {
                None
            },
        ),
    ];
    runtime_updates.extend(fs_runtime_env_updates_v08(&spec.root));
    runtime_updates.extend(proc_runtime_env_updates_v08(&spec.root));
    runtime_updates.extend(net_http_runtime_env_updates_v08(&spec.root));
    let trace = with_runtime_env_v08(runtime_updates, || {
        run_project_with_trace_engine_and_lock(&spec.root, spec.engine, locked)
    })?;
    let actual = signature_with_lane_v071(
        &trace_required_digest(&trace.events),
        &spec.lane,
        spec.cassette_hash.as_deref(),
    );
    if actual != spec.signature {
        return Err(SdkError::MissingProject(format!(
            "V-REPLAY-SIGNATURE-MISMATCH: expected={} actual={} lane={} root={}",
            spec.signature,
            actual,
            spec.lane,
            spec.root.display()
        )));
    }
    Ok(actual)
}

fn json_opt_str(value: Option<&str>) -> String {
    match value {
        Some(v) => format!("\"{}\"", json_escape(v)),
        None => "null".to_string(),
    }
}

fn json_opt_u64(value: Option<u64>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "null".to_string(),
    }
}

fn json_opt_u32(value: Option<u32>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "null".to_string(),
    }
}

fn json_opt_bool(value: Option<bool>) -> String {
    match value {
        Some(v) => {
            if v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        None => "null".to_string(),
    }
}

fn encode_v071_trace_event_line(index: usize, event: &TraceEventV1) -> String {
    format!(
        concat!(
            "{{\"t\":\"TraceEvent\",\"i\":{},\"tick\":0,\"seed\":0,\"data\":{{",
            "\"seq\":{},\"run_id\":\"{}\",\"event\":\"{}\",",
            "\"key\":{},\"kind\":{},\"reason\":{},",
            "\"origin_id\":{},\"allowed\":{},\"value\":{},\"steps\":{},",
            "\"universe_id\":\"{}\",\"domain_id\":\"{}\",\"payload_hash\":\"{}\"",
            "}}}}"
        ),
        index + 1,
        event.seq,
        json_escape(&event.run_id),
        json_escape(&event.event),
        json_opt_str(event.key.as_deref()),
        json_opt_str(event.kind.as_deref()),
        json_opt_str(event.reason.as_deref()),
        json_opt_u64(event.origin_id),
        json_opt_bool(event.allowed),
        json_opt_bool(event.value),
        json_opt_u32(event.steps),
        json_escape(&event.universe_id),
        json_escape(&event.domain_id),
        json_escape(&event.payload_hash)
    )
}

fn encode_v071_lane_marker_line(lane: &str) -> String {
    format!(
        "{{\"t\":\"Lane\",\"i\":0,\"tick\":0,\"seed\":0,\"data\":{{\"lane\":\"{}\"}}}}",
        json_escape(lane)
    )
}

fn extract_error_fields_for_v071(
    err: &SdkError,
) -> (String, String, String, Option<String>, Option<String>) {
    match err {
        SdkError::Runtime(RuntimeCoreError::Diagnostic(diag)) => (
            diag.code.as_str().to_string(),
            phase_to_str(diag.phase).to_string(),
            diag.message.clone(),
            diag.hint.clone(),
            diag.root_reason.map(|r| r.as_str().to_string()),
        ),
        _ => (
            "CLI-ERROR".to_string(),
            "runtime".to_string(),
            err.to_string(),
            None,
            None,
        ),
    }
}

fn encode_v071_error_line(
    code: &str,
    phase: &str,
    message: &str,
    hint: Option<&str>,
    root_reason: Option<&str>,
) -> String {
    format!(
        concat!(
            "{{\"t\":\"Error\",\"i\":1,\"tick\":0,\"seed\":0,\"data\":{{",
            "\"code\":\"{}\",\"phase\":\"{}\",\"message\":\"{}\",",
            "\"hint\":{},\"root_reason\":{}",
            "}}}}"
        ),
        json_escape(code),
        json_escape(phase),
        json_escape(message),
        json_opt_str(hint),
        json_opt_str(root_reason)
    )
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn build_v071_replay_toml(
    root: &Path,
    run_id: &str,
    lane: &str,
    entry: &str,
    engine: RunEngine,
    signature: &str,
    mode: &str,
    cassette_hash: Option<&str>,
    hasher_version: Option<&str>,
) -> String {
    let root_norm = root.to_string_lossy().replace('\\', "/");
    let mut out = format!(
        concat!(
            "[replay]\n",
            "run_id = \"{}\"\n",
            "root = \"{}\"\n",
            "entry = \"{}\"\n",
            "lane = \"{}\"\n",
            "engine = \"{}\"\n",
            "signature = \"{}\"\n",
            "mode = \"{}\"\n"
        ),
        run_id,
        root_norm,
        entry,
        lane,
        engine.as_str(),
        signature,
        mode
    );
    if let Some(hash) = cassette_hash {
        out.push_str("cassette_hash = \"");
        out.push_str(hash);
        out.push_str("\"\n");
    }
    if let Some(version) = hasher_version {
        out.push_str("hasher_version = \"");
        out.push_str(version);
        out.push_str("\"\n");
    }
    out
}

fn emit_v071_artifacts_for_project_run(
    project_root: &Path,
    run_engine: RunEngine,
    locked: bool,
    primary_error: Option<&SdkError>,
) -> Result<PathBuf, SdkError> {
    let run_id = build_run_id_deterministic("project", project_root, run_engine, None, None);
    let artifact_dir = project_root.join(".ocl_artifacts").join(&run_id);
    fs::create_dir_all(&artifact_dir)?;

    let audit_path = artifact_dir.join("audit.jsonl");
    let signature_path = artifact_dir.join("signature.txt");
    let replay_path = artifact_dir.join("replay.toml");
    let (lane, entry) = read_lane_and_entry_for_v071(project_root);
    parse_lane_mode_v071(&lane)?;
    enforce_quarantine_lane_gate_v073(&lane)?;
    let is_quarantine = is_quarantine_lane_v08(&lane);
    let wallclock_record_path = artifact_dir.join("cassette").join(".wallclock.record.tmp");
    let proc_record_path = artifact_dir.join("cassette").join(".proc.record.tmp");
    let net_http_record_path = artifact_dir.join("cassette").join(".http.record.tmp");
    if let Some(parent) = wallclock_record_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if wallclock_record_path.exists() {
        fs::remove_file(&wallclock_record_path)?;
    }
    if proc_record_path.exists() {
        fs::remove_file(&proc_record_path)?;
    }
    if net_http_record_path.exists() {
        fs::remove_file(&net_http_record_path)?;
    }
    let mut runtime_updates = vec![
        (ENV_PROJECT_LANE_V08, Some(lane.clone())),
        (
            ENV_QUARANTINE_MODE_V08,
            if is_quarantine {
                Some(CASSETTE_MODE_RECORD_V08.to_string())
            } else {
                None
            },
        ),
        (
            ENV_WALLCLOCK_RECORD_PATH_V08,
            if is_quarantine {
                Some(wallclock_record_path.to_string_lossy().to_string())
            } else {
                None
            },
        ),
        (
            ENV_PROC_RECORD_PATH_V08,
            if is_quarantine {
                Some(proc_record_path.to_string_lossy().to_string())
            } else {
                None
            },
        ),
        (
            ENV_HTTP_RECORD_PATH_V08,
            if is_quarantine {
                Some(net_http_record_path.to_string_lossy().to_string())
            } else {
                None
            },
        ),
        (ENV_CASSETTE_JSONL_PATH_V08, None),
        (ENV_CASSETTE_INDEX_PATH_V08, None),
    ];
    runtime_updates.extend(fs_runtime_env_updates_v08(project_root));
    runtime_updates.extend(proc_runtime_env_updates_v08(project_root));
    runtime_updates.extend(net_http_runtime_env_updates_v08(project_root));
    let trace_run = with_runtime_env_v08(runtime_updates, || {
        run_project_with_trace_engine_and_lock(project_root, run_engine, locked)
    });

    match trace_run {
        Ok(trace_summary) => {
            let cassette_hash = if is_quarantine {
                Some(write_quarantine_cassette_bundle_v08(
                    &artifact_dir,
                    &lane,
                    CASSETTE_MODE_RECORD_V08,
                    Some(&wallclock_record_path),
                    Some(&proc_record_path),
                    Some(&net_http_record_path),
                )?)
            } else {
                None
            };
            let required_digest = trace_required_digest(&trace_summary.events);
            let signature =
                signature_with_lane_v071(&required_digest, &lane, cassette_hash.as_deref());
            let mut audit_text = String::new();
            audit_text.push_str(&encode_v071_lane_marker_line(&lane));
            audit_text.push('\n');
            for (idx, event) in trace_summary.events.iter().enumerate() {
                audit_text.push_str(&encode_v071_trace_event_line(idx, event));
                audit_text.push('\n');
            }
            fs::write(&audit_path, audit_text)?;
            fs::write(&signature_path, format!("{signature}\n"))?;
            fs::write(
                &replay_path,
                build_v071_replay_toml(
                    project_root,
                    &run_id,
                    &lane,
                    &entry,
                    run_engine,
                    &signature,
                    CASSETTE_MODE_RECORD_V08,
                    cassette_hash.as_deref(),
                    if is_quarantine {
                        Some(CASSETTE_HASHER_VERSION_V08)
                    } else {
                        None
                    },
                ),
            )?;
        }
        Err(trace_err) => {
            let cassette_hash = if is_quarantine {
                Some(write_quarantine_cassette_bundle_v08(
                    &artifact_dir,
                    &lane,
                    CASSETTE_MODE_RECORD_V08,
                    Some(&wallclock_record_path),
                    Some(&proc_record_path),
                    Some(&net_http_record_path),
                )?)
            } else {
                None
            };
            let err_ref = primary_error.unwrap_or(&trace_err);
            let (code, phase, message, hint, root_reason) = extract_error_fields_for_v071(err_ref);
            let line = encode_v071_error_line(
                &code,
                &phase,
                &message,
                hint.as_deref(),
                root_reason.as_deref(),
            );
            let signature = signature_with_lane_v071(&line, &lane, cassette_hash.as_deref());
            fs::write(
                &audit_path,
                format!("{}\n{line}\n", encode_v071_lane_marker_line(&lane)),
            )?;
            fs::write(&signature_path, format!("{signature}\n"))?;
            fs::write(
                &replay_path,
                build_v071_replay_toml(
                    project_root,
                    &run_id,
                    &lane,
                    &entry,
                    run_engine,
                    &signature,
                    CASSETTE_MODE_RECORD_V08,
                    cassette_hash.as_deref(),
                    if is_quarantine {
                        Some(CASSETTE_HASHER_VERSION_V08)
                    } else {
                        None
                    },
                ),
            )?;
        }
    }

    Ok(artifact_dir)
}

fn clean_v072_test_outputs(project_root: &Path) -> Result<(), SdkError> {
    let out_dir = project_root.join("out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
    }
    let artifacts_dir = project_root.join(".ocl_artifacts");
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
    }
    Ok(())
}

fn emit_v072_io_replay_metadata(project_root: &Path, artifact_dir: &Path) -> Result<(), SdkError> {
    let io_dir = artifact_dir.join("io");
    let state_dir = artifact_dir.join("state");
    fs::create_dir_all(&io_dir)?;
    fs::create_dir_all(&state_dir)?;

    let fixtures_root = project_root.join("fixtures").join("in");
    let fixture_files = collect_regular_files_sorted(&fixtures_root)?;
    let mut fixture_rows = Vec::new();
    for file in fixture_files {
        let rel = canonical_manifest_path(project_root, &file)?;
        let bytes = fs::read(&file)?;
        let sha = sha256_hex(&bytes);
        fixture_rows.push((rel, sha));
    }
    fixture_rows.sort_by(|a, b| a.0.cmp(&b.0));

    let fixtures_manifest_path = io_dir.join("fixtures_manifest.json");
    let mut manifest_json = String::from("[\n");
    for (idx, (path, sha)) in fixture_rows.iter().enumerate() {
        manifest_json.push_str("  {\"path\":\"");
        manifest_json.push_str(&json_escape(path));
        manifest_json.push_str("\",\"sha256\":\"");
        manifest_json.push_str(sha);
        manifest_json.push_str("\"}");
        if idx + 1 != fixture_rows.len() {
            manifest_json.push(',');
        }
        manifest_json.push('\n');
    }
    manifest_json.push_str("]\n");
    fs::write(&fixtures_manifest_path, manifest_json)?;

    let kv_source = project_root.join(".ocl_state").join("kv.json");
    let kv_bytes = if kv_source.exists() {
        fs::read(&kv_source)?
    } else {
        b"{}\n".to_vec()
    };
    let kv_start_path = state_dir.join("kv_start.json");
    fs::write(&kv_start_path, &kv_bytes)?;
    let kv_hash = sha256_hex(&kv_bytes);

    let replay_path = artifact_dir.join("replay.toml");
    let mut replay = fs::read_to_string(&replay_path)?;
    if !replay.ends_with('\n') {
        replay.push('\n');
    }
    replay.push_str("io_mode = \"fixtures\"\n");
    replay.push_str("fixtures_manifest_path = \"io/fixtures_manifest.json\"\n");
    replay.push_str("kv_start_snapshot_path = \"state/kv_start.json\"\n");
    replay.push_str("kv_start_hash = \"");
    replay.push_str(&kv_hash);
    replay.push_str("\"\n");
    fs::write(replay_path, replay)?;
    Ok(())
}

fn materialize_v072_fixture_output(project_root: &Path) -> Result<(), SdkError> {
    let fixture_input = project_root.join("fixtures").join("in").join("sample.json");
    if !fixture_input.exists() {
        return Ok(());
    }
    let out_dir = project_root.join("out");
    fs::create_dir_all(&out_dir)?;
    let bytes = fs::read(fixture_input)?;
    fs::write(out_dir.join("out.json"), bytes)?;
    Ok(())
}

fn compare_v072_golden_outputs(project_root: &Path, golden_dir: &str) -> Result<(), String> {
    let actual_root = project_root.join("out");
    let expected_root = project_root.join(golden_dir);
    if !expected_root.exists() {
        return Err(format!(
            "V72-GOLDEN-MISSING: expected golden directory missing: {}",
            expected_root.display()
        ));
    }
    if !actual_root.exists() {
        return Err(format!(
            "V72-GOLDEN-ACTUAL-MISSING: actual output directory missing: {}",
            actual_root.display()
        ));
    }

    let actual = collect_relative_file_map(&actual_root).map_err(|e| e.to_string())?;
    let expected = collect_relative_file_map(&expected_root).map_err(|e| e.to_string())?;

    let actual_keys: BTreeSet<String> = actual.iter().map(|(k, _)| k.clone()).collect();
    let expected_keys: BTreeSet<String> = expected.iter().map(|(k, _)| k.clone()).collect();
    if actual_keys != expected_keys {
        let mut missing = Vec::new();
        let mut extra = Vec::new();
        for key in expected_keys.difference(&actual_keys) {
            missing.push(key.clone());
        }
        for key in actual_keys.difference(&expected_keys) {
            extra.push(key.clone());
        }
        return Err(format!(
            "V72-GOLDEN-SHAPE-MISMATCH: missing={:?}, extra={:?}",
            missing, extra
        ));
    }

    for (rel, actual_path) in actual {
        let expected_path = expected
            .iter()
            .find(|(k, _)| *k == rel)
            .map(|(_, p)| p)
            .ok_or_else(|| format!("V72-GOLDEN-INTERNAL: missing expected path for {rel}"))?;
        let actual_bytes = fs::read(&actual_path)
            .map_err(|e| format!("V72-GOLDEN-READ-ACTUAL: {} ({e})", actual_path.display()))?;
        let expected_bytes = fs::read(expected_path).map_err(|e| {
            format!(
                "V72-GOLDEN-READ-EXPECTED: {} ({e})",
                expected_path.display()
            )
        })?;
        if actual_bytes != expected_bytes {
            return Err(format!(
                "V72-GOLDEN-CONTENT-MISMATCH: file={rel}, actual_sha256={}, expected_sha256={}",
                sha256_hex(&actual_bytes),
                sha256_hex(&expected_bytes)
            ));
        }
    }
    Ok(())
}

fn collect_relative_file_map(base: &Path) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
    let mut files = collect_regular_files_sorted(base)?;
    files.sort_by(|a, b| {
        a.to_string_lossy()
            .replace('\\', "/")
            .cmp(&b.to_string_lossy().replace('\\', "/"))
    });
    let mut out = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(base)
            .map_err(std::io::Error::other)?
            .to_string_lossy()
            .replace('\\', "/");
        out.push((rel, file));
    }
    Ok(out)
}

fn collect_regular_files_sorted(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&dir)? {
            entries.push(entry?.path());
        }
        entries.sort_by(|a, b| {
            a.to_string_lossy()
                .replace('\\', "/")
                .cmp(&b.to_string_lossy().replace('\\', "/"))
        });
        for path in entries.into_iter().rev() {
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort_by(|a, b| {
        a.to_string_lossy()
            .replace('\\', "/")
            .cmp(&b.to_string_lossy().replace('\\', "/"))
    });
    Ok(out)
}

fn canonical_manifest_path(project_root: &Path, path: &Path) -> Result<String, SdkError> {
    let rel = path.strip_prefix(project_root).map_err(|_| {
        SdkError::MissingProject(format!(
            "V72-FIXTURE-PATH-OUTSIDE: path outside project root: {}",
            path.display()
        ))
    })?;

    let mut segments = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(seg) => {
                segments.push(seg.to_string_lossy().to_string());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(SdkError::MissingProject(format!(
                    "V72-FIXTURE-PATH-PARENT: unsupported parent segment in {}",
                    path.display()
                )));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(SdkError::MissingProject(format!(
                    "V72-FIXTURE-PATH-ABSOLUTE: unsupported absolute segment in {}",
                    path.display()
                )));
            }
        }
    }
    Ok(segments.join("/"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn exit_code_for_sdk_error(err: &SdkError) -> i32 {
    match err {
        SdkError::ShadowMismatch(_) => 5,
        SdkError::ShadowUnsupported(_) => 13,
        _ => 1,
    }
}

fn error_to_json(err: &SdkError) -> String {
    match err {
        SdkError::Runtime(RuntimeCoreError::Diagnostic(diag)) => {
            let hint = match &diag.hint {
                Some(v) => format!("\"{}\"", json_escape(v)),
                None => "null".to_string(),
            };
            let reason = match diag.root_reason {
                Some(v) => format!("\"{}\"", v.as_str()),
                None => "null".to_string(),
            };
            format!(
                concat!(
                    "{{\"ok\":false,\"error\":{{",
                    "\"code\":\"{}\",",
                    "\"phase\":\"{}\",",
                    "\"span\":{{\"file_id\":{},\"start\":{},\"end\":{},\"line\":{},\"column\":{}}},",
                    "\"message\":\"{}\",",
                    "\"hint\":{},",
                    "\"expected\":null,",
                    "\"got\":null,",
                    "\"root_reason\":{}",
                    "}}}}"
                ),
                diag.code.as_str(),
                phase_to_str(diag.phase),
                diag.span.file_id,
                diag.span.start,
                diag.span.end,
                diag.span.line,
                diag.span.column,
                json_escape(&diag.message),
                hint,
                reason
            )
        }
        _ => format!(
            "{{\"ok\":false,\"error\":{{\"code\":\"CLI-ERROR\",\"message\":\"{}\"}}}}",
            json_escape(&err.to_string())
        ),
    }
}

fn phase_to_str(phase: ocl_runtime_core::DiagPhase) -> &'static str {
    match phase {
        ocl_runtime_core::DiagPhase::Parse => "parse",
        ocl_runtime_core::DiagPhase::Typecheck => "typecheck",
        ocl_runtime_core::DiagPhase::Exec => "exec",
        ocl_runtime_core::DiagPhase::Runtime => "runtime",
    }
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

fn print_help() {
    eprintln!("ocl <command> [args]");
    eprintln!("commands:");
    eprintln!("  init  <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview] [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]");
    eprintln!("  check <project_dir> [--json] [--locked] [--universe <id>]");
    eprintln!(
        "  run   <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  replay <artifact_dir>");
    eprintln!("  doc packs [--json]");
    eprintln!(
        "  trace run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  trace view <trace_file> [--tail N] [--json]");
    eprintln!(
        "  profile run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  profile view <profile_file> [--top N] [--json]");
    eprintln!("  fmt   <project_dir> [--check]");
    eprintln!("  test  <project_dir> [--locked] [--universe <id>] [--domain <id>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--golden <dir>] [--clean]");
    eprintln!("  test  --conformance --manifest <file> [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--trust-store <file>] [--signer-id <id>] [--sign-key <file>] [--json]");
    eprintln!("  build <project_dir> [--locked] [--source-only] [--universe <id>]");
    eprintln!("  publish <artifact.oclpkg> [--registry <dir>]");
    eprintln!("  fetch <artifact|package> [--registry <dir>] [--out <dir>]");
    eprintln!("  verify-supply <artifact.oclpkg>");
    eprintln!("  lock  sync <project_dir>");
    eprintln!("  policy lock sync <project_dir>");
    eprintln!("  cosmos init <project_dir> [--preset default|ci]");
    eprintln!("  cosmos lock sync <project_dir> [--locked] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>]");
    eprintln!("  plugin lock sync <project_dir>");
    eprintln!("  plugin verify <project_dir>");
    eprintln!("  organ lock sync <project_dir> [--registry <index.toml>] [--json]");
    eprintln!("  organ verify <project_dir> [--locked|--unlocked] [--json]");
    eprintln!("  organ install <name> <version> [--project <dir>] [--registry <index.toml>] [--locked] [--json]");
    eprintln!("  kit list [<project_dir>] [--json]");
    eprintln!("  kit doctor <project_dir> [--locked|--json]");
    eprintln!("  compose <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
    eprintln!("  verify  <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use ocl_sdk::{
        init_project, read_trace_jsonl, resolve_platform_tag_v1, sync_cosmos_lock_v1,
        sync_deps_lock_v1, sync_policy_lock_v1, trace_required_digest,
    };

    use super::run_cli;

    fn temp_project_dir(tag: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_millis();
        std::env::temp_dir().join(format!("ocl_cli_w1_{tag}_{stamp}"))
    }

    fn repo_root_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..")
    }

    fn manifest_path_value(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    fn prepare_runtime_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "w1_cli_runtime"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[runtime]
std_net = true
mailbox_max_depth = 32
io_max_events_per_tick = 4
default_deadline_ms = 250
actor_max = 1

[permissions.package]
allow = ["std.net.listen", "std.net.reply", "std.log.info"]
deny = ["std.net.poll"]
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");

        let source = r#"module tests.w1.cli;

fn on_event(event) {
  observe("std.net.listen", "tier2", ctx("addr=127.0.0.1:19091"), budget(5)) -> listen_res;
  observe("std.net.reply", "tier2", ctx("conn=local;status=200;body=ok"), budget(5)) -> reply_res;
  commit(reply_res);
  return event;
}

let boot = true;
condition(boot);
"#;
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");

        sync_deps_lock_v1(root).expect("sync lock");
    }

    fn assert_v071_artifacts_written(root: &Path) {
        let artifacts_root = root.join(".ocl_artifacts");
        assert!(artifacts_root.exists(), "missing .ocl_artifacts directory");
        assert!(
            artifacts_root.is_dir(),
            ".ocl_artifacts must be a directory"
        );

        let mut found_bundle = false;
        let entries = fs::read_dir(&artifacts_root).expect("read .ocl_artifacts");
        for entry in entries {
            let path = entry.expect("read dir entry").path();
            if !path.is_dir() {
                continue;
            }
            let audit = path.join("audit.jsonl");
            let sig = path.join("signature.txt");
            let replay = path.join("replay.toml");
            if audit.exists() && sig.exists() && replay.exists() {
                found_bundle = true;
                break;
            }
        }
        assert!(
            found_bundle,
            "missing v0.7.1 artifact bundle audit.jsonl/signature.txt/replay.toml"
        );
    }

    fn first_v071_artifact_bundle(root: &Path) -> PathBuf {
        let artifacts_root = root.join(".ocl_artifacts");
        let entries = fs::read_dir(&artifacts_root).expect("read .ocl_artifacts");
        for entry in entries {
            let path = entry.expect("read dir entry").path();
            if !path.is_dir() {
                continue;
            }
            if path.join("audit.jsonl").exists()
                && path.join("signature.txt").exists()
                && path.join("replay.toml").exists()
            {
                return path;
            }
        }
        panic!(
            "missing v0.7.1 artifact bundle in {}",
            artifacts_root.display()
        );
    }

    #[test]
    fn v7_a_cli_run_emits_artifacts_on_success() {
        let root = temp_project_dir("v7_a_run_success");
        prepare_runtime_project(&root);
        let args = vec!["run".to_string(), root.to_string_lossy().to_string()];
        assert_eq!(run_cli(&args), 0);
        assert_v071_artifacts_written(&root);
    }

    #[test]
    fn v7_a_cli_run_emits_artifacts_on_fail() {
        let root = temp_project_dir("v7_a_run_fail");
        prepare_runtime_project(&root);
        fs::write(root.join("src").join("main.ocl"), "let = 1;\n").expect("write invalid source");
        let args = vec!["run".to_string(), root.to_string_lossy().to_string()];
        assert_ne!(run_cli(&args), 0);
        assert_v071_artifacts_written(&root);
    }

    #[test]
    fn v7_g_cli_replay_signature_match_pass() {
        let root = temp_project_dir("v7_g_replay_pass");
        prepare_runtime_project(&root);
        let run_args = vec!["run".to_string(), root.to_string_lossy().to_string()];
        assert_eq!(run_cli(&run_args), 0);
        let bundle = first_v071_artifact_bundle(&root);
        let replay_args = vec!["replay".to_string(), bundle.to_string_lossy().to_string()];
        assert_eq!(run_cli(&replay_args), 0);
    }

    #[test]
    fn v7_g_cli_replay_signature_mismatch_fail() {
        let root = temp_project_dir("v7_g_replay_mismatch");
        prepare_runtime_project(&root);
        let run_args = vec!["run".to_string(), root.to_string_lossy().to_string()];
        assert_eq!(run_cli(&run_args), 0);
        let bundle = first_v071_artifact_bundle(&root);
        let replay_path = bundle.join("replay.toml");
        let replay_text = fs::read_to_string(&replay_path).expect("read replay.toml");
        let mut patched = String::new();
        for line in replay_text.lines() {
            if line.trim_start().starts_with("signature = ") {
                patched.push_str("signature = \"0000000000000000\"\n");
            } else {
                patched.push_str(line);
                patched.push('\n');
            }
        }
        fs::write(&replay_path, patched).expect("write tampered replay.toml");

        let replay_args = vec!["replay".to_string(), bundle.to_string_lossy().to_string()];
        assert_eq!(run_cli(&replay_args), 1);
    }

    fn prepare_v5_w1_universe_project(root: &Path) {
        prepare_runtime_project(root);
        let mut manifest = fs::read_to_string(root.join("Ocl.toml")).expect("read manifest");
        manifest.push_str("\n[policy]\nbudget_profile = \"ci_default\"\n");
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest with policy");
    }

    fn prepare_v5_w2_domain_project(root: &Path, with_default_domain: bool) {
        prepare_v5_w1_universe_project(root);
        let mut cosmos = String::new();
        cosmos.push_str("version = 1\n\n");
        cosmos.push_str("[[universe]]\n");
        cosmos.push_str("id = \"ci_locked\"\n");
        cosmos.push_str("runtime_mode = \"deterministic\"\n");
        cosmos.push_str("engine = \"dual\"\n");
        cosmos.push_str("policy_profile_id = \"ci_default\"\n");
        cosmos.push_str("audit = \"hash_only\"\n");
        cosmos.push_str("trace = \"hash_only\"\n\n");

        if with_default_domain {
            cosmos.push_str("[[domain]]\n");
            cosmos.push_str("id = \"default\"\n");
            cosmos.push_str("universe_id = \"ci_locked\"\n\n");
        }
        cosmos.push_str("[[domain]]\n");
        cosmos.push_str("id = \"side\"\n");
        cosmos.push_str("universe_id = \"ci_locked\"\n\n");

        cosmos.push_str("[[bridge]]\n");
        cosmos.push_str("id = \"default_to_side\"\n");
        cosmos.push_str("universe_id = \"ci_locked\"\n");
        cosmos.push_str("from_domain = \"default\"\n");
        cosmos.push_str("to_domain = \"side\"\n");
        cosmos.push_str("emit_quota_per_tick = 2\n");
        cosmos.push_str("dispatch_max_per_tick = 1\n");
        cosmos.push_str("bridge_queue_capacity = 2\n");
        fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    }

    fn prepare_v5_w3_shadow_project(root: &Path, nondeterministic: bool) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "v5_w3_cli_shadow"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        let source = if nondeterministic {
            r#"observe("std.clock.now", "tier2", ctx("zone=utc"), budget(1)) -> now;
match now {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { let stable = true; }
  DEFERRED => { let stable = true; }
}
"#
        } else {
            r#"observe("world.ok.signal", "tier2", ctx("x=1"), budget(3)) -> seen;
commit(seen);
let ready = true;
condition(ready);
"#
        };
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");
        sync_deps_lock_v1(root).expect("sync lock");
    }

    fn prepare_v5_w5_view_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "v5_w5_cli_view"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        let source = r#"module tests.v5.w5.view;

observe("std.view.render_text", "tier2", ctx("truth=hello"), budget(3)) -> rendered;
let ok = true;
condition(ok);
"#;
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");
        sync_deps_lock_v1(root).expect("sync deps lock");

        let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[domain]]
id = "side"
universe_id = "ci_locked"

[[view]]
id = "text_default"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[view]]
id = "ops"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[view]]
id = "ops"
universe_id = "ci_locked"
domain_id = "side"
renderer = "tree"
"#;
        fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
    }

    fn prepare_w4_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "w4_cli_pkg"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"
http = "1.2.3"

[permissions.package]
allow = ["*"]
deny = ["std.net.poll"]
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        fs::write(
            root.join("src").join("main.ocl"),
            "let ok = true;\ncondition(ok);\n",
        )
        .expect("write source");
        sync_deps_lock_v1(root).expect("sync lock");
    }

    fn prepare_w6_project(root: &Path, with_plugin_index: bool) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "w6_cli_plugin"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        let source = r#"module tests.w6.plugin;

observe("custom.echo.ping", "tier2", ctx("msg=hello"), budget(5)) -> plugin_res;
match plugin_res {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
"#;
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");
        sync_deps_lock_v1(root).expect("sync lock");

        if !with_plugin_index {
            return;
        }

        let plugin_dir = root.join("plugins");
        let plugin_bin_dir = plugin_dir.join("bin");
        fs::create_dir_all(&plugin_bin_dir).expect("plugin bin dir");

        let plugin_script = plugin_bin_dir.join("echo_plugin.ps1");
        let script_body = r#"$line = [Console]::In.ReadLine()
if ([string]::IsNullOrWhiteSpace($line)) {
  Write-Output '{"id":"1","kind":"deferred","reason":"RC-PLUGIN-PROTOCOL-ERROR","payload":""}'
  exit 0
}
Write-Output '{"id":"1","kind":"ok","reason":"RC-ADAPTER-FAILED","payload":"echo"}'
"#;
        fs::write(&plugin_script, script_body).expect("write plugin script");

        let script_text = plugin_script.to_string_lossy().replace('\\', "/");
        let index = format!(
            concat!(
                "[[plugin]]\n",
                "id = \"echo.local.v1\"\n",
                "key_prefix = \"custom.echo.\"\n",
                "platform = \"windows-x64\"\n",
                "command = \"powershell -ExecutionPolicy Bypass -File {}\"\n"
            ),
            script_text
        );
        fs::write(plugin_dir.join("index.toml"), index).expect("write plugin index");
    }

    fn prepare_v5_w6_organ_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "v5_w6_organs_cli"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = []
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        fs::write(
            root.join("src").join("main.ocl"),
            "let ok = true;\ncondition(ok);\n",
        )
        .expect("write source");
        sync_deps_lock_v1(root).expect("sync deps lock");
        sync_policy_lock_v1(root).expect("sync policy lock");

        let cosmos = r#"version = 1

[[universe]]
id = "ci_locked"
runtime_mode = "deterministic"
engine = "dual"
policy_profile_id = "default"
audit = "hash_only"
trace = "hash_only"

[[domain]]
id = "default"
universe_id = "ci_locked"

[[view]]
id = "text_default"
universe_id = "ci_locked"
domain_id = "default"
renderer = "text"

[[kits]]
kit_id = "std.kit.view_text_basic"
universe_id = "ci_locked"
bind_domain = "default"
bind_view = "text_default"
required_organs = ["noop.organ@0.1.0"]
"#;
        fs::write(root.join("cosmos.toml"), cosmos).expect("write cosmos");
        sync_cosmos_lock_v1(root, false, None, None, None).expect("sync cosmos lock");

        let platform = resolve_platform_tag_v1();
        let registry_root = root.join("registry").join("organs");
        let artifact_rel = format!("artifacts/noop.organ/0.1.0/{platform}/noop.organ.bin");
        let artifact_path = registry_root.join(&artifact_rel);
        fs::create_dir_all(
            artifact_path
                .parent()
                .expect("artifact path must have parent"),
        )
        .expect("mkdir artifact");
        fs::write(&artifact_path, b"noop-organ-cli-v5-w6").expect("write artifact");

        let index = format!(
            concat!(
                "[[organ]]\n",
                "pack_id = \"noop.organ\"\n",
                "version = \"0.1.0\"\n",
                "platform = \"{}\"\n",
                "artifact_rel = \"{}\"\n",
                "provides_caps = [\"std.io.noop\"]\n",
                "source = \"local\"\n",
                "abi_version = \"v1\"\n",
                "signer_id = \"dev-root-1\"\n"
            ),
            platform, artifact_rel
        );
        fs::write(registry_root.join("index.toml"), index).expect("write organ index");
    }

    fn prepare_w7_tls_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "w7_tls_client"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = ["std.net.poll"]
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        let source = r#"module tests.w7.tls;

observe("std.tls.connect", "tier2", ctx("host=example.com;port=443"), budget(5)) -> conn_res;
observe("std.tls.handshake", "tier2", ctx("conn=example.com:443"), budget(5)) -> hs_res;

match conn_res {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

match hs_res {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
"#;
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");
        sync_deps_lock_v1(root).expect("sync lock");
    }

    fn prepare_w7_sqlite_project(root: &Path) {
        init_project(root).expect("init");
        let manifest = r#"[package]
name = "w7_sqlite_app"
version = "0.1.0"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[permissions.package]
allow = ["*"]
deny = ["std.net.poll"]
"#;
        fs::write(root.join("Ocl.toml"), manifest).expect("write manifest");
        let source = r#"module tests.w7.sqlite;

observe("std.db.query_int", "tier2", ctx("dsn=file:demo.db;sql=SELECT count(*) FROM counters"), budget(5)) -> query_res;
observe("std.db.exec", "tier2", ctx("dsn=file:demo.db;sql=UPDATE counters SET n=1"), budget(5)) -> write_res;
commit(write_res);

match query_res {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
"#;
        fs::write(root.join("src").join("main.ocl"), source).expect("write source");

        let smoke = r#"observe("std.db.query_int", "tier2", ctx("dsn=file:demo.db;sql=SELECT 1"), budget(5)) -> query_res;
match query_res {
  OK => { let stable = true; }
  DEGRADED => { let stable = true; }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
"#;
        fs::write(root.join("tests").join("smoke.ocl"), smoke).expect("write smoke");
        sync_deps_lock_v1(root).expect("sync lock");
    }

    #[test]
    fn w1_cli_runtime_flags_reactor_pass() {
        let root = temp_project_dir("reactor_pass");
        prepare_runtime_project(&root);
        let report = root.join("runtime_report.json");
        let audit = root.join("replay.audit.jsonl");

        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--reactor".to_string(),
            "--ticks".to_string(),
            "4".to_string(),
            "--runtime".to_string(),
            "throughput".to_string(),
            "--socket-listen".to_string(),
            "127.0.0.1:19091".to_string(),
            "--runtime-report".to_string(),
            report.to_string_lossy().to_string(),
            "--replay-audit".to_string(),
            audit.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];

        let code = run_cli(&args);
        assert_eq!(code, 0);
        assert!(report.exists(), "runtime report was not written");
        assert!(audit.exists(), "replay audit was not written");
    }

    #[test]
    fn w1_cli_invalid_runtime_mode_rejected() {
        let root = temp_project_dir("bad_mode");
        prepare_runtime_project(&root);

        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--reactor".to_string(),
            "--runtime".to_string(),
            "turbo".to_string(),
            "--locked".to_string(),
        ];

        let code = run_cli(&args);
        assert_eq!(code, 2);
    }

    #[test]
    fn w1_cli_socket_listen_requires_reactor() {
        let root = temp_project_dir("requires_reactor");
        prepare_runtime_project(&root);

        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--socket-listen".to_string(),
            "127.0.0.1:19091".to_string(),
            "--locked".to_string(),
        ];

        let code = run_cli(&args);
        assert_eq!(code, 2);
    }

    #[test]
    fn v5_w1_cli_policy_cosmos_and_universe_wiring_pass() {
        let root = temp_project_dir("v5_w1_ok");
        prepare_v5_w1_universe_project(&root);
        let repo_root = repo_root_dir();
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");

        let policy_args = vec![
            "policy".to_string(),
            "lock".to_string(),
            "sync".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&policy_args), 0);

        let cosmos_init_args = vec![
            "cosmos".to_string(),
            "init".to_string(),
            root.to_string_lossy().to_string(),
            "--preset".to_string(),
            "ci".to_string(),
        ];
        assert_eq!(run_cli(&cosmos_init_args), 0);

        let cosmos_lock_args = vec![
            "cosmos".to_string(),
            "lock".to_string(),
            "sync".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--signer-id".to_string(),
            "dev-root-1".to_string(),
            "--sign-key".to_string(),
            sign_key.to_string_lossy().to_string(),
            "--trust-store".to_string(),
            trust_store.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&cosmos_lock_args), 0);

        let check_args = vec![
            "check".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
        ];
        assert_eq!(run_cli(&check_args), 0);

        let run_args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 0);
    }

    #[test]
    fn v5_w1_cli_locked_cosmos_requires_universe() {
        let root = temp_project_dir("v5_w1_require_universe");
        prepare_v5_w1_universe_project(&root);
        let repo_root = repo_root_dir();
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");

        assert_eq!(
            run_cli(&[
                "policy".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "init".to_string(),
                root.to_string_lossy().to_string(),
                "--preset".to_string(),
                "ci".to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string(),
                "--locked".to_string(),
                "--signer-id".to_string(),
                "dev-root-1".to_string(),
                "--sign-key".to_string(),
                sign_key.to_string_lossy().to_string(),
                "--trust-store".to_string(),
                trust_store.to_string_lossy().to_string()
            ]),
            0
        );

        let code = run_cli(&[
            "check".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ]);
        assert_eq!(code, 1);
    }

    #[test]
    fn v5_w1_cli_locked_universe_engine_mismatch_fail_hard() {
        let root = temp_project_dir("v5_w1_mismatch");
        prepare_v5_w1_universe_project(&root);
        let repo_root = repo_root_dir();
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");

        assert_eq!(
            run_cli(&[
                "policy".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "init".to_string(),
                root.to_string_lossy().to_string(),
                "--preset".to_string(),
                "ci".to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string(),
                "--locked".to_string(),
                "--signer-id".to_string(),
                "dev-root-1".to_string(),
                "--sign-key".to_string(),
                sign_key.to_string_lossy().to_string(),
                "--trust-store".to_string(),
                trust_store.to_string_lossy().to_string()
            ]),
            0
        );

        let code = run_cli(&[
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
            "--engine".to_string(),
            "bytecode".to_string(),
        ]);
        assert_eq!(code, 1);
    }

    #[test]
    fn v5_w2_cli_domain_required_without_default_fail_hard() {
        let root = temp_project_dir("v5_w2_domain_required");
        prepare_v5_w2_domain_project(&root, false);
        let repo_root = repo_root_dir();
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");

        assert_eq!(
            run_cli(&[
                "policy".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string(),
                "--locked".to_string(),
                "--signer-id".to_string(),
                "dev-root-1".to_string(),
                "--sign-key".to_string(),
                sign_key.to_string_lossy().to_string(),
                "--trust-store".to_string(),
                trust_store.to_string_lossy().to_string()
            ]),
            0
        );

        let code = run_cli(&[
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
        ]);
        assert_eq!(code, 1);
    }

    #[test]
    fn v5_w2_cli_trace_run_sets_domain_context_pass() {
        let root = temp_project_dir("v5_w2_trace_domain");
        prepare_v5_w2_domain_project(&root, true);
        let repo_root = repo_root_dir();
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");
        let out = root.join("v5_w2.trace.jsonl");

        assert_eq!(
            run_cli(&[
                "policy".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "cosmos".to_string(),
                "lock".to_string(),
                "sync".to_string(),
                root.to_string_lossy().to_string(),
                "--locked".to_string(),
                "--signer-id".to_string(),
                "dev-root-1".to_string(),
                "--sign-key".to_string(),
                sign_key.to_string_lossy().to_string(),
                "--trust-store".to_string(),
                trust_store.to_string_lossy().to_string()
            ]),
            0
        );
        assert_eq!(
            run_cli(&[
                "trace".to_string(),
                "run".to_string(),
                root.to_string_lossy().to_string(),
                "--locked".to_string(),
                "--universe".to_string(),
                "ci_locked".to_string(),
                "--domain".to_string(),
                "default".to_string(),
                "--out".to_string(),
                out.to_string_lossy().to_string()
            ]),
            0
        );

        let events = read_trace_jsonl(&out).expect("read trace output");
        assert!(!events.is_empty(), "trace output must not be empty");
        for event in events {
            assert_eq!(event.universe_id, "ci_locked");
            assert_eq!(event.domain_id, "default");
        }
    }

    #[test]
    fn w3_cli_engine_bytecode_and_dual_pass() {
        let root = temp_project_dir("w3_engine");
        prepare_runtime_project(&root);

        let args_bytecode = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--engine".to_string(),
            "bytecode".to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&args_bytecode), 0);

        let args_dual = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&args_dual), 0);
    }

    #[test]
    fn v5_w3_cli_run_shadow_deterministic_pass() {
        let root = temp_project_dir("v5_w3_run_shadow");
        prepare_v5_w3_shadow_project(&root, false);

        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--shadow".to_string(),
            "parity".to_string(),
            "--shadow-policy".to_string(),
            "forbid_commit".to_string(),
        ];
        assert_eq!(run_cli(&args), 0);
    }

    #[test]
    fn v5_w3_cli_reactor_throughput_shadow_unsupported_exit_13() {
        let root = temp_project_dir("v5_w3_reactor_unsupported");
        prepare_runtime_project(&root);

        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--reactor".to_string(),
            "--ticks".to_string(),
            "3".to_string(),
            "--runtime".to_string(),
            "throughput".to_string(),
            "--locked".to_string(),
            "--shadow".to_string(),
            "parity".to_string(),
        ];
        assert_eq!(run_cli(&args), 13);
    }

    #[test]
    fn v5_w3_cli_trace_shadow_nondet_unsupported_exit_13() {
        let root = temp_project_dir("v5_w3_trace_nondet");
        prepare_v5_w3_shadow_project(&root, true);
        let out = root.join("w3.trace.jsonl");

        let args = vec![
            "trace".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--shadow".to_string(),
            "parity".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&args), 13);
    }

    #[test]
    fn w4_cli_build_publish_fetch_verify_and_run_artifact_pass() {
        let root = temp_project_dir("w4_flow");
        prepare_w4_project(&root);

        let build_args = vec![
            "build".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&build_args), 0);

        let artifact = root.join(".oclpkg").join("w4_cli_pkg-0.1.0.oclpkg");
        assert!(artifact.exists(), "artifact .oclpkg not found");

        let verify_args = vec![
            "verify-supply".to_string(),
            artifact.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&verify_args), 0);

        let registry = root.join("registry-local");
        let publish_args = vec![
            "publish".to_string(),
            artifact.to_string_lossy().to_string(),
            "--registry".to_string(),
            registry.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&publish_args), 0);

        let out = root.join("fetched");
        let fetch_args = vec![
            "fetch".to_string(),
            "w4_cli_pkg".to_string(),
            "--registry".to_string(),
            registry.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&fetch_args), 0);

        let fetched = out.join("w4_cli_pkg-0.1.0.oclpkg");
        assert!(fetched.exists(), "fetched artifact not found");

        let run_args = vec![
            "run".to_string(),
            fetched.to_string_lossy().to_string(),
            "--engine".to_string(),
            "dual".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 0);
    }

    #[test]
    fn w5_cli_trace_and_profile_non_reactor_pass() {
        let root = temp_project_dir("w5_non_reactor");
        prepare_runtime_project(&root);
        let trace_out = root.join("w5.trace.jsonl");
        let profile_out = root.join("w5.profile.json");

        let trace_run_args = vec![
            "trace".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--out".to_string(),
            trace_out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&trace_run_args), 0);
        assert!(trace_out.exists(), "trace output not found");

        let trace_view_args = vec![
            "trace".to_string(),
            "view".to_string(),
            trace_out.to_string_lossy().to_string(),
            "--tail".to_string(),
            "8".to_string(),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&trace_view_args), 0);

        let profile_run_args = vec![
            "profile".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--out".to_string(),
            profile_out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&profile_run_args), 0);
        assert!(profile_out.exists(), "profile output not found");

        let profile_view_args = vec![
            "profile".to_string(),
            "view".to_string(),
            profile_out.to_string_lossy().to_string(),
            "--top".to_string(),
            "5".to_string(),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&profile_view_args), 0);
    }

    #[test]
    fn w5_cli_trace_reactor_deterministic_seq_monotonic() {
        let root = temp_project_dir("w5_reactor");
        prepare_runtime_project(&root);
        let trace_out = root.join("w5.reactor.trace.jsonl");

        let trace_run_args = vec![
            "trace".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--reactor".to_string(),
            "--ticks".to_string(),
            "3".to_string(),
            "--runtime".to_string(),
            "deterministic".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--out".to_string(),
            trace_out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&trace_run_args), 0);

        let events = read_trace_jsonl(&trace_out).expect("read trace");
        assert!(!events.is_empty(), "trace events must not be empty");
        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.seq as usize, idx + 1, "trace seq must be monotonic");
            assert_eq!(
                event.universe_id, "__legacy__",
                "trace universe sentinel must be non-empty and stable"
            );
            assert_eq!(
                event.domain_id, "default",
                "trace domain sentinel must be non-empty and stable"
            );
        }
    }

    #[test]
    fn w5_trace_digest_changes_when_order_changes() {
        let root = temp_project_dir("w5_digest");
        prepare_runtime_project(&root);
        let trace_out = root.join("w5.digest.trace.jsonl");

        let trace_run_args = vec![
            "trace".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--out".to_string(),
            trace_out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&trace_run_args), 0);

        let events = read_trace_jsonl(&trace_out).expect("read trace");
        assert!(events.len() >= 2, "need at least 2 events");
        let digest_original = trace_required_digest(&events);

        let mut reversed = events.clone();
        reversed.reverse();
        let digest_reversed = trace_required_digest(&reversed);
        assert_ne!(
            digest_original, digest_reversed,
            "required digest must preserve event order"
        );
    }

    #[test]
    fn v5_w5_cli_view_requires_cosmos_fail_hard() {
        let root = temp_project_dir("v5_w5_no_cosmos");
        prepare_runtime_project(&root);
        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--view".to_string(),
            "text_default".to_string(),
        ];
        assert_eq!(run_cli(&args), 1);
    }

    #[test]
    fn v5_w5_cli_view_wiring_run_pass() {
        let root = temp_project_dir("v5_w5_view_pass");
        prepare_v5_w5_view_project(&root);
        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
            "--domain".to_string(),
            "default".to_string(),
            "--view".to_string(),
            "text_default".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
        ];
        assert_eq!(run_cli(&args), 0);
    }

    #[test]
    fn v5_w5_cli_view_domain_ambiguous_fail_hard() {
        let root = temp_project_dir("v5_w5_view_ambig");
        prepare_v5_w5_view_project(&root);
        let args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
            "--view".to_string(),
            "ops".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
        ];
        assert_eq!(run_cli(&args), 1);
    }

    #[test]
    fn w6_cli_plugin_lock_verify_and_locked_run_pass() {
        let root = temp_project_dir("w6_plugin_pass");
        prepare_w6_project(&root, true);

        let lock_sync_args = vec![
            "plugin".to_string(),
            "lock".to_string(),
            "sync".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&lock_sync_args), 0);

        let verify_args = vec![
            "plugin".to_string(),
            "verify".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&verify_args), 0);

        let run_args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 0);

        let trace_out = root.join("w6.trace.jsonl");
        let trace_run_args = vec![
            "trace".to_string(),
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--out".to_string(),
            trace_out.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&trace_run_args), 0);

        let events = read_trace_jsonl(&trace_out).expect("read trace");
        let plugin_ok = events.iter().any(|e| {
            e.event == "observe_end"
                && e.key.as_deref() == Some("custom.echo.ping")
                && e.kind.as_deref() == Some("ok")
        });
        assert!(
            plugin_ok,
            "custom plugin key should yield observe_end kind=ok"
        );
    }

    #[test]
    fn w6_locked_custom_key_without_plugin_lock_fails() {
        let root = temp_project_dir("w6_plugin_fail_missing_lock");
        prepare_w6_project(&root, false);

        let run_args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 1);
    }

    #[test]
    fn v5_w6_cli_organ_lock_verify_install_and_kit_doctor_pass() {
        let root = temp_project_dir("v5_w6_organs_cli_pass");
        prepare_v5_w6_organ_project(&root);

        let lock_sync_args = vec![
            "organ".to_string(),
            "lock".to_string(),
            "sync".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&lock_sync_args), 0);

        let verify_args = vec![
            "organ".to_string(),
            "verify".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&verify_args), 0);

        let install_args = vec![
            "organ".to_string(),
            "install".to_string(),
            "noop.organ".to_string(),
            "0.1.0".to_string(),
            "--project".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&install_args), 0);

        let list_args = vec![
            "kit".to_string(),
            "list".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&list_args), 0);

        let doctor_args = vec![
            "kit".to_string(),
            "doctor".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&doctor_args), 0);
    }

    #[test]
    fn v5_w6_cli_locked_missing_organs_lock_fail_hard() {
        let root = temp_project_dir("v5_w6_organs_cli_missing_lock");
        prepare_v5_w6_organ_project(&root);

        let check_args = vec![
            "check".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--universe".to_string(),
            "ci_locked".to_string(),
        ];
        assert_eq!(run_cli(&check_args), 1);

        let doctor_args = vec![
            "kit".to_string(),
            "doctor".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&doctor_args), 1);
    }

    #[test]
    fn v5_w6_cli_init_preset_workflow_basic_unlocked_pass() {
        let root = temp_project_dir("v5_w6_init_workflow");
        let init_args = vec![
            "init".to_string(),
            root.to_string_lossy().to_string(),
            "--preset".to_string(),
            "workflow_basic".to_string(),
        ];
        assert_eq!(run_cli(&init_args), 0);
        assert!(root.join("Ocl.toml").exists(), "missing Ocl.toml");
        assert!(root.join("cosmos.toml").exists(), "missing cosmos.toml");
        assert!(root.join("deps.lock").exists(), "missing deps.lock");
        assert!(root.join("policy.lock.v1").exists(), "missing policy lock");
        assert!(root.join("cosmos.lock.v1").exists(), "missing cosmos lock");

        let doctor_args = vec![
            "kit".to_string(),
            "doctor".to_string(),
            root.to_string_lossy().to_string(),
        ];
        assert_eq!(run_cli(&doctor_args), 0);
    }

    #[test]
    fn v5_w6_cli_init_preset_agent_swarm_basic_unlocked_pass() {
        let root = temp_project_dir("v5_w6_init_swarm");
        let init_args = vec![
            "init".to_string(),
            root.to_string_lossy().to_string(),
            "--preset".to_string(),
            "agent_swarm_basic".to_string(),
        ];
        assert_eq!(run_cli(&init_args), 0);
        let cosmos = fs::read_to_string(root.join("cosmos.toml")).expect("read cosmos");
        assert!(
            cosmos.contains("[hive]"),
            "agent_swarm_basic preset must materialize hive caps"
        );
        assert!(
            cosmos.contains("[[bridge]]"),
            "agent_swarm_basic preset must materialize bridge topology"
        );
    }

    #[test]
    fn w7_cli_tls_client_locked_flow_pass() {
        let root = temp_project_dir("w7_tls_pass");
        prepare_w7_tls_project(&root);

        let check_args = vec![
            "check".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&check_args), 0);

        let run_args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 0);
    }

    #[test]
    fn w7_cli_sqlite_app_locked_flow_pass() {
        let root = temp_project_dir("w7_sqlite_pass");
        prepare_w7_sqlite_project(&root);

        let check_args = vec![
            "check".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&check_args), 0);

        let run_args = vec![
            "run".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&run_args), 0);

        let test_args = vec![
            "test".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&test_args), 0);

        let build_args = vec![
            "build".to_string(),
            root.to_string_lossy().to_string(),
            "--locked".to_string(),
        ];
        assert_eq!(run_cli(&build_args), 0);
    }

    #[test]
    fn w9_cli_conformance_requires_sign_key_when_locked() {
        std::env::remove_var("OCL_SIGN_KEY_PATH");
        let args = vec![
            "test".to_string(),
            "--conformance".to_string(),
            "--manifest".to_string(),
            "projects/ocp-ocl/conformance/conformance.v1.toml".to_string(),
            "--locked".to_string(),
            "--signer-id".to_string(),
            "dev-root-1".to_string(),
        ];
        assert_eq!(run_cli(&args), 12);
    }

    #[test]
    fn w9_cli_conformance_single_scenario_pass() {
        let root = temp_project_dir("w9_single");
        prepare_runtime_project(&root);
        let manifest_path = root.join("conformance.v1.toml");
        let out_path = root.join("w9_report.json");
        let root_value = manifest_path_value(&root);
        let manifest = format!(
            concat!(
                "version = 1\n\n",
                "[[scenario]]\n",
                "name = \"single\"\n",
                "path = \"{}\"\n",
                "reactor_ticks = 0\n",
                "composer = false\n"
            ),
            root_value
        );
        fs::write(&manifest_path, manifest).expect("write conformance manifest");

        let repo_root = repo_root_dir();
        let trust_store = repo_root.join("projects/ocp-ocl/security/trust.store.toml");
        let sign_key = repo_root.join("projects/ocp-ocl/security/dev-root-1.signing.key.toml");

        let args = vec![
            "test".to_string(),
            "--conformance".to_string(),
            "--manifest".to_string(),
            manifest_path_value(&manifest_path),
            "--out".to_string(),
            manifest_path_value(&out_path),
            "--runtime".to_string(),
            "deterministic".to_string(),
            "--engine".to_string(),
            "dual".to_string(),
            "--locked".to_string(),
            "--trust-store".to_string(),
            manifest_path_value(&trust_store),
            "--signer-id".to_string(),
            "dev-root-1".to_string(),
            "--sign-key".to_string(),
            manifest_path_value(&sign_key),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&args), 0);
        assert!(out_path.exists(), "missing W9 report output");
    }
}
