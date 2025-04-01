use crate::{
    parse::{
        span::{Span, SrcPos, SrcSpan},
        token::{Token, TokenCursor, TokenKind},
    },
    structs::{ParseRegisterError, Register, Section},
};

use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub enum AstKind {
    // a branch label
    Label(String),

    // Immediates
    Symbol(String),
    Immediate(u32),

    // Directives
    Asciiz(String),
    Section(Section),

    // constructs
    Instruction(String, Vec<Ast>),
    Register(Register),
}

impl AstKind {
    pub fn get_register(self) -> Option<Register> {
        if let AstKind::Register(reg) = self {
            Some(reg)
        } else {
            None
        }
    }

    pub fn get_immediate(self) -> Option<u32> {
        if let AstKind::Immediate(imm) = self {
            Some(imm)
        } else {
            None
        }
    }
}

impl fmt::Display for AstKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO: write the actual display information
        write!(f, "{:?}", self)
    }
}

pub type Ast = Span<AstKind>;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnexpectedToken(TokenKind),
    InvalidRegister(ParseRegisterError),
    UnexpectedEof,
    InvalidNumber(ParseIntError),
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
            Self::UnexpectedToken(tok) => write!(f, "unexpected token {:?}", tok),
            Self::InvalidRegister(reg_err) => {
                write!(f, "invalid register {:#?}", reg_err)
            }
            Self::UnexpectedEof => write!(f, "unexpected eof"),
            Self::InvalidNumber(int_err) => write!(f, "invalid number {:#?}", int_err),
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

pub type ParseError = Span<ErrorKind>;
type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    cursor: TokenCursor<'a>,
    src: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(toks: Vec<Token<'a>>, src: &'a str) -> Self {
        Parser {
            cursor: TokenCursor::new(toks),
            src,
        }
    }

    pub fn ast<F: Fn(&mut Self) -> ParseResult<AstKind>>(&mut self, f: F) -> ParseResult<Ast> {
        let start = self.try_peek()?.token.start();
        let kind = f(self)?;
        let end = self
            .cursor
            .prev()
            .map(|tok| tok.token.end())
            .unwrap_or_default();

        // make the src span
        let src_span = SrcSpan { start, end };

        // return the ast
        Ok(Span { src_span, kind })
    }

    pub fn unexpected_eof(&self) -> Span<ErrorKind> {
        let prev = self.cursor.prev();
        let pos = prev
            .map(|tok| tok.token.src_span.end.clone())
            .unwrap_or_default();
        Span {
            kind: ErrorKind::UnexpectedEof,
            src_span: SrcSpan {
                start: pos.clone(),
                end: pos,
            },
        }
    }

    pub fn try_peek(&self) -> ParseResult<&Token<'a>> {
        self.cursor.peek().ok_or(self.unexpected_eof())
    }

    pub fn try_next(&mut self) -> ParseResult<&Token<'a>> {
        if self.cursor.peek().is_some() {
            Ok(self.cursor.next().unwrap())
        } else {
            Err(self.unexpected_eof())
        }
    }

    pub fn try_next_if(&mut self, kind: TokenKind) -> ParseResult<&Token<'a>> {
        let tok = self.try_next()?;
        if tok.token.is_kind(kind) {
            Ok(tok)
        } else {
            Err(tok.token.clone().map(|k| ErrorKind::UnexpectedToken(k)))
        }
    }

    pub fn try_advance_if(&mut self, kind: TokenKind) -> ParseResult<()> {
        let _ = self.try_next_if(kind)?;
        Ok(())
    }

    pub fn parse_ident(&mut self) -> ParseResult<String> {
        let tok = self.try_next_if(TokenKind::Ident)?;
        Ok(tok.src.to_string())
    }

    pub fn parse_string(&mut self) -> ParseResult<String> {
        let src = self.try_next_if(TokenKind::String)?.src;
        Ok(src[1..src.len() - 1].to_string())
    }

    pub fn parse_register(&mut self) -> ParseResult<Register> {
        let tok = self.try_next_if(TokenKind::Register)?;

        tok.src.parse::<Register>().map_err(|e| Span {
            kind: ErrorKind::InvalidRegister(e),
            src_span: tok.token.src_span.clone(),
        })
    }

    pub fn parse_label(&mut self) -> ParseResult<AstKind> {
        let label = self.try_next_if(TokenKind::Ident)?.src.to_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(AstKind::Label(label))
    }

    pub fn parse_char(&mut self) -> ParseResult<u32> {
        let tok = self.try_next_if(TokenKind::Char)?;
        // skip the first quote
        let mut chars = tok.src.chars().skip(1);
        let char = match (chars.next(), chars.next()) {
            (Some(c), _) => c,
            (Some('\\'), Some(c)) => match c {
                't' => '\t',
                'r' => '\r',
                'n' => '\n',
                '\'' => '\'',
                '"' => '"',
                '\\' => '\\',
                _ => return Err(tok.token.clone().map(|_| ErrorKind::InvalidEscape)),
            },
            _ => return Err(tok.token.clone().map(|_| ErrorKind::InvalidChar)),
        };
        Ok(char as u32)
    }

    pub fn parse_number(&mut self) -> ParseResult<u32> {
        let is_minus = self.cursor.next_if(TokenKind::Minus).is_some();
        let tok = self.try_next()?;

        let mut num = match tok.token.kind {
            TokenKind::HexNumber => u32::from_str_radix(&tok.src[2..], 16),
            TokenKind::DecimalNumber => u32::from_str_radix(tok.src, 10),
            TokenKind::OctalNumber => u32::from_str_radix(&tok.src[2..], 8),
            TokenKind::BinaryNumber => u32::from_str_radix(&tok.src[2..], 2),
            _ => return Err(tok.token.clone().map(|k| ErrorKind::UnexpectedToken(k))),
        }
        .map_err(|e| Span {
            kind: ErrorKind::InvalidNumber(e),
            src_span: tok.token.src_span.clone(),
        })?;

        if is_minus {
            num = (num as i32 * -1i32) as u32;
        }

        Ok(num)
    }

    pub fn parse_literal(&mut self) -> ParseResult<u32> {
        let literal = &self.try_peek()?.token;

        match literal.kind {
            tok if tok.is_number() => self.parse_number(),
            TokenKind::Char => self.parse_char(),
            _ => return Err(literal.clone().map(|k| ErrorKind::UnexpectedToken(k))),
        }
    }

    pub fn parse_immediate(&mut self) -> ParseResult<AstKind> {
        let immediate = &self.try_peek()?.token;
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
        Ok(ast)
    }

    pub fn parse_directive(&mut self) -> ParseResult<AstKind> {
        let tok = self.try_next_if(TokenKind::Directive)?;

        let ast = match tok.src {
            // ".eqv" => AstKind::Eqv(self.ast(self.parse_ident())?, self.parse_literal()?),
            // ".include" => AstKind::Include(self.parse_string()?),
            ".text" => AstKind::Section(Section::Text),
            ".data" => AstKind::Section(Section::Data),
            ".asciiz" => AstKind::Asciiz(self.parse_string()?),
            ".align" => todo!(),
            ".macro" => todo!(),
            ".word" => todo!(),
            _ => {
                return Err(Span {
                    kind: ErrorKind::InvalidDirective,
                    src_span: tok.token.src_span.clone(),
                })
            }
        };
        Ok(ast)
    }

    pub fn parse_arg(&mut self) -> ParseResult<AstKind> {
        let tok = &self.try_peek()?.token;
        let ast = match tok.kind {
            TokenKind::Register => {
                let reg = self.parse_register()?;
                AstKind::Register(reg)
            }
            tok if tok.is_immediate() => self.parse_immediate()?,
            TokenKind::LParen => {
                self.try_advance_if(TokenKind::LParen)?;
                let reg = self.parse_register()?;
                self.try_advance_if(TokenKind::RParen)?;
                AstKind::Register(reg)
            }
            _ => {
                return Err(Span {
                    kind: ErrorKind::UnexpectedToken(tok.kind),
                    src_span: tok.src_span.clone(),
                })
            }
        };
        Ok(ast)
    }

    pub fn parse_args(&mut self) -> ParseResult<Vec<Ast>> {
        let mut args = Vec::new();

        // check if there are no args
        if self.cursor.peek_is_kind(TokenKind::Newline) {
            return Ok(args);
        }

        // parse an arg, stop parsing when there is not comma after arg
        loop {
            args.push(self.ast(Self::parse_arg)?);

            // if there is not a comma delimiter or the next token does not indicate a base adress
            if self.cursor.next_if(TokenKind::Comma).is_none()
                && !self.cursor.peek_is_kind(TokenKind::LParen)
            {
                break;
            }
        }

        Ok(args)
    }

    pub fn parse_root(&mut self) -> ParseResult<AstKind> {
        let tok = self.try_peek()?;
        let ast: AstKind = match tok.token.kind {
            TokenKind::Directive => self.parse_directive()?,
            TokenKind::Ident => {
                let sym = self.parse_ident()?;

                // if it's a label declaration
                if self.cursor.next_if(TokenKind::Colon).is_some() {
                    AstKind::Label(sym)

                // if it's an instruction
                } else {
                    let args = self.parse_args()?;
                    let instr = AstKind::Instruction(sym, args);
                    instr
                }
            }
            _ => {
                // TODO add more info to error
                let err = tok.token.clone().map(|k| ErrorKind::UnexpectedToken(k));
                self.cursor.advance();
                return Err(err);
            }
        };
        Ok(ast)
    }

    pub fn parse(&mut self) -> (Vec<ParseError>, Vec<Ast>) {
        // root units of our ast, directives, instructions and labels
        let mut entries = Vec::new();
        let mut errs = Vec::new();

        while let Some(tok) = self.cursor.peek() {
            if tok.token.is_kind(TokenKind::Newline) {
                self.cursor.advance();
                continue;
            } else {
                match self.ast(Self::parse_root) {
                    Ok(ast) => entries.push(ast),
                    Err(err) => errs.push(err),
                }
            }
        }

        (errs, entries)
    }
}
