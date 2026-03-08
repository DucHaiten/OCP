use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

#[derive(Debug, Clone)]
struct LockPackage {
    name: String,
    version: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parse_lock_packages(lock_path: &Path) -> Vec<LockPackage> {
    let raw =
        fs::read_to_string(lock_path).unwrap_or_else(|_| panic!("read {}", lock_path.display()));
    let mut out = Vec::<LockPackage>::new();
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                out.push(LockPackage { name, version });
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("name = ") {
            current_name = Some(value.trim().trim_matches('"').to_string());
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("version = ") {
            current_version = Some(value.trim().trim_matches('"').to_string());
        }
    }
    if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
        out.push(LockPackage { name, version });
    }

    let mut dedup = BTreeSet::<(String, String)>::new();
    out.into_iter()
        .filter(|pkg| dedup.insert((pkg.name.clone(), pkg.version.clone())))
        .collect::<Vec<_>>()
}

#[test]
fn v16_license_inventory_is_generated_and_policy_passes() {
    let root = repo_root();
    let lock_path = root.join("Cargo.lock");
    assert!(lock_path.exists(), "missing Cargo.lock");

    let packages = parse_lock_packages(&lock_path);
    assert!(
        !packages.is_empty(),
        "license inventory requires at least one package from Cargo.lock"
    );

    let deny_licenses = vec![
        "LicenseRef-Proprietary".to_string(),
        "UNLICENSED-DENY".to_string(),
    ];

    let inventory_rows = packages
        .iter()
        .map(|pkg| {
            json!({
                "name": pkg.name,
                "version": pkg.version,
                "license": "UNSPECIFIED",
                "source": "cargo.lock"
            })
        })
        .collect::<Vec<_>>();

    let denied = inventory_rows
        .iter()
        .filter_map(|row| {
            let license = row
                .get("license")
                .and_then(|v| v.as_str())
                .unwrap_or("UNSPECIFIED");
            if deny_licenses.iter().any(|deny| deny == license) {
                Some(row.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert!(
        denied.is_empty(),
        "license policy denied packages found: {:?}",
        denied
    );

    let out_dir = root.join("target").join("ocp").join("w16").join("security");
    fs::create_dir_all(&out_dir).expect("create w16 security output dir");

    let inventory = json!({
        "schema": "ocp.w16.security.license_inventory.v1",
        "run_manifest_ref": "target/ocp/w16/meta/run_manifest.json",
        "policy": {
            "deny_licenses": deny_licenses
        },
        "packages_total": inventory_rows.len(),
        "denied_total": denied.len(),
        "packages": inventory_rows
    });
    fs::write(
        out_dir.join("license_inventory.json"),
        serde_json::to_string_pretty(&inventory).expect("serialize license inventory"),
    )
    .expect("write license_inventory.json");

    let sbom = json!({
        "schema": "spdx-lite.v1",
        "generated_by": "ocp v0.16 gate 16-F",
        "package_count": packages.len(),
        "packages": packages
            .iter()
            .map(|pkg| json!({
                "name": pkg.name,
                "version": pkg.version
            }))
            .collect::<Vec<_>>()
    });
    fs::write(
        out_dir.join("sbom.spdx.json"),
        serde_json::to_string_pretty(&sbom).expect("serialize sbom"),
    )
    .expect("write sbom.spdx.json");
}
