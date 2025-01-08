use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use crate::assembler::assembly_utils::*;
use crate::definitions::structs::LineComponent;
use name_core::instruction::information::{ArgumentType, InstructionInformation, InstructionType};
use name_core::parse::parse::AstKind;

// Big logic for instruction assembly - this is the main driver code for actual packing of instructions once parsed.
pub fn assemble_instruction(
    info: &InstructionInformation,
    arguments: Vec<AstKind>,
) -> Result<u32, String> {
    let has_alternate_configurations: bool = info.alt_args.is_some();

    // Find proper argument configuration early
    let configuration_to_use: &[ArgumentType];
    let existing_configuration_works: bool = arg_configuration_is_ok(&arguments, info.args);

    if !has_alternate_configurations {
        if !existing_configuration_works {
            return Err(format!(" - Bad arguments provided during operand checks."));
        } else {
            configuration_to_use = info.args;
        }
    } else if !existing_configuration_works {
        let found_configuration = info
            .alt_args
            .unwrap()
            .iter()
            .find(|config| arg_configuration_is_ok(&arguments, config));
        match found_configuration {
            Some(configuration) => configuration_to_use = configuration,
            None => return Err(" - Bad arguments provided; Alternative argument configurations exist, but none matched.".to_string()),
        }
    } else {
        configuration_to_use = info.args;
    }

    match info.instruction_type {
        InstructionType::RType => {
            let funct: u32 = info.funct_code.expect("Improper implmentation of instructions (funct field undefined for R-type instr)\nIf you are a student reading this, understand this error comes entirely from a fundamental failure in the codebase of this vscode extension.") as u32;

            let (rd, rs, rt, shamt) = match assign_r_type_arguments(arguments, configuration_to_use)
            {
                Ok((rd, rs, rt, shamt)) => (rd, rs, rt, shamt),
                Err(e) => return Err(e),
            };

            return assemble_r_type(rd, rs, rt, shamt, funct);
        }
        InstructionType::IType => {
            let opcode: u32 = info.op_code as u32;

            let (rs, rt, imm) = match assign_i_type_arguments(arguments, configuration_to_use) {
                Ok((rs, rt, imm)) => (rs, rt, imm),
                Err(e) => return Err(e),
            };

            return assemble_i_type(opcode, rs, rt, imm);
        }
        InstructionType::JType => {
            let opcode: u32 = info.op_code as u32;

            return Ok(assemble_j_type(opcode));
        }
    }
}
