use crate::ocp_ocl::ast::{Expr, MatchStmt, Program, Stmt};
use crate::ocp_ocl::lex::{lex, Token, TokenKind};
use crate::ocp_ocl::{DiagPhase, Diagnostic, ErrorCode, Span};

pub fn parse_program(source: &str, file_id: u32) -> Result<Program, Diagnostic> {
    let tokens = lex(source, file_id)?;
    Parser::new(tokens).parse_program()
}

pub struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, idx: 0 }
    }

    pub fn parse_program(mut self) -> Result<Program, Diagnostic> {
        let mut statements = Vec::new();
        while !self.at(TokenKind::Eof) {
            statements.push(self.parse_stmt()?);
        }
        Ok(Program { statements })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek().kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Observe => self.parse_observe(),
            TokenKind::Commit => self.parse_commit(),
            TokenKind::Condition => self.parse_condition(),
            TokenKind::Entangle => self.parse_entangle(),
            TokenKind::Match => self.parse_match(),
            _ => Err(self.error_here(
                ErrorCode::PUnexpectedToken,
                "expected statement (`let`, `observe`, `commit`, `condition`, `entangle`, `match`)",
            )),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Let, "expected `let`")?.span;
        let name = self.expect_ident("expected variable name after `let`")?;
        self.expect(TokenKind::Eq, "expected `=` after let name")?;
        let value = self.parse_expr()?;
        let end = self.expect(TokenKind::Semi, "expected `;` after let statement")?;
        Ok(Stmt::Let {
            name,
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_observe(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Observe, "expected `observe`")?.span;
        self.expect(TokenKind::LParen, "expected `(` after observe")?;
        let key = self.parse_expr()?;
        self.expect(TokenKind::Comma, "expected `,` after observe key")?;
        let tier = self.parse_expr()?;
        self.expect(TokenKind::Comma, "expected `,` after observe tier")?;
        let ctx = self.parse_expr()?;
        self.expect(TokenKind::Comma, "expected `,` after observe ctx")?;
        let budget = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected `)` to close observe args")?;
        self.expect(TokenKind::Arrow, "expected `->` after observe(...)")?;
        let bind = self.expect_ident("expected binding identifier after `->`")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after observe statement")?;
        Ok(Stmt::Observe {
            key,
            tier,
            ctx,
            budget,
            bind,
            span: merge_span(start, end.span),
        })
    }

    fn parse_commit(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Commit, "expected `commit`")?.span;
        self.expect(TokenKind::LParen, "expected `(` after commit")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected `)` after commit expr")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after commit statement")?;
        Ok(Stmt::Commit {
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_condition(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self
            .expect(TokenKind::Condition, "expected `condition`")?
            .span;
        self.expect(TokenKind::LParen, "expected `(` after condition")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected `)` after condition expr")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after condition statement")?;
        Ok(Stmt::Condition {
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_entangle(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self
            .expect(TokenKind::Entangle, "expected `entangle`")?
            .span;
        self.expect(TokenKind::LParen, "expected `(` after entangle")?;
        let left = self.expect_ident("expected left binding identifier in entangle")?;
        self.expect(TokenKind::Comma, "expected `,` after entangle left binding")?;
        let right = self.expect_ident("expected right binding identifier in entangle")?;
        self.expect(
            TokenKind::Comma,
            "expected `,` after entangle right binding",
        )?;
        let constraint = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected `)` after entangle args")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after entangle statement")?;
        Ok(Stmt::Entangle {
            left,
            right,
            constraint,
            span: merge_span(start, end.span),
        })
    }

    fn parse_match(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Match, "expected `match`")?.span;
        let value = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "expected `{` after match expr")?;

        let mut ok_arm: Option<Vec<Stmt>> = None;
        let mut degraded_arm: Option<Vec<Stmt>> = None;
        let mut insufficient_arm: Option<Vec<Stmt>> = None;
        let mut deferred_arm: Option<Vec<Stmt>> = None;

        while !self.at(TokenKind::RBrace) {
            let arm_kind = self.peek().kind;
            self.advance();
            self.expect(TokenKind::FatArrow, "expected `=>` after match arm")?;
            let arm_body = self.parse_block()?;

            match arm_kind {
                TokenKind::Ok => set_arm(&mut ok_arm, arm_body, "OK", self.peek().span)?,
                TokenKind::Degraded => {
                    set_arm(&mut degraded_arm, arm_body, "DEGRADED", self.peek().span)?
                }
                TokenKind::Insufficient => set_arm(
                    &mut insufficient_arm,
                    arm_body,
                    "INSUFFICIENT",
                    self.peek().span,
                )?,
                TokenKind::Deferred => {
                    set_arm(&mut deferred_arm, arm_body, "DEFERRED", self.peek().span)?
                }
                _ => {
                    return Err(self.error_here(
                        ErrorCode::PUnexpectedToken,
                        "expected match arm `OK|DEGRADED|INSUFFICIENT|DEFERRED`",
                    ));
                }
            }
        }

        let end = self.expect(TokenKind::RBrace, "expected `}` to close match")?;

        if ok_arm.is_none()
            || degraded_arm.is_none()
            || insufficient_arm.is_none()
            || deferred_arm.is_none()
        {
            return Err(Diagnostic::new(
                ErrorCode::PUnexpectedToken,
                DiagPhase::Parse,
                end.span,
                "match must contain all 4 arms: OK, DEGRADED, INSUFFICIENT, DEFERRED",
            ));
        }

        Ok(Stmt::Match(MatchStmt {
            value,
            ok_arm: ok_arm.unwrap_or_default(),
            degraded_arm: degraded_arm.unwrap_or_default(),
            insufficient_arm: insufficient_arm.unwrap_or_default(),
            deferred_arm: deferred_arm.unwrap_or_default(),
            span: merge_span(start, end.span),
        }))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.expect(TokenKind::LBrace, "expected `{` to open block")?;
        let mut stmts = Vec::new();
        while !self.at(TokenKind::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace, "expected `}` to close block")?;
        Ok(stmts)
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;
        while self.consume_if(TokenKind::Dot) {
            let field_tok = self.expect_kind(TokenKind::Ident, "expected field name after `.`")?;
            let span = merge_span(expr.span(), field_tok.span);
            expr = Expr::FieldAccess {
                base: Box::new(expr),
                field: field_tok.text,
                span,
            };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Int => {
                let value = tok.text.parse::<i64>().map_err(|_| {
                    Diagnostic::new(
                        ErrorCode::PInvalidLiteral,
                        DiagPhase::Parse,
                        tok.span,
                        format!("invalid integer literal: {}", tok.text),
                    )
                })?;
                Ok(Expr::Int {
                    value,
                    span: tok.span,
                })
            }
            TokenKind::True => Ok(Expr::Bool {
                value: true,
                span: tok.span,
            }),
            TokenKind::False => Ok(Expr::Bool {
                value: false,
                span: tok.span,
            }),
            TokenKind::Str => Ok(Expr::String {
                value: tok.text,
                span: tok.span,
            }),
            TokenKind::Ident => {
                if self.consume_if(TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !self.at(TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.consume_if(TokenKind::Comma) {
                                continue;
                            }
                            break;
                        }
                    }
                    let rparen = self.expect(TokenKind::RParen, "expected `)` after call args")?;
                    Ok(Expr::Call {
                        callee: tok.text,
                        args,
                        span: merge_span(tok.span, rparen.span),
                    })
                } else {
                    Ok(Expr::Ident {
                        name: tok.text,
                        span: tok.span,
                    })
                }
            }
            _ => Err(Diagnostic::new(
                ErrorCode::PUnexpectedToken,
                DiagPhase::Parse,
                tok.span,
                format!("expected expression, found {:?}", tok.kind),
            )),
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.idx]
    }

    fn advance(&mut self) -> &Token {
        let i = self.idx;
        if self.idx < self.tokens.len() - 1 {
            self.idx += 1;
        }
        &self.tokens[i]
    }

    fn consume_if(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<&Token, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(self.error_here(ErrorCode::PUnexpectedToken, message))
        }
    }

    fn expect_kind(&mut self, kind: TokenKind, message: &str) -> Result<Token, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_here(ErrorCode::PUnexpectedToken, message))
        }
    }

    fn expect_ident(&mut self, message: &str) -> Result<String, Diagnostic> {
        let tok = self.expect_kind(TokenKind::Ident, message)?;
        Ok(tok.text)
    }

    fn error_here(&self, code: ErrorCode, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(code, DiagPhase::Parse, self.peek().span, message)
    }
}

fn merge_span(a: Span, b: Span) -> Span {
    Span::new(a.file_id, a.start, b.end, a.line, a.column)
}

fn set_arm(
    slot: &mut Option<Vec<Stmt>>,
    body: Vec<Stmt>,
    arm_name: &str,
    span: Span,
) -> Result<(), Diagnostic> {
    if slot.is_some() {
        return Err(Diagnostic::new(
            ErrorCode::PUnexpectedToken,
            DiagPhase::Parse,
            span,
            format!("duplicate match arm `{arm_name}`"),
        ));
    }
    *slot = Some(body);
    Ok(())
}
