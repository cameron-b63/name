use crate::parse::span::Span;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TokenKind {
    Ident,
    Register,
    FpRegister,
    Directive,
    MacroArg,

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

pub type Token = Span<TokenKind>;

#[derive(Debug, Clone)]
pub struct TokenCursor {
    pub toks: VecDeque<Token>,
    pub pos: usize,
}

impl TokenCursor {
    pub fn new(toks: VecDeque<Token>) -> Self {
        let pos = toks.front().map(|tok| tok.src_span.pos).unwrap_or_default();

        TokenCursor { toks, pos }
    }

    pub fn next(&mut self) -> Option<Token> {
        if let Some(tok) = self.toks.pop_front() {
            self.pos += tok.src_span.length;
            Some(tok)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.toks.front()
    }

    pub fn peek_is_kind(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|tok| tok.is_kind(kind))
    }

    pub fn next_if(&mut self, kind: TokenKind) -> Option<Token> {
        if self.peek_is_kind(kind) {
            self.next()
        } else {
            None
        }
    }

    pub fn try_next_if(&mut self, kind: TokenKind) -> Result<Token, String> {
        if let Some(tok) = self.next() {
            if tok.is_kind(kind) {
                Ok(tok)
            } else {
                Err(format!("Expected {:?} found {:?}", kind, tok.kind))
            }
        } else {
            Err("Unexpected eof in token cursor".into())
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

    pub fn is_immediate(&self) -> bool {
        self.is_number() || *self == TokenKind::Char
    }
}
