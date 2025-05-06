use crate::{
    elf_def::RelocationEntryType, instruction::instruction::RawInstruction, structs::ProgramState,
};
use std::fmt::Debug;

pub struct InstructionInformation {
    pub mnemonic: &'static str,
    pub instruction_type: InstructionType,
    pub op_code: u32,
    pub funct_code: Option<u32>,
    pub implementation: Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send>,
    pub args: &'static [ArgumentType],
    pub alt_args: Option<&'static [&'static [ArgumentType]]>,
    pub relocation_type: Option<RelocationEntryType>,
}

impl PartialEq for InstructionInformation {
    fn eq(&self, other: &Self) -> bool {
        self.mnemonic == other.mnemonic
    }
}

impl Debug for InstructionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InstructionInformation {{
                mnemonic: {:?},
                instruction_type: {:?}
                op_code: {:?},
                funct_code {:?},
                implementation: {:?},
                args: {:?},
                alt_args: {:?}
                relocation_type: {:?}
            }}",
            self.mnemonic,
            self.instruction_type,
            self.op_code,
            self.funct_code,
            self.instruction_type,
            self.args,
            self.alt_args,
            self.relocation_type,
        )
    }
}

impl InstructionInformation {
    pub fn lookup_code(&self) -> u32 {
        self.op_code << 6 | self.funct_code.unwrap_or(0)
    }
}

pub struct FpInstructionInformation {
    pub mnemonic: &'static str,
    pub instruction_type: InstructionType,
    pub op_code: u32,
    pub funct_code: Option<u32>,
    pub fmt: Option<FpFmt>,
    pub implementation: Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send>,
    pub args: &'static [ArgumentType],
    pub alt_args: Option<&'static [&'static [ArgumentType]]>,
    pub relocation_type: Option<RelocationEntryType>,
}

impl PartialEq for FpInstructionInformation {
    fn eq(&self, other: &Self) -> bool {
        self.mnemonic == other.mnemonic
    }
}

impl Debug for FpInstructionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InstructionInformation {{
                mnemonic: {:?},
                instruction_type: {:?}
                op_code: {:?},
                funct_code {:?},
                fmt {:?},
                implementation: {:?},
                args: {:?},
                alt_args: {:?},
                relocation_type: {:?}
            }}",
            self.mnemonic,
            self.instruction_type,
            self.op_code,
            self.funct_code,
            self.fmt,
            self.instruction_type,
            self.args,
            self.alt_args,
            self.relocation_type,
        )
    }
}

impl FpInstructionInformation {
    pub fn lookup_code(&self) -> u32 {
        (self.op_code << 11)
            | (self.funct_code.unwrap_or(0) << 5)
            | u32::from(self.fmt.unwrap_or(FpFmt::Reserved))
    }
}

pub fn wrap_imp<Args: From<RawInstruction> + 'static>(
    f: fn(&mut ProgramState, Args) -> (),
) -> Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send> {
    Box::new(move |program_state, instr| f(program_state, Args::from(instr)))
}

#[derive(Debug, PartialEq)]
pub enum InstructionType {
    RType,
    IType,
    JType,
    FpRType,
}

#[derive(Debug, PartialEq)]
pub enum ArgumentType {
    Rd,
    Rs,
    Rt,
    Fd,
    Fs,
    Ft,
    Immediate,
    Identifier,
    BranchLabel,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FpFmt {
    Reserved,
    Single,
    Double,
}

// Cast FpFmt to its proper u32 rep, page 115
// https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00082-2B-MIPS32INT-AFP-06.01.pdf
impl From<FpFmt> for u32 {
    fn from(fmt: FpFmt) -> Self {
        match fmt {
            FpFmt::Reserved => 0,
            FpFmt::Single => 16,
            FpFmt::Double => 17,
        }
    }
}
