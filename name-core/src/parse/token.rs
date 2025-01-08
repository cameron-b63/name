use crate::parse::span::Span;

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

    Newline,
}

pub type Token<'a> = Span<'a, TokenKind>;

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
