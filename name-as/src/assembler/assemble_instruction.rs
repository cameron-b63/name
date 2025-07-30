use crate::assembler::assembly_helpers::arg_configuration_is_ok;
use name_core::instruction::formats::bit_field_type::BitFieldArgs;
use name_core::instruction::formats::cache_type::CacheArgs;
use name_core::instruction::formats::cop_mov_r_type::CopMovRArgs;
use name_core::instruction::formats::fp_cc_branch_type::FpCCBranchArgs;
use name_core::instruction::formats::fp_cc_type::FpCCArgs;
use name_core::instruction::formats::fp_four_reg_type::FpFourRegArgs;
use name_core::instruction::formats::fp_r_type::FpRArgs;
use name_core::instruction::formats::i_type::IArgs;
use name_core::instruction::formats::j_type::JArgs;
use name_core::instruction::formats::r_type::RArgs;
use name_core::instruction::formats::regimm_i_type::RegImmIArgs;
use name_core::instruction::information::{InstructionInformation, InstructionType};
use name_core::instruction::{AssembleResult, ErrorKind, RawInstruction};
use name_core::parse::parse::AstKind;

pub fn assemble_instruction(
    info: &InstructionInformation,
    arguments: Vec<AstKind>,
) -> AssembleResult<RawInstruction> {
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

    // Dispatch on instruction types
    match info.basis {
        InstructionType::BitFieldType(basis) => {
            let mut bit_field = BitFieldArgs::assign_bit_field_arguments(arguments, config)?;
            bit_field.opcode = basis.opcode;
            bit_field.funct = basis.funct;
            Ok(RawInstruction::from(bit_field))
        }
        InstructionType::CacheType(basis) => {
            let mut cache = CacheArgs::assign_cache_type_arguments(arguments, config)?;
            cache.opcode = basis.opcode;
            Ok(RawInstruction::from(cache))
        }
        InstructionType::CopMovRType(basis) => {
            let mut cop_mov = CopMovRArgs::assign_cop_mov_arguments(arguments, config)?;
            cop_mov.opcode = basis.opcode;
            cop_mov.funct_code = basis.funct_code;
            Ok(RawInstruction::from(cop_mov))
        }
        InstructionType::FpBranchType(basis) => {
            let mut fp_branch = FpCCBranchArgs::assign_fp_cc_branch_arguments(arguments, config)?;
            fp_branch.opcode = basis.opcode;
            fp_branch.funky_funct = basis.funky_funct;
            fp_branch.nd = basis.nd;
            fp_branch.tf = basis.tf;
            Ok(RawInstruction::from(fp_branch))
        }
        InstructionType::FpCCType(basis) => {
            let mut fp_cc = FpCCArgs::assign_fp_cc_arguments(arguments, config)?;
            fp_cc.opcode = basis.opcode;
            fp_cc.fmt = basis.fmt;
            fp_cc.funct = basis.funct;
            Ok(RawInstruction::from(fp_cc))
        }
        InstructionType::FpFourRegister(basis) => {
            let mut fp_four = FpFourRegArgs::assign_fp_four_reg_arguments(arguments, config)?;
            fp_four.opcode = basis.opcode;
            fp_four.op4 = basis.op4;
            fp_four.fmt3 = basis.fmt3;
            Ok(RawInstruction::from(fp_four))
        }
        InstructionType::FpRType(basis) => {
            let mut fp_r = FpRArgs::assign_fp_r_arguments(arguments, config)?;
            fp_r.opcode = basis.opcode;
            fp_r.fmt = basis.fmt;
            fp_r.funct = basis.funct;
            Ok(RawInstruction::from(fp_r))
        }
        InstructionType::IType(basis) => {
            let mut i_args = IArgs::assign_i_type_arguments(arguments, config)?;
            i_args.opcode = basis.opcode;
            Ok(RawInstruction::from(i_args))
        }
        InstructionType::JType(basis) => {
            let mut j_args = JArgs::assign_j_type_arguments(arguments, config)?;
            j_args.opcode = basis.opcode;
            Ok(RawInstruction::from(j_args))
        }
        InstructionType::RType(basis) => {
            let mut r_args = RArgs::assign_r_type_arguments(arguments, config)?;
            r_args.funct = basis.funct;
            r_args.opcode = basis.opcode;
            Ok(RawInstruction::from(r_args))
        }
        InstructionType::RegImmIType(basis) => {
            let mut regimm = RegImmIArgs::assign_regimm_i_arguments(arguments, config)?;
            regimm.opcode = basis.opcode;
            regimm.regimm_funct_code = basis.regimm_funct_code;
            Ok(RawInstruction::from(regimm))
        }
    }
}
