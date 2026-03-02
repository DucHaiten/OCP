use ocp_ocl::ocp_ocl::{ConformanceProfile, CtxFieldType, CORE_SEMANTIC_CTX_FIELDS};

#[test]
fn conformance_profiles_are_frozen_for_v2_a() {
    assert_eq!(ConformanceProfile::P0Core.as_str(), "P0");
    assert_eq!(ConformanceProfile::P1Extended.as_str(), "P1");
}

#[test]
fn core_semantic_ctx_fields_are_frozen_for_v2_a() {
    assert_eq!(CORE_SEMANTIC_CTX_FIELDS.len(), 4);

    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[0].name, "seed");
    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[0].ty, CtxFieldType::U64);

    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[1].name, "tick");
    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[1].ty, CtxFieldType::U64);

    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[2].name, "observer_id");
    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[2].ty, CtxFieldType::U64);

    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[3].name, "policy_id");
    assert_eq!(CORE_SEMANTIC_CTX_FIELDS[3].ty, CtxFieldType::U32);
}
