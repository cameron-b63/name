use name_core::instruction::{AssembleResult, ErrorKind};
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