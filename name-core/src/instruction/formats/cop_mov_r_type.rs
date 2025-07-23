/// This file contains the definition of the CopMovR (Coprocessor Move R-Type) instruction.

/*
    The CopMovR format is defined as:
    | opcode | function | rt | rd | 0 | sel |
    with:

    function as a multiplex;
    rt as a general-purpose register;
    rd as a coprocessor register;
    sel as a 3-bit immediate;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// CopMovR-Type instructions support operations like mfc0 and mtc1.
#[derive(Debug)]
pub struct CopMovRArgs {
    pub op_code: u32,
    pub funct_code: u32,
    pub rt: u32,
    pub rd: u32,
    pub sel: u32,
}

// Define how to pack to raw
impl From<CopMovRArgs> for RawInstruction {
    fn from(cop_mov_r_args: CopMovRArgs) -> Self {
        RawInstruction::new(
            (cop_mov_r_args.op_code << 26)
                | ((cop_mov_r_args.funct_code << 21))
                | ((cop_mov_r_args.rt << 16))
                | ((cop_mov_r_args.rd << 11))
                | (cop_mov_r_args.sel)
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for CopMovRArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            op_code: (raw.raw >> 26) & 0b11_1111,
            funct_code: (raw.raw >> 21) & 0b1_1111,
            rt: (raw.raw >> 16) & 0b1_1111,
            rd: (raw.raw >> 11) & 0b1_1111,
            sel: raw.raw & 0b111
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl CopMovRArgs {
    pub fn assign_cop_mov_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType]
    ) -> AssembleResult<Self> {
        let mut rt = 0;
        let mut rd = 0;
        let mut sel = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rt => rt = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Rd => rd = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Immediate => {
                    let imm = passed.get_immediate().unwrap_or(0);
                    if imm < 8 {
                        sel = imm;
                    } else {
                        return Err(ErrorKind::InvalidArgument);
                    }
                },
                _ => unreachable!(),
            }
        }

        Ok(Self {
            op_code: 0, // Will be filled in by caller
            funct_code: 0,  // Will be filled in by caller
            rt,
            rd,
            sel
        })
    }
}