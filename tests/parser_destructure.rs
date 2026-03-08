use ocp::ocp::{parse_program, Expr, LetPattern, Stmt};

#[test]
fn parse_let_record_destructure_ok() {
    let src = r#"
let rec = { a: 1, b: 2 };
let {a, b} = rec;
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    assert_eq!(program.statements.len(), 2);

    match &program.statements[1] {
        Stmt::Let { pattern, value, .. } => {
            assert_eq!(
                pattern,
                &LetPattern::Record(vec!["a".to_string(), "b".to_string()])
            );
            assert!(matches!(value, Expr::Ident { name, .. } if name == "rec"));
        }
        other => panic!("expected let destructure stmt, got {other:?}"),
    }
}

#[test]
fn parse_record_literal_and_map_literal_are_distinct() {
    let src = r#"
let r = { a: 1 };
let m = {"a": 1};
"#;
    let program = parse_program(src, 1).expect("parse should pass");
    assert_eq!(program.statements.len(), 2);

    match &program.statements[0] {
        Stmt::Let {
            value: Expr::Record { .. },
            ..
        } => {}
        other => panic!("expected record literal in first let, got {other:?}"),
    }

    match &program.statements[1] {
        Stmt::Let {
            value: Expr::Map { .. },
            ..
        } => {}
        other => panic!("expected map literal in second let, got {other:?}"),
    }
}
