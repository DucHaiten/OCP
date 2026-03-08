use std::collections::BTreeMap;
use std::fs;

use serde_json::Value as JsonValue;

mod w17_perm_common;

#[test]
fn v17_perm_rubberstamp_guard_is_strict_in_locked_v071_and_warn_only_in_locked_v06() {
    let strict_root = w17_perm_common::temp_project_dir("rubber_strict");
    w17_perm_common::init_demo_project(&strict_root);
    w17_perm_common::write_manifest_with_fs_rule(&strict_root, "locked_v071", "./**");

    let strict_root_s = strict_root.to_string_lossy().to_string();
    let strict_output =
        w17_perm_common::run_ocp_cli(&["perm", "doctor", &strict_root_s], &BTreeMap::new());
    assert!(
        !strict_output.status.success(),
        "locked_v071 must reject wildcard permission pattern"
    );

    let compat_root = w17_perm_common::temp_project_dir("rubber_compat");
    w17_perm_common::init_demo_project(&compat_root);
    w17_perm_common::write_manifest_with_fs_rule(&compat_root, "locked_v06", "./**");

    let compat_root_s = compat_root.to_string_lossy().to_string();
    let compat_output =
        w17_perm_common::run_ocp_cli(&["perm", "doctor", &compat_root_s], &BTreeMap::new());
    w17_perm_common::assert_ok(&compat_output, "perm doctor in locked_v06");

    let report_path = compat_root
        .join("target")
        .join("ocp")
        .join("w17")
        .join("dx")
        .join("dx_friction_report.json");
    let report = serde_json::from_str::<JsonValue>(
        &fs::read_to_string(&report_path).expect("read doctor report"),
    )
    .expect("parse doctor report json");
    let blocking_total = report
        .get("blocking_total")
        .and_then(JsonValue::as_u64)
        .unwrap_or(u64::MAX);
    assert_eq!(
        blocking_total, 0,
        "compat lane must not block wildcard by default"
    );
}
