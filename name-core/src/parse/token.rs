#[derive(Debug, PartialEq)]
pub struct SrcSpan<'a> {
    pub start: usize,
    pub end: usize,
    pub src: &'a str,
    pub line: usize,
    pub line_pos: usize,
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
    EndOfFile,
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

    pub fn is_eof(&self) -> bool {
        match self.kind {
            TokenKind::EndOfFile => true,
            _ => false,
        }
    }

    pub fn is_section_directive(&self) -> bool {
        matches!(self.src_span.src, ".data" | ".text" | ".ktext")
    }

    pub fn is_data_directive(&self) -> bool {
        matches!(self.src_span.src, ".asciiz" | ".word" | ".byte")
    }
}
