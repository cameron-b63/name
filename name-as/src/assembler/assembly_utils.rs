use crate::assembler::assembler::{AssembleResult, ErrorKind};
use crate::definitions::constants::{MAX_U16, MIN_U16};
use name_core::{instruction::information::ArgumentType, parse::parse::AstKind, structs::Register};

/*

  _____     _________     _______  ______
 |  __ \   |__   __\ \   / /  __ \|  ____|
 | |__) |_____| |   \ \_/ /| |__) | |__
 |  _  /______| |    \   / |  ___/|  __|
 | | \ \      | |     | |  | |    | |____
 |_|  \_\     |_|     |_|  |_|    |______|



*/
pub fn assemble_r_type(
    rd: Register,
    rs: Register,
    rt: Register,
    shamt: u32,
    funct: u32,
) -> AssembleResult<u32> {
    // I'm using these unwrap_or statements to ensure that when packing R-type instructions that don't use all 3, the fields default to 0 in the packed word.
    // The '?' operators are to ensure the proper error message is propagated up through to the assembler's 'errors' vec.

    // Check shamt for range
    if shamt > 31 {
        return Err(ErrorKind::InvalidShamt);
    }

    // The opcode for all R-type instructions is 0.
    let opcode: u32 = 0;

    return Ok((opcode << 26)
        | ((rs as u32) << 21)
        | ((rt as u32) << 16)
        | ((rd as u32) << 11)
        | ((shamt as u32) << 6)
        | funct);
}

// I understand this function header can be... hairy. The added context of usage in the assemble_instruction function makes this far easier to parse.
pub fn assign_r_type_arguments(
    arguments: Vec<AstKind>,
    args_to_use: &[ArgumentType],
) -> AssembleResult<(Register, Register, Register, u32)> {
    let mut rd = Register::Zero;
    let mut rs = Register::Zero;
    let mut rt = Register::Zero;
    let mut shamt = 0;

    for (i, passed) in arguments.into_iter().enumerate() {
        match args_to_use[i] {
            ArgumentType::Rd => rd = passed.get_register().ok_or(ErrorKind::InvalidArgument)?,
            ArgumentType::Rs => rs = passed.get_register().ok_or(ErrorKind::InvalidArgument)?,
            ArgumentType::Rt => rt = passed.get_register().ok_or(ErrorKind::InvalidArgument)?,
            ArgumentType::Immediate => {
                shamt = passed.get_immediate().ok_or(ErrorKind::InvalidArgument)?;
            }
            _ => unreachable!(),
        }
    }

    return Ok((rd, rs, rt, shamt));
}

/*

  _____   _________     _______  ______
 |_   _| |__   __\ \   / /  __ \|  ____|
   | |______| |   \ \_/ /| |__) | |__
   | |______| |    \   / |  ___/|  __|
  _| |_     | |     | |  | |    | |____
 |_____|    |_|     |_|  |_|    |______|



*/

pub fn assemble_i_type(
    opcode: u32,
    rs: Register,
    rt: Register,
    immediate: u32,
) -> AssembleResult<u32> {
    // Check range on immediate value
    if (immediate as i32) > MAX_U16 || (immediate as i32) < MIN_U16 {
        return Err(ErrorKind::ImmediateOverflow);
    }

    let parsed_immediate: u32 = immediate as u16 as u32;

    Ok((opcode << 26) | ((rs as u32) << 21) | ((rt as u32) << 16) | (parsed_immediate))
}

pub fn assign_i_type_arguments(
    arguments: Vec<AstKind>,
    args_to_use: &[ArgumentType],
) -> AssembleResult<(Register, Register, u32)> {
    let mut rs = Register::Zero;
    let mut rt = Register::Zero;
    let mut imm = 0;

    for (i, passed) in arguments.into_iter().enumerate() {
        match args_to_use[i] {
            ArgumentType::Rs => rs = passed.get_register().unwrap_or(rs),
            ArgumentType::Rt => rt = passed.get_register().unwrap_or(rt),
            ArgumentType::Immediate => imm = passed.get_immediate().unwrap_or(imm),
            ArgumentType::Identifier | ArgumentType::BranchLabel => (),
            _ => unreachable!(),
        }
    }

    return Ok((rs, rt, imm));
}

/*

       _     _________     _______  ______
      | |   |__   __\ \   / /  __ \|  ____|
      | |______| |   \ \_/ /| |__) | |__
  _   | |______| |    \   / |  ___/|  __|
 | |__| |      | |     | |  | |    | |____
  \____/       |_|     |_|  |_|    |______|



*/

/// "Assemble" a j-type instruction. Since the immediate won't be known until relocation, only have to shift opcode.
pub fn assemble_j_type(opcode: u32) -> u32 {
    return opcode << 26;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_i_type_test() {
        let opcode: u32 = 13;
        let rt: Option<String> = Some("$t0".to_string());
        let rs: Option<String> = Some("$t2".to_string());
        let immediate: Option<i32> = Some(0xBEEF);

        let assembled_output = assemble_i_type(opcode, rs, rt, immediate);
        assert_eq!(assembled_output, Ok(0x3548BEEF));
    }

    #[test]
    fn assemble_j_type_test() {
        let opcode: u32 = 2;

        let assembled_output = assemble_j_type(opcode);
        assert_eq!(assembled_output, 0x08000000);
    }

    #[test]
    fn assemble_r_type_test() {
        let rd = Some("$t0".to_string());
        let rs = Some("$t1".to_string());
        let rt = Some("$t2".to_string());
        let shamt = Some(0);
        let assembled_output = assemble_r_type(rd, rs, rt, shamt, 32);
        assert_eq!(assembled_output, Ok(0x012A4020));

        let assembled_err = assemble_r_type(Some("bad register".to_string()), None, None, None, 32);
        assert!(assembled_err.is_err());

        let rd = Some("$t0".to_string());
        let rs = Some("$t1".to_string());
        let shamt = Some(32);
        let assembled_shamt_err = assemble_r_type(rd, rs, None, shamt, 32);
        assert!(assembled_shamt_err.is_err());
    }
}
