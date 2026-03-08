#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use ocp_sdk::{
    evaluate_adapter_cve_policy_v17, evaluate_capability_edge_v17, evaluate_pack_boundary_v17,
    evaluate_pack_trust_policy_v17, inspect_pack_abi_spec_v17, AdapterCveSeverityV17,
    W17_PACK_BOUNDARY_NATIVE_CAP_V1, W17_PACK_BOUNDARY_WASI_V1,
};
use serde_json::json;

pub fn write_gate_e_report() {
    let contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join("w17")
        .join("pack_abi_spec.v1.json");
    let pack_abi = inspect_pack_abi_spec_v17(&contract_path).expect("inspect pack abi");

    let trust_locked_ok =
        evaluate_pack_trust_policy_v17("locked_v071", true, true, true, true, true);
    let trust_locked_deny =
        evaluate_pack_trust_policy_v17("locked_v071", true, false, true, true, true);
    let trust_quarantine =
        evaluate_pack_trust_policy_v17("quarantine", false, false, false, false, false);

    let boundary_wasi = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_WASI_V1,
        &["wasi.fs.read".to_string(), "wasi.net.http".to_string()],
        false,
    );
    let boundary_native = evaluate_pack_boundary_v17(
        W17_PACK_BOUNDARY_NATIVE_CAP_V1,
        &["native.fs.read".to_string(), "native.proc.exec".to_string()],
        false,
    );

    let laundering_deny = evaluate_capability_edge_v17(
        "pkg:caller",
        "pkg:callee",
        "std.fs.read_text",
        "locked_v071",
        true,
        true,
        false,
    );
    let laundering_ok = evaluate_capability_edge_v17(
        "pkg:caller",
        "pkg:callee",
        "std.fs.read_text",
        "locked_v071",
        true,
        true,
        true,
    );

    let cve_strict = evaluate_adapter_cve_policy_v17(
        "locked_v071",
        AdapterCveSeverityV17::Critical,
        false,
        true,
        true,
    );
    let cve_quarantine = evaluate_adapter_cve_policy_v17(
        "quarantine",
        AdapterCveSeverityV17::Critical,
        false,
        true,
        true,
    );

    let report = json!({
        "schema": "ocp.w17.extension_governance_report.v1",
        "run_manifest_ref": "target/ocp/w17/meta/run_manifest.json",
        "pack_abi": {
            "contract_id": pack_abi.contract_id,
            "version": pack_abi.version,
            "contract_path": pack_abi.contract_path.to_string_lossy().replace('\\', "/"),
            "supports_wasi_v1": pack_abi.supports_wasi_v1,
            "supports_native_cap_v1": pack_abi.supports_native_cap_v1,
            "observe_hook": pack_abi.observe_hook,
            "commit_hook": pack_abi.commit_hook,
            "permission_schema_declaration_required": pack_abi.permission_schema_declaration_required,
            "boundaries": pack_abi.boundaries.iter().map(|item| json!({
                "boundary_id": item.boundary_id,
                "required": item.required
            })).collect::<Vec<_>>()
        },
        "trust_policy": {
            "locked_v071_ok": {
                "allowed": trust_locked_ok.allowed,
                "requires_audit_marker": trust_locked_ok.requires_audit_marker,
                "reason_code": trust_locked_ok.reason_code
            },
            "locked_v071_missing_attestation": {
                "allowed": trust_locked_deny.allowed,
                "requires_audit_marker": trust_locked_deny.requires_audit_marker,
                "reason_code": trust_locked_deny.reason_code
            },
            "quarantine_untrusted": {
                "allowed": trust_quarantine.allowed,
                "requires_audit_marker": trust_quarantine.requires_audit_marker,
                "reason_code": trust_quarantine.reason_code
            }
        },
        "sandbox_boundaries": {
            "wasi": {
                "allowed": boundary_wasi.allowed,
                "reason_code": boundary_wasi.reason_code
            },
            "native_cap": {
                "allowed": boundary_native.allowed,
                "reason_code": boundary_native.reason_code
            }
        },
        "capability_laundering": {
            "deny_sample": {
                "allowed": laundering_deny.allowed,
                "reason_code": laundering_deny.reason_code,
                "edge_id": laundering_deny.edge_id
            },
            "allow_sample": {
                "allowed": laundering_ok.allowed,
                "reason_code": laundering_ok.reason_code,
                "edge_id": laundering_ok.edge_id
            }
        },
        "adapter_cve": {
            "locked_v071_unpatched_critical": {
                "allowed": cve_strict.allowed,
                "requires_audit_marker": cve_strict.requires_audit_marker,
                "reason_code": cve_strict.reason_code
            },
            "quarantine_unpatched_critical": {
                "allowed": cve_quarantine.allowed,
                "requires_audit_marker": cve_quarantine.requires_audit_marker,
                "reason_code": cve_quarantine.reason_code
            }
        }
    });

    let out_dir = PathBuf::from("target")
        .join("ocp")
        .join("w17")
        .join("ecosystem");
    fs::create_dir_all(&out_dir).expect("create ecosystem output dir");
    fs::write(
        out_dir.join("extension_governance_report.json"),
        serde_json::to_string_pretty(&report).expect("serialize extension_governance_report"),
    )
    .expect("write extension_governance_report.json");
}
