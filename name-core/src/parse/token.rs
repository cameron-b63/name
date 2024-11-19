#[derive(Debug, PartialEq)]
pub struct SrcSpan<'a> {
    pub start: usize,
    pub end: usize,
    pub src: &'a str,
    pub line: usize,
    pub line_pos: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Symbol,
    Register,
    Directive,

    HexNumber,
    BinaryNumber,
    OctalNumber,
    DecimalNumber,
    Fractional,

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

    Newline,
    EndOfFile,
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub src_span: SrcSpan<'a>,
}

impl<'a> Token<'a> {
    pub fn is_eof(&self) -> bool {
        match self.kind {
            TokenKind::EndOfFile => true,
            _ => false,
        }
    }
}
