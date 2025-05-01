use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use crate::assembler::assembly_utils::*;
use name_core::instruction::information::{InstructionInformation, InstructionType};
use name_core::instruction::{AssembleResult, ErrorKind, IArgs, RawInstruction};
use name_core::parse::parse::AstKind;

// Big logic for instruction assembly - this is the main driver code for actual packing of instructions once parsed.
pub fn assemble_instruction(
    info: &InstructionInformation,
    arguments: Vec<AstKind>,
) -> AssembleResult<RawInstruction> {
    // dbg!(info);
    // dbg!(&arguments);
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
            Ok(RawInstruction::new(assemble_r_type(rd, rs, rt, shamt, funct)?))
        }
        InstructionType::IType => {
            let mut i_args = IArgs::assign_i_type_arguments(arguments, config)?;
            i_args.opcode = info.op_code as u32;

            Ok(RawInstruction::from(i_args))
        }
        InstructionType::JType => {
            let opcode: u32 = info.op_code as u32;

            // "Assemble" a j-type instruction. Since the immediate won't be known until relocation, only have to shift opcode.

            Ok(RawInstruction::new(opcode << 26))
        }
    }
}
