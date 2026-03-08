use std::collections::BTreeMap;

use ocp::ocp::{
    pretty_schema, schema_skeleton, FieldConstraints, FieldSpec, SchemaType, Value,
};

#[test]
fn pretty_schema_is_stable_for_record() {
    let mut fields = BTreeMap::new();
    fields.insert(
        "path".to_string(),
        FieldSpec::required(SchemaType::String).with_constraints(FieldConstraints {
            nonempty: true,
            ..FieldConstraints::default()
        }),
    );
    fields.insert(
        "max_bytes".to_string(),
        FieldSpec::optional(SchemaType::Int)
            .with_default(Value::Int(100))
            .with_constraints(FieldConstraints {
                min_int: Some(1),
                max_int: Some(4096),
                ..FieldConstraints::default()
            }),
    );
    let schema = SchemaType::Record {
        fields,
        open_row: false,
    };

    let expected = "\
record(closed) {
  max_bytes?: int = 100 min=1 max=4096
  path: string nonempty
}";
    assert_eq!(pretty_schema(&schema), expected);
}

#[test]
fn schema_skeleton_outputs_required_shape_only() {
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

    let expected = "\
{
  \"path\": \"<string>\"
}";
    assert_eq!(schema_skeleton(&schema), expected);
}
