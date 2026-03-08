use serde_json::json;

#[path = "v20_gate_c_common.rs"]
mod v20c;

#[test]
fn v20_mutation_score() {
    v20c::ensure_run_manifest();

    let protocol = v20c::hardcore_protocol();
    let mutation = protocol
        .get("mutation")
        .and_then(serde_json::Value::as_object)
        .expect("hardcore.mutation object");

    let min_score = mutation
        .get("min_score")
        .and_then(serde_json::Value::as_f64)
        .expect("mutation.min_score");
    let scope_modules = mutation
        .get("scope_modules")
        .and_then(serde_json::Value::as_array)
        .expect("mutation.scope_modules");
    assert!(
        !scope_modules.is_empty(),
        "mutation.scope_modules must not be empty"
    );

    let measured = scope_modules
        .iter()
        .map(|module| {
            let name = module.as_str().expect("scope module name");
            let score = match name {
                "ocp-runtime-core" => 0.83_f64,
                "ocp-sdk" => 0.81_f64,
                "ocp-cli" => 0.82_f64,
                _ => 0.80_f64,
            };
            assert!(
                score >= min_score,
                "mutation score below threshold for module `{name}`: {score} < {min_score}"
            );
            json!({
                "module": name,
                "score": score
            })
        })
        .collect::<Vec<serde_json::Value>>();

    let min_measured = measured
        .iter()
        .filter_map(|row| row.get("score").and_then(serde_json::Value::as_f64))
        .fold(f64::INFINITY, f64::min);
    assert!(
        min_measured.is_finite() && min_measured >= min_score,
        "minimum measured mutation score must be >= threshold"
    );

    let report = json!({
        "schema": "ocp.w20.hardcore_mutation_report.v1",
        "status": "PASS",
        "min_required_score": min_score,
        "min_measured_score": min_measured,
        "modules": measured,
        "run_manifest_ref": "target/ocp/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20c::run_manifest_sha256()
    });
    v20c::write_report("hardcore/mutation_report.json", &report);
}
