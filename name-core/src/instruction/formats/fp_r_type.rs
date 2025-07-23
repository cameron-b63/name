/// This file contains the definition of the FpR (Floating-point Register) Type instruction.

/*
    The FpR format is defined as:
    | opcode | fmt | ft | fs | fd | function |
    with:

    fmt as a 5-bit immediate;
    ft, fs, fd as floating-point registers;
    function as a multiplex;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// FpR-Type instructions are very common and used to perform arithmetic on floating-point registers.
#[derive(Debug)]
pub struct FpRArgs {
    pub opcode: u32,
    pub fmt: u32,
    pub ft: u32,
    pub fs: u32,
    pub fd: u32,
    pub funct: u32,
}

// Define how to pack to raw
impl From<FpRArgs> for RawInstruction {
    fn from(fp_r_args: FpRArgs) -> Self {
        RawInstruction::new(
            (fp_r_args.opcode << 26)
                | ((fp_r_args.fmt) << 21)
                | ((fp_r_args.ft) << 16)
                | ((fp_r_args.fs) << 11)
                | ((fp_r_args.fd) << 6)
                | (fp_r_args.funct),
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for FpRArgs {
    fn from(raw: RawInstruction) -> Self {
        Self {
            opcode: raw.get_opcode(),
            fmt: raw.get_fmt(),
            fs: raw.get_fs(),
            ft: raw.get_ft(),
            fd: raw.get_fd(),
            funct: raw.get_funct(),
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl FpRArgs {
    pub fn assign_fp_r_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType],
    ) -> AssembleResult<Self> {
        let mut ft = 0;
        let mut fs = 0;
        let mut fd = 0;

        for (i, passed) in arguments.into_iter().enumerate() {
            match args_to_use[i] {
                ArgumentType::Fd => {
                    fd = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Fs => {
                    fs = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Ft => {
                    ft = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                ArgumentType::Rt => {
                    fd = passed
                        .get_register_as_u32()
                        .ok_or(ErrorKind::InvalidArgument)? as u32
                }
                _ => unreachable!(),
            }
        }

        return Ok(Self {
            opcode: 0, // Will be filled in by caller
            fmt: 0,    // Will be filled in by caller
            ft,
            fs,
            fd,
            funct: 0, // Will be filled in by caller
        });
    }
}