/// This file contains the definition of the FpFourReg (Formatted FPU Four-Register) Type instruction.

/*
    The FpFourReg format is defined as:
    | opcode | fr | ft | fs | fd | op4 | fmt3 |
    with:

    fr, ft, fs, fd as floating-point registers;
    op4 as a multiplex;
    fmt3 as a 3-bit immediate;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// FpFourReg instructions are used to perform arithmetic operations between four registers (like madd.fmt, multiply-then-add).
#[derive(Debug)]
pub struct FpFourRegArgs {
    pub op_code: u32,
    pub fr: u32,
    pub ft: u32,
    pub fs: u32,
    pub fd: u32,
    pub op4: u32,
    pub fmt3: u32,
}

// Define how to pack to raw
impl From<FpFourRegArgs> for RawInstruction {
    fn from(fp_four_reg_args: FpFourRegArgs) -> Self {
        RawInstruction::new(
            (fp_four_reg_args.op_code << 26)
                | ((fp_four_reg_args.fr << 21))
                | ((fp_four_reg_args.ft << 16))
                | ((fp_four_reg_args.fs << 11))
                | ((fp_four_reg_args.fd << 6))
                | ((fp_four_reg_args.op4 << 3))
                | (fp_four_reg_args.fmt3)
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for FpFourRegArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            op_code: (raw.raw >> 26) & 0b111111,
            fr: (raw.raw >> 21) & 0b11111,
            ft: (raw.raw >> 16) & 0b11111,
            fs: (raw.raw >> 11) & 0b11111,
            fd: (raw.raw >> 6) & 0b11111,
            op4: (raw.raw >> 3) & 0b111,
            fmt3: (raw.raw >> 3) & 0b111,
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl FpFourRegArgs {
    pub fn assign_fp_four_reg_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut fr = 0;
        let mut ft = 0;
        let mut fs = 0;
        let mut fd = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Fr => fr = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Ft => ft = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Fs => fs = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                ArgumentType::Fd => fd = passed.get_register_as_u32().ok_or(ErrorKind::InvalidArgument)? as u32,
                _ => unreachable!(),
            }
        }

        return Ok(
            Self {
                op_code: 0, // Caller will fill this in
                fr,
                ft,
                fs,
                fd,
                op4: 0, // Caller has this
                fmt3: 0, // Caller has this
            }
        );
    }
}
