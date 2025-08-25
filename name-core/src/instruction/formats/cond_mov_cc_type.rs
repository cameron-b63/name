/*
    The CondMovCCType format is defined as:
    | opcode | rs | cc | 0 | tf | rd | 0 | funct |
    with:

    rs, rd as general-purpose registers;
    cc as a 3-bit condition code immediate;
    tf as a 1-bit selector;
    funct as a selector;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// CondMovCCArgs is pretty much just reserved for movf and movt.
/// It is syntactic sugar for us (semantic wrapper over r-type)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CondMovCCArgs {
    pub opcode: u32,
    pub rs: u32,
    pub cc: u32,
    pub tf: u32,
    pub rd: u32,
    pub funct: u32,
}

// Define how to pack to raw
impl From<CondMovCCArgs> for RawInstruction {
    fn from(cond_mov_cc_args: CondMovCCArgs) -> Self {
        RawInstruction::new(
            (cond_mov_cc_args.opcode << 26)
            | (cond_mov_cc_args.rs << 21)
            | (cond_mov_cc_args.cc << 18)
            | (cond_mov_cc_args.tf << 16)
            | (cond_mov_cc_args.rd << 11)
            | cond_mov_cc_args.funct
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for CondMovCCArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: (raw.raw >> 26) & 0b11_1111,
            rs: (raw.raw >> 21) & 0b1_1111,
            cc: (raw.raw >> 18) & 0b111,
            tf: (raw.raw >> 16) & 0b1,
            rd: (raw.raw >> 11) & 0b1_1111,
            funct: raw.raw & 0b11_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl CondMovCCArgs {
    pub fn assign_cond_mov_cc_args(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType]
    ) -> AssembleResult<Self> {
        let mut rs = 0;
        let mut cc = 0;
        let mut rd = 0;

        for(i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Rs => rs = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Immediate => cc = passed.get_immediate().unwrap_or(0),
                ArgumentType::Rd => rd = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                _ => unreachable!(),
            }
        }

        Ok(Self {
            opcode: 0, // Will be filled in by caller
            rs,
            cc,
            tf: 0, // Will be filled in by caller
            rd,
            funct: 0, // Will be filled in by caller
        })
    }
}