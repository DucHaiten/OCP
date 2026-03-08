use ocp_sdk::{enforce_build_env_allowlist_v17, toolchain_digest_v17, ToolchainDigestInputV17};

#[test]
fn v17_toolchain_digest_is_deterministic_under_reordered_inputs() {
    let input_a = ToolchainDigestInputV17 {
        rustc_version: "rustc 1.99.0-nightly".to_string(),
        cargo_version: "cargo 1.99.0-nightly".to_string(),
        target_triple: "x86_64-pc-windows-msvc".to_string(),
        build_flags: vec!["--release".to_string(), "-Ztrim-paths=yes".to_string()],
        allowed_env: vec![
            ("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string()),
            ("CARGO_TERM_COLOR".to_string(), "never".to_string()),
        ],
    };
    let input_b = ToolchainDigestInputV17 {
        rustc_version: "rustc 1.99.0-nightly".to_string(),
        cargo_version: "cargo 1.99.0-nightly".to_string(),
        target_triple: "x86_64-pc-windows-msvc".to_string(),
        build_flags: vec!["-Ztrim-paths=yes".to_string(), "--release".to_string()],
        allowed_env: vec![
            ("CARGO_TERM_COLOR".to_string(), "never".to_string()),
            ("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string()),
        ],
    };
    let da = toolchain_digest_v17(&input_a);
    let db = toolchain_digest_v17(&input_b);
    assert_eq!(da, db, "toolchain digest must be order-independent");
}

#[test]
fn v17_build_env_allowlist_rejects_unknown_flags() {
    let err = enforce_build_env_allowlist_v17(
        &["RUSTFLAGS", "CARGO_TERM_COLOR"],
        &[
            ("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string()),
            ("UNEXPECTED_FLAG".to_string(), "1".to_string()),
        ],
    )
    .expect_err("unknown env flag must be rejected");
    assert!(
        err.to_string().contains("X-TOOLCHAIN-ENV-DENY"),
        "unexpected error: {err}"
    );
}

#[test]
fn v17_build_env_allowlist_sorts_output_deterministically() {
    let filtered = enforce_build_env_allowlist_v17(
        &["RUSTFLAGS", "CARGO_TERM_COLOR"],
        &[
            ("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string()),
            ("CARGO_TERM_COLOR".to_string(), "never".to_string()),
        ],
    )
    .expect("allowlist should pass");
    assert_eq!(
        filtered,
        vec![
            ("CARGO_TERM_COLOR".to_string(), "never".to_string()),
            ("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string())
        ]
    );
}
