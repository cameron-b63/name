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
    // refrence to a branch label
    LabelRef(String),

    // Immediates
    Symbol(String),
    Immediate(u32),

    // Directives
    Include(String),
    Asciiz(String),
    Section(Section),
    Eqv(String, Box<Ast>),

    // constructs
    Instruction(&'static InstructionInformation, Vec<Ast>),
    Register(Register),
    BaseAddress(u32, Box<Ast>),
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

    pub fn parse_register(&mut self) -> ParseResult<Register> {
        self.try_next_if(TokenKind::Register).and_then(|tok| {
            tok.src_span
                .src
                .parse::<Register>()
                .map_err(|e| ParseError::InvalidRegister(e))
        })
    }

    pub fn parse_label(&mut self) -> ParseResult<Ast> {
        let label = self.try_next_if(TokenKind::Symbol)?.src_string();
        self.try_advance_if(TokenKind::Colon)?;
        Ok(Ast::Label(label))
    }

    pub fn parse_op(&mut self) -> ParseResult<&'static InstructionInformation> {
        let instr = self.try_next_if(TokenKind::Symbol)?.src_span.src;

        INSTRUCTION_TABLE
            .get(instr)
            .ok_or(ParseError::InvalidInstruction(instr.to_string()))
            .copied()
    }

    pub fn parse_char(&mut self) -> ParseResult<u32> {
        let src = self.try_next_if(TokenKind::Char)?.src_span.src;
        let mut chars = src[1..(src.len() - 1)].chars();
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
            TokenKind::Symbol => Ast::Symbol(self.parse_symbol()?),
            _ => return Err(ParseError::InvalidImmediate),
        };
        Ok(ast)
    }

    pub fn parse_argument_type(&mut self, t: ArgumentType) -> ParseResult<Ast> {
        use ArgumentType::*;

        let ast = match t {
            Rd | Rs | Rt => Ast::Register(self.parse_register()?),
            Immediate | Identifier => self.parse_immediate()?,
            BranchLabel => Ast::LabelRef(self.parse_symbol()?),
        };

        Ok(ast)
    }

    // pub fn parse_data_section(&mut self) -> ParseResult<Vec<Ast>> {
    //     let mut entries = Vec::new();
    //
    //     while let Some(tok) = self.peek() {
    //         let ast = match tok.kind {
    //             TokenKind::Symbol => self.parse_label(),
    //             TokenKind::Directive => {
    //                 if tok.is_section_directive() {
    //                     break;
    //                 } else {
    //                     self.parse_directive()?
    //                 }
    //             }
    //             _ => return Err(ParseError::InvalidData),
    //         };
    //         entries.push(ast);
    //     }
    //     Ok(entries)
    // }

    pub fn parse_instruction(&mut self) -> ParseResult<Ast> {
        let info = self.parse_op()?;

        todo!("parse an instruction")
    }

    // pub fn parse_text_section(&mut self) -> ParseResult<Vec<Ast>> {
    //     let mut entries = Vec::new();
    //     while let Some(tok) = self.peek() {
    //         let ast = match tok.kind {
    //             TokenKind::Symbol => {
    //                 if let Some(TokenKind::Colon) = self.peek2().map(|t| t.kind) {
    //                     Ast::Label(self.parse_label()?)
    //                 } else {
    //                     self.parse_instruction()?
    //                 }
    //             }
    //             TokenKind::Directive => {
    //                 if tok.is_section_directive() {
    //                     break;
    //                 } else if tok.is_data_directive() {
    //                     return Err(ParseError::WrongSection);
    //                 } else {
    //                     self.parse_directive()?
    //                 }
    //             }
    //             _ => return Err(ParseError::InvalidText),
    //         };
    //         entries.push(ast);
    //     }
    //     Ok(entries)
    // }

    pub fn parse_directive(&mut self) -> ParseResult<Ast> {
        let directive = self.try_next_if(TokenKind::Directive)?.src_span.src;

        let ast = match directive {
            ".eqv" => Ast::Eqv(self.parse_symbol()?, Box::new(self.parse_immediate()?)),
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

    pub fn parse(&mut self) -> (Vec<ParseError>, Ast) {
        // root units of our ast, directives, instructions and labels
        let mut entries = Vec::new();
        let mut errs = Vec::new();

        while let Some(tok) = self.peek() {
            let res = match tok.kind {
                TokenKind::Directive => self.parse_directive(),
                TokenKind::Symbol => {
                    if let Some(tok) = self.peek2().filter(|tok| tok.is_kind(TokenKind::Colon)) {
                        self.parse_label()
                    } else {
                        self.parse_instruction()
                    }
                }
                _ => Err(ParseError::UnexpectedToken),
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
