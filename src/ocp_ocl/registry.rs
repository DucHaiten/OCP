use std::collections::{HashMap, HashSet};

use crate::ocp_ocl::keys::{parse_key, KeyRef};
use crate::ocp_ocl::ReasonCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryCheckError {
    KeyUnknown,
    CapabilityDenied,
    CtxInvalid,
}

impl RegistryCheckError {
    pub const fn to_reason_code(&self) -> ReasonCode {
        match self {
            Self::KeyUnknown => ReasonCode::KeyUnknown,
            Self::CapabilityDenied => ReasonCode::CapabilityDenied,
            Self::CtxInvalid => ReasonCode::CtxInvalid,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPattern {
    Any,
    Exact(String),
    Prefix(String),
}

impl KeyPattern {
    pub fn from_pattern(raw: &str) -> Self {
        let p = raw.trim();
        if p == "*" {
            return Self::Any;
        }
        if let Some(prefix) = p.strip_suffix('*') {
            return Self::Prefix(prefix.to_string());
        }
        Self::Exact(p.to_string())
    }

    pub fn matches(&self, key: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(v) => key == v,
            Self::Prefix(v) => key.starts_with(v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    enabled_families: HashSet<String>,
    allow_patterns: Vec<KeyPattern>,
    deny_patterns: Vec<KeyPattern>,
    ctx_required: HashMap<String, Vec<String>>,
    commit_allowed: HashMap<String, bool>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::v1_baseline()
    }
}

impl CapabilityRegistry {
    pub fn v1_baseline() -> Self {
        let mut enabled_families = HashSet::new();
        for f in [
            "world", "render", "mem", "dlg", "self", "rel", "plan", "std",
        ] {
            enabled_families.insert(f.to_string());
        }

        let mut ctx_required = HashMap::new();
        ctx_required.insert("std.args.get".to_string(), vec!["value".to_string()]);
        ctx_required.insert("std.fs.read".to_string(), vec!["path".to_string()]);
        ctx_required.insert(
            "std.fs.write".to_string(),
            vec!["path".to_string(), "content".to_string()],
        );
        ctx_required.insert("std.fs.list".to_string(), vec!["path".to_string()]);
        ctx_required.insert("std.fs.stat".to_string(), vec!["path".to_string()]);
        ctx_required.insert("std.http.get".to_string(), vec!["url".to_string()]);
        ctx_required.insert(
            "std.http.post".to_string(),
            vec!["url".to_string(), "body".to_string()],
        );
        ctx_required.insert("std.json.parse".to_string(), vec!["raw".to_string()]);
        ctx_required.insert("std.json.emit".to_string(), vec!["value".to_string()]);
        ctx_required.insert("std.time.sleep".to_string(), vec!["ms".to_string()]);
        ctx_required.insert("std.log.info".to_string(), vec!["message".to_string()]);

        Self {
            enabled_families,
            allow_patterns: vec![KeyPattern::Any],
            deny_patterns: Vec::new(),
            ctx_required,
            commit_allowed: HashMap::new(),
        }
    }

    pub fn add_enabled_family(&mut self, family: impl Into<String>) {
        self.enabled_families.insert(family.into());
    }

    pub fn add_allow_pattern(&mut self, pattern: impl AsRef<str>) {
        self.allow_patterns
            .push(KeyPattern::from_pattern(pattern.as_ref()));
    }

    pub fn add_deny_pattern(&mut self, pattern: impl AsRef<str>) {
        self.deny_patterns
            .push(KeyPattern::from_pattern(pattern.as_ref()));
    }

    pub fn set_ctx_required_for_key(
        &mut self,
        key_pattern: impl Into<String>,
        required_keys: Vec<String>,
    ) {
        self.ctx_required.insert(key_pattern.into(), required_keys);
    }

    pub fn set_commit_allowed_for_key(&mut self, key: impl Into<String>, allowed: bool) {
        self.commit_allowed.insert(key.into(), allowed);
    }

    pub fn commit_allowed_for_key(&self, key: &str) -> bool {
        self.commit_allowed.get(key).copied().unwrap_or(true)
    }

    pub fn check_observe(
        &self,
        key_literal: &str,
        ctx_literal: &str,
    ) -> Result<KeyRef, RegistryCheckError> {
        let key = parse_key(key_literal).ok_or(RegistryCheckError::KeyUnknown)?;

        if !self.enabled_families.contains(&key.family) {
            return Err(RegistryCheckError::CapabilityDenied);
        }

        if self.deny_patterns.iter().any(|p| p.matches(&key.raw)) {
            return Err(RegistryCheckError::CapabilityDenied);
        }

        if !self.allow_patterns.is_empty()
            && !self.allow_patterns.iter().any(|p| p.matches(&key.raw))
        {
            return Err(RegistryCheckError::CapabilityDenied);
        }

        if !self.ctx_matches_required(&key.raw, ctx_literal) {
            return Err(RegistryCheckError::CtxInvalid);
        }

        Ok(key)
    }

    fn ctx_matches_required(&self, key: &str, ctx_literal: &str) -> bool {
        let ctx = parse_ctx_pairs(ctx_literal);
        for (pattern, required) in &self.ctx_required {
            let kp = KeyPattern::from_pattern(pattern);
            if kp.matches(key) {
                for req in required {
                    if !ctx.contains_key(req) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

fn parse_ctx_pairs(ctx_literal: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for part in ctx_literal.split(';') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            let key = k.trim();
            let val = v.trim();
            if !key.is_empty() {
                out.insert(key.to_string(), val.to_string());
            }
        }
    }
    out
}
