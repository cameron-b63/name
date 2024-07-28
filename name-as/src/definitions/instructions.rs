use crate::definitions::structs::{ArgumentType, InstructionInformation, InstructionType};

pub(crate) const INSTRUCTION_SET: &[InstructionInformation] = &[
    InstructionInformation {
        mnemonic: "add",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(32),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "addi",
        instruction_type: InstructionType::IType,
        opcode: Some(8),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "addiu",
        instruction_type: InstructionType::IType,
        opcode: Some(9),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },    
    InstructionInformation {
        mnemonic: "addu",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(33),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "and",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(36),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "andi",
        instruction_type: InstructionType::IType,
        opcode: Some(12),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "beq",
        instruction_type: InstructionType::IType,
        opcode: Some(4),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::Rt, ArgumentType::BranchLabel],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "bgtz",
        instruction_type: InstructionType::IType,
        opcode: Some(7),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::BranchLabel],
        alt_args: None,  
    },
    InstructionInformation {
        mnemonic: "blez",
        instruction_type: InstructionType::IType,
        opcode: Some(6),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::BranchLabel],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "bne",
        instruction_type: InstructionType::IType,
        opcode: Some(5),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::Rt, ArgumentType::BranchLabel],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "j",
        instruction_type: InstructionType::JType,
        opcode: Some(2),
        funct: None,
        args: &[ArgumentType::BranchLabel],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "jal",
        instruction_type: InstructionType::JType,
        opcode: Some(3),
        funct: None,
        args: &[ArgumentType::BranchLabel],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "jalr",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(9),
        args: &[ArgumentType::Rs],
        alt_args: Some(&[
            &[ArgumentType::Rd, ArgumentType::Rs],
        ]),
    },
    InstructionInformation {
        mnemonic: "jr",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(8),
        args: &[ArgumentType::Rs],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "lb",
        instruction_type: InstructionType::IType,
        opcode: Some(32),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Immediate, ArgumentType::Rs],
        alt_args: Some(&[
            &[ArgumentType::Rt, ArgumentType::Rs],
            &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
        ]),
    },
    InstructionInformation {
        mnemonic: "lui",
        instruction_type: InstructionType::IType,
        opcode: Some(15),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "lw",
        instruction_type: InstructionType::IType,
        opcode: Some(35),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::Immediate, ArgumentType::Rt],
        alt_args: Some(&[
            &[ArgumentType::Rt, ArgumentType::Rs],
            &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
        ]),
    },
    InstructionInformation {
        mnemonic: "mfhi",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(16),
        args: &[ArgumentType::Rd],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "mflo",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(18),
        args: &[ArgumentType::Rd],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "nor",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(39),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "nop",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(0),
        args: &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "or",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(37),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "ori",
        instruction_type: InstructionType::IType,
        opcode: Some(13),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "sb",
        instruction_type: InstructionType::IType,
        opcode: Some(40),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::Immediate, ArgumentType::Rt],
        alt_args: Some(&[
            &[ArgumentType::Rt, ArgumentType::Rs],
            &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
        ]),
    },
    InstructionInformation {
        mnemonic: "sll",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(0),
        args: &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "slt",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(42),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "slti",
        instruction_type: InstructionType::IType,
        opcode: Some(10),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },    
    InstructionInformation {
        mnemonic: "sltiu",
        instruction_type: InstructionType::IType,
        opcode: Some(11),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "sltu",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(43),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "srl",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(2),
        args: &[ArgumentType::Rd, ArgumentType::Rt, ArgumentType::Immediate],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "sub",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(34),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "subu",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(35),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "sw",
        instruction_type: InstructionType::IType,
        opcode: Some(43),
        funct: None,
        args: &[ArgumentType::Rs, ArgumentType::Immediate, ArgumentType::Rt],
        alt_args: Some(&[
            &[ArgumentType::Rt, ArgumentType::Rs],
            &[ArgumentType::Rt, ArgumentType::Identifier, ArgumentType::Rs],
        ]),
    },
    InstructionInformation {
        mnemonic: "syscall",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(12),
        args: &[],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "xor",
        instruction_type: InstructionType::RType,
        opcode: None,
        funct: Some(38),
        args: &[ArgumentType::Rd, ArgumentType::Rs, ArgumentType::Rt],
        alt_args: None,
    },
    InstructionInformation {
        mnemonic: "xori",
        instruction_type: InstructionType::IType,
        opcode: Some(14),
        funct: None,
        args: &[ArgumentType::Rt, ArgumentType::Rs, ArgumentType::Immediate],
        alt_args: None,
    },
];