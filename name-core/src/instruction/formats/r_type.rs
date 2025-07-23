/// This file contains the definition of the R-Type instruction.
/*
    The R-Type (Register) instruction is defined as:
    | opcode | rs | rt | rd | shamt | funct |
    with:

    rs, rt, and rd as general-purpose registers;
    shamt as a 5-bit unsigned immediate;
    funct as a multiplex;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// The R-Type instruction is used for most register = register (op) register operations.
#[derive(Debug)]
pub struct RArgs {
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub rd: u32,
    pub shamt: u32,
    pub funct: u32,
}

// Define how to pack to raw
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

// Define how to unpack from raw
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

// Define how to pack a set of arguments into this struct during assembly
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
                    let sa = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                    // Bounds check on shift amount
                    if sa > 31 {
                        return Err(ErrorKind::InvalidShamt);
                    } else {
                        shamt = sa;
                    }
                }
                _ => unreachable!(),
            }
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