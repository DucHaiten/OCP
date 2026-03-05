use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{
    dep_package_id_v10, load_permissions_for_layout, parse_manifest_dependencies,
    parse_project_language_config_v071, parse_requested_permissions_from_package_manifest_v10,
    project_layout, project_package_id_v10, sha256_hex, verify_project_exists,
    PermissionApproveSummaryV15, PermissionDiffSummaryV15, PermissionDoctorFindingV17,
    PermissionDoctorSummaryV17, PermissionFixApplyOptionsV17, PermissionFixApplySummaryV17,
    PermissionFixPlanSummaryV17, PermissionRules, PermissionSnapshotSummaryV15, ProjectPermissions,
    SdkError, StdFsPermissionConfig, StdGamePermissionConfig, StdKvPermissionConfig,
    StdNetHttpPermissionConfig, StdProcPermissionConfig, StdShadowPermissionConfig,
    StdTimePermissionConfig, StdUiPermissionConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionSnapshotPackageRowV15 {
    package_id: String,
    requested_declared: bool,
    requested: ProjectPermissions,
    granted: ProjectPermissions,
    effective: ProjectPermissions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionApprovalEntryV15 {
    diff_hash: String,
    approved_by: String,
    date: String,
    note: String,
}

fn json_string_array_v15(values: &[String]) -> JsonValue {
    JsonValue::Array(
        values
            .iter()
            .map(|v| JsonValue::String(v.clone()))
            .collect(),
    )
}

fn permission_rules_json_v15(rules: &PermissionRules) -> JsonValue {
    let mut allow = rules.allow.clone();
    allow.sort();
    allow.dedup();
    let mut deny = rules.deny.clone();
    deny.sort();
    deny.dedup();
    let mut out = JsonMap::new();
    out.insert("allow".to_string(), json_string_array_v15(&allow));
    out.insert("deny".to_string(), json_string_array_v15(&deny));
    JsonValue::Object(out)
}

fn std_fs_permission_json_v15(cfg: &StdFsPermissionConfig) -> JsonValue {
    let mut read = cfg.read.clone();
    read.sort();
    read.dedup();
    let mut write = cfg.write.clone();
    write.sort();
    write.dedup();
    let mut remove = cfg.remove.clone();
    remove.sort();
    remove.dedup();
    let mut rename = cfg.rename.clone();
    rename.sort();
    rename.dedup();
    let mut list = cfg.list.clone();
    list.sort();
    list.dedup();

    let mut out = JsonMap::new();
    out.insert("list".to_string(), json_string_array_v15(&list));
    out.insert(
        "max_list_entries".to_string(),
        JsonValue::from(cfg.max_list_entries),
    );
    out.insert(
        "max_read_bytes".to_string(),
        JsonValue::from(cfg.max_read_bytes),
    );
    out.insert(
        "max_write_bytes".to_string(),
        JsonValue::from(cfg.max_write_bytes),
    );
    out.insert("read".to_string(), json_string_array_v15(&read));
    out.insert("remove".to_string(), json_string_array_v15(&remove));
    out.insert("rename".to_string(), json_string_array_v15(&rename));
    out.insert("write".to_string(), json_string_array_v15(&write));
    JsonValue::Object(out)
}

fn std_net_http_permission_json_v15(cfg: &StdNetHttpPermissionConfig) -> JsonValue {
    let mut allow_hosts = cfg.allow_hosts.clone();
    allow_hosts.sort();
    allow_hosts.dedup();
    let mut allow_methods = cfg.allow_methods.clone();
    allow_methods.sort();
    allow_methods.dedup();

    let mut out = JsonMap::new();
    out.insert(
        "allow_hosts".to_string(),
        json_string_array_v15(&allow_hosts),
    );
    out.insert(
        "allow_methods".to_string(),
        json_string_array_v15(&allow_methods),
    );
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert(
        "max_body_bytes".to_string(),
        JsonValue::from(cfg.max_body_bytes),
    );
    out.insert("timeout_ms".to_string(), JsonValue::from(cfg.timeout_ms));
    JsonValue::Object(out)
}

fn std_kv_permission_json_v15(cfg: &StdKvPermissionConfig) -> JsonValue {
    let mut out = JsonMap::new();
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert(
        "key_prefix".to_string(),
        cfg.key_prefix
            .as_ref()
            .map(|v| JsonValue::String(v.clone()))
            .unwrap_or(JsonValue::Null),
    );
    out.insert("max_keys".to_string(), JsonValue::from(cfg.max_keys));
    out.insert(
        "max_value_bytes".to_string(),
        JsonValue::from(cfg.max_value_bytes),
    );
    JsonValue::Object(out)
}

fn std_time_permission_json_v15(cfg: &StdTimePermissionConfig) -> JsonValue {
    let mut out = JsonMap::new();
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    JsonValue::Object(out)
}

fn std_proc_permission_json_v15(cfg: &StdProcPermissionConfig) -> JsonValue {
    let mut allow_bins = cfg.allow_bins.clone();
    allow_bins.sort();
    allow_bins.dedup();

    let mut out = JsonMap::new();
    out.insert("allow_bins".to_string(), json_string_array_v15(&allow_bins));
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert(
        "max_stderr_bytes".to_string(),
        JsonValue::from(cfg.max_stderr_bytes),
    );
    out.insert(
        "max_stdout_bytes".to_string(),
        JsonValue::from(cfg.max_stdout_bytes),
    );
    out.insert("timeout_ms".to_string(), JsonValue::from(cfg.timeout_ms));
    JsonValue::Object(out)
}

fn std_game_permission_json_v15(cfg: &StdGamePermissionConfig) -> JsonValue {
    let mut rng_streams = cfg.rng_streams.clone();
    rng_streams.sort();
    rng_streams.dedup();

    let mut out = JsonMap::new();
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert("fixed_dt_ms".to_string(), JsonValue::from(cfg.fixed_dt_ms));
    out.insert(
        "rng_max_count".to_string(),
        JsonValue::from(cfg.rng_max_count),
    );
    out.insert(
        "rng_streams".to_string(),
        json_string_array_v15(&rng_streams),
    );
    out.insert(
        "state_delta_max_bytes".to_string(),
        JsonValue::from(cfg.state_delta_max_bytes),
    );
    JsonValue::Object(out)
}

fn std_shadow_permission_json_v15(cfg: &StdShadowPermissionConfig) -> JsonValue {
    let mut out = JsonMap::new();
    out.insert(
        "branch_budget_cap".to_string(),
        JsonValue::from(cfg.branch_budget_cap),
    );
    out.insert(
        "branch_step_cap".to_string(),
        JsonValue::from(cfg.branch_step_cap),
    );
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert(
        "max_branches".to_string(),
        JsonValue::from(cfg.max_branches),
    );
    out.insert(
        "max_diff_keys".to_string(),
        JsonValue::from(cfg.max_diff_keys),
    );
    out.insert(
        "max_report_bytes".to_string(),
        JsonValue::from(cfg.max_report_bytes),
    );
    JsonValue::Object(out)
}

fn std_ui_permission_json_v15(cfg: &StdUiPermissionConfig) -> JsonValue {
    let mut assets_read = cfg.assets_read.clone();
    assets_read.sort();
    assets_read.dedup();

    let mut out = JsonMap::new();
    out.insert(
        "assets_read".to_string(),
        json_string_array_v15(&assets_read),
    );
    out.insert("enabled".to_string(), JsonValue::Bool(cfg.enabled));
    out.insert(
        "max_asset_bytes".to_string(),
        JsonValue::from(cfg.max_asset_bytes),
    );
    out.insert(
        "max_draw_cmds".to_string(),
        JsonValue::from(cfg.max_draw_cmds),
    );
    out.insert(
        "max_input_events".to_string(),
        JsonValue::from(cfg.max_input_events),
    );
    JsonValue::Object(out)
}

fn project_permissions_json_v15(permissions: &ProjectPermissions) -> JsonValue {
    let mut global_deny = permissions.global_deny.clone();
    global_deny.sort();
    global_deny.dedup();

    let mut modules = BTreeMap::<String, JsonValue>::new();
    for (module, rules) in &permissions.modules {
        modules.insert(module.clone(), permission_rules_json_v15(rules));
    }
    let mut modules_json = JsonMap::new();
    for (module, value) in modules {
        modules_json.insert(module, value);
    }

    let mut out = JsonMap::new();
    out.insert(
        "global_deny".to_string(),
        json_string_array_v15(&global_deny),
    );
    out.insert("modules".to_string(), JsonValue::Object(modules_json));
    out.insert(
        "package".to_string(),
        permissions
            .package
            .as_ref()
            .map(permission_rules_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_fs".to_string(),
        permissions
            .std_fs
            .as_ref()
            .map(std_fs_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_game".to_string(),
        permissions
            .std_game
            .as_ref()
            .map(std_game_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_kv".to_string(),
        permissions
            .std_kv
            .as_ref()
            .map(std_kv_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_net_http".to_string(),
        permissions
            .std_net_http
            .as_ref()
            .map(std_net_http_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_proc".to_string(),
        permissions
            .std_proc
            .as_ref()
            .map(std_proc_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_shadow".to_string(),
        permissions
            .std_shadow
            .as_ref()
            .map(std_shadow_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_time".to_string(),
        permissions
            .std_time
            .as_ref()
            .map(std_time_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    out.insert(
        "std_ui".to_string(),
        permissions
            .std_ui
            .as_ref()
            .map(std_ui_permission_json_v15)
            .unwrap_or(JsonValue::Null),
    );
    JsonValue::Object(out)
}

fn canonicalize_json_value_v15(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = JsonMap::new();
            for key in keys {
                if let Some(v) = map.get(key) {
                    out.insert(key.clone(), canonicalize_json_value_v15(v));
                }
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
            "X-CANONICAL-JSON-SERIALIZE: failed to serialize canonical JSON ({err})"
        ))
    })
}

fn write_canonical_json_file_v15(path: &Path, value: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonical_json_string_v15(value)?;
    fs::write(path, format!("{canonical}\n"))?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn read_json_file_v15(path: &Path) -> Result<JsonValue, SdkError> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str::<JsonValue>(&raw).map_err(|err| {
        SdkError::SupplyInvalid(format!(
            "X-JSON-PARSE-FAILED: failed to parse `{}` ({err})",
            path.display()
        ))
    })
}

fn collect_permission_snapshot_rows_v15(
    root: &Path,
) -> Result<Vec<PermissionSnapshotPackageRowV15>, SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let project_permissions = load_permissions_for_layout(&layout, false)?;
    let project_package_id = project_package_id_v10(&layout)?;

    let manifest_raw = fs::read_to_string(&layout.manifest)?;
    let deps = parse_manifest_dependencies(&manifest_raw)?;

    let mut requested_by_package = HashMap::<String, (bool, ProjectPermissions)>::new();
    requested_by_package.insert(
        project_package_id.clone(),
        (true, project_permissions.clone()),
    );
    for dep in deps {
        if dep.source == "builtin" {
            continue;
        }
        let dep_root = layout.root.join("deps").join(&dep.alias);
        let package_manifest_path = dep_root.join("package.oclp");
        if !package_manifest_path.exists() {
            return Err(SdkError::MissingProject(format!(
                "missing dependency package manifest for permission snapshot: {}",
                package_manifest_path.display()
            )));
        }
        let raw = fs::read_to_string(&package_manifest_path)?;
        let requested = parse_requested_permissions_from_package_manifest_v10(&raw);
        requested_by_package.insert(
            dep_package_id_v10(&dep),
            (requested.has_declared, requested.permissions),
        );
    }

    let effective_by_package =
        super::compute_effective_permissions_by_package_v10(&layout, &project_permissions)?;

    let mut package_ids = BTreeSet::<String>::new();
    for package_id in requested_by_package.keys() {
        package_ids.insert(package_id.clone());
    }
    for package_id in effective_by_package.keys() {
        package_ids.insert(package_id.clone());
    }

    let mut out = Vec::<PermissionSnapshotPackageRowV15>::new();
    for package_id in package_ids {
        let (requested_declared, requested) = requested_by_package
            .get(&package_id)
            .cloned()
            .unwrap_or((false, ProjectPermissions::default()));
        let effective = effective_by_package
            .get(&package_id)
            .cloned()
            .unwrap_or_else(|| project_permissions.clone());
        out.push(PermissionSnapshotPackageRowV15 {
            package_id,
            requested_declared,
            requested,
            granted: project_permissions.clone(),
            effective,
        });
    }
    out.sort_by(|a, b| a.package_id.cmp(&b.package_id));
    Ok(out)
}

fn requested_snapshot_json_v15(rows: &[PermissionSnapshotPackageRowV15]) -> JsonValue {
    let mut packages = Vec::<JsonValue>::new();
    for row in rows {
        let mut package = JsonMap::new();
        package.insert(
            "package_id".to_string(),
            JsonValue::String(row.package_id.clone()),
        );
        package.insert(
            "requested_declared".to_string(),
            JsonValue::Bool(row.requested_declared),
        );
        package.insert(
            "permissions".to_string(),
            project_permissions_json_v15(&row.requested),
        );
        packages.push(JsonValue::Object(package));
    }
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("packages".to_string(), JsonValue::Array(packages));
    JsonValue::Object(out)
}

fn granted_snapshot_json_v15(rows: &[PermissionSnapshotPackageRowV15]) -> JsonValue {
    let mut packages = Vec::<JsonValue>::new();
    for row in rows {
        let mut package = JsonMap::new();
        package.insert(
            "package_id".to_string(),
            JsonValue::String(row.package_id.clone()),
        );
        package.insert(
            "permissions".to_string(),
            project_permissions_json_v15(&row.granted),
        );
        packages.push(JsonValue::Object(package));
    }
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("packages".to_string(), JsonValue::Array(packages));
    JsonValue::Object(out)
}

fn effective_snapshot_json_v15(rows: &[PermissionSnapshotPackageRowV15]) -> JsonValue {
    let mut packages = Vec::<JsonValue>::new();
    for row in rows {
        let mut package = JsonMap::new();
        package.insert(
            "package_id".to_string(),
            JsonValue::String(row.package_id.clone()),
        );
        package.insert(
            "permissions".to_string(),
            project_permissions_json_v15(&row.effective),
        );
        packages.push(JsonValue::Object(package));
    }
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("packages".to_string(), JsonValue::Array(packages));
    JsonValue::Object(out)
}

fn full_snapshot_json_v15(rows: &[PermissionSnapshotPackageRowV15]) -> JsonValue {
    let mut packages = Vec::<JsonValue>::new();
    for row in rows {
        let mut package = JsonMap::new();
        package.insert(
            "package_id".to_string(),
            JsonValue::String(row.package_id.clone()),
        );
        package.insert(
            "requested_declared".to_string(),
            JsonValue::Bool(row.requested_declared),
        );
        package.insert(
            "requested".to_string(),
            project_permissions_json_v15(&row.requested),
        );
        package.insert(
            "granted".to_string(),
            project_permissions_json_v15(&row.granted),
        );
        package.insert(
            "effective".to_string(),
            project_permissions_json_v15(&row.effective),
        );
        packages.push(JsonValue::Object(package));
    }
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("packages".to_string(), JsonValue::Array(packages));
    JsonValue::Object(out)
}

pub fn write_permission_snapshot_v15(
    root: &Path,
    out_dir: Option<&Path>,
) -> Result<PermissionSnapshotSummaryV15, SdkError> {
    let rows = collect_permission_snapshot_rows_v15(root)?;
    let out_root = out_dir.unwrap_or(root);
    fs::create_dir_all(out_root)?;

    let requested_path = out_root.join("permissions_requested.json");
    let granted_path = out_root.join("permissions_granted.json");
    let effective_path = out_root.join("permissions_effective.json");
    let snapshot_path = out_root.join("permissions.snapshot.json");

    let requested = requested_snapshot_json_v15(&rows);
    let granted = granted_snapshot_json_v15(&rows);
    let effective = effective_snapshot_json_v15(&rows);
    let snapshot = full_snapshot_json_v15(&rows);

    let _ = write_canonical_json_file_v15(&requested_path, &requested)?;
    let _ = write_canonical_json_file_v15(&granted_path, &granted)?;
    let _ = write_canonical_json_file_v15(&effective_path, &effective)?;
    let snapshot_hash_sha256 = write_canonical_json_file_v15(&snapshot_path, &snapshot)?;

    Ok(PermissionSnapshotSummaryV15 {
        requested_path,
        granted_path,
        effective_path,
        snapshot_path,
        snapshot_hash_sha256,
        packages: rows.len(),
    })
}

fn flatten_json_leaf_paths_v15(
    value: &JsonValue,
    prefix: &str,
    out: &mut BTreeMap<String, JsonValue>,
) {
    match value {
        JsonValue::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(next) = map.get(key) {
                    let child = if prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    flatten_json_leaf_paths_v15(next, &child, out);
                }
            }
        }
        JsonValue::Array(items) => {
            for (idx, next) in items.iter().enumerate() {
                let child = format!("{prefix}[{idx}]");
                flatten_json_leaf_paths_v15(next, &child, out);
            }
        }
        _ => {
            out.insert(prefix.to_string(), value.clone());
        }
    }
}

fn parse_permission_approval_entries_v15(raw: &str) -> Vec<PermissionApprovalEntryV15> {
    let mut out = Vec::<PermissionApprovalEntryV15>::new();
    let mut in_approval = false;
    let mut current = PermissionApprovalEntryV15 {
        diff_hash: String::new(),
        approved_by: String::new(),
        date: String::new(),
        note: String::new(),
    };

    let flush_current = |entries: &mut Vec<PermissionApprovalEntryV15>,
                         row: &mut PermissionApprovalEntryV15| {
        if row.diff_hash.trim().is_empty() {
            return;
        }
        entries.push(row.clone());
        row.diff_hash.clear();
        row.approved_by.clear();
        row.date.clear();
        row.note.clear();
    };

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[approval]]" {
            if in_approval {
                flush_current(&mut out, &mut current);
            }
            in_approval = true;
            continue;
        }
        if !in_approval {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "diff_hash" => current.diff_hash = value,
            "approved_by" => current.approved_by = value,
            "date" => current.date = value,
            "note" => current.note = value,
            _ => {}
        }
    }
    if in_approval {
        flush_current(&mut out, &mut current);
    }

    out.sort_by(|a, b| {
        a.diff_hash
            .cmp(&b.diff_hash)
            .then(a.date.cmp(&b.date))
            .then(a.approved_by.cmp(&b.approved_by))
    });
    out.dedup_by(|a, b| a.diff_hash == b.diff_hash);
    out
}

fn read_permission_approval_entries_v15(
    path: &Path,
) -> Result<Vec<PermissionApprovalEntryV15>, SdkError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(parse_permission_approval_entries_v15(&raw))
}

fn write_permission_approval_entries_v15(
    path: &Path,
    entries: &[PermissionApprovalEntryV15],
) -> Result<(), SdkError> {
    let mut rows = entries.to_vec();
    rows.sort_by(|a, b| {
        a.diff_hash
            .cmp(&b.diff_hash)
            .then(a.date.cmp(&b.date))
            .then(a.approved_by.cmp(&b.approved_by))
    });
    rows.dedup_by(|a, b| a.diff_hash == b.diff_hash);

    let mut out = String::new();
    out.push_str("version = 1\n");
    out.push_str("hasher_version = \"sha256-v1\"\n");
    for row in rows {
        out.push_str("\n[[approval]]\n");
        out.push_str("diff_hash = \"");
        out.push_str(&row.diff_hash.replace('"', ""));
        out.push_str("\"\n");
        out.push_str("approved_by = \"");
        out.push_str(&row.approved_by.replace('"', ""));
        out.push_str("\"\n");
        out.push_str("date = \"");
        out.push_str(&row.date.replace('"', ""));
        out.push_str("\"\n");
        out.push_str("note = \"");
        out.push_str(&row.note.replace('"', "'").replace(['\n', '\r'], " "));
        out.push_str("\"\n");
    }
    fs::write(path, out)?;
    Ok(())
}

fn permission_diff_report_value_v15(
    old_value: &JsonValue,
    new_value: &JsonValue,
    old_path: &Path,
    new_path: &Path,
) -> Result<(JsonValue, String, bool, bool), SdkError> {
    let mut old_leaves = BTreeMap::<String, JsonValue>::new();
    let mut new_leaves = BTreeMap::<String, JsonValue>::new();
    flatten_json_leaf_paths_v15(old_value, "", &mut old_leaves);
    flatten_json_leaf_paths_v15(new_value, "", &mut new_leaves);

    let mut added_paths = Vec::<String>::new();
    let mut removed_paths = Vec::<String>::new();
    let mut changed = Vec::<JsonValue>::new();
    let mut cap_increases = Vec::<JsonValue>::new();
    let mut new_paths_added = Vec::<String>::new();
    let mut quarantine_permission_changes = Vec::<String>::new();
    let mut new_capabilities_requested = Vec::<String>::new();

    for (path, new_leaf) in &new_leaves {
        let Some(old_leaf) = old_leaves.get(path) else {
            added_paths.push(path.clone());
            if path.contains(".effective.std_fs.") {
                new_paths_added.push(path.clone());
            }
            if path.contains(".effective.std_net_http.")
                || path.contains(".effective.std_proc.")
                || path.contains(".effective.std_time.")
            {
                quarantine_permission_changes.push(path.clone());
            }
            if path.contains(".requested.") || path.contains(".effective.") {
                new_capabilities_requested.push(path.clone());
            }
            if path.contains(".effective.") && matches!(new_leaf, JsonValue::Bool(true)) {
                new_capabilities_requested.push(path.clone());
            }
            continue;
        };
        if old_leaf == new_leaf {
            continue;
        }
        let old_rendered = canonical_json_string_v15(old_leaf)?;
        let new_rendered = canonical_json_string_v15(new_leaf)?;
        let mut row = JsonMap::new();
        row.insert("path".to_string(), JsonValue::String(path.clone()));
        row.insert("old".to_string(), JsonValue::String(old_rendered));
        row.insert("new".to_string(), JsonValue::String(new_rendered));
        changed.push(JsonValue::Object(row));

        if path.contains(".effective.") {
            if let (Some(old_num), Some(new_num)) = (old_leaf.as_f64(), new_leaf.as_f64()) {
                if new_num > old_num {
                    let mut cap = JsonMap::new();
                    cap.insert("path".to_string(), JsonValue::String(path.clone()));
                    cap.insert("old".to_string(), JsonValue::from(old_num));
                    cap.insert("new".to_string(), JsonValue::from(new_num));
                    cap_increases.push(JsonValue::Object(cap));
                }
            }
            if matches!(old_leaf, JsonValue::Bool(false))
                && matches!(new_leaf, JsonValue::Bool(true))
            {
                new_capabilities_requested.push(path.clone());
            }
        }

        if path.contains(".effective.std_net_http.")
            || path.contains(".effective.std_proc.")
            || path.contains(".effective.std_time.")
        {
            quarantine_permission_changes.push(path.clone());
        }
        if path.contains(".requested.") || path.contains(".effective.") {
            new_capabilities_requested.push(path.clone());
        }
    }

    for path in old_leaves.keys() {
        if !new_leaves.contains_key(path) {
            removed_paths.push(path.clone());
            if path.contains(".effective.std_net_http.")
                || path.contains(".effective.std_proc.")
                || path.contains(".effective.std_time.")
            {
                quarantine_permission_changes.push(path.clone());
            }
        }
    }

    added_paths.sort();
    added_paths.dedup();
    removed_paths.sort();
    removed_paths.dedup();
    new_paths_added.sort();
    new_paths_added.dedup();
    quarantine_permission_changes.sort();
    quarantine_permission_changes.dedup();
    new_capabilities_requested.sort();
    new_capabilities_requested.dedup();
    changed.sort_by(|a, b| {
        let pa = a
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let pb = b
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        pa.cmp(pb)
    });
    cap_increases.sort_by(|a, b| {
        let pa = a
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let pb = b
            .get("path")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        pa.cmp(pb)
    });

    let has_changes = !added_paths.is_empty() || !removed_paths.is_empty() || !changed.is_empty();
    let introduces_new_permissions = !new_capabilities_requested.is_empty()
        || !cap_increases.is_empty()
        || !new_paths_added.is_empty();

    let mut report = JsonMap::new();
    report.insert("schema_version".to_string(), JsonValue::from(1u64));
    report.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    report.insert(
        "old_path".to_string(),
        JsonValue::String(old_path.to_string_lossy().replace('\\', "/")),
    );
    report.insert(
        "new_path".to_string(),
        JsonValue::String(new_path.to_string_lossy().replace('\\', "/")),
    );
    report.insert("has_changes".to_string(), JsonValue::Bool(has_changes));
    report.insert(
        "introduces_new_permissions".to_string(),
        JsonValue::Bool(introduces_new_permissions),
    );
    report.insert(
        "new_capabilities_requested".to_string(),
        json_string_array_v15(&new_capabilities_requested),
    );
    report.insert(
        "added_paths".to_string(),
        json_string_array_v15(&added_paths),
    );
    report.insert(
        "removed_paths".to_string(),
        json_string_array_v15(&removed_paths),
    );
    report.insert("changed_values".to_string(), JsonValue::Array(changed));
    report.insert("cap_increases".to_string(), JsonValue::Array(cap_increases));
    report.insert(
        "new_paths_added".to_string(),
        json_string_array_v15(&new_paths_added),
    );
    report.insert(
        "quarantine_permission_changes".to_string(),
        json_string_array_v15(&quarantine_permission_changes),
    );

    let report_value = JsonValue::Object(report);
    let report_canonical = canonical_json_string_v15(&report_value)?;
    let diff_hash = sha256_hex(report_canonical.as_bytes());

    let mut wrapper = JsonMap::new();
    wrapper.insert("schema_version".to_string(), JsonValue::from(1u64));
    wrapper.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    wrapper.insert(
        "permission_diff_hash".to_string(),
        JsonValue::String(diff_hash.clone()),
    );
    wrapper.insert("report".to_string(), report_value);
    Ok((
        JsonValue::Object(wrapper),
        diff_hash,
        has_changes,
        introduces_new_permissions,
    ))
}

fn extract_permission_diff_hash_v15(report: &JsonValue) -> Result<String, SdkError> {
    let Some(diff_hash) = report
        .get("permission_diff_hash")
        .and_then(JsonValue::as_str)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    else {
        return Err(SdkError::SupplyInvalid(
            "permission diff report missing `permission_diff_hash`".to_string(),
        ));
    };
    Ok(diff_hash.to_string())
}

pub fn write_permission_diff_report_v15(
    old_path: &Path,
    new_path: &Path,
    report_path: &Path,
    approval_path: Option<&Path>,
) -> Result<PermissionDiffSummaryV15, SdkError> {
    let old_value = read_json_file_v15(old_path)?;
    let new_value = read_json_file_v15(new_path)?;
    let (report_value, permission_diff_hash, has_changes, introduces_new_permissions) =
        permission_diff_report_value_v15(&old_value, &new_value, old_path, new_path)?;
    let _ = write_canonical_json_file_v15(report_path, &report_value)?;

    let mut approval_checked = false;
    let mut approved = false;
    if let Some(approval_file) = approval_path {
        approval_checked = true;
        let approvals = read_permission_approval_entries_v15(approval_file)?;
        approved = approvals
            .iter()
            .any(|entry| entry.diff_hash == permission_diff_hash);
        if introduces_new_permissions && !approved {
            return Err(SdkError::PermissionDenied(format!(
                "X-PERMISSION-APPROVAL-REQUIRED: permission diff `{}` is not approved in `{}`; reason=`RC-PERMISSION-UNAPPROVED`. Run `ocl perm approve {}` first.",
                permission_diff_hash,
                approval_file.display(),
                report_path.display()
            )));
        }
    }

    Ok(PermissionDiffSummaryV15 {
        report_path: report_path.to_path_buf(),
        permission_diff_hash,
        has_changes,
        introduces_new_permissions,
        approval_checked,
        approved,
    })
}

pub fn approve_permission_diff_v15(
    diff_report_path: &Path,
    approval_path: &Path,
    approved_by: &str,
    date: &str,
    note: Option<&str>,
) -> Result<PermissionApproveSummaryV15, SdkError> {
    let approved_by = approved_by.trim();
    if approved_by.is_empty() {
        return Err(SdkError::SupplyInvalid(
            "approved_by must not be empty".to_string(),
        ));
    }
    let date = date.trim();
    if date.is_empty() {
        return Err(SdkError::SupplyInvalid(
            "date must not be empty".to_string(),
        ));
    }

    let report = read_json_file_v15(diff_report_path)?;
    let diff_hash = extract_permission_diff_hash_v15(&report)?;
    let mut approvals = read_permission_approval_entries_v15(approval_path)?;
    if !approvals.iter().any(|entry| entry.diff_hash == diff_hash) {
        approvals.push(PermissionApprovalEntryV15 {
            diff_hash: diff_hash.clone(),
            approved_by: approved_by.to_string(),
            date: date.to_string(),
            note: note.unwrap_or_default().to_string(),
        });
        write_permission_approval_entries_v15(approval_path, &approvals)?;
    }
    let approvals_total = read_permission_approval_entries_v15(approval_path)?.len();
    Ok(PermissionApproveSummaryV15 {
        approval_path: approval_path.to_path_buf(),
        diff_hash,
        approvals_total,
    })
}

pub(crate) fn enforce_permission_review_policy_v15(root: &Path) -> Result<(), SdkError> {
    let layout = project_layout(root);
    if !layout.manifest.exists() {
        return Ok(());
    }
    let manifest_text = fs::read_to_string(&layout.manifest)?;
    let language_cfg = parse_project_language_config_v071(&manifest_text);
    if language_cfg.lane.trim() != "locked_v071" {
        return Ok(());
    }
    let baseline = layout.root.join("permissions.snapshot.json");
    if !baseline.exists() {
        return Ok(());
    }
    let approval = layout.root.join("permissions.approval.toml");
    let old_value = read_json_file_v15(&baseline)?;
    let rows = collect_permission_snapshot_rows_v15(root)?;
    let new_value = full_snapshot_json_v15(&rows);
    let (_, diff_hash, _, introduces_new_permissions) =
        permission_diff_report_value_v15(&old_value, &new_value, &baseline, &baseline)?;
    if !introduces_new_permissions {
        return Ok(());
    }
    let approvals = read_permission_approval_entries_v15(&approval)?;
    let approved = approvals.iter().any(|entry| entry.diff_hash == diff_hash);
    if !approved {
        return Err(SdkError::PermissionDenied(format!(
            "X-PERMISSION-APPROVAL-REQUIRED: permission diff `{}` is not approved in `{}`; reason=`RC-PERMISSION-UNAPPROVED`. Run `ocl perm snapshot`, `ocl perm diff`, and `ocl perm approve` first.",
            diff_hash,
            approval.display()
        )));
    }
    Ok(())
}

const PERM_WILDCARD_GUARD_CODE_V17: &str = "X-PERM-RUBBERSTAMP-GUARD";
const PERM_WILDCARD_GUARD_REASON_V17: &str = "RC-PERM-WILDCARD-DENIED";
const PERM_WILDCARD_GUARD_ALIAS_V17: &str = "X-PERMISSION-REVIEW-REQUIRED";
const PERM_RISK_ACK_CODE_V17: &str = "X-PERMISSION-RISK-ACK-REQUIRED";
const PERM_RISK_ACK_REASON_V17: &str = "RC-PERMISSION-RISK-ACK-REQUIRED";

fn default_w17_dx_root_v17(root: &Path) -> PathBuf {
    root.join("target").join("ocl").join("w17").join("dx")
}

fn wildcard_risk_hint_v17() -> String {
    "Run `ocl perm fix --plan <project_dir>` and replace wildcard with explicit scoped allowlist."
        .to_string()
}

fn looks_like_wildcard_risk_v17(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    trimmed.contains('*') || lower == "0.0.0.0/0" || lower == "::/0"
}

fn push_wildcard_findings_v17(
    values: &[String],
    location_prefix: &str,
    lane: &str,
    findings: &mut Vec<PermissionDoctorFindingV17>,
) {
    let severity = if lane == "locked_v071" {
        "error"
    } else {
        "warn"
    };
    for (idx, value) in values.iter().enumerate() {
        if !looks_like_wildcard_risk_v17(value) {
            continue;
        }
        findings.push(PermissionDoctorFindingV17 {
            code: PERM_WILDCARD_GUARD_CODE_V17.to_string(),
            reason: PERM_WILDCARD_GUARD_REASON_V17.to_string(),
            severity: severity.to_string(),
            location: format!("{location_prefix}[{idx}]"),
            value: value.clone(),
            hint: wildcard_risk_hint_v17(),
        });
    }
}

fn collect_permission_findings_v17(
    root: &Path,
) -> Result<(String, Vec<PermissionDoctorFindingV17>), SdkError> {
    let layout = project_layout(root);
    verify_project_exists(&layout)?;
    let manifest_text = fs::read_to_string(&layout.manifest)?;
    let language_cfg = parse_project_language_config_v071(&manifest_text);
    let lane = language_cfg.lane.trim().to_string();
    let permissions = load_permissions_for_layout(&layout, false)?;

    let mut findings = Vec::<PermissionDoctorFindingV17>::new();

    if let Some(package) = permissions.package.as_ref() {
        push_wildcard_findings_v17(
            &package.allow,
            "permissions.package.allow",
            &lane,
            &mut findings,
        );
    }
    for (module_id, rules) in &permissions.modules {
        push_wildcard_findings_v17(
            &rules.allow,
            &format!("permissions.module.{module_id}.allow"),
            &lane,
            &mut findings,
        );
    }
    if let Some(std_fs) = permissions.std_fs.as_ref() {
        push_wildcard_findings_v17(
            &std_fs.read,
            "permissions.std_fs.read",
            &lane,
            &mut findings,
        );
        push_wildcard_findings_v17(
            &std_fs.write,
            "permissions.std_fs.write",
            &lane,
            &mut findings,
        );
        push_wildcard_findings_v17(
            &std_fs.remove,
            "permissions.std_fs.remove",
            &lane,
            &mut findings,
        );
        push_wildcard_findings_v17(
            &std_fs.rename,
            "permissions.std_fs.rename",
            &lane,
            &mut findings,
        );
        push_wildcard_findings_v17(
            &std_fs.list,
            "permissions.std_fs.list",
            &lane,
            &mut findings,
        );
    }
    if let Some(std_net_http) = permissions.std_net_http.as_ref() {
        push_wildcard_findings_v17(
            &std_net_http.allow_hosts,
            "permissions.std_net_http.allow_hosts",
            &lane,
            &mut findings,
        );
        push_wildcard_findings_v17(
            &std_net_http.allow_methods,
            "permissions.std_net_http.allow_methods",
            &lane,
            &mut findings,
        );
    }
    if let Some(std_proc) = permissions.std_proc.as_ref() {
        push_wildcard_findings_v17(
            &std_proc.allow_bins,
            "permissions.std_proc.allow_bins",
            &lane,
            &mut findings,
        );
    }
    if let Some(std_ui) = permissions.std_ui.as_ref() {
        push_wildcard_findings_v17(
            &std_ui.assets_read,
            "permissions.std_ui.assets_read",
            &lane,
            &mut findings,
        );
    }
    findings.sort_by(|a, b| {
        a.location
            .cmp(&b.location)
            .then(a.value.cmp(&b.value))
            .then(a.code.cmp(&b.code))
    });
    findings.dedup_by(|a, b| {
        a.location == b.location && a.value == b.value && a.code == b.code && a.reason == b.reason
    });
    Ok((lane, findings))
}

fn doctor_report_json_v17(
    lane: &str,
    findings: &[PermissionDoctorFindingV17],
    report_path: &Path,
) -> JsonValue {
    let blocking_total = findings.iter().filter(|f| f.severity == "error").count();
    let risk_score = (findings.len().min(10) as u32) * 10;
    let findings_json = findings
        .iter()
        .map(|finding| {
            let mut row = JsonMap::new();
            row.insert("code".to_string(), JsonValue::String(finding.code.clone()));
            row.insert(
                "reason".to_string(),
                JsonValue::String(finding.reason.clone()),
            );
            row.insert(
                "severity".to_string(),
                JsonValue::String(finding.severity.clone()),
            );
            row.insert(
                "location".to_string(),
                JsonValue::String(finding.location.clone()),
            );
            row.insert(
                "value".to_string(),
                JsonValue::String(finding.value.clone()),
            );
            row.insert("hint".to_string(), JsonValue::String(finding.hint.clone()));
            JsonValue::Object(row)
        })
        .collect::<Vec<JsonValue>>();
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("lane".to_string(), JsonValue::String(lane.to_string()));
    out.insert(
        "report_path".to_string(),
        JsonValue::String(report_path.to_string_lossy().replace('\\', "/")),
    );
    out.insert(
        "findings_total".to_string(),
        JsonValue::from(findings.len() as u64),
    );
    out.insert(
        "blocking_total".to_string(),
        JsonValue::from(blocking_total as u64),
    );
    out.insert("risk_score".to_string(), JsonValue::from(risk_score));
    out.insert("findings".to_string(), JsonValue::Array(findings_json));
    JsonValue::Object(out)
}

pub fn write_permission_doctor_report_v17(
    root: &Path,
    out_path: Option<&Path>,
) -> Result<PermissionDoctorSummaryV17, SdkError> {
    let (lane, findings) = collect_permission_findings_v17(root)?;
    let out_path = out_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_w17_dx_root_v17(root).join("dx_friction_report.json"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let report_json = doctor_report_json_v17(&lane, &findings, &out_path);
    let _ = write_canonical_json_file_v15(&out_path, &report_json)?;

    let blocking_total = findings.iter().filter(|f| f.severity == "error").count();
    let risk_score = (findings.len().min(10) as u32) * 10;
    if lane == "locked_v071" && blocking_total > 0 {
        return Err(SdkError::PermissionDenied(format!(
            "{PERM_WILDCARD_GUARD_CODE_V17}: wildcard permissions are blocked in locked_v071; reason=`{PERM_WILDCARD_GUARD_REASON_V17}`; alias=`{PERM_WILDCARD_GUARD_ALIAS_V17}`; severity=`error`; hint=`{}`; report=`{}`",
            wildcard_risk_hint_v17(),
            out_path.display()
        )));
    }
    Ok(PermissionDoctorSummaryV17 {
        report_path: out_path,
        lane,
        findings_total: findings.len(),
        blocking_total,
        risk_score,
    })
}

fn recommended_value_for_location_v17(location: &str) -> String {
    if location.contains("permissions.std_fs.read") || location.contains("permissions.std_fs.list")
    {
        "./data/**".to_string()
    } else if location.contains("permissions.std_fs.write")
        || location.contains("permissions.std_fs.remove")
        || location.contains("permissions.std_fs.rename")
    {
        "./out/**".to_string()
    } else if location.contains("permissions.std_net_http.allow_hosts") {
        "api.example.com".to_string()
    } else if location.contains("permissions.std_net_http.allow_methods") {
        "GET".to_string()
    } else if location.contains("permissions.std_proc.allow_bins") {
        "tool.exe".to_string()
    } else if location.contains("permissions.std_ui.assets_read") {
        "./assets/**".to_string()
    } else if location.contains("permissions.package.allow")
        || location.contains("permissions.module.")
    {
        "std.log.info".to_string()
    } else {
        "<explicit-scope>".to_string()
    }
}

fn fix_plan_json_v17(lane: &str, findings: &[PermissionDoctorFindingV17]) -> JsonValue {
    let suggestions = findings
        .iter()
        .map(|finding| {
            let mut row = JsonMap::new();
            row.insert(
                "location".to_string(),
                JsonValue::String(finding.location.clone()),
            );
            row.insert(
                "current".to_string(),
                JsonValue::String(finding.value.clone()),
            );
            row.insert(
                "recommended".to_string(),
                JsonValue::String(recommended_value_for_location_v17(&finding.location)),
            );
            row.insert("hint".to_string(), JsonValue::String(finding.hint.clone()));
            JsonValue::Object(row)
        })
        .collect::<Vec<JsonValue>>();
    let mut out = JsonMap::new();
    out.insert("schema_version".to_string(), JsonValue::from(1u64));
    out.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    out.insert("lane".to_string(), JsonValue::String(lane.to_string()));
    out.insert(
        "findings_total".to_string(),
        JsonValue::from(findings.len() as u64),
    );
    out.insert("suggestions".to_string(), JsonValue::Array(suggestions));
    JsonValue::Object(out)
}

pub fn write_permission_fix_plan_v17(
    root: &Path,
    out_path: Option<&Path>,
) -> Result<PermissionFixPlanSummaryV17, SdkError> {
    let (lane, findings) = collect_permission_findings_v17(root)?;
    let out_path = out_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_w17_dx_root_v17(root).join("permission_fix_plan.json"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let plan_json = fix_plan_json_v17(&lane, &findings);
    let plan_hash_sha256 = write_canonical_json_file_v15(&out_path, &plan_json)?;
    Ok(PermissionFixPlanSummaryV17 {
        plan_path: out_path,
        plan_hash_sha256,
        lane,
        findings_total: findings.len(),
    })
}

fn plan_hash_from_json_v17(plan_json: &JsonValue) -> Result<String, SdkError> {
    let canonical = canonical_json_string_v15(plan_json)?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn extract_string_field_v17(value: &JsonValue, path: &str) -> Option<String> {
    value
        .pointer(path)
        .and_then(JsonValue::as_str)
        .map(ToString::to_string)
}

pub fn apply_permission_fix_plan_v17(
    root: &Path,
    options: &PermissionFixApplyOptionsV17,
) -> Result<PermissionFixApplySummaryV17, SdkError> {
    if !options.ack_risk {
        return Err(SdkError::PermissionDenied(format!(
            "{PERM_RISK_ACK_CODE_V17}: missing `--ack-risk`; reason=`{PERM_RISK_ACK_REASON_V17}`; alias=`X-PERMISSION-APPROVAL-REQUIRED`; severity=`error`; hint=`rerun with --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>`."
        )));
    }
    let justification = options.justification.trim();
    if justification.is_empty() {
        return Err(SdkError::PermissionDenied(format!(
            "{PERM_RISK_ACK_CODE_V17}: empty justification; reason=`{PERM_RISK_ACK_REASON_V17}`; alias=`X-PERMISSION-APPROVAL-REQUIRED`; severity=`error`; hint=`provide --justification with concrete risk rationale`."
        )));
    }
    if options.approved_by.trim().is_empty() || options.date.trim().is_empty() {
        return Err(SdkError::SupplyInvalid(
            "perm fix --apply requires non-empty --by and --date".to_string(),
        ));
    }

    let dx_root = default_w17_dx_root_v17(root);
    fs::create_dir_all(&dx_root)?;
    let plan_path = options
        .plan_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| dx_root.join("permission_fix_plan.json"));
    if !plan_path.exists() {
        write_permission_fix_plan_v17(root, Some(&plan_path))?;
    }
    let plan_json = read_json_file_v15(&plan_path)?;
    let plan_hash_sha256 = plan_hash_from_json_v17(&plan_json)?;
    let lane =
        extract_string_field_v17(&plan_json, "/lane").unwrap_or_else(|| "unknown".to_string());
    let findings_total = plan_json
        .pointer("/findings_total")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0) as usize;

    let patch_path = options
        .patch_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| dx_root.join("permission_fix.patch.toml"));
    if let Some(parent) = patch_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut patch = String::new();
    patch.push_str("# OCL v0.17 permission fix patch (advisory-only)\n");
    patch.push_str("# This file does not auto-apply permissions.\n");
    patch.push_str("# Review and patch Ocl.toml manually.\n");
    patch.push_str("# plan_hash = \"");
    patch.push_str(&plan_hash_sha256);
    patch.push_str("\"\n");
    patch.push_str("# lane = \"");
    patch.push_str(&lane);
    patch.push_str("\"\n");
    patch.push_str("# justification = \"");
    patch.push_str(&justification.replace('"', "'").replace(['\n', '\r'], " "));
    patch.push_str("\"\n");
    if let Some(suggestions) = plan_json.get("suggestions").and_then(JsonValue::as_array) {
        for suggestion in suggestions {
            let location = suggestion
                .get("location")
                .and_then(JsonValue::as_str)
                .unwrap_or("<unknown>");
            let current = suggestion
                .get("current")
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            let recommended = suggestion
                .get("recommended")
                .and_then(JsonValue::as_str)
                .unwrap_or("<explicit-scope>");
            patch.push_str("\n# location: ");
            patch.push_str(location);
            patch.push_str("\n# current: \"");
            patch.push_str(&current.replace('"', "'"));
            patch.push_str("\"\n# recommended: \"");
            patch.push_str(&recommended.replace('"', "'"));
            patch.push_str("\"\n");
        }
    }
    fs::write(&patch_path, patch)?;

    let approval_path = options
        .approval_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("permissions.approval.toml"));
    let virtual_diff_path = dx_root.join("permission_fix_virtual_diff.json");
    let mut virtual_report = JsonMap::new();
    virtual_report.insert("schema_version".to_string(), JsonValue::from(1u64));
    virtual_report.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    virtual_report.insert(
        "permission_diff_hash".to_string(),
        JsonValue::String(plan_hash_sha256.clone()),
    );
    let mut payload = JsonMap::new();
    payload.insert(
        "plan_path".to_string(),
        JsonValue::String(plan_path.to_string_lossy().replace('\\', "/")),
    );
    payload.insert(
        "note".to_string(),
        JsonValue::String("virtual diff for perm fix apply".to_string()),
    );
    virtual_report.insert("report".to_string(), JsonValue::Object(payload));
    let _ = write_canonical_json_file_v15(&virtual_diff_path, &JsonValue::Object(virtual_report))?;
    let _ = approve_permission_diff_v15(
        &virtual_diff_path,
        &approval_path,
        options.approved_by.trim(),
        options.date.trim(),
        Some(&format!("perm fix apply: {justification}")),
    )?;

    let report_path = options
        .report_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| dx_root.join("permission_fix_safety_report.json"));
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut report = JsonMap::new();
    report.insert("schema_version".to_string(), JsonValue::from(1u64));
    report.insert(
        "hasher_version".to_string(),
        JsonValue::String("sha256-v1".to_string()),
    );
    report.insert("lane".to_string(), JsonValue::String(lane));
    report.insert(
        "plan_path".to_string(),
        JsonValue::String(plan_path.to_string_lossy().replace('\\', "/")),
    );
    report.insert(
        "patch_path".to_string(),
        JsonValue::String(patch_path.to_string_lossy().replace('\\', "/")),
    );
    report.insert(
        "approval_path".to_string(),
        JsonValue::String(approval_path.to_string_lossy().replace('\\', "/")),
    );
    report.insert(
        "plan_hash_sha256".to_string(),
        JsonValue::String(plan_hash_sha256.clone()),
    );
    report.insert(
        "findings_total".to_string(),
        JsonValue::from(findings_total as u64),
    );
    report.insert("ack_risk".to_string(), JsonValue::Bool(options.ack_risk));
    report.insert(
        "approved_by".to_string(),
        JsonValue::String(options.approved_by.trim().to_string()),
    );
    report.insert(
        "date".to_string(),
        JsonValue::String(options.date.trim().to_string()),
    );
    report.insert(
        "justification".to_string(),
        JsonValue::String(justification.to_string()),
    );
    let _ = write_canonical_json_file_v15(&report_path, &JsonValue::Object(report))?;

    Ok(PermissionFixApplySummaryV17 {
        plan_path,
        patch_path,
        report_path,
        approval_path,
        plan_hash_sha256,
        findings_total,
    })
}
