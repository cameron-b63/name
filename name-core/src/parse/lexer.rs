use crate::parse::span::{Span, SrcPos, SrcSpan};
use crate::parse::token::{Token, TokenKind};
use std::{fmt, iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    UnexpectedEof,
    UnexpectedChar(char),
    InvalidChar(char),
    ExpectedChar(char),
    InvalidEscape(char),
    WrongRadix(char, u32),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedEof => write!(f, "unexpected eof"),
            ErrorKind::UnexpectedChar(c) => write!(f, "{} was not an expected char", c),
            ErrorKind::InvalidChar(c) => write!(f, "{} is not a valid char", c),
            ErrorKind::ExpectedChar(c) => write!(f, "expected char {}", c),
            ErrorKind::InvalidEscape(c) => write!(f, "{} is not a valid escape character", c),
            ErrorKind::WrongRadix(c, r) => write!(f, "{} is not of radix {}", c, r),
        }
    }
}

type LexError = Span<ErrorKind>;
type LexerResult<T> = Result<T, LexError>;

type CharScanner<'a> = Peekable<Chars<'a>>;

pub struct Lexer<'a> {
    chars: CharScanner<'a>,
    src: &'a str,
    pos: SrcPos,
    lexeme_start: Option<SrcPos>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().peekable(),
            src,
            pos: SrcPos {
                pos: 0,
                line_pos: 0,
                line: 0,
            },
            lexeme_start: None,
        }
    }

    fn src_span(&self) -> SrcSpan {
        let start = self.lexeme_start.as_ref().unwrap_or(&self.pos).clone();

        SrcSpan {
            start,
            end: self.pos.clone(),
        }
    }

    fn span<T>(&self, kind: T) -> Span<T> {
        Span {
            src_span: self.src_span(),
            kind,
        }
    }

    /// get next char for lexer and advance the position
    fn next_char(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.pos.pos += 1;
            self.pos.line_pos += 1;
            if c == '\n' {
                self.pos.line_pos = 0;
                self.pos.line += 1;
            }

            c
        })
    }

    /// fallibly  get the next char and return an UnexpectedEof if it's not there
    fn try_next_char(&mut self) -> LexerResult<char> {
        self.next_char().ok_or(self.span(ErrorKind::UnexpectedEof))
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
    fn consume_char(&mut self, c: char) -> LexerResult<()> {
        let _ = self
            .next_char_if(|d| d == c)
            .ok_or(self.span(ErrorKind::ExpectedChar(c)))?;
        Ok(())
    }

    /// helper to consume until char
    fn consume_until(&mut self, c: char) -> LexerResult<()> {
        self.consume_while(|d| d != c);
        let _ = self.try_next_char()?;
        Ok(())
    }

    /// consumes digits of the passed radix will fail with WrongRadix if a digit is valid
    /// hexidecminal but not the passed radix
    fn consume_while_radix(&mut self, radix: u32) -> LexerResult<()> {
        self.consume_while(|c| c.is_digit(radix));

        if let Some(c) = self.next_char_if(|c| c.is_digit(16)) {
            Err(self.span(ErrorKind::WrongRadix(c, radix)))
        } else {
            Ok(())
        }
    }

    // consumes a strings continuation fails if no terminating char
    fn consume_string(&mut self) -> LexerResult<()> {
        self.consume_until('"')
            .map_err(|_e| self.span(ErrorKind::ExpectedChar('"')))?;
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

    fn consume_char_lit(&mut self) -> LexerResult<()> {
        if '\\' == self.try_next_char()? {
            let c = self.try_next_char()?;
            if !matches!(c, 'n' | 't' | '\\' | 'r' | '\'' | '\"') {
                return Err(self.span(ErrorKind::InvalidEscape(c)));
            }
        }

        // make sure the next char is a '
        self.consume_char('\'')
    }

    fn consume_register(&mut self) {
        self.consume_while(|c| matches!(c, 'a'..='z' | '0'..='9'))
    }

    fn lex_token(&mut self) -> LexerResult<Option<Token>> {
        self.lexeme_start = Some(self.pos.clone());

        if let Some(c) = self.next_char() {
            let tok_kind = match c {
                'a'..='z' | 'A'..='Z' => {
                    self.consume_name();
                    TokenKind::Ident
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
                '%' => TokenKind::Percent,
                '\n' => TokenKind::Newline,
                _ => return Err(self.span(ErrorKind::InvalidChar(c))),
            };
            Ok(Some(self.span(tok_kind)))
        } else {
            Ok(None)
        }
    }

    pub fn next_tok(&mut self) -> LexerResult<Option<Token>> {
        // eat any whitce space that may prepend next token
        self.consume_while(|c| c.is_whitespace() && c != '\n');

        // check for comments and eat the rest of them it
        while self.next_char_if(|c| c == '#').is_some() {
            self.consume_while(|c| c != '\n');

            // consume any whitespace that may occur before next token
            self.consume_while(|c| c.is_whitespace() && c != '\n')
        }

        self.lex_token()
    }

    pub fn lex(&mut self) -> (Vec<LexError>, Vec<Token>) {
        let mut toks = Vec::new();
        let mut errs = Vec::new();

        loop {
            match self.next_tok() {
                Ok(Some(tok)) => toks.push(tok),
                Ok(None) => break,
                Err(err) => errs.push(err),
            }
        }

        (errs, toks)
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
                    assert_eq!(lex.next_tok().map(|t| t.unwrap().kind), Ok(tk).cloned());
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
            TokenKind::Ident,
            TokenKind::Colon,
            TokenKind::Directive,
            TokenKind::String,
        ],
        array: "my_array: .word 0 : 0xAA" = &[
            TokenKind::Ident,
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
            TokenKind::Ident,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
        ],
        itype: "addi $a1, $a2, 0xDEADBEEF" = &[
            TokenKind::Ident,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::Register,
            TokenKind::Comma,
            TokenKind::HexNumber,
        ],
        jtype: "j my_label" = &[
            TokenKind::Ident,
            TokenKind::Ident,
        ]
    }
}
