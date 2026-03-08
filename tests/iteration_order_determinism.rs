#[path = "w17_gate_c_common.rs"]
mod w17;

use ocp::ocp::{
    canonicalize_env_entries, canonicalize_iteration_paths, parse_supported_platform_profile,
};

#[test]
fn filesystem_listing_is_canonicalized_before_use() {
    let profile = parse_supported_platform_profile("linux-x64-ext4").expect("parse profile");
    let listing = vec![
        "./zeta.txt".to_string(),
        "./alpha.txt".to_string(),
        "./beta/../beta.txt".to_string(),
        "./alpha.txt".to_string(),
    ];
    let canonical = canonicalize_iteration_paths(&listing, profile).expect("canonical listing");
    assert_eq!(
        canonical,
        vec![
            "alpha.txt".to_string(),
            "beta.txt".to_string(),
            "zeta.txt".to_string()
        ]
    );
    w17::write_gate_c_report();
}

#[test]
fn env_iteration_order_is_bytewise_stable() {
    let sorted = canonicalize_env_entries(&[
        ("Z_KEY".to_string(), "b".to_string()),
        ("A_KEY".to_string(), "c".to_string()),
        ("A_KEY".to_string(), "a".to_string()),
    ]);
    assert_eq!(
        sorted,
        vec![
            ("A_KEY".to_string(), "a".to_string()),
            ("A_KEY".to_string(), "c".to_string()),
            ("Z_KEY".to_string(), "b".to_string())
        ]
    );
    w17::write_gate_c_report();
}
