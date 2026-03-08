use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

fn main() {
    let mut args = env::args().skip(1);
    let Some(task) = args.next() else {
        print_help();
        std::process::exit(0);
    };

    if task == "--help" || task == "-h" || task == "help" {
        print_help();
        std::process::exit(0);
    }

    let result = match task.as_str() {
        "editor-ci" => run_editor_ci(),
        _ => Err(format!("unknown xtask `{task}`")),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("xtask commands:");
    println!("  cargo xtask editor-ci");
}

fn run_editor_ci() -> Result<(), String> {
    let repo_root = repo_root()?;
    let mut steps = Vec::<serde_json::Value>::new();
    let commands = [
        vec!["test", "--test", "v19_vscode_integration"],
        vec!["test", "--test", "v19_platform_matrix_aggregate"],
        vec!["test", "--test", "v19_platform_matrix_provenance"],
        vec!["test", "--test", "v19_editor_perf_budget"],
        vec!["test", "--test", "v19_editor_perf_budget_protocol"],
        vec!["test", "--test", "v19_ci_harness_contract"],
        vec!["test", "--test", "v19_editor_ci_entrypoint"],
        vec!["test", "--test", "v19_release_positioning_guard"],
    ];

    for args in commands {
        let status = Command::new("cargo")
            .args(&args)
            .current_dir(&repo_root)
            .status()
            .map_err(|err| format!("run cargo {:?}: {err}", args))?;
        let ok = status.success();
        steps.push(json!({
            "command": format!("cargo {}", args.join(" ")),
            "status": if ok { "PASS" } else { "ERROR" }
        }));
        if !ok {
            write_editor_ci_report(&repo_root, "ERROR", &steps)?;
            return Err(format!("editor-ci step failed: cargo {}", args.join(" ")));
        }
    }

    write_editor_ci_report(&repo_root, "PASS", &steps)
}

fn write_editor_ci_report(
    repo_root: &Path,
    status: &str,
    steps: &[serde_json::Value],
) -> Result<(), String> {
    let report = json!({
        "schema": "ocp.w19.community.editor_ci_entrypoint_report.v1",
        "status": status,
        "entrypoint": "cargo xtask editor-ci",
        "steps": steps,
        "run_manifest_ref": "target/ocp/w19/meta/run_manifest.json"
    });
    let out = repo_root
        .join("target")
        .join("ocp")
        .join("w19")
        .join("community")
        .join("editor_ci_entrypoint_report.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create dir {}: {err}", parent.display()))?;
    }
    let body =
        serde_json::to_string_pretty(&report).map_err(|err| format!("serialize report: {err}"))?;
    fs::write(&out, body).map_err(|err| format!("write report {}: {err}", out.display()))
}

fn repo_root() -> Result<PathBuf, String> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = here.parent() else {
        return Err("resolve repo root from xtask".to_string());
    };
    Ok(root.to_path_buf())
}
