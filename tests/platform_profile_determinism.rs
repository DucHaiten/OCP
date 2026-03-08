#[path = "w17_gate_c_common.rs"]
mod w17;

use ocp::ocp::{
    canonicalize_path_for_profile, enforce_locale_timezone, enforce_supported_runtime_profile,
    parse_supported_platform_profile, profile_matches_runtime, supported_platform_profile_ids,
    DeterminismError,
};

#[test]
fn supported_platform_profiles_are_locked() {
    let mut ids = supported_platform_profile_ids();
    ids.sort();
    assert_eq!(ids, vec!["linux-x64-ext4", "win-x64-ntfs"]);
    w17::write_gate_c_report();
}

#[test]
fn profile_path_rules_are_canonical_and_escape_safe() {
    let win = parse_supported_platform_profile("win-x64-ntfs").expect("parse win profile");
    let linux = parse_supported_platform_profile("linux-x64-ext4").expect("parse linux profile");

    let win_path = canonicalize_path_for_profile(".\\SRC\\..\\Data\\File.TXT", win)
        .expect("normalize win path");
    assert_eq!(win_path, "data/file.txt");

    let linux_path = canonicalize_path_for_profile("./SRC/../Data/File.TXT", linux)
        .expect("normalize linux path");
    assert_eq!(linux_path, "Data/File.TXT");

    let err = canonicalize_path_for_profile("../../secret.txt", linux)
        .expect_err("path escape must be denied");
    assert!(
        matches!(err, DeterminismError::PathEscape { .. }),
        "unexpected error: {err}"
    );
    w17::write_gate_c_report();
}

#[test]
fn profile_runtime_and_locale_timezone_are_fail_honest() {
    let win = parse_supported_platform_profile("win-x64-ntfs").expect("parse win profile");
    let linux = parse_supported_platform_profile("linux-x64-ext4").expect("parse linux profile");

    assert!(
        profile_matches_runtime(win, "windows", "x86_64"),
        "windows x64 must match win profile"
    );
    assert!(
        profile_matches_runtime(linux, "linux", "x86_64"),
        "linux x64 must match linux profile"
    );
    assert!(
        enforce_supported_runtime_profile(win, "linux", "x86_64").is_err(),
        "cross profile runtime must fail-honest"
    );

    enforce_locale_timezone("C.UTF-8", "UTC").expect("pinned locale/timezone should pass");
    assert!(enforce_locale_timezone("vi_VN.UTF-8", "UTC").is_err());
    assert!(enforce_locale_timezone("C.UTF-8", "Asia/Saigon").is_err());

    w17::write_gate_c_report();
}
