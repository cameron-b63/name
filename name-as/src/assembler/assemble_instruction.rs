use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use name_core::instruction::information::InstructionType;
use name_core::instruction::{
    AssembleResult, ErrorKind, FpRArgs, IArgs, InstructionMeta, JArgs, RArgs, RawInstruction,
};
use name_core::parse::parse::AstKind;

pub fn assemble_instruction(
    meta: &InstructionMeta,
    arguments: Vec<AstKind>,
) -> AssembleResult<RawInstruction> {
    match meta {
        InstructionMeta::Int(info) => {
            // Determine which arg‐config fits
            let config = if arg_configuration_is_ok(&arguments, info.args) {
                info.args
            } else {
                info.alt_args
                    .and_then(|alts| {
                        alts.iter()
                            .find(|alt| arg_configuration_is_ok(&arguments, alt))
                    })
                    .ok_or(ErrorKind::BadArguments)?
            };

            // Dispatch on integer instruction types
            match info.instruction_type {
                InstructionType::RType => {
                    let mut r_args = RArgs::assign_r_type_arguments(arguments, config)?;
                    r_args.funct = info.funct_code.ok_or(ErrorKind::MissingFunct)? as u32;
                    r_args.opcode = info.op_code;
                    Ok(RawInstruction::from(r_args))
                }
                InstructionType::IType => {
                    let mut i_args = IArgs::assign_i_type_arguments(arguments, config)?;
                    i_args.opcode = info.op_code;
                    Ok(RawInstruction::from(i_args))
                }
                InstructionType::JType => {
                    let mut j_args = JArgs::assign_j_type_arguments(arguments, config)?;
                    j_args.opcode = info.op_code;
                    Ok(RawInstruction::from(j_args))
                }
                _ => Err(ErrorKind::WrongInstructionType)?,
            }
        }

        InstructionMeta::Fp(info) => {
            // Determine which arg‐config fits
            let config = if arg_configuration_is_ok(&arguments, info.args) {
                info.args
            } else {
                info.alt_args
                    .and_then(|alts| {
                        alts.iter()
                            .find(|alt| arg_configuration_is_ok(&arguments, alt))
                    })
                    .ok_or(ErrorKind::BadArguments)?
            };

            // Dispatch on FP instruction types
            match info.instruction_type {
                InstructionType::FpRType => {
                    let mut fp_r = FpRArgs::assign_fp_r_arguments(arguments, config)?;
                    fp_r.opcode = info.op_code;
                    fp_r.fmt = u32::from(info.fmt.ok_or(ErrorKind::MissingFmt)?);
                    fp_r.funct = info.funct_code.ok_or(ErrorKind::MissingFunct)?;
                    Ok(RawInstruction::from(fp_r))
                }
                _ => Err(ErrorKind::WrongInstructionType)?,
            }
        }
    }
}
