use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use name_core::instruction::information::{InstructionInformation, InstructionType};
use name_core::instruction::{
    AssembleResult, ErrorKind, FpRArgs, IArgs, JArgs, RArgs, RawInstruction,
};
use name_core::parse::parse::AstKind;

// Big logic for instruction assembly - this is the main driver code for actual packing of instructions once parsed.
pub fn assemble_instruction(
    info: &InstructionInformation,
    arguments: Vec<AstKind>,
) -> AssembleResult<RawInstruction> {
    // Find proper argument configuration early
    let config = if arg_configuration_is_ok(&arguments, info.args) {
        info.args
    } else {
        // Check alternate argument configurations to see if they fit
        info.alt_args
            .and_then(|args| {
                args.iter()
                    .find(|alt| arg_configuration_is_ok(&arguments, alt))
            })
            // Reachable only if no argument configurations matched
            .ok_or(ErrorKind::BadArguments)?
    };

    // Match on the instruction type to structure operands correctly
    // Pack the instructions via RawInstruction::from(_)
    match info.instruction_type {
        InstructionType::RType => {
            let mut r_args = RArgs::assign_r_type_arguments(arguments, config)?;
            r_args.funct = info.funct_code.ok_or(ErrorKind::MissingFunct)? as u32;
            r_args.opcode = info.op_code as u32;

            Ok(RawInstruction::from(r_args))
        }
        InstructionType::IType => {
            let mut i_args = IArgs::assign_i_type_arguments(arguments, config)?;
            i_args.opcode = info.op_code as u32;

            Ok(RawInstruction::from(i_args))
        }
        InstructionType::JType => {
            let opcode: u32 = info.op_code as u32;

            // "Assemble" a j-type instruction.
            // Since the immediate won't be known until relocation, only have to shift opcode.
            // Still using the same approach for consistency,
            // but this will be a set of wrappers over opcode << 26.
            let mut j_args = JArgs::assign_j_type_arguments(arguments, config)?;
            j_args.opcode = opcode;
            
            Ok(RawInstruction::from(j_args))
        }
        InstructionType::FpRType => {
            let mut fp_r_args = FpRArgs::assign_fp_r_arguments(arguments, config)?;
            fp_r_args.opcode = info.op_code as u32;
            fp_r_args.funct = info.funct_code.ok_or(ErrorKind::MissingFunct)? as u32;

            Ok(RawInstruction::from(fp_r_args))
        }
    }
}
