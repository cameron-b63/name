use crate::{
    instruction::{information::InstructionInformation, instruction_set::INSTRUCTION_TABLE},
    parse::token::{Token, TokenKind},
    structs::{ParseRegisterError, Register},
};

//pub enum Ast {
//    Symbol(String),
//    Label(String),
//
//    // literals
//    String(String),
//    Number(u32),
//    Char(char),
//
//    // constructs
//    Instruction(String, Vec<Ast>),
//    Register(Register),
//    Directive(String, Vec<Ast>),
//    BaseAddress(u32, Box<Ast>),
//
//    Root(Vec<Ast>),
//}

pub enum ParseError {
    UnexpectedToken,
    InvalidRegister(ParseRegisterError),
    InvalidInstruction(String),
    UnexpectedEof,
    InvalidNumber,
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

    pub fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn try_next(&mut self) -> ParseResult<&Token> {
        self.next().ok_or(ParseError::UnexpectedEof)
    }

    pub fn next_if(&mut self, kind: TokenKind) -> Option<&Token> {
        self.tokens
            .get(self.pos)
            .filter(|tok| tok.is_kind(kind))
            .inspect(|_| {
                self.pos += 1;
            })
    }

    pub fn try_next_if(&mut self, kind: TokenKind) -> ParseResult<&Token> {
        self.next()
            .filter(|tok| tok.is_kind(kind))
            .ok_or(ParseError::UnexpectedToken)
    }

    pub fn try_advance_if(&mut self, kind: TokenKind) -> ParseResult<()> {
        self.try_next_if(kind).map(|_| ())
    }

    pub fn parse_symbol(&mut self) -> ParseResult<String> {
        self.try_next_if(TokenKind::Symbol)
            .map(|tok| tok.src_string())
    }

    pub fn parse_string(&mut self) -> ParseResult<String> {
        self.try_next_if(TokenKind::String).map(|tok| {
            let src = tok.src_span.src;
            src[1..src.len() - 2].to_string()
        })
    }

    pub fn parse_char(&mut self) -> ParseResult<char> {
        self.try_next_if(TokenKind::Char).and_then(|tok| {
            let src = tok.src_span.src;

            let char = &src[1..(src.len() - 2)];

            char.parse::<char>()
                .map_err(|_| ParseError::UnexpectedToken)
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

    pub fn parse_label(&mut self) -> ParseResult<String> {
        let label = self.try_next_if(TokenKind::Symbol)?.src_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(label)
    }

    pub fn parse_op(&mut self) -> ParseResult<&'static InstructionInformation> {
        let instr = self.try_next_if(TokenKind::Symbol)?.src_span.src;

        INSTRUCTION_TABLE
            .get(instr)
            .ok_or(ParseError::InvalidInstruction(instr.to_string()))
            .copied()
    }

    pub fn parse_number(&mut self) -> ParseResult<u32> {
        let is_minus = self.next_if(TokenKind::Minus).is_some();
        self.try_next().and_then(|tok| {
            match tok.kind {
                TokenKind::HexNumber => {
                    u32::from_str_radix(tok.src_span.src, 16).map_err(|_| ParseError::InvalidNumber)
                }
                TokenKind::DecimalNumber => {
                    u32::from_str_radix(tok.src_span.src, 10).map_err(|_| ParseError::InvalidNumber)
                }
                TokenKind::OctalNumber => {
                    u32::from_str_radix(tok.src_span.src, 8).map_err(|_| ParseError::InvalidNumber)
                }
                TokenKind::Fractional => tok
                    .src_span
                    .src
                    .parse::<f32>()
                    .map(|num| num as u32)
                    .map_err(|_| ParseError::InvalidNumber),
                _ => Err(ParseError::UnexpectedToken),
            }
            .map(|num| {
                if is_minus {
                    if tok.is_kind(TokenKind::Fractional) {
                        // set the first bit to a one
                        num | 0x8000
                    } else {
                        // will always twos complement
                        (num as i32 * -1i32) as u32
                    }
                } else {
                    num
                }
            })
        })
    }

    pub fn parse_directive(&mut self) -> ParseResult<String> {
        self.try_next_if(TokenKind::Directive)
            .map(|tok| tok.src_string())
    }

    pub fn parse_base_address() {
        todo!()
    }

    // pub fn parse_instruction() {}
}
