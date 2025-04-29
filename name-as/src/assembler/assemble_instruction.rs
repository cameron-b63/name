use crate::assembler::assembler::{AssembleResult, ErrorKind};
use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use crate::assembler::assembly_utils::*;
use name_core::instruction::information::{InstructionInformation, InstructionType};
use name_core::parse::parse::AstKind;

// Big logic for instruction assembly - this is the main driver code for actual packing of instructions once parsed.
pub fn assemble_instruction(
    info: &InstructionInformation,
    arguments: Vec<AstKind>,
) -> AssembleResult<u32> {
    dbg!(info);
    dbg!(&arguments);
    // Find proper argument configuration early
    let config = if arg_configuration_is_ok(&arguments, info.args) {
        info.args
    } else {
        info.alt_args
            .and_then(|args| {
                args.iter()
                    .find(|alt| arg_configuration_is_ok(&arguments, alt))
            })
            .ok_or(ErrorKind::BadArguments)?
    };

    match info.instruction_type {
        InstructionType::RType => {
            let funct: u32 = info.funct_code.expect("Improper implmentation of instructions (funct field undefined for R-type instr)\nIf you are a student reading this, understand this error comes entirely from a fundamental failure in the codebase of this vscode extension.") as u32;

            let (rd, rs, rt, shamt) = assign_r_type_arguments(arguments, config)?;
            assemble_r_type(rd, rs, rt, shamt, funct)
        }
        InstructionType::IType => {
            let opcode: u32 = info.op_code as u32;

            let (rs, rt, imm) = assign_i_type_arguments(arguments, config)?;

            assemble_i_type(opcode, rs, rt, imm)
        }
        InstructionType::JType => {
            let opcode: u32 = info.op_code as u32;

            Ok(assemble_j_type(opcode))
        }
    }
}
