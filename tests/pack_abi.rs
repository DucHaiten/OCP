#[path = "w17_gate_e_common.rs"]
mod w17;

use std::path::PathBuf;

use ocp_sdk::{
    inspect_pack_abi_spec_v17, W17_PACK_BOUNDARY_NATIVE_CAP_V1, W17_PACK_BOUNDARY_WASI_V1,
};

#[test]
fn v17_pack_abi_requires_dual_boundary_contracts() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join("w17")
        .join("pack_abi_spec.v1.json");
    let summary = inspect_pack_abi_spec_v17(&path).expect("inspect pack abi");

    assert_eq!(summary.contract_id, "w17.pack_abi_spec");
    assert_eq!(summary.version, "v1");
    assert!(summary.supports_wasi_v1, "wasi boundary must be required");
    assert!(
        summary.supports_native_cap_v1,
        "native-cap boundary must be required"
    );
    assert!(summary.observe_hook, "observe hook must exist");
    assert!(summary.commit_hook, "commit hook must exist");
    assert!(
        summary.permission_schema_declaration_required,
        "permission schema declaration must be required"
    );

    let ids = summary
        .boundaries
        .iter()
        .map(|item| item.boundary_id.clone())
        .collect::<Vec<String>>();
    assert!(ids.contains(&W17_PACK_BOUNDARY_WASI_V1.to_string()));
    assert!(ids.contains(&W17_PACK_BOUNDARY_NATIVE_CAP_V1.to_string()));

    w17::write_gate_e_report();
}
