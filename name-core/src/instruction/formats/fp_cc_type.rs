/// This file contains the definition of the FpCC (Floating-point Condition Code) instruction.
/*
    The FpCC format is defined as:
    | opcode | fmt | ft | fs | cc | 0 | function |
    with:

    fmt as a 5-bit immediate;
    ft, fs as floating-point registers;
    cc as a 3-bit immediate;
    function as a multiplex;
*/
use crate::{
    instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction},
    parse::parse::AstKind,
};

/// FpCC instructions are used to perform comparisons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpCCArgs {
    pub opcode: u32,
    pub fmt: u32,
    pub ft: u32,
    pub fs: u32,
    pub cc: u32,
    pub funct: u32,
}

// Define how to pack to raw
impl From<FpCCArgs> for RawInstruction {
    fn from(fp_cc_args: FpCCArgs) -> Self {
        RawInstruction::new(
            (fp_cc_args.opcode << 26)
                | ((fp_cc_args.fmt) << 21)
                | ((fp_cc_args.ft) << 16)
                | ((fp_cc_args.fs) << 11)
                | ((fp_cc_args.cc) << 8)    // Note that this is 8, not 6.
                | (fp_cc_args.funct),
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for FpCCArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: raw.get_opcode(),
            fmt: (raw.raw >> 21) & 0b1_1111,
            ft: (raw.raw >> 16) & 0b1_1111,
            fs: (raw.raw >> 11) & 0b1_1111,
            cc: (raw.raw >> 8) & 0b111,
            funct: raw.raw & 0b11_1111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl FpCCArgs {
    pub fn assign_fp_cc_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut ft = 0;
        let mut fs = 0;
        let mut cc = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Ft => {
                    ft = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32;
                }
                ArgumentType::Fs => {
                    fs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32;
                }
                ArgumentType::Immediate => {
                    if let AstKind::Immediate(num) = passed {
                        // This may need more editing later should we end up implementing paired-single format.
                        if num < 8 {
                            cc = num;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        return Ok(Self {
            opcode: 0, // Filled in by caller
            fmt: 0,    // Filled in by caller
            ft,
            fs,
            cc,
            funct: 0, // Filled in by caller
        });
    }
}
