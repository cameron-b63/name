use crate::parse::token::{Token, TokenKind};
use std::{
    iter::{Map, Peekable},
    ops::{Bound, RangeBounds, RangeInclusive},
    slice::SliceIndex,
    str::CharIndices,
};

#[derive(Debug, PartialEq)]
pub struct Span<T> {
    pub range: (Bound<usize>, Bound<usize>),
    pub data: T,
}

impl<T> Span<T> {
    pub fn new<R: RangeBounds<usize>>(range: R, data: T) -> Self {
        Self {
            range: (range.start_bound().cloned(), range.end_bound().cloned()),
            data,
        }
    }
}

type Spanned<'a> = Span<Token<'a>>;

type LexerError = Span<LexerErrorType>;

#[derive(Debug, PartialEq)]
pub enum LexerErrorType {
    UnexpectedEof,
    InvalidChar(char),
    UnterminatedString,
    WrongRadix(char, u32),
}

type LexerResult<'a> = Result<Spanned<'a>, LexerError>;

type CharScanner<'a> = Peekable<CharIndices<'a>>;

pub struct Lexer<'a> {
    chars: CharScanner<'a>,
    src: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.char_indices().peekable(),
            src,
        }
    }

    fn spanned<R: RangeBounds<usize> + Clone + SliceIndex<str, Output = str>>(
        &self,
        range: R,
        kind: TokenKind,
    ) -> Spanned<'a> {
        Span::new(
            range.clone(),
            Token::new(kind, self.src.get(range).unwrap_or("")),
        )
    }

    fn char_tok(&mut self, pos: usize, c: char) -> LexerResult<'a> {
        let tok_kind = match c {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            _ => Err(Span::new(pos..=pos, LexerErrorType::InvalidChar(c)))?,
        };

        Ok(self.spanned(pos..=pos, tok_kind))
    }

    fn lex_string_cont(&mut self, start: usize) -> LexerResult<'a> {
        match self.chars.find(|(_, c)| c == &'"') {
            Some((end, _)) => Ok(self.spanned(start..=end, TokenKind::String)),
            None => Err(Span::new(start.., LexerErrorType::UnterminatedString)),
        }
    }

    fn lex_name_cont(&mut self, start: usize) -> Spanned<'a> {
        let count = self
            .chars
            .by_ref()
            .take_while(|(_, c)| matches!(c, 'a'..'z' | 'A'..'Z' | '0'..'9' | '_' | '-'))
            .count();

        self.spanned(start..=(start + count), TokenKind::Name)
    }

    fn lex_radix_cont(&mut self, start: usize, radix: u32) -> LexerResult<'a> {
        let count = self
            .chars
            .by_ref()
            .take_while(|(_, c)| c.is_digit(radix))
            .count();

        if let Some((end, c)) = self.chars.next_if(|(_, c)| c.is_digit(16)) {
            Err(Span::new(
                start..=*end,
                LexerErrorType::WrongRadix(*c, radix),
            ))
        } else {
            let tok_kind = match radix {
                2 => TokenKind::BinaryNumber,
                8 => TokenKind::OctalNumber,
                16 => TokenKind::HexNumber,
                _ => panic!("lex_radix_cont {radix} is not supported"),
            };
            Ok(self.spanned(start..(start + count), tok_kind))
        }
    }

    fn lex_number_cont(&mut self, start: usize) -> LexerResult<'a> {
        if let Some((_, r)) = self.chars.next_if(|(_, r)| matches!(r, 'x' | 'o' | 'b')) {
            match r {
                'x' => self.lex_radix_cont(start, 16),
                'o' => self.lex_radix_cont(start, 8),
                'b' => self.lex_radix_cont(start, 2),
                _ => panic!("lex_number radix fallthrough"),
            }
        } else {
            self.lex_decimal_cont(start)
        }
    }
    fn lex_decimal_cont(&mut self, start: usize) -> LexerResult<'a> {
        let  = self.chars.by_ref().take_while(|c| c.is_digit(10)).count();

    }

    fn lex_register_cont(&mut self, start: usize) -> LexerResult<'a> {
        todo!()
    }

    fn lex_directive_cont(&mut self, start: usize) -> Spanned<'a> {
        let count = self
            .chars
            .by_ref()
            .take_while(|(_, c)| matches!(c, 'a'..'z'))
            .count();

        self.spanned(start..=(start + count), TokenKind::Directive)
    }

    fn lex(&mut self) -> LexerResult<'a> {
        if let Some((i, c)) = self.chars.next() {
            let tok = match c {
                'a'..'z' | 'A'..'Z' => self.lex_name_cont(i),
                '$' => self.lex_register_cont(i)?,
                '0'..'9' => self.lex_number_cont(i)?,
                '"' => self.lex_string_cont(i)?,
                '.' => self.lex_directive_cont(i),
                _ => self.char_tok(i, c)?,
            };
            Ok(tok)
        } else {
            let len = self.src.len();
            Ok(self.spanned(len..len, TokenKind::EndOfFile))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lex_string() {
        let mut lex = Lexer::new("\"hello\" there");
        assert_eq!(
            Ok(Span::new(0..=6, Token::new(TokenKind::String, "\"hello\""))),
            lex.lex()
        );
    }
    #[test]
    fn test_lex_name() {
        let mut lex = Lexer::new("my-func_tion12 there");
        assert_eq!(
            Ok(Span::new(
                0..=13,
                Token::new(TokenKind::Name, "my-func_tion12")
            )),
            lex.lex()
        );
    }
}
