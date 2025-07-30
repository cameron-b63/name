/// This file contains the definition of the CacheType instruction.
/// CacheType is a semantic wrapper over a simple I-Type instruction.
/*
    The $instruction format is defined as:
    | opcode | base | op | offset |
    with:

    base as a general-purpose register;
    op as a 5-bit unsigned immediate code;
    offset as a 16-bit signed immediate;
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction},
    parse::parse::AstKind,
};

///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheArgs {
    pub opcode: u32,
    pub base: u32,
    pub op: u32,
    pub offset: u32,
}

// Define how to pack to raw
impl From<CacheArgs> for RawInstruction {
    fn from(cache: CacheArgs) -> Self {
        RawInstruction::new(
            (cache.opcode << 26) | (cache.base << 21) | (cache.op << 16) | cache.offset,
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for CacheArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: (raw.raw >> 26) & 0b11_1111,
            base: (raw.raw >> 21) & 0b1_1111,
            op: (raw.raw >> 16) & 0b1_1111,
            offset: raw.raw & 0b1111_1111_1111_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl CacheArgs {
    pub fn assign_cache_type_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut base = 0;
        let mut op = 0;
        let mut offset = 0;

        let mut immediates_encountered = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Immediate => match immediates_encountered {
                    0 => {
                        let passed_op = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                        if passed_op < 32 {
                            op = passed_op;
                        } else {
                            return Err(ErrorKind::InvalidArgument);
                        }

                        immediates_encountered += 1;
                    }
                    1 => {
                        offset = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
                        immediates_encountered += 1;
                    }
                    _ => unreachable!(),
                },
                ArgumentType::Rs => {
                    base = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)?
                }
                _ => unreachable!(),
            }
        }

        Ok(Self {
            opcode: 0, // Will be filled in by caller
            base,
            op,
            offset,
        })
    }
}
