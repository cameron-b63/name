use crate::exception::definitions::ExceptionType;
use crate::instruction::{IArgs, JArgs, RArgs};
use crate::structs::{
    ProgramState,
    Register::{At, Ra},
};

/*

  ______ _    _ _   _  _____ _______
 |  ____| |  | | \ | |/ ____|__   __|
 | |__  | |  | |  \| | |       | |
 |  __| | |  | | . ` | |       | |
 | |    | |__| | |\  | |____   | |
 |_|     \____/|_| \_|\_____|  |_|



*/

// 0x00 - sll
pub fn sll(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rt as usize] << args.shamt;
}

// 0x02 - srl
pub fn srl(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rt as usize] >> args.shamt;
}

// 0x08 - jr
pub fn jr(program_state: &mut ProgramState, args: RArgs) -> () {
    if !program_state
        .memory
        .allows_execution_of(program_state.cpu.general_purpose_registers[args.rs as usize])
    {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
    }

    program_state.cpu.pc = program_state.cpu.general_purpose_registers[args.rs as usize];
}

// 0x09 - jalr
pub fn jalr(program_state: &mut ProgramState, args: RArgs) -> () {
    let rd = match args.rd {
        0 => 31,
        x => x,
    };

    if !program_state
        .memory
        .allows_execution_of(program_state.cpu.general_purpose_registers[args.rs as usize])
    {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
    }
    program_state.cpu.general_purpose_registers[rd as usize] = program_state.cpu.pc;
    program_state.cpu.pc = program_state.cpu.general_purpose_registers[args.rs as usize];
}

// 0x0A - slti
pub fn slti(program_state: &mut ProgramState, args: IArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32) < (args.imm as i32) {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 1 as u32;
    } else {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 0 as u32;
    }
}

// 0x0B - sltiu
pub fn sltiu(program_state: &mut ProgramState, args: IArgs) -> () {
    if program_state.cpu.general_purpose_registers[args.rs as usize] < (args.imm as u32) {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 1 as u32;
    } else {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 0 as u32;
    }
}

// 0x0C - syscall
pub fn syscall(program_state: &mut ProgramState, _args: RArgs) -> () {
    program_state.set_exception(ExceptionType::Syscall);
}

// 0x0D - break
pub fn break_instr(program_state: &mut ProgramState, _args: RArgs) -> () {
    program_state.set_exception(ExceptionType::Breakpoint);
}

// 0x20 - add
pub fn add(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            + program_state.cpu.general_purpose_registers[args.rt as usize];

    // println!("Adding ${}({}) to ${}({}) and storing in ${}, now it's {}", rs, program_state.cpu.general_purpose_registers[args.rs as usize], rt, program_state.cpu.general_purpose_registers[args.rt as usize], rd, program_state.cpu.general_purpose_registers[args.rs as usize] + program_state.cpu.general_purpose_registers[args.rt as usize]);
}

// 0x21 - addu
pub fn addu(program_state: &mut ProgramState, args: RArgs) -> () {
    // check that below works
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            .overflowing_add(program_state.cpu.general_purpose_registers[args.rt as usize])
            .0;
}

// 0x22 - sub
pub fn sub(program_state: &mut ProgramState, args: RArgs) -> () {
    let temp: (u32, bool) = program_state.cpu.general_purpose_registers[args.rs as usize]
        .overflowing_sub(program_state.cpu.general_purpose_registers[args.rt as usize]);

    program_state.cpu.general_purpose_registers[At as usize] = temp.0;

    if temp.1 {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::ArithmeticOverflow);
    } else {
        program_state.cpu.general_purpose_registers[args.rd as usize] =
            program_state.cpu.general_purpose_registers[At as usize];
    }
}

// 0x23 - subu
pub fn subu(program_state: &mut ProgramState, args: RArgs) -> () {
    let temp: (u32, bool) = program_state.cpu.general_purpose_registers[args.rs as usize]
        .overflowing_sub(program_state.cpu.general_purpose_registers[args.rt as usize]);

    program_state.cpu.general_purpose_registers[args.rd as usize] = temp.0;
}

// 0x24 - and
pub fn and(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            & program_state.cpu.general_purpose_registers[args.rt as usize];
}

// 0x25 - or
pub fn or(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            | program_state.cpu.general_purpose_registers[args.rt as usize];
}

// 0x26 - xor
pub fn xor(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            ^ program_state.cpu.general_purpose_registers[args.rt as usize];
}

// 0x27 - nor
pub fn nor(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        !(program_state.cpu.general_purpose_registers[args.rs as usize]
            | program_state.cpu.general_purpose_registers[args.rt as usize]);
}

// 0x2A - slt
pub fn slt(program_state: &mut ProgramState, args: RArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32)
        < (program_state.cpu.general_purpose_registers[args.rt as usize] as i32)
    {
        program_state.cpu.general_purpose_registers[args.rd as usize] = 1 as u32;
    } else {
        program_state.cpu.general_purpose_registers[args.rd as usize] = 0 as u32;
    }
}

// 0x2A - sltu
pub fn sltu(program_state: &mut ProgramState, args: RArgs) -> () {
    if program_state.cpu.general_purpose_registers[args.rs as usize]
        < program_state.cpu.general_purpose_registers[args.rt as usize]
    {
        program_state.cpu.general_purpose_registers[args.rd as usize] = 1; // check if this is kosher or if i need to do 00..001 for some reason
    } else {
        program_state.cpu.general_purpose_registers[args.rd as usize] = 0;
    }
}

/*

   ____  _____   _____ ____  _____  ______
  / __ \|  __ \ / ____/ __ \|  __ \|  ____|
 | |  | | |__) | |   | |  | | |  | | |__
 | |  | |  ___/| |   | |  | | |  | |  __|
 | |__| | |    | |___| |__| | |__| | |____
  \____/|_|     \_____\____/|_____/|______|



*/

// 0x02 - j
pub fn j(program_state: &mut ProgramState, args: JArgs) -> () {
    let address: u32 = (args.address << 2) | (program_state.cpu.pc & 0xF0000000);

    if !program_state.memory.allows_execution_of(address) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    program_state.cpu.pc = address;
}

// 0x03 - jal
pub fn jal(program_state: &mut ProgramState, args: JArgs) -> () {
    let address: u32 = (args.address << 2) | (program_state.cpu.pc & 0xF0000000);

    if !program_state.memory.allows_execution_of(address) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    program_state.cpu.general_purpose_registers[Ra as usize] = program_state.cpu.pc;
    program_state.cpu.pc = address;
}

// 0x04 - beq
pub fn beq(program_state: &mut ProgramState, args: IArgs) -> () {
    // Sign extend offset
    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    if program_state.cpu.general_purpose_registers[args.rs as usize]
        != program_state.cpu.general_purpose_registers[args.rt as usize]
    {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    if !program_state.memory.allows_execution_of(temp) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // Bro forgot the actual jump logic
    program_state.cpu.pc = temp;
}

// 0x05 - bne
pub fn bne(program_state: &mut ProgramState, args: IArgs) -> () {
    // Sign extend offset
    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    if program_state.cpu.general_purpose_registers[args.rs as usize]
        == program_state.cpu.general_purpose_registers[args.rt as usize]
    {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    if !program_state.memory.allows_execution_of(temp) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // Bro once again forgot the actual jump logic
    program_state.cpu.pc = temp;
}

// 0x06 - blez
pub fn blez(program_state: &mut ProgramState, args: IArgs) -> () {
    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32) > 0 {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    if !program_state.memory.allows_execution_of(temp) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // BRO HAS ONCE AGAIN FORGOTTEN THE ACTUAL JUMP
    program_state.cpu.pc = temp;
}

// 0x07 - bgtz
pub fn bgtz(program_state: &mut ProgramState, args: IArgs) -> () {
    // Sign extend offset
    let offset: i32 = (args.imm as i16 as i32) << 2;

    if program_state.cpu.general_purpose_registers[args.rs as usize] as i32 <= 0 {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    if !program_state.memory.allows_execution_of(temp) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    program_state.cpu.pc = temp;
}

// 0x08 - addi
pub fn addi(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
            + (args.imm as i16 as i32)) as u32;
}

// 0x09 - addiu
pub fn addiu(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            .overflowing_add(args.imm as u32)
            .0;
}

// 0x0C - andi
pub fn andi(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize] & (args.imm as i16 as u32);
}

// 0x0D - ori
pub fn ori(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            | (args.imm as i16 as i32 as u32);
}

// 0x0E - xori
pub fn xori(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            ^ (args.imm as i16 as i32 as u32);
}

// 0x0F - lui
pub fn lui(program_state: &mut ProgramState, args: IArgs) -> () {
    // SUPER DUPER PROBLEM SPOT
    program_state.cpu.general_purpose_registers[args.rt as usize] = (args.imm as u32) << 16;
}

// 0x20 - lb
pub fn lb(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp: u32 = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if !program_state.memory.allows_read_from(temp) {
        // TODO: Use a function which sets the proper values in cp0 for us
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }
    let return_byte: u8 = match program_state.memory.read_byte(temp) {
        Ok(b) => b,
        Err(_) => {
            program_state.set_exception(ExceptionType::AddressExceptionLoad);
            return;
        }
    };
    program_state.cpu.general_purpose_registers[args.rt as usize] = return_byte as u32;
}

// 0x23 - lw
pub fn lw(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if temp % 4 != 0 {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    if !program_state.memory.allows_read_from(temp)
        || !program_state.memory.allows_read_from(temp + 3)
    {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // Checks passed. Load word.
    let mut i = 0;
    let mut result_word: u32 = 0;
    while i < 4 {
        match program_state.memory.read_byte(temp + i) {
            Ok(b) => result_word |= (b as u32) << (24 - (i * 8)),
            Err(_) => program_state.set_exception(ExceptionType::AddressExceptionLoad),
        }
        i += 1;
    }

    program_state.cpu.general_purpose_registers[args.rt as usize] = result_word;
}

// 0x28 - sb
pub fn sb(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if !program_state.memory.allows_write_to(temp) {
        program_state.set_exception(ExceptionType::AddressExceptionStore);
        return;
    }

    match program_state.memory.set_byte(
        temp,
        program_state.cpu.general_purpose_registers[args.rt as usize] as u8,
    ) {
        Ok(_) => (),
        Err(_) => program_state.set_exception(ExceptionType::AddressExceptionStore),
    };
}

// 0x2b - sw
pub fn sw(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if temp % 4 != 0 {
        program_state.set_exception(ExceptionType::AddressExceptionStore);
        return;
    }

    if !program_state.memory.allows_write_to(temp)
        || !program_state.memory.allows_write_to(temp + 3)
    {
        program_state.set_exception(ExceptionType::AddressExceptionStore);
        return;
    }

    // Retrieve value of rt from cpu
    let value: u32 = program_state.cpu.general_purpose_registers[args.rt as usize];

    // Checks passed. Store word.
    let mut i = 0;
    while i < 4 {
        // Shift/mask value to get correct byte
        let new_byte: u8 = ((value >> (i * 8)) & 0xFF) as u8;
        // Write it to correct location
        match program_state.memory.set_byte(temp + (3 - i), new_byte) {
            Ok(_) => (),
            Err(_) => {
                // If write failed, trigger an exception
                program_state.set_exception(ExceptionType::AddressExceptionStore);
                return;
            }
        }
        i += 1;
    }
}

// 0x31 - lwc1
pub fn lwc1(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("Implement lwc1");
}