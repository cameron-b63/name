use crate::parse::token::{SrcSpan, Token, TokenKind};
use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    UnexpectedEof,
    UnexpectedChar(char),
    InvalidChar(char),
    ExpectedChar(char),
    InvalidEscape(char),
    WrongRadix(char, u32),
}

#[derive(Debug, PartialEq)]
pub struct LexerError<'a> {
    src_span: SrcSpan<'a>,
    kind: ErrorKind,
}

type LexerResult<'a, T> = Result<T, LexerError<'a>>;

type CharScanner<'a> = Peekable<Chars<'a>>;

pub struct Lexer<'a> {
    chars: CharScanner<'a>,
    src: &'a str,
    pos: Option<usize>,
    line: usize,
    line_pos: Option<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().peekable(),
            src,
            pos: None,
            line: 0,
            line_pos: None,
        }
    }

    pub fn get_pos(&self) -> usize {
        self.pos.unwrap_or(0)
    }

    pub fn inc_pos(&mut self) {
        self.pos = Some(self.pos.map(|x| x + 1).unwrap_or(0));
    }

    pub fn get_line_pos(&self) -> usize {
        self.line_pos.unwrap_or(0)
    }

    pub fn inc_line_pos(&mut self) {
        self.line_pos = Some(self.line_pos.map(|x| x + 1).unwrap_or(0));
    }

    fn src_span(&self, start: usize) -> SrcSpan<'a> {
        let pos = self.get_pos();
        SrcSpan {
            start,
            end: pos,
            line: self.line,
            line_pos: self.get_line_pos(),
            src: self.src.get(start..=pos).unwrap_or(""),
        }
    }

    fn token(&self, start: usize, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            src_span: self.src_span(start),
        }
    }

    fn error(&self, start: usize, kind: ErrorKind) -> LexerError<'a> {
        LexerError {
            kind,
            src_span: self.src_span(start),
        }
    }

    fn single_error(&self, kind: ErrorKind) -> LexerError<'a> {
        LexerError {
            kind,
            src_span: self.src_span(self.get_pos()),
        }
    }

    /// get next char for lexer and advance the position
    fn next_char(&mut self) -> Option<char> {
        self.inc_pos();
        self.inc_line_pos();

        self.chars.next().map(|c| {
            if c == '\n' {
                self.line_pos = None;
                self.line += 1;
            }

            c
        })
    }

    /// fallibly  get the next char and return an UnexpectedEof if it's not there
    fn try_next_char(&mut self) -> LexerResult<'a, char> {
        self.next_char()
            .ok_or(self.single_error(ErrorKind::UnexpectedEof))
    }

    /// lookahead without consuming
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// get the next next_char conditionally with a predicate
    fn next_char_if<P: Fn(char) -> bool>(&mut self, p: P) -> Option<char> {
        self.peek_char()
            .filter(|c| p(*c))
            .and_then(|_| self.next_char())
    }

    /// consume chars while a predicate holds
    fn consume_while<P: Fn(char) -> bool>(&mut self, p: P) {
        while self.next_char_if(&p).is_some() {}
    }

    /// helper to fallibly consume an exact char
    fn consume_char(&mut self, c: char) -> LexerResult<'a, ()> {
        let _ = self
            .next_char_if(|d| d == c)
            .ok_or(self.single_error(ErrorKind::ExpectedChar(c)))?;
        Ok(())
    }

    /// helper to consume until char
    fn consume_until(&mut self, c: char) -> LexerResult<'a, ()> {
        self.consume_while(|d| d != c);
        let _ = self.try_next_char()?;
        Ok(())
    }

    /// consumes digits of the passed radix will fail with WrongRadix if a digit is valid
    /// hexidecminal but not the passed radix
    fn consume_while_radix(&mut self, radix: u32) -> LexerResult<'a, ()> {
        self.consume_while(|c| c.is_digit(radix));

        if let Some(c) = self.next_char_if(|c| c.is_digit(16)) {
            Err(self.single_error(ErrorKind::WrongRadix(c, radix)))
        } else {
            Ok(())
        }
    }

    // consumes a strings continuation fails if no terminating char
    fn consume_string(&mut self) -> LexerResult<'a, ()> {
        let pos = self.get_pos();
        self.consume_until('"')
            .map_err(|_e| self.error(pos, ErrorKind::ExpectedChar('"')))?;
        Ok(())
    }

    // consumes a names continuation infallibly
    fn consume_name(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.'));
    }

    // consumes a directives continuation infallibly
    fn consume_directive(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..='z' | '_'));
    }

    fn consume_char_lit(&mut self) -> LexerResult<'a, ()> {
        if '\\' == self.try_next_char()? {
            let c = self.try_next_char()?;
            if !matches!(c, 'n' | 't' | '\\' | 'r') {
                return Err(self.single_error(ErrorKind::InvalidEscape(c)));
            }
        }

        // make sure the next char is a '
        self.consume_char('\'')
    }

    fn consume_register(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..='z' | '0'..='9'))
    }

    fn tokenize(&mut self) -> Vec<LexerResult<'a, Token<'a>>> {
        let mut results = Vec::new();
        while let Some(res) = self.next_tok() {
            results.push(res);
        }
        results
    }

    fn lex(&mut self) -> LexerResult<'a, Token<'a>> {
        let c = self.next_char();
        let pos = self.get_pos();

        let tok_kind = c
            .map(|c| {
                let tok_kind = match c {
                    'a'..='z' | 'A'..='Z' => {
                        self.consume_name();
                        TokenKind::Symbol
                    }
                    '0'..='9' => match self.next_char_if(|r| matches!(r, 'x' | 'o' | 'b')) {
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
                                TokenKind::Fractional
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
                        self.consume_char_lit()?;
                        TokenKind::Char
                    }
                    '$' => {
                        self.consume_register();
                        TokenKind::Register
                    }
                    '+' => TokenKind::Plus,
                    '-' => TokenKind::Minus,
                    '(' => TokenKind::LParen,
                    ')' => TokenKind::RParen,
                    ':' => TokenKind::Colon,
                    ',' => TokenKind::Comma,
                    '\n' => TokenKind::Newline,
                    _ => return Err(self.single_error(ErrorKind::InvalidChar(c))),
                };
                Ok(tok_kind)
            })
            .unwrap_or(Ok(TokenKind::EndOfFile))?;

        Ok(self.token(pos, tok_kind))
    }

    pub fn done_lexing(&self) -> bool {
        self.pos == Some(self.src.len())
    }

    pub fn next_tok(&mut self) -> Option<LexerResult<'a, Token<'a>>> {
        if self.done_lexing() {
            None
        } else {
            // eat any whitce space that may prepend next token
            self.consume_while(char::is_whitespace);

            // check for comments and eat the rest of them it
            while self.next_char_if(|c| c == '#').is_some() {
                self.consume_while(|c| c != '\n');

                // consume any whitespace that may occur before next token
                self.consume_while(char::is_whitespace)
            }

            Some(self.lex())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_group {
        { $test:ident: $str:literal = $tok:expr } => {

            #[test]
            fn $test(){
                let mut lex = Lexer::new($str);

                for tk in $tok {
                    assert_eq!(lex.next_tok().unwrap().map(|t| t.kind), Ok(tk).cloned());
                }
            }

        };


        { $test:ident: $str:literal = $tok:expr, $($tests:ident : $strs:literal = $toks:expr),+ } => {
            test_group! { $test: $str = $tok }  // Expand the first test
            test_group! { $($tests : $strs = $toks),+ }  // Recursively expand the rest
        };

        { $name:ident, $($tests:ident : $strs:literal = $toks:expr),+ } => {

            mod $name {
                use super::*;
                test_group! { $($tests: $strs = $toks),+ }
            }

        };
    }

    test_group! {
        singles,
        parens: "()" = &[TokenKind::LParen, TokenKind::RParen],
        binops: "+-" = &[TokenKind::Plus, TokenKind::Minus],
        punctuation: ":," = &[TokenKind::Colon, TokenKind::Comma]
    }

    test_group! {
        numbers,
        decimal: " 42 034" = &[
            TokenKind::DecimalNumber,
            TokenKind::DecimalNumber,
        ],
        hexidecimal: " 0xDEADBEEEF" = &[TokenKind::HexNumber],
        octal: "0o42 " = &[TokenKind::OctalNumber],
        float: "1234.00001 " = &[TokenKind::Fractional]
    }

    test_group! {
        data,
        string: "hello_world: .asciiz \"hello word\"" = &[
            TokenKind::Symbol,
            TokenKind::Colon,
            TokenKind::Directive,
            TokenKind::String,
            TokenKind::EndOfFile,
        ],
        array: "my_array: .word 0 : 0xAA" = &[
            TokenKind::Symbol,
            TokenKind::Colon,
            TokenKind::Directive,
            TokenKind::DecimalNumber,
            TokenKind::Colon,
            TokenKind::HexNumber
        ]
    }

    test_group! {
        instruction,
        rtype: "add $a1, $a2, $a3 # this is a comment" = &[
            TokenKind::Symbol,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
            TokenKind::EndOfFile
        ],
        itype: "addi $a1, $a2, 0xDEADBEEF" = &[
            TokenKind::Symbol,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::HexNumber,
            TokenKind::EndOfFile
        ],
        jtype: "j my_label" = &[
            TokenKind::Symbol,
            TokenKind::Symbol,
        ]
    }
}
