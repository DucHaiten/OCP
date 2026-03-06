use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use ocl_runtime_core::{
    pretty_schema, schema_skeleton, CapabilityRegistry, ExecConfig, KeyCapabilityKind, RunEngine,
    RuntimeCoreError,
};
use ocl_sdk::{
    apply_permission_fix_plan_v17, approve_permission_diff_v15, build_attestation_v15,
    build_oclpkg_with_lock, build_profile_from_trace, build_project_with_lock,
    build_run_id_deterministic, check_project_with_lock, compare_shadow_traces_v1,
    compose_phenotype, default_conformance_manifest_path, enforce_universe_match_v1,
    fetch_artifact, fmt_project, init_cosmos_v1, init_project, inspect_contract_json_v17,
    install_organs_v1, list_kits_from_cosmos_v1, parse_conformance_manifest_v1,
    parse_shadow_policy_v1, publish_artifact, read_profile_json, read_trace_jsonl,
    render_conformance_report_json, render_profile_view, resolve_deps_v3,
    resolve_domain_selection_v1, resolve_universe_v1, resolve_view_selection_v1, run_artifact,
    run_conformance_v1, run_kit_doctor_v1, run_project_with_engine_and_lock,
    run_project_with_shadow_compare, run_project_with_trace_engine_and_lock,
    run_project_with_trace_engine_config_and_lock, run_reactor_service_with_lock,
    run_reactor_service_with_shadow_compare, run_reactor_service_with_trace_engine_and_lock,
    sign_contract_json_v17, sign_deps_lock_v3_v15, sign_oclpkg, sync_cosmos_lock_v1,
    sync_deps_lock_v1, sync_organs_lock_v1, sync_plugin_lock_v1, sync_policy_lock_v1,
    test_project_with_lock, trace_required_digest, verify_assembly, verify_build_attestation_v15,
    verify_build_repro_v15, verify_contract_json_signature_v17, verify_contract_signature_file_v17,
    verify_deps_lock_v3, verify_deps_lock_v3_signature_v15, verify_deps_signing_and_trust_v10,
    verify_organs_lock_v1, verify_plugin_lock_v1, verify_supply_artifact,
    write_conformance_report_json, write_permission_diff_report_v15,
    write_permission_doctor_report_v17, write_permission_fix_plan_v17,
    write_permission_snapshot_v15, write_profile_json, write_shadow_compare_artifacts_v1,
    write_trace_jsonl, ConformanceManifestV1, ConformanceRunOptionsV1, InputEnvelopeV1,
    PermissionFixApplyOptionsV17, ProfileViewOptions, ReactorRuntimeMode, ReactorServiceOptions,
    SdkError, ShadowOptionsV1, TraceEventV1, TraceRunSummary,
};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
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
                    "usage: ocl init <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview|dep-permission] [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]"
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
        "cache" => {
            let Some(subcmd) = args.get(1).map(String::as_str) else {
                eprintln!(
                    "usage: ocl cache <stats|clean|bench> <project_dir> [--json]\n  stats <project_dir> [--json]\n  clean <project_dir> --yes [--json]\n  bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]"
                );
                return 2;
            };
            match subcmd {
                "stats" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl cache stats <project_dir> [--json]");
                        return 2;
                    };
                    let json_mode = args.iter().any(|a| a == "--json");
                    match collect_cache_stats_report_v13(Path::new(path)) {
                        Ok(report) => {
                            if json_mode {
                                println!("{}", render_cache_stats_json_v13(&report));
                            } else {
                                println!("{}", render_cache_stats_text_v13(&report));
                            }
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "clean" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl cache clean <project_dir> --yes [--json]");
                        return 2;
                    };
                    let yes = args.iter().any(|a| a == "--yes");
                    if !yes {
                        eprintln!(
                            "cache clean requires explicit --yes to avoid accidental deletion"
                        );
                        return 2;
                    }
                    let json_mode = args.iter().any(|a| a == "--json");
                    match clean_cache_state_v13(Path::new(path)) {
                        Ok(summary) => {
                            if json_mode {
                                println!("{}", render_cache_clean_json_v13(&summary));
                            } else {
                                println!("{}", render_cache_clean_text_v13(&summary));
                            }
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "bench" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl cache bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let json_mode = args.iter().any(|a| a == "--json");
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
                    match run_cache_benchmark_v13(Path::new(path), run_engine, locked, json_mode) {
                        Ok(rendered) => {
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
                    eprintln!(
                        "usage: ocl cache <stats|clean|bench> <project_dir> [--json]\n  stats <project_dir> [--json]\n  clean <project_dir> --yes [--json]\n  bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]"
                    );
                    2
                }
            }
        }
        "dbg" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl dbg <artifact_dir> [--script <file>]");
                return 2;
            };
            let script_path = parse_string_flag(args, "--script").map(PathBuf::from);
            match run_dbg_v11(Path::new(path), script_path.as_deref()) {
                Ok(rendered) => {
                    if !rendered.is_empty() {
                        println!("{rendered}");
                    }
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "minimize" => {
            let Some(path) = args.get(1) else {
                eprintln!(
                    "usage: ocl minimize <artifact_dir> --goal <error_code:X|divergence|kind:KIND> [--key <key>] [--against <artifactB>] [--out <dir>] [--json]"
                );
                return 2;
            };
            let Some(goal_raw) = parse_string_flag(args, "--goal") else {
                eprintln!(
                    "usage: ocl minimize <artifact_dir> --goal <error_code:X|divergence|kind:KIND> [--key <key>] [--against <artifactB>] [--out <dir>] [--json]"
                );
                return 2;
            };
            let options = MinimizerCliOptionsV11 {
                goal_raw,
                key: parse_string_flag(args, "--key"),
                against: parse_string_flag(args, "--against").map(PathBuf::from),
                out_dir: parse_string_flag(args, "--out").map(PathBuf::from),
                json: args.iter().any(|a| a == "--json"),
            };
            match run_minimize_v11(Path::new(path), &options) {
                Ok(rendered) => {
                    println!("{rendered}");
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
                    "usage: ocl trace <run|view|diff> ...\n  run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]\n  view <artifact_dir|audit.jsonl|legacy.trace> [--tail N] [--json] [--legacy-pipe] [--type <event>] [--key <key>] [--kind <kind>] [--reason <rc>] [--module <module_id>]\n  diff <artifactA|auditA> <artifactB|auditB> [--mode strict|align] [--json] [--out <dir>]"
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
                        eprintln!(
                            "usage: ocl trace view <artifact_dir|audit.jsonl|legacy.trace> [--tail N] [--json] [--legacy-pipe] [--type <event>] [--key <key>] [--kind <kind>] [--reason <rc>] [--module <module_id>]"
                        );
                        return 2;
                    };
                    let options = TraceViewCliOptionsV11 {
                        tail: parse_u32_flag(args, "--tail").map(|v| v as usize),
                        json: args.iter().any(|a| a == "--json"),
                        filter_type: parse_string_flag(args, "--type"),
                        filter_key: parse_string_flag(args, "--key"),
                        filter_kind: parse_string_flag(args, "--kind"),
                        filter_reason: parse_string_flag(args, "--reason"),
                        filter_module: parse_string_flag(args, "--module"),
                        legacy_pipe: args.iter().any(|a| a == "--legacy-pipe"),
                    };
                    let input = Path::new(trace_path);
                    match read_trace_events_for_view_v11(input, options.legacy_pipe) {
                        Ok(events) => {
                            let rendered = render_trace_view_v11(input, &events, &options);
                            println!("{}", rendered);
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "diff" => {
                    let Some(left_path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl trace diff <artifactA|auditA> <artifactB|auditB> [--mode strict|align] [--json] [--out <dir>]"
                        );
                        return 2;
                    };
                    let Some(right_path) = args.get(3) else {
                        eprintln!(
                            "usage: ocl trace diff <artifactA|auditA> <artifactB|auditB> [--mode strict|align] [--json] [--out <dir>]"
                        );
                        return 2;
                    };
                    let mode_raw =
                        parse_string_flag(args, "--mode").unwrap_or_else(|| "strict".to_string());
                    let mode = match mode_raw.as_str() {
                        "strict" => TraceDiffModeV11::Strict,
                        "align" => TraceDiffModeV11::Align,
                        _ => {
                            eprintln!("invalid diff mode: `{mode_raw}` (expected strict|align)");
                            return 2;
                        }
                    };
                    let options = TraceDiffOptionsV11 {
                        mode,
                        json: args.iter().any(|a| a == "--json"),
                        out_dir: parse_string_flag(args, "--out").map(PathBuf::from),
                    };
                    match run_trace_diff_v11(Path::new(left_path), Path::new(right_path), &options)
                    {
                        Ok(rendered) => {
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
                    eprintln!("usage: ocl trace <run|view|diff> ...");
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
        "conformance" => {
            if args.len() < 2 {
                eprintln!(
                    "usage: ocl conformance <run|list> [--manifest <file>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--trust-store <file>] [--signer-id <id>] [--sign-key <file>] [--json]"
                );
                return 2;
            }
            let mut forwarded = vec!["test".to_string(), "--conformance".to_string()];
            forwarded.extend_from_slice(&args[1..]);
            run_cli(&forwarded)
        }
        "upgrade-check" => {
            let Some(project_dir) = args.get(1) else {
                eprintln!(
                    "usage: ocl upgrade-check <project_dir> [--manifest <file>] [--target-runtime <id>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--json]"
                );
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            let locked = args.iter().any(|a| a == "--locked");
            let universe_id = parse_string_flag(args, "--universe");
            let manifest_path = parse_string_flag(args, "--manifest")
                .map(PathBuf::from)
                .unwrap_or_else(|| default_conformance_manifest_path(Path::new(".")));
            let out_path = parse_string_flag(args, "--out")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("target/ocl/w9/reports/upgrade_check_report.json")
                });
            let target_runtime = parse_string_flag(args, "--target-runtime")
                .unwrap_or_else(|| "current".to_string());
            let runtime_mode_raw =
                parse_string_flag(args, "--runtime").unwrap_or_else(|| "deterministic".to_string());
            let runtime_mode = match runtime_mode_raw.as_str() {
                "deterministic" => ReactorRuntimeMode::Deterministic,
                "throughput" => ReactorRuntimeMode::Throughput,
                _ => {
                    eprintln!(
                        "invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)"
                    );
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
            let project_root = Path::new(project_dir);
            let features = detect_upgrade_check_features_v14(project_root);
            let selected_manifest = select_upgrade_manifest_subset_v14(&manifest, &features);
            let selected_names: Vec<String> = selected_manifest
                .scenarios
                .iter()
                .map(|s| s.name.clone())
                .collect();
            if selected_manifest.scenarios.is_empty() {
                let err = SdkError::MissingProject(
                    "X-UPGRADE-CHECK-FAILED: no conformance scenarios selected from manifest"
                        .to_string(),
                );
                if json_mode {
                    println!("{}", error_to_json(&err));
                } else {
                    eprintln!("{err}");
                }
                return 10;
            }

            // Use canonical conformance runner pipeline (same runner as `ocl test --conformance`).
            let conformance = run_conformance_v1(
                Path::new("."),
                &selected_manifest,
                ConformanceRunOptionsV1 {
                    locked,
                    engine: run_engine,
                    runtime_mode,
                    universe_id,
                },
            );
            let first_divergence = conformance
                .results
                .iter()
                .enumerate()
                .find(|(_, item)| !item.ok)
                .map(|(idx, item)| {
                    json!({
                        "index": idx,
                        "scenario": item.name,
                        "reason": item.reason.clone().unwrap_or_default(),
                    })
                });
            let ok = conformance.scenarios_failed == 0;
            let report_json = json!({
                "schema": "ocl.upgrade_check.v1",
                "project_dir": project_dir,
                "target_runtime": target_runtime,
                "runner_contract": "ocl test --conformance",
                "manifest_path": manifest_path.to_string_lossy(),
                "selected_scenarios": selected_names,
                "features": {
                    "uses_tool_packs": features.uses_tool_packs,
                    "uses_consumer_packs": features.uses_consumer_packs,
                    "uses_shadow": features.uses_shadow,
                    "uses_quarantine": features.uses_quarantine,
                    "has_dependencies": features.has_dependencies,
                },
                "conformance": {
                    "run_id": conformance.run_id,
                    "scenarios_total": conformance.scenarios_total,
                    "scenarios_passed": conformance.scenarios_passed,
                    "scenarios_failed": conformance.scenarios_failed,
                    "required_digest": conformance.required_digest,
                },
                "first_divergence": first_divergence,
                "ok": ok,
                "error_code": if ok { JsonValue::Null } else { JsonValue::String("X-UPGRADE-CHECK-FAILED".to_string()) }
            });

            if let Some(parent) = out_path.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    if json_mode {
                        println!("{}", error_to_json(&SdkError::Io(err)));
                    } else {
                        eprintln!("{}", SdkError::Io(err));
                    }
                    return 11;
                }
            }
            if let Err(err) = fs::write(
                &out_path,
                serde_json::to_string_pretty(&report_json).unwrap_or_else(|_| "{}".to_string()),
            ) {
                if json_mode {
                    println!("{}", error_to_json(&SdkError::Io(err)));
                } else {
                    eprintln!("{}", SdkError::Io(err));
                }
                return 11;
            }

            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report_json).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!(
                    "upgrade-check done (ok={}, selected={}, fail={}, digest={}, out={})",
                    ok,
                    report_json["selected_scenarios"]
                        .as_array()
                        .map(|v| v.len())
                        .unwrap_or(0),
                    conformance.scenarios_failed,
                    conformance.required_digest,
                    out_path.display()
                );
                if !ok {
                    if let Some(first) = report_json["first_divergence"].as_object() {
                        let scenario = first
                            .get("scenario")
                            .and_then(JsonValue::as_str)
                            .unwrap_or("unknown");
                        let reason = first
                            .get("reason")
                            .and_then(JsonValue::as_str)
                            .unwrap_or("");
                        eprintln!(
                            "X-UPGRADE-CHECK-FAILED: first_divergence={scenario} reason={reason}"
                        );
                    } else {
                        eprintln!("X-UPGRADE-CHECK-FAILED: conformance subset failed");
                    }
                }
            }

            if ok {
                0
            } else {
                10
            }
        }
        "lts" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!(
                    "usage: ocl lts <check|report> ...\n  check: ocl lts check <project_dir> [--manifest <file>] [--target-runtime <id>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--universe <id>] [--json]\n  report: ocl lts report <report_file> [--json]"
                );
                return 2;
            };
            match scope {
                "check" => {
                    let Some(project_dir) = args.get(2) else {
                        eprintln!(
                            "usage: ocl lts check <project_dir> [--manifest <file>] [--target-runtime <id>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--universe <id>] [--json]"
                        );
                        return 2;
                    };
                    let json_mode = args.iter().any(|a| a == "--json");
                    let profile = parse_string_flag(args, "--profile")
                        .unwrap_or_else(|| "strict".to_string());
                    if profile != "strict" {
                        eprintln!(
                            "V-LTS-PROFILE-INVALID: unsupported profile `{profile}` (expected `strict`)"
                        );
                        return 2;
                    }
                    let manifest_path = parse_string_flag(args, "--manifest")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| default_conformance_manifest_path(Path::new(".")));
                    let out_path = parse_string_flag(args, "--out")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| {
                            PathBuf::from("target/ocl/w15/reports/lts_check_report.json")
                        });
                    let target_runtime = parse_string_flag(args, "--target-runtime")
                        .unwrap_or_else(|| "current".to_string());
                    let runtime_mode_raw = parse_string_flag(args, "--runtime")
                        .unwrap_or_else(|| "deterministic".to_string());
                    let runtime_mode = match runtime_mode_raw.as_str() {
                        "deterministic" => ReactorRuntimeMode::Deterministic,
                        "throughput" => ReactorRuntimeMode::Throughput,
                        _ => {
                            eprintln!(
                                "invalid runtime mode: `{runtime_mode_raw}` (expected deterministic|throughput)"
                            );
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
                    let universe_id = parse_string_flag(args, "--universe");

                    let report_json = match run_lts_check_report_v15(
                        Path::new(project_dir),
                        &manifest_path,
                        &target_runtime,
                        run_engine,
                        runtime_mode,
                        universe_id,
                        &profile,
                    ) {
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

                    if let Some(parent) = out_path.parent() {
                        if let Err(err) = fs::create_dir_all(parent) {
                            if json_mode {
                                println!("{}", error_to_json(&SdkError::Io(err)));
                            } else {
                                eprintln!("{}", SdkError::Io(err));
                            }
                            return 11;
                        }
                    }
                    if let Err(err) = fs::write(
                        &out_path,
                        serde_json::to_string_pretty(&report_json)
                            .unwrap_or_else(|_| "{}".to_string()),
                    ) {
                        if json_mode {
                            println!("{}", error_to_json(&SdkError::Io(err)));
                        } else {
                            eprintln!("{}", SdkError::Io(err));
                        }
                        return 11;
                    }

                    if json_mode {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report_json)
                                .unwrap_or_else(|_| "{}".to_string())
                        );
                    } else {
                        println!(
                            "lts check done (ok={}, profile={}, out={})",
                            report_json
                                .get("ok")
                                .and_then(JsonValue::as_bool)
                                .unwrap_or(false),
                            report_json
                                .get("profile")
                                .and_then(JsonValue::as_str)
                                .unwrap_or("strict"),
                            out_path.display()
                        );
                        println!("{}", render_lts_report_text_v15(&report_json));
                    }

                    if report_json
                        .get("ok")
                        .and_then(JsonValue::as_bool)
                        .unwrap_or(false)
                    {
                        0
                    } else {
                        10
                    }
                }
                "report" => {
                    let report_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                        PathBuf::from("target/ocl/w15/reports/lts_check_report.json")
                    });
                    let json_mode = args.iter().any(|a| a == "--json");
                    let raw = match fs::read_to_string(&report_path) {
                        Ok(value) => value,
                        Err(err) => {
                            if json_mode {
                                println!("{}", error_to_json(&SdkError::Io(err)));
                            } else {
                                eprintln!("{}", SdkError::Io(err));
                            }
                            return 1;
                        }
                    };
                    let report_json: JsonValue = match serde_json::from_str(&raw) {
                        Ok(value) => value,
                        Err(err) => {
                            let diag = SdkError::MissingProject(format!(
                                "X-LTS-REPORT-INVALID: failed to parse report JSON `{}` ({err})",
                                report_path.display()
                            ));
                            if json_mode {
                                println!("{}", error_to_json(&diag));
                            } else {
                                eprintln!("{diag}");
                            }
                            return 1;
                        }
                    };
                    if json_mode {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report_json)
                                .unwrap_or_else(|_| "{}".to_string())
                        );
                    } else {
                        println!("{}", render_lts_report_text_v15(&report_json));
                    }
                    0
                }
                _ => {
                    eprintln!("usage: ocl lts <check|report> ...");
                    2
                }
            }
        }
        "test" => {
            if args.iter().any(|a| a == "--conformance") {
                let conformance_marker =
                    args.iter().position(|a| a == "--conformance").unwrap_or(0);
                let conformance_mode = args
                    .get(conformance_marker.saturating_add(1))
                    .map(String::as_str);
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

                if conformance_mode == Some("list") {
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
                    if json_mode {
                        let scenarios: Vec<JsonValue> = manifest
                            .scenarios
                            .iter()
                            .map(|s| {
                                json!({
                                    "name": s.name,
                                    "path": s.path,
                                    "lane": s.lane,
                                    "steps": s.steps.iter().map(|st| st.as_str()).collect::<Vec<_>>(),
                                })
                            })
                            .collect();
                        println!(
                            "{}",
                            json!({
                                "schema": manifest.schema,
                                "total": scenarios.len(),
                                "scenarios": scenarios
                            })
                        );
                    } else {
                        println!(
                            "conformance list (schema={}, total={})",
                            manifest.schema,
                            manifest.scenarios.len()
                        );
                        for scenario in &manifest.scenarios {
                            let steps = if scenario.steps.is_empty() {
                                "-".to_string()
                            } else {
                                scenario
                                    .steps
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            };
                            println!(
                                "- {} lane={} path={} steps={}",
                                scenario.name, scenario.lane, scenario.path, steps
                            );
                        }
                    }
                    return 0;
                }

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
                    "usage: ocl build <project_dir> [--locked] [--source-only] [--universe <id>] [--attest] [--attest-key <keyid>]"
                );
                return 2;
            };
            let locked = args.iter().any(|a| a == "--locked");
            let source_only = args.iter().any(|a| a == "--source-only");
            let attest = args.iter().any(|a| a == "--attest");
            let attest_key = parse_string_flag(args, "--attest-key");
            let universe_id = parse_string_flag(args, "--universe");
            if let Err(err) = resolve_universe_v1(Path::new(path), locked, universe_id.as_deref()) {
                eprintln!("{err}");
                return 1;
            }
            if source_only {
                match build_project_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        if attest {
                            match build_attestation_v15(Path::new(path), attest_key.as_deref()) {
                                Ok(attest_summary) => {
                                    println!(
                                        "build source-only ok (files_bundled={}, attestation={}, sig={}, manifest_hash={})",
                                        summary.files_bundled,
                                        attest_summary.manifest_path.display(),
                                        attest_summary.sig_path.display(),
                                        attest_summary.manifest_hash_sha256
                                    );
                                    0
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    1
                                }
                            }
                        } else {
                            println!(
                                "build source-only ok (files_bundled={})",
                                summary.files_bundled
                            );
                            0
                        }
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                match build_oclpkg_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        if attest {
                            match build_attestation_v15(Path::new(path), attest_key.as_deref()) {
                                Ok(attest_summary) => {
                                    println!(
                                        "build ok (.oclpkg={}, files_bundled={}, payload_hash_blake3={}, attestation={}, sig={}, manifest_hash={})",
                                        summary.artifact_path.display(),
                                        summary.files_bundled,
                                        summary.payload_hash_blake3,
                                        attest_summary.manifest_path.display(),
                                        attest_summary.sig_path.display(),
                                        attest_summary.manifest_hash_sha256
                                    );
                                    0
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    1
                                }
                            }
                        } else {
                            println!(
                                "build ok (.oclpkg={}, files_bundled={}, payload_hash_blake3={})",
                                summary.artifact_path.display(),
                                summary.files_bundled,
                                summary.payload_hash_blake3
                            );
                            0
                        }
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
        "deps" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl deps <resolve|update|verify> ...");
                return 2;
            };
            match scope {
                "resolve" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl deps resolve <project_dir> [--write-legacy-lock]");
                        return 2;
                    };
                    let write_legacy = args.iter().any(|a| a == "--write-legacy-lock");
                    match resolve_deps_v3(Path::new(path), write_legacy) {
                        Ok(summary) => {
                            println!(
                                "deps resolve ok (deps_resolved={}, lock_hash={}, lock_v3={}, ocl_lock={}, legacy_v2={})",
                                summary.deps_resolved,
                                summary.lock_hash,
                                summary.lock_v3_path.display(),
                                summary.ocl_lock_path.display(),
                                summary.wrote_legacy_lock_v2
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "update" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl deps update <project_dir> [pkg] [--write-legacy-lock]"
                        );
                        return 2;
                    };
                    let target = args
                        .iter()
                        .skip(3)
                        .find(|a| !a.starts_with("--"))
                        .map(String::as_str)
                        .unwrap_or("all");
                    let write_legacy = args.iter().any(|a| a == "--write-legacy-lock");
                    match resolve_deps_v3(Path::new(path), write_legacy) {
                        Ok(summary) => {
                            println!(
                                "deps update ok (target={}, deps_resolved={}, lock_hash={}, lock_v3={}, ocl_lock={}, legacy_v2={})",
                                target,
                                summary.deps_resolved,
                                summary.lock_hash,
                                summary.lock_v3_path.display(),
                                summary.ocl_lock_path.display(),
                                summary.wrote_legacy_lock_v2
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
                        eprintln!("usage: ocl deps verify <project_dir> [--no-lane-policy]");
                        return 2;
                    };
                    let root = Path::new(path);
                    let enforce_lane_policy = !args.iter().any(|a| a == "--no-lane-policy");
                    match verify_dependency_override_guardrails_v10(root)
                        .and_then(|_| verify_deps_lock_v3(root))
                        .and_then(|_| verify_deps_signing_and_trust_v10(root, enforce_lane_policy))
                    {
                        Ok(()) => {
                            println!(
                                "deps verify ok (root={}, lane_policy={})",
                                root.display(),
                                enforce_lane_policy
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
                    eprintln!("usage: ocl deps <resolve|update|verify> ...");
                    2
                }
            }
        }
        "pack" => {
            let Some(scope) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl pack <build|sign|publish|verify> ...");
                return 2;
            };
            match scope {
                "build" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl pack build <project_dir> [--locked]");
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    match build_oclpkg_with_lock(Path::new(path), locked) {
                        Ok(summary) => {
                            println!(
                                "pack build ok (artifact={}, files_bundled={}, payload_hash={}, content_hash={})",
                                summary.artifact_path.display(),
                                summary.files_bundled,
                                summary.payload_hash_blake3,
                                summary.content_hash_sha256
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "sign" => {
                    let Some(artifact) = args.get(2) else {
                        eprintln!("usage: ocl pack sign <artifact.oclpkg>");
                        return 2;
                    };
                    match sign_oclpkg(Path::new(artifact)) {
                        Ok(summary) => {
                            println!(
                                "pack sign ok (artifact={}, package={}, payload_hash={}, signer={})",
                                summary.artifact_path.display(),
                                summary.package_name,
                                summary.payload_hash_blake3,
                                summary.signer_pub_ed25519_b64
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "publish" => {
                    let Some(artifact) = args.get(2) else {
                        eprintln!("usage: ocl pack publish <artifact.oclpkg> [--registry <dir>]");
                        return 2;
                    };
                    let registry = parse_string_flag(args, "--registry")
                        .unwrap_or_else(|| "registry".to_string());
                    match publish_artifact(Path::new(artifact), Path::new(&registry)) {
                        Ok(summary) => {
                            println!(
                                "pack publish ok (artifact={}, registry_index={}, package={}, hash={})",
                                summary.artifact_path.display(),
                                summary.registry_index.display(),
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
                "verify" => {
                    let Some(artifact) = args.get(2) else {
                        eprintln!("usage: ocl pack verify <artifact.oclpkg>");
                        return 2;
                    };
                    match verify_supply_artifact(Path::new(artifact)) {
                        Ok(summary) => {
                            println!(
                                "pack verify ok (package={}, hash={})",
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
                _ => {
                    eprintln!("usage: ocl pack <build|sign|publish|verify> ...");
                    2
                }
            }
        }
        "lock" => {
            let Some(sub) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl lock <sync|sign|verify> ...");
                return 2;
            };
            match sub {
                "sync" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl lock sync <project_dir> [--write-legacy-lock]");
                        return 2;
                    };
                    let write_legacy = args.iter().any(|a| a == "--write-legacy-lock");
                    match resolve_deps_v3(Path::new(path), write_legacy) {
                        Ok(summary) => {
                            println!(
                                "lock sync ok (deps_resolved={}, lock_hash={}, lock_v3={}, ocl_lock={}, legacy_v2={})",
                                summary.deps_resolved,
                                summary.lock_hash,
                                summary.lock_v3_path.display(),
                                summary.ocl_lock_path.display(),
                                summary.wrote_legacy_lock_v2
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "sign" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]");
                        return 2;
                    };
                    let Some(key_id) = parse_string_flag(args, "--key") else {
                        eprintln!("usage: ocl lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]");
                        return 2;
                    };
                    if let Some(lock_value) = parse_string_flag(args, "--lock") {
                        if lock_value.replace('\\', "/") != "deps.lock.v3" {
                            eprintln!(
                                "V-LOCK-PATH-INVALID: v0.15 lock sign currently supports only `--lock deps.lock.v3`"
                            );
                            return 2;
                        }
                    }
                    match sign_deps_lock_v3_v15(Path::new(path), &key_id) {
                        Ok(summary) => {
                            println!(
                                "lock sign ok (lock={}, sig={}, key_id={}, ast_hash={})",
                                summary.lock_path.display(),
                                summary.sig_path.display(),
                                summary.key_id,
                                summary.lock_ast_hash_sha256
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
                        eprintln!("usage: ocl lock verify <project_dir> [--lock deps.lock.v3]");
                        return 2;
                    };
                    if let Some(lock_value) = parse_string_flag(args, "--lock") {
                        if lock_value.replace('\\', "/") != "deps.lock.v3" {
                            eprintln!(
                                "V-LOCK-PATH-INVALID: v0.15 lock verify currently supports only `--lock deps.lock.v3`"
                            );
                            return 2;
                        }
                    }
                    match verify_deps_lock_v3_signature_v15(Path::new(path)) {
                        Ok(summary) => {
                            println!(
                                "lock verify ok (lock={}, sig={}, key_id={}, ast_hash={})",
                                summary.lock_path.display(),
                                summary.sig_path.display(),
                                summary.key_id,
                                summary.lock_ast_hash_sha256
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
                    eprintln!("usage: ocl lock <sync|sign|verify> ...");
                    2
                }
            }
        }
        "perm" => {
            let Some(sub_raw) = args.get(1).map(String::as_str) else {
                eprintln!("usage: ocl perm <snapshot|diff|approve|review|doctor|fix> ...");
                return 2;
            };
            let sub = if sub_raw == "review" { "diff" } else { sub_raw };
            match sub {
                "doctor" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl perm doctor <project_dir> [--out <report.json>]");
                        return 2;
                    };
                    let out_path = parse_string_flag(args, "--out").map(PathBuf::from);
                    match write_permission_doctor_report_v17(Path::new(path), out_path.as_deref()) {
                        Ok(summary) => {
                            println!(
                                "perm doctor ok (lane={}, findings_total={}, blocking_total={}, risk_score={}, report={})",
                                summary.lane,
                                summary.findings_total,
                                summary.blocking_total,
                                summary.risk_score,
                                summary.report_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "fix" => {
                    let Some(mode) = args.get(2).map(String::as_str) else {
                        eprintln!("usage: ocl perm fix <--plan|--apply> <project_dir> [options]");
                        return 2;
                    };
                    match mode {
                        "--plan" | "plan" => {
                            let Some(path) = args.get(3) else {
                                eprintln!(
                                    "usage: ocl perm fix --plan <project_dir> [--out <permission_fix_plan.json>]"
                                );
                                return 2;
                            };
                            let out_path = parse_string_flag(args, "--out").map(PathBuf::from);
                            match write_permission_fix_plan_v17(
                                Path::new(path),
                                out_path.as_deref(),
                            ) {
                                Ok(summary) => {
                                    println!(
                                        "perm fix plan ok (lane={}, findings_total={}, plan_hash={}, plan={})",
                                        summary.lane,
                                        summary.findings_total,
                                        summary.plan_hash_sha256,
                                        summary.plan_path.display()
                                    );
                                    0
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    1
                                }
                            }
                        }
                        "--apply" | "apply" => {
                            let Some(path) = args.get(3) else {
                                eprintln!(
                                    "usage: ocl perm fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>"
                                );
                                return 2;
                            };
                            let Some(approved_by) = parse_string_flag(args, "--by") else {
                                eprintln!(
                                    "usage: ocl perm fix --apply <project_dir> ... --by <id> --date <YYYY-MM-DD>"
                                );
                                return 2;
                            };
                            let Some(date) = parse_string_flag(args, "--date") else {
                                eprintln!(
                                    "usage: ocl perm fix --apply <project_dir> ... --by <id> --date <YYYY-MM-DD>"
                                );
                                return 2;
                            };
                            let Some(justification) = parse_string_flag(args, "--justification")
                            else {
                                eprintln!(
                                    "usage: ocl perm fix --apply <project_dir> ... --justification <text> --ack-risk"
                                );
                                return 2;
                            };
                            let plan_file =
                                parse_string_flag(args, "--plan-file").map(PathBuf::from);
                            let patch_path = parse_string_flag(args, "--patch").map(PathBuf::from);
                            let out_path = parse_string_flag(args, "--out").map(PathBuf::from);
                            let approval_path =
                                parse_string_flag(args, "--approval").map(PathBuf::from);
                            let ack_risk = has_flag(args, "--ack-risk");
                            let options = PermissionFixApplyOptionsV17 {
                                plan_path: plan_file,
                                patch_path,
                                report_path: out_path,
                                approval_path,
                                approved_by,
                                date,
                                justification,
                                ack_risk,
                            };
                            match apply_permission_fix_plan_v17(Path::new(path), &options) {
                                Ok(summary) => {
                                    println!(
                                        "perm fix apply ok (findings_total={}, plan_hash={}, plan={}, patch={}, report={}, approval={})",
                                        summary.findings_total,
                                        summary.plan_hash_sha256,
                                        summary.plan_path.display(),
                                        summary.patch_path.display(),
                                        summary.report_path.display(),
                                        summary.approval_path.display()
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
                            eprintln!(
                                "usage: ocl perm fix <--plan|--apply> <project_dir> [options]"
                            );
                            2
                        }
                    }
                }
                "snapshot" => {
                    let Some(path) = args.get(2) else {
                        eprintln!("usage: ocl perm snapshot <project_dir> [--out-dir <dir>]");
                        return 2;
                    };
                    let out_dir = parse_string_flag(args, "--out-dir").map(PathBuf::from);
                    match write_permission_snapshot_v15(Path::new(path), out_dir.as_deref()) {
                        Ok(summary) => {
                            println!(
                                "perm snapshot ok (packages={}, snapshot_hash={}, snapshot={}, requested={}, granted={}, effective={})",
                                summary.packages,
                                summary.snapshot_hash_sha256,
                                summary.snapshot_path.display(),
                                summary.requested_path.display(),
                                summary.granted_path.display(),
                                summary.effective_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "diff" => {
                    let Some(old_path) = args.get(2) else {
                        eprintln!("usage: ocl perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]");
                        return 2;
                    };
                    let Some(new_path) = args.get(3) else {
                        eprintln!("usage: ocl perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]");
                        return 2;
                    };
                    let out = parse_string_flag(args, "--out")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("permission_diff_report.json"));
                    if let Some(parent) = out.parent() {
                        if !parent.as_os_str().is_empty() {
                            if let Err(err) = fs::create_dir_all(parent) {
                                eprintln!("{err}");
                                return 1;
                            }
                        }
                    }
                    let approval_path = parse_string_flag(args, "--approval").map(PathBuf::from);
                    match write_permission_diff_report_v15(
                        Path::new(old_path),
                        Path::new(new_path),
                        &out,
                        approval_path.as_deref(),
                    ) {
                        Ok(summary) => {
                            println!(
                                "perm diff ok (diff_hash={}, has_changes={}, introduces_new_permissions={}, approval_checked={}, approved={}, report={})",
                                summary.permission_diff_hash,
                                summary.has_changes,
                                summary.introduces_new_permissions,
                                summary.approval_checked,
                                summary.approved,
                                summary.report_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
                }
                "approve" => {
                    let Some(diff_report) = args.get(2) else {
                        eprintln!("usage: ocl perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]");
                        return 2;
                    };
                    let Some(approved_by) = parse_string_flag(args, "--by") else {
                        eprintln!("usage: ocl perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]");
                        return 2;
                    };
                    let Some(date) = parse_string_flag(args, "--date") else {
                        eprintln!("usage: ocl perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]");
                        return 2;
                    };
                    let approval_path = parse_string_flag(args, "--approval")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("permissions.approval.toml"));
                    let note = parse_string_flag(args, "--note");
                    match approve_permission_diff_v15(
                        Path::new(diff_report),
                        &approval_path,
                        &approved_by,
                        &date,
                        note.as_deref(),
                    ) {
                        Ok(summary) => {
                            println!(
                                "perm approve ok (diff_hash={}, approvals_total={}, approval={})",
                                summary.diff_hash,
                                summary.approvals_total,
                                summary.approval_path.display()
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
                    eprintln!("usage: ocl perm <snapshot|diff|approve|review|doctor|fix> ...");
                    2
                }
            }
        }
        "cassette" => run_cassette_command(args),
        "budget" => run_budget_command(args),
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
            if args.get(1).map(String::as_str) == Some("--contract-sign") {
                let Some(contract_path) = args.get(2) else {
                    eprintln!("usage: ocl verify --contract-sign <contract.json> [--signer-id <id>] [--trust-epoch <u32>]");
                    return 2;
                };
                let signer_id = parse_string_flag(args, "--signer-id")
                    .unwrap_or_else(|| "w17-default".to_string());
                let trust_epoch = parse_u32_flag(args, "--trust-epoch").unwrap_or(1) as u64;
                match sign_contract_json_v17(Path::new(contract_path), &signer_id, trust_epoch) {
                    Ok(sig_path) => {
                        println!(
                            "verify contract sign ok (contract={}, sig={}, signer_id={}, trust_epoch={})",
                            contract_path,
                            sig_path.display(),
                            signer_id,
                            trust_epoch
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else if args.get(1).map(String::as_str) == Some("--contract") {
                let Some(contract_path) = args.get(2) else {
                    eprintln!("usage: ocl verify --contract <contract.json>");
                    return 2;
                };
                match inspect_contract_json_v17(Path::new(contract_path)) {
                    Ok(summary) => {
                        println!(
                            "verify contract ok (contract_id={}, version={}, hash_sha256={}, sig={})",
                            summary.contract_id,
                            summary.version,
                            summary.contract_hash_sha256,
                            summary.signature_path.display()
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else if args.get(1).map(String::as_str) == Some("--contract-signature") {
                let Some(sig_or_contract) = args.get(2) else {
                    eprintln!(
                        "usage: ocl verify --contract-signature <contract.json.sig|contract.json>"
                    );
                    return 2;
                };
                let path = Path::new(sig_or_contract);
                let result = if sig_or_contract.ends_with(".sig") {
                    verify_contract_signature_file_v17(path)
                } else {
                    verify_contract_json_signature_v17(path)
                };
                match result {
                    Ok(summary) => {
                        println!(
                            "verify contract signature ok (contract_id={}, hash_sha256={}, signer_id={}, trust_epoch={})",
                            summary.contract_id,
                            summary.contract_hash_sha256,
                            summary.signer_id.as_deref().unwrap_or("-"),
                            summary.trust_epoch.unwrap_or(0)
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else if args.get(1).map(String::as_str) == Some("--attest") {
                let Some(artifact_dir) = args.get(2) else {
                    eprintln!("usage: ocl verify --attest <artifact_dir>");
                    return 2;
                };
                match verify_build_attestation_v15(Path::new(artifact_dir)) {
                    Ok(summary) => {
                        println!(
                            "verify attest ok (manifest={}, sig={}, key_id={}, manifest_hash={})",
                            summary.manifest_path.display(),
                            summary.sig_path.display(),
                            summary.key_id,
                            summary.manifest_hash_sha256
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else if args.get(1).map(String::as_str) == Some("--repro") {
                let Some(artifact_dir) = args.get(2) else {
                    eprintln!("usage: ocl verify --repro <artifact_dir>");
                    return 2;
                };
                match verify_build_repro_v15(Path::new(artifact_dir)) {
                    Ok(summary) => {
                        println!(
                            "verify repro ok (artifact_dir={}, baseline_hash={}, repro_hash={}, exclusions={})",
                            summary.artifact_dir.display(),
                            summary.baseline_hash_sha256,
                            summary.repro_hash_sha256,
                            summary.exclusions.join(",")
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                let Some(path) = args.get(1) else {
                    eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                    eprintln!("       ocl verify --contract-sign <contract.json> [--signer-id <id>] [--trust-epoch <u32>]");
                    eprintln!("       ocl verify --contract <contract.json>");
                    eprintln!(
                        "       ocl verify --contract-signature <contract.json.sig|contract.json>"
                    );
                    eprintln!("       ocl verify --attest <artifact_dir>");
                    eprintln!("       ocl verify --repro <artifact_dir>");
                    return 2;
                };
                let Some(phenotype) = parse_string_flag(args, "--phenotype") else {
                    eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]");
                    eprintln!("       ocl verify --contract-sign <contract.json> [--signer-id <id>] [--trust-epoch <u32>]");
                    eprintln!("       ocl verify --contract <contract.json>");
                    eprintln!(
                        "       ocl verify --contract-signature <contract.json.sig|contract.json>"
                    );
                    eprintln!("       ocl verify --attest <artifact_dir>");
                    eprintln!("       ocl verify --repro <artifact_dir>");
                    return 2;
                };
                let registry =
                    parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
                let locked = args.iter().any(|a| a == "--locked");
                let universe_id = parse_string_flag(args, "--universe");
                if let Err(err) =
                    resolve_universe_v1(Path::new(path), locked, universe_id.as_deref())
                {
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

#[derive(Debug, Default)]
struct UpgradeCheckFeaturesV14 {
    uses_tool_packs: bool,
    uses_consumer_packs: bool,
    uses_shadow: bool,
    uses_quarantine: bool,
    has_dependencies: bool,
}

fn detect_upgrade_check_features_v14(project_root: &Path) -> UpgradeCheckFeaturesV14 {
    let mut out = UpgradeCheckFeaturesV14::default();
    let manifest_path = project_root.join("Ocl.toml");
    if let Ok(raw_manifest) = fs::read_to_string(&manifest_path) {
        let lowered = raw_manifest.to_ascii_lowercase();
        if lowered.contains("[dependencies]")
            || lowered.contains("[dependency.")
            || lowered.contains("source =")
        {
            out.has_dependencies = true;
        }
        if lowered.contains("lane = \"quarantine\"") {
            out.uses_quarantine = true;
        }
    }

    let mut ocl_files = Vec::<PathBuf>::new();
    collect_ocl_sources_v14(&project_root.join("src"), &mut ocl_files);
    if ocl_files.is_empty() {
        collect_ocl_sources_v14(project_root, &mut ocl_files);
    }

    for path in ocl_files {
        if let Ok(raw) = fs::read_to_string(path) {
            let lowered = raw.to_ascii_lowercase();
            if lowered.contains("std.fs.")
                || lowered.contains("std.kv.")
                || lowered.contains("std.time.")
            {
                out.uses_tool_packs = true;
            }
            if lowered.contains("std.ui.") || lowered.contains("std.game.") {
                out.uses_consumer_packs = true;
            }
            if lowered.contains("std.shadow.") {
                out.uses_shadow = true;
            }
            if lowered.contains("std.net.")
                || lowered.contains("std.proc.")
                || lowered.contains("std.time.wallclock")
            {
                out.uses_quarantine = true;
            }
        }
    }

    out
}

fn collect_ocl_sources_v14(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ocl_sources_v14(&path, out);
            continue;
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("ocl"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

fn select_upgrade_manifest_subset_v14(
    manifest: &ConformanceManifestV1,
    features: &UpgradeCheckFeaturesV14,
) -> ConformanceManifestV1 {
    let scenarios = manifest
        .scenarios
        .iter()
        .filter(|sc| is_upgrade_scenario_relevant_v14(&sc.name, features))
        .cloned()
        .collect::<Vec<_>>();
    if scenarios.is_empty() {
        return manifest.clone();
    }
    ConformanceManifestV1 {
        schema: manifest.schema.clone(),
        scenarios,
    }
}

fn is_upgrade_scenario_relevant_v14(name: &str, features: &UpgradeCheckFeaturesV14) -> bool {
    if name.starts_with("core-") || name.starts_with("foundation-") || name.starts_with("cache-") {
        return true;
    }
    if name.starts_with("packs-tool-") {
        return features.uses_tool_packs;
    }
    if name.starts_with("packs-consumer-") {
        return features.uses_consumer_packs;
    }
    if name.starts_with("shadow-") {
        return features.uses_shadow;
    }
    if name.starts_with("quarantine-") {
        return features.uses_quarantine;
    }
    if name.starts_with("supplychain-") {
        return features.has_dependencies;
    }
    true
}

fn extract_error_code_token_v15(message: &str, fallback: &str) -> String {
    let token = message
        .trim()
        .split(|ch: char| ch == ':' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim();
    if token.is_empty() {
        return fallback.to_string();
    }
    if token.starts_with("X-")
        || token.starts_with("RC-")
        || token.starts_with("V-")
        || token.starts_with("W9-")
    {
        return token.to_string();
    }
    fallback.to_string()
}

fn push_lts_risk_v15(
    risks: &mut Vec<(String, String, String)>,
    gate: &str,
    message: &str,
    fallback_code: &str,
) {
    let code = extract_error_code_token_v15(message, fallback_code);
    risks.push((gate.to_string(), code, message.to_string()));
}

fn run_lts_check_report_v15(
    project_root: &Path,
    manifest_path: &Path,
    target_runtime: &str,
    run_engine: RunEngine,
    runtime_mode: ReactorRuntimeMode,
    universe_id: Option<String>,
    profile: &str,
) -> Result<JsonValue, SdkError> {
    let lane = read_lane_and_entry_for_v071(project_root).0;
    let mut risks = Vec::<(String, String, String)>::new();

    let lock_gate = match verify_deps_lock_v3_signature_v15(project_root) {
        Ok(summary) => json!({
            "ok": true,
            "lock_path": summary.lock_path.to_string_lossy(),
            "sig_path": summary.sig_path.to_string_lossy(),
            "key_id": summary.key_id,
            "lock_ast_hash_sha256": summary.lock_ast_hash_sha256
        }),
        Err(err) => {
            let msg = err.to_string();
            push_lts_risk_v15(
                &mut risks,
                "lock_signature",
                &msg,
                "X-LOCK-SIGNATURE-INVALID",
            );
            json!({
                "ok": false,
                "error_code": extract_error_code_token_v15(&msg, "X-LOCK-SIGNATURE-INVALID"),
                "message": msg
            })
        }
    };

    let trust_gate = match verify_deps_signing_and_trust_v10(project_root, true) {
        Ok(_) => json!({ "ok": true }),
        Err(err) => {
            let msg = err.to_string();
            push_lts_risk_v15(&mut risks, "trust", &msg, "X-TRUST-SIGNATURE-REQUIRED");
            json!({
                "ok": false,
                "error_code": extract_error_code_token_v15(&msg, "X-TRUST-SIGNATURE-REQUIRED"),
                "message": msg
            })
        }
    };

    let permission_gate = if lane == "locked_v071" {
        let baseline = project_root.join("permissions.snapshot.json");
        if !baseline.exists() {
            let msg = format!(
                "X-PERMISSION-REVIEW-REQUIRED: missing baseline snapshot `{}`; run `ocl perm snapshot {}` first",
                baseline.display(),
                project_root.display()
            );
            push_lts_risk_v15(
                &mut risks,
                "permission_review",
                &msg,
                "X-PERMISSION-REVIEW-REQUIRED",
            );
            json!({
                "ok": false,
                "error_code": "X-PERMISSION-REVIEW-REQUIRED",
                "message": msg,
                "baseline_present": false
            })
        } else {
            let lts_tmp_root = project_root
                .join("target")
                .join("ocl")
                .join("w15")
                .join("lts");
            fs::create_dir_all(&lts_tmp_root)?;
            match write_permission_snapshot_v15(project_root, Some(&lts_tmp_root)) {
                Ok(snapshot_summary) => {
                    let approval_path = project_root.join("permissions.approval.toml");
                    let diff_path = lts_tmp_root.join("permission_diff_report.json");
                    match write_permission_diff_report_v15(
                        &baseline,
                        &snapshot_summary.snapshot_path,
                        &diff_path,
                        Some(&approval_path),
                    ) {
                        Ok(diff_summary) => json!({
                            "ok": true,
                            "baseline_present": true,
                            "snapshot_hash_sha256": snapshot_summary.snapshot_hash_sha256,
                            "permission_diff_hash": diff_summary.permission_diff_hash,
                            "has_changes": diff_summary.has_changes,
                            "introduces_new_permissions": diff_summary.introduces_new_permissions,
                            "approval_checked": diff_summary.approval_checked,
                            "approved": diff_summary.approved,
                            "report_path": diff_summary.report_path.to_string_lossy()
                        }),
                        Err(err) => {
                            let msg = err.to_string();
                            push_lts_risk_v15(
                                &mut risks,
                                "permission_review",
                                &msg,
                                "X-PERMISSION-APPROVAL-MISSING",
                            );
                            json!({
                                "ok": false,
                                "baseline_present": true,
                                "error_code": extract_error_code_token_v15(&msg, "X-PERMISSION-APPROVAL-MISSING"),
                                "message": msg
                            })
                        }
                    }
                }
                Err(err) => {
                    let msg = err.to_string();
                    push_lts_risk_v15(
                        &mut risks,
                        "permission_review",
                        &msg,
                        "X-PERMISSION-REVIEW-REQUIRED",
                    );
                    json!({
                        "ok": false,
                        "baseline_present": true,
                        "error_code": extract_error_code_token_v15(&msg, "X-PERMISSION-REVIEW-REQUIRED"),
                        "message": msg
                    })
                }
            }
        }
    } else {
        json!({
            "ok": true,
            "skipped": true,
            "reason": format!("lane {} does not require strict permission baseline gate", lane)
        })
    };

    let manifest = parse_conformance_manifest_v1(manifest_path)?;
    let features = detect_upgrade_check_features_v14(project_root);
    let selected_manifest = select_upgrade_manifest_subset_v14(&manifest, &features);
    let selected_names: Vec<String> = selected_manifest
        .scenarios
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let locked_mode = matches!(lane.as_str(), "locked_v071" | "locked_v06");
    let run_options = ConformanceRunOptionsV1 {
        locked: locked_mode,
        engine: run_engine,
        runtime_mode,
        universe_id,
    };

    let (upgrade_gate, conformance_gate, first_divergence) =
        if selected_manifest.scenarios.is_empty() {
            let msg = "X-UPGRADE-CHECK-FAILED: no conformance scenarios selected from manifest"
                .to_string();
            push_lts_risk_v15(&mut risks, "upgrade_check", &msg, "X-UPGRADE-CHECK-FAILED");
            push_lts_risk_v15(&mut risks, "conformance", &msg, "X-UPGRADE-CHECK-FAILED");
            (
                json!({
                    "ok": false,
                    "error_code": "X-UPGRADE-CHECK-FAILED",
                    "selected_scenarios": selected_names,
                    "message": msg
                }),
                json!({
                    "ok": false,
                    "error_code": "X-UPGRADE-CHECK-FAILED",
                    "scenarios_total": 0u64,
                    "scenarios_passed": 0u64,
                    "scenarios_failed": 0u64,
                    "required_digest": JsonValue::Null,
                    "message": msg
                }),
                JsonValue::Null,
            )
        } else {
            let conformance = run_conformance_v1(Path::new("."), &selected_manifest, run_options);
            let first_divergence_value = conformance
                .results
                .iter()
                .enumerate()
                .find(|(_, item)| !item.ok)
                .map(|(idx, item)| {
                    json!({
                        "index": idx,
                        "scenario": item.name,
                        "reason": item.reason.clone().unwrap_or_default(),
                    })
                })
                .unwrap_or(JsonValue::Null);
            let ok = conformance.scenarios_failed == 0;
            if !ok {
                let reason = first_divergence_value
                    .get("reason")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("X-UPGRADE-CHECK-FAILED");
                push_lts_risk_v15(
                    &mut risks,
                    "upgrade_check",
                    reason,
                    "X-UPGRADE-CHECK-FAILED",
                );
                push_lts_risk_v15(&mut risks, "conformance", reason, "X-UPGRADE-CHECK-FAILED");
            }
            (
                json!({
                    "ok": ok,
                    "target_runtime": target_runtime,
                    "selected_scenarios": selected_names,
                    "scenarios_total": conformance.scenarios_total,
                    "scenarios_failed": conformance.scenarios_failed,
                    "required_digest": conformance.required_digest,
                    "runner_contract": "ocl test --conformance"
                }),
                json!({
                    "ok": ok,
                    "run_id": conformance.run_id,
                    "scenarios_total": conformance.scenarios_total,
                    "scenarios_passed": conformance.scenarios_passed,
                    "scenarios_failed": conformance.scenarios_failed,
                    "required_digest": conformance.required_digest
                }),
                first_divergence_value,
            )
        };

    let mut risk_values = risks
        .into_iter()
        .map(|(gate, code, message)| {
            json!({
                "gate": gate,
                "code": code,
                "message": message
            })
        })
        .collect::<Vec<JsonValue>>();
    risk_values.sort_by(|a, b| {
        let ag = a
            .get("gate")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let bg = b
            .get("gate")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let ac = a
            .get("code")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let bc = b
            .get("code")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let am = a
            .get("message")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let bm = b
            .get("message")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        ag.cmp(bg).then(ac.cmp(bc)).then(am.cmp(bm))
    });
    risk_values.dedup_by(|a, b| {
        a.get("gate") == b.get("gate")
            && a.get("code") == b.get("code")
            && a.get("message") == b.get("message")
    });

    let ok = lock_gate
        .get("ok")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
        && trust_gate
            .get("ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        && permission_gate
            .get("ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        && upgrade_gate
            .get("ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        && conformance_gate
            .get("ok")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false);

    Ok(json!({
        "schema": "ocl.lts_check.v1",
        "project_dir": project_root.to_string_lossy(),
        "project_lane": lane,
        "profile": profile,
        "manifest_path": manifest_path.to_string_lossy(),
        "target_runtime": target_runtime,
        "gates": {
            "lock_signature": lock_gate,
            "trust": trust_gate,
            "permission_review": permission_gate,
            "upgrade_check": upgrade_gate,
            "conformance": conformance_gate
        },
        "first_divergence": first_divergence,
        "risks": risk_values,
        "ok": ok,
        "error_code": if ok { JsonValue::Null } else { JsonValue::String("X-LTS-CHECK-FAILED".to_string()) }
    }))
}

fn render_lts_report_text_v15(report: &JsonValue) -> String {
    let ok = report
        .get("ok")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let profile = report
        .get("profile")
        .and_then(JsonValue::as_str)
        .unwrap_or("strict");
    let lane = report
        .get("project_lane")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    let mut out = String::new();
    out.push_str("lts report\n");
    out.push_str("  schema: ");
    out.push_str(
        report
            .get("schema")
            .and_then(JsonValue::as_str)
            .unwrap_or("ocl.lts_check.v1"),
    );
    out.push('\n');
    out.push_str("  profile: ");
    out.push_str(profile);
    out.push('\n');
    out.push_str("  lane: ");
    out.push_str(lane);
    out.push('\n');
    out.push_str("  ok: ");
    out.push_str(if ok { "true" } else { "false" });
    out.push('\n');

    let gate_names = [
        "lock_signature",
        "trust",
        "permission_review",
        "upgrade_check",
        "conformance",
    ];
    for gate in gate_names {
        let gate_ok = report
            .get("gates")
            .and_then(|g| g.get(gate))
            .and_then(|g| g.get("ok"))
            .and_then(JsonValue::as_bool)
            .unwrap_or(false);
        out.push_str("  gate.");
        out.push_str(gate);
        out.push_str(": ");
        out.push_str(if gate_ok { "ok" } else { "fail" });
        out.push('\n');
    }

    let risks = report
        .get("risks")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    out.push_str("  risks: ");
    out.push_str(&risks.len().to_string());
    out.push('\n');
    for item in risks {
        let gate = item
            .get("gate")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown");
        let code = item
            .get("code")
            .and_then(JsonValue::as_str)
            .unwrap_or("X-LTS-CHECK-FAILED");
        let msg = item
            .get("message")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        out.push_str("    - ");
        out.push_str(gate);
        out.push_str(": ");
        out.push_str(code);
        if !msg.is_empty() {
            out.push_str(" :: ");
            out.push_str(msg);
        }
        out.push('\n');
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

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
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

const CASSETTE_SCHEMA_VERSION_V17: u64 = 17;
const CASSETTE_BLOCK_STORE_VERSION_V17: u64 = 1;
const CASSETTE_HASHER_VERSION_V17: &str = "sha256-v1";
const CASSETTE_BLOCKS_INDEX_FILE_V17: &str = "cassette_blocks_index.json";
const CASSETTE_STORAGE_META_FILE_V17: &str = "cassette_storage_meta.toml";
const CASSETTE_BLOCKS_DIR_V17: &str = "blocks";
const CASSETTE_DEFAULT_MAX_ENTRIES_V17: usize = 200_000;
const CASSETTE_DEFAULT_MAX_BLOCK_BYTES_V17: usize = 256 * 1024;

#[derive(Debug, Clone)]
struct CassetteEntryUpgradeV17 {
    entry_id: String,
    canonical_entry: String,
    block_id: String,
}

#[derive(Debug, Clone, Default)]
struct CassettePrunePlanV17 {
    total_blocks: usize,
    referenced_blocks: usize,
    orphan_blocks: Vec<String>,
    missing_blocks: Vec<String>,
    prune_bytes: u64,
    ttl_days: Option<u32>,
}

#[derive(Debug, Clone, Default)]
struct BudgetEdgeAggV17 {
    caller: String,
    key: String,
    observe_count: u32,
    insufficient_count: u32,
    deferred_count: u32,
    budget_pressure_count: u32,
}

fn run_cassette_command(args: &[String]) -> i32 {
    let Some(subcmd) = args.get(1).map(String::as_str) else {
        eprintln!(
            "usage: ocl cassette <stats|prune|gc|upgrade> ...\n  stats <artifact_dir> [--json]\n  prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]\n  gc <artifact_dir> [--json]\n  upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]"
        );
        return 2;
    };
    match subcmd {
        "stats" => run_cassette_stats_command_v17(args),
        "prune" => run_cassette_prune_command_v17(args),
        "gc" => run_cassette_gc_command_v17(args),
        "upgrade" => run_cassette_upgrade_command_v17(args),
        _ => {
            eprintln!(
                "usage: ocl cassette <stats|prune|gc|upgrade> ...\n  stats <artifact_dir> [--json]\n  prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]\n  gc <artifact_dir> [--json]\n  upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]"
            );
            2
        }
    }
}

fn run_budget_command(args: &[String]) -> i32 {
    let Some(subcmd) = args.get(1).map(String::as_str) else {
        eprintln!("usage: ocl budget <analyze|doctor> <artifact_dir|audit.jsonl> [--json]");
        return 2;
    };
    let Some(path) = args.get(2) else {
        eprintln!("usage: ocl budget <analyze|doctor> <artifact_dir|audit.jsonl> [--json]");
        return 2;
    };
    let json_mode = has_flag(args, "--json");
    let events = match read_trace_events_for_view_v11(Path::new(path), false) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    let report = build_budget_analyze_report_v17(&events);
    match subcmd {
        "analyze" => {
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("budget analyze");
                println!(
                    "observe_events={}",
                    json_u64_or_zero(&report, "observe_events")
                );
                println!(
                    "budget_pressure_events={}",
                    json_u64_or_zero(&report, "budget_pressure_events")
                );
                println!("edges={}", json_u64_or_zero(&report, "edge_count"));
            }
            0
        }
        "doctor" => {
            let doctor = build_budget_doctor_report_v17(&report);
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&doctor).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("budget doctor");
                println!(
                    "issues={}",
                    doctor
                        .get("issues")
                        .and_then(JsonValue::as_array)
                        .map(|v| v.len())
                        .unwrap_or(0)
                );
                if let Some(items) = doctor.get("issues").and_then(JsonValue::as_array) {
                    for item in items.iter().take(5) {
                        let caller = item
                            .get("caller")
                            .and_then(JsonValue::as_str)
                            .unwrap_or("-");
                        let key = item.get("key").and_then(JsonValue::as_str).unwrap_or("-");
                        let pressure = item
                            .get("budget_pressure_count")
                            .and_then(JsonValue::as_u64)
                            .unwrap_or(0);
                        println!("  - caller={caller} key={key} pressure={pressure}");
                    }
                }
            }
            0
        }
        _ => {
            eprintln!("usage: ocl budget <analyze|doctor> <artifact_dir|audit.jsonl> [--json]");
            2
        }
    }
}

fn json_u64_or_zero(value: &JsonValue, key: &str) -> u64 {
    value.get(key).and_then(JsonValue::as_u64).unwrap_or(0)
}

fn resolve_artifact_dir_for_cassette_arg_v17(path: &Path) -> Result<PathBuf, SdkError> {
    if path.join("cassette").is_dir() {
        return Ok(path.to_path_buf());
    }
    let artifacts_root = path.join(".ocl_artifacts");
    if artifacts_root.is_dir() {
        return latest_artifact_dir_v17(&artifacts_root);
    }
    Err(SdkError::MissingProject(format!(
        "V-CASSETTE-PATH: `{}` is neither an artifact dir nor a project dir with .ocl_artifacts",
        path.display()
    )))
}

fn latest_artifact_dir_v17(artifacts_root: &Path) -> Result<PathBuf, SdkError> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(artifacts_root)? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs.pop().ok_or_else(|| {
        SdkError::MissingProject(format!(
            "V-CASSETTE-PATH: no artifact directories found in {}",
            artifacts_root.display()
        ))
    })
}

fn run_cassette_stats_command_v17(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: ocl cassette stats <artifact_dir> [--json]");
        return 2;
    };
    let json_mode = has_flag(args, "--json");
    let artifact_dir = match resolve_artifact_dir_for_cassette_arg_v17(Path::new(path)) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    match build_cassette_stats_report_v17(&artifact_dir) {
        Ok((report, report_path)) => {
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("cassette stats");
                println!("artifact={}", artifact_dir.display());
                println!("entries={}", json_u64_or_zero(&report, "entries"));
                println!("blocks={}", json_u64_or_zero(&report, "blocks"));
                println!(
                    "referenced_blocks={}",
                    json_u64_or_zero(&report, "referenced_blocks")
                );
                println!(
                    "orphan_blocks={}",
                    json_u64_or_zero(&report, "orphan_blocks")
                );
                println!("bytes_total={}", json_u64_or_zero(&report, "bytes_total"));
                println!("report={}", report_path.display());
            }
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn run_cassette_prune_command_v17(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!(
            "usage: ocl cassette prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]"
        );
        return 2;
    };
    let plan_mode = has_flag(args, "--plan");
    let apply_mode = has_flag(args, "--apply");
    if plan_mode == apply_mode {
        eprintln!(
            "usage: ocl cassette prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]"
        );
        return 2;
    }
    let ttl_days = parse_u32_flag(args, "--ttl-days");
    let json_mode = has_flag(args, "--json");
    let artifact_dir = match resolve_artifact_dir_for_cassette_arg_v17(Path::new(path)) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    let plan = match build_cassette_prune_plan_v17(&artifact_dir, ttl_days) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    if apply_mode {
        match apply_cassette_prune_plan_v17(&artifact_dir, &plan) {
            Ok(applied_blocks) => {
                let payload = json!({
                    "mode": "apply",
                    "applied_blocks": applied_blocks,
                    "orphan_blocks": plan.orphan_blocks,
                    "missing_blocks": plan.missing_blocks,
                    "ttl_days": plan.ttl_days,
                    "prune_bytes": plan.prune_bytes
                });
                if json_mode {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                    );
                } else {
                    println!("cassette prune apply");
                    println!("artifact={}", artifact_dir.display());
                    println!("applied_blocks={applied_blocks}");
                    println!(
                        "prune_bytes={}",
                        payload
                            .get("prune_bytes")
                            .and_then(JsonValue::as_u64)
                            .unwrap_or(0)
                    );
                }
                0
            }
            Err(err) => {
                eprintln!("{err}");
                1
            }
        }
    } else {
        let payload = render_cassette_prune_plan_json_v17(&plan);
        if json_mode {
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
            );
        } else {
            println!("cassette prune plan");
            println!("artifact={}", artifact_dir.display());
            println!("orphan_blocks={}", plan.orphan_blocks.len());
            println!("missing_blocks={}", plan.missing_blocks.len());
            println!("prune_bytes={}", plan.prune_bytes);
        }
        0
    }
}

fn run_cassette_gc_command_v17(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: ocl cassette gc <artifact_dir> [--json]");
        return 2;
    };
    let json_mode = has_flag(args, "--json");
    let artifact_dir = match resolve_artifact_dir_for_cassette_arg_v17(Path::new(path)) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    let plan = match build_cassette_prune_plan_v17(&artifact_dir, None) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    if !plan.missing_blocks.is_empty() {
        eprintln!(
            "X-CASSETTE-MISSING-BLOCK: cannot run gc because {} referenced blocks are missing",
            plan.missing_blocks.len()
        );
        return 1;
    }
    match apply_cassette_prune_plan_v17(&artifact_dir, &plan) {
        Ok(applied_blocks) => {
            let payload = json!({
                "mode": "gc",
                "applied_blocks": applied_blocks,
                "orphan_blocks_before": plan.orphan_blocks.len(),
                "missing_blocks": plan.missing_blocks,
            });
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("cassette gc");
                println!("artifact={}", artifact_dir.display());
                println!("applied_blocks={applied_blocks}");
            }
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn run_cassette_upgrade_command_v17(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: ocl cassette upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]");
        return 2;
    };
    let plan_mode = has_flag(args, "--plan");
    let apply_mode = has_flag(args, "--apply");
    if plan_mode == apply_mode {
        eprintln!("usage: ocl cassette upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]");
        return 2;
    }
    let max_entries = parse_u32_flag(args, "--max-entries")
        .map(|v| v as usize)
        .unwrap_or(CASSETTE_DEFAULT_MAX_ENTRIES_V17);
    let max_block_bytes = parse_u32_flag(args, "--max-block-bytes")
        .map(|v| v as usize)
        .unwrap_or(CASSETTE_DEFAULT_MAX_BLOCK_BYTES_V17);
    let json_mode = has_flag(args, "--json");
    let artifact_dir = match resolve_artifact_dir_for_cassette_arg_v17(Path::new(path)) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    if plan_mode {
        match build_cassette_upgrade_plan_v17(&artifact_dir, max_entries, max_block_bytes) {
            Ok(payload) => {
                if json_mode {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                    );
                } else {
                    println!("cassette upgrade plan");
                    println!("artifact={}", artifact_dir.display());
                    println!(
                        "requires_upgrade={}",
                        payload
                            .get("requires_upgrade")
                            .and_then(JsonValue::as_bool)
                            .unwrap_or(false)
                    );
                    println!("entries={}", json_u64_or_zero(&payload, "entries"));
                    println!("blocks={}", json_u64_or_zero(&payload, "unique_blocks"));
                }
                0
            }
            Err(err) => {
                eprintln!("{err}");
                1
            }
        }
    } else {
        match apply_cassette_upgrade_v17(&artifact_dir, max_entries, max_block_bytes) {
            Ok((payload, report_path)) => {
                if json_mode {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                    );
                } else {
                    println!("cassette upgrade apply");
                    println!("artifact={}", artifact_dir.display());
                    println!("entries={}", json_u64_or_zero(&payload, "entries"));
                    println!("blocks={}", json_u64_or_zero(&payload, "unique_blocks"));
                    println!("report={}", report_path.display());
                }
                0
            }
            Err(err) => {
                eprintln!("{err}");
                1
            }
        }
    }
}

fn read_cassette_entries_for_upgrade_v17(
    artifact_dir: &Path,
    max_entries: usize,
    max_block_bytes: usize,
) -> Result<Vec<CassetteEntryUpgradeV17>, SdkError> {
    let cassette_path = artifact_dir.join("cassette").join("cassette.jsonl");
    if !cassette_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-MISSING: missing cassette.jsonl at {}",
            cassette_path.display()
        )));
    }
    let raw = fs::read_to_string(&cassette_path)?;
    let mut out = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if out.len() >= max_entries {
            return Err(SdkError::MissingProject(format!(
                "X-CASSETTE-MAX-ENTRIES: entry cap exceeded (max_entries={max_entries})"
            )));
        }
        let parsed: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-FORMAT: invalid json at line {} in {} ({err})",
                idx + 1,
                cassette_path.display()
            ))
        })?;
        let Some(entry_id) = parsed
            .as_object()
            .and_then(|obj| obj.get("id"))
            .and_then(JsonValue::as_str)
        else {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-FORMAT: line {} missing `id` in {}",
                idx + 1,
                cassette_path.display()
            )));
        };
        let canonical_entry = serde_json::to_string(&parsed).map_err(|err| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-FORMAT: cannot canonicalize entry `{entry_id}` ({err})"
            ))
        })?;
        if canonical_entry.len() > max_block_bytes {
            return Err(SdkError::MissingProject(format!(
                "X-CASSETTE-BLOCK-TOO-LARGE: entry `{entry_id}` size={} exceeds max_block_bytes={max_block_bytes}",
                canonical_entry.len()
            )));
        }
        out.push(CassetteEntryUpgradeV17 {
            entry_id: entry_id.to_string(),
            block_id: sha256_hex(canonical_entry.as_bytes()),
            canonical_entry,
        });
    }
    Ok(out)
}

fn read_cassette_block_index_v17(
    artifact_dir: &Path,
) -> Result<BTreeMap<String, Vec<String>>, SdkError> {
    let index_path = artifact_dir
        .join("cassette")
        .join(CASSETTE_BLOCKS_INDEX_FILE_V17);
    if !index_path.exists() {
        return Ok(BTreeMap::new());
    }
    let parsed: JsonValue =
        serde_json::from_str(&fs::read_to_string(&index_path)?).map_err(|err| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-INDEX: invalid json {} ({err})",
                index_path.display()
            ))
        })?;
    let Some(entries) = parsed.get("entry_refs").and_then(JsonValue::as_array) else {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-INDEX: missing entry_refs in {}",
            index_path.display()
        )));
    };
    let mut out = BTreeMap::new();
    for row in entries {
        let Some(entry_id) = row.get("entry_id").and_then(JsonValue::as_str) else {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-INDEX: entry_refs missing entry_id in {}",
                index_path.display()
            )));
        };
        let Some(blocks) = row.get("blocks").and_then(JsonValue::as_array) else {
            return Err(SdkError::MissingProject(format!(
                "V-CASSETTE-INDEX: entry_refs missing blocks array for `{entry_id}`"
            )));
        };
        let mut refs = Vec::new();
        for block in blocks {
            let Some(id) = block.as_str() else {
                return Err(SdkError::MissingProject(format!(
                    "V-CASSETTE-INDEX: block id must be string for `{entry_id}`"
                )));
            };
            refs.push(id.to_string());
        }
        out.insert(entry_id.to_string(), refs);
    }
    Ok(out)
}

fn collect_block_files_v17(artifact_dir: &Path) -> Result<BTreeMap<String, PathBuf>, SdkError> {
    let blocks_dir = artifact_dir.join("cassette").join(CASSETTE_BLOCKS_DIR_V17);
    if !blocks_dir.exists() {
        return Ok(BTreeMap::new());
    }
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(&blocks_dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let Some(stem) = path.file_stem() else {
            continue;
        };
        let block_id = stem.to_string_lossy().to_string();
        out.insert(block_id, path);
    }
    Ok(out)
}

fn verify_cassette_block_integrity_v17(artifact_dir: &Path) -> Result<(), SdkError> {
    let index = read_cassette_block_index_v17(artifact_dir)?;
    if index.is_empty() {
        return Ok(());
    }
    let blocks = collect_block_files_v17(artifact_dir)?;
    for block_refs in index.values() {
        for block_id in block_refs {
            let Some(path) = blocks.get(block_id) else {
                return Err(SdkError::MissingProject(format!(
                    "X-CASSETTE-UPGRADE-TAMPER: missing block `{block_id}` referenced by index"
                )));
            };
            let bytes = fs::read(path)?;
            let actual = sha256_hex(&bytes);
            if actual != *block_id {
                return Err(SdkError::MissingProject(format!(
                    "X-CASSETTE-UPGRADE-TAMPER: block `{block_id}` hash mismatch (actual={actual})"
                )));
            }
        }
    }
    Ok(())
}

fn build_cassette_upgrade_plan_v17(
    artifact_dir: &Path,
    max_entries: usize,
    max_block_bytes: usize,
) -> Result<JsonValue, SdkError> {
    let entries =
        read_cassette_entries_for_upgrade_v17(artifact_dir, max_entries, max_block_bytes)?;
    let mut unique_blocks = BTreeSet::<String>::new();
    let mut total_entry_bytes = 0u64;
    for entry in &entries {
        unique_blocks.insert(entry.block_id.clone());
        total_entry_bytes = total_entry_bytes.saturating_add(entry.canonical_entry.len() as u64);
    }
    let v17_index_exists = artifact_dir
        .join("cassette")
        .join(CASSETTE_BLOCKS_INDEX_FILE_V17)
        .exists();
    let blocks_dir_exists = artifact_dir
        .join("cassette")
        .join(CASSETTE_BLOCKS_DIR_V17)
        .is_dir();
    if v17_index_exists || blocks_dir_exists {
        verify_cassette_block_integrity_v17(artifact_dir)?;
    }
    Ok(json!({
        "cassette_schema_version": CASSETTE_SCHEMA_VERSION_V17,
        "block_store_version": CASSETTE_BLOCK_STORE_VERSION_V17,
        "hasher_version": CASSETTE_HASHER_VERSION_V17,
        "requires_upgrade": !(v17_index_exists && blocks_dir_exists),
        "entries": entries.len(),
        "unique_blocks": unique_blocks.len(),
        "total_entry_bytes": total_entry_bytes,
        "artifact_dir": artifact_dir.to_string_lossy(),
    }))
}

fn build_cassette_v17_index_json(entries: &[CassetteEntryUpgradeV17]) -> JsonValue {
    let mut entry_refs = Vec::new();
    let mut block_ids = BTreeSet::<String>::new();
    for entry in entries {
        block_ids.insert(entry.block_id.clone());
        entry_refs.push(json!({
            "entry_id": entry.entry_id,
            "blocks": [entry.block_id],
        }));
    }
    json!({
        "cassette_schema_version": CASSETTE_SCHEMA_VERSION_V17,
        "block_store_version": CASSETTE_BLOCK_STORE_VERSION_V17,
        "hasher_version": CASSETTE_HASHER_VERSION_V17,
        "entry_refs": entry_refs,
        "block_ids": block_ids.into_iter().collect::<Vec<_>>(),
    })
}

fn infer_project_root_from_artifact_dir_v17(artifact_dir: &Path) -> PathBuf {
    let parent = artifact_dir.parent();
    let grand = parent.and_then(Path::parent);
    if let Some(artifacts_root) = parent {
        if artifacts_root
            .file_name()
            .map(|v| v.to_string_lossy() == ".ocl_artifacts")
            .unwrap_or(false)
        {
            if let Some(root) = grand {
                return root.to_path_buf();
            }
        }
    }
    PathBuf::from(".")
}

fn write_w17_cassette_report_v17(
    artifact_dir: &Path,
    file_name: &str,
    payload: &JsonValue,
) -> Result<PathBuf, SdkError> {
    let root = infer_project_root_from_artifact_dir_v17(artifact_dir);
    let report_path = root
        .join("target")
        .join("ocl")
        .join("w17")
        .join("cassette")
        .join(file_name);
    if let Some(parent) = report_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(
        &report_path,
        serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string()),
    )?;
    Ok(report_path)
}

fn apply_cassette_upgrade_v17(
    artifact_dir: &Path,
    max_entries: usize,
    max_block_bytes: usize,
) -> Result<(JsonValue, PathBuf), SdkError> {
    let entries =
        read_cassette_entries_for_upgrade_v17(artifact_dir, max_entries, max_block_bytes)?;
    let blocks_dir = artifact_dir.join("cassette").join(CASSETTE_BLOCKS_DIR_V17);
    fs::create_dir_all(&blocks_dir)?;

    let mut unique_block_map = BTreeMap::<String, String>::new();
    let mut total_block_bytes = 0u64;
    for entry in &entries {
        unique_block_map
            .entry(entry.block_id.clone())
            .or_insert_with(|| entry.canonical_entry.clone());
    }
    for (block_id, content) in &unique_block_map {
        let block_path = blocks_dir.join(format!("{block_id}.json"));
        fs::write(&block_path, content)?;
        total_block_bytes = total_block_bytes.saturating_add(content.len() as u64);
    }

    let index_json = build_cassette_v17_index_json(&entries);
    let index_path = artifact_dir
        .join("cassette")
        .join(CASSETTE_BLOCKS_INDEX_FILE_V17);
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_json).unwrap_or_else(|_| "{}".to_string()),
    )?;

    let meta_path = artifact_dir
        .join("cassette")
        .join(CASSETTE_STORAGE_META_FILE_V17);
    let meta_text = format!(
        concat!(
            "cassette_schema_version = {}\n",
            "block_store_version = {}\n",
            "hasher_version = \"{}\"\n",
            "entry_count = {}\n",
            "block_count = {}\n"
        ),
        CASSETTE_SCHEMA_VERSION_V17,
        CASSETTE_BLOCK_STORE_VERSION_V17,
        CASSETTE_HASHER_VERSION_V17,
        entries.len(),
        unique_block_map.len()
    );
    fs::write(&meta_path, meta_text)?;

    verify_cassette_block_integrity_v17(artifact_dir)?;

    let payload = json!({
        "cassette_schema_version": CASSETTE_SCHEMA_VERSION_V17,
        "block_store_version": CASSETTE_BLOCK_STORE_VERSION_V17,
        "hasher_version": CASSETTE_HASHER_VERSION_V17,
        "artifact_dir": artifact_dir.to_string_lossy(),
        "entries": entries.len(),
        "unique_blocks": unique_block_map.len(),
        "total_block_bytes": total_block_bytes,
        "index_path": index_path.to_string_lossy(),
        "meta_path": meta_path.to_string_lossy(),
    });
    let report_path =
        write_w17_cassette_report_v17(artifact_dir, "cassette_migration_report.json", &payload)?;
    Ok((payload, report_path))
}

fn build_cassette_prune_plan_v17(
    artifact_dir: &Path,
    ttl_days: Option<u32>,
) -> Result<CassettePrunePlanV17, SdkError> {
    let index = read_cassette_block_index_v17(artifact_dir)?;
    let blocks = collect_block_files_v17(artifact_dir)?;

    let mut referenced = BTreeSet::<String>::new();
    for refs in index.values() {
        for block_id in refs {
            referenced.insert(block_id.clone());
        }
    }

    let mut orphan_blocks = Vec::new();
    let mut missing_blocks = Vec::new();
    let mut prune_bytes = 0u64;

    for block_id in &referenced {
        if !blocks.contains_key(block_id) {
            missing_blocks.push(block_id.clone());
        }
    }
    for (block_id, path) in &blocks {
        if !referenced.contains(block_id) {
            orphan_blocks.push(block_id.clone());
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            prune_bytes = prune_bytes.saturating_add(size);
        }
    }

    orphan_blocks.sort();
    missing_blocks.sort();

    Ok(CassettePrunePlanV17 {
        total_blocks: blocks.len(),
        referenced_blocks: referenced.len(),
        orphan_blocks,
        missing_blocks,
        prune_bytes,
        ttl_days,
    })
}

fn render_cassette_prune_plan_json_v17(plan: &CassettePrunePlanV17) -> JsonValue {
    json!({
        "total_blocks": plan.total_blocks,
        "referenced_blocks": plan.referenced_blocks,
        "orphan_blocks": plan.orphan_blocks,
        "missing_blocks": plan.missing_blocks,
        "prune_bytes": plan.prune_bytes,
        "ttl_days": plan.ttl_days,
    })
}

fn apply_cassette_prune_plan_v17(
    artifact_dir: &Path,
    plan: &CassettePrunePlanV17,
) -> Result<usize, SdkError> {
    let blocks_dir = artifact_dir.join("cassette").join(CASSETTE_BLOCKS_DIR_V17);
    if !blocks_dir.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    for block_id in &plan.orphan_blocks {
        let path = blocks_dir.join(format!("{block_id}.json"));
        if path.exists() {
            fs::remove_file(path)?;
            removed = removed.saturating_add(1);
        }
    }
    Ok(removed)
}

fn detect_sensitive_marker_v17(raw: &str) -> Option<&'static str> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("authorization:") {
        return Some("authorization");
    }
    if lower.contains("bearer ") {
        return Some("bearer");
    }
    if lower.contains("api_key") || lower.contains("api-key") {
        return Some("api_key");
    }
    if lower.contains("password=") || lower.contains("\"password\"") {
        return Some("password");
    }
    if lower.contains("secret=") || lower.contains("\"secret\"") {
        return Some("secret");
    }
    None
}

fn build_cassette_stats_report_v17(artifact_dir: &Path) -> Result<(JsonValue, PathBuf), SdkError> {
    let cassette_dir = artifact_dir.join("cassette");
    let cassette_path = cassette_dir.join("cassette.jsonl");
    if !cassette_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-MISSING: missing cassette.jsonl at {}",
            cassette_path.display()
        )));
    }
    let cassette_raw = fs::read_to_string(&cassette_path)?;
    if let Some(marker) = detect_sensitive_marker_v17(&cassette_raw) {
        return Err(SdkError::PermissionDenied(format!(
            "X-CASSETTE-PRIVACY-LEAK: sensitive marker `{marker}` detected in {}",
            cassette_path.display()
        )));
    }

    let entries = cassette_raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as u64;
    let index = read_cassette_block_index_v17(artifact_dir)?;
    let blocks = collect_block_files_v17(artifact_dir)?;
    let mut referenced = BTreeSet::<String>::new();
    for refs in index.values() {
        for block_id in refs {
            referenced.insert(block_id.clone());
        }
    }
    let orphan_blocks = blocks.keys().filter(|id| !referenced.contains(*id)).count() as u64;
    let bytes_total = {
        let mut total = 0u64;
        for path in blocks.values() {
            total = total.saturating_add(fs::metadata(path).map(|m| m.len()).unwrap_or(0));
        }
        total
    };
    let payload = json!({
        "cassette_schema_version": CASSETTE_SCHEMA_VERSION_V17,
        "block_store_version": CASSETTE_BLOCK_STORE_VERSION_V17,
        "hasher_version": CASSETTE_HASHER_VERSION_V17,
        "artifact_dir": artifact_dir.to_string_lossy(),
        "entries": entries,
        "blocks": blocks.len() as u64,
        "referenced_blocks": referenced.len() as u64,
        "orphan_blocks": orphan_blocks,
        "bytes_total": bytes_total,
    });
    let report_path =
        write_w17_cassette_report_v17(artifact_dir, "cassette_operability_report.json", &payload)?;
    Ok((payload, report_path))
}

fn build_budget_analyze_report_v17(events: &[TraceViewEventV11]) -> JsonValue {
    let mut edge_map = BTreeMap::<(String, String), BudgetEdgeAggV17>::new();
    let mut budget_path = Vec::<JsonValue>::new();
    let mut observe_events = 0u32;
    for event in events {
        if event.base.event != "observe_end" {
            continue;
        }
        observe_events = observe_events.saturating_add(1);
        let caller = event
            .base
            .callsite_package_id
            .clone()
            .unwrap_or_else(|| "root".to_string());
        let key = event.base.key.clone().unwrap_or_else(|| "-".to_string());
        let kind = event.base.kind.clone().unwrap_or_else(|| "-".to_string());
        let reason = event.base.reason.clone().unwrap_or_else(|| "-".to_string());
        let edge_key = (caller.clone(), key.clone());
        let entry = edge_map
            .entry(edge_key)
            .or_insert_with(|| BudgetEdgeAggV17 {
                caller: caller.clone(),
                key: key.clone(),
                ..BudgetEdgeAggV17::default()
            });
        entry.observe_count = entry.observe_count.saturating_add(1);
        if kind == "insufficient" {
            entry.insufficient_count = entry.insufficient_count.saturating_add(1);
        }
        if kind == "deferred" {
            entry.deferred_count = entry.deferred_count.saturating_add(1);
        }
        let is_budget_pressure =
            reason.contains("BUDGET") || reason.contains("LIMIT") || reason.contains("CAP");
        if is_budget_pressure {
            entry.budget_pressure_count = entry.budget_pressure_count.saturating_add(1);
            budget_path.push(json!({
                "seq": event.base.seq,
                "call_id": event.call_id,
                "caller": caller,
                "key": key,
                "kind": kind,
                "reason": reason,
            }));
        }
    }

    let mut edges = edge_map.into_values().collect::<Vec<_>>();
    edges.sort_by(|a, b| {
        b.budget_pressure_count
            .cmp(&a.budget_pressure_count)
            .then_with(|| b.observe_count.cmp(&a.observe_count))
            .then_with(|| a.caller.cmp(&b.caller))
            .then_with(|| a.key.cmp(&b.key))
    });

    json!({
        "observe_events": observe_events,
        "budget_pressure_events": budget_path.len() as u64,
        "edge_count": edges.len() as u64,
        "edges": edges.into_iter().map(|edge| json!({
            "caller": edge.caller,
            "key": edge.key,
            "observe_count": edge.observe_count,
            "insufficient_count": edge.insufficient_count,
            "deferred_count": edge.deferred_count,
            "budget_pressure_count": edge.budget_pressure_count,
        })).collect::<Vec<_>>(),
        "budget_path": budget_path,
    })
}

fn build_budget_doctor_report_v17(analyze: &JsonValue) -> JsonValue {
    let mut issues = Vec::<JsonValue>::new();
    if let Some(edges) = analyze.get("edges").and_then(JsonValue::as_array) {
        for edge in edges {
            let pressure = edge
                .get("budget_pressure_count")
                .and_then(JsonValue::as_u64)
                .unwrap_or(0);
            if pressure == 0 {
                continue;
            }
            let caller = edge
                .get("caller")
                .and_then(JsonValue::as_str)
                .unwrap_or("root");
            let key = edge.get("key").and_then(JsonValue::as_str).unwrap_or("-");
            issues.push(json!({
                "caller": caller,
                "key": key,
                "budget_pressure_count": pressure,
                "suggestion": "increase edge budget cap or split call-path to isolate pressure",
            }));
        }
    }
    json!({
        "issue_count": issues.len(),
        "issues": issues,
    })
}

fn verify_dependency_override_guardrails_v10(root: &Path) -> Result<(), SdkError> {
    let manifest_path = root.join("Ocl.toml");
    if !manifest_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "missing manifest: {}",
            manifest_path.display()
        )));
    }
    let raw = fs::read_to_string(&manifest_path)?;
    let mut has_override_section = false;
    let mut in_security = false;
    let mut manifest_allows_overrides = false;

    for line_raw in raw.lines() {
        let line = line_raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let section = line.trim_start_matches('[').trim_end_matches(']');
            in_security = section == "security";
            if section.starts_with("permission_overrides.") {
                has_override_section = true;
            }
            continue;
        }
        if !in_security {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() != "allow_overrides" {
            continue;
        }
        manifest_allows_overrides = matches!(
            v.trim().trim_matches('"').to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        );
    }

    if !has_override_section {
        return Ok(());
    }
    if !manifest_allows_overrides {
        return Err(SdkError::PermissionDenied(
            "X-PERMISSION-OVERRIDE-DISABLED: permission_overrides declared but [security].allow_overrides=true is missing.".to_string(),
        ));
    }
    if std::env::var("OCL_ALLOW_OVERRIDES").unwrap_or_default() != "1" {
        return Err(SdkError::PermissionDenied(
            "V-PERMISSION-OVERRIDE-ENV-REQUIRED: set OCL_ALLOW_OVERRIDES=1 to allow permission_overrides in this run.".to_string(),
        ));
    }
    Ok(())
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
        "dep-permission" => apply_dep_permission_template_v10(root),
        _ => Err(SdkError::MissingProject(format!(
            "V-INIT-TEMPLATE-UNKNOWN: unsupported template `{template}` (expected `tool-cli`, `tool-http`, `tool-proc`, `tool-wallclock`, `mini-game`, `shadow-preview`, or `dep-permission`)"
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
            "allow = [\"std.shadow.*\"]\n",
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
        "observe(\"std.shadow.search\", \"tier2\", ctx(\"policy=portfolio;variants_json=[{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"},{\\\"score\\\":50,\\\"move\\\":\\\"hold\\\"}];max_branches=8;rounds=8;top_k=8;per_branch_step_cap=80;per_branch_budget_cap=5000;global_step_cap=640;global_budget_cap=40000;state_score_weight=1;outcome_weight=0;cost_budget_weight=0;cost_steps_weight=0;reason_penalty_weight=0;max_diff_keys=32;max_report_bytes=4096\"), budget(20)) -> search;\n",
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

fn apply_dep_permission_template_v10(root: &Path) -> Result<(), SdkError> {
    let project_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("dep_permission")
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
            "std = \"0.1.0\"\n",
            "widgets = {{ name=\"engine-ui-widgets\", version=\"0.3.0\", source=\"path\" }}\n\n",
            "[permissions.package]\n",
            "allow = [\"std.proc.*\"]\n",
            "deny = []\n\n",
            "[permissions.std_proc]\n",
            "enabled = true\n",
            "allow_bins = [\"python\"]\n",
            "timeout_ms = 3000\n",
            "max_stdout_bytes = 256\n",
            "max_stderr_bytes = 256\n"
        ),
        project_name
    );
    fs::write(root.join("Ocl.toml"), manifest)?;

    let source = concat!(
        "module app.dep_permission;\n\n",
        "import widgets.widget;\n",
        "let ready = true;\n",
        "condition(ready);\n"
    );
    fs::write(root.join("src").join("main.ocl"), source)?;

    let dep_dir = root.join("deps").join("widgets");
    fs::create_dir_all(dep_dir.join("src"))?;
    let dep_manifest = concat!(
        "[package]\n",
        "name = \"engine-ui-widgets\"\n",
        "version = \"0.3.0\"\n",
        "entry = \"src/widget.ocl\"\n\n",
        "[exports]\n",
        "modules = [\"widget\"]\n\n",
        "[requested_permissions]\n",
        "std_proc.enabled = true\n",
        "std_proc.allow_bins = [\"python\", \"node\"]\n",
        "std_proc.timeout_ms = 3000\n",
        "std_proc.max_stdout_bytes = 512\n",
        "std_proc.max_stderr_bytes = 512\n"
    );
    fs::write(dep_dir.join("package.oclp"), dep_manifest)?;
    fs::write(
        dep_dir.join("src").join("widget.ocl"),
        "module widgets.widget;\nlet stable = true;\ncondition(stable);\n",
    )?;

    let readme = concat!(
        "# dep-permission template\n\n",
        "- Muc tieu: demo dependency + permission gating theo package.\n",
        "- Thu nghiem nhanh:\n",
        "  - `ocl deps resolve .`\n",
        "  - `ocl deps verify .`\n",
        "  - `ocl pack build .`\n",
        "  - `ocl pack sign ./.oclpkg/<artifact>.oclpkg`\n"
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
struct QuarantinePolicyV15 {
    require_signed_cassette: bool,
    redact_headers: Vec<String>,
    redact_env_patterns: Vec<String>,
}

impl Default for QuarantinePolicyV15 {
    fn default() -> Self {
        Self {
            require_signed_cassette: false,
            redact_headers: vec!["authorization".to_string(), "cookie".to_string()],
            redact_env_patterns: vec!["*_TOKEN".to_string(), "*_KEY".to_string()],
        }
    }
}

fn parse_bool_literal_v15(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_quarantine_policy_v15(root: &Path) -> QuarantinePolicyV15 {
    let manifest_path = manifest_path_for_v071(root);
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return QuarantinePolicyV15::default();
    };

    let mut out = QuarantinePolicyV15::default();
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
        if section != "quarantine" {
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value_raw = v_raw.trim();
        match key {
            "require_signed_cassette" => {
                if let Some(value) = parse_bool_literal_v15(value_raw.trim_matches('"')) {
                    out.require_signed_cassette = value;
                }
            }
            "redact_headers" => {
                out.redact_headers = parse_string_array_literal_v08(value_raw)
                    .into_iter()
                    .map(|v| v.trim().to_ascii_lowercase())
                    .filter(|v| !v.is_empty())
                    .collect();
            }
            "redact_env_patterns" => {
                out.redact_env_patterns = parse_string_array_literal_v08(value_raw)
                    .into_iter()
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect();
            }
            _ => {}
        }
    }

    if out.redact_headers.is_empty() {
        out.redact_headers = vec!["authorization".to_string(), "cookie".to_string()];
    }
    if out.redact_env_patterns.is_empty() {
        out.redact_env_patterns = vec!["*_TOKEN".to_string(), "*_KEY".to_string()];
    }
    out
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
    require_signed_cassette: bool,
}

const CASSETTE_SCHEMA_VERSION_V08: &str = "v0.8";
const CASSETTE_HASHER_VERSION_V08: &str = "sha256-v1";
const CASSETTE_MODE_RECORD_V08: &str = "record";
const CASSETTE_MODE_REPLAY_V08: &str = "replay";
const CASSETTE_SIGN_SCHEMA_VERSION_V15: &str = "v0.15";
const CASSETTE_SIGN_HASHER_VERSION_V15: &str = "sha256-v1";
const CASSETTE_SIGN_DEFAULT_KEY_ID_V15: &str = "project-local-v15";
const CASSETTE_SIGN_SECRET_REL_PATH_V15: &str = ".ocl_signing/cassette_sign.key.v15";
const TRACE_SCHEMA_VERSION_V11: u64 = 2;
const CHECKPOINT_STATE_SCHEMA_VERSION_V11: &str = "v1";
const CHECKPOINT_DEFAULT_EVERY_V11: u64 = 200;
const CHECKPOINT_MAX_SELECTED_KEYS_V11: usize = 32;
const CHECKPOINT_MAX_KEY_BYTES_V11: usize = 128;
const DBG_LOCALS_MAX_KEYS_V11: usize = 16;
const DBG_PRINT_MAX_BYTES_V11: usize = 4096;
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
    let mut require_signed_cassette = false;

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
            "require_signed_cassette" if !value.is_empty() => {
                if let Some(parsed) = parse_bool_literal_v15(value) {
                    require_signed_cassette = parsed;
                }
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
        require_signed_cassette,
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

fn compute_redaction_policy_hash_v15(policy: &QuarantinePolicyV15) -> String {
    let mut headers = policy.redact_headers.clone();
    headers.sort();
    headers.dedup();
    let mut env_patterns = policy.redact_env_patterns.clone();
    env_patterns.sort();
    env_patterns.dedup();
    let canonical =
        format!(
        "version=v0.15\nrequire_signed_cassette={}\nredact_headers={}\nredact_env_patterns={}\n",
        if policy.require_signed_cassette { "true" } else { "false" },
        headers.join(","),
        env_patterns.join(",")
    );
    sha256_hex(canonical.as_bytes())
}

fn cassette_sign_secret_path_v15(project_root: &Path) -> PathBuf {
    project_root.join(CASSETTE_SIGN_SECRET_REL_PATH_V15)
}

fn cassette_signature_value_v15(
    key_id: &str,
    signer_pub: &str,
    signer_secret: &str,
    lane: &str,
    mode: &str,
    cassette_hash: &str,
) -> String {
    sha256_hex(
        format!(
            "cassette-sign-v15|key_id={key_id}|signer_pub={signer_pub}|lane={lane}|mode={mode}|cassette_hash={cassette_hash}|signer_secret={signer_secret}"
        )
        .as_bytes(),
    )
}

fn load_or_create_cassette_sign_secret_v15(project_root: &Path) -> Result<String, SdkError> {
    let key_path = cassette_sign_secret_path_v15(project_root);
    if key_path.exists() {
        let loaded = fs::read_to_string(&key_path)
            .map(|v| v.trim().to_string())
            .map_err(|err| {
                SdkError::MissingProject(format!(
                    "V-CASSETTE-SIGNATURE-INVALID: failed to read project signing key {}: {}",
                    key_path.display(),
                    err
                ))
            })?;
        if !loaded.is_empty() {
            return Ok(loaded);
        }
    }

    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-SIGNATURE-INVALID: failed to prepare key dir {}: {}",
                parent.display(),
                err
            ))
        })?;
    }

    let seed = format!(
        "cassette-sign-secret-v15|pid={}|unix_ms={}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let secret = sha256_hex(seed.as_bytes());
    fs::write(&key_path, format!("{secret}\n")).map_err(|err| {
        SdkError::MissingProject(format!(
            "V-CASSETTE-SIGNATURE-INVALID: failed to write project signing key {}: {}",
            key_path.display(),
            err
        ))
    })?;
    Ok(secret)
}

fn load_cassette_sign_secret_v15(project_root: &Path) -> Result<String, SdkError> {
    let key_path = cassette_sign_secret_path_v15(project_root);
    if !key_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-SIGNATURE-INVALID: missing project signing key {}",
            key_path.display()
        )));
    }
    let secret = fs::read_to_string(&key_path)
        .map(|v| v.trim().to_string())
        .map_err(|err| {
            SdkError::MissingProject(format!(
                "V-CASSETTE-SIGNATURE-INVALID: failed to read project signing key {}: {}",
                key_path.display(),
                err
            ))
        })?;
    if secret.is_empty() {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-SIGNATURE-INVALID: empty project signing key {}",
            key_path.display()
        )));
    }
    Ok(secret)
}

fn cassette_signer_pub_v15(key_id: &str, signer_secret: &str) -> String {
    sha256_hex(format!("cassette-signer-v15|{key_id}|{signer_secret}").as_bytes())
}

fn write_cassette_signature_v15(
    project_root: &Path,
    cassette_dir: &Path,
    key_id: &str,
    lane: &str,
    mode: &str,
    cassette_hash: &str,
) -> Result<(), SdkError> {
    let signer_secret = load_or_create_cassette_sign_secret_v15(project_root)?;
    let signer_pub = cassette_signer_pub_v15(key_id, &signer_secret);
    let signature = cassette_signature_value_v15(
        key_id,
        &signer_pub,
        &signer_secret,
        lane,
        mode,
        cassette_hash,
    );
    let sig_text = format!(
        concat!(
            "schema_version = \"{}\"\n",
            "hasher_version = \"{}\"\n",
            "key_id = \"{}\"\n",
            "lane = \"{}\"\n",
            "mode = \"{}\"\n",
            "cassette_hash = \"{}\"\n",
            "signer_pub = \"{}\"\n",
            "signature = \"{}\"\n"
        ),
        CASSETTE_SIGN_SCHEMA_VERSION_V15,
        CASSETTE_SIGN_HASHER_VERSION_V15,
        key_id,
        lane,
        mode,
        cassette_hash,
        signer_pub,
        signature
    );
    fs::write(cassette_dir.join("cassette.sig"), sig_text)?;
    Ok(())
}

fn parse_cassette_signature_v15(
    path: &Path,
) -> Result<
    (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ),
    SdkError,
> {
    let raw = fs::read_to_string(path)?;
    let mut schema_version = None::<String>;
    let mut hasher_version = None::<String>;
    let mut key_id = None::<String>;
    let mut lane = None::<String>;
    let mut mode = None::<String>;
    let mut cassette_hash = None::<String>;
    let mut signer_pub = None::<String>;
    let mut signature = None::<String>;
    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let Some((k_raw, v_raw)) = line.split_once('=') else {
            continue;
        };
        let key = k_raw.trim();
        let value = v_raw.trim().trim_matches('"').to_string();
        match key {
            "schema_version" => schema_version = Some(value),
            "hasher_version" => hasher_version = Some(value),
            "key_id" => key_id = Some(value),
            "lane" => lane = Some(value),
            "mode" => mode = Some(value),
            "cassette_hash" => cassette_hash = Some(value),
            "signer_pub" => signer_pub = Some(value),
            "signature" => signature = Some(value),
            _ => {}
        }
    }
    let schema_version = schema_version.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing schema_version".to_string())
    })?;
    let hasher_version = hasher_version.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing hasher_version".to_string())
    })?;
    let key_id = key_id.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing key_id".to_string())
    })?;
    let lane = lane.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing lane".to_string())
    })?;
    let mode = mode.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing mode".to_string())
    })?;
    let cassette_hash = cassette_hash.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing cassette_hash".to_string())
    })?;
    let signer_pub = signer_pub.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing signer_pub".to_string())
    })?;
    let signature = signature.ok_or_else(|| {
        SdkError::MissingProject("V-CASSETTE-SIGNATURE-INVALID: missing signature".to_string())
    })?;
    Ok((
        schema_version,
        hasher_version,
        key_id,
        lane,
        mode,
        cassette_hash,
        signer_pub,
        signature,
    ))
}

fn verify_cassette_signature_v15(
    project_root: &Path,
    cassette_dir: &Path,
    lane: &str,
    mode: &str,
    cassette_hash: &str,
) -> Result<(), SdkError> {
    let sig_path = cassette_dir.join("cassette.sig");
    if !sig_path.exists() {
        return Err(SdkError::MissingProject(
            "V-CASSETTE-SIGNATURE-MISSING: missing cassette.sig while signed cassette is required"
                .to_string(),
        ));
    }
    let (
        schema_version,
        hasher_version,
        key_id,
        file_lane,
        file_mode,
        file_hash,
        file_signer_pub,
        file_signature,
    ) = parse_cassette_signature_v15(&sig_path)?;
    if schema_version != CASSETTE_SIGN_SCHEMA_VERSION_V15 {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-SIGNATURE-INVALID: unsupported schema_version `{schema_version}`"
        )));
    }
    if hasher_version != CASSETTE_SIGN_HASHER_VERSION_V15 {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-SIGNATURE-INVALID: unsupported hasher_version `{hasher_version}`"
        )));
    }
    if file_lane != lane || file_mode != mode || file_hash != cassette_hash {
        return Err(SdkError::MissingProject(
            "V-CASSETTE-SIGNATURE-INVALID: cassette signature context mismatch".to_string(),
        ));
    }
    let signer_secret = load_cassette_sign_secret_v15(project_root)?;
    let signer_pub = cassette_signer_pub_v15(&key_id, &signer_secret);
    let expected_signature = cassette_signature_value_v15(
        &key_id,
        &signer_pub,
        &signer_secret,
        lane,
        mode,
        cassette_hash,
    );
    if signer_pub != file_signer_pub || expected_signature != file_signature {
        return Err(SdkError::MissingProject(
            "V-CASSETTE-SIGNATURE-INVALID: cassette signature verification failed".to_string(),
        ));
    }
    Ok(())
}

fn decode_headers_text_v15(raw_hex: &str) -> Option<String> {
    let bytes = hex_decode_v08_cli(raw_hex).ok()?;
    String::from_utf8(bytes).ok()
}

fn decode_hex_text_lossy_v15(raw_hex: &str) -> Option<String> {
    let bytes = hex_decode_v08_cli(raw_hex).ok()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

fn wildcard_match_v15(pattern: &str, value: &str) -> bool {
    let p = pattern.as_bytes();
    let v = value.as_bytes();
    let mut pi = 0usize;
    let mut vi = 0usize;
    let mut star = None::<usize>;
    let mut vi_after_star = 0usize;
    while vi < v.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == v[vi]) {
            pi += 1;
            vi += 1;
            continue;
        }
        if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            vi_after_star = vi;
            continue;
        }
        if let Some(star_idx) = star {
            pi = star_idx + 1;
            vi_after_star += 1;
            vi = vi_after_star;
            continue;
        }
        return false;
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

fn normalize_env_patterns_v15(policy: &QuarantinePolicyV15) -> Vec<String> {
    let mut patterns = policy
        .redact_env_patterns
        .iter()
        .map(|v| v.trim().to_ascii_uppercase())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    patterns.sort();
    patterns.dedup();
    patterns
}

fn collect_proc_env_redaction_issues_v15(
    proc_rows: &[ProcRecordV08],
    env_patterns: &[String],
) -> Vec<String> {
    if env_patterns.is_empty() {
        return Vec::new();
    }
    let mut issues = Vec::new();
    for row in proc_rows {
        for (stream, raw_hex) in [("stdout", &row.stdout_hex), ("stderr", &row.stderr_hex)] {
            let Some(text) = decode_hex_text_lossy_v15(raw_hex) else {
                continue;
            };
            for token in
                text.split(|ch: char| ch.is_whitespace() || [';', ',', '&', '|'].contains(&ch))
            {
                let trimmed = token.trim();
                let Some((k_raw, v_raw)) = trimmed.split_once('=') else {
                    continue;
                };
                let key = k_raw.trim().to_ascii_uppercase();
                let value = v_raw.trim();
                if key.is_empty() || value.is_empty() {
                    continue;
                }
                if value.eq_ignore_ascii_case("<redacted>") {
                    continue;
                }
                if env_patterns
                    .iter()
                    .any(|pattern| wildcard_match_v15(pattern, &key))
                {
                    issues.push(format!(
                        "call_id={} env_key={} stream={}",
                        row.call_id, key, stream
                    ));
                }
            }
        }
    }
    issues
}

fn detect_redaction_issues_v15(
    net_http_rows: &[NetHttpRecordV08],
    proc_rows: &[ProcRecordV08],
    policy: &QuarantinePolicyV15,
) -> Vec<String> {
    let mut headers = policy
        .redact_headers
        .iter()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    headers.sort();
    headers.dedup();
    if headers.is_empty() {
        return collect_proc_env_redaction_issues_v15(
            proc_rows,
            &normalize_env_patterns_v15(policy),
        );
    }
    let mut issues = Vec::new();
    for row in net_http_rows {
        let Some(headers_text) = decode_headers_text_v15(&row.headers_hex) else {
            continue;
        };
        for line in headers_text.lines() {
            let Some((name_raw, value_raw)) = line.split_once(':') else {
                continue;
            };
            let name = name_raw.trim().to_ascii_lowercase();
            if !headers.iter().any(|h| h == &name) {
                continue;
            }
            let value = value_raw.trim();
            if value.is_empty() {
                continue;
            }
            if value.eq_ignore_ascii_case("<redacted>") {
                continue;
            }
            issues.push(format!("call_id={} header={}", row.call_id, name));
        }
    }
    issues.extend(collect_proc_env_redaction_issues_v15(
        proc_rows,
        &normalize_env_patterns_v15(policy),
    ));
    issues.sort();
    issues.dedup();
    issues
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
    project_root: &Path,
    artifact_dir: &Path,
    lane: &str,
    mode: &str,
    policy: &QuarantinePolicyV15,
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
    let redaction_issues = detect_redaction_issues_v15(&net_http_rows, &proc_rows, policy);

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

    if !redaction_issues.is_empty()
        && (policy.require_signed_cassette
            || matches!(parse_lane_mode_v071(lane), Ok(LaneModeV071::LockedV071)))
    {
        return Err(SdkError::MissingProject(format!(
            "V-CASSETTE-REDACTION-REQUIRED: found {} sensitive rows without redaction",
            redaction_issues.len()
        )));
    }

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
    let redaction_policy_hash = compute_redaction_policy_hash_v15(policy);
    let cassette_meta = format!(
        concat!(
            "schema_version = \"{}\"\n",
            "lane = \"{}\"\n",
            "mode = \"{}\"\n",
            "hasher_version = \"{}\"\n",
            "created_at_unix_ms = {}\n",
            "redaction_policy_hash = \"{}\"\n",
            "redaction_warning_count = {}\n",
            "require_signed_cassette = {}\n"
        ),
        CASSETTE_SCHEMA_VERSION_V08,
        lane,
        mode,
        CASSETTE_HASHER_VERSION_V08,
        created_at_unix_ms,
        redaction_policy_hash,
        redaction_issues.len(),
        if policy.require_signed_cassette {
            "true"
        } else {
            "false"
        }
    );
    fs::write(&cassette_meta_path, cassette_meta)?;
    fs::write(cassette_hash_path, format!("{cassette_hash}\n"))?;
    if !redaction_issues.is_empty() {
        fs::write(
            cassette_dir.join("redaction_warning.log"),
            format!("{}\n", redaction_issues.join("\n")),
        )?;
    }
    if policy.require_signed_cassette {
        write_cassette_signature_v15(
            project_root,
            &cassette_dir,
            CASSETTE_SIGN_DEFAULT_KEY_ID_V15,
            lane,
            mode,
            &cassette_hash,
        )?;
    }
    Ok(cassette_hash)
}

fn validate_quarantine_cassette_bundle_v08(
    project_root: &Path,
    artifact_dir: &Path,
    lane: &str,
    mode: &str,
    expected_hash: &str,
    require_signed_cassette: bool,
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
    if require_signed_cassette {
        verify_cassette_signature_v15(project_root, &cassette_dir, lane, mode, expected_hash)?;
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
    let replay_policy = parse_quarantine_policy_v15(&spec.root);
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
            &spec.root,
            artifact_dir,
            &spec.lane,
            &spec.mode,
            expected_hash,
            spec.require_signed_cassette || replay_policy.require_signed_cassette,
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
    let bundle = build_replay_checkpoint_bundle_v11(&trace.events, CHECKPOINT_DEFAULT_EVERY_V11)?;
    if !bundle.rewind_match {
        return Err(SdkError::MissingProject(
            "X-DBG-CHECKPOINT-LOAD: rewind checkpoint validation mismatch".to_string(),
        ));
    }
    write_replay_checkpoints_v11(artifact_dir, &bundle)?;
    Ok(actual)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayCheckpointV11 {
    event_i: u64,
    replay_cursor: u64,
    state_digest: String,
    env_digest: String,
    selected_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayCheckpointBundleV11 {
    checkpoint_every: u64,
    event_count: u64,
    final_state_digest: String,
    final_env_digest: String,
    rewind_target_i: u64,
    rewind_direct_digest: String,
    rewind_restored_digest: String,
    rewind_match: bool,
    checkpoints: Vec<ReplayCheckpointV11>,
}

fn checkpoint_seed_state_digest_v11() -> String {
    sha256_hex(b"ocl.v11.checkpoint.state.seed")
}

fn checkpoint_seed_env_digest_v11() -> String {
    sha256_hex(b"ocl.v11.checkpoint.env.seed")
}

fn normalize_checkpoint_key_v11(raw: &str) -> String {
    if raw.len() <= CHECKPOINT_MAX_KEY_BYTES_V11 {
        return raw.to_string();
    }
    let mut end = CHECKPOINT_MAX_KEY_BYTES_V11;
    while !raw.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    raw[..end].to_string()
}

fn checkpoint_event_fingerprint_v11(event: &TraceEventV1) -> String {
    format!(
        "seq={}|event={}|key={}|kind={}|reason={}|origin={}|allowed={}|value={}|steps={}|universe={}|domain={}|payload={}",
        event.seq,
        event.event,
        event.key.as_deref().unwrap_or("-"),
        event.kind.as_deref().unwrap_or("-"),
        event.reason.as_deref().unwrap_or("-"),
        event
            .origin_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string()),
        event
            .allowed
            .map(|v| if v { "true" } else { "false" }.to_string())
            .unwrap_or_else(|| "-".to_string()),
        event
            .value
            .map(|v| if v { "true" } else { "false" }.to_string())
            .unwrap_or_else(|| "-".to_string()),
        event
            .steps
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string()),
        event.universe_id,
        event.domain_id,
        event.payload_hash
    )
}

fn advance_state_digest_v11(prev: &str, event: &TraceEventV1) -> String {
    let canonical = format!("{prev}\n{}", checkpoint_event_fingerprint_v11(event));
    sha256_hex(canonical.as_bytes())
}

fn advance_env_digest_v11(prev: &str, event: &TraceEventV1) -> String {
    let canonical = format!(
        "{prev}\nkey={}|kind={}|reason={}|origin={}",
        event.key.as_deref().unwrap_or("-"),
        event.kind.as_deref().unwrap_or("-"),
        event.reason.as_deref().unwrap_or("-"),
        event
            .origin_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    sha256_hex(canonical.as_bytes())
}

fn push_checkpoint_key_v11(selected_keys: &mut Vec<String>, key: Option<&str>) {
    let Some(raw_key) = key else {
        return;
    };
    let key_norm = normalize_checkpoint_key_v11(raw_key);
    if selected_keys.iter().any(|existing| existing == &key_norm) {
        return;
    }
    selected_keys.push(key_norm);
    if selected_keys.len() > CHECKPOINT_MAX_SELECTED_KEYS_V11 {
        let overflow = selected_keys.len() - CHECKPOINT_MAX_SELECTED_KEYS_V11;
        selected_keys.drain(0..overflow);
    }
}

fn compute_state_digest_until_v11(events: &[TraceEventV1], target_event_i: u64) -> String {
    let target = usize::min(target_event_i as usize, events.len());
    let mut digest = checkpoint_seed_state_digest_v11();
    for event in events.iter().take(target) {
        digest = advance_state_digest_v11(&digest, event);
    }
    digest
}

fn restore_state_digest_from_checkpoints_v11(
    events: &[TraceEventV1],
    checkpoints: &[ReplayCheckpointV11],
    target_event_i: u64,
) -> Result<String, SdkError> {
    let target = usize::min(target_event_i as usize, events.len());
    if target == 0 {
        return Ok(checkpoint_seed_state_digest_v11());
    }
    let mut digest = checkpoint_seed_state_digest_v11();
    let mut start = 0usize;
    if let Some(base) = checkpoints
        .iter()
        .filter(|cp| cp.event_i as usize <= target)
        .max_by_key(|cp| cp.event_i)
    {
        digest = base.state_digest.clone();
        start = base.event_i as usize;
    }
    for event in events.iter().skip(start).take(target.saturating_sub(start)) {
        digest = advance_state_digest_v11(&digest, event);
    }
    Ok(digest)
}

fn build_replay_checkpoint_bundle_v11(
    events: &[TraceEventV1],
    checkpoint_every: u64,
) -> Result<ReplayCheckpointBundleV11, SdkError> {
    let checkpoint_stride = if checkpoint_every == 0 {
        1
    } else {
        checkpoint_every
    };
    let mut checkpoints = Vec::<ReplayCheckpointV11>::new();
    let mut state_digest = checkpoint_seed_state_digest_v11();
    let mut env_digest = checkpoint_seed_env_digest_v11();
    let mut selected_keys = Vec::<String>::new();

    for (idx, event) in events.iter().enumerate() {
        state_digest = advance_state_digest_v11(&state_digest, event);
        env_digest = advance_env_digest_v11(&env_digest, event);
        push_checkpoint_key_v11(&mut selected_keys, event.key.as_deref());

        let event_i = (idx as u64).saturating_add(1);
        if event_i % checkpoint_stride == 0 || event_i == events.len() as u64 {
            checkpoints.push(ReplayCheckpointV11 {
                event_i,
                replay_cursor: event_i,
                state_digest: state_digest.clone(),
                env_digest: env_digest.clone(),
                selected_keys: selected_keys.clone(),
            });
        }
    }

    let target = if events.is_empty() {
        0
    } else {
        ((events.len() as u64).saturating_add(1)) / 2
    };
    let rewind_direct_digest = compute_state_digest_until_v11(events, target);
    let rewind_restored_digest =
        restore_state_digest_from_checkpoints_v11(events, &checkpoints, target)?;
    let rewind_match = rewind_direct_digest == rewind_restored_digest;

    Ok(ReplayCheckpointBundleV11 {
        checkpoint_every: checkpoint_stride,
        event_count: events.len() as u64,
        final_state_digest: state_digest,
        final_env_digest: env_digest,
        rewind_target_i: target,
        rewind_direct_digest,
        rewind_restored_digest,
        rewind_match,
        checkpoints,
    })
}

fn write_replay_checkpoints_v11(
    artifact_dir: &Path,
    bundle: &ReplayCheckpointBundleV11,
) -> Result<PathBuf, SdkError> {
    let checkpoints_dir = artifact_dir.join("checkpoints");
    fs::create_dir_all(&checkpoints_dir)?;
    let output_path = checkpoints_dir.join("replay.checkpoints.json");
    let checkpoints_json: Vec<JsonValue> = bundle
        .checkpoints
        .iter()
        .map(|cp| {
            json!({
                "event_i": cp.event_i,
                "replay_cursor": cp.replay_cursor,
                "state_digest": cp.state_digest,
                "env_digest": cp.env_digest,
                "selected_keys": cp.selected_keys,
            })
        })
        .collect();
    let rendered = serde_json::to_string_pretty(&json!({
        "trace_schema_version": TRACE_SCHEMA_VERSION_V11,
        "state_digest_schema_version": CHECKPOINT_STATE_SCHEMA_VERSION_V11,
        "checkpoint_every": bundle.checkpoint_every,
        "event_count": bundle.event_count,
        "caps": {
            "max_selected_keys": CHECKPOINT_MAX_SELECTED_KEYS_V11,
            "max_key_bytes": CHECKPOINT_MAX_KEY_BYTES_V11,
        },
        "final_state_digest": bundle.final_state_digest,
        "final_env_digest": bundle.final_env_digest,
        "rewind_validation": {
            "target_event_i": bundle.rewind_target_i,
            "direct_state_digest": bundle.rewind_direct_digest,
            "restored_state_digest": bundle.rewind_restored_digest,
            "rewind_match": bundle.rewind_match,
        },
        "checkpoints": checkpoints_json,
    }))
    .map_err(|err| SdkError::MissingProject(format!("V-CHECKPOINT-JSON-ENCODE: {err}")))?;
    fs::write(&output_path, rendered)?;
    Ok(output_path)
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceSpanV11 {
    module_id: String,
    start_byte: u32,
    end_byte: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceViewEventV11 {
    base: TraceEventV1,
    tick: u64,
    seed: u64,
    call_id: Option<u64>,
    span: Option<TraceSpanV11>,
}

#[derive(Debug, Clone, Default)]
struct TraceViewCliOptionsV11 {
    tail: Option<usize>,
    json: bool,
    filter_type: Option<String>,
    filter_key: Option<String>,
    filter_kind: Option<String>,
    filter_reason: Option<String>,
    filter_module: Option<String>,
    legacy_pipe: bool,
}

#[derive(Debug, Clone)]
enum DbgBreakpointV11 {
    EventType(String),
    Key(String),
    Kind(String),
    Reason(String),
    Loc {
        module_id: String,
        line: Option<u32>,
    },
}

#[derive(Debug, Clone)]
struct DbgCheckpointBundleViewV11 {
    checkpoint_every: u64,
    checkpoints: Vec<ReplayCheckpointV11>,
}

#[derive(Debug, Clone)]
struct DbgSessionV11 {
    events: Vec<TraceViewEventV11>,
    base_events: Vec<TraceEventV1>,
    cursor: usize,
    breakpoints: Vec<DbgBreakpointV11>,
    checkpoint_bundle: Option<DbgCheckpointBundleViewV11>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceDiffModeV11 {
    Strict,
    Align,
}

#[derive(Debug, Clone)]
struct TraceDiffOptionsV11 {
    mode: TraceDiffModeV11,
    json: bool,
    out_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct TraceDiffPairV11 {
    left_index: Option<usize>,
    right_index: Option<usize>,
}

#[derive(Debug, Clone)]
struct TraceDiffReportV11 {
    mode: TraceDiffModeV11,
    left_event_count: usize,
    right_event_count: usize,
    first_divergence_index: Option<usize>,
    changed_outcomes: usize,
    changed_key_calls: Vec<String>,
    added_events: usize,
    removed_events: usize,
    compared_pairs: usize,
    required_digest_left: String,
    required_digest_right: String,
}

#[derive(Debug, Clone)]
struct MinimizerCliOptionsV11 {
    goal_raw: String,
    key: Option<String>,
    against: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    json: bool,
}

#[derive(Debug, Clone)]
enum MinimizerGoalV11 {
    ErrorCode(String),
    Divergence { against: PathBuf },
    Kind { kind: String, key: Option<String> },
}

#[derive(Debug, Clone)]
struct MinimizerEventLineV11 {
    raw_line: String,
    key: Option<String>,
    kind: Option<String>,
    reason: Option<String>,
    error_code: Option<String>,
    call_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct MinimizerReportV11 {
    goal: String,
    source_artifact: PathBuf,
    output_artifact: PathBuf,
    event_count_before: usize,
    event_count_after: usize,
    reduction_percent: u32,
    ddmin_iterations: u32,
    cassette_entries_before: usize,
    cassette_entries_after: usize,
    fixture_entries_before: usize,
    fixture_entries_after: usize,
}

fn derive_call_id_for_trace_event_v11(event: &TraceEventV1) -> Option<u64> {
    match event.event.as_str() {
        "observe_start" | "observe_end" | "commit_attempt" | "commit_result" => Some(event.seq),
        "wallclock_observe" | "proc_observe" | "net_http_observe" => event.steps.map(u64::from),
        _ => None,
    }
}

fn encode_v071_trace_event_line(index: usize, event: &TraceEventV1) -> String {
    format!(
        concat!(
            "{{\"t\":\"TraceEvent\",\"i\":{},\"tick\":0,\"seed\":0,\"call_id\":{},\"span\":null,\"data\":{{",
            "\"seq\":{},\"run_id\":\"{}\",\"event\":\"{}\",",
            "\"key\":{},\"kind\":{},\"reason\":{},",
            "\"origin_id\":{},\"allowed\":{},\"value\":{},\"steps\":{},",
            "\"universe_id\":\"{}\",\"domain_id\":\"{}\",\"payload_hash\":\"{}\"",
            "}}}}"
        ),
        index + 1,
        json_opt_u64(derive_call_id_for_trace_event_v11(event)),
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

fn encode_v11_program_start_line(lane: &str) -> String {
    format!(
        concat!(
            "{{\"t\":\"ProgramStart\",\"i\":0,\"tick\":0,\"seed\":0,",
            "\"call_id\":null,\"span\":null,\"data\":{{",
            "\"trace_schema_version\":{},\"lane\":\"{}\"",
            "}}}}"
        ),
        TRACE_SCHEMA_VERSION_V11,
        json_escape(lane)
    )
}

fn encode_v071_lane_marker_line(lane: &str) -> String {
    format!(
        "{{\"t\":\"Lane\",\"i\":0,\"tick\":0,\"seed\":0,\"call_id\":null,\"span\":null,\"data\":{{\"lane\":\"{}\"}}}}",
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
            "{{\"t\":\"Error\",\"i\":1,\"tick\":0,\"seed\":0,\"call_id\":null,\"span\":null,\"data\":{{",
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

fn trace_format_error_v11(message: impl Into<String>) -> SdkError {
    SdkError::MissingProject(format!("X-TRACE-FORMAT-UNSUPPORTED: {}", message.into()))
}

fn json_obj_field<'a>(
    obj: &'a JsonMap<String, JsonValue>,
    key: &str,
) -> Result<&'a JsonMap<String, JsonValue>, SdkError> {
    match obj.get(key) {
        Some(JsonValue::Object(value)) => Ok(value),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be object"
        ))),
        None => Err(trace_format_error_v11(format!(
            "missing required object field `{key}`"
        ))),
    }
}

fn json_string_field(obj: &JsonMap<String, JsonValue>, key: &str) -> Result<String, SdkError> {
    match obj.get(key) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be string"
        ))),
        None => Err(trace_format_error_v11(format!(
            "missing required string field `{key}`"
        ))),
    }
}

fn json_opt_string_field(
    obj: &JsonMap<String, JsonValue>,
    key: &str,
) -> Result<Option<String>, SdkError> {
    match obj.get(key) {
        Some(JsonValue::Null) | None => Ok(None),
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be string|null"
        ))),
    }
}

fn json_opt_bool_field(
    obj: &JsonMap<String, JsonValue>,
    key: &str,
) -> Result<Option<bool>, SdkError> {
    match obj.get(key) {
        Some(JsonValue::Null) | None => Ok(None),
        Some(JsonValue::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be bool|null"
        ))),
    }
}

fn json_u64_field(obj: &JsonMap<String, JsonValue>, key: &str) -> Result<u64, SdkError> {
    match obj.get(key) {
        Some(JsonValue::Number(value)) => value.as_u64().ok_or_else(|| {
            trace_format_error_v11(format!("field `{key}` must be unsigned integer"))
        }),
        Some(JsonValue::String(value)) => value.parse::<u64>().map_err(|_| {
            trace_format_error_v11(format!("field `{key}` has invalid integer literal"))
        }),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be integer"
        ))),
        None => Err(trace_format_error_v11(format!(
            "missing required integer field `{key}`"
        ))),
    }
}

fn json_opt_u64_field(
    obj: &JsonMap<String, JsonValue>,
    key: &str,
) -> Result<Option<u64>, SdkError> {
    match obj.get(key) {
        Some(JsonValue::Null) | None => Ok(None),
        Some(JsonValue::Number(value)) => value.as_u64().map(Some).ok_or_else(|| {
            trace_format_error_v11(format!("field `{key}` must be unsigned integer|null"))
        }),
        Some(JsonValue::String(value)) => value.parse::<u64>().map(Some).map_err(|_| {
            trace_format_error_v11(format!("field `{key}` has invalid integer literal"))
        }),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` must be integer|null"
        ))),
    }
}

fn json_opt_u32_field(
    obj: &JsonMap<String, JsonValue>,
    key: &str,
) -> Result<Option<u32>, SdkError> {
    let parsed = json_opt_u64_field(obj, key)?;
    match parsed {
        Some(value) if value <= u32::MAX as u64 => Ok(Some(value as u32)),
        Some(_) => Err(trace_format_error_v11(format!(
            "field `{key}` exceeds u32 range"
        ))),
        None => Ok(None),
    }
}

fn parse_trace_event_v1_from_audit_data_v11(
    data: &JsonMap<String, JsonValue>,
) -> Result<TraceEventV1, SdkError> {
    Ok(TraceEventV1 {
        seq: json_u64_field(data, "seq")?,
        run_id: json_string_field(data, "run_id")?,
        event: json_string_field(data, "event")?,
        key: json_opt_string_field(data, "key")?,
        callsite_package_id: json_opt_string_field(data, "callsite_package_id")?,
        kind: json_opt_string_field(data, "kind")?,
        reason: json_opt_string_field(data, "reason")?,
        origin_id: json_opt_u64_field(data, "origin_id")?,
        allowed: json_opt_bool_field(data, "allowed")?,
        value: json_opt_bool_field(data, "value")?,
        steps: json_opt_u32_field(data, "steps")?,
        universe_id: json_string_field(data, "universe_id")?,
        domain_id: json_string_field(data, "domain_id")?,
        payload_hash: json_string_field(data, "payload_hash")?,
    })
}

fn parse_trace_error_event_v11(
    data: &JsonMap<String, JsonValue>,
    event_i: u64,
) -> Result<TraceEventV1, SdkError> {
    let code = json_string_field(data, "code")?;
    let phase = json_string_field(data, "phase")?;
    let message = json_string_field(data, "message")?;
    let root_reason = json_opt_string_field(data, "root_reason")?;
    let reason = root_reason.unwrap_or(code.clone());
    let payload_hash = sha256_hex(message.as_bytes());
    Ok(TraceEventV1 {
        seq: event_i,
        run_id: "replay.error".to_string(),
        event: "error".to_string(),
        key: None,
        callsite_package_id: None,
        kind: Some("error".to_string()),
        reason: Some(reason),
        origin_id: None,
        allowed: None,
        value: None,
        steps: None,
        universe_id: phase,
        domain_id: "error".to_string(),
        payload_hash,
    })
}

fn parse_trace_span_v11(value: Option<&JsonValue>) -> Result<Option<TraceSpanV11>, SdkError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let obj = raw
        .as_object()
        .ok_or_else(|| trace_format_error_v11("field `span` must be object|null"))?;
    let module_id = json_string_field(obj, "module_id")?;
    if module_id.starts_with('/')
        || module_id.contains("..")
        || module_id.contains(':')
        || module_id.starts_with('\\')
    {
        return Err(trace_format_error_v11(format!(
            "invalid span.module_id `{module_id}`"
        )));
    }
    let start_byte = match obj.get("start_byte").or_else(|| obj.get("start")) {
        Some(JsonValue::Number(v)) => v
            .as_u64()
            .ok_or_else(|| trace_format_error_v11("span.start_byte must be unsigned integer"))?,
        Some(JsonValue::String(v)) => v
            .parse::<u64>()
            .map_err(|_| trace_format_error_v11("span.start_byte has invalid integer literal"))?,
        _ => return Err(trace_format_error_v11("missing span.start_byte")),
    };
    let end_byte = match obj.get("end_byte").or_else(|| obj.get("end")) {
        Some(JsonValue::Number(v)) => v
            .as_u64()
            .ok_or_else(|| trace_format_error_v11("span.end_byte must be unsigned integer"))?,
        Some(JsonValue::String(v)) => v
            .parse::<u64>()
            .map_err(|_| trace_format_error_v11("span.end_byte has invalid integer literal"))?,
        _ => return Err(trace_format_error_v11("missing span.end_byte")),
    };
    if end_byte < start_byte {
        return Err(trace_format_error_v11(
            "span.end_byte must be >= span.start_byte",
        ));
    }
    if end_byte > u32::MAX as u64 {
        return Err(trace_format_error_v11("span.end_byte exceeds u32 range"));
    }
    Ok(Some(TraceSpanV11 {
        module_id: module_id.replace('\\', "/"),
        start_byte: start_byte as u32,
        end_byte: end_byte as u32,
    }))
}

fn trace_event_requires_call_id_v11(event_name: &str) -> bool {
    matches!(
        event_name,
        "observe_start"
            | "observe_end"
            | "commit_attempt"
            | "commit_result"
            | "wallclock_observe"
            | "proc_observe"
            | "net_http_observe"
    )
}

fn read_trace_events_from_audit_v11(path: &Path) -> Result<Vec<TraceViewEventV11>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut events = Vec::new();
    let mut saw_program_start = false;
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            trace_format_error_v11(format!(
                "invalid json line at {} in {}: {err}",
                idx + 1,
                path.display()
            ))
        })?;
        let obj = parsed.as_object().ok_or_else(|| {
            trace_format_error_v11(format!(
                "line {} in {} must be object",
                idx + 1,
                path.display()
            ))
        })?;
        let kind = json_string_field(obj, "t")?;
        if !saw_program_start {
            if kind != "ProgramStart" {
                return Err(trace_format_error_v11(format!(
                    "line 1 in {} must be `ProgramStart` with `trace_schema_version=2`",
                    path.display()
                )));
            }
            let data = json_obj_field(obj, "data")?;
            let version = json_u64_field(data, "trace_schema_version")?;
            if version != TRACE_SCHEMA_VERSION_V11 {
                return Err(trace_format_error_v11(format!(
                    "unsupported trace_schema_version `{version}` in {}",
                    path.display()
                )));
            }
            saw_program_start = true;
            continue;
        }
        match kind.as_str() {
            "TraceEvent" => {
                let tick = json_u64_field(obj, "tick")?;
                let seed = json_u64_field(obj, "seed")?;
                let call_id = json_opt_u64_field(obj, "call_id")?;
                let span = parse_trace_span_v11(obj.get("span"))?;
                let data = json_obj_field(obj, "data")?;
                let event = parse_trace_event_v1_from_audit_data_v11(data)?;
                if trace_event_requires_call_id_v11(&event.event) && call_id.is_none() {
                    return Err(trace_format_error_v11(format!(
                        "event `{}` requires non-null call_id in {}",
                        event.event,
                        path.display()
                    )));
                }
                events.push(TraceViewEventV11 {
                    base: event,
                    tick,
                    seed,
                    call_id,
                    span,
                });
            }
            "Error" => {
                let tick = json_u64_field(obj, "tick")?;
                let seed = json_u64_field(obj, "seed")?;
                let span = parse_trace_span_v11(obj.get("span"))?;
                let data = json_obj_field(obj, "data")?;
                let event = parse_trace_error_event_v11(data, json_u64_field(obj, "i")?)?;
                events.push(TraceViewEventV11 {
                    base: event,
                    tick,
                    seed,
                    call_id: None,
                    span,
                });
            }
            _ => {}
        }
    }
    if !saw_program_start {
        return Err(trace_format_error_v11(format!(
            "missing `ProgramStart` schema marker in {}",
            path.display()
        )));
    }
    Ok(events)
}

fn read_trace_events_for_view_v11(
    path: &Path,
    legacy_pipe: bool,
) -> Result<Vec<TraceViewEventV11>, SdkError> {
    if legacy_pipe {
        let legacy = read_trace_jsonl(path)?;
        let wrapped = legacy
            .into_iter()
            .map(|base| TraceViewEventV11 {
                base,
                tick: 0,
                seed: 0,
                call_id: None,
                span: None,
            })
            .collect();
        return Ok(wrapped);
    }
    let audit_path = if path.is_dir() {
        path.join("audit.jsonl")
    } else {
        path.to_path_buf()
    };
    if !audit_path.exists() {
        return Err(trace_format_error_v11(format!(
            "missing audit.jsonl at {}",
            audit_path.display()
        )));
    }
    read_trace_events_from_audit_v11(&audit_path)
}

fn trace_diff_mode_label_v11(mode: TraceDiffModeV11) -> &'static str {
    match mode {
        TraceDiffModeV11::Strict => "strict",
        TraceDiffModeV11::Align => "align",
    }
}

fn read_trace_events_for_diff_v11(path: &Path) -> Result<Vec<TraceViewEventV11>, SdkError> {
    read_trace_events_for_view_v11(path, false)
}

fn span_fingerprint_v11(span: Option<&TraceSpanV11>) -> String {
    match span {
        Some(value) => format!(
            "{}:{}:{}",
            value.module_id, value.start_byte, value.end_byte
        ),
        None => "-".to_string(),
    }
}

fn align_identity_for_event_v11(event: &TraceViewEventV11, index: usize) -> String {
    if trace_event_requires_call_id_v11(&event.base.event) {
        if let Some(call_id) = event.call_id {
            return format!(
                "call|{}|{}|{}",
                event.base.event,
                call_id,
                event.base.key.as_deref().unwrap_or("-")
            );
        }
    }
    if event.base.event.starts_with("stmt_") || event.base.event.starts_with("expr_") {
        return format!(
            "seq|{}|{}|{}",
            event.base.event,
            index,
            span_fingerprint_v11(event.span.as_ref())
        );
    }
    format!(
        "fallback|{}|{}|{}",
        event.base.event,
        index,
        span_fingerprint_v11(event.span.as_ref())
    )
}

fn build_trace_diff_pairs_strict_v11(
    left: &[TraceViewEventV11],
    right: &[TraceViewEventV11],
) -> Vec<TraceDiffPairV11> {
    let total = usize::max(left.len(), right.len());
    let mut pairs = Vec::with_capacity(total);
    for idx in 0..total {
        pairs.push(TraceDiffPairV11 {
            left_index: (idx < left.len()).then_some(idx),
            right_index: (idx < right.len()).then_some(idx),
        });
    }
    pairs
}

fn build_trace_diff_pairs_align_v11(
    left: &[TraceViewEventV11],
    right: &[TraceViewEventV11],
) -> Vec<TraceDiffPairV11> {
    let mut right_by_identity: BTreeMap<String, VecDeque<usize>> = BTreeMap::new();
    for (idx, event) in right.iter().enumerate() {
        let identity = align_identity_for_event_v11(event, idx);
        right_by_identity
            .entry(identity)
            .or_default()
            .push_back(idx);
    }

    let mut pairs = Vec::new();
    for (left_idx, left_event) in left.iter().enumerate() {
        let identity = align_identity_for_event_v11(left_event, left_idx);
        let right_idx = right_by_identity
            .get_mut(&identity)
            .and_then(VecDeque::pop_front);
        pairs.push(TraceDiffPairV11 {
            left_index: Some(left_idx),
            right_index: right_idx,
        });
    }

    let mut leftovers = Vec::new();
    for queue in right_by_identity.values() {
        for idx in queue {
            leftovers.push(*idx);
        }
    }
    leftovers.sort_unstable();
    for right_idx in leftovers {
        pairs.push(TraceDiffPairV11 {
            left_index: None,
            right_index: Some(right_idx),
        });
    }
    pairs
}

fn events_equal_for_diff_v11(left: &TraceViewEventV11, right: &TraceViewEventV11) -> bool {
    left.base.event == right.base.event
        && left.base.key == right.base.key
        && left.base.kind == right.base.kind
        && left.base.reason == right.base.reason
        && left.base.payload_hash == right.base.payload_hash
        && left.call_id == right.call_id
        && span_fingerprint_v11(left.span.as_ref()) == span_fingerprint_v11(right.span.as_ref())
}

fn build_trace_diff_report_v11(
    left: &[TraceViewEventV11],
    right: &[TraceViewEventV11],
    mode: TraceDiffModeV11,
) -> TraceDiffReportV11 {
    let pairs = match mode {
        TraceDiffModeV11::Strict => build_trace_diff_pairs_strict_v11(left, right),
        TraceDiffModeV11::Align => build_trace_diff_pairs_align_v11(left, right),
    };

    let mut first_divergence_index = None::<usize>;
    let mut changed_outcomes = 0usize;
    let mut changed_keys = BTreeSet::<String>::new();
    let mut added_events = 0usize;
    let mut removed_events = 0usize;
    let mut compared_pairs = 0usize;

    for pair in &pairs {
        match (pair.left_index, pair.right_index) {
            (Some(left_idx), Some(right_idx)) => {
                let left_event = &left[left_idx];
                let right_event = &right[right_idx];
                compared_pairs = compared_pairs.saturating_add(1);
                let mismatch = !events_equal_for_diff_v11(left_event, right_event);
                if mismatch && first_divergence_index.is_none() {
                    first_divergence_index = Some(usize::min(left_idx, right_idx));
                }
                if left_event.base.kind != right_event.base.kind
                    || left_event.base.reason != right_event.base.reason
                {
                    changed_outcomes = changed_outcomes.saturating_add(1);
                    if let Some(key) = left_event.base.key.as_ref() {
                        changed_keys.insert(key.clone());
                    } else if let Some(key) = right_event.base.key.as_ref() {
                        changed_keys.insert(key.clone());
                    }
                }
            }
            (Some(left_idx), None) => {
                removed_events = removed_events.saturating_add(1);
                if first_divergence_index.is_none() {
                    first_divergence_index = Some(left_idx);
                }
                if let Some(key) = left[left_idx].base.key.as_ref() {
                    changed_keys.insert(key.clone());
                }
            }
            (None, Some(right_idx)) => {
                added_events = added_events.saturating_add(1);
                if first_divergence_index.is_none() {
                    first_divergence_index = Some(right_idx);
                }
                if let Some(key) = right[right_idx].base.key.as_ref() {
                    changed_keys.insert(key.clone());
                }
            }
            (None, None) => {}
        }
    }

    let left_base: Vec<TraceEventV1> = left.iter().map(|event| event.base.clone()).collect();
    let right_base: Vec<TraceEventV1> = right.iter().map(|event| event.base.clone()).collect();
    TraceDiffReportV11 {
        mode,
        left_event_count: left.len(),
        right_event_count: right.len(),
        first_divergence_index,
        changed_outcomes,
        changed_key_calls: changed_keys.into_iter().collect(),
        added_events,
        removed_events,
        compared_pairs,
        required_digest_left: trace_required_digest(&left_base),
        required_digest_right: trace_required_digest(&right_base),
    }
}

fn render_trace_diff_text_v11(report: &TraceDiffReportV11) -> String {
    format!(
        concat!(
            "trace diff\n",
            "mode={}\n",
            "left_event_count={}\n",
            "right_event_count={}\n",
            "first_divergence_index={}\n",
            "changed_outcomes={}\n",
            "changed_key_calls={:?}\n",
            "added_events={}\n",
            "removed_events={}\n",
            "compared_pairs={}\n",
            "required_digest_left={}\n",
            "required_digest_right={}\n"
        ),
        trace_diff_mode_label_v11(report.mode),
        report.left_event_count,
        report.right_event_count,
        report
            .first_divergence_index
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        report.changed_outcomes,
        report.changed_key_calls,
        report.added_events,
        report.removed_events,
        report.compared_pairs,
        report.required_digest_left,
        report.required_digest_right
    )
}

fn render_trace_diff_json_v11(report: &TraceDiffReportV11) -> Result<String, SdkError> {
    serde_json::to_string_pretty(&json!({
        "trace_schema_version": TRACE_SCHEMA_VERSION_V11,
        "mode": trace_diff_mode_label_v11(report.mode),
        "left_event_count": report.left_event_count,
        "right_event_count": report.right_event_count,
        "first_divergence_index": report.first_divergence_index,
        "changed_outcomes": report.changed_outcomes,
        "changed_key_calls": report.changed_key_calls,
        "added_events": report.added_events,
        "removed_events": report.removed_events,
        "compared_pairs": report.compared_pairs,
        "required_digest_left": report.required_digest_left,
        "required_digest_right": report.required_digest_right,
    }))
    .map_err(|err| SdkError::MissingProject(format!("X-TRACE-DIFF-ENCODE: {err}")))
}

fn run_trace_diff_v11(
    left: &Path,
    right: &Path,
    options: &TraceDiffOptionsV11,
) -> Result<String, SdkError> {
    let left_events = read_trace_events_for_diff_v11(left)?;
    let right_events = read_trace_events_for_diff_v11(right)?;
    let report = build_trace_diff_report_v11(&left_events, &right_events, options.mode);
    let text_report = render_trace_diff_text_v11(&report);
    let json_report = render_trace_diff_json_v11(&report)?;

    let mut out_note = String::new();
    if let Some(dir) = options.out_dir.as_ref() {
        fs::create_dir_all(dir)?;
        let text_path = dir.join("diff_report.txt");
        let json_path = dir.join("diff_report.json");
        fs::write(&text_path, &text_report)?;
        fs::write(&json_path, &json_report)?;
        out_note.push_str(&format!(
            "report_text={}\nreport_json={}\n",
            text_path.display(),
            json_path.display()
        ));
    }

    if options.json {
        if out_note.is_empty() {
            Ok(json_report)
        } else {
            Ok(format!("{json_report}\n{out_note}"))
        }
    } else if out_note.is_empty() {
        Ok(text_report)
    } else {
        Ok(format!("{text_report}{out_note}"))
    }
}

fn parse_minimizer_goal_v11(
    options: &MinimizerCliOptionsV11,
) -> Result<MinimizerGoalV11, SdkError> {
    let raw = options.goal_raw.trim();
    if let Some(code) = raw.strip_prefix("error_code:") {
        let code = code.trim();
        if code.is_empty() {
            return Err(SdkError::MissingProject(
                "X-MINIMIZE-NO-SUCCESS: empty error code in --goal".to_string(),
            ));
        }
        return Ok(MinimizerGoalV11::ErrorCode(code.to_string()));
    }
    if raw == "divergence" {
        let Some(against) = options.against.as_ref() else {
            return Err(SdkError::MissingProject(
                "X-MINIMIZE-NO-SUCCESS: --goal divergence requires --against <artifactB>"
                    .to_string(),
            ));
        };
        return Ok(MinimizerGoalV11::Divergence {
            against: against.clone(),
        });
    }
    if let Some(kind) = raw.strip_prefix("kind:") {
        let kind = kind.trim();
        if kind.is_empty() {
            return Err(SdkError::MissingProject(
                "X-MINIMIZE-NO-SUCCESS: empty kind in --goal".to_string(),
            ));
        }
        return Ok(MinimizerGoalV11::Kind {
            kind: kind.to_string(),
            key: options.key.clone(),
        });
    }
    Err(SdkError::MissingProject(format!(
        "X-MINIMIZE-NO-SUCCESS: unsupported goal `{raw}`"
    )))
}

fn parse_minimizer_event_line_v11(
    raw_line: &str,
    parsed: &JsonMap<String, JsonValue>,
) -> Result<Option<MinimizerEventLineV11>, SdkError> {
    let line_type = json_string_field(parsed, "t")?;
    match line_type.as_str() {
        "ProgramStart" => Ok(None),
        "TraceEvent" => {
            let data = json_obj_field(parsed, "data")?;
            Ok(Some(MinimizerEventLineV11 {
                raw_line: raw_line.to_string(),
                key: json_opt_string_field(data, "key")?,
                kind: json_opt_string_field(data, "kind")?,
                reason: json_opt_string_field(data, "reason")?,
                error_code: None,
                call_id: json_opt_u64_field(parsed, "call_id")?,
            }))
        }
        "Error" => {
            let data = json_obj_field(parsed, "data")?;
            let code = json_string_field(data, "code")?;
            let reason = json_opt_string_field(data, "root_reason")?.or(Some(code.clone()));
            Ok(Some(MinimizerEventLineV11 {
                raw_line: raw_line.to_string(),
                key: None,
                kind: Some("error".to_string()),
                reason,
                error_code: Some(code),
                call_id: None,
            }))
        }
        _ => Ok(None),
    }
}

fn read_minimizer_audit_v11(
    artifact_dir: &Path,
) -> Result<(String, Vec<MinimizerEventLineV11>), SdkError> {
    let audit_path = artifact_dir.join("audit.jsonl");
    if !audit_path.exists() {
        return Err(SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: missing audit.jsonl at {}",
            audit_path.display()
        )));
    }
    let raw = fs::read_to_string(&audit_path)?;
    let mut program_start_line = None::<String>;
    let mut events = Vec::<MinimizerEventLineV11>::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            SdkError::MissingProject(format!(
                "X-MINIMIZE-NO-SUCCESS: invalid json line {} in {} ({err})",
                idx + 1,
                audit_path.display()
            ))
        })?;
        let obj = parsed.as_object().ok_or_else(|| {
            SdkError::MissingProject(format!(
                "X-MINIMIZE-NO-SUCCESS: line {} in {} must be object",
                idx + 1,
                audit_path.display()
            ))
        })?;
        let line_type = json_string_field(obj, "t")?;
        if line_type == "ProgramStart" {
            if program_start_line.is_none() {
                program_start_line = Some(trimmed.to_string());
            }
            continue;
        }
        if let Some(event) = parse_minimizer_event_line_v11(trimmed, obj)? {
            events.push(event);
        }
    }
    let Some(program_start) = program_start_line else {
        return Err(SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: missing ProgramStart in {}",
            audit_path.display()
        )));
    };
    Ok((program_start, events))
}

fn find_minimize_target_index_v11(
    source_artifact: &Path,
    events: &[MinimizerEventLineV11],
    goal: &MinimizerGoalV11,
) -> Result<usize, SdkError> {
    match goal {
        MinimizerGoalV11::ErrorCode(code) => events
            .iter()
            .position(|event| {
                event.error_code.as_deref() == Some(code.as_str())
                    || event.reason.as_deref() == Some(code.as_str())
            })
            .ok_or_else(|| {
                SdkError::MissingProject(format!(
                    "X-MINIMIZE-NO-SUCCESS: goal `error_code:{code}` not found in source trace"
                ))
            }),
        MinimizerGoalV11::Kind { kind, key } => events
            .iter()
            .position(|event| {
                event.kind.as_deref() == Some(kind.as_str())
                    && key
                        .as_ref()
                        .map(|want| event.key.as_deref() == Some(want.as_str()))
                        .unwrap_or(true)
            })
            .ok_or_else(|| {
                SdkError::MissingProject(format!(
                    "X-MINIMIZE-NO-SUCCESS: goal `kind:{kind}` not found in source trace"
                ))
            }),
        MinimizerGoalV11::Divergence { against } => {
            let left_events = read_trace_events_for_diff_v11(source_artifact)?;
            let right_events = read_trace_events_for_diff_v11(against)?;
            let report =
                build_trace_diff_report_v11(&left_events, &right_events, TraceDiffModeV11::Strict);
            report.first_divergence_index.ok_or_else(|| {
                SdkError::MissingProject(
                    "X-MINIMIZE-NO-SUCCESS: no divergence found between source and --against"
                        .to_string(),
                )
            })
        }
    }
}

#[derive(Debug, Clone)]
enum MinimizerGoalContextV11 {
    Simple,
    DivergenceAgainst { events: Vec<MinimizerEventLineV11> },
}

fn build_minimizer_goal_context_v11(
    goal: &MinimizerGoalV11,
) -> Result<MinimizerGoalContextV11, SdkError> {
    match goal {
        MinimizerGoalV11::Divergence { against } => {
            let (_, events) = read_minimizer_audit_v11(against)?;
            Ok(MinimizerGoalContextV11::DivergenceAgainst { events })
        }
        _ => Ok(MinimizerGoalContextV11::Simple),
    }
}

fn first_divergence_index_for_minimizer_events_v11(
    left: &[MinimizerEventLineV11],
    right: &[MinimizerEventLineV11],
) -> Option<usize> {
    let total = usize::max(left.len(), right.len());
    for idx in 0..total {
        match (left.get(idx), right.get(idx)) {
            (Some(a), Some(b)) => {
                if a.raw_line != b.raw_line {
                    return Some(idx);
                }
            }
            (None, Some(_)) | (Some(_), None) => {
                return Some(idx);
            }
            (None, None) => {}
        }
    }
    None
}

fn goal_holds_for_candidate_v11(
    goal: &MinimizerGoalV11,
    context: &MinimizerGoalContextV11,
    candidate: &[MinimizerEventLineV11],
) -> bool {
    if candidate.is_empty() {
        return false;
    }
    match goal {
        MinimizerGoalV11::ErrorCode(code) => candidate.iter().any(|event| {
            event.error_code.as_deref() == Some(code.as_str())
                || event.reason.as_deref() == Some(code.as_str())
        }),
        MinimizerGoalV11::Kind { kind, key } => candidate.iter().any(|event| {
            event.kind.as_deref() == Some(kind.as_str())
                && key
                    .as_ref()
                    .map(|want| event.key.as_deref() == Some(want.as_str()))
                    .unwrap_or(true)
        }),
        MinimizerGoalV11::Divergence { .. } => match context {
            MinimizerGoalContextV11::DivergenceAgainst { events } => {
                first_divergence_index_for_minimizer_events_v11(candidate, events).is_some()
            }
            MinimizerGoalContextV11::Simple => false,
        },
    }
}

fn ddmin_events_v11(
    initial: Vec<MinimizerEventLineV11>,
    goal: &MinimizerGoalV11,
    context: &MinimizerGoalContextV11,
) -> (Vec<MinimizerEventLineV11>, u32) {
    let mut current = initial;
    let mut iterations = 0u32;
    if current.len() < 2 {
        return (current, iterations);
    }

    let mut granularity = 2usize;
    while current.len() >= 2 {
        let len = current.len();
        let chunk_size = (len + granularity - 1) / granularity;
        let mut reduced = false;
        let mut start = 0usize;
        while start < len {
            let end = usize::min(start + chunk_size, len);
            let mut candidate = Vec::with_capacity(len - (end - start));
            candidate.extend_from_slice(&current[..start]);
            candidate.extend_from_slice(&current[end..]);
            iterations = iterations.saturating_add(1);
            if goal_holds_for_candidate_v11(goal, context, &candidate) {
                current = candidate;
                granularity = usize::max(2, granularity.saturating_sub(1));
                reduced = true;
                break;
            }
            start = end;
        }
        if !reduced {
            if granularity >= len {
                break;
            }
            granularity = usize::min(len, granularity * 2);
        }
    }

    (current, iterations)
}

fn read_fixture_manifest_entries_v11(
    source_artifact: &Path,
) -> Result<Vec<(String, String)>, SdkError> {
    let manifest_path = source_artifact.join("io").join("fixtures_manifest.json");
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&manifest_path)?;
    let parsed: JsonValue = serde_json::from_str(&raw).map_err(|err| {
        SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: invalid fixtures manifest {} ({err})",
            manifest_path.display()
        ))
    })?;
    let rows = parsed.as_array().ok_or_else(|| {
        SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: fixtures manifest must be array ({})",
            manifest_path.display()
        ))
    })?;

    let mut entries = Vec::<(String, String)>::new();
    for row in rows {
        let obj = row.as_object().ok_or_else(|| {
            SdkError::MissingProject(format!(
                "X-MINIMIZE-NO-SUCCESS: fixtures manifest row must be object ({})",
                manifest_path.display()
            ))
        })?;
        let path = json_string_field(obj, "path")?;
        let sha = json_string_field(obj, "sha256")?;
        entries.push((path.replace('\\', "/"), sha));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

fn normalize_fixture_reference_v11(value: &str) -> String {
    value.replace('\\', "/").trim().to_string()
}

fn fixture_entry_is_referenced_v11(path: &str, events: &[MinimizerEventLineV11]) -> bool {
    let normalized = normalize_fixture_reference_v11(path);
    let dot_path = format!("./{normalized}");
    let file_name = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    events.iter().any(|event| {
        event.raw_line.contains(&normalized)
            || event.raw_line.contains(&dot_path)
            || file_name
                .as_ref()
                .map(|name| event.raw_line.contains(name))
                .unwrap_or(false)
    })
}

fn write_fixture_manifest_entries_v11(
    output_artifact: &Path,
    entries: &[(String, String)],
) -> Result<(), SdkError> {
    let io_dir = output_artifact.join("io");
    fs::create_dir_all(&io_dir)?;
    let manifest_path = io_dir.join("fixtures_manifest.json");
    let json_rows: Vec<JsonValue> = entries
        .iter()
        .map(|(path, sha)| {
            json!({
                "path": path,
                "sha256": sha,
            })
        })
        .collect();
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&json_rows)
            .map_err(|err| SdkError::MissingProject(format!("X-MINIMIZE-NO-SUCCESS: {err}")))?,
    )?;
    Ok(())
}

fn copy_fixture_subset_for_minimizer_v11(
    source_artifact: &Path,
    output_artifact: &Path,
    entries: &[(String, String)],
) -> Result<(), SdkError> {
    for (rel_path, _) in entries {
        let src = source_artifact.join(rel_path.split('/').collect::<PathBuf>());
        if !src.exists() || !src.is_file() {
            continue;
        }
        let dst = output_artifact.join(rel_path.split('/').collect::<PathBuf>());
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn reduce_fixtures_for_minimizer_v11(
    source_artifact: &Path,
    output_artifact: &Path,
    selected_events: &[MinimizerEventLineV11],
) -> Result<(usize, usize), SdkError> {
    let entries = read_fixture_manifest_entries_v11(source_artifact)?;
    if entries.is_empty() {
        return Ok((0, 0));
    }
    let before = entries.len();
    let mut kept = Vec::<(String, String)>::new();
    for (path, sha) in &entries {
        if fixture_entry_is_referenced_v11(path, selected_events) {
            kept.push((path.clone(), sha.clone()));
        }
    }
    if kept.is_empty() {
        kept = entries.clone();
    }
    kept.sort_by(|a, b| a.0.cmp(&b.0));
    write_fixture_manifest_entries_v11(output_artifact, &kept)?;
    copy_fixture_subset_for_minimizer_v11(source_artifact, output_artifact, &kept)?;
    Ok((before, kept.len()))
}

fn copy_optional_file_v11(source: &Path, target: &Path) -> Result<(), SdkError> {
    if source.exists() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target)?;
    }
    Ok(())
}

fn reduce_cassette_for_minimizer_v11(
    source_artifact: &Path,
    output_artifact: &Path,
    kept_call_ids: &BTreeSet<u64>,
) -> Result<(usize, usize), SdkError> {
    let source_cassette = source_artifact.join("cassette");
    if !source_cassette.exists() {
        return Ok((0, 0));
    }
    let output_cassette = output_artifact.join("cassette");
    fs::create_dir_all(&output_cassette)?;

    let source_index_path = source_cassette.join("cassette_index.json");
    let source_jsonl_path = source_cassette.join("cassette.jsonl");
    let source_meta_path = source_cassette.join("cassette_meta.toml");
    let output_index_path = output_cassette.join("cassette_index.json");
    let output_jsonl_path = output_cassette.join("cassette.jsonl");
    let output_meta_path = output_cassette.join("cassette_meta.toml");
    copy_optional_file_v11(&source_meta_path, &output_meta_path)?;

    if !source_index_path.exists() || !source_jsonl_path.exists() {
        copy_optional_file_v11(&source_index_path, &output_index_path)?;
        copy_optional_file_v11(&source_jsonl_path, &output_jsonl_path)?;
        return Ok((0, 0));
    }

    let index_raw = fs::read_to_string(&source_index_path)?;
    let mut index_value: JsonValue = serde_json::from_str(&index_raw).map_err(|err| {
        SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: invalid cassette_index.json {} ({err})",
            source_index_path.display()
        ))
    })?;
    let index_obj = index_value.as_object_mut().ok_or_else(|| {
        SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: cassette_index.json must be object ({})",
            source_index_path.display()
        ))
    })?;
    let mapping_obj = index_obj
        .get("call_id_to_entry_id")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            SdkError::MissingProject(format!(
                "X-MINIMIZE-NO-SUCCESS: cassette_index.json missing call_id_to_entry_id ({})",
                source_index_path.display()
            ))
        })?;

    let mut kept_mapping = JsonMap::<String, JsonValue>::new();
    let mut kept_entry_ids = BTreeSet::<String>::new();
    for call_id in kept_call_ids {
        let key = call_id.to_string();
        if let Some(entry_id) = mapping_obj.get(&key).and_then(JsonValue::as_str) {
            kept_mapping.insert(key, JsonValue::String(entry_id.to_string()));
            kept_entry_ids.insert(entry_id.to_string());
        }
    }

    let jsonl_raw = fs::read_to_string(&source_jsonl_path)?;
    let mut before_entries = 0usize;
    let mut after_entries = 0usize;
    let mut kept_lines = Vec::<String>::new();
    for line in jsonl_raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        before_entries = before_entries.saturating_add(1);
        let parsed: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            SdkError::MissingProject(format!(
                "X-MINIMIZE-NO-SUCCESS: invalid cassette line in {} ({err})",
                source_jsonl_path.display()
            ))
        })?;
        let entry_id = parsed
            .as_object()
            .and_then(|obj| obj.get("id"))
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                SdkError::MissingProject(format!(
                    "X-MINIMIZE-NO-SUCCESS: cassette entry missing `id` in {}",
                    source_jsonl_path.display()
                ))
            })?;
        if kept_entry_ids.contains(entry_id) {
            after_entries = after_entries.saturating_add(1);
            kept_lines.push(trimmed.to_string());
        }
    }

    index_obj.insert(
        "call_id_to_entry_id".to_string(),
        JsonValue::Object(kept_mapping),
    );
    index_obj.insert(
        "entries".to_string(),
        JsonValue::Number(serde_json::Number::from(after_entries as u64)),
    );
    index_obj.insert(
        "next_call_id".to_string(),
        JsonValue::Number(serde_json::Number::from(after_entries as u64)),
    );

    fs::write(
        &output_index_path,
        serde_json::to_string_pretty(&index_value)
            .map_err(|err| SdkError::MissingProject(format!("X-MINIMIZE-NO-SUCCESS: {err}")))?,
    )?;
    let mut jsonl_out = String::new();
    for line in kept_lines {
        jsonl_out.push_str(&line);
        jsonl_out.push('\n');
    }
    fs::write(&output_jsonl_path, jsonl_out)?;
    Ok((before_entries, after_entries))
}

fn render_minimizer_report_v11(report: &MinimizerReportV11, json_mode: bool) -> String {
    if json_mode {
        return serde_json::to_string_pretty(&json!({
            "goal": report.goal,
            "source_artifact": report.source_artifact.to_string_lossy(),
            "output_artifact": report.output_artifact.to_string_lossy(),
            "event_count_before": report.event_count_before,
            "event_count_after": report.event_count_after,
            "reduction_percent": report.reduction_percent,
            "ddmin_iterations": report.ddmin_iterations,
            "cassette_entries_before": report.cassette_entries_before,
            "cassette_entries_after": report.cassette_entries_after,
            "fixture_entries_before": report.fixture_entries_before,
            "fixture_entries_after": report.fixture_entries_after,
        }))
        .unwrap_or_else(|_| "{}".to_string());
    }
    format!(
        concat!(
            "minimize ok\n",
            "goal={}\n",
            "source_artifact={}\n",
            "output_artifact={}\n",
            "event_count_before={}\n",
            "event_count_after={}\n",
            "reduction_percent={}\n",
            "ddmin_iterations={}\n",
            "cassette_entries_before={}\n",
            "cassette_entries_after={}\n",
            "fixture_entries_before={}\n",
            "fixture_entries_after={}\n"
        ),
        report.goal,
        report.source_artifact.display(),
        report.output_artifact.display(),
        report.event_count_before,
        report.event_count_after,
        report.reduction_percent,
        report.ddmin_iterations,
        report.cassette_entries_before,
        report.cassette_entries_after,
        report.fixture_entries_before,
        report.fixture_entries_after
    )
}

fn run_minimize_v11(
    artifact_dir: &Path,
    options: &MinimizerCliOptionsV11,
) -> Result<String, SdkError> {
    if !artifact_dir.is_dir() {
        return Err(SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: artifact dir not found: {}",
            artifact_dir.display()
        )));
    }
    let goal = parse_minimizer_goal_v11(options)?;
    let goal_context = build_minimizer_goal_context_v11(&goal)?;
    let (program_start, events) = read_minimizer_audit_v11(artifact_dir)?;
    if events.is_empty() {
        return Err(SdkError::MissingProject(
            "X-MINIMIZE-NO-SUCCESS: source audit has no minimizable events".to_string(),
        ));
    }
    let target_index = find_minimize_target_index_v11(artifact_dir, &events, &goal)?;
    if target_index >= events.len() {
        return Err(SdkError::MissingProject(
            "X-MINIMIZE-NO-SUCCESS: target index out of range".to_string(),
        ));
    }
    let selected_events = events[..=target_index].to_vec();
    let (selected_events, ddmin_iterations) =
        ddmin_events_v11(selected_events, &goal, &goal_context);
    let output_artifact = options
        .out_dir
        .clone()
        .unwrap_or_else(|| artifact_dir.join("minimized"));
    if output_artifact.exists() {
        return Err(SdkError::MissingProject(format!(
            "X-MINIMIZE-NO-SUCCESS: output dir already exists: {}",
            output_artifact.display()
        )));
    }
    fs::create_dir_all(&output_artifact)?;

    let output_audit = output_artifact.join("audit.jsonl");
    let mut audit_out = String::new();
    audit_out.push_str(&program_start);
    audit_out.push('\n');
    for event in &selected_events {
        audit_out.push_str(&event.raw_line);
        audit_out.push('\n');
    }
    fs::write(&output_audit, audit_out)?;

    let parsed = read_trace_events_from_audit_v11(&output_audit)?;
    let parsed_base: Vec<TraceEventV1> = parsed.into_iter().map(|event| event.base).collect();
    write_trace_index_v11(&output_artifact.join("trace_index.json"), &parsed_base)?;

    copy_optional_file_v11(
        &artifact_dir.join("replay.toml"),
        &output_artifact.join("replay.toml"),
    )?;
    copy_optional_file_v11(
        &artifact_dir.join("signature.txt"),
        &output_artifact.join("signature.txt"),
    )?;

    let kept_call_ids: BTreeSet<u64> = selected_events
        .iter()
        .filter_map(|event| event.call_id)
        .collect();
    let (cassette_before, cassette_after) =
        reduce_cassette_for_minimizer_v11(artifact_dir, &output_artifact, &kept_call_ids)?;
    let (fixture_before, fixture_after) =
        reduce_fixtures_for_minimizer_v11(artifact_dir, &output_artifact, &selected_events)?;

    let before = events.len();
    let after = selected_events.len();
    let reduction_percent = if before == 0 {
        0
    } else {
        (((before.saturating_sub(after)) * 100) / before) as u32
    };
    let goal_label = options.goal_raw.clone();
    let report = MinimizerReportV11 {
        goal: goal_label,
        source_artifact: artifact_dir.to_path_buf(),
        output_artifact: output_artifact.clone(),
        event_count_before: before,
        event_count_after: after,
        reduction_percent,
        ddmin_iterations,
        cassette_entries_before: cassette_before,
        cassette_entries_after: cassette_after,
        fixture_entries_before: fixture_before,
        fixture_entries_after: fixture_after,
    };
    let report_json = render_minimizer_report_v11(&report, true);
    fs::write(output_artifact.join("minimize_report.json"), report_json)?;
    Ok(render_minimizer_report_v11(&report, options.json))
}

fn compute_env_digest_until_v11(events: &[TraceEventV1], target_event_i: u64) -> String {
    let target = usize::min(target_event_i as usize, events.len());
    let mut digest = checkpoint_seed_env_digest_v11();
    for event in events.iter().take(target) {
        digest = advance_env_digest_v11(&digest, event);
    }
    digest
}

fn read_dbg_checkpoint_bundle_v11(
    artifact_dir: &Path,
) -> Result<Option<DbgCheckpointBundleViewV11>, SdkError> {
    let path = artifact_dir
        .join("checkpoints")
        .join("replay.checkpoints.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let parsed: JsonValue = serde_json::from_str(&raw).map_err(|err| {
        SdkError::MissingProject(format!(
            "X-DBG-CHECKPOINT-LOAD: invalid checkpoint json {} ({err})",
            path.display()
        ))
    })?;
    let root = parsed.as_object().ok_or_else(|| {
        SdkError::MissingProject(format!(
            "X-DBG-CHECKPOINT-LOAD: checkpoint file must be json object ({})",
            path.display()
        ))
    })?;
    let schema_version = json_u64_field(root, "trace_schema_version")?;
    if schema_version != TRACE_SCHEMA_VERSION_V11 {
        return Err(SdkError::MissingProject(format!(
            "X-DBG-CHECKPOINT-LOAD: unsupported trace schema `{schema_version}` in {}",
            path.display()
        )));
    }
    let checkpoint_every = json_u64_field(root, "checkpoint_every")?;
    let checkpoint_rows = root
        .get("checkpoints")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            SdkError::MissingProject(format!(
                "X-DBG-CHECKPOINT-LOAD: missing checkpoints array in {}",
                path.display()
            ))
        })?;
    let mut checkpoints = Vec::new();
    for row in checkpoint_rows {
        let item = row.as_object().ok_or_else(|| {
            SdkError::MissingProject(format!(
                "X-DBG-CHECKPOINT-LOAD: checkpoint row must be object in {}",
                path.display()
            ))
        })?;
        let event_i = json_u64_field(item, "event_i")?;
        let replay_cursor = json_u64_field(item, "replay_cursor")?;
        let state_digest = json_string_field(item, "state_digest")?;
        let env_digest = json_string_field(item, "env_digest")?;
        let selected_keys = match item.get("selected_keys") {
            Some(JsonValue::Array(values)) => {
                let mut out = Vec::new();
                for value in values {
                    let key = value.as_str().ok_or_else(|| {
                        SdkError::MissingProject(format!(
                            "X-DBG-CHECKPOINT-LOAD: selected_keys contains non-string in {}",
                            path.display()
                        ))
                    })?;
                    out.push(normalize_checkpoint_key_v11(key));
                }
                out
            }
            _ => {
                return Err(SdkError::MissingProject(format!(
                    "X-DBG-CHECKPOINT-LOAD: missing selected_keys array in {}",
                    path.display()
                )));
            }
        };
        checkpoints.push(ReplayCheckpointV11 {
            event_i,
            replay_cursor,
            state_digest,
            env_digest,
            selected_keys,
        });
    }
    checkpoints.sort_by(|a, b| a.event_i.cmp(&b.event_i));
    Ok(Some(DbgCheckpointBundleViewV11 {
        checkpoint_every,
        checkpoints,
    }))
}

fn nearest_checkpoint_for_event_v11<'a>(
    checkpoints: &'a [ReplayCheckpointV11],
    target_event_i: u64,
) -> Option<&'a ReplayCheckpointV11> {
    checkpoints
        .iter()
        .filter(|cp| cp.event_i <= target_event_i)
        .max_by_key(|cp| cp.event_i)
}

fn parse_dbg_index_arg_v11(raw: &str, max_len: usize) -> Result<usize, SdkError> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| SdkError::MissingProject(format!("X-DBG-COMMAND: invalid index `{raw}`")))?;
    if value >= max_len {
        return Err(SdkError::MissingProject(format!(
            "X-DBG-COMMAND: index `{value}` out of range (event_count={max_len})"
        )));
    }
    Ok(value)
}

fn parse_dbg_loc_breakpoint_v11(raw: &str) -> DbgBreakpointV11 {
    if let Some((module, line_raw)) = raw.rsplit_once(':') {
        if let Ok(line) = line_raw.parse::<u32>() {
            return DbgBreakpointV11::Loc {
                module_id: module.to_string(),
                line: Some(line),
            };
        }
    }
    DbgBreakpointV11::Loc {
        module_id: raw.to_string(),
        line: None,
    }
}

fn resolve_span_line_v11(span: &TraceSpanV11, sources_root: Option<&Path>) -> Option<u32> {
    let root = sources_root?;
    let rel = span.module_id.replace('\\', "/");
    if rel.contains("..") || rel.starts_with('/') || rel.contains(':') {
        return None;
    }
    let file_path = root.join(rel.split('/').collect::<PathBuf>());
    let bytes = fs::read(file_path).ok()?;
    let start = usize::min(span.start_byte as usize, bytes.len());
    let line = bytes[..start].iter().filter(|b| **b == b'\n').count() + 1;
    Some(line as u32)
}

fn dbg_breakpoint_matches_v11(
    breakpoint: &DbgBreakpointV11,
    event: &TraceViewEventV11,
    sources_root: Option<&Path>,
) -> bool {
    match breakpoint {
        DbgBreakpointV11::EventType(value) => event.base.event == *value,
        DbgBreakpointV11::Key(value) => event.base.key.as_deref() == Some(value.as_str()),
        DbgBreakpointV11::Kind(value) => event.base.kind.as_deref() == Some(value.as_str()),
        DbgBreakpointV11::Reason(value) => event.base.reason.as_deref() == Some(value.as_str()),
        DbgBreakpointV11::Loc { module_id, line } => {
            let Some(span) = event.span.as_ref() else {
                return false;
            };
            if span.module_id != *module_id {
                return false;
            }
            if let Some(want_line) = line {
                return resolve_span_line_v11(span, sources_root) == Some(*want_line);
            }
            true
        }
    }
}

fn format_dbg_where_line_v11(
    cursor: usize,
    event: &TraceViewEventV11,
    sources_root: Option<&Path>,
) -> String {
    let module = event
        .span
        .as_ref()
        .map(|value| value.module_id.as_str())
        .unwrap_or("-");
    let line = event
        .span
        .as_ref()
        .and_then(|span| resolve_span_line_v11(span, sources_root))
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "cursor={} seq={} event={} key={} kind={} reason={} module={} line={} call_id={} tick={} seed={}",
        cursor,
        event.base.seq,
        event.base.event,
        event.base.key.as_deref().unwrap_or("-"),
        event.base.kind.as_deref().unwrap_or("-"),
        event.base.reason.as_deref().unwrap_or("-"),
        module,
        line,
        event
            .call_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        event.tick,
        event.seed
    )
}

fn dbg_cap_text_v11(raw: &str) -> String {
    if raw.len() <= DBG_PRINT_MAX_BYTES_V11 {
        return raw.to_string();
    }
    let mut end = DBG_PRINT_MAX_BYTES_V11;
    while !raw.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    format!("{}...(truncated)", &raw[..end])
}

fn dbg_breakpoint_label_v11(breakpoint: &DbgBreakpointV11) -> String {
    match breakpoint {
        DbgBreakpointV11::EventType(value) => format!("type:{value}"),
        DbgBreakpointV11::Key(value) => format!("key:{value}"),
        DbgBreakpointV11::Kind(value) => format!("kind:{value}"),
        DbgBreakpointV11::Reason(value) => format!("reason:{value}"),
        DbgBreakpointV11::Loc { module_id, line } => match line {
            Some(value) => format!("loc:{module_id}:{value}"),
            None => format!("loc:{module_id}"),
        },
    }
}

fn dbg_find_last_focus_event_v11(session: &DbgSessionV11) -> Option<(usize, &TraceViewEventV11)> {
    if session.events.is_empty() {
        return None;
    }
    for idx in (0..=session.cursor).rev() {
        let event = &session.events[idx];
        let is_focus = event.base.event == "error"
            || event.base.kind.as_deref() == Some("error")
            || event.base.event == "observe_end"
            || event.base.event == "commit_result"
            || event.base.event.ends_with("_observe");
        if is_focus {
            return Some((idx, event));
        }
    }
    None
}

fn dbg_selected_keys_for_index_v11(
    bundle: Option<&DbgCheckpointBundleViewV11>,
    index: usize,
) -> Vec<String> {
    let Some(bundle) = bundle else {
        return Vec::new();
    };
    let Some(checkpoint) = nearest_checkpoint_for_event_v11(&bundle.checkpoints, index as u64 + 1)
    else {
        return Vec::new();
    };
    checkpoint
        .selected_keys
        .iter()
        .take(DBG_LOCALS_MAX_KEYS_V11)
        .cloned()
        .collect()
}

fn dbg_render_print_expr_v11(
    session: &DbgSessionV11,
    expr: &str,
    sources_root: Option<&Path>,
) -> String {
    let current = &session.events[session.cursor];
    match expr {
        "event" => {
            let rendered = serde_json::to_string(&json!({
                "cursor": session.cursor,
                "seq": current.base.seq,
                "event": current.base.event,
                "key": current.base.key,
                "kind": current.base.kind,
                "reason": current.base.reason,
                "call_id": current.call_id,
                "tick": current.tick,
                "seed": current.seed,
                "module": current.span.as_ref().map(|v| v.module_id.clone()),
            }))
            .unwrap_or_else(|_| "{\"error\":\"encode\"}".to_string());
            dbg_cap_text_v11(&rendered)
        }
        "event.key" => current
            .base
            .key
            .clone()
            .unwrap_or_else(|| "<null>".to_string()),
        "event.kind" => current
            .base
            .kind
            .clone()
            .unwrap_or_else(|| "<null>".to_string()),
        "event.reason" => current
            .base
            .reason
            .clone()
            .unwrap_or_else(|| "<null>".to_string()),
        "event.type" => current.base.event.clone(),
        "event.call_id" => current
            .call_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<null>".to_string()),
        "where" => format_dbg_where_line_v11(session.cursor, current, sources_root),
        "last" => {
            if let Some((idx, event)) = dbg_find_last_focus_event_v11(session) {
                format_dbg_where_line_v11(idx, event, sources_root)
            } else {
                "<none>".to_string()
            }
        }
        _ => "<unavailable>".to_string(),
    }
}

fn run_dbg_command_v11(
    session: &mut DbgSessionV11,
    command: &str,
    sources_root: Option<&Path>,
    out: &mut String,
) -> Result<bool, SdkError> {
    let trimmed = command.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(true);
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(true);
    }
    match tokens[0] {
        "step" | "next" => {
            if session.cursor + 1 < session.events.len() {
                session.cursor += 1;
            }
            out.push_str("step ");
            out.push_str(&format_dbg_where_line_v11(
                session.cursor,
                &session.events[session.cursor],
                sources_root,
            ));
            out.push('\n');
        }
        "back" => {
            if session.cursor > 0 {
                session.cursor -= 1;
            }
            out.push_str("back ");
            out.push_str(&format_dbg_where_line_v11(
                session.cursor,
                &session.events[session.cursor],
                sources_root,
            ));
            out.push('\n');
        }
        "jump" => {
            if tokens.len() < 2 {
                return Err(SdkError::MissingProject(
                    "X-DBG-COMMAND: `jump` requires index|first_error".to_string(),
                ));
            }
            if tokens[1] == "first_error" {
                if let Some(idx) = session.events.iter().position(|event| {
                    event.base.event == "error" || event.base.kind.as_deref() == Some("error")
                }) {
                    session.cursor = idx;
                    out.push_str("jump first_error ");
                    out.push_str(&format_dbg_where_line_v11(
                        session.cursor,
                        &session.events[session.cursor],
                        sources_root,
                    ));
                    out.push('\n');
                } else {
                    out.push_str("jump first_error <none>\n");
                }
            } else {
                let index = parse_dbg_index_arg_v11(tokens[1], session.events.len())?;
                session.cursor = index;
                out.push_str("jump ");
                out.push_str(&format_dbg_where_line_v11(
                    session.cursor,
                    &session.events[session.cursor],
                    sources_root,
                ));
                out.push('\n');
            }
        }
        "break" => {
            if tokens.len() < 4 || tokens[1] != "on" {
                return Err(SdkError::MissingProject(
                    "X-DBG-COMMAND: usage `break on type|key|kind|reason|loc <value>`".to_string(),
                ));
            }
            let value = tokens[3..].join(" ");
            let breakpoint = match tokens[2] {
                "type" => DbgBreakpointV11::EventType(value),
                "key" => DbgBreakpointV11::Key(value),
                "kind" => DbgBreakpointV11::Kind(value),
                "reason" => DbgBreakpointV11::Reason(value),
                "loc" => parse_dbg_loc_breakpoint_v11(&value),
                other => {
                    return Err(SdkError::MissingProject(format!(
                        "X-DBG-COMMAND: unsupported breakpoint selector `{other}`"
                    )));
                }
            };
            let label = dbg_breakpoint_label_v11(&breakpoint);
            session.breakpoints.push(breakpoint);
            out.push_str("breakpoint added ");
            out.push_str(&label);
            out.push('\n');
        }
        "continue" => {
            if session.breakpoints.is_empty() {
                out.push_str("continue no_breakpoints\n");
            } else {
                let mut hit: Option<(usize, String)> = None;
                for idx in session.cursor.saturating_add(1)..session.events.len() {
                    let event = &session.events[idx];
                    for breakpoint in &session.breakpoints {
                        if dbg_breakpoint_matches_v11(breakpoint, event, sources_root) {
                            hit = Some((idx, dbg_breakpoint_label_v11(breakpoint)));
                            break;
                        }
                    }
                    if hit.is_some() {
                        break;
                    }
                }
                if let Some((idx, label)) = hit {
                    session.cursor = idx;
                    out.push_str("continue hit ");
                    out.push_str(&label);
                    out.push(' ');
                    out.push_str(&format_dbg_where_line_v11(
                        session.cursor,
                        &session.events[session.cursor],
                        sources_root,
                    ));
                    out.push('\n');
                } else {
                    session.cursor = session.events.len().saturating_sub(1);
                    out.push_str("continue reached_end ");
                    out.push_str(&format_dbg_where_line_v11(
                        session.cursor,
                        &session.events[session.cursor],
                        sources_root,
                    ));
                    out.push('\n');
                }
            }
        }
        "where" => {
            out.push_str("where ");
            out.push_str(&format_dbg_where_line_v11(
                session.cursor,
                &session.events[session.cursor],
                sources_root,
            ));
            out.push('\n');
        }
        "locals" => {
            let target_event_i = session.cursor as u64 + 1;
            let checkpoint = session.checkpoint_bundle.as_ref().and_then(|bundle| {
                nearest_checkpoint_for_event_v11(&bundle.checkpoints, target_event_i)
            });
            if let Some(cp) = checkpoint {
                let keys: Vec<String> = cp
                    .selected_keys
                    .iter()
                    .take(DBG_LOCALS_MAX_KEYS_V11)
                    .cloned()
                    .collect();
                out.push_str(&format!(
                    "locals checkpoint_event_i={} env_digest={} selected_keys={:?}\n",
                    cp.event_i, cp.env_digest, keys
                ));
            } else {
                let env_digest = compute_env_digest_until_v11(&session.base_events, target_event_i);
                out.push_str(&format!(
                    "locals checkpoint_event_i=0 env_digest={} selected_keys=[]\n",
                    env_digest
                ));
            }
        }
        "last" => {
            if let Some((idx, event)) = dbg_find_last_focus_event_v11(session) {
                out.push_str("last ");
                out.push_str(&format_dbg_where_line_v11(idx, event, sources_root));
                out.push('\n');
            } else {
                out.push_str("last <none>\n");
            }
        }
        "print" => {
            let expr = trimmed
                .strip_prefix("print")
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SdkError::MissingProject(
                        "X-DBG-COMMAND: usage `print <expr>` (event|event.key|event.kind|event.reason|event.type|event.call_id|where|last)"
                            .to_string(),
                    )
                })?;
            let value = dbg_render_print_expr_v11(session, expr, sources_root);
            out.push_str("print ");
            out.push_str(expr);
            out.push_str(" => ");
            out.push_str(&dbg_cap_text_v11(&value));
            out.push('\n');
        }
        "diffenv" => {
            let (left_idx, right_idx) = match tokens.len() {
                1 => (session.cursor.saturating_sub(1), session.cursor),
                2 => (
                    parse_dbg_index_arg_v11(tokens[1], session.events.len())?,
                    session.cursor,
                ),
                3 => (
                    parse_dbg_index_arg_v11(tokens[1], session.events.len())?,
                    parse_dbg_index_arg_v11(tokens[2], session.events.len())?,
                ),
                _ => {
                    return Err(SdkError::MissingProject(
                        "X-DBG-COMMAND: usage `diffenv [left_idx] [right_idx]`".to_string(),
                    ));
                }
            };
            let left_digest =
                compute_env_digest_until_v11(&session.base_events, left_idx as u64 + 1);
            let right_digest =
                compute_env_digest_until_v11(&session.base_events, right_idx as u64 + 1);
            let left_keys: BTreeSet<String> =
                dbg_selected_keys_for_index_v11(session.checkpoint_bundle.as_ref(), left_idx)
                    .into_iter()
                    .collect();
            let right_keys: BTreeSet<String> =
                dbg_selected_keys_for_index_v11(session.checkpoint_bundle.as_ref(), right_idx)
                    .into_iter()
                    .collect();
            let added: Vec<String> = right_keys
                .difference(&left_keys)
                .cloned()
                .take(DBG_LOCALS_MAX_KEYS_V11)
                .collect();
            let removed: Vec<String> = left_keys
                .difference(&right_keys)
                .cloned()
                .take(DBG_LOCALS_MAX_KEYS_V11)
                .collect();
            out.push_str(&format!(
                "diffenv left={} right={} same={} left_digest={} right_digest={} added_keys={:?} removed_keys={:?}\n",
                left_idx,
                right_idx,
                if left_digest == right_digest { "true" } else { "false" },
                left_digest,
                right_digest,
                added,
                removed
            ));
        }
        "help" => {
            out.push_str("help commands=step,next,back,jump,break,continue,where,locals,last,print,diffenv,exit\n");
        }
        "exit" | "quit" => {
            out.push_str("exit\n");
            return Ok(false);
        }
        other => {
            return Err(SdkError::MissingProject(format!(
                "X-DBG-COMMAND: unsupported command `{other}`"
            )));
        }
    }
    Ok(true)
}

fn run_dbg_v11(artifact_dir: &Path, script_path: Option<&Path>) -> Result<String, SdkError> {
    if !artifact_dir.is_dir() {
        return Err(SdkError::MissingProject(format!(
            "X-DBG-COMMAND: artifact dir not found: {}",
            artifact_dir.display()
        )));
    }
    let events = read_trace_events_for_view_v11(artifact_dir, false)?;
    if events.is_empty() {
        return Err(SdkError::MissingProject(format!(
            "X-DBG-COMMAND: no replay events found in {}",
            artifact_dir.display()
        )));
    }
    let checkpoint_bundle = match read_dbg_checkpoint_bundle_v11(artifact_dir)? {
        Some(bundle) => Some(bundle),
        None => {
            let base_events: Vec<TraceEventV1> =
                events.iter().map(|event| event.base.clone()).collect();
            let computed =
                build_replay_checkpoint_bundle_v11(&base_events, CHECKPOINT_DEFAULT_EVERY_V11)?;
            Some(DbgCheckpointBundleViewV11 {
                checkpoint_every: computed.checkpoint_every,
                checkpoints: computed.checkpoints,
            })
        }
    };
    let mut session = DbgSessionV11 {
        base_events: events.iter().map(|event| event.base.clone()).collect(),
        events,
        cursor: 0,
        breakpoints: Vec::new(),
        checkpoint_bundle,
    };

    let mut out = String::new();
    out.push_str("dbg v11\n");
    out.push_str(&format!("artifact={}\n", artifact_dir.display()));
    out.push_str(&format!("event_count={}\n", session.events.len()));
    if let Some(bundle) = session.checkpoint_bundle.as_ref() {
        out.push_str(&format!(
            "checkpoint_every={} checkpoint_count={}\n",
            bundle.checkpoint_every,
            bundle.checkpoints.len()
        ));
    } else {
        out.push_str("checkpoint_every=- checkpoint_count=0\n");
    }

    let sources_root = resolve_sources_root_for_trace_view_v11(artifact_dir);
    out.push_str("where ");
    out.push_str(&format_dbg_where_line_v11(
        session.cursor,
        &session.events[session.cursor],
        sources_root.as_deref(),
    ));
    out.push('\n');

    let commands: Vec<String> = if let Some(path) = script_path {
        let raw = fs::read_to_string(path).map_err(|err| {
            SdkError::MissingProject(format!(
                "X-DBG-COMMAND: cannot read script {} ({err})",
                path.display()
            ))
        })?;
        raw.lines().map(|line| line.to_string()).collect()
    } else {
        let mut stdin = String::new();
        io::stdin().read_to_string(&mut stdin).map_err(|err| {
            SdkError::MissingProject(format!("X-DBG-COMMAND: stdin read error ({err})"))
        })?;
        if stdin.trim().is_empty() {
            out.push_str("help commands=step,next,back,jump,break,continue,where,locals,last,print,diffenv,exit\n");
            return Ok(out);
        }
        stdin.lines().map(|line| line.to_string()).collect()
    };

    for (idx, command) in commands.iter().enumerate() {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        out.push_str(&format!("cmd[{}]={}\n", idx + 1, trimmed));
        let keep_running =
            run_dbg_command_v11(&mut session, trimmed, sources_root.as_deref(), &mut out)?;
        if !keep_running {
            break;
        }
    }
    Ok(out)
}

fn event_matches_filters_v11(event: &TraceViewEventV11, options: &TraceViewCliOptionsV11) -> bool {
    if let Some(want) = options.filter_type.as_deref() {
        if event.base.event != want {
            return false;
        }
    }
    if let Some(want) = options.filter_key.as_deref() {
        if event.base.key.as_deref() != Some(want) {
            return false;
        }
    }
    if let Some(want) = options.filter_kind.as_deref() {
        if event.base.kind.as_deref() != Some(want) {
            return false;
        }
    }
    if let Some(want) = options.filter_reason.as_deref() {
        if event.base.reason.as_deref() != Some(want) {
            return false;
        }
    }
    if let Some(want) = options.filter_module.as_deref() {
        if event.span.as_ref().map(|s| s.module_id.as_str()) != Some(want) {
            return false;
        }
    }
    true
}

fn resolve_sources_root_for_trace_view_v11(input: &Path) -> Option<PathBuf> {
    let root = if input.is_dir() {
        input.to_path_buf()
    } else {
        input.parent()?.to_path_buf()
    };
    let sources = root.join("sources");
    if sources.exists() {
        Some(sources)
    } else {
        None
    }
}

fn build_source_snippet_v11(span: &TraceSpanV11, sources_root: &Path) -> Option<String> {
    let rel = span.module_id.replace('\\', "/");
    if rel.contains("..") || rel.starts_with('/') || rel.contains(':') {
        return None;
    }
    let file_path = sources_root.join(rel.split('/').collect::<PathBuf>());
    let bytes = fs::read(file_path).ok()?;
    let start = usize::min(span.start_byte as usize, bytes.len());
    let mut end = usize::min(span.end_byte as usize, bytes.len());
    if end <= start {
        end = usize::min(start.saturating_add(96), bytes.len());
    }
    let snippet = String::from_utf8_lossy(&bytes[start..end])
        .replace('\r', "")
        .replace('\n', "\\n");
    if snippet.is_empty() {
        None
    } else {
        Some(snippet)
    }
}

fn render_trace_view_v11(
    input: &Path,
    events: &[TraceViewEventV11],
    options: &TraceViewCliOptionsV11,
) -> String {
    let mut filtered: Vec<&TraceViewEventV11> = events
        .iter()
        .filter(|event| event_matches_filters_v11(event, options))
        .collect();
    if let Some(tail) = options.tail {
        let start = filtered.len().saturating_sub(tail);
        filtered = filtered[start..].to_vec();
    }
    let selected_base: Vec<TraceEventV1> = filtered.iter().map(|e| e.base.clone()).collect();
    let digest = trace_required_digest(&selected_base);
    let sources_root = resolve_sources_root_for_trace_view_v11(input);

    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    for event in &filtered {
        *by_type.entry(event.base.event.clone()).or_insert(0) += 1;
        if let Some(kind) = event.base.kind.as_ref() {
            *by_kind.entry(kind.clone()).or_insert(0) += 1;
        }
        if let Some(reason) = event.base.reason.as_ref() {
            *by_reason.entry(reason.clone()).or_insert(0) += 1;
        }
    }

    if options.json {
        let events_json: Vec<JsonValue> = filtered
            .iter()
            .map(|event| {
                let snippet = event
                    .span
                    .as_ref()
                    .and_then(|span| sources_root.as_ref().and_then(|root| build_source_snippet_v11(span, root)));
                json!({
                    "seq": event.base.seq,
                    "event": event.base.event,
                    "key": event.base.key,
                    "kind": event.base.kind,
                    "reason": event.base.reason,
                    "module": event.span.as_ref().map(|s| s.module_id.clone()),
                    "span": event.span.as_ref().map(|s| json!({"start_byte": s.start_byte, "end_byte": s.end_byte})),
                    "call_id": event.call_id,
                    "tick": event.tick,
                    "seed": event.seed,
                    "snippet": snippet,
                })
            })
            .collect();
        return serde_json::to_string_pretty(&json!({
            "trace_schema_version": TRACE_SCHEMA_VERSION_V11,
            "event_count": filtered.len(),
            "required_digest": digest,
            "summary": {
                "by_type": by_type,
                "by_kind": by_kind,
                "by_reason": by_reason,
            },
            "events": events_json
        }))
        .unwrap_or_else(|_| "{}".to_string());
    }

    let mut out = String::new();
    out.push_str("trace view\n");
    out.push_str("event_count=");
    out.push_str(&filtered.len().to_string());
    out.push('\n');
    out.push_str("required_digest=");
    out.push_str(&digest);
    out.push('\n');
    out.push_str("summary.by_type=");
    out.push_str(&format!("{:?}", by_type));
    out.push('\n');
    out.push_str("summary.by_kind=");
    out.push_str(&format!("{:?}", by_kind));
    out.push('\n');
    out.push_str("summary.by_reason=");
    out.push_str(&format!("{:?}", by_reason));
    out.push('\n');

    for event in filtered {
        let module = event
            .span
            .as_ref()
            .map(|s| s.module_id.as_str())
            .unwrap_or("-");
        let snippet = event
            .span
            .as_ref()
            .and_then(|span| {
                sources_root
                    .as_ref()
                    .and_then(|root| build_source_snippet_v11(span, root))
            })
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "#{} {} key={} kind={} reason={} module={} call_id={} tick={} seed={} snippet={}\n",
            event.base.seq,
            event.base.event,
            event.base.key.as_deref().unwrap_or("-"),
            event.base.kind.as_deref().unwrap_or("-"),
            event.base.reason.as_deref().unwrap_or("-"),
            module,
            event
                .call_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            event.tick,
            event.seed,
            snippet
        ));
    }
    out
}

fn build_trace_index_json_v11(events: &[TraceEventV1]) -> String {
    let mut by_type: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut by_key: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut by_reason: BTreeMap<String, Vec<u64>> = BTreeMap::new();

    for event in events {
        by_type
            .entry(event.event.clone())
            .or_default()
            .push(event.seq);
        if let Some(key) = event.key.as_ref() {
            by_key.entry(key.clone()).or_default().push(event.seq);
        }
        if let Some(reason) = event.reason.as_ref() {
            by_reason.entry(reason.clone()).or_default().push(event.seq);
        }
    }

    let pack_bucket = |bucket: BTreeMap<String, Vec<u64>>| -> JsonValue {
        let mut out = JsonMap::new();
        for (name, indexes) in bucket {
            let json_indexes: Vec<JsonValue> = indexes.into_iter().map(JsonValue::from).collect();
            out.insert(
                name,
                json!({
                    "count": json_indexes.len(),
                    "event_indexes": json_indexes
                }),
            );
        }
        JsonValue::Object(out)
    };

    serde_json::to_string_pretty(&json!({
        "trace_schema_version": TRACE_SCHEMA_VERSION_V11,
        "event_count": events.len(),
        "by_type": pack_bucket(by_type),
        "by_key": pack_bucket(by_key),
        "by_reason": pack_bucket(by_reason),
    }))
    .unwrap_or_else(|_| "{\"trace_schema_version\":2}".to_string())
}

fn write_trace_index_v11(path: &Path, events: &[TraceEventV1]) -> Result<(), SdkError> {
    fs::write(path, build_trace_index_json_v11(events))?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CachePerfCountersV13 {
    compile_hits: u32,
    compile_misses: u32,
    exec_hits: u32,
    exec_misses: u32,
    observe_hits: u32,
    observe_misses: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CacheStatsReportV13 {
    artifact_dirs: u32,
    perf_files: u32,
    compile_cache_entries: u32,
    compile_cache_bytes: u64,
    counters: CachePerfCountersV13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CacheCleanSummaryV13 {
    removed_artifacts: bool,
    removed_local_cache: bool,
    removed_target_cache: bool,
}

fn cache_perf_from_trace_summary_v13(summary: &TraceRunSummary) -> CachePerfCountersV13 {
    CachePerfCountersV13 {
        compile_hits: 0,
        compile_misses: 0,
        exec_hits: summary.exec_cache.hits,
        exec_misses: summary.exec_cache.misses,
        observe_hits: summary.observe_cache.hits,
        observe_misses: summary.observe_cache.misses,
    }
}

fn write_perf_cache_jsonl_v13(
    path: &Path,
    counters: &CachePerfCountersV13,
) -> Result<(), SdkError> {
    let mut out = String::new();
    let rows = [
        ("CacheCompileHit", counters.compile_hits),
        ("CacheCompileMiss", counters.compile_misses),
        ("CacheExecHit", counters.exec_hits),
        ("CacheExecMiss", counters.exec_misses),
        ("CacheObserveHit", counters.observe_hits),
        ("CacheObserveMiss", counters.observe_misses),
    ];
    for (idx, (kind, count)) in rows.iter().enumerate() {
        let line = json!({
            "i": idx as u64,
            "t": kind,
            "seed": 0u64,
            "tick": 0u64,
            "call_id": null,
            "span": null,
            "data": {
                "count": *count,
            }
        });
        out.push_str(&line.to_string());
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(())
}

fn merge_perf_counters_v13(into: &mut CachePerfCountersV13, other: CachePerfCountersV13) {
    into.compile_hits = into.compile_hits.saturating_add(other.compile_hits);
    into.compile_misses = into.compile_misses.saturating_add(other.compile_misses);
    into.exec_hits = into.exec_hits.saturating_add(other.exec_hits);
    into.exec_misses = into.exec_misses.saturating_add(other.exec_misses);
    into.observe_hits = into.observe_hits.saturating_add(other.observe_hits);
    into.observe_misses = into.observe_misses.saturating_add(other.observe_misses);
}

fn parse_perf_cache_jsonl_v13(path: &Path) -> Result<CachePerfCountersV13, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut out = CachePerfCountersV13::default();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed).map_err(|e| {
            SdkError::MissingProject(format!(
                "invalid perf_cache.jsonl line at {}: {e}",
                path.display()
            ))
        })?;
        let Some(kind) = value.get("t").and_then(JsonValue::as_str) else {
            continue;
        };
        let count = value
            .get("data")
            .and_then(|v| v.get("count"))
            .and_then(JsonValue::as_u64)
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32;
        match kind {
            "CacheCompileHit" => out.compile_hits = out.compile_hits.saturating_add(count),
            "CacheCompileMiss" => out.compile_misses = out.compile_misses.saturating_add(count),
            "CacheExecHit" => out.exec_hits = out.exec_hits.saturating_add(count),
            "CacheExecMiss" => out.exec_misses = out.exec_misses.saturating_add(count),
            "CacheObserveHit" => out.observe_hits = out.observe_hits.saturating_add(count),
            "CacheObserveMiss" => out.observe_misses = out.observe_misses.saturating_add(count),
            _ => {}
        }
    }
    Ok(out)
}

fn collect_compile_cache_stats_v13(cache_root: &Path) -> Result<(u32, u64), SdkError> {
    if !cache_root.exists() {
        return Ok((0, 0));
    }
    let mut entries = 0u32;
    let mut bytes = 0u64;
    for entry in fs::read_dir(cache_root)? {
        let path = entry?.path();
        if path.is_dir() {
            entries = entries.saturating_add(1);
            for child in fs::read_dir(path)? {
                let child_path = child?.path();
                if child_path.is_file() {
                    let meta = fs::metadata(&child_path)?;
                    bytes = bytes.saturating_add(meta.len());
                }
            }
        }
    }
    Ok((entries, bytes))
}

fn collect_cache_stats_report_v13(project_root: &Path) -> Result<CacheStatsReportV13, SdkError> {
    let mut report = CacheStatsReportV13::default();
    let artifacts_root = project_root.join(".ocl_artifacts");
    if artifacts_root.exists() {
        for entry in fs::read_dir(&artifacts_root)? {
            let artifact_dir = entry?.path();
            if !artifact_dir.is_dir() {
                continue;
            }
            report.artifact_dirs = report.artifact_dirs.saturating_add(1);
            let perf_path = artifact_dir.join("perf_cache.jsonl");
            if perf_path.exists() {
                report.perf_files = report.perf_files.saturating_add(1);
                let counters = parse_perf_cache_jsonl_v13(&perf_path)?;
                merge_perf_counters_v13(&mut report.counters, counters);
            }
        }
    }

    let local_compile_root = project_root.join(".ocl_cache").join("compile");
    let target_compile_root = project_root
        .join("target")
        .join("ocl")
        .join("cache")
        .join("compile");
    let (local_entries, local_bytes) = collect_compile_cache_stats_v13(&local_compile_root)?;
    let (target_entries, target_bytes) = collect_compile_cache_stats_v13(&target_compile_root)?;
    report.compile_cache_entries = local_entries.saturating_add(target_entries);
    report.compile_cache_bytes = local_bytes.saturating_add(target_bytes);

    Ok(report)
}

fn render_cache_stats_text_v13(report: &CacheStatsReportV13) -> String {
    format!(
        concat!(
            "cache stats\n",
            "artifacts_dirs={}\n",
            "perf_files={}\n",
            "compile_cache_entries={}\n",
            "compile_cache_bytes={}\n",
            "compile_hits={}\n",
            "compile_misses={}\n",
            "exec_hits={}\n",
            "exec_misses={}\n",
            "observe_hits={}\n",
            "observe_misses={}"
        ),
        report.artifact_dirs,
        report.perf_files,
        report.compile_cache_entries,
        report.compile_cache_bytes,
        report.counters.compile_hits,
        report.counters.compile_misses,
        report.counters.exec_hits,
        report.counters.exec_misses,
        report.counters.observe_hits,
        report.counters.observe_misses
    )
}

fn render_cache_stats_json_v13(report: &CacheStatsReportV13) -> String {
    serde_json::to_string_pretty(&json!({
        "artifact_dirs": report.artifact_dirs,
        "perf_files": report.perf_files,
        "compile_cache_entries": report.compile_cache_entries,
        "compile_cache_bytes": report.compile_cache_bytes,
        "counters": {
            "compile_hits": report.counters.compile_hits,
            "compile_misses": report.counters.compile_misses,
            "exec_hits": report.counters.exec_hits,
            "exec_misses": report.counters.exec_misses,
            "observe_hits": report.counters.observe_hits,
            "observe_misses": report.counters.observe_misses,
        }
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn clean_cache_state_v13(project_root: &Path) -> Result<CacheCleanSummaryV13, SdkError> {
    let mut summary = CacheCleanSummaryV13::default();
    let artifacts_dir = project_root.join(".ocl_artifacts");
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
        summary.removed_artifacts = true;
    }
    let local_cache_dir = project_root.join(".ocl_cache");
    if local_cache_dir.exists() {
        fs::remove_dir_all(&local_cache_dir)?;
        summary.removed_local_cache = true;
    }
    let target_cache_dir = project_root.join("target").join("ocl").join("cache");
    if target_cache_dir.exists() {
        fs::remove_dir_all(&target_cache_dir)?;
        summary.removed_target_cache = true;
    }
    Ok(summary)
}

fn render_cache_clean_text_v13(summary: &CacheCleanSummaryV13) -> String {
    format!(
        "cache clean\nremoved_artifacts={}\nremoved_local_cache={}\nremoved_target_cache={}",
        summary.removed_artifacts, summary.removed_local_cache, summary.removed_target_cache
    )
}

fn render_cache_clean_json_v13(summary: &CacheCleanSummaryV13) -> String {
    serde_json::to_string_pretty(&json!({
        "removed_artifacts": summary.removed_artifacts,
        "removed_local_cache": summary.removed_local_cache,
        "removed_target_cache": summary.removed_target_cache,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn run_cache_benchmark_v13(
    project_root: &Path,
    run_engine: RunEngine,
    locked: bool,
    json_mode: bool,
) -> Result<String, SdkError> {
    // Force deterministic "cold -> warm" benchmark sequence.
    let _ = clean_cache_state_v13(project_root)?;
    let cold = with_runtime_env_v08(vec![("OCL_CACHE_DISABLE", Some("0".to_string()))], || {
        run_project_with_trace_engine_and_lock(project_root, run_engine, locked)
    })?;
    let warm = with_runtime_env_v08(vec![("OCL_CACHE_DISABLE", Some("0".to_string()))], || {
        run_project_with_trace_engine_and_lock(project_root, run_engine, locked)
    })?;
    let no_cache =
        with_runtime_env_v08(vec![("OCL_CACHE_DISABLE", Some("1".to_string()))], || {
            let mut config = ExecConfig::default();
            config.step_cap = 4096;
            config.enable_exec_cache = false;
            run_project_with_trace_engine_config_and_lock(project_root, run_engine, locked, config)
        })?;

    let cold_digest = trace_required_digest(&cold.events);
    let warm_digest = trace_required_digest(&warm.events);
    let no_cache_digest = trace_required_digest(&no_cache.events);
    let signature_equal = warm_digest == no_cache_digest;
    let cold_perf = cache_perf_from_trace_summary_v13(&cold);
    let warm_perf = cache_perf_from_trace_summary_v13(&warm);
    let no_cache_perf = cache_perf_from_trace_summary_v13(&no_cache);
    let cold_executed = cold.exec_cache.node_evals_executed;
    let warm_executed = warm.exec_cache.node_evals_executed;
    let executed_reduction_bps = if cold_executed > 0 {
        cold_executed
            .saturating_sub(warm_executed)
            .saturating_mul(10_000)
            / cold_executed
    } else {
        0
    };

    let report_json = json!({
        "engine": run_engine.as_str(),
        "locked": locked,
        "signature_equal": signature_equal,
        "cold": {
            "steps_total": cold.total_steps,
            "required_digest": cold_digest,
            "exec_cache_hits": cold_perf.exec_hits,
            "exec_cache_misses": cold_perf.exec_misses,
            "observe_cache_hits": cold_perf.observe_hits,
            "observe_cache_misses": cold_perf.observe_misses,
            "exec_node_evals_executed": cold.exec_cache.node_evals_executed,
        },
        "warm": {
            "steps_total": warm.total_steps,
            "required_digest": warm_digest,
            "exec_cache_hits": warm_perf.exec_hits,
            "exec_cache_misses": warm_perf.exec_misses,
            "observe_cache_hits": warm_perf.observe_hits,
            "observe_cache_misses": warm_perf.observe_misses,
            "exec_node_evals_executed": warm.exec_cache.node_evals_executed,
        },
        "no_cache": {
            "steps_total": no_cache.total_steps,
            "required_digest": no_cache_digest,
            "exec_cache_hits": no_cache_perf.exec_hits,
            "exec_cache_misses": no_cache_perf.exec_misses,
            "observe_cache_hits": no_cache_perf.observe_hits,
            "observe_cache_misses": no_cache_perf.observe_misses,
            "exec_node_evals_executed": no_cache.exec_cache.node_evals_executed,
        },
        "executed_reduction_bps": executed_reduction_bps,
    });

    let report_dir = project_root.join("target").join("ocl").join("v13");
    fs::create_dir_all(&report_dir)?;
    fs::write(
        report_dir.join("cache_benchmark.json"),
        serde_json::to_string_pretty(&report_json).unwrap_or_else(|_| "{}".to_string()),
    )?;

    if json_mode {
        Ok(serde_json::to_string_pretty(&report_json).unwrap_or_else(|_| "{}".to_string()))
    } else {
        Ok(format!(
            concat!(
                "cache benchmark\n",
                "engine={}\n",
                "locked={}\n",
                "signature_equal={}\n",
                "cold_steps={}\n",
                "warm_steps={}\n",
                "no_cache_steps={}\n",
                "executed_reduction_bps={}\n",
                "cold_exec_hits={}\n",
                "warm_exec_hits={}\n",
                "no_cache_exec_hits={}\n",
                "cold_observe_hits={}\n",
                "warm_observe_hits={}\n",
                "no_cache_observe_hits={}"
            ),
            run_engine.as_str(),
            locked,
            signature_equal,
            cold.total_steps,
            warm.total_steps,
            no_cache.total_steps,
            executed_reduction_bps,
            cold_perf.exec_hits,
            warm_perf.exec_hits,
            no_cache_perf.exec_hits,
            cold_perf.observe_hits,
            warm_perf.observe_hits,
            no_cache_perf.observe_hits
        ))
    }
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
    require_signed_cassette: bool,
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
    if require_signed_cassette {
        out.push_str("require_signed_cassette = true\n");
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
    let trace_index_path = artifact_dir.join("trace_index.json");
    let perf_cache_path = artifact_dir.join("perf_cache.jsonl");
    let signature_path = artifact_dir.join("signature.txt");
    let replay_path = artifact_dir.join("replay.toml");
    let (lane, entry) = read_lane_and_entry_for_v071(project_root);
    let quarantine_policy = parse_quarantine_policy_v15(project_root);
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
            let perf_counters = cache_perf_from_trace_summary_v13(&trace_summary);
            let cassette_hash = if is_quarantine {
                Some(write_quarantine_cassette_bundle_v08(
                    project_root,
                    &artifact_dir,
                    &lane,
                    CASSETTE_MODE_RECORD_V08,
                    &quarantine_policy,
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
            audit_text.push_str(&encode_v11_program_start_line(&lane));
            audit_text.push('\n');
            audit_text.push_str(&encode_v071_lane_marker_line(&lane));
            audit_text.push('\n');
            for (idx, event) in trace_summary.events.iter().enumerate() {
                audit_text.push_str(&encode_v071_trace_event_line(idx, event));
                audit_text.push('\n');
            }
            fs::write(&audit_path, audit_text)?;
            write_trace_index_v11(&trace_index_path, &trace_summary.events)?;
            write_perf_cache_jsonl_v13(&perf_cache_path, &perf_counters)?;
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
                    is_quarantine && quarantine_policy.require_signed_cassette,
                ),
            )?;
        }
        Err(trace_err) => {
            write_perf_cache_jsonl_v13(&perf_cache_path, &CachePerfCountersV13::default())?;
            let cassette_hash = if is_quarantine {
                Some(write_quarantine_cassette_bundle_v08(
                    project_root,
                    &artifact_dir,
                    &lane,
                    CASSETTE_MODE_RECORD_V08,
                    &quarantine_policy,
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
                format!(
                    "{}\n{}\n{line}\n",
                    encode_v11_program_start_line(&lane),
                    encode_v071_lane_marker_line(&lane)
                ),
            )?;
            write_trace_index_v11(&trace_index_path, &[])?;
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
                    is_quarantine && quarantine_policy.require_signed_cassette,
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
    eprintln!("  init  <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview|dep-permission] [--preset workflow_basic|agent_swarm_basic] [--locked] [--registry <index.toml>] [--signer-id <id>] [--sign-key <file>] [--trust-store <file>] [--json]");
    eprintln!("  check <project_dir> [--json] [--locked] [--universe <id>]");
    eprintln!(
        "  run   <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  replay <artifact_dir>");
    eprintln!("  cache stats <project_dir> [--json]");
    eprintln!("  cache clean <project_dir> --yes [--json]");
    eprintln!(
        "  cache bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]"
    );
    eprintln!("  cassette stats <artifact_dir> [--json]");
    eprintln!("  cassette prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]");
    eprintln!("  cassette gc <artifact_dir> [--json]");
    eprintln!(
        "  cassette upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]"
    );
    eprintln!("  budget analyze <artifact_dir|audit.jsonl> [--json]");
    eprintln!("  budget doctor <artifact_dir|audit.jsonl> [--json]");
    eprintln!("  dbg   <artifact_dir> [--script <file>]");
    eprintln!("  doc packs [--json]");
    eprintln!(
        "  trace run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!(
        "  trace view <artifact_dir|audit.jsonl|legacy.trace> [--tail N] [--json] [--legacy-pipe] [--type <event>] [--key <key>] [--kind <kind>] [--reason <rc>] [--module <module_id>]"
    );
    eprintln!(
        "  trace diff <artifactA|auditA> <artifactB|auditB> [--mode strict|align] [--json] [--out <dir>]"
    );
    eprintln!(
        "  minimize <artifact_dir> --goal <error_code:X|divergence|kind:KIND> [--key <key>] [--against <artifactB>] [--out <dir>] [--json]"
    );
    eprintln!(
        "  profile run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]"
    );
    eprintln!("  profile view <profile_file> [--top N] [--json]");
    eprintln!("  fmt   <project_dir> [--check]");
    eprintln!("  test  <project_dir> [--locked] [--universe <id>] [--domain <id>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--golden <dir>] [--clean]");
    eprintln!("  test  --conformance [run|list] --manifest <file> [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--trust-store <file>] [--signer-id <id>] [--sign-key <file>] [--json]");
    eprintln!("  conformance <run|list> --manifest <file> [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--trust-store <file>] [--signer-id <id>] [--sign-key <file>] [--json]");
    eprintln!("  upgrade-check <project_dir> [--manifest <file>] [--target-runtime <id>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--universe <id>] [--json]");
    eprintln!("  lts check <project_dir> [--manifest <file>] [--target-runtime <id>] [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--universe <id>] [--json]");
    eprintln!("  lts report <report_file> [--json]");
    eprintln!(
        "  build <project_dir> [--locked] [--source-only] [--universe <id>] [--attest] [--attest-key <keyid>]"
    );
    eprintln!("  publish <artifact.oclpkg> [--registry <dir>]");
    eprintln!("  fetch <artifact|package> [--registry <dir>] [--out <dir>]");
    eprintln!("  verify-supply <artifact.oclpkg>");
    eprintln!("  deps  resolve <project_dir> [--write-legacy-lock]");
    eprintln!("  deps  update <project_dir> [pkg] [--write-legacy-lock]");
    eprintln!("  deps  verify <project_dir> [--no-lane-policy]");
    eprintln!("  pack  build <project_dir> [--locked]");
    eprintln!("  pack  sign <artifact.oclpkg>");
    eprintln!("  pack  publish <artifact.oclpkg> [--registry <dir>]");
    eprintln!("  pack  verify <artifact.oclpkg>");
    eprintln!("  lock  sync <project_dir> [--write-legacy-lock]");
    eprintln!("  lock  sign <project_dir> --key <keyid> [--lock deps.lock.v3]");
    eprintln!("  lock  verify <project_dir> [--lock deps.lock.v3]");
    eprintln!("  perm  snapshot <project_dir> [--out-dir <dir>]");
    eprintln!(
        "  perm  diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]"
    );
    eprintln!("  perm  approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]");
    eprintln!("  perm  review <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]  # alias of perm diff");
    eprintln!("  perm  doctor <project_dir> [--out <report.json>]");
    eprintln!("  perm  fix --plan <project_dir> [--out <permission_fix_plan.json>]");
    eprintln!("  perm  fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>");
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
    eprintln!("  verify --attest <artifact_dir>");
    eprintln!("  verify --repro <artifact_dir>");
    eprintln!(
        "  verify <project_dir> --phenotype <file> [--registry <dir>] [--locked] [--universe <id>]"
    );
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
            "--legacy-pipe".to_string(),
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

    #[test]
    fn w14_cli_conformance_list_alias_json_pass() {
        let root = temp_project_dir("w14_list_alias");
        prepare_runtime_project(&root);
        let manifest_path = root.join("conformance.v1.toml");
        let root_value = manifest_path_value(&root);
        let manifest = format!(
            concat!(
                "version = 1\n\n",
                "[[scenario]]\n",
                "name = \"single\"\n",
                "path = \"{}\"\n",
                "lane = \"locked_v071\"\n",
                "steps = [\"run\"]\n"
            ),
            root_value
        );
        fs::write(&manifest_path, manifest).expect("write conformance manifest");
        let args = vec![
            "conformance".to_string(),
            "list".to_string(),
            "--manifest".to_string(),
            manifest_path_value(&manifest_path),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&args), 0);
    }

    #[test]
    fn w14_cli_test_conformance_list_mode_pass() {
        let root = temp_project_dir("w14_list_test_mode");
        prepare_runtime_project(&root);
        let manifest_path = root.join("conformance.v1.toml");
        let root_value = manifest_path_value(&root);
        let manifest = format!(
            concat!(
                "version = 1\n\n",
                "[[scenario]]\n",
                "name = \"single\"\n",
                "path = \"{}\"\n",
                "lane = \"locked_v071\"\n",
                "steps = [\"run\"]\n"
            ),
            root_value
        );
        fs::write(&manifest_path, manifest).expect("write conformance manifest");
        let args = vec![
            "test".to_string(),
            "--conformance".to_string(),
            "list".to_string(),
            "--manifest".to_string(),
            manifest_path_value(&manifest_path),
            "--json".to_string(),
        ];
        assert_eq!(run_cli(&args), 0);
    }

    #[test]
    fn w14_cli_test_conformance_run_subcommand_pass() {
        let root = temp_project_dir("w14_test_run_mode");
        prepare_runtime_project(&root);
        let manifest_path = root.join("conformance.v1.toml");
        let out_path = root.join("w14_report.json");
        let root_value = manifest_path_value(&root);
        let manifest = format!(
            concat!(
                "version = 1\n\n",
                "[[scenario]]\n",
                "name = \"single\"\n",
                "path = \"{}\"\n",
                "reactor_ticks = 0\n",
                "composer = false\n",
                "lane = \"locked_v071\"\n",
                "steps = [\"run\"]\n"
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
            "run".to_string(),
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
        assert!(out_path.exists(), "missing W14 report output");
    }
}
