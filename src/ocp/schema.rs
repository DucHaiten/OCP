use std::collections::BTreeMap;

use crate::ocp::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaType {
    Bool,
    Int,
    String,
    Null,
    List {
        elem: Box<SchemaType>,
        cap: Option<usize>,
    },
    Map {
        value: Box<SchemaType>,
        cap: Option<usize>,
    },
    Record {
        fields: BTreeMap<String, FieldSpec>,
        open_row: bool,
    },
    Enum {
        values: Vec<String>,
    },
    Union {
        types: Vec<SchemaType>,
    },
    Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldConstraints {
    pub min_int: Option<i64>,
    pub max_int: Option<i64>,
    pub regex: Option<String>,
    pub nonempty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    pub required: bool,
    pub ty: SchemaType,
    pub default: Option<Value>,
    pub constraints: FieldConstraints,
}

impl FieldSpec {
    pub fn required(ty: SchemaType) -> Self {
        Self {
            required: true,
            ty,
            default: None,
            constraints: FieldConstraints::default(),
        }
    }

    pub fn optional(ty: SchemaType) -> Self {
        Self {
            required: false,
            ty,
            default: None,
            constraints: FieldConstraints::default(),
        }
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn with_constraints(mut self, constraints: FieldConstraints) -> Self {
        self.constraints = constraints;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaIssueCode {
    MissingField,
    UnknownField,
    TypeMismatch,
    ConstraintViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIssue {
    pub code: SchemaIssueCode,
    pub path: String,
    pub message: String,
    pub expected: Option<String>,
    pub got: Option<String>,
}

impl SchemaIssue {
    fn missing_field(path: String, field: &str) -> Self {
        Self {
            code: SchemaIssueCode::MissingField,
            path,
            message: format!("missing required field `{field}`"),
            expected: None,
            got: None,
        }
    }

    fn unknown_field(path: String, field: &str) -> Self {
        Self {
            code: SchemaIssueCode::UnknownField,
            path,
            message: format!("unknown field `{field}`"),
            expected: None,
            got: None,
        }
    }

    fn type_mismatch(path: String, expected: String, got: String) -> Self {
        Self {
            code: SchemaIssueCode::TypeMismatch,
            path,
            message: "type mismatch".to_string(),
            expected: Some(expected),
            got: Some(got),
        }
    }

    fn constraint_violation(path: String, message: String) -> Self {
        Self {
            code: SchemaIssueCode::ConstraintViolation,
            path,
            message,
            expected: None,
            got: None,
        }
    }
}

pub fn validate_schema_value(schema: &SchemaType, value: &Value) -> Result<(), Vec<SchemaIssue>> {
    let mut issues = Vec::new();
    validate_inner(schema, value, "$", &mut issues);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

fn validate_inner(schema: &SchemaType, value: &Value, path: &str, issues: &mut Vec<SchemaIssue>) {
    match schema {
        SchemaType::Bool => expect_bool(value, path, issues),
        SchemaType::Int => expect_int(value, path, issues, None),
        SchemaType::String => expect_string(value, path, issues, None),
        SchemaType::Null => {
            if !matches!(value, Value::Unit) {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    "null".to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        }
        SchemaType::Bytes => {
            if !matches!(value, Value::String(_)) {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    "bytes".to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        }
        SchemaType::List { elem, cap } => match value {
            Value::List(items) => {
                if let Some(limit) = cap {
                    if items.len() > *limit {
                        issues.push(SchemaIssue::constraint_violation(
                            path.to_string(),
                            format!("list length {} exceeds cap {}", items.len(), limit),
                        ));
                    }
                }
                for (idx, item) in items.iter().enumerate() {
                    validate_inner(elem, item, &format!("{path}[{idx}]"), issues);
                }
            }
            _ => {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    schema_type_name(schema).to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        },
        SchemaType::Map { value: val_ty, cap } => match value {
            Value::Map(map) => {
                if let Some(limit) = cap {
                    if map.len() > *limit {
                        issues.push(SchemaIssue::constraint_violation(
                            path.to_string(),
                            format!("map length {} exceeds cap {}", map.len(), limit),
                        ));
                    }
                }
                for (key, item) in map {
                    validate_inner(val_ty, item, &format!("{path}.{key}"), issues);
                }
            }
            _ => {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    schema_type_name(schema).to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        },
        SchemaType::Record { fields, open_row } => match value {
            Value::Map(map) => {
                for (name, spec) in fields {
                    match map.get(name) {
                        Some(v) => {
                            let field_path = format!("{path}.{name}");
                            validate_field_value(spec, v, &field_path, issues);
                        }
                        None => {
                            if spec.required && spec.default.is_none() {
                                issues.push(SchemaIssue::missing_field(path.to_string(), name));
                            }
                        }
                    }
                }

                if !open_row {
                    for key in map.keys() {
                        if !fields.contains_key(key) {
                            issues.push(SchemaIssue::unknown_field(path.to_string(), key));
                        }
                    }
                }
            }
            _ => {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    "record".to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        },
        SchemaType::Enum { values } => match value {
            Value::String(raw) => {
                if !values.iter().any(|v| v == raw) {
                    issues.push(SchemaIssue::constraint_violation(
                        path.to_string(),
                        format!("`{raw}` is not in enum set"),
                    ));
                }
            }
            _ => {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    "enum".to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        },
        SchemaType::Union { types } => {
            if types.is_empty() {
                issues.push(SchemaIssue::constraint_violation(
                    path.to_string(),
                    "union must contain at least one type".to_string(),
                ));
                return;
            }

            let mut any_passed = false;
            for ty in types {
                if validate_schema_value(ty, value).is_ok() {
                    any_passed = true;
                    break;
                }
            }
            if !any_passed {
                issues.push(SchemaIssue::type_mismatch(
                    path.to_string(),
                    schema_type_name(schema).to_string(),
                    value_type_name(value).to_string(),
                ));
            }
        }
    }
}

fn validate_field_value(
    spec: &FieldSpec,
    value: &Value,
    path: &str,
    issues: &mut Vec<SchemaIssue>,
) {
    match &spec.ty {
        SchemaType::Int => {
            expect_int(value, path, issues, Some(&spec.constraints));
        }
        SchemaType::String => {
            expect_string(value, path, issues, Some(&spec.constraints));
        }
        _ => validate_inner(&spec.ty, value, path, issues),
    }
}

fn expect_bool(value: &Value, path: &str, issues: &mut Vec<SchemaIssue>) {
    if !matches!(value, Value::Bool(_)) {
        issues.push(SchemaIssue::type_mismatch(
            path.to_string(),
            "bool".to_string(),
            value_type_name(value).to_string(),
        ));
    }
}

fn expect_int(
    value: &Value,
    path: &str,
    issues: &mut Vec<SchemaIssue>,
    constraints: Option<&FieldConstraints>,
) {
    let Value::Int(raw) = value else {
        issues.push(SchemaIssue::type_mismatch(
            path.to_string(),
            "int".to_string(),
            value_type_name(value).to_string(),
        ));
        return;
    };

    if let Some(c) = constraints {
        if let Some(min) = c.min_int {
            if *raw < min {
                issues.push(SchemaIssue::constraint_violation(
                    path.to_string(),
                    format!("int {raw} is below min {min}"),
                ));
            }
        }
        if let Some(max) = c.max_int {
            if *raw > max {
                issues.push(SchemaIssue::constraint_violation(
                    path.to_string(),
                    format!("int {raw} is above max {max}"),
                ));
            }
        }
    }
}

fn expect_string(
    value: &Value,
    path: &str,
    issues: &mut Vec<SchemaIssue>,
    constraints: Option<&FieldConstraints>,
) {
    let Value::String(raw) = value else {
        issues.push(SchemaIssue::type_mismatch(
            path.to_string(),
            "string".to_string(),
            value_type_name(value).to_string(),
        ));
        return;
    };

    if let Some(c) = constraints {
        if c.nonempty && raw.is_empty() {
            issues.push(SchemaIssue::constraint_violation(
                path.to_string(),
                "string must be non-empty".to_string(),
            ));
        }
        if let Some(pattern) = &c.regex {
            if !raw.contains(pattern) {
                issues.push(SchemaIssue::constraint_violation(
                    path.to_string(),
                    format!("string does not satisfy regex-like pattern `{pattern}`"),
                ));
            }
        }
    }
}

pub fn pretty_schema(schema: &SchemaType) -> String {
    let mut out = String::new();
    pretty_schema_inner(schema, 0, &mut out);
    out
}

fn pretty_schema_inner(schema: &SchemaType, indent: usize, out: &mut String) {
    match schema {
        SchemaType::Bool => out.push_str("bool"),
        SchemaType::Int => out.push_str("int"),
        SchemaType::String => out.push_str("string"),
        SchemaType::Null => out.push_str("null"),
        SchemaType::Bytes => out.push_str("bytes"),
        SchemaType::List { elem, cap } => {
            out.push_str("list<");
            pretty_schema_inner(elem, indent, out);
            out.push('>');
            if let Some(limit) = cap {
                out.push_str(&format!(" cap={limit}"));
            }
        }
        SchemaType::Map { value, cap } => {
            out.push_str("map<string, ");
            pretty_schema_inner(value, indent, out);
            out.push('>');
            if let Some(limit) = cap {
                out.push_str(&format!(" cap={limit}"));
            }
        }
        SchemaType::Record { fields, open_row } => {
            out.push_str(if *open_row {
                "record(open) {\n"
            } else {
                "record(closed) {\n"
            });
            for (name, spec) in fields {
                out.push_str(&" ".repeat(indent + 2));
                out.push_str(name);
                if !spec.required {
                    out.push('?');
                }
                out.push_str(": ");
                pretty_schema_inner(&spec.ty, indent + 2, out);
                if let Some(default) = &spec.default {
                    out.push_str(" = ");
                    out.push_str(&value_brief(default));
                }
                if let Some(min) = spec.constraints.min_int {
                    out.push_str(&format!(" min={min}"));
                }
                if let Some(max) = spec.constraints.max_int {
                    out.push_str(&format!(" max={max}"));
                }
                if spec.constraints.nonempty {
                    out.push_str(" nonempty");
                }
                if let Some(pattern) = &spec.constraints.regex {
                    out.push_str(&format!(" regex={pattern}"));
                }
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push('}');
        }
        SchemaType::Enum { values } => {
            out.push_str("enum(");
            out.push_str(&values.join("|"));
            out.push(')');
        }
        SchemaType::Union { types } => {
            out.push_str("union(");
            for (idx, ty) in types.iter().enumerate() {
                if idx > 0 {
                    out.push_str(" | ");
                }
                pretty_schema_inner(ty, indent, out);
            }
            out.push(')');
        }
    }
}

pub fn schema_skeleton(schema: &SchemaType) -> String {
    let mut out = String::new();
    render_skeleton(schema, 0, &mut out);
    out
}

fn render_skeleton(schema: &SchemaType, indent: usize, out: &mut String) {
    match schema {
        SchemaType::Bool => out.push_str("false"),
        SchemaType::Int => out.push('0'),
        SchemaType::String => out.push_str("\"<string>\""),
        SchemaType::Null => out.push_str("null"),
        SchemaType::Bytes => out.push_str("\"<bytes>\""),
        SchemaType::List { .. } => out.push_str("[]"),
        SchemaType::Map { .. } => out.push_str("{}"),
        SchemaType::Enum { values } => {
            if let Some(first) = values.first() {
                out.push('"');
                out.push_str(first);
                out.push('"');
            } else {
                out.push_str("\"<enum>\"");
            }
        }
        SchemaType::Union { types } => {
            if let Some(first) = types.first() {
                render_skeleton(first, indent, out);
            } else {
                out.push_str("null");
            }
        }
        SchemaType::Record { fields, .. } => {
            out.push_str("{\n");
            let mut first = true;
            for (name, spec) in fields {
                if !spec.required {
                    continue;
                }
                if !first {
                    out.push_str(",\n");
                }
                first = false;
                out.push_str(&" ".repeat(indent + 2));
                out.push('"');
                out.push_str(name);
                out.push_str("\": ");
                if let Some(default) = &spec.default {
                    out.push_str(&value_brief(default));
                } else {
                    render_skeleton(&spec.ty, indent + 2, out);
                }
            }
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push('}');
        }
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "int",
        Value::Bool(_) => "bool",
        Value::String(_) => "string",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Budget(_) => "budget",
        Value::Ctx(_) => "ctx",
        Value::Payload(_) => "payload",
        Value::Result4(_) => "result4",
        Value::Unit => "unit",
        Value::Unknown => "unknown",
    }
}

fn schema_type_name(schema: &SchemaType) -> &'static str {
    match schema {
        SchemaType::Bool => "bool",
        SchemaType::Int => "int",
        SchemaType::String => "string",
        SchemaType::Null => "null",
        SchemaType::List { .. } => "list",
        SchemaType::Map { .. } => "map",
        SchemaType::Record { .. } => "record",
        SchemaType::Enum { .. } => "enum",
        SchemaType::Union { .. } => "union",
        SchemaType::Bytes => "bytes",
    }
}

fn value_brief(value: &Value) -> String {
    match value {
        Value::Int(v) => v.to_string(),
        Value::Bool(v) => v.to_string(),
        Value::String(v) => format!("\"{v}\""),
        Value::List(_) => "[]".to_string(),
        Value::Map(_) => "{}".to_string(),
        Value::Budget(v) => format!("budget({v})"),
        Value::Ctx(_) => "\"<ctx>\"".to_string(),
        Value::Payload(_) => "\"<payload>\"".to_string(),
        Value::Result4(_) => "\"<result4>\"".to_string(),
        Value::Unit => "null".to_string(),
        Value::Unknown => "\"<unknown>\"".to_string(),
    }
}
