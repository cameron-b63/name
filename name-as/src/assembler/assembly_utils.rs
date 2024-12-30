use crate::assembler::assembly_helpers::parse_register_to_u32;
use crate::definitions::constants::{MAX_U16, MIN_U16};
use crate::definitions::structs::LineComponent;
use name_core::instruction::information::ArgumentType;

/*

  _____     _________     _______  ______
 |  __ \   |__   __\ \   / /  __ \|  ____|
 | |__) |_____| |   \ \_/ /| |__) | |__
 |  _  /______| |    \   / |  ___/|  __|
 | | \ \      | |     | |  | |    | |____
 |_|  \_\     |_|     |_|  |_|    |______|



*/
pub fn assemble_r_type(
    rd: Option<String>,
    rs: Option<String>,
    rt: Option<String>,
    shamt: Option<i32>,
    funct: u32,
) -> Result<u32, String> {
    // I'm using these unwrap_or statements to ensure that when packing R-type instructions that don't use all 3, the fields default to 0 in the packed word.
    // The '?' operators are to ensure the proper error message is propagated up through to the assembler's 'errors' vec.
    let parsed_rd: u32 = parse_register_to_u32(&rd.unwrap_or("$0".to_string()))?;
    let parsed_rs: u32 = parse_register_to_u32(&rs.unwrap_or("$0".to_string()))?;
    let parsed_rt: u32 = parse_register_to_u32(&rt.unwrap_or("$0".to_string()))?;

    let unchecked_shamt: i32 = shamt.unwrap_or(0);

    // The opcode for all R-type instructions is 0.
    let opcode: u32 = 0;

    // Check shamt for range
    let parsed_shamt: u32 = unchecked_shamt as u32;
    if unchecked_shamt < 0 || unchecked_shamt > 31 {
        return Err("Shift amount out of range on shift instruction.".to_string());
    }

    return Ok((opcode << 26)
        | (parsed_rs << 21)
        | (parsed_rt << 16)
        | (parsed_rd << 11)
        | (parsed_shamt << 6)
        | funct);
}

// I understand this function header can be... hairy. The added context of usage in the assemble_instruction function makes this far easier to parse.
pub fn assign_r_type_arguments(
    arguments: &Vec<LineComponent>,
    args_to_use: &[ArgumentType],
) -> Result<(Option<String>, Option<String>, Option<String>, Option<i32>), String> {
    let mut rd: Option<String> = None;
    let mut rs: Option<String> = None;
    let mut rt: Option<String> = None;
    let mut shamt: Option<i32> = None;

    for (i, passed) in arguments.iter().enumerate() {
        let mut content = String::from("");
        let mut immediate = 0;
        match passed {
            LineComponent::Register(register) => content = register.clone(),
            LineComponent::Immediate(imm) => immediate = imm.clone(),
            _ => return Err(" - Bad argument types provided to instruction.".to_string()),
        }

        match args_to_use[i] {
            ArgumentType::Rd => rd = Some(content.clone()),
            ArgumentType::Rs => rs = Some(content.clone()),
            ArgumentType::Rt => rt = Some(content.clone()),
            ArgumentType::Immediate => shamt = Some(immediate),
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
    rs: Option<String>,
    rt: Option<String>,
    immediate: Option<i32>,
) -> Result<u32, String> {
    // Default any non-provided registers to $zero - should have no bearing.
    let parsed_rs: u32 = parse_register_to_u32(&rs.unwrap_or("$0".to_string()))?;
    let parsed_rt: u32 = parse_register_to_u32(&rt.unwrap_or("$0".to_string()))?;
    let unchecked_immediate: i32 = immediate.unwrap_or(0);

    // Check range on immediate value
    if unchecked_immediate > MAX_U16 || unchecked_immediate < MIN_U16 {
        return Err("Immediate exceeds 16 bits".to_string());
    }

    let parsed_immediate: u32 = unchecked_immediate as i16 as u16 as u32;

    Ok((opcode << 26) | (parsed_rs << 21) | (parsed_rt << 16) | (parsed_immediate))
}

pub fn assign_i_type_arguments(
    arguments: &Vec<LineComponent>,
    args_to_use: &[ArgumentType],
) -> Result<(Option<String>, Option<String>, Option<i32>), String> {
    let mut rs: Option<String> = None;
    let mut rt: Option<String> = None;
    let mut imm: Option<i32> = None;

    for (i, passed) in arguments.iter().enumerate() {
        let mut content: String = String::from("");
        let mut immediate: i32 = 0;
        match passed {
            LineComponent::Register(register) => content = register.clone(),
            LineComponent::Immediate(number) => immediate = number.clone(),
            LineComponent::Identifier(identifier) => content = identifier.clone(),
            _ => return Err(" - Bad arguments provided during i-type assignment".to_string()),
        }

        match args_to_use[i] {
            ArgumentType::Rs => rs = Some(content.clone()),
            ArgumentType::Rt => rt = Some(content.clone()),
            ArgumentType::Immediate => imm = Some(immediate.clone()),
            ArgumentType::Identifier | ArgumentType::BranchLabel => imm = None,
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
