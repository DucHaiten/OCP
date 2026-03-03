use std::fs;
use std::path::{Path, PathBuf};

use ocl_runtime_core::RunEngine;
use ocl_runtime_core::RuntimeCoreError;
use ocl_sdk::{
    build_oclpkg_with_lock, build_profile_from_trace, build_project_with_lock,
    check_project_with_lock, compare_shadow_traces_v1, compose_phenotype,
    default_conformance_manifest_path, enforce_universe_match_v1, fetch_artifact, fmt_project,
    init_cosmos_v1, init_project, install_organs_v1, list_kits_from_cosmos_v1,
    parse_conformance_manifest_v1, parse_shadow_policy_v1, publish_artifact, read_profile_json,
    read_trace_jsonl, render_conformance_report_json, render_profile_view, render_trace_view,
    resolve_domain_selection_v1, resolve_universe_v1, resolve_view_selection_v1, run_artifact,
    run_conformance_v1, run_kit_doctor_v1, run_project_with_engine_and_lock,
    run_project_with_shadow_compare, run_project_with_trace_engine_and_lock,
    run_reactor_service_with_lock, run_reactor_service_with_shadow_compare,
    run_reactor_service_with_trace_engine_and_lock, sync_cosmos_lock_v1, sync_deps_lock_v1,
    sync_organs_lock_v1, sync_plugin_lock_v1, sync_policy_lock_v1, test_project_with_lock,
    verify_assembly, verify_organs_lock_v1, verify_plugin_lock_v1, verify_supply_artifact,
    write_conformance_report_json, write_profile_json, write_shadow_compare_artifacts_v1,
    write_trace_jsonl, ConformanceRunOptionsV1, InputEnvelopeV1, ProfileViewOptions,
    ReactorRuntimeMode, ReactorServiceOptions, SdkError, ShadowOptionsV1, TraceViewOptions,
};

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
                    "usage: ocl init <project_dir> [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]"
                );
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            let preset = parse_string_flag(args, "--preset");
            let locked = args.iter().any(|a| a == "--locked");
            let signer_id = parse_string_flag(args, "--signer-id");
            let sign_key = parse_string_flag(args, "--sign-key").map(PathBuf::from);
            let trust_store = parse_string_flag(args, "--trust-store").map(PathBuf::from);
            let registry = parse_string_flag(args, "--registry").map(PathBuf::from);
            let root = Path::new(path);

            match init_project(root) {
                Ok(layout) => {
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
                            "{{\"ok\":true,\"root\":\"{}\",\"preset\":{},\"locked\":{}}}",
                            json_escape(&layout.root.to_string_lossy()),
                            preset
                                .as_ref()
                                .map(|v| format!("\"{}\"", json_escape(v)))
                                .unwrap_or_else(|| "null".to_string()),
                            if locked { "true" } else { "false" }
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
            let locked = args.iter().any(|a| a == "--locked");
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
                    match run_project_with_shadow_compare(path_ref, run_engine, locked, shadow_cfg)
                    {
                        Ok(summary) => {
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
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            exit_code_for_sdk_error(&err)
                        }
                    }
                } else {
                    match run_project_with_engine_and_lock(path_ref, run_engine, locked) {
                        Ok(summary) => {
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
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
            })
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
                        "usage: ocl test <project_dir> [--locked] [--universe <id>] [--domain <id>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log]"
                    );
                    return 2;
                };
                let locked = args.iter().any(|a| a == "--locked");
                let shadow_options = match parse_shadow_args(args) {
                    Ok(value) => value,
                    Err(code) => return code,
                };
                let universe_id = parse_string_flag(args, "--universe");
                let domain_id = parse_string_flag(args, "--domain");
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
                        println!("test ok (tests_run={})", summary.tests_run);
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
    eprintln!("  init  <project_dir> [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]");
    eprintln!("  check <project_dir> [--json] [--locked] [--universe <id>]");
    eprintln!(
        "  run   <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!(
        "  trace run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  trace view <trace_file> [--tail N] [--json]");
    eprintln!(
        "  profile run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  profile view <profile_file> [--top N] [--json]");
    eprintln!("  fmt   <project_dir> [--check]");
    eprintln!("  test  <project_dir> [--locked] [--universe <id>] [--domain <id>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log]");
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
