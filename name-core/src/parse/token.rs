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

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub token: Span<TokenKind>,
    pub src: &'a str,
}

#[derive(Debug, Clone)]
pub struct TokenCursor<'a> {
    idx: usize,
    toks: Vec<Token<'a>>,
}

impl<'a> TokenCursor<'a> {
    pub fn new(toks: Vec<Token<'a>>) -> Self {
        TokenCursor { idx: 0, toks }
    }

    pub fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn next(&mut self) -> Option<&Token<'a>> {
        let tok = self.toks.get(self.idx);
        self.idx += 1;
        tok
    }

    pub fn get(&self, i: usize) -> Option<&Token<'a>> {
        self.toks.get(i)
    }

    pub fn peek(&self) -> Option<&Token<'a>> {
        self.toks.get(self.idx)
    }

    pub fn prev(&self) -> Option<&Token<'a>> {
        self.toks.get(self.idx.checked_sub(1).unwrap_or(0))
    }

    pub fn peek_is_kind(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|tok| tok.token.is_kind(kind))
    }

    pub fn next_if(&mut self, kind: TokenKind) -> Option<&Token<'a>> {
        if self.peek_is_kind(kind) {
            self.next()
        } else {
            None
        }
    }
}

impl TokenKind {
    // Token is a whole number type or start of one
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            TokenKind::BinaryNumber
                | TokenKind::OctalNumber
                | TokenKind::DecimalNumber
                | TokenKind::HexNumber
                | TokenKind::Minus
        )
    }

    pub fn is_literal(&self) -> bool {
        self.is_number() || *self == TokenKind::Char
    }

    // Token can appear in an immediates idxition
    pub fn is_immediate(&self) -> bool {
        self.is_literal() || *self == TokenKind::Ident
    }
}
