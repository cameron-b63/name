/// This file contains the definition of the RegImm (Register-Immediate I-Type) instruction.
/// RegImm is an idiomatic instruction type, and is an alias of I-Type.

/*
    The RegImm format is defined as:
    | opcode | rs | function | offset |
    with:

    rs as a general-purpose register;
    function as a multiplex;
    offset as a 16-bit signed immediate;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// Register-Immediate instructions are used for instructions like bltz (which just test a single register)
#[derive(Debug)]
pub struct RegImmIArgs {
    pub op_code: u32,
    pub rs: u32,
    pub regimm_funct_code: u32,
    pub imm: u32,
}

// Define how to pack to raw
impl From<RegImmIArgs> for RawInstruction {
    fn from(reg_imm_args: RegImmIArgs) -> Self {
        RawInstruction::new(
            (reg_imm_args.op_code << 26)
                | (reg_imm_args.rs << 21)
                | (reg_imm_args.regimm_funct_code << 16)
                | reg_imm_args.imm
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for RegImmIArgs {
    fn from(raw: RawInstruction) -> Self {
        RegImmIArgs {
            op_code: raw.get_opcode(),
            rs: raw.get_rs(),
            regimm_funct_code: raw.get_rt(),
            imm: raw.get_immediate() as u32,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl RegImmIArgs {
    pub fn assign_regimm_i_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut rs = 0;
        let mut imm = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rs => {
                    rs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Immediate => {
                    imm = passed.get_immediate().unwrap_or(0);
                }
                _ => unreachable!(),
            }
        }

        return Ok(Self {
            op_code: 0, // Will be filled in by caller
            rs,
            regimm_funct_code: 0, // Will be filled in by caller
            imm,
        });
    }
}