/// This file contains the definition of the J-Type instruction.
/*
    The J-Type (jump) format is defined as:
    | opcode | instruction_index |
    with:

    instruction_index as a 26-bit unsigned immediate instruction address
        (shifted right by 2 because of the necessary MIPS32 4-bit alignment)
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, RawInstruction},
    parse::parse::AstKind,
};

/// The J-Type instruction is used to facilitate jumps where the target PC exceeds the branch range
/// offered by the 16-bit signed offset in I-Type instructions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JArgs {
    pub opcode: u32,
    pub address: u32,
}

// Define how to pack to raw
impl From<JArgs> for RawInstruction {
    fn from(j_args: JArgs) -> Self {
        RawInstruction::new((j_args.opcode << 26) | (j_args.address))
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for JArgs {
    fn from(raw: RawInstruction) -> JArgs {
        JArgs {
            opcode: raw.get_opcode(),
            address: raw.raw & 0b0000_0011_1111_1111_1111_1111_1111_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
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
