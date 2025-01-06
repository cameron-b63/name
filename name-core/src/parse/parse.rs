use crate::{
    instruction::{
        information::{ArgumentType, InstructionInformation},
        instruction_set::INSTRUCTION_TABLE,
    },
    parse::token::{SrcSpan, Token, TokenKind},
    structs::{ParseRegisterError, Register, Section},
};

use std::fmt;

#[derive(Debug, Clone)]
pub enum Ast {
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

    //Macros
    //Arguments of name and arguments
    MacroDefintion(String, Vec<Ast>, Vec<Ast>),
    // macro identifier and arguments to call macro with
    MacroCall(String, Vec<Ast>),
    MacroArg(String),

    // constructs
    Instruction(String, Vec<Ast>),
    Register(Register),
    BaseAddress(Box<Ast>, Register),
    Root(Vec<Ast>),
}

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
            Self::InvalidRegister(regErr) => {
                write!(f, "invalid register {:#?}", regErr)
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

#[derive(Debug, Clone)]
pub struct ParseError<'a> {
    pub src_span: SrcSpan<'a>,
    pub kind: ErrorKind,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.src_span)
    }
}

impl<'a> ParseError<'a> {
    fn unexpected_token(src_span: SrcSpan<'a>) -> Self {
        ParseError {
            src_span,
            kind: ErrorKind::UnexpectedToken,
        }
    }
}

type ParseResult<'a, T> = Result<T, ParseError<'a>>;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
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

    pub fn unexpected_eof(&self) -> ParseError<'a> {
        let prev = self.prev();
        let pos = prev.map(|tok| tok.src_span.end).unwrap_or(0);
        ParseError {
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
            Err(ParseError::unexpected_token(tok.src_span.clone()))
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
            tok.src_span
                .src
                .parse::<Register>()
                .map_err(|e| ParseError {
                    kind: ErrorKind::InvalidRegister(e),
                    src_span: tok.src_span.clone(),
                })
        })
    }

    pub fn parse_label(&mut self) -> ParseResult<'a, Ast> {
        let tok = self.try_next_if(TokenKind::Ident)?;
        let label = tok.src_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(Ast::Label(label))
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
                    return Err(ParseError {
                        kind: ErrorKind::InvalidEscape,
                        src_span: tok.src_span.clone(),
                    })
                }
            },
            _ => {
                return Err(ParseError {
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
            _ => return Err(ParseError::unexpected_token(tok.src_span.clone())),
        }
        .map_err(|_| ParseError {
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
            _ => return Err(ParseError::unexpected_token(literal.src_span.clone())),
        }
    }

    pub fn parse_immediate(&mut self) -> ParseResult<'a, Ast> {
        let immediate = self.try_peek()?;
        let ast = match immediate.kind {
            tok if tok.is_literal() => Ast::Immediate(self.parse_literal()?),
            // these will be resolved on ast passover
            TokenKind::Ident => Ast::Symbol(self.parse_ident()?),
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::InvalidImmediate,
                    src_span: immediate.src_span.clone(),
                })
            }
        };
        Ok(ast)
    }

    pub fn parse_directive(&mut self) -> ParseResult<'a, Ast> {
        let directive = self.try_next_if(TokenKind::Directive)?;

        let ast = match directive.src_span.src {
            ".eqv" => Ast::Eqv(self.parse_ident()?, self.parse_literal()?),
            ".include" => Ast::Include(self.parse_string()?),
            ".text" => Ast::Section(Section::Text),
            ".data" => Ast::Section(Section::Data),
            ".asciiz" => Ast::Asciiz(self.parse_string()?),
            ".align" => todo!(),
            ".macro" => self.parse_macro_defintion()?,
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::InvalidDirective,
                    src_span: directive.src_span.clone(),
                })
            }
        };

        Ok(ast)
    }

    pub fn parse_base(&mut self) -> ParseResult<'a, Register> {
        self.try_advance_if(TokenKind::LParen)?;
        let reg = self.parse_register()?;
        self.try_advance_if(TokenKind::RParen)?;
        Ok(reg)
    }

    pub fn parse_arg(&mut self) -> ParseResult<'a, Ast> {
        let tok = self.try_peek()?;
        let ast = match tok.kind {
            TokenKind::Register => Ast::Register(self.parse_register()?),
            tok if tok.is_immediate() => {
                let immediate = self.parse_immediate()?;

                if self.peek_is_kind(TokenKind::LParen) {
                    let reg = self.parse_base()?;
                    Ast::BaseAddress(Box::new(immediate), reg)
                } else {
                    immediate
                }
            }
            TokenKind::LParen => Ast::BaseAddress(Box::new(Ast::Immediate(0)), self.parse_base()?),
            _ => return Err(ParseError::unexpected_token(tok.src_span.clone())),
        };
        Ok(ast)
    }

    pub fn parse_args(&mut self) -> ParseResult<'a, Vec<Ast>> {
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

    pub fn parse_macro_arg(&mut self) -> ParseResult<'a, Ast> {
        self.try_advance_if(TokenKind::Percent)?;
        let ident = self.parse_ident()?;
        Ok(Ast::MacroArg(ident))
    }

    pub fn parse_macro_args(&mut self) -> ParseResult<'a, Vec<Ast>> {
        let mut args = Vec::new();

        if self.peek_is_kind(TokenKind::Newline) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_arg()?);
            if self.peek_is_kind(TokenKind::Newline) {
                break;
            }
        }
        Ok(args)
    }
    pub fn parse_macro_defintion(&mut self) -> ParseResult<'a, Ast> {
        let ident = self.parse_ident()?;
        let args = self.parse_macro_args()?;

        let mut body = Vec::new();

        while let Some(tok) = self.peek().filter(|x| x.src_span.src == ".end_macro") {
            body.push(self.parse_root_element()?);
        }

        Ok(Ast::MacroDefintion(ident, args, body))
    }

    pub fn parse_macro_call(&mut self) -> ParseResult<'a, Ast> {
        let ident = self.parse_ident()?;
        self.try_advance_if(TokenKind::LParen)?;
        let args = self.parse_macro_args()?;
        self.try_advance_if(TokenKind::RParen)?;
        Ok(Ast::MacroCall(ident, args))
    }

    pub fn parse_root_element(&mut self) -> ParseResult<'a, Ast> {
        let tok = self.try_peek()?;

        match tok.kind {
            TokenKind::Directive => self.parse_directive(),
            TokenKind::Ident => self.parse_ident().and_then(|sym| {
                // if it's a label declaration
                if self.next_if(TokenKind::Colon).is_some() {
                    Ok(Ast::Label(sym))

                // if it's an instruction
                } else {
                    let args = self.parse_args()?;
                    Ok(Ast::Instruction(sym, args))
                }
            }),
            _ => {
                // TODO add more info to error
                let src_span = tok.src_span.clone();
                self.advance();
                Err(ParseError::unexpected_token(src_span))
            }
        }
    }
    pub fn parse(&mut self) -> (Vec<ParseError<'a>>, Ast) {
        // root units of our ast, directives, instructions and labels
        let mut entries = Vec::new();
        let mut errs = Vec::new();

        while let Some(tok) = self.peek() {
            if tok.is_kind(TokenKind::Newline) {
                self.advance();
                continue;
            } else {
                let res = self.parse_root_element();
                // add the result to our vectors
                match res {
                    Ok(ast) => entries.push(ast),
                    Err(err) => errs.push(err),
                }
            }
        }

        (errs, Ast::Root(entries))
    }
}
