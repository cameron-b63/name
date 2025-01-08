use crate::{
    parse::{
        span::{Span, SrcSpan},
        token::{Token, TokenKind},
    },
    structs::{ParseRegisterError, Register, Section},
};

use std::fmt;

#[derive(Debug, Clone)]
pub enum AstKind<'a> {
    // a branch label
    Label(String),

    // Immediates
    Symbol(String),
    Immediate(u32),

    // Directives
    Include(String),
    Asciiz(String),
    Section(Section),
    Eqv(String, u32),

    // constructs
    Instruction(String, Vec<Ast<'a>>),
    Register(Register),
    BaseAddress(Option<Box<Ast<'a>>>, Register),
}

pub type Ast<'a> = Span<'a, AstKind<'a>>;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnexpectedToken,
    InvalidRegister(ParseRegisterError),
    UnexpectedEof,
    InvalidNumber,
    InvalidChar,
    InvalidEscape,
    InvalidImmediate,
    InvalidDirective,
    WrongSection,
    InvalidData,
    InvalidText,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken => write!(f, "unexpected token"),
            Self::InvalidRegister(reg_err) => {
                write!(f, "invalid register {:#?}", reg_err)
            }
            Self::UnexpectedEof => write!(f, "unexpected eof"),
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidChar => write!(f, "invalid char"),
            Self::InvalidEscape => write!(f, "invalid escape"),
            Self::InvalidImmediate => write!(f, "invalid immediate"),
            Self::InvalidDirective => write!(f, "invalid directive"),
            Self::WrongSection => write!(f, "wrong section"),
            Self::InvalidData => write!(f, "invalid data"),
            Self::InvalidText => write!(f, "invalid text"),
        }
    }
}

pub type ParseError<'a> = Span<'a, ErrorKind>;
type ParseResult<'a, T> = Result<T, ParseError<'a>>;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    src: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, src: &'a str) -> Self {
        Parser {
            tokens,
            pos: 0,
            src,
        }
    }

    pub fn ast(&self, pos: usize, kind: AstKind<'a>) -> Ast<'a> {
        // get the start infromation for the ast
        let (start, line, line_pos) = self
            .tokens
            .get(pos)
            .map(|tok| {
                let span = &tok.src_span;
                (span.start, span.line, span.line_pos)
            })
            .unwrap_or((0, 0, 0));

        // get the end information for the ast
        let end = self.prev().map(|tok| tok.src_span.end).unwrap_or(0);

        // make the src span
        let src_span = SrcSpan {
            start,
            end,
            line,
            line_pos,
            src: self.src.get(start..end).unwrap_or(""),
        };

        // return the ast
        Span { src_span, kind }
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn is_eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    pub fn next(&mut self) -> Option<&Token<'a>> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    pub fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    pub fn prev(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos - 1)
    }

    pub fn unexpected_eof(&self) -> Span<'a, ErrorKind> {
        let prev = self.prev();
        let pos = prev.map(|tok| tok.src_span.end).unwrap_or(0);
        Span {
            kind: ErrorKind::UnexpectedEof,
            src_span: SrcSpan {
                start: pos,
                end: pos,
                src: "",
                line: prev.map(|tok| tok.src_span.line).unwrap_or(0),
                line_pos: prev
                    .map(|tok| tok.src_span.line_pos + (tok.src_span.end - tok.src_span.start))
                    .unwrap_or(0),
            },
        }
    }

    pub fn try_peek(&self) -> ParseResult<'a, &Token<'a>> {
        self.peek().ok_or(self.unexpected_eof())
    }

    pub fn try_next(&mut self) -> ParseResult<'a, &Token<'a>> {
        if let Some(tok) = self.tokens.get(self.pos) {
            self.pos += 1;
            Ok(tok)
        } else {
            Err(self.unexpected_eof())
        }
    }

    pub fn peek_is_kind(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|tok| tok.is_kind(kind))
    }

    pub fn next_if(&mut self, kind: TokenKind) -> Option<&Token<'a>> {
        if self.peek_is_kind(kind) {
            self.next()
        } else {
            None
        }
    }

    pub fn try_next_if(&mut self, kind: TokenKind) -> ParseResult<'a, &Token<'a>> {
        let tok = self.try_next()?;
        if tok.is_kind(kind) {
            Ok(tok)
        } else {
            Err(Span {
                kind: ErrorKind::UnexpectedToken,
                src_span: tok.src_span.clone(),
            })
        }
    }

    pub fn try_advance_if(&mut self, kind: TokenKind) -> ParseResult<'a, ()> {
        self.try_next_if(kind).map(|_| ())
    }

    pub fn parse_ident(&mut self) -> ParseResult<'a, String> {
        self.try_next_if(TokenKind::Ident)
            .map(move |tok| tok.src_string())
    }

    pub fn parse_string(&mut self) -> ParseResult<'a, String> {
        self.try_next_if(TokenKind::String).map(|tok| {
            let src = tok.src_span.src;
            src[1..src.len() - 1].to_string()
        })
    }

    pub fn parse_register(&mut self) -> ParseResult<'a, Register> {
        self.try_next_if(TokenKind::Register).and_then(|tok| {
            tok.src_span.src.parse::<Register>().map_err(|e| Span {
                kind: ErrorKind::InvalidRegister(e),
                src_span: tok.src_span.clone(),
            })
        })
    }

    pub fn parse_label(&mut self) -> ParseResult<'a, Ast<'a>> {
        let pos = self.pos;
        let tok = self.try_next_if(TokenKind::Ident)?;
        let label = tok.src_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(self.ast(pos, AstKind::Label(label)))
    }

    pub fn parse_char(&mut self) -> ParseResult<'a, u32> {
        let tok = self.try_next_if(TokenKind::Char)?;
        // skip the first quote
        let mut chars = tok.src_span.src.chars().skip(1);
        let char = match (chars.next(), chars.next()) {
            (Some(c), None) => c,
            (Some('\\'), Some(c)) => match c {
                't' => '\t',
                'r' => '\r',
                'n' => '\n',
                '\'' => '\'',
                '"' => '"',
                '\\' => '\\',
                _ => {
                    return Err(Span {
                        kind: ErrorKind::InvalidEscape,
                        src_span: tok.src_span.clone(),
                    })
                }
            },
            _ => {
                return Err(Span {
                    kind: ErrorKind::InvalidChar,
                    src_span: tok.src_span.clone(),
                })
            }
        };
        Ok(char as u32)
    }

    pub fn parse_number(&mut self) -> ParseResult<'a, u32> {
        let is_minus = self.next_if(TokenKind::Minus).is_some();
        let tok = self.try_next()?;

        let mut num = match tok.kind {
            TokenKind::HexNumber => u32::from_str_radix(tok.src_span.src, 16),
            TokenKind::DecimalNumber => u32::from_str_radix(tok.src_span.src, 10),
            TokenKind::OctalNumber => u32::from_str_radix(tok.src_span.src, 8),
            _ => {
                return Err(Span {
                    kind: ErrorKind::UnexpectedToken,
                    src_span: tok.src_span.clone(),
                })
            }
        }
        .map_err(|_| Span {
            kind: ErrorKind::InvalidNumber,
            src_span: tok.src_span.clone(),
        })?;

        if is_minus {
            num = (num as i32 * -1i32) as u32;
        }

        Ok(num)
    }

    pub fn parse_literal(&mut self) -> ParseResult<'a, u32> {
        let literal = self.try_peek()?;

        match literal.kind {
            tok if tok.is_number() => self.parse_number(),
            TokenKind::Char => self.parse_char(),
            _ => {
                return Err(Span {
                    src_span: literal.src_span.clone(),
                    kind: ErrorKind::UnexpectedToken,
                })
            }
        }
    }

    pub fn parse_immediate(&mut self) -> ParseResult<'a, Ast<'a>> {
        let pos = self.pos;
        let immediate = self.try_peek()?;
        let ast = match immediate.kind {
            tok if tok.is_literal() => AstKind::Immediate(self.parse_literal()?),
            // these will be resolved on ast passover
            TokenKind::Ident => AstKind::Symbol(self.parse_ident()?),
            _ => {
                return Err(Span {
                    kind: ErrorKind::InvalidImmediate,
                    src_span: immediate.src_span.clone(),
                })
            }
        };
        Ok(self.ast(pos, ast))
    }

    pub fn parse_directive(&mut self) -> ParseResult<'a, Ast<'a>> {
        let pos = self.pos;

        let directive = self.try_next_if(TokenKind::Directive)?;

        let ast = match directive.src_span.src {
            ".eqv" => AstKind::Eqv(self.parse_ident()?, self.parse_literal()?),
            ".include" => AstKind::Include(self.parse_string()?),
            ".text" => AstKind::Section(Section::Text),
            ".data" => AstKind::Section(Section::Data),
            ".asciiz" => AstKind::Asciiz(self.parse_string()?),
            ".align" => todo!(),
            ".macro" => todo!(),
            _ => {
                return Err(Span {
                    kind: ErrorKind::InvalidDirective,
                    src_span: directive.src_span.clone(),
                })
            }
        };

        Ok(self.ast(pos, ast))
    }

    pub fn parse_base(&mut self) -> ParseResult<'a, Register> {
        self.try_advance_if(TokenKind::LParen)?;
        let reg = self.parse_register()?;
        self.try_advance_if(TokenKind::RParen)?;
        Ok(reg)
    }

    pub fn parse_arg(&mut self) -> ParseResult<'a, Ast<'a>> {
        let pos = self.pos;
        let tok = self.try_peek()?;
        let ast = match tok.kind {
            TokenKind::Register => {
                let reg = self.parse_register()?;
                self.ast(pos, AstKind::Register(reg))
            }
            tok if tok.is_immediate() => {
                let immediate = self.parse_immediate()?;

                if self.peek_is_kind(TokenKind::LParen) {
                    let reg = self.parse_base()?;
                    let base_address = AstKind::BaseAddress(Some(Box::new(immediate)), reg);
                    self.ast(pos, base_address)
                } else {
                    immediate
                }
            }
            TokenKind::LParen => {
                let base = self.parse_base()?;
                let base_address = AstKind::BaseAddress(None, base);
                self.ast(pos, base_address)
            }
            _ => {
                return Err(Span {
                    kind: ErrorKind::UnexpectedToken,
                    src_span: tok.src_span.clone(),
                })
            }
        };
        Ok(ast)
    }

    pub fn parse_args(&mut self) -> ParseResult<'a, Vec<Ast<'a>>> {
        let mut args = Vec::new();

        // check if there are no args
        if self.peek_is_kind(TokenKind::Newline) {
            return Ok(args);
        }

        // parse an arg, stop parsing when there is not comma after arg
        loop {
            args.push(self.parse_arg()?);

            if self.next_if(TokenKind::Comma).is_none() {
                break;
            }
        }

        Ok(args)
    }

    pub fn parse(&mut self) -> (Vec<ParseError<'a>>, Vec<Ast<'a>>) {
        // root units of our ast, directives, instructions and labels
        let mut entries = Vec::new();
        let mut errs = Vec::new();

        while let Some(tok) = self.peek() {
            let pos = self.pos;
            let res = match tok.kind {
                TokenKind::Directive => self.parse_directive(),
                TokenKind::Ident => self.parse_ident().and_then(|sym| {
                    // if it's a label declaration
                    if self.next_if(TokenKind::Colon).is_some() {
                        let label = AstKind::Label(sym);
                        Ok(self.ast(pos, label))

                    // if it's an instruction
                    } else {
                        let args = self.parse_args()?;
                        let instr = AstKind::Instruction(sym, args);
                        Ok(self.ast(pos, instr))
                    }
                }),
                TokenKind::Newline => {
                    self.advance();
                    continue;
                }
                _ => {
                    // TODO add more info to error
                    let src_span = tok.src_span.clone();
                    self.advance();
                    Err(Span {
                        kind: ErrorKind::UnexpectedToken,
                        src_span,
                    })
                }
            };

            // add the result to our vectors
            match res {
                Ok(ast) => entries.push(ast),
                Err(err) => errs.push(err),
            }
        }

        (errs, entries)
    }
}
