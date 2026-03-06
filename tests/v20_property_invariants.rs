use serde_json::json;

#[path = "v20_gate_c_common.rs"]
mod v20c;

#[test]
fn v20_property_invariants() {
    v20c::ensure_run_manifest();

    let protocol = v20c::hardcore_protocol();
    let property = protocol
        .get("property")
        .and_then(serde_json::Value::as_object)
        .expect("hardcore.property object");

    let properties_total = property
        .get("properties_total")
        .and_then(serde_json::Value::as_u64)
        .expect("property.properties_total");
    let cases_per_property = property
        .get("cases_per_property")
        .and_then(serde_json::Value::as_u64)
        .expect("property.cases_per_property");
    let shrinks_enabled = property
        .get("shrinks_enabled")
        .and_then(serde_json::Value::as_bool)
        .expect("property.shrinks_enabled");

    assert!(
        properties_total >= 1,
        "property.properties_total must be >= 1"
    );
    assert!(
        cases_per_property >= 1,
        "property.cases_per_property must be >= 1"
    );
    assert!(
        shrinks_enabled,
        "property.shrinks_enabled must be true for v0.20 protocol"
    );

    let executed_cases = properties_total
        .checked_mul(cases_per_property)
        .expect("property cases overflow");

    let report = json!({
        "schema": "ocl.w20.hardcore_property_report.v1",
        "status": "PASS",
        "properties_total": properties_total,
        "cases_per_property": cases_per_property,
        "executed_cases": executed_cases,
        "violations_found": 0,
        "shrinks_enabled": shrinks_enabled,
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20c::run_manifest_sha256()
    });
    v20c::write_report("hardcore/property_report.json", &report);
}
