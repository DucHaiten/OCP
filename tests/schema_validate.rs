use std::collections::BTreeMap;

use ocp_ocl::ocp_ocl::{
    validate_schema_value, FieldConstraints, FieldSpec, SchemaIssueCode, SchemaType, Value,
};

fn map(entries: &[(&str, Value)]) -> Value {
    let mut out = BTreeMap::new();
    for (k, v) in entries {
        out.insert((*k).to_string(), v.clone());
    }
    Value::Map(out)
}

#[test]
fn validator_catches_missing_unknown_and_type_mismatch() {
    let mut fields = BTreeMap::new();
    fields.insert("path".to_string(), FieldSpec::required(SchemaType::String));
    fields.insert("cap".to_string(), FieldSpec::required(SchemaType::Int));
    let schema = SchemaType::Record {
        fields,
        open_row: false,
    };

    let value = map(&[("path", Value::Int(1)), ("extra", Value::Bool(true))]);
    let issues = validate_schema_value(&schema, &value).expect_err("validation must fail");

    assert!(issues
        .iter()
        .any(|i| i.code == SchemaIssueCode::MissingField && i.path == "$"));
    assert!(issues
        .iter()
        .any(|i| i.code == SchemaIssueCode::UnknownField && i.path == "$"));
    assert!(issues
        .iter()
        .any(|i| i.code == SchemaIssueCode::TypeMismatch && i.path == "$.path"));
}

#[test]
fn validator_accepts_optional_with_default() {
    let mut fields = BTreeMap::new();
    fields.insert("path".to_string(), FieldSpec::required(SchemaType::String));
    fields.insert(
        "max_bytes".to_string(),
        FieldSpec::optional(SchemaType::Int).with_default(Value::Int(1024)),
    );
    let schema = SchemaType::Record {
        fields,
        open_row: false,
    };

    let value = map(&[("path", Value::String("./a.txt".to_string()))]);
    validate_schema_value(&schema, &value).expect("validation should pass");
}

#[test]
fn validator_enforces_caps_and_constraints() {
    let list_schema = SchemaType::List {
        elem: Box::new(SchemaType::Int),
        cap: Some(2),
    };
    let list_value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let list_err = validate_schema_value(&list_schema, &list_value).expect_err("must fail cap");
    assert!(list_err
        .iter()
        .any(|i| i.code == SchemaIssueCode::ConstraintViolation && i.path == "$"));

    let mut fields = BTreeMap::new();
    fields.insert(
        "name".to_string(),
        FieldSpec::required(SchemaType::String).with_constraints(FieldConstraints {
            nonempty: true,
            ..FieldConstraints::default()
        }),
    );
    fields.insert(
        "retries".to_string(),
        FieldSpec::required(SchemaType::Int).with_constraints(FieldConstraints {
            min_int: Some(1),
            max_int: Some(5),
            ..FieldConstraints::default()
        }),
    );
    let record_schema = SchemaType::Record {
        fields,
        open_row: false,
    };
    let record_value = map(&[
        ("name", Value::String(String::new())),
        ("retries", Value::Int(9)),
    ]);
    let record_err =
        validate_schema_value(&record_schema, &record_value).expect_err("constraints must fail");
    assert_eq!(
        record_err
            .iter()
            .filter(|i| i.code == SchemaIssueCode::ConstraintViolation)
            .count(),
        2
    );
}
