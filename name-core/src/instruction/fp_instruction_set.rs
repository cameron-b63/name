use std::sync::LazyLock;

use crate::instruction::{
    fp_implementations,
    information::{wrap_imp, ArgumentType, FpFmt, InstructionType},
};

use super::information::FpInstructionInformation;

/// The floating point instructions are stored separately from normal CPU instructions
/// since they have a different needed field for assembly - fmt.
/// This table is still based on https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00086-2B-MIPS32BIS-AFP-6.06.pdf
pub static FP_INSTRUCTION_SET: LazyLock<Vec<FpInstructionInformation>> = LazyLock::new(|| {
    vec![
        FpInstructionInformation {
            mnemonic: "abs.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x05),
            fmt: Some(FpFmt::Double),
            implementation: wrap_imp(fp_implementations::abs_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "abs.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x05),
            fmt: Some(FpFmt::Single),
            implementation: wrap_imp(fp_implementations::abs_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
    ]
});
