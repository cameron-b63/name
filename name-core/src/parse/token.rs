use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct SrcSpan<'a> {
    pub start: usize,
    pub end: usize,
    pub src: &'a str,
    pub line: usize,
    pub line_pos: usize,
}

impl fmt::Display for SrcSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.line_pos, self.src)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Ident,
    Register,
    Directive,

    // whole numbers
    HexNumber,
    BinaryNumber,
    OctalNumber,
    DecimalNumber,

    // need to figure these out
    Float,

    String,
    Char,

    // Punctuation
    Colon,
    Comma,
    Period,

    LParen,
    RParen,

    Minus,
    Plus,

    Percent,

    Newline,
}

impl TokenKind {
    // Token is a whole number type or start of one
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            TokenKind::BinaryNumber
                | TokenKind::OctalNumber
                | TokenKind::DecimalNumber
                | TokenKind::Minus
        )
    }

    pub fn is_literal(&self) -> bool {
        self.is_number() || *self == TokenKind::Char
    }

    // Token can appear in an immediates position
    pub fn is_immediate(&self) -> bool {
        self.is_literal() || *self == TokenKind::Ident
    }
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub src_span: SrcSpan<'a>,
}

impl<'a> Token<'a> {
    pub fn src_string(&self) -> String {
        self.src_span.src.to_string()
    }

    pub fn is_kind(&self, tk: TokenKind) -> bool {
        self.kind == tk
    }
}
