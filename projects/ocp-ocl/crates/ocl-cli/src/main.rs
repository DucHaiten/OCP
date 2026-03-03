use std::path::{Path, PathBuf};

use ocl_runtime_core::RunEngine;
use ocl_runtime_core::RuntimeCoreError;
use ocl_sdk::{
    build_oclpkg_with_lock, build_profile_from_trace, build_project_with_lock,
    check_project_with_lock, compose_phenotype, fetch_artifact, fmt_project, init_project,
    parse_conformance_manifest_v1, publish_artifact, read_profile_json, read_trace_jsonl,
    render_conformance_report_json, render_profile_view, render_trace_view, run_artifact,
    run_conformance_v1, run_project_with_engine_and_lock, run_project_with_trace_engine_and_lock,
    run_reactor_service_with_lock, run_reactor_service_with_trace_engine_and_lock,
    sync_deps_lock_v1, sync_plugin_lock_v1, test_project_with_lock, verify_assembly,
    verify_plugin_lock_v1, verify_supply_artifact, write_conformance_report_json,
    write_profile_json, write_trace_jsonl, ConformanceRunOptionsV1, ProfileViewOptions,
    ReactorRuntimeMode, ReactorServiceOptions, SdkError, TraceViewOptions,
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
                eprintln!("usage: ocl init <project_dir>");
                return 2;
            };
            match init_project(Path::new(path)) {
                Ok(layout) => {
                    println!("initialized {}", layout.root.display());
                    0
                }
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        "check" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl check <project_dir> [--json] [--locked]");
                return 2;
            };
            let json_mode = args.iter().any(|a| a == "--json");
            let locked = args.iter().any(|a| a == "--locked");
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
                    "usage: ocl run <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--locked]"
                );
                return 2;
            };
            let reactor_mode = args.iter().any(|a| a == "--reactor");
            let locked = args.iter().any(|a| a == "--locked");
            let engine_raw =
                parse_string_flag(args, "--engine").unwrap_or_else(|| "interpreter".to_string());
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
            let runtime_mode_raw =
                parse_string_flag(args, "--runtime").unwrap_or_else(|| "deterministic".to_string());
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
            if reactor_mode && parse_string_flag(args, "--engine").is_some() {
                eprintln!("`--engine` currently supports non-reactor `run` only");
                return 2;
            }

            if reactor_mode {
                let options = ReactorServiceOptions {
                    ticks,
                    runtime_mode,
                    socket_listen: socket_listen.clone(),
                    runtime_report: runtime_report.clone(),
                    replay_audit: replay_audit.clone(),
                };
                match run_reactor_service_with_lock(Path::new(path), &options, locked) {
                    Ok(summary) => {
                        println!(
                            "reactor run ok (mode={}, ticks={}, events={}, total_steps={}, backpressure={})",
                            summary.mode.as_str(),
                            summary.ticks,
                            summary.event_count,
                            summary.total_steps,
                            summary.backpressure_count
                        );
                        if let Some(path) = runtime_report {
                            println!("runtime report written: {}", path.display());
                        }
                        if let Some(path) = replay_audit {
                            println!("replay audit written: {}", path.display());
                        }
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                let path_ref = Path::new(path);
                if path_ref
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
                } else {
                    match run_project_with_engine_and_lock(path_ref, run_engine, locked) {
                        Ok(summary) => {
                            println!(
                                "run ok (engine={}, steps={})",
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
                }
            }
        }
        "trace" => {
            let Some(subcmd) = args.get(1).map(String::as_str) else {
                eprintln!(
                    "usage: ocl trace <run|view> ...\n  run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]\n  view <trace_file> [--tail N] [--json]"
                );
                return 2;
            };
            match subcmd {
                "run" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl trace run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let reactor_mode = args.iter().any(|a| a == "--reactor");
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

                    let result = if reactor_mode {
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
                        };
                        run_reactor_service_with_trace_engine_and_lock(
                            Path::new(path),
                            &options,
                            run_engine,
                            locked,
                        )
                    } else {
                        run_project_with_trace_engine_and_lock(Path::new(path), run_engine, locked)
                    };

                    match result {
                        Ok(summary) => {
                            if let Err(err) = write_trace_jsonl(&out_path, &summary.events) {
                                eprintln!("{err}");
                                return 1;
                            }
                            println!(
                                "trace run ok (events={}, steps={}, run_id={}, out={})",
                                summary.events.len(),
                                summary.total_steps,
                                summary.run_id,
                                out_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
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
                    "usage: ocl profile <run|view> ...\n  run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]\n  view <profile_file> [--top N] [--json]"
                );
                return 2;
            };
            match subcmd {
                "run" => {
                    let Some(path) = args.get(2) else {
                        eprintln!(
                            "usage: ocl profile run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]"
                        );
                        return 2;
                    };
                    let locked = args.iter().any(|a| a == "--locked");
                    let reactor_mode = args.iter().any(|a| a == "--reactor");
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

                    let trace_result = if reactor_mode {
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
                        };
                        run_reactor_service_with_trace_engine_and_lock(
                            Path::new(path),
                            &options,
                            run_engine,
                            locked,
                        )
                    } else {
                        run_project_with_trace_engine_and_lock(Path::new(path), run_engine, locked)
                    };

                    match trace_result {
                        Ok(trace_summary) => {
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
                                "profile run ok (events={}, observe={}, digest={}, out={})",
                                report.event_count,
                                report.observe_count,
                                report.required_digest,
                                out_path.display()
                            );
                            0
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            1
                        }
                    }
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
                let manifest_path = parse_string_flag(args, "--manifest")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        PathBuf::from("projects/ocp-ocl/conformance/conformance.v1.toml")
                    });
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
                    eprintln!("usage: ocl test <project_dir> [--locked]");
                    return 2;
                };
                let locked = args.iter().any(|a| a == "--locked");
                match test_project_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
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
                eprintln!("usage: ocl build <project_dir> [--locked] [--source-only]");
                return 2;
            };
            let locked = args.iter().any(|a| a == "--locked");
            let source_only = args.iter().any(|a| a == "--source-only");
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
                eprintln!("usage: ocl compose <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
                return 2;
            };
            let Some(phenotype) = parse_string_flag(args, "--phenotype") else {
                eprintln!("usage: ocl compose <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            let locked = args.iter().any(|a| a == "--locked");
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
                eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
                return 2;
            };
            let Some(phenotype) = parse_string_flag(args, "--phenotype") else {
                eprintln!("usage: ocl verify <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
                return 2;
            };
            let registry =
                parse_string_flag(args, "--registry").unwrap_or_else(|| "registry".to_string());
            let locked = args.iter().any(|a| a == "--locked");
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
    eprintln!("  init  <project_dir>");
    eprintln!("  check <project_dir> [--json] [--locked]");
    eprintln!(
        "  run   <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen ADDR --runtime-report FILE --replay-audit FILE] [--locked]"
    );
    eprintln!(
        "  trace run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]"
    );
    eprintln!("  trace view <trace_file> [--tail N] [--json]");
    eprintln!(
        "  profile run  <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--locked]"
    );
    eprintln!("  profile view <profile_file> [--top N] [--json]");
    eprintln!("  fmt   <project_dir> [--check]");
    eprintln!("  test  <project_dir> [--locked]");
    eprintln!("  test  --conformance --manifest <file> [--out <file>] [--runtime deterministic|throughput] [--engine interpreter|bytecode|dual] [--locked] [--trust-store <file>] [--signer-id <id>] [--sign-key <file>] [--json]");
    eprintln!("  build <project_dir> [--locked] [--source-only]");
    eprintln!("  publish <artifact.oclpkg> [--registry <dir>]");
    eprintln!("  fetch <artifact|package> [--registry <dir>] [--out <dir>]");
    eprintln!("  verify-supply <artifact.oclpkg>");
    eprintln!("  lock  sync <project_dir>");
    eprintln!("  plugin lock sync <project_dir>");
    eprintln!("  plugin verify <project_dir>");
    eprintln!("  compose <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
    eprintln!("  verify  <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use ocl_sdk::{init_project, read_trace_jsonl, sync_deps_lock_v1, trace_required_digest};

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
