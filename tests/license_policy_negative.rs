#[derive(Debug, Clone)]
struct LicenseEntry {
    name: String,
    version: String,
    license: String,
}

fn evaluate_license_policy(entries: &[LicenseEntry], deny: &[&str]) -> Result<(), String> {
    for entry in entries {
        if deny.iter().any(|rule| *rule == entry.license) {
            return Err(format!(
                "X-LICENSE-POLICY-DENIED: {}@{} uses denied license `{}`",
                entry.name, entry.version, entry.license
            ));
        }
    }
    Ok(())
}

#[test]
fn license_policy_negative_denied_license_must_fail() {
    let entries = vec![
        LicenseEntry {
            name: "safe-lib".to_string(),
            version: "1.0.0".to_string(),
            license: "MIT".to_string(),
        },
        LicenseEntry {
            name: "blocked-lib".to_string(),
            version: "2.0.0".to_string(),
            license: "LicenseRef-Proprietary".to_string(),
        },
    ];
    let err = evaluate_license_policy(&entries, &["LicenseRef-Proprietary"])
        .expect_err("denied license must fail policy");
    assert!(
        err.contains("X-LICENSE-POLICY-DENIED"),
        "unexpected error: {err}"
    );
}
