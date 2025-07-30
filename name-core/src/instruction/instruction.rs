use std::{fmt, io, path::PathBuf};

use crate::{exception::definitions::ExceptionType, parse::span::Span};

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

    /// Uses a bitmask to obtain the key bits (opcode, funct code, fmt, etc.)
    /// The bitmask can be determined by matching on the opcode.
    /// Some opcodes specify instruction classes. These classes will all
    /// have the same method of multiplexing to select the correct instruction.
    pub fn get_lookup(self) -> Result<u32, ExceptionType> {
        let bitmask: u32 = match self.get_opcode() {
            0x00 => {
                // SPECIAL opcode. Multiplex using funct code in bottom 6 bits.
                // Instruction class provides another layer of indirection.
                match self.raw & 0b11_1111 {
                    // Most instructions will be uniquely identified by the opcode and funct code.
                    // However, there are some instruction classes.
                    // MOVCI instruction class:
                    0x01 => 0b1111_1100_0000_0001_0000_0000_0011_1111,
                    // SRL instruction class:
                    0x02 => 0b1111_1100_0010_0000_0000_0000_0011_1111,
                    // SRLV instruction class:
                    0x06 => 0b1111_1100_0000_0000_0000_0000_0111_1111,
                    // Default case (no additional indirection)
                    _ => 0b1111_1100_0000_0000_0000_0000_0011_1111,
                }
            }
            0x01 => {
                // REGIMM opcode. Multiplex using funct code in 20..16
                // no additional indirection.
                0b1111_1100_0001_1111_0000_0000_0000_0000
            }
            0x10 => {
                // COP0 opcode. Multiplex using funct code in 25..21
                match (self.raw >> 21) & 0b1_1111 {
                    // MFMC0 instruction class:
                    0x0b => 0b1111_1111_1110_0000_0000_0000_0010_0000,
                    // rs = C0 instruction class:
                    0x10..=0x1f => 0b1111_1111_1110_0000_0000_0011_1111,
                    _ => 0b1111_1111_1110_0000_0000_0000_0000_0000,
                }
            }
            0x11 => {
                // COP1 opcode. Multiplex using funct code in bottom 6 as well as fmt in 25..21
                // There exist multiple layers of indirection for this instruction.
                match (self.raw >> 21) & 0b1_1111 {
                    // BC1 instruction class:
                    0x04 => 0b1111_1111_1110_0011_0000_0000_0000_0000,
                    // BC1ANY2 instruction class:
                    // Formatted instructions:
                    0x10 | 0x11 | 0x16 => {
                        // Select using funct code in bottom 6 bits
                        match self.raw & 0b11_1111 {
                            // MOVCF instruction class:
                            0x11 => 0b1111_1111_1110_0001_0000_0000_0011_1111,
                            _ => 0b1111_1111_1110_0000_0000_0000_0011_1111,
                        }
                    }
                    _ => 0b1111_1111_1110_0000_0000_0000_0011_1111,
                }
            }
            0x13 => {
                // COP1X opcode. Multiplex using funct code in bottom 6 bits.
                // No additional indirection.
                0b1111_1100_0000_0000_0000_0000_0011_1111
            }
            0x1c => {
                // SPECIAL2 opcode. Multiplex using funct code in bottom 6 bits.
                // No additional indirection.
                0b1111_1100_0000_0000_0000_0000_0011_1111
            }
            0x1f => {
                // SPECIAL3 opcode. Multiplex using funct code in bottom 6 bits.
                match self.raw & 0b11_1111 {
                    // There is exactly one instruction class defined: BSHFL.
                    0x20 => 0b1111_1100_0000_0000_0000_0111_1111_1111,
                    _ => 0b1111_1100_0000_0000_0000_0000_0011_1111,
                }
            }
            0x12 | 0x32 | 0x36 | 0x3a | 0x3e => {
                // These opcodes represent pieces of functionality that are
                // attached to coprocessor 2 and therefore
                // out of scope for NAME.
                return Err(ExceptionType::CoprocessorUnusable);
            }
            _ => {
                // For any other opcode, no multiplexing takes place.
                0b1111_1100_0000_0000_0000_0000_0000_0000
            }
        };

        // Mask out any of the operands for the purposes of lookup
        return Ok(self.raw & bitmask);
    }

    pub fn to_be_bytes(self) -> [u8; 4] {
        self.raw.to_be_bytes()
    }
}
