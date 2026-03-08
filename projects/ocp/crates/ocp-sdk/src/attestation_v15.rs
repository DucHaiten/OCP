use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{
    build_project_with_lock, default_conformance_manifest_path, deterministic_sign,
    deterministic_verify, lock_v3_ast_hash_v15, lock_v3_sig_path, parse_lock_v3,
    parse_project_language_config_v071, project_layout, sha256_hex,
    verify_deps_lock_v3_signature_v15, verify_project_exists, write_permission_snapshot_v15,
    BuildAttestationSummaryV15, BuildReproVerifySummaryV15, BuildVerifyAttestationSummaryV15,
    SdkError,
};

const REPRO_EXCLUSIONS_V15: [&str; 3] = ["created_at", "machine_id", "cwd"];

#[derive(Debug, Clone)]
struct ParsedBuildAttestationSigV15 {
    manifest_path: String,
    key_id: String,
    manifest_hash_sha256: String,
    signature_b64: String,
    signer_pub_b64: String,
}

fn normalize_path_slash_v15(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_text_value_v15(value: &str) -> String {
    value.trim().to_string()
}

fn canonicalize_json_value_v15(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut sorted = BTreeMap::<String, JsonValue>::new();
            for (key, item) in map {
                sorted.insert(key.clone(), canonicalize_json_value_v15(item));
            }
            let mut out = JsonMap::new();
            for (key, item) in sorted {
                out.insert(key, item);
            }
            JsonValue::Object(out)
        }
        JsonValue::Array(items) => JsonValue::Array(
            items
                .iter()
                .map(canonicalize_json_value_v15)
                .collect::<Vec<JsonValue>>(),
        ),
        _ => value.clone(),
    }
}

fn canonical_json_string_v15(value: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonicalize_json_value_v15(value);
    serde_json::to_string(&canonical).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-ATTEST-CANONICAL-SERIALIZE: failed to serialize canonical JSON ({err})"
        ))
    })
}

fn write_canonical_json_file_v15(path: &Path, value: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonical_json_string_v15(value)?;
    fs::write(path, format!("{canonical}\n"))?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn read_canonical_json_file_v15(path: &Path) -> Result<(JsonValue, String), SdkError> {
    let raw = fs::read_to_string(path)?;
    let value: JsonValue = serde_json::from_str(&raw).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-ATTEST-MANIFEST-PARSE: invalid JSON in {} ({err})",
            path.display()
        ))
    })?;
    let canonical = canonical_json_string_v15(&value)?;
    Ok((value, sha256_hex(canonical.as_bytes())))
}

fn parse_attestation_sig_file_v15(path: &Path) -> Result<ParsedBuildAttestationSigV15, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut has_version = false;
    let mut has_hasher = false;
    let mut manifest_path = None::<String>;
    let mut key_id = None::<String>;
    let mut manifest_hash_sha256 = None::<String>;
    let mut signature_b64 = None::<String>;
    let mut signer_pub_b64 = None::<String>;

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "version=1" {
            has_version = true;
            continue;
        }
        if line == "hasher_version=sha256-v1" {
            has_hasher = true;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = normalize_text_value_v15(v);
        match key {
            "manifest_path" => manifest_path = Some(value),
            "key_id" => key_id = Some(value),
            "manifest_hash_sha256" => manifest_hash_sha256 = Some(value),
            "signature_b64" => signature_b64 = Some(value),
            "signer_pub_b64" => signer_pub_b64 = Some(value),
            _ => {}
        }
    }

    if !has_version {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `version=1` header".to_string(),
        ));
    }
    if !has_hasher {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `hasher_version=sha256-v1` header".to_string(),
        ));
    }

    let Some(manifest_path) = manifest_path else {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `manifest_path`".to_string(),
        ));
    };
    let Some(key_id) = key_id else {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `key_id`".to_string(),
        ));
    };
    let Some(manifest_hash_sha256) = manifest_hash_sha256 else {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `manifest_hash_sha256`".to_string(),
        ));
    };
    let Some(signature_b64) = signature_b64 else {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `signature_b64`".to_string(),
        ));
    };
    let Some(signer_pub_b64) = signer_pub_b64 else {
        return Err(SdkError::SupplyInvalid(
            "build_manifest.sig missing `signer_pub_b64`".to_string(),
        ));
    };

    Ok(ParsedBuildAttestationSigV15 {
        manifest_path,
        key_id,
        manifest_hash_sha256,
        signature_b64,
        signer_pub_b64,
    })
}

fn build_manifest_value_v15(
    root: &Path,
    out_dir: &Path,
    run_build: bool,
) -> Result<JsonValue, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;

    let manifest_raw = fs::read_to_string(&layout.manifest)?;
    let language_cfg = parse_project_language_config_v071(&manifest_raw);
    let lane = language_cfg.lane.trim().to_string();
    let locked = lane == "locked_v071";
    if run_build {
        let _ = build_project_with_lock(root, locked)?;
    }

    let lock_path = layout.root.join("deps.lock.v3");
    if !lock_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing deps.lock.v3: {} (run `ocp deps resolve <project_dir>`)",
            lock_path.display()
        )));
    }
    let raw_lock = fs::read_to_string(&lock_path)?;
    let deps = parse_lock_v3(&raw_lock)?;
    let lock_hash_sha256 = lock_v3_ast_hash_v15(&deps);
    let lock_sig_path = lock_v3_sig_path(&lock_path);
    let lock_signature_present = lock_sig_path.exists();
    let lock_signature_verified = if lock_signature_present {
        let _ = verify_deps_lock_v3_signature_v15(&layout.root)?;
        true
    } else {
        false
    };

    let permission_snapshot = write_permission_snapshot_v15(root, Some(out_dir))?;

    let conformance_manifest = default_conformance_manifest_path(&layout.root);
    let (conformance_manifest_rel, conformance_hash_sha256) = if conformance_manifest.exists() {
        let raw = fs::read(&conformance_manifest)?;
        (
            Some(
                conformance_manifest
                    .strip_prefix(&layout.root)
                    .unwrap_or(conformance_manifest.as_path())
                    .to_string_lossy()
                    .replace('\\', "/"),
            ),
            Some(sha256_hex(&raw)),
        )
    } else {
        (None, None)
    };

    let deps_json = JsonValue::Array(
        deps.iter()
            .map(|dep| {
                let mut item = JsonMap::new();
                item.insert("alias".to_string(), JsonValue::String(dep.alias.clone()));
                item.insert("name".to_string(), JsonValue::String(dep.name.clone()));
                item.insert(
                    "version".to_string(),
                    JsonValue::String(dep.version.clone()),
                );
                item.insert("source".to_string(), JsonValue::String(dep.source.clone()));
                item.insert(
                    "content_hash_sha256".to_string(),
                    JsonValue::String(dep.content_hash_sha256.clone()),
                );
                item.insert(
                    "signature_b64".to_string(),
                    JsonValue::String(dep.signature_b64.clone()),
                );
                item.insert(
                    "signer_pub_b64".to_string(),
                    JsonValue::String(dep.signer_pub_b64.clone()),
                );
                item.insert(
                    "trust_decision".to_string(),
                    JsonValue::String(dep.trust_decision.clone()),
                );
                item.insert(
                    "requested_permissions_hash".to_string(),
                    JsonValue::String(dep.requested_permissions_hash.clone()),
                );
                item.insert(
                    "dependencies".to_string(),
                    JsonValue::Array(
                        dep.dependencies
                            .iter()
                            .map(|value| JsonValue::String(value.clone()))
                            .collect(),
                    ),
                );
                JsonValue::Object(item)
            })
            .collect(),
    );

    let cache_mode = match std::env::var("OCP_DISABLE_CACHE") {
        Ok(value)
            if value.trim() == "1"
                || value.trim().eq_ignore_ascii_case("true")
                || value.trim().eq_ignore_ascii_case("yes") =>
        {
            "off".to_string()
        }
        _ => "on".to_string(),
    };

    let cassette_hash_path = out_dir.join("cassette_hash.txt");
    let cassette_hash = if cassette_hash_path.exists() {
        Some(fs::read_to_string(&cassette_hash_path)?.trim().to_string())
    } else {
        None
    };

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let machine_id = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let cwd = std::env::current_dir()
        .map(|v| normalize_path_slash_v15(&v))
        .unwrap_or_else(|_| ".".to_string());
    let project_root = normalize_path_slash_v15(
        &layout
            .root
            .canonicalize()
            .unwrap_or_else(|_| layout.root.clone()),
    );

    let mut runtime = JsonMap::new();
    runtime.insert(
        "sdk_version".to_string(),
        JsonValue::String(env!("CARGO_PKG_VERSION").to_string()),
    );
    runtime.insert(
        "git_commit".to_string(),
        JsonValue::String(
            std::env::var("OCP_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        ),
    );

    let mut lock = JsonMap::new();
    lock.insert(
        "path".to_string(),
        JsonValue::String("deps.lock.v3".to_string()),
    );
    lock.insert(
        "hash_sha256".to_string(),
        JsonValue::String(lock_hash_sha256),
    );
    lock.insert(
        "signature_present".to_string(),
        JsonValue::Bool(lock_signature_present),
    );
    lock.insert(
        "signature_verified".to_string(),
        JsonValue::Bool(lock_signature_verified),
    );

    let mut permissions = JsonMap::new();
    permissions.insert(
        "snapshot_file".to_string(),
        JsonValue::String("permissions.snapshot.json".to_string()),
    );
    permissions.insert(
        "snapshot_hash_sha256".to_string(),
        JsonValue::String(permission_snapshot.snapshot_hash_sha256),
    );

    let mut conformance = JsonMap::new();
    conformance.insert(
        "manifest".to_string(),
        conformance_manifest_rel
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    );
    conformance.insert(
        "manifest_hash_sha256".to_string(),
        conformance_hash_sha256
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    );

    let mut manifest = JsonMap::new();
    manifest.insert("version".to_string(), JsonValue::Number(1u64.into()));
    manifest.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    manifest.insert("runtime".to_string(), JsonValue::Object(runtime));
    manifest.insert("project_root".to_string(), JsonValue::String(project_root));
    manifest.insert("lane".to_string(), JsonValue::String(lane));
    manifest.insert("lock".to_string(), JsonValue::Object(lock));
    manifest.insert("deps".to_string(), deps_json);
    manifest.insert("permissions".to_string(), JsonValue::Object(permissions));
    manifest.insert("conformance".to_string(), JsonValue::Object(conformance));
    manifest.insert("cache_mode".to_string(), JsonValue::String(cache_mode));
    manifest.insert(
        "cassette_hash".to_string(),
        cassette_hash
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    );
    manifest.insert("created_at".to_string(), JsonValue::String(created_at));
    manifest.insert("machine_id".to_string(), JsonValue::String(machine_id));
    manifest.insert("cwd".to_string(), JsonValue::String(cwd));
    Ok(JsonValue::Object(manifest))
}

fn attestation_artifact_dir_v15(root: &Path) -> PathBuf {
    root.join("target").join("ocp").join("attestation")
}

fn strip_repro_exclusions_v15(value: &mut JsonValue) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    for key in REPRO_EXCLUSIONS_V15 {
        obj.remove(key);
    }
}

pub fn build_attestation_v15(
    root: &Path,
    key_id: Option<&str>,
) -> Result<BuildAttestationSummaryV15, SdkError> {
    let key_id = key_id.unwrap_or("local-project-key").trim();
    if key_id.is_empty() {
        return Err(SdkError::SupplyInvalid(
            "attestation key id must not be empty".to_string(),
        ));
    }

    let artifact_dir = attestation_artifact_dir_v15(root);
    fs::create_dir_all(&artifact_dir)?;
    let manifest_path = artifact_dir.join("build_manifest.json");
    let sig_path = artifact_dir.join("build_manifest.sig");

    let manifest_value = build_manifest_value_v15(root, &artifact_dir, true)?;
    let manifest_hash_sha256 = write_canonical_json_file_v15(&manifest_path, &manifest_value)?;
    let manifest_rel = "build_manifest.json";
    let message = format!(
        "build-attest-v15|key_id={}|manifest_path={}|manifest_hash_sha256={}",
        key_id, manifest_rel, manifest_hash_sha256
    );
    let sign_context = format!("build-attest-v15|{}", key_id);
    let (signature_b64, signer_pub_b64) = deterministic_sign(&sign_context, message.as_bytes());
    let sig_raw = format!(
        concat!(
            "version=1\n",
            "hasher_version=sha256-v1\n",
            "manifest_path={}\n",
            "key_id={}\n",
            "manifest_hash_sha256={}\n",
            "signature_b64={}\n",
            "signer_pub_b64={}\n"
        ),
        manifest_rel, key_id, manifest_hash_sha256, signature_b64, signer_pub_b64
    );
    fs::write(&sig_path, sig_raw)?;

    Ok(BuildAttestationSummaryV15 {
        artifact_dir,
        manifest_path,
        sig_path,
        key_id: key_id.to_string(),
        manifest_hash_sha256,
    })
}

pub fn verify_build_attestation_v15(
    artifact_dir: &Path,
) -> Result<BuildVerifyAttestationSummaryV15, SdkError> {
    let manifest_path = artifact_dir.join("build_manifest.json");
    let sig_path = artifact_dir.join("build_manifest.sig");
    if !manifest_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing build_manifest.json: {}",
            manifest_path.display()
        )));
    }
    if !sig_path.exists() {
        return Err(SdkError::SupplyInvalid(format!(
            "missing build_manifest.sig: {}",
            sig_path.display()
        )));
    }

    let sig = parse_attestation_sig_file_v15(&sig_path)?;
    if sig.manifest_path.replace('\\', "/") != "build_manifest.json" {
        return Err(SdkError::SupplyInvalid(format!(
            "X-ATTEST-SIGNATURE-MISMATCH: manifest path mismatch in build_manifest.sig (expected `build_manifest.json`, got `{}`).",
            sig.manifest_path
        )));
    }

    let (_, actual_manifest_hash_sha256) = read_canonical_json_file_v15(&manifest_path)?;
    if sig.manifest_hash_sha256 != actual_manifest_hash_sha256 {
        return Err(SdkError::SupplyInvalid(format!(
            "X-ATTEST-SIGNATURE-MISMATCH: build manifest hash mismatch (expected {}, got {}).",
            sig.manifest_hash_sha256, actual_manifest_hash_sha256
        )));
    }
    let message = format!(
        "build-attest-v15|key_id={}|manifest_path={}|manifest_hash_sha256={}",
        sig.key_id, "build_manifest.json", sig.manifest_hash_sha256
    );
    deterministic_verify(message.as_bytes(), &sig.signature_b64, &sig.signer_pub_b64).map_err(
        |err| {
            SdkError::SupplyInvalid(format!(
                "X-ATTEST-SIGNATURE-MISMATCH: signature verification failed ({err})"
            ))
        },
    )?;

    Ok(BuildVerifyAttestationSummaryV15 {
        artifact_dir: artifact_dir.to_path_buf(),
        manifest_path,
        sig_path,
        key_id: sig.key_id,
        manifest_hash_sha256: sig.manifest_hash_sha256,
    })
}

pub fn verify_build_repro_v15(artifact_dir: &Path) -> Result<BuildReproVerifySummaryV15, SdkError> {
    let verify = verify_build_attestation_v15(artifact_dir)?;
    let baseline_raw = fs::read_to_string(&verify.manifest_path)?;
    let baseline_value: JsonValue = serde_json::from_str(&baseline_raw).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-ATTEST-MANIFEST-PARSE: invalid JSON in {} ({err})",
            verify.manifest_path.display()
        ))
    })?;
    let project_root = baseline_value
        .get("project_root")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            SdkError::SupplyInvalid(
                "build_manifest.json missing required field `project_root`".to_string(),
            )
        })?;

    let repro_root = PathBuf::from(project_root);
    let repro_value = build_manifest_value_v15(&repro_root, artifact_dir, true)?;
    let mut baseline_cmp = baseline_value.clone();
    let mut repro_cmp = repro_value.clone();
    strip_repro_exclusions_v15(&mut baseline_cmp);
    strip_repro_exclusions_v15(&mut repro_cmp);
    let baseline_canonical = canonical_json_string_v15(&baseline_cmp)?;
    let repro_canonical = canonical_json_string_v15(&repro_cmp)?;
    let baseline_hash_sha256 = sha256_hex(baseline_canonical.as_bytes());
    let repro_hash_sha256 = sha256_hex(repro_canonical.as_bytes());
    if baseline_hash_sha256 != repro_hash_sha256 {
        return Err(SdkError::SupplyInvalid(format!(
            "X-ATTEST-REPRO-MISMATCH: reproducibility mismatch (baseline={}, repro={}); allowed exclusions are {:?}.",
            baseline_hash_sha256, repro_hash_sha256, REPRO_EXCLUSIONS_V15
        )));
    }

    Ok(BuildReproVerifySummaryV15 {
        artifact_dir: artifact_dir.to_path_buf(),
        baseline_hash_sha256,
        repro_hash_sha256,
        exclusions: REPRO_EXCLUSIONS_V15
            .iter()
            .map(|value| value.to_string())
            .collect(),
    })
}
