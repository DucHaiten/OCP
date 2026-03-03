use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::w2::resolve_domain_selection_v1;
use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewProfileV1 {
    pub id: String,
    pub universe_id: String,
    pub domain_id: String,
    pub renderer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSelectionV1 {
    pub universe_id: String,
    pub domain_id: String,
    pub view_id: String,
    pub renderer: String,
    pub is_legacy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewKitBindingV1 {
    pub kit_id: String,
    pub universe_id: String,
    pub bind_domain: String,
    pub bind_view: String,
    pub required_organs: Vec<String>,
}

pub fn resolve_view_kit_bindings_v1(
    root: &Path,
    selection: &ViewSelectionV1,
) -> Result<Vec<ViewKitBindingV1>, SdkError> {
    if selection.is_legacy {
        return Ok(Vec::new());
    }

    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Err(SdkError::LockMismatch(
            "V-VIEW-NO-COSMOS: `--view` requires cosmos.toml".to_string(),
        ));
    }

    let all = parse_cosmos_kits_v1(&cosmos_path)?;
    Ok(all
        .into_iter()
        .filter(|kit| {
            kit.universe_id == selection.universe_id
                && kit.bind_domain == selection.domain_id
                && kit.bind_view == selection.view_id
        })
        .collect())
}

pub fn resolve_view_observe_key_v1(
    root: &Path,
    selection: &ViewSelectionV1,
) -> Result<String, SdkError> {
    let kits = resolve_view_kit_bindings_v1(root, selection)?;
    if kits
        .iter()
        .any(|kit| kit.kit_id == "std.kit.view_text_basic")
    {
        return Ok("std.view.render_text".to_string());
    }

    match selection.renderer.as_str() {
        "text" => Ok("std.view.render_text".to_string()),
        "tree" => Ok("std.view.render_tree".to_string()),
        other => Err(SdkError::LockMismatch(format!(
            "V-VIEW-RENDERER-INVALID: unsupported renderer `{other}` for view `{}`",
            selection.view_id
        ))),
    }
}

pub fn resolve_view_selection_v1(
    root: &Path,
    locked: bool,
    universe_id: Option<&str>,
    domain_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<ViewSelectionV1, SdkError> {
    let domain = resolve_domain_selection_v1(root, locked, universe_id, domain_id)?;
    if domain.is_legacy {
        if view_id.is_some() {
            return Err(SdkError::LockMismatch(
                "V-VIEW-NO-COSMOS: `--view` requires cosmos.toml".to_string(),
            ));
        }
        return Ok(ViewSelectionV1 {
            universe_id: "__legacy__".to_string(),
            domain_id: "default".to_string(),
            view_id: "default".to_string(),
            renderer: "text".to_string(),
            is_legacy: true,
        });
    }

    let cosmos_path = root.join("cosmos.toml");
    if !cosmos_path.exists() {
        return Err(SdkError::LockMismatch(
            "V-VIEW-NO-COSMOS: `--view` requires cosmos.toml".to_string(),
        ));
    }
    let views = parse_cosmos_views_v1(&cosmos_path)?;

    if let Some(requested_view) = view_id {
        let universe_matches: Vec<&ViewProfileV1> = views
            .iter()
            .filter(|view| view.universe_id == domain.universe_id && view.id == requested_view)
            .collect();

        if domain_id.is_none() {
            let distinct_domain_count = universe_matches
                .iter()
                .map(|view| view.domain_id.clone())
                .collect::<HashSet<String>>()
                .len();
            if distinct_domain_count > 1 {
                return Err(SdkError::LockMismatch(format!(
                    "V-VIEW-DOMAIN-AMBIG: view `{requested_view}` exists in multiple domains of universe `{}`; pass `--domain <id>`",
                    domain.universe_id
                )));
            }
        }

        if let Some(selected) = universe_matches
            .iter()
            .find(|view| view.domain_id == domain.domain_id)
        {
            return Ok(ViewSelectionV1 {
                universe_id: selected.universe_id.clone(),
                domain_id: selected.domain_id.clone(),
                view_id: selected.id.clone(),
                renderer: selected.renderer.clone(),
                is_legacy: false,
            });
        }

        if !universe_matches.is_empty() {
            return Err(SdkError::LockMismatch(format!(
                "V-VIEW-DOMAIN-MISMATCH: view `{requested_view}` is not wired for domain `{}` in universe `{}`",
                domain.domain_id, domain.universe_id
            )));
        }

        return Err(SdkError::LockMismatch(format!(
            "V-VIEW-NOT-FOUND: view `{requested_view}` is not defined for universe `{}`",
            domain.universe_id
        )));
    }

    let scoped_views: Vec<&ViewProfileV1> = views
        .iter()
        .filter(|view| view.universe_id == domain.universe_id && view.domain_id == domain.domain_id)
        .collect();

    if scoped_views.is_empty() {
        return Ok(ViewSelectionV1 {
            universe_id: domain.universe_id,
            domain_id: domain.domain_id,
            view_id: "default".to_string(),
            renderer: "text".to_string(),
            is_legacy: false,
        });
    }

    if let Some(default_view) = scoped_views.iter().find(|view| view.id == "default") {
        return Ok(ViewSelectionV1 {
            universe_id: default_view.universe_id.clone(),
            domain_id: default_view.domain_id.clone(),
            view_id: default_view.id.clone(),
            renderer: default_view.renderer.clone(),
            is_legacy: false,
        });
    }

    if scoped_views.len() == 1 {
        let selected = scoped_views[0];
        return Ok(ViewSelectionV1 {
            universe_id: selected.universe_id.clone(),
            domain_id: selected.domain_id.clone(),
            view_id: selected.id.clone(),
            renderer: selected.renderer.clone(),
            is_legacy: false,
        });
    }

    Err(SdkError::LockMismatch(format!(
        "V-VIEW-REQUIRED: multiple views wired for universe `{}` domain `{}`; pass `--view <id>`",
        domain.universe_id, domain.domain_id
    )))
}

pub(crate) fn parse_cosmos_views_v1(path: &Path) -> Result<Vec<ViewProfileV1>, SdkError> {
    let raw = fs::read_to_string(path)?;

    let mut views = Vec::<ViewProfileV1>::new();
    let mut section = String::new();
    let mut builder = ViewBuilder::default();

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line == "[[view]]" || line == "[[views]]" {
            if section == "view" {
                views.push(builder.build()?);
                builder = ViewBuilder::default();
            }
            section = "view".to_string();
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if section == "view" {
                views.push(builder.build()?);
                builder = ViewBuilder::default();
            }
            section.clear();
            continue;
        }

        if section != "view" {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "id" => builder.id = Some(value),
            "universe_id" => builder.universe_id = Some(value),
            "domain_id" => builder.domain_id = Some(value),
            "renderer" => builder.renderer = Some(value),
            _ => {}
        }
    }

    if section == "view" {
        views.push(builder.build()?);
    }

    let mut seen = HashSet::<(String, String, String)>::new();
    for view in &views {
        let key = (
            view.universe_id.clone(),
            view.domain_id.clone(),
            view.id.clone(),
        );
        if !seen.insert(key) {
            return Err(SdkError::LockMismatch(format!(
                "V-VIEW-DUPLICATE: duplicate view `{}` for universe `{}` domain `{}`",
                view.id, view.universe_id, view.domain_id
            )));
        }
    }

    Ok(views)
}

pub(crate) fn parse_cosmos_kits_v1(path: &Path) -> Result<Vec<ViewKitBindingV1>, SdkError> {
    let raw = fs::read_to_string(path)?;
    let mut kits = Vec::<ViewKitBindingV1>::new();
    let mut section = String::new();
    let mut builder = KitBuilder::default();

    for raw_line in raw.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line == "[[kits]]" {
            if section == "kits" {
                kits.push(builder.build()?);
                builder = KitBuilder::default();
            }
            section = "kits".to_string();
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if section == "kits" {
                kits.push(builder.build()?);
                builder = KitBuilder::default();
            }
            section.clear();
            continue;
        }

        if section != "kits" {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let value = v.trim().trim_matches('"').to_string();
        match key {
            "kit_id" => builder.kit_id = Some(value),
            "universe_id" => builder.universe_id = Some(value),
            "bind_domain" => builder.bind_domain = Some(value),
            "bind_view" => builder.bind_view = Some(value),
            "required_organs" => builder.required_organs = parse_string_list_v1(v.trim()),
            _ => {}
        }
    }

    if section == "kits" {
        kits.push(builder.build()?);
    }

    let mut seen = HashSet::<(String, String, String, String)>::new();
    for kit in &kits {
        let key = (
            kit.kit_id.clone(),
            kit.universe_id.clone(),
            kit.bind_domain.clone(),
            kit.bind_view.clone(),
        );
        if !seen.insert(key) {
            return Err(SdkError::LockMismatch(format!(
                "V-KIT-WIRING-DUPLICATE: duplicate wiring `{}` for universe `{}` domain `{}` view `{}`",
                kit.kit_id, kit.universe_id, kit.bind_domain, kit.bind_view
            )));
        }
    }

    Ok(kits)
}

#[derive(Debug, Default)]
struct ViewBuilder {
    id: Option<String>,
    universe_id: Option<String>,
    domain_id: Option<String>,
    renderer: Option<String>,
}

impl ViewBuilder {
    fn build(self) -> Result<ViewProfileV1, SdkError> {
        let id = self.id.ok_or_else(|| {
            SdkError::LockMismatch("V-VIEW-ID-MISSING: view is missing `id`".to_string())
        })?;
        if id.is_empty() {
            return Err(SdkError::LockMismatch(
                "V-VIEW-ID-EMPTY: view id must not be empty".to_string(),
            ));
        }

        let universe_id = self.universe_id.unwrap_or_else(|| "default".to_string());
        let domain_id = self.domain_id.unwrap_or_else(|| "default".to_string());
        let renderer = self.renderer.unwrap_or_else(|| "text".to_string());

        if renderer != "text" && renderer != "tree" {
            return Err(SdkError::LockMismatch(format!(
                "V-VIEW-RENDERER-INVALID: view `{id}` renderer must be `text` or `tree`"
            )));
        }

        Ok(ViewProfileV1 {
            id,
            universe_id,
            domain_id,
            renderer,
        })
    }
}

#[derive(Debug, Default)]
struct KitBuilder {
    kit_id: Option<String>,
    universe_id: Option<String>,
    bind_domain: Option<String>,
    bind_view: Option<String>,
    required_organs: Vec<String>,
}

impl KitBuilder {
    fn build(self) -> Result<ViewKitBindingV1, SdkError> {
        let kit_id = self.kit_id.ok_or_else(|| {
            SdkError::LockMismatch("V-KIT-ID-MISSING: kit wiring is missing `kit_id`".to_string())
        })?;
        if kit_id.is_empty() {
            return Err(SdkError::LockMismatch(
                "V-KIT-ID-EMPTY: kit_id must not be empty".to_string(),
            ));
        }

        let bind_domain = self.bind_domain.ok_or_else(|| {
            SdkError::LockMismatch(
                "V-KIT-BIND-DOMAIN-MISSING: kit wiring is missing `bind_domain`".to_string(),
            )
        })?;
        if bind_domain.is_empty() {
            return Err(SdkError::LockMismatch(
                "V-KIT-BIND-DOMAIN-EMPTY: bind_domain must not be empty".to_string(),
            ));
        }

        let bind_view = self.bind_view.ok_or_else(|| {
            SdkError::LockMismatch(
                "V-KIT-BIND-VIEW-MISSING: kit wiring is missing `bind_view`".to_string(),
            )
        })?;
        if bind_view.is_empty() {
            return Err(SdkError::LockMismatch(
                "V-KIT-BIND-VIEW-EMPTY: bind_view must not be empty".to_string(),
            ));
        }

        let mut required_organs = self.required_organs;
        required_organs.sort();
        required_organs.dedup();

        Ok(ViewKitBindingV1 {
            kit_id,
            universe_id: self.universe_id.unwrap_or_else(|| "default".to_string()),
            bind_domain,
            bind_view,
            required_organs,
        })
    }
}

fn parse_string_list_v1(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Vec::new();
    }
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Vec::new();
        }
        return inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    let single = value.trim_matches('"').to_string();
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}
