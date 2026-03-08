use crate::ocp::{DiagPhase, Diagnostic, ErrorCode, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Module,
    Import,
    Export,
    Struct,
    Enum,
    Fn,
    Return,
    TryKw,
    Else,
    Guard,
    Repeat,
    For,
    In,
    Cap,
    Let,
    Observe,
    Commit,
    Match,
    Condition,
    Entangle,
    Ok,
    Degraded,
    Insufficient,
    Deferred,
    True,
    False,
    Ident,
    Int,
    Str,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Dot,
    Range,
    Question,
    Eq,
    Arrow,
    FatArrow,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

pub fn lex(source: &str, file_id: u32) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer::new(source, file_id);
    lx.lex_all()
}

struct Lexer<'a> {
    source: &'a str,
    file_id: u32,
    idx: usize,
    line: u32,
    col: u32,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, file_id: u32) -> Self {
        Self {
            source,
            file_id,
            idx: 0,
            line: 1,
            col: 1,
        }
    }

    fn lex_all(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            let start = self.mark();
            let Some(ch) = self.peek() else {
                out.push(Token {
                    kind: TokenKind::Eof,
                    text: String::new(),
                    span: self.span_from(start),
                });
                break;
            };

            match ch {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let text = self.read_ident();
                    let kind = match text.as_str() {
                        "module" => TokenKind::Module,
                        "import" => TokenKind::Import,
                        "export" => TokenKind::Export,
                        "struct" => TokenKind::Struct,
                        "enum" => TokenKind::Enum,
                        "fn" => TokenKind::Fn,
                        "return" => TokenKind::Return,
                        "try" => TokenKind::TryKw,
                        "else" => TokenKind::Else,
                        "guard" => TokenKind::Guard,
                        "repeat" => TokenKind::Repeat,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "cap" => TokenKind::Cap,
                        "let" => TokenKind::Let,
                        "observe" => TokenKind::Observe,
                        "commit" => TokenKind::Commit,
                        "match" => TokenKind::Match,
                        "condition" => TokenKind::Condition,
                        "entangle" => TokenKind::Entangle,
                        "OK" => TokenKind::Ok,
                        "DEGRADED" => TokenKind::Degraded,
                        "INSUFFICIENT" => TokenKind::Insufficient,
                        "DEFERRED" => TokenKind::Deferred,
                        "true" => TokenKind::True,
                        "false" => TokenKind::False,
                        _ => TokenKind::Ident,
                    };
                    out.push(Token {
                        kind,
                        text,
                        span: self.span_from(start),
                    });
                }
                '0'..='9' => {
                    let text = self.read_int();
                    out.push(Token {
                        kind: TokenKind::Int,
                        text,
                        span: self.span_from(start),
                    });
                }
                '"' => {
                    let text = self.read_string(start)?;
                    out.push(Token {
                        kind: TokenKind::Str,
                        text,
                        span: self.span_from(start),
                    });
                }
                '(' => out.push(self.single(TokenKind::LParen)),
                ')' => out.push(self.single(TokenKind::RParen)),
                '{' => out.push(self.single(TokenKind::LBrace)),
                '}' => out.push(self.single(TokenKind::RBrace)),
                '[' => out.push(self.single(TokenKind::LBracket)),
                ']' => out.push(self.single(TokenKind::RBracket)),
                ',' => out.push(self.single(TokenKind::Comma)),
                ';' => out.push(self.single(TokenKind::Semi)),
                ':' => out.push(self.single(TokenKind::Colon)),
                '?' => out.push(self.single(TokenKind::Question)),
                '.' => {
                    self.bump();
                    if self.peek() == Some('.') {
                        self.bump();
                        out.push(Token {
                            kind: TokenKind::Range,
                            text: "..".to_string(),
                            span: self.span_from(start),
                        });
                    } else {
                        out.push(Token {
                            kind: TokenKind::Dot,
                            text: ".".to_string(),
                            span: self.span_from(start),
                        });
                    }
                }
                '=' => {
                    self.bump();
                    if self.peek() == Some('>') {
                        self.bump();
                        out.push(Token {
                            kind: TokenKind::FatArrow,
                            text: "=>".to_string(),
                            span: self.span_from(start),
                        });
                    } else {
                        out.push(Token {
                            kind: TokenKind::Eq,
                            text: "=".to_string(),
                            span: self.span_from(start),
                        });
                    }
                }
                '-' => {
                    self.bump();
                    if self.peek() == Some('>') {
                        self.bump();
                        out.push(Token {
                            kind: TokenKind::Arrow,
                            text: "->".to_string(),
                            span: self.span_from(start),
                        });
                    } else {
                        return Err(self.error_at(
                            ErrorCode::PUnexpectedToken,
                            start,
                            "expected '->'",
                        ));
                    }
                }
                _ => {
                    return Err(self.error_at(
                        ErrorCode::PUnexpectedToken,
                        start,
                        format!("unexpected character: '{ch}'"),
                    ));
                }
            }
        }
        Ok(out)
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let start = self.mark();
        let ch = self.bump().expect("single token needs one char");
        Token {
            kind,
            text: ch.to_string(),
            span: self.span_from(start),
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.idx;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.bump();
            } else {
                break;
            }
        }
        self.source[start..self.idx].to_string()
    }

    fn read_int(&mut self) -> String {
        let start = self.idx;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }
        self.source[start..self.idx].to_string()
    }

    fn read_string(&mut self, start: Mark) -> Result<String, Diagnostic> {
        self.bump();
        let mut out = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.bump();
                    return Ok(out);
                }
                Some('\\') => {
                    self.bump();
                    let Some(next) = self.peek() else {
                        return Err(self.error_at(
                            ErrorCode::PUnexpectedEof,
                            start,
                            "unterminated string literal",
                        ));
                    };
                    self.bump();
                    let escaped = match next {
                        '"' => '"',
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        other => other,
                    };
                    out.push(escaped);
                }
                Some(ch) => {
                    self.bump();
                    out.push(ch);
                }
                None => {
                    return Err(self.error_at(
                        ErrorCode::PUnexpectedEof,
                        start,
                        "unterminated string literal",
                    ));
                }
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.idx..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.idx += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn mark(&self) -> Mark {
        Mark {
            idx: self.idx as u32,
            line: self.line,
            col: self.col,
        }
    }

    fn span_from(&self, start: Mark) -> Span {
        Span::new(
            self.file_id,
            start.idx,
            self.idx as u32,
            start.line,
            start.col,
        )
    }

    fn error_at(&self, code: ErrorCode, start: Mark, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(code, DiagPhase::Parse, self.span_from(start), message)
    }
}

#[derive(Debug, Clone, Copy)]
struct Mark {
    idx: u32,
    line: u32,
    col: u32,
}
