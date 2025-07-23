use std::sync::LazyLock;

use crate::{
    elf_def::RelocationEntryType,
    instruction::{
        fp_implementations,
        information::{wrap_imp, ArgumentType, FpFmt, InstructionType},
    },
};

use super::information::FpInstructionInformation;

#[macro_export]
macro_rules! defcomp {
    ($mnemonic: expr, $implementation: expr, $multiplexer: expr, $format: expr) => {
        FpInstructionInformation {
            mnemonic: $mnemonic, // Use passed mnemonic
            instruction_type: InstructionType::FpCCType,    // Same for all comparisons
            op_code: 0x11,  // Coprocessor 1 default
            funct_code: Some(0b110000+$multiplexer),    // The multiplexer is appended to 0x3
            fmt: Some($format), // The format depends on what's given
            additional_code: None,  // No additional code for comparisons
            implementation: wrap_imp($implementation),  // does not assume implementation is provided in fp_implementations
            // below fields same for all comparisons
            args: &[ArgumentType::Fs, ArgumentType::Ft],
            alt_args: Some(&[&[ArgumentType::Immediate, ArgumentType::Fs, ArgumentType::Ft]]),
            relocation_type: None,
        }
    };
}

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
        // What follows is a whole, whole lot of comparison instruction definitions.
        // The workaround for parsing was to define an FpInstructionInformation
        // For every type of comparison in every format.
        // There are a lot of those!
        // To minimize typing, I wrote a macro at the top of the file.
        // These will be in order of their multiplexers (instead of alphabetical).
        //
        // Signaling predicates will raise an Invalid Operation exception
        // if at least one operand is NaN (even QNaN).
        //
        // All this information came from MIPS Volume II-A pages 91-92.
        //
        // Non-signaling comparisons:
        // 0b0000; FALSE predicate
        defcomp!("c.f.d", fp_implementations::c_f_d, 0b0000, FpFmt::Double),
        defcomp!("c.f.s", fp_implementations::c_f_s, 0b0000, FpFmt::Single),
        // 0b0001; UNORDERED predicate
        defcomp!("c.un.d", fp_implementations::c_un_d, 0b0001, FpFmt::Double),
        defcomp!("c.un.s", fp_implementations::c_un_s, 0b0001, FpFmt::Single),
        // 0b0010; EQUAL predicate
        defcomp!("c.eq.d", fp_implementations::c_eq_d, 0b0010, FpFmt::Double),
        defcomp!("c.eq.s", fp_implementations::c_eq_s, 0b0010, FpFmt::Single),
        // 0b0011; UNORDERED OR EQUAL predicate
        defcomp!(
            "c.ueq.d",
            fp_implementations::c_ueq_d,
            0b0011,
            FpFmt::Double
        ),
        defcomp!(
            "c.ueq.s",
            fp_implementations::c_ueq_s,
            0b0011,
            FpFmt::Single
        ),
        // 0b0100; ORDERED OR LESS THAN predicate
        defcomp!(
            "c.olt.d",
            fp_implementations::c_olt_d,
            0b0100,
            FpFmt::Double
        ),
        defcomp!(
            "c.olt.s",
            fp_implementations::c_olt_s,
            0b0100,
            FpFmt::Single
        ),
        // 0b0101; UNORDERED OR LESS THAN predicate
        defcomp!(
            "c.ult.d",
            fp_implementations::c_ult_d,
            0b0101,
            FpFmt::Double
        ),
        defcomp!(
            "c.ult.s",
            fp_implementations::c_ult_s,
            0b0101,
            FpFmt::Single
        ),
        // 0b0110; ORDERED OR LESS THAN OR EQUAL TO predicate
        defcomp!(
            "c.ole.d",
            fp_implementations::c_ole_d,
            0b0110,
            FpFmt::Double
        ),
        defcomp!(
            "c.ole.s",
            fp_implementations::c_ole_s,
            0b0110,
            FpFmt::Single
        ),
        // 0b0111; UNORDERED OR LESS THAN OR EQUAL TO predicate
        defcomp!(
            "c.ule.d",
            fp_implementations::c_ule_d,
            0b0111,
            FpFmt::Double
        ),
        defcomp!(
            "c.ule.s",
            fp_implementations::c_ule_s,
            0b0111,
            FpFmt::Single
        ),
        // Signaling comparisons:
        // 0b1000; SIGNALING FALSE predicate
        defcomp!("c.sf.d", fp_implementations::c_sf_d, 0b1000, FpFmt::Double),
        defcomp!("c.sf.s", fp_implementations::c_sf_s, 0b1000, FpFmt::Single),
        // 0b1001; NOT GREATER THAN OR LESS THAN OR EQUAL TO predicate (note that this is essentially SIGNALING UNORDERED)
        defcomp!(
            "c.ngle.d",
            fp_implementations::c_ngle_d,
            0b1001,
            FpFmt::Double
        ),
        defcomp!(
            "c.ngle.s",
            fp_implementations::c_ngle_s,
            0b1001,
            FpFmt::Single
        ),
        // 0b1010; SIGNALING EQUAL predicate
        defcomp!(
            "c.seq.d",
            fp_implementations::c_seq_d,
            0b1010,
            FpFmt::Double
        ),
        defcomp!(
            "c.seq.s",
            fp_implementations::c_seq_s,
            0b1010,
            FpFmt::Single
        ),
        // 0b1011; NOT GREATER THAN OR LESS THAN predicate
        defcomp!(
            "c.ngl.d",
            fp_implementations::c_ngl_d,
            0b1011,
            FpFmt::Double
        ),
        defcomp!(
            "c.ngl.s",
            fp_implementations::c_ngl_s,
            0b1011,
            FpFmt::Single
        ),
        // 0b1100; LESS THAN predicate
        defcomp!("c.lt.d", fp_implementations::c_lt_d, 0b1100, FpFmt::Double),
        defcomp!("c.lt.s", fp_implementations::c_lt_s, 0b1100, FpFmt::Single),
        // 0b1101; NOT GREATER THAN OR EQUAL predicate
        defcomp!(
            "c.nge.d",
            fp_implementations::c_nge_d,
            0b1101,
            FpFmt::Double
        ),
        defcomp!(
            "c.nge.s",
            fp_implementations::c_nge_s,
            0b1101,
            FpFmt::Single
        ),
        // 0b1110; LESS THAN OR EQUAL predicate
        defcomp!("c.le.d", fp_implementations::c_le_d, 0b1110, FpFmt::Double),
        defcomp!("c.le.s", fp_implementations::c_le_s, 0b1110, FpFmt::Single),
        // 0b1111; NOT GREATER THAN predicate
        defcomp!(
            "c.ngt.d",
            fp_implementations::c_ngt_d,
            0b1111,
            FpFmt::Double
        ),
        defcomp!(
            "c.ngt.s",
            fp_implementations::c_ngt_s,
            0b1111,
            FpFmt::Single
        ),
        // End of comparison definitions.
        //
        FpInstructionInformation {
            mnemonic: "ceil.l.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0a),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::ceil_l_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "ceil.l.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0a),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::ceil_l_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "ceil.w.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x09),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::ceil_w_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "ceil.w.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x09),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::ceil_w_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "cfc1",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeCF),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::cfc1),
            args: &[ArgumentType::Rt, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "ctc1",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: None,
            fmt: Some(FpFmt::ReservedFunctCodeCT),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::ctc1),
            args: &[ArgumentType::Rt, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        // Conversion from format to format.
        // Formats, for reference:
        // - Double
        // - Long
        // - Single
        // - Word
        //
        // DOUBLE
        FpInstructionInformation {
            mnemonic: "cvt.d.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x21),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::cvt_d_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        // SINGLE
        FpInstructionInformation {
            mnemonic: "cvt.s.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x20),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::cvt_s_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
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
            mnemonic: "floor.l.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0b),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::floor_l_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "floor.l.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0b),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::floor_l_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "floor.w.d",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0f),
            fmt: Some(FpFmt::Double),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::floor_w_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "floor.w.s",
            instruction_type: InstructionType::FpRType,
            op_code: 0x11,
            funct_code: Some(0x0f),
            fmt: Some(FpFmt::Single),
            additional_code: None,
            implementation: wrap_imp(fp_implementations::floor_w_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        FpInstructionInformation {
            mnemonic: "madd.d",
            instruction_type: InstructionType::FpFourRegister,
            op_code: 0x13,
            funct_code: Some(0b100_000+(FpFmt::Double).to_fmt3()),    // | op4 | fmt |
            fmt: None,
            additional_code: None,
            implementation: wrap_imp(fp_implementations::madd_d),
            args: &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
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
