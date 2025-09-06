/// This file contains the definition of the I-Type instruction.
/*
    The I-Type (Immediate) format is defined as:
    | opcode | rs | rt | immediate |
    with:

    rs, rt as general-purpose registers;
    immediate as a 16-bit signed immediate;
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction},
    parse::parse::AstKind,
};

/// The I-Type instruction is most commonly used for
/// performing arithmetic operations with immediates
/// and branching.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IArgs {
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub imm: u32,
}

// Define how to pack to raw
impl From<IArgs> for RawInstruction {
    fn from(i_args: IArgs) -> Self {
        RawInstruction::new(
            (i_args.opcode << 26) | ((i_args.rs) << 21) | ((i_args.rt) << 16) | (i_args.imm as u32),
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for IArgs {
    fn from(raw: RawInstruction) -> IArgs {
        IArgs {
            opcode: raw.get_opcode(),
            rs: (raw.raw >> 21) & 0b1_1111,
            rt: (raw.raw >> 16) & 0b1_1111,
            imm: raw.raw & 0b1111_1111_1111_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
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
            imm: imm as i32 as i16 as u16 as u32,
        });
    }
}
