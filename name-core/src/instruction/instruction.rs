use std::{fmt, io};

use super::information::{ArgumentType, FpInstructionInformation, InstructionInformation};
use crate::{parse::{parse::AstKind, span::Span}, structs::ProgramState};

/// Possible assemble error codes
#[derive(Debug)]
pub enum ErrorKind {
    DuplicateSymbol(String),
    Io(io::Error),
    String(String),
    BadArguments,
    LabelOutsideOfSection,
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
            ErrorKind::Io(err) => write!(f, "{:#?}", err),
            ErrorKind::String(s) => write!(f, "{}", s),
            ErrorKind::BadArguments => write!(f, "bad arguments"),
            ErrorKind::LabelOutsideOfSection => write!(f, "label outside of section"),
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
    pub fn get_implementation(&self) -> &Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send> {
        match self {
            Self::Fp(info) => &info.implementation,
            Self::Int(info) => &info.implementation,
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
        let base = self.get_opcode() << 6;
        if self.is_rtype() {
            base | self.get_funct()
        } else if self.is_regimm() {
            base | self.get_rt()
        } else if self.is_floating() {
            (self.get_opcode() << 11) | (self.get_funct() << 5) | self.get_fmt()
        } else {
            base
        }
    }

    pub fn to_be_bytes(self) -> [u8; 4] {
        self.raw.to_be_bytes()
    }
}

// RawInstruction to XArgs conversion
impl From<IArgs> for RawInstruction {
    fn from(i_args: IArgs) -> Self {
        RawInstruction::new(
            (i_args.opcode << 26) | ((i_args.rs) << 21) | ((i_args.rt) << 16) | (i_args.imm as u32),
        )
    }
}

impl From<JArgs> for RawInstruction {
    fn from(j_args: JArgs) -> Self {
        RawInstruction::new((j_args.opcode << 26) | (j_args.address))
    }
}

impl From<RArgs> for RawInstruction {
    fn from(r_args: RArgs) -> Self {
        RawInstruction::new(
            (r_args.opcode << 26)
                | ((r_args.rs) << 21)
                | ((r_args.rt) << 16)
                | ((r_args.rd) << 11)
                | ((r_args.shamt) << 6)
                | (r_args.funct),
        )
    }
}

impl From<FpRArgs> for RawInstruction {
    fn from(fp_r_args: FpRArgs) -> Self {
        RawInstruction::new(
            (fp_r_args.opcode << 26)
                | ((fp_r_args.fmt) << 21)
                | ((fp_r_args.ft) << 16)
                | ((fp_r_args.fs) << 11)
                | ((fp_r_args.fd) << 6)
                | (fp_r_args.funct),
        )
    }
}

// IArgs
#[derive(Debug)]
pub struct IArgs {
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub imm: u16,
}

impl IArgs {
    pub fn assign_i_type_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut rs: u32 = 0;
        let mut rt: u32 = 0;
        let mut imm: u32 = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rs | ArgumentType::Fs => {
                    rs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Rt | ArgumentType::Ft => {
                    rt = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Immediate => imm = passed.get_immediate().unwrap_or(0),
                ArgumentType::Identifier | ArgumentType::BranchLabel => (),
                _ => unreachable!(),
            }
        }

        // Check if the extracted immediate falls within valid range
        if ((imm as i32) as i16) as u16 != imm as u16 {
            return Err(ErrorKind::ImmediateOverflow(imm));
        }

        return Ok(Self {
            opcode: 0,
            rs,
            rt,
            imm: imm as u16,
        });
    }
}

impl From<RawInstruction> for IArgs {
    fn from(raw: RawInstruction) -> IArgs {
        IArgs {
            opcode: raw.get_opcode(),
            rs: raw.get_rs(),
            rt: raw.get_rt(),
            imm: raw.get_immediate(),
        }
    }
}

// JArgs
pub struct JArgs {
    pub opcode: u32,
    pub address: u32,
}

impl JArgs {
    pub fn assign_j_type_arguments(
        _arguments: Vec<AstKind>,
        _args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        // Nothing ever happens...
        Ok(Self {
            opcode: 0,  // Will be filled in by caller
            address: 0, // Will be handled by linker
        })
    }
}

impl From<RawInstruction> for JArgs {
    fn from(raw: RawInstruction) -> JArgs {
        JArgs {
            opcode: raw.get_opcode(),
            address: raw.get_jump(),
        }
    }
}

// RArgs
pub struct RArgs {
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub rd: u32,
    pub shamt: u32,
    pub funct: u32,
}

impl RArgs {
    pub fn assign_r_type_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut rd = 0;
        let mut rs = 0;
        let mut rt = 0;
        let mut shamt = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rd | ArgumentType::Fd => {
                    rd = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Rs | ArgumentType::Fs => {
                    rs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Rt | ArgumentType::Ft => {
                    rt = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Immediate => {
                    shamt = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                }
                _ => unreachable!(),
            }
        }

        // Bounds check on shift amount
        if shamt > 31 {
            return Err(ErrorKind::InvalidShamt);
        }

        return Ok(Self {
            opcode: 0, // Will be filled in by the caller.
            rd,
            rs,
            rt,
            shamt,
            funct: 0, // This will be filled in by the caller.
        });
    }
}

impl From<RawInstruction> for RArgs {
    fn from(raw: RawInstruction) -> RArgs {
        RArgs {
            opcode: raw.get_opcode(),
            rs: raw.get_rs(),
            rt: raw.get_rt(),
            rd: raw.get_rd(),
            shamt: raw.get_shamt(),
            funct: raw.get_funct(),
        }
    }
}

// FpRArgs
pub struct FpRArgs {
    pub opcode: u32,
    pub fmt: u32,
    pub ft: u32,
    pub fs: u32,
    pub fd: u32,
    pub funct: u32,
}

impl FpRArgs {
    pub fn assign_fp_r_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut ft = 0;
        let mut fs = 0;
        let mut fd = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Fd => {
                    fd = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Fs => {
                    fs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Ft => {
                    ft = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                _ => unreachable!(),
            }
        }

        return Ok(Self {
            opcode: 0, // Will be filled in by caller
            fmt: 0,    // Will be filled in by caller
            ft,
            fs,
            fd,
            funct: 0, // Will be filled in by caller
        });
    }
}

impl From<RawInstruction> for FpRArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: raw.get_opcode(),
            fmt: raw.get_fmt(),
            fs: raw.get_fs(),
            ft: raw.get_ft(),
            fd: raw.get_fd(),
            funct: raw.get_funct(),
        }
    }
}
