use serde_json::json;

#[path = "v20_gate_g_common.rs"]
mod v20g;

#[test]
fn v20_release_profile_no_chaos() {
    v20g::ensure_run_manifest();
    let fixture = v20g::ensure_release_fixture_v20();
    assert!(fixture.manifest_path.exists(), "release manifest missing");

    assert!(
        option_env!("CARGO_FEATURE_W20_CHAOS").is_none(),
        "release profile unexpectedly has CARGO_FEATURE_W20_CHAOS"
    );

    let cargo_toml =
        std::fs::read_to_string(v20g::repo_root().join("Cargo.toml")).expect("read Cargo.toml");
    assert!(
        cargo_toml.contains("[features]"),
        "Cargo.toml must declare [features]"
    );
    assert!(
        cargo_toml.contains("w20_chaos"),
        "Cargo.toml must define `w20_chaos` feature for test-only chaos gates"
    );
    assert!(
        cargo_toml.contains("default = []"),
        "Cargo.toml default features must not include chaos"
    );

    let report = json!({
        "schema": "ocl.w20.release.release_profile_no_chaos_report.v1",
        "status": "PASS",
        "w20_chaos_enabled_at_compile_time": false,
        "cargo_feature_env_present": false,
        "release_manifest_ref": "target/ocl/w20/release/v1_rc_manifest.json",
        "run_manifest_ref": "target/ocl/w20/meta/run_manifest.json",
        "run_manifest_sha256": v20g::run_manifest_sha256()
    });
    v20g::write_report("release/release_profile_no_chaos_report.json", &report);
}
