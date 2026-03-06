use ocl_sdk::{
    artifact_hash_from_bytes_v17, semantic_hash_from_bytes_v17, W17_ARTIFACT_HASH_VERSION,
    W17_SEMANTIC_HASH_VERSION,
};

#[test]
fn v17_semantic_hash_ignores_diagnostic_text() {
    let semantic_bytes = b"canonical-ast:v1";
    let h1 = semantic_hash_from_bytes_v17(semantic_bytes, "diag-a");
    let h2 = semantic_hash_from_bytes_v17(semantic_bytes, "diag-b");

    assert_eq!(
        h1, h2,
        "semantic hash must stay stable when only diagnostic text changes"
    );
}

#[test]
fn v17_artifact_hash_changes_when_diagnostic_changes() {
    let artifact_bytes = b"artifact-layout:v1";
    let h1 = artifact_hash_from_bytes_v17(artifact_bytes, "diag-a");
    let h2 = artifact_hash_from_bytes_v17(artifact_bytes, "diag-b");

    assert_ne!(
        h1, h2,
        "artifact hash must capture diagnostic/presentation bytes when present"
    );
}

#[test]
fn v17_hash_versions_are_locked() {
    assert_eq!(W17_SEMANTIC_HASH_VERSION, "semantic_hash_v1");
    assert_eq!(W17_ARTIFACT_HASH_VERSION, "artifact_hash_v1");
}
