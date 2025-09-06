/// This file contains the definition of the BitFieldType instruction.
/// This is a semantic wrapper over R-Type
/*
    The BitField format is defined as:
    | opcode | rs | rt | msbd | lsb | function |
    with:

    rs, rt as general-purpose registers;
    msbd, lsb as 5-bit immediate bit-indices;
    function as a multiplex;
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction},
    parse::parse::AstKind,
};

/// The BitFieldType instruction format exists for instructions like EXT and INS, which deal with specific bit fields.
/// It is a semantic wrapper over R-Type, but with two immediates. This special packing case means it's more ergonomic
/// to have a separate type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BitFieldArgs {
    pub opcode: u32,
    pub rs: u32,
    pub rt: u32,
    pub msbd: u32,
    pub lsb: u32,
    pub funct: u32,
}

// Define how to pack to raw
impl From<BitFieldArgs> for RawInstruction {
    fn from(bit_field_args: BitFieldArgs) -> Self {
        RawInstruction::new(
            (bit_field_args.opcode << 26)
                | (bit_field_args.rs << 21)
                | (bit_field_args.rt << 16)
                | (bit_field_args.msbd << 11)
                | (bit_field_args.lsb << 6)
                | bit_field_args.funct,
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for BitFieldArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: (raw.raw >> 26) & 0b11_1111,
            rs: (raw.raw >> 21) & 0b1_1111,
            rt: (raw.raw >> 16) & 0b1_1111,
            msbd: (raw.raw >> 11) & 0b1_1111,
            lsb: (raw.raw >> 6) & 0b1_1111,
            funct: raw.raw & 0b11_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl BitFieldArgs {
    pub fn assign_bit_field_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut rs = 0;
        let mut rt = 0;
        let mut msbd = 0;
        let mut lsb = 0;

        let mut encountered_immediates = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rs => {
                    rs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Rt => {
                    rt = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                ArgumentType::Immediate => match encountered_immediates {
                    0 => {
                        let first_imm = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                        if 0 < first_imm && first_imm < 32 {
                            msbd = first_imm;
                        } else {
                            return Err(ErrorKind::InvalidArgument);
                        }
                        encountered_immediates += 1;
                    }
                    1 => {
                        let second_imm =
                            passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                        if second_imm < 32 && (second_imm + msbd) <= 32 {
                            lsb = second_imm;
                        } else {
                            return Err(ErrorKind::InvalidArgument);
                        }
                        encountered_immediates += 1;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }

        Ok(Self {
            opcode: 0, // Will be filled in by caller
            rs,
            rt,
            msbd,
            lsb,
            funct: 0, // Will be filled in by caller
        })
    }
}
