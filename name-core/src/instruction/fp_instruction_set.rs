use std::sync::LazyLock;

use crate::{
    elf_def::RelocationEntryType,
    instruction::{
        formats::{
            cop_mov_r_type::CopMovRArgs, fp_cc_branch_type::FpCCBranchArgs, fp_cc_type::FpCCArgs,
            fp_four_reg_type::FpFourRegArgs, fp_r_type::FpRArgs,
        },
        fp_implementations,
        information::{wrap_imp, ArgumentType, FpFmt, InstructionInformation, InstructionType},
    },
};

/// This macro exists to make the definitions of all floating-point comparisons a bit more automated.
/// It saved me some typing.
#[macro_export]
macro_rules! defcomp {
    ($mnemonic: expr, $implementation: expr, $multiplexer: expr, $format: expr) => {
        InstructionInformation {
            mnemonic: $mnemonic, // Use passed mnemonic
            basis: InstructionType::FpCCType(FpCCArgs {
                opcode: 0x11,
                fmt: $format as u32,
                ft: 0,
                fs: 0,
                cc: 0,
                funct: 0b11_0000 + $multiplexer,
            }),
            implementation: wrap_imp($implementation),  // does not assume implementation is provided in fp_implementations
            // below fields same for all comparisons
            args: &[ArgumentType::Fs, ArgumentType::Ft],
            alt_args: Some(&[&[ArgumentType::Immediate, ArgumentType::Fs, ArgumentType::Ft]]),
            relocation_type: None,
        }
    };
}

/// A note about condition codes for comparisons:
/// there exist three bits of specificity for condition codes, for 8 possible condition codes.
/// When making a comparison, the implied condition code to store the result in is 0.
/// However, you may reference whichever condition code you like.
/// For paired-single operations, the condition code must be even.
/// For all condition code usage, it must be 0..=7; This is because there are only 3 bits.
pub static FP_INSTRUCTION_SET: LazyLock<Vec<InstructionInformation>> = LazyLock::new(|| {
    vec![
        InstructionInformation {
            mnemonic: "abs.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x05,
            }),
            implementation: wrap_imp(fp_implementations::abs_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "abs.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x05,
            }),
            implementation: wrap_imp(fp_implementations::abs_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "add.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(fp_implementations::add_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "add.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(fp_implementations::add_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        // ALNV.PS does not apply to our FPU.
        InstructionInformation {
            mnemonic: "bc1f",
            basis: InstructionType::FpBranchType(FpCCBranchArgs {
                opcode: 0x11,
                funky_funct: 0x08,
                cc: 0,
                tf: 0,
                nd: 0,
                offset: 0,
            }),
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bc1fl",
            basis: InstructionType::FpBranchType(FpCCBranchArgs {
                opcode: 0x11,
                funky_funct: 0x08,
                cc: 0,
                tf: 0,
                nd: 1,
                offset: 0,
            }),
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bc1t",
            basis: InstructionType::FpBranchType(FpCCBranchArgs {
                opcode: 0x11,
                funky_funct: 0x08,
                cc: 0,
                tf: 1,
                nd: 0,
                offset: 0,
            }),
            implementation: wrap_imp(fp_implementations::bc1),
            args: &[ArgumentType::Identifier],
            alt_args: Some(&[
                &[ArgumentType::Immediate, ArgumentType::Identifier],
                &[ArgumentType::Immediate, ArgumentType::Immediate],
                &[ArgumentType::Immediate],
            ]),
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bc1tl",
            basis: InstructionType::FpBranchType(FpCCBranchArgs {
                opcode: 0x11,
                funky_funct: 0x08,
                cc: 0,
                tf: 1,
                nd: 1,
                offset: 0,
            }),
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
        // The workaround for parsing was to define an InstructionInformation
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
        InstructionInformation {
            mnemonic: "ceil.l.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0a,
            }),
            implementation: wrap_imp(fp_implementations::ceil_l_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ceil.l.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0a,
            }),
            implementation: wrap_imp(fp_implementations::ceil_l_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ceil.w.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(fp_implementations::ceil_w_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ceil.w.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(fp_implementations::ceil_w_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "cfc1",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x11,
                funct_code: 0x02,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(fp_implementations::cfc1),
            args: &[ArgumentType::Rt, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ctc1",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x11,
                funct_code: 0x06,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
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
        InstructionInformation {
            mnemonic: "cvt.d.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x21,
            }),
            implementation: wrap_imp(fp_implementations::cvt_d_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        // SINGLE
        InstructionInformation {
            mnemonic: "cvt.s.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(fp_implementations::cvt_s_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "div.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x03,
            }),
            implementation: wrap_imp(fp_implementations::div_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "floor.l.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0b,
            }),
            implementation: wrap_imp(fp_implementations::floor_l_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "floor.l.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0b,
            }),
            implementation: wrap_imp(fp_implementations::floor_l_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "floor.w.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0f,
            }),
            implementation: wrap_imp(fp_implementations::floor_w_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "floor.w.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0f,
            }),
            implementation: wrap_imp(fp_implementations::floor_w_s),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "madd.d",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                fr: 0,
                ft: 0,
                fs: 0,
                fd: 0,
                op4: 0x04,
                fmt3: FpFmt::Double.to_fmt3(),
            }),
            implementation: wrap_imp(fp_implementations::madd_d),
            args: &[
                ArgumentType::Fd,
                ArgumentType::Fr,
                ArgumentType::Fs,
                ArgumentType::Ft,
            ],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mov.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x06,
            }),
            implementation: wrap_imp(fp_implementations::mov_d),
            args: &[ArgumentType::Fd, ArgumentType::Fs],
            alt_args: None,
            relocation_type: None,
        },
    ]
});
