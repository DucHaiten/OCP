use crate::ocp::ast::{Expr, LetPattern, MatchStmt, Program, Stmt};
use crate::ocp::lex::{lex, Token, TokenKind};
use crate::ocp::{DiagPhase, Diagnostic, ErrorCode, Span};

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
            TokenKind::Module => self.parse_module_decl(),
            TokenKind::Import => self.parse_import_decl(),
            TokenKind::Export => self.parse_export_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Fn => self.parse_fn_def(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Guard => self.parse_guard(),
            TokenKind::Repeat => self.parse_repeat(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Observe => self.parse_observe(),
            TokenKind::Commit => self.parse_commit(),
            TokenKind::Condition => self.parse_condition(),
            TokenKind::Entangle => self.parse_entangle(),
            TokenKind::Match => self.parse_match(),
            _ => Err(self.error_here(ErrorCode::PUnexpectedToken, "expected statement")),
        }
    }

    fn parse_module_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Module, "expected `module`")?.span;
        let path = self.parse_path("expected module path after `module`")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after module declaration")?;
        Ok(Stmt::ModuleDecl {
            path,
            span: merge_span(start, end.span),
        })
    }

    fn parse_import_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Import, "expected `import`")?.span;
        let path = self.parse_path("expected import path after `import`")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after import declaration")?;
        Ok(Stmt::ImportDecl {
            path,
            span: merge_span(start, end.span),
        })
    }

    fn parse_export_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Export, "expected `export`")?.span;
        let name = self.expect_ident("expected exported name after `export`")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after export declaration")?;
        Ok(Stmt::ExportDecl {
            name,
            span: merge_span(start, end.span),
        })
    }

    fn parse_struct_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Struct, "expected `struct`")?.span;
        let name = self.expect_ident("expected struct name")?;
        self.expect(TokenKind::LBrace, "expected `{` after struct name")?;
        let mut fields = Vec::new();
        if !self.at(TokenKind::RBrace) {
            loop {
                fields.push(self.expect_ident("expected struct field name")?);
                if self.consume_if(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        let rbrace_span = self
            .expect(TokenKind::RBrace, "expected `}` after struct fields")?
            .span;
        let end = self.expect(TokenKind::Semi, "expected `;` after struct declaration")?;
        Ok(Stmt::StructDecl {
            name,
            fields,
            span: merge_span(start, merge_span(rbrace_span, end.span)),
        })
    }

    fn parse_enum_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Enum, "expected `enum`")?.span;
        let name = self.expect_ident("expected enum name")?;
        self.expect(TokenKind::LBrace, "expected `{` after enum name")?;
        let mut variants = Vec::new();
        if !self.at(TokenKind::RBrace) {
            loop {
                variants.push(self.expect_ident("expected enum variant name")?);
                if self.consume_if(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        let rbrace_span = self
            .expect(TokenKind::RBrace, "expected `}` after enum variants")?
            .span;
        let end = self.expect(TokenKind::Semi, "expected `;` after enum declaration")?;
        Ok(Stmt::EnumDecl {
            name,
            variants,
            span: merge_span(start, merge_span(rbrace_span, end.span)),
        })
    }

    fn parse_fn_def(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Fn, "expected `fn`")?.span;
        let name = self.expect_ident("expected function name")?;
        self.expect(TokenKind::LParen, "expected `(` after function name")?;
        let mut params = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                params.push(self.expect_ident("expected function parameter name")?);
                if self.consume_if(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen, "expected `)` after function parameters")?;
        self.expect(TokenKind::LBrace, "expected `{` to open function body")?;
        let mut body = Vec::new();
        while !self.at(TokenKind::RBrace) {
            body.push(self.parse_stmt()?);
        }
        let rbrace = self.expect(TokenKind::RBrace, "expected `}` to close function body")?;
        Ok(Stmt::FnDef {
            name,
            params,
            body,
            span: merge_span(start, rbrace.span),
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Return, "expected `return`")?.span;
        let value = self.parse_expr()?;
        let end = self.expect(TokenKind::Semi, "expected `;` after return")?;
        Ok(Stmt::Return {
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_guard(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Guard, "expected `guard`")?.span;
        let value = self.parse_expr()?;
        let end = self.expect(TokenKind::Semi, "expected `;` after guard")?;
        Ok(Stmt::Guard {
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_repeat(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Repeat, "expected `repeat`")?.span;
        let count = self.parse_expr()?;
        let body = self.parse_block()?;
        let end_span = body.last().map(stmt_span).unwrap_or(count.span());
        Ok(Stmt::Repeat {
            count,
            body,
            span: merge_span(start, end_span),
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::For, "expected `for`")?.span;
        let var = self.expect_ident("expected loop variable after `for`")?;
        self.expect(TokenKind::In, "expected `in` after loop variable")?;
        let first_expr = self.parse_expr()?;

        if self.consume_if(TokenKind::Range) {
            let end_expr = self.parse_expr()?;
            let body = self.parse_block()?;
            let end_span = body.last().map(stmt_span).unwrap_or(first_expr.span());
            return Ok(Stmt::ForRange {
                var,
                start: first_expr,
                end: end_expr,
                body,
                span: merge_span(start, end_span),
            });
        }

        self.expect(TokenKind::Cap, "expected `cap` in bounded for loop")?;
        let cap = self.parse_expr()?;
        let body = self.parse_block()?;
        let end_span = body.last().map(stmt_span).unwrap_or(first_expr.span());
        Ok(Stmt::ForEachCap {
            var,
            iter: first_expr,
            cap,
            body,
            span: merge_span(start, end_span),
        })
    }

    fn parse_let(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.expect(TokenKind::Let, "expected `let`")?.span;
        let pattern = self.parse_let_pattern()?;
        self.expect(TokenKind::Eq, "expected `=` after let name")?;
        if let LetPattern::Ident(name) = &pattern {
            if self.at(TokenKind::TryKw) {
                return self.parse_try_let(name.clone(), start);
            }
        }
        let value = self.parse_expr()?;
        let end = self.expect(TokenKind::Semi, "expected `;` after let statement")?;
        Ok(Stmt::Let {
            pattern,
            value,
            span: merge_span(start, end.span),
        })
    }

    fn parse_let_pattern(&mut self) -> Result<LetPattern, Diagnostic> {
        if self.consume_if(TokenKind::LBrace) {
            let mut fields = Vec::new();
            if !self.at(TokenKind::RBrace) {
                loop {
                    fields.push(self.expect_ident("expected field name in let destructure")?);
                    if self.consume_if(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect(
                TokenKind::RBrace,
                "expected `}` after let destructure pattern",
            )?;
            return Ok(LetPattern::Record(fields));
        }
        Ok(LetPattern::Ident(
            self.expect_ident("expected variable name after `let`")?,
        ))
    }

    fn parse_try_let(&mut self, name: String, start: Span) -> Result<Stmt, Diagnostic> {
        self.expect(TokenKind::TryKw, "expected `try` after `let <name> =`")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Else, "expected `else` in try/else let")?;
        self.expect(TokenKind::LBrace, "expected `{` to open try/else block")?;

        let (else_returns, else_expr) = if self.consume_if(TokenKind::Return) {
            let expr = self.parse_expr()?;
            self.expect(
                TokenKind::Semi,
                "expected `;` after return expression in try/else block",
            )?;
            (true, expr)
        } else {
            let expr = self.parse_expr()?;
            (false, expr)
        };

        self.expect(TokenKind::RBrace, "expected `}` to close try/else block")?;
        let end = self.expect(TokenKind::Semi, "expected `;` after try/else let")?;
        Ok(Stmt::TryLet {
            name,
            value,
            else_expr,
            else_returns,
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
        loop {
            if self.consume_if(TokenKind::Dot) {
                let field_tok =
                    self.expect_kind(TokenKind::Ident, "expected field name after `.`")?;
                let span = merge_span(expr.span(), field_tok.span);
                expr = Expr::FieldAccess {
                    base: Box::new(expr),
                    field: field_tok.text,
                    span,
                };
                continue;
            }
            if self.consume_if(TokenKind::Question) {
                let span = merge_span(expr.span(), self.peek().span);
                expr = Expr::Try {
                    value: Box::new(expr),
                    span,
                };
                continue;
            }
            break;
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
                } else if self.lookahead_namespaced_call_after_ident() {
                    let mut callee = tok.text;
                    while self.consume_if(TokenKind::Dot) {
                        let seg =
                            self.expect_kind(TokenKind::Ident, "expected name segment after `.`")?;
                        callee.push('.');
                        callee.push_str(&seg.text);
                    }
                    self.expect(
                        TokenKind::LParen,
                        "expected `(` after namespaced call target",
                    )?;
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
                        callee,
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
            TokenKind::LBracket => {
                let mut items = Vec::new();
                if !self.at(TokenKind::RBracket) {
                    loop {
                        items.push(self.parse_expr()?);
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBracket, "expected `]` after list literal")?;
                Ok(Expr::List {
                    items,
                    span: merge_span(tok.span, end.span),
                })
            }
            TokenKind::LBrace => {
                if self.at(TokenKind::RBrace) {
                    let end = self.expect(TokenKind::RBrace, "expected `}` after map literal")?;
                    return Ok(Expr::Map {
                        entries: Vec::new(),
                        span: merge_span(tok.span, end.span),
                    });
                }

                if self.at(TokenKind::Str) {
                    let mut entries = Vec::new();
                    loop {
                        let key =
                            self.expect_kind(TokenKind::Str, "expected string key in map literal")?;
                        self.expect(TokenKind::Colon, "expected `:` after map key")?;
                        let value = self.parse_expr()?;
                        entries.push((key.text, value));
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                    let end = self.expect(TokenKind::RBrace, "expected `}` after map literal")?;
                    Ok(Expr::Map {
                        entries,
                        span: merge_span(tok.span, end.span),
                    })
                } else if self.at(TokenKind::Ident) {
                    let mut fields = Vec::new();
                    loop {
                        let key = self.expect_ident("expected field name in record literal")?;
                        self.expect(TokenKind::Colon, "expected `:` after record field")?;
                        let value = self.parse_expr()?;
                        fields.push((key, value));
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                    let end =
                        self.expect(TokenKind::RBrace, "expected `}` after record literal")?;
                    Ok(Expr::Record {
                        fields,
                        span: merge_span(tok.span, end.span),
                    })
                } else {
                    Err(self.error_here(
                        ErrorCode::PUnexpectedToken,
                        "expected string-key map or record literal fields",
                    ))
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

    fn parse_path(&mut self, message: &str) -> Result<Vec<String>, Diagnostic> {
        let mut path = vec![self.expect_ident(message)?];
        while self.consume_if(TokenKind::Dot) {
            path.push(self.expect_ident("expected identifier segment after `.`")?);
        }
        Ok(path)
    }

    fn lookahead_namespaced_call_after_ident(&self) -> bool {
        let mut i = self.idx;
        let mut saw_dot = false;
        while i + 1 < self.tokens.len() {
            if self.tokens[i].kind != TokenKind::Dot {
                break;
            }
            if self.tokens[i + 1].kind != TokenKind::Ident {
                return false;
            }
            saw_dot = true;
            i += 2;
        }
        saw_dot && i < self.tokens.len() && self.tokens[i].kind == TokenKind::LParen
    }

    fn error_here(&self, code: ErrorCode, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(code, DiagPhase::Parse, self.peek().span, message)
            .with_hint("check syntax near current token")
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

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::ModuleDecl { span, .. }
        | Stmt::ImportDecl { span, .. }
        | Stmt::ExportDecl { span, .. }
        | Stmt::StructDecl { span, .. }
        | Stmt::EnumDecl { span, .. }
        | Stmt::FnDef { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::TryLet { span, .. }
        | Stmt::Guard { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::ForEachCap { span, .. }
        | Stmt::ForRange { span, .. }
        | Stmt::Observe { span, .. }
        | Stmt::Commit { span, .. }
        | Stmt::Condition { span, .. }
        | Stmt::Entangle { span, .. } => *span,
        Stmt::Match(m) => m.span,
    }
}
