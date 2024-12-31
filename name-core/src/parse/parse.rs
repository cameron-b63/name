use crate::{
    instruction::{
        information::{ArgumentType, InstructionInformation},
        instruction_set::INSTRUCTION_TABLE,
    },
    parse::token::{Token, TokenKind},
    structs::{ParseRegisterError, Register, Section},
};

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
    Eqv(String, Box<Ast>),

    // constructs
    Instruction(String, Vec<Ast>),
    Register(Register),
    BaseAddress(Box<Ast>, Register),
    Root(Vec<Ast>),
}

pub enum ParseError {
    UnexpectedToken,
    InvalidRegister(ParseRegisterError),
    InvalidInstruction(String),
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

type ParseResult<T> = Result<T, ParseError>;

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

    pub fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn peek2(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    pub fn try_peek(&self) -> ParseResult<&Token> {
        self.peek().ok_or(ParseError::UnexpectedEof)
    }

    pub fn try_next(&mut self) -> ParseResult<&Token> {
        self.next().ok_or(ParseError::UnexpectedEof)
    }

    pub fn peek_is_kind(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|tok| tok.is_kind(kind))
    }

    pub fn next_if(&mut self, kind: TokenKind) -> Option<&Token> {
        if self.peek_is_kind(kind) {
            self.next()
        } else {
            None
        }
    }

    pub fn try_next_if(&mut self, kind: TokenKind) -> ParseResult<&Token> {
        self.next()
            .filter(|tok| tok.is_kind(kind))
            .ok_or(ParseError::UnexpectedToken)
    }

    pub fn try_advance_if(&mut self, kind: TokenKind) -> ParseResult<()> {
        self.try_next_if(kind).map(|_| ())
    }

    pub fn parse_ident(&mut self) -> ParseResult<String> {
        self.try_next_if(TokenKind::Ident)
            .map(|tok| tok.src_string())
    }

    pub fn parse_string(&mut self) -> ParseResult<String> {
        self.try_next_if(TokenKind::String).map(|tok| {
            let src = tok.src_span.src;
            src[1..src.len() - 2].to_string()
        })
    }

    pub fn parse_register(&mut self) -> ParseResult<Register> {
        self.try_next_if(TokenKind::Register).and_then(|tok| {
            tok.src_span
                .src
                .parse::<Register>()
                .map_err(|e| ParseError::InvalidRegister(e))
        })
    }

    pub fn parse_label(&mut self) -> ParseResult<Ast> {
        let label = self.try_next_if(TokenKind::Ident)?.src_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(Ast::Label(label))
    }

    pub fn parse_char(&mut self) -> ParseResult<u32> {
        let src = self.try_next_if(TokenKind::Char)?.src_span.src;
        // skip the first quote
        let mut chars = src.chars().skip(1);
        let char = match (chars.next(), chars.next()) {
            (Some(c), None) => c,
            (Some('\\'), Some(c)) => match c {
                't' => '\t',
                'r' => '\r',
                'n' => '\n',
                _ => return Err(ParseError::InvalidEscape),
            },
            _ => return Err(ParseError::InvalidChar),
        };
        Ok(char as u32)
    }

    pub fn parse_number(&mut self) -> ParseResult<u32> {
        let is_minus = self.next_if(TokenKind::Minus).is_some();
        let tok = self.try_next()?;
        let num =
            match tok.kind {
                TokenKind::HexNumber => u32::from_str_radix(tok.src_span.src, 16)
                    .map_err(|_| ParseError::InvalidNumber)?,
                TokenKind::DecimalNumber => u32::from_str_radix(tok.src_span.src, 10)
                    .map_err(|_| ParseError::InvalidNumber)?,
                TokenKind::OctalNumber => u32::from_str_radix(tok.src_span.src, 8)
                    .map_err(|_| ParseError::InvalidNumber)?,
                TokenKind::Fractional => tok
                    .src_span
                    .src
                    .parse::<f32>()
                    .map(|num| num as u32)
                    .map_err(|_| ParseError::InvalidNumber)?,
                _ => return Err(ParseError::UnexpectedToken),
            };
        let signed_num = if is_minus {
            if tok.is_kind(TokenKind::Fractional) {
                // set the first bit to a one
                num | 0x8000
            } else {
                // will always twos complement
                (num as i32 * -1i32) as u32
            }
        } else {
            num
        };
        Ok(signed_num)
    }

    pub fn parse_base_address(&mut self) -> ParseResult<()> {
        self.try_advance_if(TokenKind::LParen)?;
        let reg = self.parse_register()?;
        self.try_advance_if(TokenKind::RParen)?;
        Ok(())
    }

    pub fn parse_immediate(&mut self) -> ParseResult<Ast> {
        let ast = match self.try_peek()?.kind {
            tok if tok.is_number() => Ast::Immediate(self.parse_number()?),
            TokenKind::Char => Ast::Immediate(self.parse_char()?),
            // these will be resolved on ast passover
            TokenKind::Ident => Ast::Symbol(self.parse_ident()?),
            _ => return Err(ParseError::InvalidImmediate),
        };
        Ok(ast)
    }

    pub fn parse_directive(&mut self) -> ParseResult<Ast> {
        let directive = self.try_next_if(TokenKind::Directive)?.src_span.src;

        let ast = match directive {
            ".eqv" => Ast::Eqv(self.parse_ident()?, Box::new(self.parse_immediate()?)),
            ".include" => Ast::Include(self.parse_string()?),
            ".text" => Ast::Section(Section::Text),
            ".data" => Ast::Section(Section::Data),
            ".asciiz" => Ast::Asciiz(self.parse_string()?),
            ".align" => todo!(),
            ".macro" => todo!(),
            _ => return Err(ParseError::InvalidDirective),
        };

        Ok(ast)
    }

    pub fn parse_base(&mut self) -> ParseResult<Register> {
        self.try_advance_if(TokenKind::LParen)?;
        let reg = self.parse_register()?;
        self.try_advance_if(TokenKind::RParen)?;
        Ok(reg)
    }

    pub fn parse_arg(&mut self) -> ParseResult<Ast> {
        let ast = match self.try_peek()?.kind {
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
            _ => return Err(ParseError::UnexpectedToken),
        };
        Ok(ast)
    }

    pub fn parse_args(&mut self) -> ParseResult<Vec<Ast>> {
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

    pub fn parse(&mut self) -> (Vec<ParseError>, Ast) {
        // root units of our ast, directives, instructions and labels
        let mut entries = Vec::new();
        let mut errs = Vec::new();

        while let Some(tok) = self.peek() {
            let res = match tok.kind {
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
                TokenKind::Newline => continue,
                _ => {
                    // TODO add more info to error
                    self.advance();
                    Err(ParseError::UnexpectedToken)
                }
            };

            // add the result to our vectors
            match res {
                Ok(ast) => entries.push(ast),
                Err(err) => errs.push(err),
            }
        }

        (errs, Ast::Root(entries))
    }
}
