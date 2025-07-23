use crate::exception::definitions::ExceptionType;
use crate::instruction::formats::cop_mov_r_type::CopMovRArgs;
use crate::instruction::formats::i_type::IArgs;
use crate::instruction::formats::j_type::JArgs;
use crate::instruction::formats::r_type::RArgs;
use crate::instruction::formats::regimm_i_type::RegImmIArgs;
use crate::structs::{
    ProgramState,
    Register::{At, Ra},
};

use super::implementation_helpers::{extract_u64, is_register_aligned, pack_up_u64};

// Special (not sure where to organize):
// EJTAG exceptions:
// 0x10/0x1f
pub fn deret(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("deret");
}

// Other MIPS cop0 instructions:
// 0x10/0x18
// eretnc is in here
pub fn eret(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("eret");
}

// 0x10/mfc0
pub fn mfc0(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("mfc0");
}

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
    program_state.jump_if_valid(program_state.cpu.general_purpose_registers[args.rs as usize])
}

// 0x09 - jalr
pub fn jalr(program_state: &mut ProgramState, args: RArgs) -> () {
    let rd = match args.rd {
        0 => 31,
        x => x,
    };

    program_state.cpu.general_purpose_registers[rd as usize] = program_state.cpu.pc;
    program_state.jump_if_valid(program_state.cpu.general_purpose_registers[args.rs as usize]);
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

// 0x1A - div
pub fn div(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("div");
}

// 0x1B - divu
pub fn divu(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("divu");
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

    program_state.jump_if_valid(address);
}

// 0x03 - jal
pub fn jal(program_state: &mut ProgramState, args: JArgs) -> () {
    let address: u32 = (args.address << 2) | (program_state.cpu.pc & 0xF0000000);

    program_state.cpu.general_purpose_registers[Ra as usize] = program_state.cpu.pc;
    program_state.jump_if_valid(address);
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

    program_state.jump_if_valid(temp);
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

    program_state.jump_if_valid(temp);
}

// 0x06 - blez
pub fn blez(program_state: &mut ProgramState, args: IArgs) -> () {
    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32) > 0 {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    program_state.jump_if_valid(temp);
}

// 0x07 - bgtz
pub fn bgtz(program_state: &mut ProgramState, args: IArgs) -> () {
    // Sign extend offset
    let offset: i32 = (args.imm as i16 as i32) << 2;

    if program_state.cpu.general_purpose_registers[args.rs as usize] as i32 <= 0 {
        return;
    }

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    program_state.jump_if_valid(temp);
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

// 0x14 - beql
pub fn beql(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("beql");
}

// 0x15 - bnel
pub fn bnel(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("bnel");
}

// 0x16 - blezl
pub fn blezl(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("blezl")
}

// 0x17 - bgtl
pub fn bgtzl(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("bgtzl")
}

// 0x20 - lb
pub fn lb(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp: u32 = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if !program_state.memory.allows_read_from(temp) {
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
    program_state.cpu.general_purpose_registers[args.rt as usize] = return_byte as i8 as i32 as u32;
    // explicit sign-extension
}

// 0x21 - lh
pub fn lh(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("lh");
}

// 0x22 - lwl
pub fn lwl(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("lwl");
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

// 0x24 - lbu
pub fn lbu(program_state: &mut ProgramState, args: IArgs) -> () {
    let temp: u32 = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if !program_state.memory.allows_read_from(temp) {
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
    program_state.cpu.general_purpose_registers[args.rt as usize] = (return_byte as u32) & 0xFF;
    // Clear any sign-extension
}

// 0x25 - lhu
pub fn lhu(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("lhu");
}

// 0x26 - lwr
pub fn lwr(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("lwr");
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

// 0x2f - cache
pub fn cache(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("cache implementation");
}

// 0x30 - ll
pub fn ll(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("ll");
}

// 0x31 - lwc1
pub fn lwc1(program_state: &mut ProgramState, args: IArgs) -> () {
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
            Err(_) => {
                program_state.set_exception(ExceptionType::AddressExceptionLoad);
            }
        }
        i += 1;
    }

    program_state.cp1.registers[args.rt as usize] = f32::from_bits(result_word);
}

// 0x35 - ldc1
pub fn ldc1(program_state: &mut ProgramState, args: IArgs) -> () {
    let _ = is_register_aligned(program_state, args.rt);

    let temp = (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
        + args.imm as i32) as u32;

    if temp % 4 != 0 {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    if !program_state.memory.allows_read_from(temp)
        || !program_state.memory.allows_read_from(temp + 7)
    {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // Checks passed. Load double word.
    let mut i = 0;
    let mut result_double: u64 = 0;
    while i < 8 {
        match program_state.memory.read_byte(temp + i) {
            Ok(b) => result_double |= (b as u64) << (56 - (i * 8)),
            Err(_) => {
                program_state.set_exception(ExceptionType::AddressExceptionLoad);
            }
        }
        i += 1;
    }

    pack_up_u64(program_state, args.rt, result_double);
}

// 0x39 - swc1
pub fn swc1(program_state: &mut ProgramState, args: IArgs) -> () {
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

    // Retrieve value of ft from coprocessor 1
    let value: u32 = f32::to_bits(program_state.cp1.registers[args.rt as usize]);

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

// 0x3d - sdc1
pub fn sdc1(program_state: &mut ProgramState, args: IArgs) -> () {
    let _ = is_register_aligned(program_state, args.rt);

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

    // Retrieve value of ft from coprocessor 1
    let value: u64 = extract_u64(program_state, args.rt);

    // Checks passed. Store word.
    let mut i = 0;
    while i < 8 {
        // Shift/mask value to get correct byte
        let new_byte: u8 = ((value >> (i * 8)) & 0xFF) as u8;
        // Write it to correct location
        match program_state.memory.set_byte(temp + (7 - i), new_byte) {
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

/*


  _____  ______ _____ _____ __  __ __  __
 |  __ \|  ____/ ____|_   _|  \/  |  \/  |
 | |__) | |__ | |  __  | | | \  / | \  / |
 |  _  /|  __|| | |_ | | | | |\/| | |\/| |
 | | \ \| |___| |__| |_| |_| |  | | |  | |
 |_|  \_\______\_____|_____|_|  |_|_|  |_|




*/

// 0x00 - bltz
pub fn bltz(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltz");
}

// 0x01 - bgez
pub fn bgez(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgez");
}

// 0x02 - bltzl
pub fn bltzl(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzl");
}

// 0x03 - bgezl
pub fn bgezl(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgezl");
}

// 0x10 - bltzal
pub fn bltzal(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzal");
}

// 0x11 - bgezal
pub fn bgezal(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgezal");
}

// 0x12 - bltzall
pub fn bltzall(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzall");
}

// 0x13 - bgezall
pub fn bgezall(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgezall");
}

/*


   ____  _____   _____ ____  _____  ________   __
  / __ \|  __ \ / ____/ __ \|  __ \|  ____\ \ / /
 | |  | | |__) | |   | |  | | |  | | |__   \ V / 
 | |  | |  ___/| |   | |  | | |  | |  __|   > <  
 | |__| | |    | |___| |__| | |__| | |____ / . \ 
  \____/|_|     \_____\____/|_____/|______/_/ \_\
                                                 
                                                 


*/

// 0x00 - lwxc1
pub fn lwxc1(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("lwxc1");
}

// 0x01 - ldxc1
pub fn ldxc1(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("ldxc1");
}

// 0x05 - luxc1
pub fn luxc1(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("luxc1");
}

/*


   _____ _____  ______ _____ _____          _      ___
  / ____|  __ \|  ____/ ____|_   _|   /\   | |    |__ \
 | (___ | |__) | |__ | |      | |    /  \  | |       ) |
  \___ \|  ___/|  __|| |      | |   / /\ \ | |      / /
  ____) | |    | |___| |____ _| |_ / ____ \| |____ / /_
 |_____/|_|    |______\_____|_____/_/    \_\______|____|




*/

// 0x00 - madd
pub fn madd(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("madd");
}

// 0x01 - maddu
pub fn maddu(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("maddu");
}

// 0x20 - clz
pub fn clz(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("clz");
}

// 0x21 - clo
pub fn clo(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("clo");
}

/*


   _____ _____  ______ _____ _____          _      ____
  / ____|  __ \|  ____/ ____|_   _|   /\   | |    |___ \
 | (___ | |__) | |__ | |      | |    /  \  | |      __) |
  \___ \|  ___/|  __|| |      | |   / /\ \ | |     |__ <
  ____) | |    | |___| |____ _| |_ / ____ \| |____ ___) |
 |_____/|_|    |______\_____|_____/_/    \_\______|____/




*/

// 0x00 - ext (extract bit fields)
pub fn ext(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("ext");
}

// 0x04 - ins (insert bit fields)
pub fn ins(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("ins");
}
