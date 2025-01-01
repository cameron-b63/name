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
    Fractional,

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

    // Token can appear in an immediates position
    pub fn is_immediate(&self) -> bool {
        matches!(self, TokenKind::Ident | TokenKind::Char) || self.is_number()
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
