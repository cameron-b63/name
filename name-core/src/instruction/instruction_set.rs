use crate::{elf_def::RelocationEntryType, instruction::{
    implementation,
    information::{wrap_imp, ArgumentType, InstructionInformation, InstructionType},
}};

use std::sync::LazyLock;

/// This is the entire implemented instruction set for NAME.
/// The assembler searches through this table using the mnemonic field.
/// The emulator performs a lookup based on op_code and funct_code, and then uses the associated implementation.
/// The implementation below is based on the following TIS: https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00086-2B-MIPS32BIS-AFP-6.06.pdf

// The definition for this struct is very descriptive - I encourage you to go read it.
pub static INSTRUCTION_SET: LazyLock<Vec<InstructionInformation>> = LazyLock::new(|| {
    vec![
        InstructionInformation {
            mnemonic: "add",
            op_code: 0x00,
            funct_code: Some(0x20),
            implementation: wrap_imp(implementation::add),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addi",
            op_code: 0x08,
            funct_code: None,
            implementation: wrap_imp(implementation::addi),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addiu",
            op_code: 0x09,
            funct_code: None,
            implementation: wrap_imp(implementation::addiu),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "addu",
            op_code: 0x00,
            funct_code: Some(0x21),
            implementation: wrap_imp(implementation::addu),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "and",
            op_code: 0x00,
            funct_code: Some(0x24),
            implementation: wrap_imp(implementation::and),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "andi",
            op_code: 0x0c,
            funct_code: None,
            implementation: wrap_imp(implementation::andi),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "beq",
            op_code: 0x04,
            funct_code: None,
            implementation: wrap_imp(implementation::beq),
            instruction_type: InstructionType::IType,
            args: &[
                ArgumentType::Rs,
                ArgumentType::Rt,
                ArgumentType::BranchLabel,
            ],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bgtz",
            op_code: 0x07,
            funct_code: None,
            implementation: wrap_imp(implementation::bgtz),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rs, ArgumentType::BranchLabel],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "blez",
            op_code: 0x06,
            funct_code: None,
            implementation: wrap_imp(implementation::blez),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rs, ArgumentType::BranchLabel],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "bne",
            op_code: 0x05,
            funct_code: None,
            implementation: wrap_imp(implementation::bne),
            instruction_type: InstructionType::IType,
            args: &[
                ArgumentType::Rs,
                ArgumentType::Rt,
                ArgumentType::BranchLabel,
            ],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::Pc16),
        },
        InstructionInformation {
            mnemonic: "j",
            op_code: 0x02,
            funct_code: None,
            implementation: wrap_imp(implementation::j),
            instruction_type: InstructionType::JType,
            args: &[ArgumentType::BranchLabel],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::R26),
        },
        InstructionInformation {
            mnemonic: "jal",
            op_code: 0x03,
            funct_code: None,
            implementation: wrap_imp(implementation::jal),
            instruction_type: InstructionType::JType,
            args: &[ArgumentType::BranchLabel],
            alt_args: None,
            relocation_type: Some(RelocationEntryType::R26),
        },
        InstructionInformation {
            mnemonic: "jalr",
            op_code: 0x00,
            funct_code: Some(0x09),
            implementation: wrap_imp(implementation::jalr),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs],
            alt_args: Some(&[&[ArgumentType::Rs]]),
            relocation_type: None,

        },
        InstructionInformation {
            mnemonic: "jr",
            op_code: 0x00,
            funct_code: Some(0x08),
            implementation: wrap_imp(implementation::jr),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rs],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lb",
            op_code: 0x20,
            funct_code: None,
            implementation: wrap_imp(implementation::lb),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
            alt_args: Some(&[
                &[ArgumentType::Rt, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
            ]),
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lui",
            op_code: 0x0F,
            funct_code: None,
            implementation: wrap_imp(implementation::lui),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "lw",
            op_code: 0x23,
            funct_code: None,
            implementation: wrap_imp(implementation::lw),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
            alt_args: Some(&[
                &[ArgumentType::Rt, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
            ]),
            relocation_type: None,
        },
        /*
          Instruction::InstructionInformation {
            mnemonic: "mfhi",
            op_code: 0x00,
            instruction_type: InstructionType::RType,
            opcode: None,
            funct_code: Some(16),
            args: &[ArgumentType::Rd],
            alt_args: None,
        },
          Instruction::InstructionInformation {
            mnemonic: "mflo",
            op_code: 0x00,
            instruction_type: InstructionType::RType,
            opcode: None,
            funct_code: Some(18),
            args: &[ArgumentType::Rd],
            alt_args: None,
        },
        */
        InstructionInformation {
            mnemonic: "nor",
            op_code: 0x00,
            funct_code: Some(0x27),
            implementation: wrap_imp(implementation::nor),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "nop",
            op_code: 0x00,
            funct_code: Some(0x00),
            implementation: wrap_imp(implementation::sll),
            instruction_type: InstructionType::RType,
            args: &[],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "or",
            op_code: 0x00,
            funct_code: Some(0x25),
            implementation: wrap_imp(implementation::or),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "ori",
            op_code: 0x0D,
            funct_code: None,
            implementation: wrap_imp(implementation::ori),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sb",
            op_code: 0x28,
            funct_code: None,
            implementation: wrap_imp(implementation::sb),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
            alt_args: Some(&[
                &[ArgumentType::Rt, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier],
            ]),
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sll",
            op_code: 0x00,
            funct_code: Some(0x00),
            implementation: wrap_imp(implementation::sll),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "slt",
            op_code: 0x00,
            funct_code: Some(0x2A),
            implementation: wrap_imp(implementation::slt),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "slti",
            op_code: 0x0A,
            funct_code: None,
            implementation: wrap_imp(implementation::slti),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sltiu",
            op_code: 0x0B,
            funct_code: None,
            implementation: wrap_imp(implementation::sltiu),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sltu",
            op_code: 0x00,
            funct_code: Some(0x2B),
            implementation: wrap_imp(implementation::sltu),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "srl",
            op_code: 0x00,
            funct_code: Some(0x02),
            implementation: wrap_imp(implementation::srl),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sub",
            op_code: 0x00,
            funct_code: Some(0x22),
            implementation: wrap_imp(implementation::sub),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "subu",
            op_code: 0x00,
            funct_code: Some(0x23),
            implementation: wrap_imp(implementation::subu),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "sw",
            op_code: 0x2B,
            funct_code: None,
            implementation: wrap_imp(implementation::sw),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
            alt_args: Some(&[
                &[ArgumentType::Rt, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
                &[ArgumentType::Rt, ArgumentType::Identifier],
            ]),
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "syscall",
            op_code: 0x00,
            funct_code: Some(0x0C),
            implementation: wrap_imp(implementation::syscall),
            instruction_type: InstructionType::RType,
            args: &[],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "xor",
            op_code: 0x00,
            funct_code: Some(0x26),
            implementation: wrap_imp(implementation::xor),
            instruction_type: InstructionType::RType,
            args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
            alt_args: None,
            relocation_type: None,
        },
        InstructionInformation {
            mnemonic: "xori",
            op_code: 0x0E,
            funct_code: None,
            implementation: wrap_imp(implementation::xori),
            instruction_type: InstructionType::IType,
            args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
            alt_args: None,
            relocation_type: None,
        },
    ]
});
