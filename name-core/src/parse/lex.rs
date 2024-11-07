use crate::parse::token::{Token, TokenKind};
use std::{
    iter::{Map, Peekable},
    ops::{Bound, RangeBounds, RangeInclusive},
    slice::SliceIndex,
    str::CharIndices,
};

#[derive(Debug, PartialEq)]
pub struct Span<T> {
    pub start: usize,
    pub data: T,
    pub end: usize,
}

impl<T> Span<T> {
    pub fn new(start: usize, data: T, end: usize) -> Self {
        Self { start, data, end }
    }
}

type Spanned<'a> = Span<Token<'a>>;

#[derive(Debug, PartialEq)]
pub enum LexerError {
    UnexpectedEof,
    UnexpectedChar(char),
    InvalidChar(char),
    ExpectedChar(char),
    InvalidEscape(char),
    WrongRadix(char, u32),
    Because(Box<LexerError>, Box<Span<LexerError>>),
    Context(&'static str, Box<LexerError>),
}

type LexerResult<T> = Result<T, Span<LexerError>>;

type CharScanner<'a> = Peekable<CharIndices<'a>>;

pub struct Lexer<'a> {
    chars: CharScanner<'a>,
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.char_indices().peekable(),
            src,
            pos: 0,
        }
    }

    fn span<T>(&self, start: usize, data: T) -> Span<T> {
        Span::new(start, data, self.pos)
    }

    fn emit(&self, start: usize, kind: TokenKind) -> Spanned<'a> {
        self.span(
            start,
            Token::new(kind, self.src.get(start..=self.pos).unwrap_or("")),
        )
    }

    fn single<T>(&self, data: T) -> Span<T> {
        self.span(self.pos, data)
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next().map(|(i, c)| {
            self.pos = i;
            c
        })
    }

    fn try_next_char(&mut self) -> LexerResult<char> {
        self.next_char()
            .ok_or(self.single(LexerError::UnexpectedEof))
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn next_char_if<P: Fn(char) -> bool>(&mut self, p: P) -> Option<char> {
        self.chars.next_if(|(_, c)| p(*c)).map(|(i, c)| {
            self.pos = i;
            c
        })
    }

    fn consume_while<P: Fn(char) -> bool>(&mut self, p: P) {
        while self.next_char_if(&p).is_some() {}
    }

    fn consume_char(&mut self, c: char) -> LexerResult<()> {
        let _ = self
            .next_char_if(|d| d == c)
            .ok_or(self.single(LexerError::ExpectedChar(c)))?;
        Ok(())
    }

    fn consume_until(&mut self, c: char) -> LexerResult<()> {
        self.consume_while(|d| d != c);
        let _ = self.try_next_char()?;
        Ok(())
    }

    fn consume_while_radix(&mut self, radix: u32) -> LexerResult<()> {
        self.consume_while(|c| c.is_digit(radix));

        if let Some(c) = self.next_char_if(|c| c.is_digit(16)) {
            Err(self.single(LexerError::WrongRadix(c, radix)))
        } else {
            Ok(())
        }
    }

    fn consume_string(&mut self) -> LexerResult<()> {
        let pos = self.pos;
        self.consume_until('"')
            .map_err(|e| self.span(pos, LexerError::ExpectedChar('"')))?;
        Ok(())
    }

    fn consume_name(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..'z' | 'A'..'Z' | '0'..'9' | '_' | '-'));
    }

    fn consume_directive(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..'z'));
    }

    fn consume_char_lit(&mut self) -> LexerResult<()> {
        let pos = self.pos;

        if '\\' == self.try_next_char()? {
            let c = self.try_next_char()?;
            if !matches!(c, 'n' | 't' | '\\' | 'r') {
                return Err(self.single(LexerError::InvalidEscape(c)));
            }
        }

        // make sure the next char is a '
        self.consume_char('\'')
    }

    fn lex(&mut self) -> LexerResult<TokenKind> {
        self.next_char()
            .map(|c| {
                let tok_kind = match c {
                    'a'..'z' | 'A'..'Z' => {
                        self.consume_name();
                        TokenKind::Name
                    }
                    // '$' => self.lex_register(),
                    '0'..'9' => match self.next_char_if(|r| matches!(r, 'x' | 'o' | 'b')) {
                        Some(r) if c == '0' => match r {
                            'x' => {
                                self.consume_while_radix(16)?;
                                TokenKind::HexNumber
                            }
                            'o' => {
                                self.consume_while_radix(8)?;
                                TokenKind::OctalNumber
                            }
                            'b' => {
                                self.consume_while_radix(2)?;
                                TokenKind::BinaryNumber
                            }
                            _ => unreachable!(),
                        },
                        _ => {
                            self.consume_while_radix(10)?;

                            if self.next_char_if(|c| c == '.').is_some() {
                                self.consume_while_radix(10)?;
                                TokenKind::Float
                            } else {
                                TokenKind::DecimalNumber
                            }
                        }
                    },
                    '"' => {
                        self.consume_string()?;
                        TokenKind::String
                    }
                    '.' => {
                        self.consume_directive();
                        TokenKind::Directive
                    }
                    '\'' => {
                        self.consume_char_lit();
                        TokenKind::Char
                    }
                    '+' => TokenKind::Plus,
                    '-' => TokenKind::Minus,
                    '(' => TokenKind::LParen,
                    ')' => TokenKind::RParen,
                    ':' => TokenKind::Colon,
                    ',' => TokenKind::Comma,
                    _ => return Err(self.single(LexerError::InvalidChar(c))),
                };
                Ok(tok_kind)
            })
            .unwrap_or(Ok(TokenKind::EndOfFile))
    }

    fn next_tok(&mut self) -> LexerResult<Spanned<'a>> {
        self.consume_while(char::is_whitespace);
        let pos = self.pos + 1;
        let tok = self.lex()?;
        Ok(self.emit(pos, tok))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_sep_next_tok() {
        let cases = vec![
            ("my-func_tion12", TokenKind::Name),
            ("\"hello\"", TokenKind::String),
            ("'c'", TokenKind::Char),
            (".macro", TokenKind::Directive),
            ("1234", TokenKind::DecimalNumber),
            ("1234.12345", TokenKind::Float),
            ("0xFFFFFFFA", TokenKind::HexNumber),
            ("0b1111111", TokenKind::BinaryNumber),
            ("0o12345", TokenKind::OctalNumber),
            ("", TokenKind::EndOfFile),
        ];

        let lex_str = cases.iter().fold(String::new(), |mut acc: String, (s, _)| {
            acc.push(' ');
            acc.push_str(s);
            acc
        });

        dbg!(lex_str.chars().enumerate().collect::<Vec<(usize, char)>>());

        let mut lex = Lexer::new(&lex_str);

        let _ = cases.into_iter().fold(1, |pos, (s, tk)| {
            let end = pos + s.len() - 1;
            assert_eq!(Ok(Span::new(pos, Token::new(tk, s), end)), lex.next_tok());

            // increment passed the one inserted space and to the next char
            end + 2
        });
    }
}
