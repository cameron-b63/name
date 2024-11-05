#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Name,
    Register,
    Directive,

    HexNumber,
    BinaryNumber,
    OctalNumber,
    DecimalNumber,

    String,
    Char,

    DoubleQuote,
    SingleQuote,

    // Punctuation
    Colon,
    Comma,
    Period,

    LParen,
    RParen,

    Minus,
    Plus,

    EndOfFile,
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub src: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, src: &'a str) -> Self {
        Token { kind, src }
    }
}
