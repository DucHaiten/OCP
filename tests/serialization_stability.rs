use ocp_sdk::{canonical_json_string_v17, canonical_json_value_v17};
use serde_json::json;

#[test]
fn v17_canonical_json_is_stable_and_sorted_by_utf8_bytes() {
    let source = json!({
        "zeta": 1,
        "Alpha": 2,
        "nested": {
            "b": 1,
            "a": 2
        },
        "arr": [
            { "y": 1, "x": 2 }
        ]
    });
    let first = canonical_json_string_v17(&source).expect("first canonical json");
    let second = canonical_json_string_v17(&source).expect("second canonical json");
    assert_eq!(first, second, "canonical json must be deterministic");
    assert_eq!(
        first,
        "{\"Alpha\":2,\"arr\":[{\"x\":2,\"y\":1}],\"nested\":{\"a\":2,\"b\":1},\"zeta\":1}"
    );
}

#[test]
fn v17_canonical_json_value_recursively_sorts_object_keys() {
    let source = json!({
        "m": {"z": 1, "a": 2},
        "a": {"d": 4, "c": 3}
    });
    let canonical = canonical_json_value_v17(&source);
    assert_eq!(
        canonical,
        json!({
            "a": {"c": 3, "d": 4},
            "m": {"a": 2, "z": 1}
        })
    );
}
