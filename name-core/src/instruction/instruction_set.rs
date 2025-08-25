use crate::{
    elf_def::RelocationEntryType,
    instruction::{
        formats::{
            bit_field_type::BitFieldArgs, cache_type::CacheArgs, cond_mov_cc_type::CondMovCCArgs, cop_mov_r_type::CopMovRArgs, fp_cc_branch_type::FpCCBranchArgs, fp_cc_type::FpCCArgs, fp_four_reg_type::FpFourRegArgs, fp_r_type::FpRArgs, i_type::IArgs, j_type::JArgs, r_type::RArgs, regimm_i_type::RegImmIArgs
        },
        implementation,
        information::{wrap_imp, ArgumentType, FpFmt, InstructionInformation, InstructionType},
    },
};

use std::sync::LazyLock;

/// This is the entire implemented instruction set (regular CPU instructions) for NAME.
/// The assembler searches through this table using the mnemonic field.
/// The emulator performs a lookup based on instruction basis, and then uses the associated implementation.
/// The implementation below is based on the following [specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00086-2B-MIPS32BIS-AFP-6.06.pdf).

/// CONSTANTS:

/// Used for instructions like `j label`
pub const BRANCH_LABEL_ARGS: &'static [&'static [ArgumentType]] = &[
    &[ArgumentType::Identifier],
    &[ArgumentType::Immediate, ArgumentType::Identifier],
    &[ArgumentType::Immediate, ArgumentType::Immediate],
    &[ArgumentType::Immediate],
];

/// Used for instructions like `beq reg, reg, label`
pub const SIMPLE_COMPARISON_ARGS: &'static [&'static [ArgumentType]] = &[
    &[ArgumentType::Rs, ArgumentType::Rt, ArgumentType::Immediate],
    &[ArgumentType::Rs, ArgumentType::Rt, ArgumentType::Identifier],
];

/// Used for instructions like `lb reg, address`
pub const DIRECT_MEMORY_ARGS: &'static [&'static [ArgumentType]] = &[
    &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
    &[ArgumentType::Rt, ArgumentType::Rs],
    &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
];

/*


  __  __          _____ _____   ____   _____
 |  \/  |   /\   / ____|  __ \ / __ \ / ____|
 | \  / |  /  \ | |    | |__) | |  | | (___
 | |\/| | / /\ \| |    |  _  /| |  | |\___ \
 | |  | |/ ____ \ |____| | \ \| |__| |____) |
 |_|  |_/_/    \_\_____|_|  \_\\____/|_____/




*/

/// This macro exists to make the definitions of all floating-point comparisons a bit more automated.
/// It saved me some typing.
///
/// A note about condition codes for comparisons:
/// there exist three bits of specificity for condition codes, for 8 possible condition codes.
/// When making a comparison, the implied condition code to store the result in is 0.
/// However, you may reference whichever condition code you like.
/// For paired-single operations, the condition code must be even.
/// For all condition code usage, it must be 0..=7; This is because there are only 3 bits.
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
            implementation: wrap_imp($implementation),  // does not assume implementation is provided in implementation
            // below fields same for all comparisons
            args: &[
                &[ArgumentType::Fs, ArgumentType::Ft],
                &[ArgumentType::Immediate, ArgumentType::Fs, ArgumentType::Ft],
                ],
            relocation_type: None,
        }
    };
}

/*


  _____ _   _  _____ _______ _____  _    _  _____ _______ _____ ____  _   _  _____
 |_   _| \ | |/ ____|__   __|  __ \| |  | |/ ____|__   __|_   _/ __ \| \ | |/ ____|
   | | |  \| | (___    | |  | |__) | |  | | |       | |    | || |  | |  \| | (___
   | | | . ` |\___ \   | |  |  _  /| |  | | |       | |    | || |  | | . ` |\___ \
  _| |_| |\  |____) |  | |  | | \ \| |__| | |____   | |   _| || |__| | |\  |____) |
 |_____|_| \_|_____/   |_|  |_|  \_\\____/ \_____|  |_|  |_____\____/|_| \_|_____/




*/

// The definition for this struct is very descriptive - I encourage you to go read it.
pub static INSTRUCTION_SET: LazyLock<Vec<InstructionInformation>> = LazyLock::new(|| {
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
            implementation: wrap_imp(implementation::abs_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::abs_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "add",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::add),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
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
            implementation: wrap_imp(implementation::add_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft]],
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
            implementation: wrap_imp(implementation::add_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addi",
            basis: InstructionType::IType(IArgs {
                opcode: 0x08,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::addi),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addiu",
            basis: InstructionType::IType(IArgs {
                opcode: 0x09,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::addiu),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x21,
            }),
            implementation: wrap_imp(implementation::addu),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        // ALNV.PS does not apply to our FPU.
        InstructionInformation {
            mnemonic: "and",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x24,
            }),
            implementation: wrap_imp(implementation::and),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "andi",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0c,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::andi),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
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
            implementation: wrap_imp(implementation::bc1),
            args: BRANCH_LABEL_ARGS,
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
            implementation: wrap_imp(implementation::bc1),
            args: BRANCH_LABEL_ARGS,
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
            implementation: wrap_imp(implementation::bc1),
            args: BRANCH_LABEL_ARGS,
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
            implementation: wrap_imp(implementation::bc1),
            args: BRANCH_LABEL_ARGS,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "beq",
            basis: InstructionType::IType(IArgs {
                opcode: 0x04,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::beq),
            args: SIMPLE_COMPARISON_ARGS,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "beql",
            basis: InstructionType::IType(IArgs {
                opcode: 0x14,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::beql),
            args: SIMPLE_COMPARISON_ARGS,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgez",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x01,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgez),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgezal",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x11,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgezal),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgezall",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x13,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgezall),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgezl",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x03,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgezl),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgtz",
            basis: InstructionType::IType(IArgs {
                opcode: 0x07,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgtz),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgtzl",
            basis: InstructionType::IType(IArgs {
                opcode: 0x17,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bgtzl),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "blez",
            basis: InstructionType::IType(IArgs {
                opcode: 0x06,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::blez),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "blezl",
            basis: InstructionType::IType(IArgs {
                opcode: 0x16,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::blezl),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bltz",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x00,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bltz),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bltzal",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x10,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bltzal),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bltzall",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x12,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bltzall),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bltzl",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x02,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bltzl),
            args: &[&[ArgumentType::Rs, ArgumentType::Identifier]],
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bne",
            basis: InstructionType::IType(IArgs {
                opcode: 0x05,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bne),
            args: SIMPLE_COMPARISON_ARGS,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bnel",
            basis: InstructionType::IType(IArgs {
                opcode: 0x15,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::bnel),
            args: SIMPLE_COMPARISON_ARGS,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "break",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0d,
            }),
            implementation: wrap_imp(implementation::break_instr),
            args: &[&[]],
            relocation_type: None,
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
        defcomp!("c.f.d", implementation::c_f_d, 0b0000, FpFmt::Double),
        defcomp!("c.f.s", implementation::c_f_s, 0b0000, FpFmt::Single),
        // 0b0001; UNORDERED predicate
        defcomp!("c.un.d", implementation::c_un_d, 0b0001, FpFmt::Double),
        defcomp!("c.un.s", implementation::c_un_s, 0b0001, FpFmt::Single),
        // 0b0010; EQUAL predicate
        defcomp!("c.eq.d", implementation::c_eq_d, 0b0010, FpFmt::Double),
        defcomp!("c.eq.s", implementation::c_eq_s, 0b0010, FpFmt::Single),
        // 0b0011; UNORDERED OR EQUAL predicate
        defcomp!("c.ueq.d", implementation::c_ueq_d, 0b0011, FpFmt::Double),
        defcomp!("c.ueq.s", implementation::c_ueq_s, 0b0011, FpFmt::Single),
        // 0b0100; ORDERED OR LESS THAN predicate
        defcomp!("c.olt.d", implementation::c_olt_d, 0b0100, FpFmt::Double),
        defcomp!("c.olt.s", implementation::c_olt_s, 0b0100, FpFmt::Single),
        // 0b0101; UNORDERED OR LESS THAN predicate
        defcomp!("c.ult.d", implementation::c_ult_d, 0b0101, FpFmt::Double),
        defcomp!("c.ult.s", implementation::c_ult_s, 0b0101, FpFmt::Single),
        // 0b0110; ORDERED OR LESS THAN OR EQUAL TO predicate
        defcomp!("c.ole.d", implementation::c_ole_d, 0b0110, FpFmt::Double),
        defcomp!("c.ole.s", implementation::c_ole_s, 0b0110, FpFmt::Single),
        // 0b0111; UNORDERED OR LESS THAN OR EQUAL TO predicate
        defcomp!("c.ule.d", implementation::c_ule_d, 0b0111, FpFmt::Double),
        defcomp!("c.ule.s", implementation::c_ule_s, 0b0111, FpFmt::Single),
        // Signaling comparisons:
        // 0b1000; SIGNALING FALSE predicate
        defcomp!("c.sf.d", implementation::c_sf_d, 0b1000, FpFmt::Double),
        defcomp!("c.sf.s", implementation::c_sf_s, 0b1000, FpFmt::Single),
        // 0b1001; NOT GREATER THAN OR LESS THAN OR EQUAL TO predicate
        // (note that this is essentially SIGNALING UNORDERED)
        defcomp!("c.ngle.d", implementation::c_ngle_d, 0b1001, FpFmt::Double),
        defcomp!("c.ngle.s", implementation::c_ngle_s, 0b1001, FpFmt::Single),
        // 0b1010; SIGNALING EQUAL predicate
        defcomp!("c.seq.d", implementation::c_seq_d, 0b1010, FpFmt::Double),
        defcomp!("c.seq.s", implementation::c_seq_s, 0b1010, FpFmt::Single),
        // 0b1011; NOT GREATER THAN OR LESS THAN predicate
        defcomp!("c.ngl.d", implementation::c_ngl_d, 0b1011, FpFmt::Double),
        defcomp!("c.ngl.s", implementation::c_ngl_s, 0b1011, FpFmt::Single),
        // 0b1100; LESS THAN predicate
        defcomp!("c.lt.d", implementation::c_lt_d, 0b1100, FpFmt::Double),
        defcomp!("c.lt.s", implementation::c_lt_s, 0b1100, FpFmt::Single),
        // 0b1101; NOT GREATER THAN OR EQUAL predicate
        defcomp!("c.nge.d", implementation::c_nge_d, 0b1101, FpFmt::Double),
        defcomp!("c.nge.s", implementation::c_nge_s, 0b1101, FpFmt::Single),
        // 0b1110; LESS THAN OR EQUAL predicate
        defcomp!("c.le.d", implementation::c_le_d, 0b1110, FpFmt::Double),
        defcomp!("c.le.s", implementation::c_le_s, 0b1110, FpFmt::Single),
        // 0b1111; NOT GREATER THAN predicate
        defcomp!("c.ngt.d", implementation::c_ngt_d, 0b1111, FpFmt::Double),
        defcomp!("c.ngt.s", implementation::c_ngt_s, 0b1111, FpFmt::Single),
        // End of comparison definitions.
        InstructionInformation {
            mnemonic: "cache",
            basis: InstructionType::CacheType(CacheArgs {
                opcode: 0x2f,
                base: 0,
                op: 0,
                offset: 0,
            }),
            implementation: wrap_imp(implementation::cache),
            args: &[&[
                ArgumentType::Immediate,
                ArgumentType::Immediate,
                ArgumentType::Rs,
            ]],
            relocation_type: None,
        },
        // I'm going to purposefully ignore CACHEE since we don't have an acutal priviliged resource setup.
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
            implementation: wrap_imp(implementation::ceil_l_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::ceil_l_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::ceil_w_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::ceil_w_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::cfc1),
            args: &[&[ArgumentType::Rt, ArgumentType::Fs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "clo",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x21,
            }),
            implementation: wrap_imp(implementation::clo),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "clz",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::clz),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs]],
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
            implementation: wrap_imp(implementation::ctc1),
            args: &[&[ArgumentType::Rt, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::cvt_d_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::cvt_s_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
            relocation_type: None,
        },
        // I'm ignoring DERET as it doesn't apply to use yet.
        // I'm ignoring the DI (disable interrupts) instruction
        // since I don't even know where to really start.
        // I know for sure that it doesn't really apply to us.
        InstructionInformation {
            mnemonic: "div",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x1a,
            }),
            implementation: wrap_imp(implementation::div),
            args: &[&[ArgumentType::Rs, ArgumentType::Rt]],
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
            implementation: wrap_imp(implementation::div_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "divu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x1b,
            }),
            implementation: wrap_imp(implementation::divu),
            args: &[&[ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        // Similarly ignoring EI (enable interrupts).
        // See DI.
        // Ignoring ERET and ERETNC
        InstructionInformation {
            mnemonic: "ext",
            basis: InstructionType::BitFieldType(BitFieldArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                msbd: 0,
                lsb: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(implementation::ext),
            args: &[&[
                ArgumentType::Rt,
                ArgumentType::Rs,
                ArgumentType::Immediate,
                ArgumentType::Immediate,
            ]],
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
            implementation: wrap_imp(implementation::floor_l_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::floor_l_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::floor_w_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
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
            implementation: wrap_imp(implementation::floor_w_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ins",
            basis: InstructionType::BitFieldType(BitFieldArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                msbd: 0,
                lsb: 0,
                funct: 0x04,
            }),
            implementation: wrap_imp(implementation::ins),
            args: &[&[
                ArgumentType::Rt,
                ArgumentType::Rs,
                ArgumentType::Immediate,
                ArgumentType::Immediate,
            ]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "j",
            basis: InstructionType::JType(JArgs {
                opcode: 0x02,
                address: 0,
            }),
            implementation: wrap_imp(implementation::j),
            args: BRANCH_LABEL_ARGS,
            relocation_type: Some(RelocationEntryType::R26),
        },
        InstructionInformation {
            mnemonic: "jal",
            basis: InstructionType::JType(JArgs {
                opcode: 0x03,
                address: 0,
            }),
            implementation: wrap_imp(implementation::jal),
            args: BRANCH_LABEL_ARGS,
            relocation_type: Some(RelocationEntryType::R26),
        },
        InstructionInformation {
            mnemonic: "jalr",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(implementation::jalr),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs], &[ArgumentType::Rs]],
            relocation_type: None,
        },
        // I'm going to purposefully ignore *.HB because
        // that's some hardware business I won't get to for a long time.
        // JALX also doesn't apply since we're not including micro MIPS.
        InstructionInformation {
            mnemonic: "jr",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x08,
            }),
            implementation: wrap_imp(implementation::jr),
            args: &[&[ArgumentType::Rs]],
            relocation_type: None,
        },
        // JR.HB is ignored, as stated above.
        InstructionInformation {
            mnemonic: "lb",
            basis: InstructionType::IType(IArgs {
                opcode: 0x20,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lb),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // LBE (load byte EVA) doesn't apply to us since we don't have priv. arch.
        InstructionInformation {
            mnemonic: "lbu",
            basis: InstructionType::IType(IArgs {
                opcode: 0x24,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lbu),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // LBUE (load byte unsigned EVA) doesn't apply.
        InstructionInformation {
            mnemonic: "ldc1",
            basis: InstructionType::IType(IArgs {
                opcode: 0x35,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::ldc1),
            args: &[
                &[ArgumentType::Ft, ArgumentType::Immediate, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier],
                &[ArgumentType::Ft, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ldxc1",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::ldxc1),
            args: &[&[ArgumentType::Fd, ArgumentType::Immediate, ArgumentType::Rs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lh",
            basis: InstructionType::IType(IArgs {
                opcode: 0x21,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lh),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // lhe (load half EVA) ignored
        InstructionInformation {
            mnemonic: "lhu",
            basis: InstructionType::IType(IArgs {
                opcode: 0x25,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lhu),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // lhue (load half unsigned EVA) ignored
        InstructionInformation {
            mnemonic: "ll",
            basis: InstructionType::IType(IArgs {
                opcode: 0x30,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::ll),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // lle (load linked EVA) ignored
        InstructionInformation {
            mnemonic: "lui",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0f,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lui),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Immediate],
                &[ArgumentType::Rt, ArgumentType::Identifier],
            ],
            relocation_type: Some(RelocationEntryType::Hi16),
        },
        InstructionInformation {
            mnemonic: "luxc1",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x05,
            }),
            implementation: wrap_imp(implementation::luxc1),
            args: &[&[ArgumentType::Fd, ArgumentType::Immediate, ArgumentType::Rs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lw",
            basis: InstructionType::IType(IArgs {
                opcode: 0x23,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lw),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lwc1",
            basis: InstructionType::IType(IArgs {
                opcode: 0x31,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lwc1),
            args: &[
                &[ArgumentType::Ft, ArgumentType::Immediate, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier],
                &[ArgumentType::Ft, ArgumentType::Immediate],
            ],
            relocation_type: Some(RelocationEntryType::Lo16),
        },
        // lwe (load word EVA) ignored
        InstructionInformation {
            mnemonic: "lwl",
            basis: InstructionType::IType(IArgs {
                opcode: 0x22,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lwl),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // lwle (load word left EVA) ignored
        InstructionInformation {
            mnemonic: "lwr",
            basis: InstructionType::IType(IArgs {
                opcode: 0x26,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::lwr),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // lwre (load word right EVA) ignored
        InstructionInformation {
            mnemonic: "lwxc1",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(implementation::lwxc1),
            args: &[&[ArgumentType::Fd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "madd",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(implementation::madd),
            args: &[&[ArgumentType::Rs, ArgumentType::Rt]],
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
            implementation: wrap_imp(implementation::madd_d),
            args: &[&[
                ArgumentType::Fd,
                ArgumentType::Fr,
                ArgumentType::Fs,
                ArgumentType::Ft,
            ]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "madd.s",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                fr: 0,
                ft: 0,
                fs: 0,
                fd: 0,
                op4: 0x04,
                fmt3: FpFmt::Single.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::madd_s),
            args: &[&[
                ArgumentType::Fd,
                ArgumentType::Fr,
                ArgumentType::Fs,
                ArgumentType::Ft,
            ]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "maddu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::maddu),
            args: &[&[ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mfc0",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x10,
                funct_code: 0x00,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(implementation::mfc0),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Rd],
                &[ArgumentType::Rt, ArgumentType::Rd, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mfc1",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x11,
                funct_code: 0x00,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(implementation::mfc1),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        // mfhc0 does not apply to our setup (XPA disabled).
        // we don't have a 64-bit cp0.
        // same story for mfhc1.
        // we don't have a 64-bit FPU.
        InstructionInformation {
            mnemonic: "mfhi",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x10,
            }),
            implementation: wrap_imp(implementation::mfhi),
            args: &[
                &[ArgumentType::Rd],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mflo",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x12,
            }),
            implementation: wrap_imp(implementation::mfhi),
            args: &[
                &[ArgumentType::Rd],
            ],
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
            implementation: wrap_imp(implementation::mov_d),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mov.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x06,
            }),
            implementation: wrap_imp(implementation::mov_s),
            args: &[&[ArgumentType::Fd, ArgumentType::Fs]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movf",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x00,
                rs: 0,
                cc: 0,
                tf: 0x00,
                rd: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::mov_conditional),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movf.d",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x11,
                rs: FpFmt::Double as u32,
                cc: 0,
                tf: 0x01,
                rd: 0,
                funct: 0x11,
            }),
            implementation: wrap_imp(implementation::mov_conditional_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movf.s",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x11,
                rs: FpFmt::Single as u32,
                cc: 0,
                tf: 0x01,
                rd: 0,
                funct: 0x11,
            }),
            implementation: wrap_imp(implementation::mov_conditional_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movn",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0b,
            }),
            implementation: wrap_imp(implementation::movn),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movn.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x13
            }),
            implementation: wrap_imp(implementation::movn_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movn.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x13
            }),
            implementation: wrap_imp(implementation::movn_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movt",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x00,
                rs: 0,
                cc: 0,
                tf: 0x01,
                rd: 0,
                funct: 0x01
            }),
            implementation: wrap_imp(implementation::mov_conditional),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movt.d",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x11,
                rs: FpFmt::Double as u32,
                cc: 0,
                tf: 0x01,
                rd: 0,
                funct: 0x11,
            }),
            implementation: wrap_imp(implementation::mov_conditional_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movt.s",
            basis: InstructionType::CondMovCCType(CondMovCCArgs {
                opcode: 0x11,
                rs: FpFmt::Single as u32,
                cc: 0,
                tf: 0x01,
                rd: 0,
                funct: 0x11,
            }),
            implementation: wrap_imp(implementation::mov_conditional_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movz",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0a,
            }),
            implementation: wrap_imp(implementation::movz),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movz.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x12,
            }),
            implementation: wrap_imp(implementation::movz_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "movz.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x12,
            }),
            implementation: wrap_imp(implementation::movz_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "msub",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x04,
            }),
            implementation: wrap_imp(implementation::msub),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "msub.d",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                fr: 0,
                ft: 0,
                fs: 0,
                fd: 0,
                op4: 0x05,
                fmt3: FpFmt::Double.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::msub_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "msub.s",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                fr: 0,
                ft: 0,
                fs: 0,
                fd: 0,
                op4: 0x05,
                fmt3: FpFmt::Single.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::msub_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "msubu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x05,
            }),
            implementation: wrap_imp(implementation::msubu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mtc0",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x10,
                funct_code: 0x04,
                rt: 0,
                rd: 0,
                sel: 0, 
            }), 
            implementation: wrap_imp(implementation::mtc0),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Rd],
                &[ArgumentType::Rt, ArgumentType::Rd, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mtc1",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x11,
                funct_code: 0x04,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(implementation::mtc1),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Fs,],
            ],
            relocation_type: None,
        },
        // ignoring mtc2 since we didn't implement cop2
        // ignoring mthc0 because we aren't allowing XPA
        // et cetera
        InstructionInformation {
            mnemonic: "mthi",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x11,
            }),
            implementation: wrap_imp(implementation::mthi),
            args: &[
                &[ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mtlo",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x13,
            }),
            implementation: wrap_imp(implementation::mtlo),
            args: &[
                &[ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mul",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::mul),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mul.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::mul_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mul.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::mul_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "mult",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x18,
            }),
            implementation: wrap_imp(implementation::mult),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt]
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "multu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x19,
            }),
            implementation: wrap_imp(implementation::multu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt]
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "neg.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x07,
            }),
            implementation: wrap_imp(implementation::neg_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "neg.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x07,
            }),
            implementation: wrap_imp(implementation::neg_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nmadd.d",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                ft: 0,
                fr: 0,
                fs: 0,
                fd: 0,
                op4: 0x06,
                fmt3: FpFmt::Double.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::nmadd_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nmadd.s",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                ft: 0,
                fr: 0,
                fs: 0,
                fd: 0,
                op4: 0x06,
                fmt3: FpFmt::Single.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::nmadd_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nmsub.d",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                ft: 0,
                fr: 0,
                fs: 0,
                fd: 0,
                op4: 0x07,
                fmt3: FpFmt::Double.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::nmsub_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nmsub.s",
            basis: InstructionType::FpFourRegister(FpFourRegArgs {
                opcode: 0x13,
                ft: 0,
                fr: 0,
                fs: 0,
                fd: 0,
                op4: 0x07,
                fmt3: FpFmt::Single.to_fmt3(),
            }),
            implementation: wrap_imp(implementation::nmsub_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fr, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nor",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x27,
            }),

            implementation: wrap_imp(implementation::nor),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "or",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x25,
            }),
            implementation: wrap_imp(implementation::or),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ori",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0d,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::ori),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
                &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Identifier],
            ],
            relocation_type: Some(RelocationEntryType::Lo16),
        },
        // skipping paired single business because 32-bit FPU
        InstructionInformation {
            mnemonic: "pref",
            basis: InstructionType::IType(IArgs {
                opcode: 0x33,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::pref),
            args: &[
                &[ArgumentType::Immediate, ArgumentType::Immediate, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        // ignoring prefe because I am not planning on EVA arch
        InstructionInformation {
            mnemonic: "prefx",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0f,
            }),
            implementation: wrap_imp(implementation::prefx),
            args: &[
                &[ArgumentType::Immediate, ArgumentType::Rt, ArgumentType::Rs]
            ],
            relocation_type: None,
        },
        // ignoring more PS instructions (paired single not supported)
        InstructionInformation {
            mnemonic: "rdhwr",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x3b,
            }),
            implementation: wrap_imp(implementation::rdhwr),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Rd]
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "rdpgpr",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x10,
                funct_code: 0x0a,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(implementation::rdpgpr),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "recip.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x15,
            }),
            implementation: wrap_imp(implementation::recip_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "recip.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x15,
            }),
            implementation: wrap_imp(implementation::recip_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "rotr",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::srl),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "rotrv",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x06,
            }),
            implementation: wrap_imp(implementation::srlv),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "round.l.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x08,
            }),
            implementation: wrap_imp(implementation::round_l_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "round.l.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x08,
            }),
            implementation: wrap_imp(implementation::round_l_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
                InstructionInformation {
            mnemonic: "round.w.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0c,
            }),
            implementation: wrap_imp(implementation::round_w_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "round.w.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0c,
            }),
            implementation: wrap_imp(implementation::round_w_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "rsqrt.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x16,
            }),
            implementation: wrap_imp(implementation::rsqrt_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "rsqrt.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x16,
            }),
            implementation: wrap_imp(implementation::rsqrt_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sb",
            basis: InstructionType::IType(IArgs {
                opcode: 0x28,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sb),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // ignoring sbe due to no EVA architecture implemented
        InstructionInformation {
            mnemonic: "sc",
            basis: InstructionType::IType(IArgs {
                opcode: 0x38,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sc),
            args: &[
                &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        // sce ignored due to no EVA implementation
        InstructionInformation {
            mnemonic: "sdbbp",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1c,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x3f,
            }),
            implementation: wrap_imp(implementation::sdbbp),
            args: &[
                &[ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sdc1",
            basis: InstructionType::IType(IArgs {
                opcode: 0x3d,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sdc1),
            args: &[
                &[ArgumentType::Ft, ArgumentType::Immediate, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier, ArgumentType::Rs],
                &[ArgumentType::Ft, ArgumentType::Identifier],
                &[ArgumentType::Ft, ArgumentType::Immediate],
            ],
            relocation_type: Some(RelocationEntryType::Lo16),
        },
        InstructionInformation {
            mnemonic: "sdxc1",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(implementation::sdxc1),
            args: &[
                &[ArgumentType::Fs, ArgumentType::Immediate, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "seb",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::bshfl),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "seh",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::bshfl),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sh",
            basis: InstructionType::IType(IArgs {
                opcode: 0x19,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sh),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        // sh eva ignored because i am not doing EVA
        InstructionInformation {
            mnemonic: "sll",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x00,
            }),
            implementation: wrap_imp(implementation::sll),
            args: &[&[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sllv",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x04,
            }),
            implementation: wrap_imp(implementation::sllv),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "slt",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x2a,
            }),
            implementation: wrap_imp(implementation::slt),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "slti",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0a,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::slti),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sltiu",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0b,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sltiu),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sltu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x2b,
            }),
            implementation: wrap_imp(implementation::sltu),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sqrt.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x04,
            }),
            implementation: wrap_imp(implementation::sqrt_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sqrt.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x04,
            }),
            implementation: wrap_imp(implementation::sqrt_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sra",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x03,
            }),
            implementation: wrap_imp(implementation::sra),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "srav",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x07,
            }),
            implementation: wrap_imp(implementation::srav),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "srl",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::srl),
            args: &[&[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "srlv",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x06,
            }),
            implementation: wrap_imp(implementation::srlv),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sub",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x22,
            }),
            implementation: wrap_imp(implementation::sub),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sub.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::sub_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sub.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::sub_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs, ArgumentType::Ft],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "subu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x23,
            }),
            implementation: wrap_imp(implementation::subu),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        // SUXC1 doesn't apply to our setup with a 32-bit FPU.
        InstructionInformation {
            mnemonic: "sw",
            basis: InstructionType::IType(IArgs {
                opcode: 0x2b,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::sw),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "swc1",
            basis: InstructionType::IType(IArgs {
                opcode: 0x39,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::swc1),
            args: &[
                &[ArgumentType::Ft, ArgumentType::Immediate, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "swl",
            basis: InstructionType::IType(IArgs {
                opcode: 0x1a,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::swl),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "swr",
            basis: InstructionType::IType(IArgs {
                opcode: 0x1e,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::swr),
            args: DIRECT_MEMORY_ARGS,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "swxc1",
            basis: InstructionType::RType(RArgs {
                opcode: 0x13,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x08,
            }),
            implementation: wrap_imp(implementation::swxc1),
            args: &[
                &[ArgumentType::Fs, ArgumentType::Rt, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sync",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0f,
            }),
            implementation: wrap_imp(implementation::sync),
            args: &[
                &[],
                &[ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "synci",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x1f,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::synci),
            args: &[
                &[ArgumentType::Immediate, ArgumentType::Rs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "syscall",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x0c,
            }),
            implementation: wrap_imp(implementation::syscall),
            args: &[&[]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "teq",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x34,
            }),
            implementation: wrap_imp(implementation::teq),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "teqi",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x0c,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::teqi),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tge",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x30,
            }),
            implementation: wrap_imp(implementation::tge),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tgei",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x08,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::tgei),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tgeiu",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x09,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::tgeiu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tgeu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x31,
            }),
            implementation: wrap_imp(implementation::tgeu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        // not implementing TLBINV or TLBINVF
        InstructionInformation {
            mnemonic: "tlbp",
            basis: InstructionType::RType(RArgs {
                opcode: 0x10,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x08,
            }),
            implementation: wrap_imp(implementation::tlbp),
            args: &[
                &[],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tlbr",
            basis: InstructionType::RType(RArgs {
                opcode: 0x10,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x01,
            }),
            implementation: wrap_imp(implementation::tlbr),
            args: &[
                &[],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tlbwi",
            basis: InstructionType::RType(RArgs {
                opcode: 0x10,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x02,
            }),
            implementation: wrap_imp(implementation::tlbwi),
            args: &[
                &[],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tlbwi",
            basis: InstructionType::RType(RArgs {
                opcode: 0x10,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x06,
            }),
            implementation: wrap_imp(implementation::tlbwr),
            args: &[
                &[],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tlt",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x32,
            }),
            implementation: wrap_imp(implementation::tlt),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tlti",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x0a,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::tlti),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tltiu",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x0b,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::tltiu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tltu",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x33,
            }),
            implementation: wrap_imp(implementation::tltu),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tne",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x36,
            }),
            implementation: wrap_imp(implementation::tne),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "tnei",
            basis: InstructionType::RegImmIType(RegImmIArgs {
                opcode: 0x01,
                rs: 0,
                regimm_funct_code: 0x0e,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::tnei),
            args: &[
                &[ArgumentType::Rs, ArgumentType::Immediate],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "trunc.l.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(implementation::trunc_l_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "trunc.l.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x09,
            }),
            implementation: wrap_imp(implementation::trunc_l_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "trunc.w.d",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Double as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0d,
            }),
            implementation: wrap_imp(implementation::trunc_w_d),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "trunc.w.s",
            basis: InstructionType::FpRType(FpRArgs {
                opcode: 0x11,
                fmt: FpFmt::Single as u32,
                ft: 0,
                fs: 0,
                fd: 0,
                funct: 0x0d,
            }),
            implementation: wrap_imp(implementation::trunc_w_s),
            args: &[
                &[ArgumentType::Fd, ArgumentType::Fs],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "wait",
            basis: InstructionType::RType(RArgs {
                opcode: 0x10,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::wait),
            args: &[
                &[]
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "wrpgpr",
            basis: InstructionType::CopMovRType(CopMovRArgs {
                opcode: 0x10,
                funct_code: 0x0e,
                rt: 0,
                rd: 0,
                sel: 0,
            }),
            implementation: wrap_imp(implementation::wrpgpr),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "wsbh",
            basis: InstructionType::RType(RArgs {
                opcode: 0x1f,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x20,
            }),
            implementation: wrap_imp(implementation::bshfl),
            args: &[
                &[ArgumentType::Rd, ArgumentType::Rt],
            ],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "xor",
            basis: InstructionType::RType(RArgs {
                opcode: 0x00,
                rs: 0,
                rt: 0,
                rd: 0,
                shamt: 0,
                funct: 0x26,
            }),
            implementation: wrap_imp(implementation::xor),
            args: &[&[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt]],
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "xori",
            basis: InstructionType::IType(IArgs {
                opcode: 0x0e,
                rs: 0,
                rt: 0,
                imm: 0,
            }),
            implementation: wrap_imp(implementation::xori),
            args: &[&[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate]],
            relocation_type: None,
        },
    ]
});
