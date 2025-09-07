use crate::{
    elf_def::RelocationEntryType,
    instruction::{
        formats::{
            bit_field_type::BitFieldArgs, cond_mov_cc_type::CondMovCCArgs,
            cop_mov_r_type::CopMovRArgs, fp_cc_branch_type::FpCCBranchArgs, fp_cc_type::FpCCArgs,
            fp_four_reg_type::FpFourRegArgs, fp_r_type::FpRArgs, i_type::IArgs, j_type::JArgs,
            r_type::RArgs, regimm_i_type::RegImmIArgs,
        },
        instruction::RawInstruction,
    },
    structs::ProgramState,
};
use std::fmt::Debug;

pub struct InstructionInformation {
    pub mnemonic: &'static str,
    pub basis: InstructionType,
    pub implementation: Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send>,
    pub args: &'static [&'static [ArgumentType]],
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
                args: {:?},
                relocation_type: {:?}
            }}",
            self.mnemonic, self.args, self.relocation_type,
        )
    }
}

impl InstructionInformation {
    pub fn lookup_code(&self) -> u32 {
        return RawInstruction::from(&self.basis).raw;
    }
}

pub fn wrap_imp<Args: From<RawInstruction> + 'static>(
    f: fn(&mut ProgramState, Args) -> (),
) -> Box<dyn Fn(&mut ProgramState, RawInstruction) -> () + Sync + Send> {
    Box::new(move |program_state, instr| f(program_state, Args::from(instr)))
}

/// InstructionType variants contain an instance of their arguments to create the "basis" for instruction information.
/// The primary key for decoding in the fetch-decode-execute cycle is derived from the packed version of this "basis".
#[derive(Debug, PartialEq)]
pub enum InstructionType {
    BitFieldType(BitFieldArgs),
    CondMovCCType(CondMovCCArgs),
    CopMovRType(CopMovRArgs),
    FpBranchType(FpCCBranchArgs),
    FpCCType(FpCCArgs),
    FpFourRegister(FpFourRegArgs),
    FpRType(FpRArgs),
    IType(IArgs),
    JType(JArgs),
    RegImmIType(RegImmIArgs),
    RType(RArgs),
}

impl From<&InstructionType> for RawInstruction {
    fn from(instr_type: &InstructionType) -> Self {
        match instr_type {
            InstructionType::BitFieldType(args) => RawInstruction::from(*args),
            InstructionType::CondMovCCType(args) => RawInstruction::from(*args),
            InstructionType::CopMovRType(args) => RawInstruction::from(*args),
            InstructionType::FpBranchType(args) => RawInstruction::from(*args),
            InstructionType::FpCCType(args) => RawInstruction::from(*args),
            InstructionType::FpFourRegister(args) => RawInstruction::from(*args),
            InstructionType::FpRType(args) => RawInstruction::from(*args),
            InstructionType::IType(args) => RawInstruction::from(*args),
            InstructionType::JType(args) => RawInstruction::from(*args),
            InstructionType::RegImmIType(args) => RawInstruction::from(*args),
            InstructionType::RType(args) => RawInstruction::from(*args),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ArgumentType {
    Rd,
    Rs,
    Rt,
    Fd,
    Fs,
    Ft,
    Fr,
    Immediate,
    Identifier,
    BranchLabel,
}

// Define FpFmt with its proper u32 rep, page 115
// https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00082-2B-MIPS32INT-AFP-06.01.pdf
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FpFmt {
    Reserved = 0,
    Single = 16,
    Double = 17,
}

// Cast FpFmt to its proper u32 rep in fmt3 form, page 115
// https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00082-2B-MIPS32INT-AFP-06.01.pdf
impl FpFmt {
    pub fn to_fmt3(&self) -> u32 {
        match self {
            FpFmt::Single => 0,
            FpFmt::Double => 1,
            _ => 2,
        }
    }
}
