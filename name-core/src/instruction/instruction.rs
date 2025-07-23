use std::{fmt, io, path::PathBuf};

use super::information::{FpInstructionInformation, InstructionInformation};
use crate::{
    instruction::information::FpFmt,
    parse::span::Span,
    structs::ProgramState,
};

/// Possible assemble error codes
#[derive(Debug)]
pub enum ErrorKind {
    DuplicateSymbol(String),
    FileNotFound(PathBuf),
    Io(io::Error),
    String(String),
    BadArguments,
    LabelOutsideOfSection,
    MissingAdditional,
    MissingFmt,
    MissingFunct,
    UnknownInstruction(String),
    UndefinedSymbol(String),
    InvalidShamt,
    InvalidArgument,
    ImmediateOverflow(u32),
    WrongInstructionType,
}

// ErrorKind enumeration
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::DuplicateSymbol(str) => write!(f, "duplicate symbol: {}", str),
            ErrorKind::FileNotFound(path) => write!(f, "File {:?} not found", path),
            ErrorKind::Io(err) => write!(f, "{:#?}", err),
            ErrorKind::String(s) => write!(f, "{}", s),
            ErrorKind::BadArguments => write!(f, "bad arguments"),
            ErrorKind::LabelOutsideOfSection => write!(f, "label outside of section"),
            ErrorKind::MissingAdditional => write!(
                f,
                "Improper implementation of instructions, \
                missing additional needed info for instruction (possibly bc1t or similar)."
            ),
            ErrorKind::MissingFunct => write!(
                f,
                "Improper implmentation of instructions (funct field undefined for R-type instr)
                    If you are a student reading this, understand this error comes entirely from a \
                    fundamental failure in the codebase of this assembler.",
            ),
            ErrorKind::MissingFmt => write!(
                f,
                "Improper implementation of instructions, \
                    missing fmt field for instruction."
            ),
            ErrorKind::UnknownInstruction(s) => write!(f, "unkown instruction {}", s),
            ErrorKind::InvalidShamt => write!(f, "invalid shift amount"),
            ErrorKind::InvalidArgument => write!(f, "invalid argument"),
            ErrorKind::ImmediateOverflow(imm) => write!(
                f,
                "immediate overflow on {} (valid range {},{})",
                imm,
                i16::MIN as u32,
                i16::MAX as u32
            ),
            ErrorKind::UndefinedSymbol(s) => write!(f, "undefined symbol {} found.", s),
            ErrorKind::WrongInstructionType => write!(
                f,
                "wrong instruction type \
                defined for instruction."
            ),
        }
    }
}

// Types
pub type AssembleResult<T> = Result<T, ErrorKind>;
pub type AssembleError = Span<ErrorKind>;

// Wrapper type to keep InstructionInformation and FpInstructionInformation together.
pub enum InstructionMeta {
    Int(&'static InstructionInformation),
    Fp(&'static FpInstructionInformation),
}

impl InstructionMeta {
    /// Get the mnemonic of the instruction inside the InstructionMeta wrapper.
    pub fn get_mnemonic(&self) -> String {
        match self {
            Self::Fp(info) => String::from(info.mnemonic),
            Self::Int(info) => String::from(info.mnemonic),
        }
    }

    /// Get a reference to the implementation function inside the InstructionMeta wrapper.
    pub fn get_implementation(
        &self,
    ) -> &Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send> {
        match self {
            Self::Fp(info) => &info.implementation,
            Self::Int(info) => &info.implementation,
        }
    }

    /// Get the lookup code of the underlying information
    pub fn get_lookup(&self) -> u32 {
        match self {
            Self::Fp(i) => i.lookup_code(),
            Self::Int(i) => i.lookup_code(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RawInstruction {
    pub raw: u32,
}

// RawInstruction impls
impl RawInstruction {
    pub fn new(raw: u32) -> RawInstruction {
        RawInstruction { raw }
    }

    pub fn get_opcode(self) -> u32 {
        self.raw >> 26
    }

    pub fn get_funct(self) -> u32 {
        self.raw & 0x3F
    }

    pub fn is_rtype(self) -> bool {
        let op = self.get_opcode();
        op == 0x00 || op == 0x1C
    }

    pub fn is_jtype(self) -> bool {
        let op = self.get_opcode();
        op == 0x02 || op == 0x03
    }

    pub fn is_itype(self) -> bool {
        !self.is_rtype() && !self.is_jtype()
    }

    pub fn is_regimm(self) -> bool {
        self.get_opcode() == 0x01
    }

    pub fn is_floating(self) -> bool {
        self.get_opcode() == 0x11
    }

    pub fn get_rs(self) -> u32 {
        self.raw >> 21 & 0x1F
    }

    pub fn get_rt(self) -> u32 {
        self.raw >> 16 & 0x1F
    }

    pub fn get_rd(self) -> u32 {
        self.raw >> 11 & 0x1F
    }

    pub fn get_shamt(self) -> u32 {
        self.raw >> 6 & 0x1F
    }

    pub fn get_fmt(self) -> u32 {
        self.raw >> 21 & 0x1F
    }

    pub fn get_ft(self) -> u32 {
        self.raw >> 16 & 0x1F
    }

    pub fn get_fs(self) -> u32 {
        self.raw >> 11 & 0x1F
    }

    pub fn get_fd(self) -> u32 {
        self.raw >> 6 & 0x1F
    }

    pub fn get_immediate(self) -> u16 {
        (self.raw & 0xFFFF) as u16
    }

    pub fn get_jump(self) -> u32 {
        self.raw & 0x3FFFFFF
    }

    pub fn get_lookup(self) -> u32 {
        // Normal instructions follow this form:
        // | opcode | multiplexer |
        // Where "multiplexer" might be a funct code or some other secondary identifier.
        let base = self.get_opcode() << 6;
        if self.is_rtype() {
            base | self.get_funct()
        } else if self.is_regimm() {
            base | self.get_rt()
        } else if self.is_floating() {
            // Floating-point instructions have a special format that deviates from standard base.
            // | opcode | funct | fmt | add'l |
            if self.get_fmt() == u32::from(FpFmt::ReservedFunctCodeBC) {
                // If the instruction is a comparison branch, there exists a special case.
                (self.get_opcode() << 13) | (self.get_fmt() << 2) | self.get_ft() & 0b11
            } else {
                // Most floating-point instructions follow this pattern.
                (self.get_opcode() << 13) | (self.get_funct() << 7) | (self.get_fmt() << 2)
            }
        } else {
            base
        }
    }

    pub fn to_be_bytes(self) -> [u8; 4] {
        self.raw.to_be_bytes()
    }
}

// RawInstruction to XArgs conversion

// IArgs

// JArgs

// RArgs

// FpCCArgs

// FpCCBranchArgs

// FpRArgs

// FpFourRegArgs

// RegImmIArgs

// CopMovRArgs