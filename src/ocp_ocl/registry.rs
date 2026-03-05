use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ocp_ocl::keys::{parse_key, KeyRef};
use crate::ocp_ocl::schema::{FieldConstraints, FieldSpec, SchemaType};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCapabilityKind {
    ObserveOnly,
    ObserveAndCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismClass {
    Deterministic,
    NonDeterministic,
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
    ctx_schemas: HashMap<String, SchemaType>,
    payload_schemas: HashMap<String, SchemaType>,
    commit_allowed: HashMap<String, bool>,
    determinism_classes: HashMap<String, DeterminismClass>,
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
            "world", "render", "mem", "dlg", "self", "rel", "plan", "std", "engine", "custom",
        ] {
            enabled_families.insert(f.to_string());
        }

        let mut ctx_required = HashMap::new();
        ctx_required.insert("std.args.get".to_string(), vec!["value".to_string()]);
        ctx_required.insert("std.fs.read".to_string(), vec!["path".to_string()]);
        ctx_required.insert("std.fs.read_text".to_string(), vec!["path".to_string()]);
        ctx_required.insert(
            "std.fs.write_text".to_string(),
            vec!["path".to_string(), "text".to_string()],
        );
        ctx_required.insert("std.fs.mkdir".to_string(), vec!["path".to_string()]);
        ctx_required.insert("std.fs.remove".to_string(), vec!["path".to_string()]);
        ctx_required.insert(
            "std.fs.rename".to_string(),
            vec!["from".to_string(), "to".to_string()],
        );
        ctx_required.insert(
            "std.fs.write".to_string(),
            vec!["path".to_string(), "content".to_string()],
        );
        ctx_required.insert("std.fs.list".to_string(), vec!["path".to_string()]);
        ctx_required.insert(
            "std.fs.list_dir".to_string(),
            vec!["path".to_string(), "cap".to_string()],
        );
        ctx_required.insert("std.fs.stat".to_string(), vec!["path".to_string()]);
        ctx_required.insert("std.http.get".to_string(), vec!["url".to_string()]);
        ctx_required.insert(
            "std.http.post".to_string(),
            vec!["url".to_string(), "body".to_string()],
        );
        ctx_required.insert(
            "std.net.http.request".to_string(),
            vec!["method".to_string(), "url".to_string()],
        );
        ctx_required.insert("std.json.parse".to_string(), vec!["raw".to_string()]);
        ctx_required.insert("std.json.emit".to_string(), vec!["value".to_string()]);
        ctx_required.insert("std.kv.get".to_string(), vec!["key".to_string()]);
        ctx_required.insert("std.kv.keys".to_string(), vec!["cap".to_string()]);
        ctx_required.insert("std.kv.put".to_string(), vec!["key".to_string()]);
        ctx_required.insert("std.kv.del".to_string(), vec!["key".to_string()]);
        ctx_required.insert("std.time.sleep".to_string(), vec!["ms".to_string()]);
        ctx_required.insert("std.time.wallclock.now".to_string(), Vec::new());
        ctx_required.insert("std.log.info".to_string(), vec!["message".to_string()]);
        ctx_required.insert("std.proc.exec".to_string(), vec!["bin".to_string()]);
        ctx_required.insert("std.net.listen".to_string(), vec!["addr".to_string()]);
        ctx_required.insert(
            "std.net.reply".to_string(),
            vec!["conn".to_string(), "status".to_string(), "body".to_string()],
        );
        ctx_required.insert("std.net.close".to_string(), vec!["conn".to_string()]);
        ctx_required.insert(
            "std.tls.connect".to_string(),
            vec!["host".to_string(), "port".to_string()],
        );
        ctx_required.insert("std.tls.handshake".to_string(), vec!["conn".to_string()]);
        ctx_required.insert(
            "std.db.query_int".to_string(),
            vec!["dsn".to_string(), "sql".to_string()],
        );
        ctx_required.insert(
            "std.db.exec".to_string(),
            vec!["dsn".to_string(), "sql".to_string()],
        );
        ctx_required.insert("std.ui.frame_info".to_string(), Vec::new());
        ctx_required.insert("std.ui.input".to_string(), vec!["cap".to_string()]);
        ctx_required.insert("std.ui.draw".to_string(), vec!["cap".to_string()]);
        ctx_required.insert("std.game.tick_info".to_string(), Vec::new());
        ctx_required.insert(
            "std.game.rng".to_string(),
            vec!["stream".to_string(), "count".to_string()],
        );
        ctx_required.insert(
            "std.game.state_delta".to_string(),
            vec!["idempotency_key".to_string(), "delta".to_string()],
        );
        ctx_required.insert(
            "std.shadow.run".to_string(),
            vec!["variants_json".to_string()],
        );
        ctx_required.insert(
            "std.shadow.search".to_string(),
            vec!["variants_json".to_string()],
        );
        ctx_required.insert(
            "std.shadow.compare".to_string(),
            vec!["branches_json".to_string()],
        );
        ctx_required.insert(
            "engine.ui.run".to_string(),
            vec!["entry_module".to_string()],
        );
        ctx_required.insert(
            "engine.game.run".to_string(),
            vec!["entry_module".to_string()],
        );
        ctx_required.insert(
            "engine.shadow.preview".to_string(),
            vec!["entry_module".to_string(), "variants_json".to_string()],
        );
        ctx_required.insert(
            "std.view.render_text".to_string(),
            vec!["truth".to_string()],
        );
        ctx_required.insert(
            "std.view.render_tree".to_string(),
            vec!["truth".to_string()],
        );

        let mut commit_allowed = HashMap::new();
        commit_allowed.insert("std.fs.read_text".to_string(), false);
        commit_allowed.insert("std.fs.list_dir".to_string(), false);
        commit_allowed.insert("std.fs.stat".to_string(), false);
        commit_allowed.insert("std.db.query_int".to_string(), false);
        commit_allowed.insert("std.kv.get".to_string(), false);
        commit_allowed.insert("std.kv.keys".to_string(), false);
        commit_allowed.insert("std.tls.connect".to_string(), false);
        commit_allowed.insert("std.tls.handshake".to_string(), false);
        commit_allowed.insert("std.ui.frame_info".to_string(), false);
        commit_allowed.insert("std.ui.input".to_string(), false);
        commit_allowed.insert("std.game.tick_info".to_string(), false);
        commit_allowed.insert("std.game.rng".to_string(), false);
        commit_allowed.insert("std.time.wallclock.now".to_string(), false);
        commit_allowed.insert("std.proc.exec".to_string(), false);
        commit_allowed.insert("std.net.http.request".to_string(), false);
        commit_allowed.insert("std.game.state_delta".to_string(), true);
        commit_allowed.insert("std.shadow.run".to_string(), false);
        commit_allowed.insert("std.shadow.search".to_string(), false);
        commit_allowed.insert("std.shadow.compare".to_string(), false);
        commit_allowed.insert("engine.ui.run".to_string(), false);
        commit_allowed.insert("engine.game.run".to_string(), false);
        commit_allowed.insert("engine.shadow.preview".to_string(), false);
        commit_allowed.insert("std.view.render_text".to_string(), false);
        commit_allowed.insert("std.view.render_tree".to_string(), false);

        let mut determinism_classes = HashMap::new();
        determinism_classes.insert(
            "std.time.tick_info".to_string(),
            DeterminismClass::Deterministic,
        );
        determinism_classes.insert(
            "std.game.tick_info".to_string(),
            DeterminismClass::Deterministic,
        );
        determinism_classes.insert(
            "std.ui.frame_info".to_string(),
            DeterminismClass::Deterministic,
        );
        determinism_classes.insert(
            "std.shadow.run".to_string(),
            DeterminismClass::Deterministic,
        );
        determinism_classes.insert(
            "std.shadow.search".to_string(),
            DeterminismClass::Deterministic,
        );
        determinism_classes.insert(
            "std.shadow.compare".to_string(),
            DeterminismClass::Deterministic,
        );

        let mut ctx_schemas = HashMap::new();
        ctx_schemas.insert(
            "std.fs.read_text".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    (
                        "max_bytes",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(1_048_576),
                            ..FieldConstraints::default()
                        }),
                    ),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.list_dir".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    (
                        "cap",
                        FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(10_000),
                            ..FieldConstraints::default()
                        }),
                    ),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.net.http.request".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "method",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec![
                                "GET".to_string(),
                                "POST".to_string(),
                                "PUT".to_string(),
                                "PATCH".to_string(),
                                "DELETE".to_string(),
                            ],
                        }),
                    ),
                    (
                        "url",
                        FieldSpec::required(SchemaType::String).with_constraints(
                            FieldConstraints {
                                nonempty: true,
                                ..FieldConstraints::default()
                            },
                        ),
                    ),
                    (
                        "timeout_ms",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(60_000),
                            ..FieldConstraints::default()
                        }),
                    ),
                    (
                        "max_body_bytes",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(1_048_576),
                            ..FieldConstraints::default()
                        }),
                    ),
                    (
                        "headers",
                        FieldSpec::optional(SchemaType::Map {
                            value: Box::new(SchemaType::String),
                            cap: Some(64),
                        }),
                    ),
                    ("body", FieldSpec::optional(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.stat".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("path", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.write_text".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("text", FieldSpec::required(SchemaType::String)),
                    ("overwrite", FieldSpec::optional(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.mkdir".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("recursive", FieldSpec::optional(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.remove".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("recursive", FieldSpec::optional(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.rename".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("from", FieldSpec::required(SchemaType::String)),
                    ("to", FieldSpec::required(SchemaType::String)),
                    ("overwrite", FieldSpec::optional(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.read".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("path", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.write".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("content", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.fs.list".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("path", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.kv.get".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("key", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.kv.keys".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![(
                    "cap",
                    FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                        min_int: Some(1),
                        max_int: Some(10_000),
                        ..FieldConstraints::default()
                    }),
                )]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.kv.put".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("key", FieldSpec::required(SchemaType::String)),
                    (
                        "value",
                        FieldSpec::optional(SchemaType::Union {
                            types: vec![
                                SchemaType::String,
                                SchemaType::Int,
                                SchemaType::Bool,
                                SchemaType::Null,
                            ],
                        }),
                    ),
                    ("value_json", FieldSpec::optional(SchemaType::String)),
                    ("overwrite", FieldSpec::optional(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.kv.del".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("key", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.kv.clear".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("prefix", FieldSpec::optional(SchemaType::String))]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.time.now".to_string(),
            SchemaType::Record {
                fields: BTreeMap::new(),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.time.wallclock.now".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("scope", FieldSpec::optional(SchemaType::String))]),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.time.tick_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                    (
                        "dt_ms",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            ..FieldConstraints::default()
                        }),
                    ),
                ]),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.time.now_logical".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                    (
                        "dt_ms",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            ..FieldConstraints::default()
                        }),
                    ),
                ]),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.time.sleep".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![(
                    "ms",
                    FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                        min_int: Some(0),
                        ..FieldConstraints::default()
                    }),
                )]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.proc.exec".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "bin",
                        FieldSpec::required(SchemaType::String).with_constraints(
                            FieldConstraints {
                                nonempty: true,
                                ..FieldConstraints::default()
                            },
                        ),
                    ),
                    ("args", FieldSpec::optional(SchemaType::String)),
                    ("cwd", FieldSpec::optional(SchemaType::String)),
                    (
                        "timeout_ms",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(60_000),
                            ..FieldConstraints::default()
                        }),
                    ),
                    (
                        "max_stdout_bytes",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(1_048_576),
                            ..FieldConstraints::default()
                        }),
                    ),
                    (
                        "max_stderr_bytes",
                        FieldSpec::optional(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(1_048_576),
                            ..FieldConstraints::default()
                        }),
                    ),
                ]),
                open_row: false,
            },
        );

        let mut payload_schemas = HashMap::new();
        payload_schemas.insert(
            "std.fs.read_text".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("text", FieldSpec::required(SchemaType::String)),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                    ("bytes", FieldSpec::required(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.ui.frame_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("w", FieldSpec::optional(SchemaType::Int)),
                    ("h", FieldSpec::optional(SchemaType::Int)),
                    ("scale", FieldSpec::optional(SchemaType::Int)),
                    ("theme", FieldSpec::optional(SchemaType::String)),
                    ("locale", FieldSpec::optional(SchemaType::String)),
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.ui.input".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "cap",
                        FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            ..FieldConstraints::default()
                        }),
                    ),
                    ("events", FieldSpec::optional(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.ui.draw".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "cap",
                        FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            ..FieldConstraints::default()
                        }),
                    ),
                    ("list", FieldSpec::optional(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.ui.present".to_string(),
            SchemaType::Record {
                fields: BTreeMap::new(),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.game.tick_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: true,
            },
        );
        ctx_schemas.insert(
            "std.game.rng".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("stream", FieldSpec::required(SchemaType::String)),
                    (
                        "count",
                        FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
                            min_int: Some(1),
                            max_int: Some(1024),
                            ..FieldConstraints::default()
                        }),
                    ),
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.game.state_delta".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "idempotency_key",
                        FieldSpec::required(SchemaType::String).with_constraints(
                            FieldConstraints {
                                nonempty: true,
                                ..FieldConstraints::default()
                            },
                        ),
                    ),
                    ("delta", FieldSpec::optional(SchemaType::String)),
                    ("delta_json", FieldSpec::optional(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.shadow.run".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("variants_json", FieldSpec::required(SchemaType::String)),
                    ("effect_keys", FieldSpec::optional(SchemaType::String)),
                    ("branches", FieldSpec::optional(SchemaType::Int)),
                    ("branch_step_cap", FieldSpec::optional(SchemaType::Int)),
                    ("branch_budget_cap", FieldSpec::optional(SchemaType::Int)),
                    ("checkpoint_every", FieldSpec::optional(SchemaType::Int)),
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.shadow.search".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("variants_json", FieldSpec::required(SchemaType::String)),
                    ("effect_keys", FieldSpec::optional(SchemaType::String)),
                    ("policy", FieldSpec::optional(SchemaType::String)),
                    ("max_branches", FieldSpec::optional(SchemaType::Int)),
                    ("branches", FieldSpec::optional(SchemaType::Int)),
                    ("per_branch_step_cap", FieldSpec::optional(SchemaType::Int)),
                    ("branch_step_cap", FieldSpec::optional(SchemaType::Int)),
                    (
                        "per_branch_budget_cap",
                        FieldSpec::optional(SchemaType::Int),
                    ),
                    ("branch_budget_cap", FieldSpec::optional(SchemaType::Int)),
                    ("global_step_cap", FieldSpec::optional(SchemaType::Int)),
                    ("global_budget_cap", FieldSpec::optional(SchemaType::Int)),
                    ("rounds", FieldSpec::optional(SchemaType::Int)),
                    ("beam_width", FieldSpec::optional(SchemaType::Int)),
                    ("top_k", FieldSpec::optional(SchemaType::Int)),
                    ("score_field", FieldSpec::optional(SchemaType::String)),
                    ("outcome_weight", FieldSpec::optional(SchemaType::Int)),
                    ("cost_budget_weight", FieldSpec::optional(SchemaType::Int)),
                    ("cost_steps_weight", FieldSpec::optional(SchemaType::Int)),
                    (
                        "reason_penalty_weight",
                        FieldSpec::optional(SchemaType::Int),
                    ),
                    ("state_score_weight", FieldSpec::optional(SchemaType::Int)),
                    (
                        "reason_penalties_json",
                        FieldSpec::optional(SchemaType::String),
                    ),
                    ("max_diff_keys", FieldSpec::optional(SchemaType::Int)),
                    ("max_report_bytes", FieldSpec::optional(SchemaType::Int)),
                    ("checkpoint_every", FieldSpec::optional(SchemaType::Int)),
                    ("tick", FieldSpec::optional(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        ctx_schemas.insert(
            "std.shadow.compare".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("branches_json", FieldSpec::required(SchemaType::String)),
                    ("baseline_id", FieldSpec::optional(SchemaType::Int)),
                    ("max_diff_keys", FieldSpec::optional(SchemaType::Int)),
                    ("max_report_bytes", FieldSpec::optional(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.list_dir".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "entries",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Record {
                                fields: btree_fields(vec![
                                    ("name", FieldSpec::required(SchemaType::String)),
                                    ("is_dir", FieldSpec::required(SchemaType::Bool)),
                                    ("size", FieldSpec::required(SchemaType::Int)),
                                ]),
                                open_row: false,
                            }),
                            cap: Some(10_000),
                        }),
                    ),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.stat".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("exists", FieldSpec::required(SchemaType::Bool)),
                    ("is_dir", FieldSpec::required(SchemaType::Bool)),
                    ("size", FieldSpec::required(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.write_text".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["write_text".to_string()],
                        }),
                    ),
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("text", FieldSpec::required(SchemaType::String)),
                    ("overwrite", FieldSpec::required(SchemaType::Bool)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.mkdir".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["mkdir".to_string()],
                        }),
                    ),
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("recursive", FieldSpec::required(SchemaType::Bool)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.remove".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["remove".to_string()],
                        }),
                    ),
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("recursive", FieldSpec::required(SchemaType::Bool)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.rename".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["rename".to_string()],
                        }),
                    ),
                    ("from", FieldSpec::required(SchemaType::String)),
                    ("to", FieldSpec::required(SchemaType::String)),
                    ("overwrite", FieldSpec::required(SchemaType::Bool)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.read".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("content", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.write".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("bytes", FieldSpec::required(SchemaType::String)),
                    ("status", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.fs.list".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("path", FieldSpec::required(SchemaType::String)),
                    ("items", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );

        let kv_value_schema = SchemaType::Union {
            types: vec![
                SchemaType::Bool,
                SchemaType::Int,
                SchemaType::String,
                SchemaType::Null,
                SchemaType::Map {
                    value: Box::new(SchemaType::Union {
                        types: vec![
                            SchemaType::Bool,
                            SchemaType::Int,
                            SchemaType::String,
                            SchemaType::Null,
                        ],
                    }),
                    cap: Some(256),
                },
                SchemaType::List {
                    elem: Box::new(SchemaType::Union {
                        types: vec![
                            SchemaType::Bool,
                            SchemaType::Int,
                            SchemaType::String,
                            SchemaType::Null,
                        ],
                    }),
                    cap: Some(256),
                },
            ],
        };

        payload_schemas.insert(
            "std.kv.get".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("found", FieldSpec::required(SchemaType::Bool)),
                    ("value", FieldSpec::required(kv_value_schema.clone())),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.kv.keys".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "keys",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::String),
                            cap: Some(10_000),
                        }),
                    ),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.kv.put".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["put".to_string()],
                        }),
                    ),
                    ("key", FieldSpec::required(SchemaType::String)),
                    ("value", FieldSpec::required(kv_value_schema)),
                    ("overwrite", FieldSpec::required(SchemaType::Bool)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.kv.del".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["del".to_string()],
                        }),
                    ),
                    ("key", FieldSpec::required(SchemaType::String)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.kv.clear".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["clear".to_string()],
                        }),
                    ),
                    ("prefix", FieldSpec::optional(SchemaType::String)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.time.now".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("unix_ms", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.time.tick_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("tick", FieldSpec::required(SchemaType::String)),
                    ("dt_ms", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.time.now_logical".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![("t", FieldSpec::required(SchemaType::String))]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.time.wallclock.now".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("unix_ms", FieldSpec::required(SchemaType::String)),
                    ("iso", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.proc.exec".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("exit_code", FieldSpec::required(SchemaType::Int)),
                    ("stdout", FieldSpec::required(SchemaType::String)),
                    ("stderr", FieldSpec::required(SchemaType::String)),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.net.http.request".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("status", FieldSpec::required(SchemaType::Int)),
                    (
                        "headers",
                        FieldSpec::required(SchemaType::Map {
                            value: Box::new(SchemaType::String),
                            cap: Some(256),
                        }),
                    ),
                    ("body", FieldSpec::required(SchemaType::String)),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.ui.frame_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("w", FieldSpec::required(SchemaType::String)),
                    ("h", FieldSpec::required(SchemaType::String)),
                    ("scale", FieldSpec::required(SchemaType::String)),
                    ("theme", FieldSpec::required(SchemaType::String)),
                    ("locale", FieldSpec::required(SchemaType::String)),
                    ("ctx_tick", FieldSpec::required(SchemaType::String)),
                    ("frame", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.ui.input".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "events",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Record {
                                fields: btree_fields(vec![
                                    ("t", FieldSpec::required(SchemaType::String)),
                                    (
                                        "a",
                                        FieldSpec::required(SchemaType::Map {
                                            value: Box::new(SchemaType::Union {
                                                types: vec![
                                                    SchemaType::String,
                                                    SchemaType::Int,
                                                    SchemaType::Bool,
                                                    SchemaType::Null,
                                                ],
                                            }),
                                            cap: Some(16),
                                        }),
                                    ),
                                ]),
                                open_row: false,
                            }),
                            cap: Some(64),
                        }),
                    ),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.ui.draw".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["draw".to_string()],
                        }),
                    ),
                    (
                        "list",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Record {
                                fields: btree_fields(vec![
                                    ("kind", FieldSpec::required(SchemaType::String)),
                                    ("raw", FieldSpec::required(SchemaType::String)),
                                ]),
                                open_row: false,
                            }),
                            cap: Some(4096),
                        }),
                    ),
                    ("cmd_count", FieldSpec::required(SchemaType::Int)),
                    ("cap", FieldSpec::required(SchemaType::Int)),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.ui.present".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["present".to_string()],
                        }),
                    ),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.game.tick_info".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("tick", FieldSpec::required(SchemaType::Int)),
                    ("dt_ms", FieldSpec::required(SchemaType::Int)),
                    ("ctx_tick", FieldSpec::required(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.game.rng".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("stream", FieldSpec::required(SchemaType::String)),
                    ("count", FieldSpec::required(SchemaType::Int)),
                    ("tick", FieldSpec::required(SchemaType::Int)),
                    (
                        "values",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Int),
                            cap: Some(4096),
                        }),
                    ),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.game.state_delta".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "op",
                        FieldSpec::required(SchemaType::Enum {
                            values: vec!["state_delta".to_string()],
                        }),
                    ),
                    ("pending_write_id", FieldSpec::required(SchemaType::String)),
                    ("idempotency_key", FieldSpec::required(SchemaType::String)),
                    (
                        "delta",
                        FieldSpec::required(SchemaType::Union {
                            types: vec![
                                SchemaType::Bool,
                                SchemaType::Int,
                                SchemaType::String,
                                SchemaType::Null,
                                SchemaType::Map {
                                    value: Box::new(SchemaType::Union {
                                        types: vec![
                                            SchemaType::Bool,
                                            SchemaType::Int,
                                            SchemaType::String,
                                            SchemaType::Null,
                                        ],
                                    }),
                                    cap: Some(256),
                                },
                                SchemaType::List {
                                    elem: Box::new(SchemaType::Union {
                                        types: vec![
                                            SchemaType::Bool,
                                            SchemaType::Int,
                                            SchemaType::String,
                                            SchemaType::Null,
                                        ],
                                    }),
                                    cap: Some(256),
                                },
                            ],
                        }),
                    ),
                    ("delta_bytes", FieldSpec::required(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.shadow.run".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "branches",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Record {
                                fields: btree_fields(vec![
                                    ("id", FieldSpec::required(SchemaType::Int)),
                                    ("outcome", FieldSpec::required(SchemaType::String)),
                                    ("signature", FieldSpec::required(SchemaType::String)),
                                    (
                                        "cost",
                                        FieldSpec::required(SchemaType::Map {
                                            value: Box::new(SchemaType::Int),
                                            cap: Some(8),
                                        }),
                                    ),
                                    (
                                        "state_summary",
                                        FieldSpec::required(SchemaType::Map {
                                            value: Box::new(SchemaType::Union {
                                                types: vec![
                                                    SchemaType::Bool,
                                                    SchemaType::Int,
                                                    SchemaType::String,
                                                    SchemaType::Null,
                                                    SchemaType::Map {
                                                        value: Box::new(SchemaType::Union {
                                                            types: vec![
                                                                SchemaType::Bool,
                                                                SchemaType::Int,
                                                                SchemaType::String,
                                                                SchemaType::Null,
                                                            ],
                                                        }),
                                                        cap: Some(256),
                                                    },
                                                    SchemaType::List {
                                                        elem: Box::new(SchemaType::Union {
                                                            types: vec![
                                                                SchemaType::Bool,
                                                                SchemaType::Int,
                                                                SchemaType::String,
                                                                SchemaType::Null,
                                                            ],
                                                        }),
                                                        cap: Some(256),
                                                    },
                                                ],
                                            }),
                                            cap: Some(128),
                                        }),
                                    ),
                                ]),
                                open_row: true,
                            }),
                            cap: Some(128),
                        }),
                    ),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                    ("max_branches", FieldSpec::required(SchemaType::Int)),
                    ("branch_step_cap", FieldSpec::required(SchemaType::Int)),
                    ("branch_budget_cap", FieldSpec::required(SchemaType::Int)),
                    ("branch_digest", FieldSpec::required(SchemaType::String)),
                    ("prefix_key", FieldSpec::optional(SchemaType::String)),
                    ("checkpoint_every", FieldSpec::optional(SchemaType::Int)),
                    (
                        "checkpoint_cache",
                        FieldSpec::optional(SchemaType::Map {
                            value: Box::new(SchemaType::Union {
                                types: vec![SchemaType::Int, SchemaType::Bool, SchemaType::String],
                            }),
                            cap: Some(16),
                        }),
                    ),
                    (
                        "memo",
                        FieldSpec::optional(SchemaType::Map {
                            value: Box::new(SchemaType::Union {
                                types: vec![SchemaType::Int, SchemaType::Bool, SchemaType::String],
                            }),
                            cap: Some(16),
                        }),
                    ),
                ]),
                open_row: false,
            },
        );
        payload_schemas.insert(
            "std.shadow.search".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                    (
                        "report",
                        FieldSpec::required(SchemaType::Record {
                            fields: btree_fields(vec![
                                (
                                    "schema",
                                    FieldSpec::required(SchemaType::Enum {
                                        values: vec!["shadow.report.v2".to_string()],
                                    }),
                                ),
                                ("truncated", FieldSpec::required(SchemaType::Bool)),
                                ("report_bytes", FieldSpec::required(SchemaType::Int)),
                            ]),
                            open_row: true,
                        }),
                    ),
                ]),
                open_row: true,
            },
        );
        payload_schemas.insert(
            "std.shadow.compare".to_string(),
            SchemaType::Record {
                fields: btree_fields(vec![
                    (
                        "diff_keys",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::String),
                            cap: Some(1024),
                        }),
                    ),
                    (
                        "divergence",
                        FieldSpec::required(SchemaType::Map {
                            value: Box::new(SchemaType::Bool),
                            cap: Some(1024),
                        }),
                    ),
                    (
                        "cost_table",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Map {
                                value: Box::new(SchemaType::Union {
                                    types: vec![
                                        SchemaType::Int,
                                        SchemaType::String,
                                        SchemaType::Bool,
                                        SchemaType::Null,
                                    ],
                                }),
                                cap: Some(16),
                            }),
                            cap: Some(1024),
                        }),
                    ),
                    (
                        "reason_table",
                        FieldSpec::required(SchemaType::List {
                            elem: Box::new(SchemaType::Map {
                                value: Box::new(SchemaType::Union {
                                    types: vec![SchemaType::Int, SchemaType::String],
                                }),
                                cap: Some(8),
                            }),
                            cap: Some(1024),
                        }),
                    ),
                    ("truncated", FieldSpec::required(SchemaType::Bool)),
                    ("report_bytes", FieldSpec::required(SchemaType::Int)),
                ]),
                open_row: false,
            },
        );

        Self {
            enabled_families,
            allow_patterns: vec![KeyPattern::Any],
            deny_patterns: vec![KeyPattern::Exact("std.net.poll".to_string())],
            ctx_required,
            ctx_schemas,
            payload_schemas,
            commit_allowed,
            determinism_classes,
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

    pub fn set_ctx_schema_for_key(&mut self, key_pattern: impl Into<String>, schema: SchemaType) {
        self.ctx_schemas.insert(key_pattern.into(), schema);
    }

    pub fn ctx_schema_for_key(&self, key: &str) -> Option<&SchemaType> {
        best_schema_for_key(&self.ctx_schemas, key)
    }

    pub fn set_payload_schema_for_key(
        &mut self,
        key_pattern: impl Into<String>,
        schema: SchemaType,
    ) {
        self.payload_schemas.insert(key_pattern.into(), schema);
    }

    pub fn payload_schema_for_key(&self, key: &str) -> Option<&SchemaType> {
        best_schema_for_key(&self.payload_schemas, key)
    }

    pub fn set_commit_allowed_for_key(&mut self, key: impl Into<String>, allowed: bool) {
        self.commit_allowed.insert(key.into(), allowed);
    }

    pub fn set_key_kind_for_key(&mut self, key: impl Into<String>, kind: KeyCapabilityKind) {
        self.set_commit_allowed_for_key(key, matches!(kind, KeyCapabilityKind::ObserveAndCommit));
    }

    pub fn set_determinism_class_for_key(
        &mut self,
        key_pattern: impl Into<String>,
        class: DeterminismClass,
    ) {
        self.determinism_classes.insert(key_pattern.into(), class);
    }

    pub fn key_kind_for_key(&self, key: &str) -> KeyCapabilityKind {
        if self.commit_allowed_for_key(key) {
            KeyCapabilityKind::ObserveAndCommit
        } else {
            KeyCapabilityKind::ObserveOnly
        }
    }

    pub fn determinism_class_for_key(&self, key: &str) -> DeterminismClass {
        best_determinism_for_key(&self.determinism_classes, key)
            .copied()
            .unwrap_or(DeterminismClass::NonDeterministic)
    }

    pub fn documented_keys(&self) -> Vec<String> {
        let mut keys = BTreeSet::<String>::new();
        keys.extend(self.ctx_required.keys().cloned());
        keys.extend(self.ctx_schemas.keys().cloned());
        keys.extend(self.payload_schemas.keys().cloned());
        keys.extend(self.commit_allowed.keys().cloned());
        keys.extend(self.determinism_classes.keys().cloned());
        keys.into_iter().collect()
    }

    pub fn ctx_required_for_key(&self, key: &str) -> Vec<String> {
        let mut required = BTreeSet::<String>::new();
        for (pattern, fields) in &self.ctx_required {
            let kp = KeyPattern::from_pattern(pattern);
            if kp.matches(key) {
                for field in fields {
                    required.insert(field.clone());
                }
            }
        }
        required.into_iter().collect()
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

fn btree_fields(items: Vec<(&str, FieldSpec)>) -> BTreeMap<String, FieldSpec> {
    let mut out = BTreeMap::new();
    for (name, spec) in items {
        out.insert(name.to_string(), spec);
    }
    out
}

fn best_schema_for_key<'a>(
    schemas: &'a HashMap<String, SchemaType>,
    key: &str,
) -> Option<&'a SchemaType> {
    let mut best: Option<(&SchemaType, usize)> = None;
    for (pattern, schema) in schemas {
        let kp = KeyPattern::from_pattern(pattern);
        if kp.matches(key) {
            let score = pattern.trim_end_matches('*').len();
            match best {
                Some((_, best_score)) if best_score >= score => {}
                _ => best = Some((schema, score)),
            }
        }
    }
    best.map(|(schema, _)| schema)
}

fn best_determinism_for_key<'a>(
    classes: &'a HashMap<String, DeterminismClass>,
    key: &str,
) -> Option<&'a DeterminismClass> {
    let mut best: Option<(&DeterminismClass, usize)> = None;
    for (pattern, class) in classes {
        let kp = KeyPattern::from_pattern(pattern);
        if kp.matches(key) {
            let score = pattern.trim_end_matches('*').len();
            match best {
                Some((_, best_score)) if best_score >= score => {}
                _ => best = Some((class, score)),
            }
        }
    }
    best.map(|(class, _)| class)
}
