#[path = "w17_gate_c_common.rs"]
mod w17;

use ocp::ocp::{canonicalize_text_boundary, preserve_runtime_literal, DeterminismError};

#[test]
fn unicode_and_newline_canonicalization_is_stable_at_boundaries() {
    let input = b"\xEF\xBB\xBFCafe\xCC\x81\r\nline\rnext";
    let normalized = canonicalize_text_boundary(input).expect("boundary normalization should pass");
    assert_eq!(normalized, "Café\nline\nnext");
    w17::write_gate_c_report();
}

#[test]
fn unicode_canonicalization_rejects_invalid_utf8() {
    let err = canonicalize_text_boundary(&[0xFF, 0xFE]).expect_err("invalid utf8 must fail");
    assert!(
        matches!(err, DeterminismError::InvalidUtf8),
        "unexpected error: {err}"
    );
    w17::write_gate_c_report();
}

#[test]
fn runtime_literals_are_not_blindly_normalized() {
    let literal = "Cafe\u{301}";
    let preserved = preserve_runtime_literal(literal);
    assert_eq!(preserved, literal, "runtime literal must be preserved");
    assert_ne!(
        canonicalize_text_boundary(literal.as_bytes()).expect("canonicalize boundary"),
        preserved,
        "boundary canonicalization must stay separate from runtime literal semantics"
    );
    w17::write_gate_c_report();
}
