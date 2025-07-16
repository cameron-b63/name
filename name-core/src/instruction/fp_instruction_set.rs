use std::sync::LazyLock;

use crate::{
    elf_def::RelocationEntryType,
    instruction::{
        fp_implementations,
        information::{wrap_imp, ArgumentType, FpFmt, InstructionType},
    },
};

use super::information::FpInstructionInformation;

/// The floating point instructions are stored separately from normal CPU instructions
/// since they have a different needed field for assembly - fmt.
/// This table is still based on the [MIPS specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00086-2B-MIPS32BIS-AFP-6.06.pdf).
/// A note about condition codes for comparisons:
/// there exist three bits of specificity for condition codes, for 8 possible condition codes.
/// When making a comparison, the implied condition code to store the result in is 0.
/// However, you may reference whichever condition code you like.
/// For paired-single operations, the condition code must be even.
/// For all condition code usage, it must be 0..=7; This is because there are only 3 bits.
/// The condition code is stored in FpRType instructions where the fd field typically would be.
pub static FP_INSTRUCTION_SET: LazyLock<Vec<FpInstructionInformation>> = LazyLock::new(|| {
    vec![
        FpInstructionInformation {
            mnemonic: "abs.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x05),
            fmt: Some(FpFmt::Double),
            additional_code: None,
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
            additional_code: None,
            implementation: wrap_imp(fp_implementations::abs_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "add.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::add_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "add.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::add_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        // ALNV.PS does not apply to our FPU.
        FpInstructionInformation {
            mnemonic: "bc1f",
            instruction_type: InstructionType::FpBranchType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeBC),
            additional_code: Some(0b00), // Not "likely", branch on condition code false
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        FpInstructionInformation {
            mnemonic: "bc1fl",
            instruction_type: InstructionType::FpBranchType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeBC),
            additional_code: Some(0b10), // "likely", branch on condition code false
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        FpInstructionInformation {
            mnemonic: "bc1t",
            instruction_type: InstructionType::FpBranchType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeBC),
            additional_code: Some(0b01), // Not "likely", branch on condition code true
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        FpInstructionInformation {
            mnemonic: "bc1tl",
            instruction_type: InstructionType::FpBranchType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeBC),
            additional_code: Some(0b11), // "likely", branch on condition code true
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        FpInstructionInformation {
            mnemonic: "c.eq.d",
            instruction_type: InstructionType::FpCCType,
            op_code: 0x11,
            funct_code: Some(0x32),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::c_eq_d),
            args: &[ArgumentType::Fs, ArgumentType::Ft],
            alt_args: Some(&[&[ArgumentType::Immediate, ArgumentType::Fs, ArgumentType::Ft]]),
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "div.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x03),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::div_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "mov.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x06),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::mov_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
    ]
});
