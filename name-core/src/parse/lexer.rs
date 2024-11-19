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

    fn single_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            src_span: self.src_span(self.get_pos()),
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
        self.chars.next().map(|c| {
            self.inc_pos();
            self.inc_line_pos();

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
            .map_err(|e| self.error(pos, ErrorKind::ExpectedChar('"')))?;
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
        let pos = self.pos;

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

    fn lex(&mut self) -> LexerResult<'a, TokenKind> {
        self.next_char()
            .map(|c| {
                let tok_kind = match c {
                    'a'..='z' | 'A'..='Z' => {
                        self.consume_name();
                        TokenKind::Name
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
                        self.consume_char_lit();
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
            .unwrap_or(Ok(TokenKind::EndOfFile))
    }

    pub fn next_tok(&mut self) -> LexerResult<Token<'a>> {
        // eat any whitce space that may prepend next token
        self.consume_while(char::is_whitespace);

        // check for comments and eat the rest of them it
        while self.next_char_if(|c| c == '#').is_some() {
            self.consume_until('\n')?;

            // consume any whitespace that may occur before next token
            self.consume_while(char::is_whitespace)
        }

        // pos of next_tok or eof
        let pos = self.get_pos() + 1;

        // get the tokenkind
        let tok = self.lex()?;

        // make a token
        Ok(self.token(pos, tok))
    }
}

macro_rules! test {
    ($mod:ident, $tokens:expr) => {};
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_token_sequence(s: &str, v: &[TokenKind]) {
        let mut lex = Lexer::new(s);

        for tk in v {
            assert_eq!(lex.next_tok().map(|t| t.kind), Ok(tk))
        }
    }

    token_group! {
        string,
        string: "\"hello world\""=&[TokenKind::String],
        empty_string: "\"\""=&[TokenKind::String],
    }

    token_group! {
        comments,
        end_comment: "\n#"=&[TokenKind::Newline],
        begin_comment: "#\n"=&[TokenKind::Newline],
        double_comment: "# # # #\n"=&[TokenKind::Newline],
        after_comment: "label: # hello\n"=&[TokenKind::Name, TokenKind::Colon, TokenKind::Newline],
    }

    #[test]
    fn whitespace_sep_next_tok() {
        let cases = vec![
            ("my-func_tion12", TokenKind::Name),
            ("\"hello\"", TokenKind::String),
            ("'c'", TokenKind::Char),
            (".macro", TokenKind::Directive),
            ("1234", TokenKind::DecimalNumber),
            ("1234.12345", TokenKind::Fractional),
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

        // dbg!(lex_str.chars().enumerate().collect::<Vec<(usize, char)>>());

        let mut lex = Lexer::new(&lex_str);

        let _ = cases.into_iter().fold(1, |pos, (s, tk)| {
            let end = pos + s.len() - 1;
            assert_eq!(Ok(tk), lex.next_tok().map(|t| t.kind));

            // increment passed the one inserted space and to the next char
            end + 2
        });
    }
}
