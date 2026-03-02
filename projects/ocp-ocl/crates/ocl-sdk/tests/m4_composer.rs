use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ocl_sdk::{compose_phenotype, init_project, verify_assembly};

fn temp_project_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_millis();
    std::env::temp_dir().join(format!("ocl_m4_{tag}_{stamp}"))
}

fn write_component(
    registry_root: &Path,
    file_stem: &str,
    name: &str,
    phase: &str,
    provides: &str,
    requires: &str,
    template_rel: &str,
    template_body: &str,
) {
    let components_dir = registry_root.join("components");
    let templates_dir = registry_root.join("templates");
    fs::create_dir_all(&components_dir).expect("mkdir components");
    fs::create_dir_all(&templates_dir).expect("mkdir templates");

    let spec = format!(
        "name=\"{name}\"\nphase=\"{phase}\"\nprovides=\"{provides}\"\nrequires=\"{requires}\"\ntemplate=\"{template_rel}\"\n"
    );
    fs::write(components_dir.join(format!("{file_stem}.toml")), spec).expect("write spec");
    fs::write(registry_root.join(template_rel), template_body).expect("write template");
}

fn write_phenotype(path: &Path, components: &[&str]) {
    let mut body = String::from("version=1\n");
    for component in components {
        body.push_str("component=");
        body.push_str(component);
        body.push('\n');
    }
    fs::write(path, body).expect("write phenotype");
}

#[test]
fn m4_compose_and_verify_pass() {
    let root = temp_project_dir("compose_ok");
    let registry = root.join("registry");
    init_project(&root).expect("init");

    write_component(
        &registry,
        "core_log",
        "core.logging",
        "render",
        "cap.log.info",
        "",
        "templates/core_log.ocl",
        "module generated.core_logging;\nlet ok = true;\ncondition(ok);\n",
    );
    write_component(
        &registry,
        "core_fetch",
        "core.fetch",
        "infer",
        "cap.http.get",
        "cap.log.info",
        "templates/core_fetch.ocl",
        "module generated.core_fetch;\nlet ok = true;\ncondition(ok);\n",
    );

    let phenotype = root.join("phenotype.toml");
    write_phenotype(&phenotype, &["core.logging", "core.fetch"]);

    let compose = compose_phenotype(&root, &phenotype, &registry, false).expect("compose");
    assert_eq!(compose.selected_components, 2);
    assert!(compose.generated_files >= 3);
    assert!(compose.generated_module.exists());
    assert!(compose.proof_path.exists());

    let verify = verify_assembly(&root, &phenotype, &registry, false).expect("verify");
    assert!(verify.ok);
    assert_eq!(verify.selected_components, 2);
}

#[test]
fn m4_verify_fails_when_generated_mutated() {
    let root = temp_project_dir("verify_fail");
    let registry = root.join("registry");
    init_project(&root).expect("init");

    write_component(
        &registry,
        "core_log",
        "core.logging",
        "render",
        "cap.log.info",
        "",
        "templates/core_log.ocl",
        "module generated.core_logging;\nlet ok = true;\ncondition(ok);\n",
    );

    let phenotype = root.join("phenotype.toml");
    write_phenotype(&phenotype, &["core.logging"]);

    compose_phenotype(&root, &phenotype, &registry, false).expect("compose");
    let generated_mod = root.join("src").join("generated").join("mod.ocl");
    fs::write(
        generated_mod,
        "module generated.mod;\nlet changed = false;\n",
    )
    .expect("mutate generated");

    let err = verify_assembly(&root, &phenotype, &registry, false).expect_err("verify must fail");
    assert!(err.to_string().contains("generated hash changed"));
}

#[test]
fn m4_compose_fails_when_component_missing() {
    let root = temp_project_dir("missing_component");
    let registry = root.join("registry");
    init_project(&root).expect("init");
    fs::create_dir_all(registry.join("components")).expect("mkdir components");

    let phenotype = root.join("phenotype.toml");
    write_phenotype(&phenotype, &["core.unknown"]);

    let err = compose_phenotype(&root, &phenotype, &registry, false).expect_err("compose fail");
    assert!(err.to_string().contains("missing component"));
}
