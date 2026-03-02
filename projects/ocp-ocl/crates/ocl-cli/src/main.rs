use std::path::{Path, PathBuf};

use ocl_runtime_core::RuntimeCoreError;
use ocl_sdk::{
    build_project_with_lock, check_project_with_lock, compose_phenotype, fmt_project, init_project,
    run_project_with_lock, run_reactor_ticks_with_lock, sync_deps_lock_v1, test_project_with_lock,
    verify_assembly, SdkError,
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
                eprintln!("usage: ocl run <project_dir> [--reactor --ticks N] [--locked]");
                return 2;
            };
            let reactor_mode = args.iter().any(|a| a == "--reactor");
            let locked = args.iter().any(|a| a == "--locked");
            let ticks = parse_u32_flag(args, "--ticks").unwrap_or(32);
            if reactor_mode {
                match run_reactor_ticks_with_lock(Path::new(path), ticks, locked) {
                    Ok(summary) => {
                        println!(
                            "reactor run ok (ticks={}, total_steps={})",
                            summary.ticks, summary.total_steps
                        );
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                match run_project_with_lock(Path::new(path), locked) {
                    Ok(summary) => {
                        println!("run ok (steps={})", summary.steps);
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
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
        "build" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: ocl build <project_dir> [--locked]");
                return 2;
            };
            let locked = args.iter().any(|a| a == "--locked");
            match build_project_with_lock(Path::new(path), locked) {
                Ok(summary) => {
                    println!("build ok (files_bundled={})", summary.files_bundled);
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
    eprintln!("  run   <project_dir> [--reactor --ticks N] [--locked]");
    eprintln!("  fmt   <project_dir> [--check]");
    eprintln!("  test  <project_dir> [--locked]");
    eprintln!("  build <project_dir> [--locked]");
    eprintln!("  lock  sync <project_dir>");
    eprintln!("  compose <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
    eprintln!("  verify  <project_dir> --phenotype <file> [--registry <dir>] [--locked]");
}
