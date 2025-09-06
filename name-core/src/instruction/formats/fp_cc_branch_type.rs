/// This file contains the definition of the $instruction instruction.
/*
    The FpCC format is defined as:
    | opcode | BCC1 | cc | nd | tf | offset |
    with:

    BCC1 as a reserved immediate indicator;
    cc as a 3-bit immediate to select condition code;
    nd, tf as 1-bit immediates to dictate type of branch;
    offset as a 16-bit signed immediate pc-relative offset.
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction},
    parse::parse::AstKind,
};

/// The FpCCBranchArgs is for instructions like bc1t and bc1fl.
/// The fields tf and nd are another layer of indirection to get the right instruction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpCCBranchArgs {
    pub opcode: u32,
    pub funky_funct: u32, // The funct code is in a different place for this format. Little odd.
    pub cc: u32,
    pub tf: u32, // True/False
    pub nd: u32, // Nullify delay slot ("likely" will set this bit to 1)
    pub offset: u32,
}

// Define how to pack to raw
impl From<FpCCBranchArgs> for RawInstruction {
    fn from(fp_cc_branch_args: FpCCBranchArgs) -> Self {
        RawInstruction::new(
            (fp_cc_branch_args.opcode << 26)
                | ((fp_cc_branch_args.funky_funct) << 21)
                | ((fp_cc_branch_args.cc) << 18)
                | ((fp_cc_branch_args.nd) << 17)
                | ((fp_cc_branch_args.tf) << 16)
                | (fp_cc_branch_args.offset as u32),
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for FpCCBranchArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: raw.get_opcode(),
            funky_funct: (raw.raw >> 21) & 0b1_1111,
            cc: (raw.raw >> 18) & 0b111,
            nd: (raw.raw >> 17) & 1,
            tf: (raw.raw >> 16) & 1,
            offset: raw.raw & 0b1111_1111_1111_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl FpCCBranchArgs {
    pub fn assign_fp_cc_branch_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut cc = 0;
        let mut is_first_imm: bool = true; // This just keeps track of whether this is the first immediate we've encountered.
        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Immediate => {
                    if is_first_imm {
                        if let AstKind::Immediate(num) = passed {
                            if num < 8 {
                                cc = num;
                            } else {
                                return Err(ErrorKind::InvalidArgument);
                            }
                        }

                        is_first_imm = false;
                    } else {
                        // Must be the second immediate, should go to zero.
                        // Do nothing.
                    }
                }
                ArgumentType::Identifier => (),
                _ => todo!("Figure out what to do about bc1t cc, (imm) issue? maybe?"),
            }
        }

        Ok(Self {
            opcode: 0,
            funky_funct: 0,
            cc,
            tf: 0,
            nd: 0,
            offset: 0,
        })
    }
}
