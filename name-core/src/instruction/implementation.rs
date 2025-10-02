use crate::exception::definitions::{ExceptionType, FpExceptionType};
use crate::instruction::formats::cond_mov_cc_type::CondMovCCArgs;
use crate::instruction::formats::cop_mov_r_type::CopMovRArgs;
use crate::instruction::formats::fp_cc_branch_type::FpCCBranchArgs;
use crate::instruction::formats::fp_cc_type::FpCCArgs;
use crate::instruction::formats::fp_four_reg_type::FpFourRegArgs;
use crate::instruction::formats::fp_r_type::FpRArgs;
use crate::instruction::formats::i_type::IArgs;
use crate::instruction::formats::j_type::JArgs;
use crate::instruction::formats::r_type::RArgs;
use crate::instruction::formats::regimm_i_type::RegImmIArgs;
use crate::instruction::implementation_helpers::{apply_fpu_rounding, perform_op_with_flush};
use crate::instruction::sign_magnitude::{SignMagnitudeLong, SignMagnitudeWord};
use crate::structs::{
    ProgramState,
    Register::{At, Ra},
};

use super::implementation_helpers::{extract_u64, is_register_aligned, pack_up_u64};

// This file contains the implementations for all
// individual instructions defined in the instruction set

/// This is the double-precision QNaN that will be supplied if:
/// an instruction generates an InvalidOperation floating-point error, and
/// FCSR disables InvalidOperation explicit trap.
/// If you don't know what that means, you should DEFINITELY NOT mess with this value!
///
/// P.S., the magic number bit pattern comes from me generating a valid QNaN according to
/// MIPS Volume I-A. See table 6.3 on page 82!
const F64_QNAN: f64 = f64::from_bits(0x7ff8_0000_0000_0000);
/// See F64_QNAN
const F32_QNAN: f32 = f32::from_bits(0x7fc0_0000);
// These NaNs are going to be related to error states in conversion.
/// This NaN is for an FP number that's too big to be included as a long.
const LONG_TOO_BIG_NAN: u64 = 0x7FFF_FFFF_FFFF_FFFF;
/// This NaN is for an FP number that's too small to be included as a long.
const LONG_TOO_SMALL_NAN: u64 = 0x8000_0000_0000_0000;
/// This NaN is for an FP number that's too big to be included as a word.
const WORD_TOO_BIG_NAN: u32 = 0x7FFF_FFFF;
/// This NaN is for an FP number that's too small to be included as a word.
const WORD_TOO_SMALL_NAN: u32 = 0x8000_0000;

/*


  _   _     __      _______ _____       _______ _____ ____  _   _
 | \ | |   /\ \    / /_   _/ ____|   /\|__   __|_   _/ __ \| \ | |
 |  \| |  /  \ \  / /  | || |  __   /  \  | |    | || |  | |  \| |
 | . ` | / /\ \ \/ /   | || | |_ | / /\ \ | |    | || |  | | . ` |
 | |\  |/ ____ \  /   _| || |__| |/ ____ \| |   _| || |__| | |\  |
 |_| \_/_/    \_\/   |_____\_____/_/    \_\_|  |_____\____/|_| \_|


    Some macros are defined at the very top of the file to make logic more readable.

    To get around this file, you should know the order in which these
    instruction implementations appear.
     - OPCODE (standard opcodes that do not define an instruction class)
     - SPECIAL (opcode 0x00)
     - REGIMM (opcode 0x01)
     - COP0 (opcode 0x10)
     - COP1 (opcode 0x11)
     - COP1X (opcode 0x13)
     - SPECIAL2 (opcode 0x1c)
     - SPECIAL3 (opcode 0x1f)
*/

/*

  __  __          _____ _____   ____   _____
 |  \/  |   /\   / ____|  __ \ / __ \ / ____|
 | \  / |  /  \ | |    | |__) | |  | | (___
 | |\/| | / /\ \| |    |  _  /| |  | |\___ \
 | |  | |/ ____ \ |____| | \ \| |__| |____) |
 |_|  |_/_/    \_\_____|_|  \_\\____/|_____/



*/

/// This macro easily checks the alignment of an arbitrary number of
/// passed floating-point register locations.
/// For use in double precision and long type floating-point instructions.
#[macro_export]
macro_rules! check_alignment {
    ($program_state: expr, $($x: expr),+ ) => {
        $(
            let _ = is_register_aligned($program_state, $x);
        )+
    };
}

/// This macro supplies a 32-bit QNaN to the given program state (into fd) and returns.
/// (program_state, destination)
#[macro_export]
macro_rules! f32_qnan {
    ($program_state: expr, $destination: expr) => {
        $program_state.cp1.registers[$destination as usize] = F32_QNAN.to_bits();
        return;
    };
}

/// This macro supplies a 64-bit QNaN to the given program state (into fd, fd+1) and returns.
/// (program_state, destination)
#[macro_export]
macro_rules! f64_qnan {
    ($program_state: expr, $destination: expr) => {
        pack_up_u64($program_state, $destination, F64_QNAN.to_bits());
        return;
    };
}

/// This macro supplies the rounded version of the result using RM in FCSR to destination in program state.
/// (program_state, destination, result)
#[macro_export]
macro_rules! f32_roundoff {
    ($program_state: expr, $destination: expr, $result: expr) => {
        let rounded_result = apply_fpu_rounding($program_state, $result);
        $program_state.cp1.registers[$destination as usize] = rounded_result.to_bits();
        return;
    };
}

/// This macro supplies the rounded version of the result using RM in FCSR to destination in program state.
/// (program_state, destination, result)
#[macro_export]
macro_rules! f64_roundoff {
    ($program_state: expr, $destination: expr, $result: expr) => {
        let rounded_result = apply_fpu_rounding($program_state, $result);
        pack_up_u64($program_state, $destination, rounded_result.to_bits());
        return;
    };
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
    if program_state.cpu.general_purpose_registers[args.rs as usize]
        != program_state.cpu.general_purpose_registers[args.rt as usize]
    {
        return;
    }

    // Sign extend offset
    let offset: i32 = (args.imm as u16 as i16 as i32) << 2;

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    program_state.jump_if_valid(temp);
}

// 0x05 - bne
pub fn bne(program_state: &mut ProgramState, args: IArgs) -> () {
    if program_state.cpu.general_purpose_registers[args.rs as usize]
        == program_state.cpu.general_purpose_registers[args.rt as usize]
    {
        return;
    }

    // Sign extend offset
    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    program_state.jump_if_valid(temp);
}

// 0x06 - blez
pub fn blez(program_state: &mut ProgramState, args: IArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32) > 0 {
        return;
    }

    let offset: i32 = ((args.imm & 0xFFFF) as i16 as i32) << 2;

    let temp = (program_state.cpu.pc as i32 + offset) as u32;

    program_state.jump_if_valid(temp);
}

// 0x07 - bgtz
pub fn bgtz(program_state: &mut ProgramState, args: IArgs) -> () {
    if program_state.cpu.general_purpose_registers[args.rs as usize] as i32 <= 0 {
        return;
    }

    // Sign extend offset
    let offset: i32 = (args.imm as i16 as i32) << 2;

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

// 0x0a - slti
pub fn slti(program_state: &mut ProgramState, args: IArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32) < (args.imm as i32) {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 1 as u32;
    } else {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 0 as u32;
    }
}

// 0x0b - sltiu
pub fn sltiu(program_state: &mut ProgramState, args: IArgs) -> () {
    if program_state.cpu.general_purpose_registers[args.rs as usize] < (args.imm as u32) {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 1 as u32;
    } else {
        program_state.cpu.general_purpose_registers[args.rt as usize] = 0 as u32;
    }
}

// 0x0c - andi
pub fn andi(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize] & (args.imm as i16 as u32);
}

// 0x0d - ori
pub fn ori(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            | (args.imm as i16 as i32 as u32);
}

// 0x0e - xori
pub fn xori(program_state: &mut ProgramState, args: IArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rt as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            ^ (args.imm as i16 as i32 as u32);
}

// 0x0f - lui
pub fn lui(program_state: &mut ProgramState, args: IArgs) -> () {
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

// 0x19 - sh
pub fn sh(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("sh");
}

// 0x1a - swl
pub fn swl(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("swl");
}

// 0x1e - swr
pub fn swr(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("swr");
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

    program_state.cp1.registers[args.rt as usize] = result_word;
}

// 0x33 - pref
pub fn pref(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("pref");
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

// 0x38 - sc
pub fn sc(_program_state: &mut ProgramState, _args: IArgs) -> () {
    todo!("sc");
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
    let value: u32 = program_state.cp1.registers[args.rt as usize];

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


   _____ _____  ______ _____ _____          _
  / ____|  __ \|  ____/ ____|_   _|   /\   | |
 | (___ | |__) | |__ | |      | |    /  \  | |
  \___ \|  ___/|  __|| |      | |   / /\ \ | |
  ____) | |    | |___| |____ _| |_ / ____ \| |____
 |_____/|_|    |______\_____|_____/_/    \_\______|




*/

// 0x00 - sll
pub fn sll(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rt as usize] << args.shamt;
}

// 0x01 - mov_conditional (multiplexes on t/f in args)
pub fn mov_conditional(_program_state: &mut ProgramState, _args: CondMovCCArgs) -> () {
    todo!("movf, movt");
}

// 0x02 - srl
pub fn srl(program_state: &mut ProgramState, args: RArgs) -> () {
    if args.rs != 1 {
        program_state.cpu.general_purpose_registers[args.rd as usize] =
            program_state.cpu.general_purpose_registers[args.rt as usize] >> args.shamt;
    } else {
        todo!("rotr");
    }
}

// 0x03 - sra
pub fn sra(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("sra");
}

// 0x04 - sllv
pub fn sllv(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            << (program_state.cpu.general_purpose_registers[args.rt as usize] & 0b0001_1111);
}

// 0x06 - srlv
pub fn srlv(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            >> (program_state.cpu.general_purpose_registers[args.rt as usize] & 0b0001_1111);
}

// 0x07 - srav
pub fn srav(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("srav");
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

// 0x0a - movz
pub fn movz(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("movz");
}

// 0x0b - movn
pub fn movn(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("movn");
}

// 0x0c - syscall
pub fn syscall(program_state: &mut ProgramState, _args: RArgs) -> () {
    program_state.set_exception(ExceptionType::Syscall);
}

// 0x0d - break
pub fn break_instr(program_state: &mut ProgramState, _args: RArgs) -> () {
    program_state.set_exception(ExceptionType::Breakpoint);
}

// 0x0f - sync
pub fn sync(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("sync instruction");
}

// 0x10 - mfhi
pub fn mfhi(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("mfhi");
}

// 0x11 - mthi
pub fn mthi(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("mthi");
}

// 0x12 - mflo
pub fn mflo(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("mflo");
}

// 0x13 - mtlo
pub fn mtlo(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("mtlo");
}

// 0x18 - mult
pub fn mult(program_state: &mut ProgramState, args: RArgs) -> () {
    let temp: i64 = program_state.cpu.general_purpose_registers[args.rs as usize] as i32 as i64
        * program_state.cpu.general_purpose_registers[args.rt as usize] as i32 as i64;
    (program_state.cpu.hi, program_state.cpu.lo) = ((temp >> 32) as i32 as u32, temp as i32 as u32);
}

// 0x19 - multu
pub fn multu(program_state: &mut ProgramState, args: RArgs) -> () {
    let temp: u64 = program_state.cpu.general_purpose_registers[args.rs as usize] as u64
        * program_state.cpu.general_purpose_registers[args.rt as usize] as u64;
    (program_state.cpu.hi, program_state.cpu.lo) = ((temp >> 32) as u32, temp as u32);
}

// 0x1A - div
pub fn div(program_state: &mut ProgramState, args: RArgs) -> () {
    // I should note that dividing by zero
    // does NOT trigger any special exception per the MIPS standard.
    // quotient goes into LO, remainder into HI
    (program_state.cpu.lo, program_state.cpu.hi) = (
        (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
            / program_state.cpu.general_purpose_registers[args.rt as usize] as i32) as u32,
        (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
            % program_state.cpu.general_purpose_registers[args.rt as usize] as i32) as u32,
    );
}

// 0x1B - divu
pub fn divu(program_state: &mut ProgramState, args: RArgs) -> () {
    // I should note that dividing by zero
    // does NOT trigger any special exception per the MIPS standard.
    // quotient goes into LO, remainder into HI
    (program_state.cpu.lo, program_state.cpu.hi) = (
        program_state.cpu.general_purpose_registers[args.rs as usize]
            / program_state.cpu.general_purpose_registers[args.rt as usize],
        program_state.cpu.general_purpose_registers[args.rs as usize]
            % program_state.cpu.general_purpose_registers[args.rt as usize],
    );
}

// 0x20 - add
pub fn add(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        program_state.cpu.general_purpose_registers[args.rs as usize]
            + program_state.cpu.general_purpose_registers[args.rt as usize];
}

// 0x21 - addu
pub fn addu(program_state: &mut ProgramState, args: RArgs) -> () {
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

// 0x30 - tge
pub fn tge(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tge");
}

// 0x31 - tgeu
pub fn tgeu(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tgeu");
}

// 0x32 - tlt
pub fn tlt(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tlt");
}

// 0x33 - tlt
pub fn tltu(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tltu");
}

// 0x34 - teq
pub fn teq(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("teq");
}

// 0x36 - tne
pub fn tne(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tne");
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
pub fn bltz(program_state: &mut ProgramState, args: RegImmIArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32)
        >= (program_state.cpu.general_purpose_registers[0] as i32)
    {
        return;
    }

    let offset = (args.imm as u16 as i16 as i32) << 2;
    let target_address = (program_state.cpu.pc as i32 + offset) as u32;
    program_state.jump_if_valid(target_address);
}

// 0x01 - bgez
pub fn bgez(program_state: &mut ProgramState, args: RegImmIArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32)
        < (program_state.cpu.general_purpose_registers[0] as i32)
    {
        return;
    }

    let offset = (args.imm as u16 as i16 as i32) << 2;
    let target_address = (program_state.cpu.pc as i32 + offset) as u32;
    program_state.jump_if_valid(target_address);
}

// 0x02 - bltzl
pub fn bltzl(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzl");
}

// 0x03 - bgezl
pub fn bgezl(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgezl");
}

// 0x08 - tgei
pub fn tgei(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("tgei");
}

// 0x09 - tgeiu
pub fn tgeiu(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("tgeiu");
}

// 0x0a - tlti
pub fn tlti(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("tlti");
}

// 0x0b - tltiu
pub fn tltiu(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("tltiu");
}

// 0x0c - teqi
pub fn teqi(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("teqi");
}

// 0x0e - tnei
pub fn tnei(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("tnei");
}

// 0x10 - bltzal
pub fn bltzal(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzal");
}

// 0x11 - bgezal
pub fn bgezal(program_state: &mut ProgramState, args: RegImmIArgs) -> () {
    if (program_state.cpu.general_purpose_registers[args.rs as usize] as i32)
        < (program_state.cpu.general_purpose_registers[0] as i32)
    {
        return;
    }

    let offset = (args.imm as u16 as i16 as i32) << 2;
    let target_address = (program_state.cpu.pc as i32 + offset) as u32;

    let temp = program_state.cpu.pc;
    program_state.jump_if_valid(target_address);
    program_state.cpu.general_purpose_registers[Ra as usize] = temp;
}

// 0x12 - bltzall
pub fn bltzall(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bltzall");
}

// 0x13 - bgezall
pub fn bgezall(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("bgezall");
}

// 0x1f - synci
pub fn synci(_program_state: &mut ProgramState, _args: RegImmIArgs) -> () {
    todo!("synci");
}

/*


   _____ ____  _____   ___
  / ____/ __ \|  __ \ / _ \
 | |   | |  | | |__) | | | |
 | |   | |  | |  ___/| | | |
 | |___| |__| | |    | |_| |
  \_____\____/|_|     \___/




*/

// 0x01 - tlbr
pub fn tlbr(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tlbr");
}

// 0x02 - tlbwi
pub fn tlbwi(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tlbwi");
}

// 0x04 - mtc0
pub fn mtc0(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("mtc0");
}

// 0x06 - tlbwr
pub fn tlbwr(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tlbwr");
}

// 0x08 - tlbp
pub fn tlbp(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("tlbp");
}

// 0x0a
pub fn rdpgpr(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("rdpgpr");
}

// 0x0e - wrpgpr
pub fn wrpgpr(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("wrpgpr");
}

// EJTAG exceptions:
// 0x1f
pub fn deret(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("deret");
}

//
pub fn mfc0(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("mfc0");
}

// 0x20 - wait
pub fn wait(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("wait instruction");
}

/*

   _____ ____  _____  __
  / ____/ __ \|  __ \/_ |
 | |   | |  | | |__) || |
 | |   | |  | |  ___/ | |
 | |___| |__| | |     | |
  \_____\____/|_|     |_|



*/

// funct code 0 is a little special;
// There are some reserved values for fmt
// which trigger a special instruction class.

// Special instructions: Facilitate movement GPR <-> FPU
// 0x00;0x00 - MF (Move from) - GPR <- FPU
pub fn mfc1(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("mfc1");
}

// 0x00;0x02 - CF (Coprocessor from) - GPR -> FPU
pub fn cfc1(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("cfc1");
}

// 0x00;0x04 - MT (Move to) - GPR -> FPU
pub fn mtc1(_program_state: &mut ProgramState, _args: CopMovRArgs) -> () {
    todo!("mtc1");
}

// 0x00;0x06 - CT (Coprocessor to) - GPR <- FPU
pub fn ctc1(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ctc1");
}

// 0x00;0x08 - bc1<cond><nd>
/// All implementations (t/f, likely/unlikely) are contained in this function.
/// This simplifies the table.
pub fn bc1(program_state: &mut ProgramState, args: FpCCBranchArgs) -> () {
    // match on the type of instruction (update later to account for likely)
    match args.tf {
        0 => {
            // Branch on floating-point false (bc1f)
            if program_state.cp1.get_condition_code(args.cc) {
                return;
            }

            // Sign extend offset
            let offset: i32 = ((args.offset & 0xFFFF) as i16 as i32) << 2;
            let temp = (program_state.cpu.pc as i32 + offset) as u32;
            program_state.jump_if_valid(temp);
        }
        1 => {
            // Branch on floating-point true (bc1t)
            if !program_state.cp1.get_condition_code(args.cc) {
                return;
            }

            // Sign extend offset
            let offset: i32 = ((args.offset & 0xFFFF) as i16 as i32) << 2;
            let temp = (program_state.cpu.pc as i32 + offset) as u32;
            program_state.jump_if_valid(temp);
        }
        _ => {
            // Represents an impossible true/false. Should actually be unreachable!() but you never know...
            program_state.set_exception(ExceptionType::ReservedInstruction);
        }
    }
}

// 0x00 - add.fmt

// 0x00.d - add.d
pub fn add_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs, args.ft);

    let s = f64::from_bits(extract_u64(program_state, args.fs));
    let t = f64::from_bits(extract_u64(program_state, args.ft));

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        // If the exception path was disabled, supply a QNaN
        f64_qnan!(program_state, args.fd);
    }

    // If the operation results in magnitude subtraction of infinities, the operation is invalid.
    if s.is_infinite() && t.is_infinite() && (s.to_bits() >> 63) != (t.to_bits() >> 63)
    // (second check reads: "sign bits don't match")
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // Addition performed here
    let result = perform_op_with_flush(program_state, s + t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        // Overflow without explicit trap results in rounding with RM in FCSR
        f64_roundoff!(program_state, args.fd, result);
    }

    pack_up_u64(program_state, args.fd, result.to_bits());
}

// 0x00.s - add.s
pub fn add_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let s = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let t = f32::from_bits(program_state.cp1.registers[args.ft as usize]);

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // If the operation results in magnitude subtraction of infinities, the operation is invalid.
    if s.is_infinite() && t.is_infinite() && (s.to_bits() >> 31) != (t.to_bits() >> 31)
    // (second check reads: "sign bits don't match")
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // Addition performed here
    let result = perform_op_with_flush(program_state, s + t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f32_roundoff!(program_state, args.fd, result);
    }

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

// 0x01.fmt - sub.fmt

// 0x01.d - sub.d
pub fn sub_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs, args.ft);

    let s = f64::from_bits(extract_u64(program_state, args.fs));
    let t = f64::from_bits(extract_u64(program_state, args.ft));

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // If the operation results in magnitude subtraction of infinities, the operation is invalid.
    if s.is_infinite() && t.is_infinite() && (s.to_bits() >> 63) == (t.to_bits() >> 63)
    // (second check reads: "sign bits match")
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // Subtraction performed here
    let result = perform_op_with_flush(program_state, s - t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f64_roundoff!(program_state, args.fd, result);
    }

    pack_up_u64(program_state, args.fd, result.to_bits());
}

// 0x01.s - sub.s
pub fn sub_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let s = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let t = f32::from_bits(program_state.cp1.registers[args.ft as usize]);

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // If the operation results in magnitude subtraction of infinities, the operation is invalid.
    if s.is_infinite() && t.is_infinite() && (s.to_bits() >> 31) == (t.to_bits() >> 31)
    // (second check reads: "sign bits match")
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // Subtraction performed here
    let result = perform_op_with_flush(program_state, s - t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f32_roundoff!(program_state, args.fd, result);
    }

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

// 0x02.fmt - mul.fmt

// 0x02.d - mul.d
pub fn mul_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs, args.ft);

    let s = f64::from_bits(extract_u64(program_state, args.fs));
    let t = f64::from_bits(extract_u64(program_state, args.ft));

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // If the operation is 0 x inf, with any signs, then trigger an InvalidOperation.
    if s.abs() == 0.0 && t.is_infinite() || s.is_infinite() && t.abs() == 0.0 {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // Perform multiplication
    let result = perform_op_with_flush(program_state, s * t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f64_roundoff!(program_state, args.fd, result);
    }

    pack_up_u64(program_state, args.fd, result.to_bits());
}

// 0x02.s - mul.s
pub fn mul_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let s = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let t = f32::from_bits(program_state.cp1.registers[args.ft as usize]);

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // If the operation is 0 x inf, with any signs, then trigger an InvalidOperation.
    if s.abs() == 0.0 && t.is_infinite() || s.is_infinite() && t.abs() == 0.0 {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // Perform multiplication
    let result = perform_op_with_flush(program_state, s * t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f32_roundoff!(program_state, args.fd, result);
    }

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

// 0x03 - div.fmt

// 0x03.d - div.d
pub fn div_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs, args.ft);

    let s: f64 = f64::from_bits(extract_u64(program_state, args.fs));
    let t: f64 = f64::from_bits(extract_u64(program_state, args.ft));

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // 0/0 and inf/inf are invalid.
    if s.abs() == 0.0 && t.abs() == 0.0 || s.is_infinite() && t.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f64_qnan!(program_state, args.fd);
    }

    // Check for division by zero
    if t.abs() == 0.0 {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::DivideByZero));
        // If the explicit trap is disabled, supply a properly signed infinity
        if s.signum() == t.signum() {
            pack_up_u64(program_state, args.fd, f64::INFINITY.to_bits());
            return;
        } else {
            pack_up_u64(program_state, args.fd, f64::NEG_INFINITY.to_bits());
            return;
        }
    }

    // Division performed here
    let result: f64 = perform_op_with_flush(program_state, s / t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f64_roundoff!(program_state, args.fd, result);
    }

    pack_up_u64(program_state, args.fd, result.to_bits());
}

// 0x03.s - div.s
pub fn div_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let s = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let t = f32::from_bits(program_state.cp1.registers[args.ft as usize]);

    // If either operation is a signaling NaN, the operation is invalid.
    if s.is_nan() || t.is_nan() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // 0/0 and inf/inf are invalid.
    if s.abs() == 0.0 && t.abs() == 0.0 || s.is_infinite() && t.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        f32_qnan!(program_state, args.fd);
    }

    // Check for division by zero
    if t.abs() == 0.0 {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::DivideByZero));
        // If the explicit trap is disabled, supply a properly signed infinity
        if s.signum() == t.signum() {
            program_state.cp1.registers[args.fd as usize] = f32::INFINITY.to_bits();
            return;
        } else {
            program_state.cp1.registers[args.fd as usize] = f32::NEG_INFINITY.to_bits();
            return;
        }
    }

    // Division performed here
    let result: f32 = perform_op_with_flush(program_state, s / t);

    // Check for overflow after operation
    if result.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Overflow));
        f32_roundoff!(program_state, args.fd, result);
    }

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

// 0x04.d - sqrt.d
pub fn sqrt_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("sqrt.d");
}

// 0x04.s - sqrt.s
pub fn sqrt_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("sqrt.s");
}

// 0x05 - abs.fmt (this should NOT panic when taking abs(NaN) due to FCSR dictating IEEE 2008 revision instead of legacy MIPS!)

// 0x05.d - abs.d
pub fn abs_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs);
    let s = f64::from_bits(extract_u64(program_state, args.fs));
    let result = s.abs();
    let _ = pack_up_u64(program_state, args.fd, result.to_bits());
}

// 0x05.s - abs.s
pub fn abs_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let s = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    program_state.cp1.registers[args.fd as usize] = f32::abs(s).to_bits();
}

// 0x06.d - mov.d
pub fn mov_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs);

    let temp: u64 = extract_u64(program_state, args.fs);
    let _ = pack_up_u64(program_state, args.fd, temp);
}

// 0x06.s - mov.s
pub fn mov_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("mov.s");
}

// 0x07.fmt - neg.fmt

// 0x07.d - neg.d
pub fn neg_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs);

    let double_value = f64::from_bits(extract_u64(program_state, args.fs));
    let negated_value = double_value * -1.0;

    pack_up_u64(program_state, args.fd, negated_value.to_bits());
}

// 0x07.s - neg.s
pub fn neg_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let negated_value = single_value * -1.0;

    program_state.cp1.registers[args.fd as usize] = negated_value.to_bits();
}

// 0x08.fmt - round.l.fmt

// These higher-order functions round floating-point numbers using a specific mode.
// They allows us to write all variants of mode.fmt.fmt with simple function calls.

/// This is a generalization of <round>.l.d
fn mode_l_d<F>(program_state: &mut ProgramState, args: FpRArgs, mode: F) -> () where F: Fn(f64) -> f64 {
    check_alignment!(program_state, args.fd, args.fs);

    let double_value = f64::from_bits(extract_u64(program_state, args.fs));

    if double_value.is_nan() || double_value.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        // Non-explicit path requires some pretty iffy checks.
        if double_value.is_nan() {
            // Supply 0 if NaN in conversion.
            pack_up_u64(program_state, args.fd, 0);
            return;
        } else if double_value.is_sign_positive() {
            // Positive infinity should supply a very specific NaN:
            pack_up_u64(program_state, args.fd, LONG_TOO_BIG_NAN);
            return;
        } else {
            pack_up_u64(program_state, args.fd, LONG_TOO_SMALL_NAN);
            return;
        }
    }

    let rounded_double = mode(double_value);

    if !SignMagnitudeLong::can_represent_double(rounded_double) {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        if rounded_double > 0.0 {
            pack_up_u64(program_state, args.fd, LONG_TOO_BIG_NAN);
            return;
        } else {
            pack_up_u64(program_state, args.fd, LONG_TOO_SMALL_NAN);
            return;
        }
    }

    let long_value = SignMagnitudeLong::from(rounded_double);

    pack_up_u64(program_state, args.fd, long_value.to_bits());
}

/// This is a generalization of <round>.l.s
fn mode_l_s<F>(program_state: &mut ProgramState, args: FpRArgs, mode: F) where F: Fn(f32) -> f32 {
        check_alignment!(program_state, args.fd);

    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);

    if single_value.is_nan() || single_value.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::InvalidOperation));
        if single_value.is_nan() {
            // Supply 0 if NaN
            pack_up_u64(program_state, args.fd, 0);
            return;
        } else if single_value.is_sign_positive() {
            pack_up_u64(program_state, args.fd, LONG_TOO_BIG_NAN);
            return;
        } else {
            pack_up_u64(program_state, args.fd, LONG_TOO_SMALL_NAN);
            return;
        }
    }

    let rounded_single = mode(single_value);

    if !SignMagnitudeLong::can_represent_single(rounded_single) {
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::InvalidOperation));
        if rounded_single > 0.0 {
            pack_up_u64(program_state, args.fd, LONG_TOO_BIG_NAN);
            return;
        } else {
            pack_up_u64(program_state, args.fd, LONG_TOO_SMALL_NAN);
            return;
        }
    }

    let long_value = SignMagnitudeLong::from(rounded_single as f64);
    pack_up_u64(program_state, args.fd, long_value.to_bits());
}

/// This is a generalization of <round>.w.d
fn mode_w_d<F>(program_state: &mut ProgramState, args: FpRArgs, mode: F) where F: Fn(f64) -> f64 {
    check_alignment!(program_state, args.fs);

    let double_value = f64::from_bits(extract_u64(program_state, args.fs));

    if double_value.is_nan() || double_value.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        // If no explicit trap, NaNs should be 0, infinities too big:
        if double_value.is_nan() {
            program_state.cp1.registers[args.fd as usize] = 0;
            return;
        } else if double_value.is_sign_positive() {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_BIG_NAN;
            return;
        } else {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_SMALL_NAN;
            return;
        }
    }

    let rounded_double = mode(double_value);

    if !SignMagnitudeWord::can_represent_double(rounded_double) {
        // The result cannot actually be represented as a word
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        if rounded_double > 0.0 {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_BIG_NAN;
            return;
        } else {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_SMALL_NAN;
            return;
        }
    }

    let result = SignMagnitudeWord::from(rounded_double as f32);

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

/// This is a generalization of <round>.w.s
fn mode_w_s<F>(program_state: &mut ProgramState, args: FpRArgs, mode: F) where F: Fn(f32) -> f32 {
    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);

    if single_value.is_nan() || single_value.is_infinite() {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        // If no explicit trap, supply 0 on NaN and special on infty
        if single_value.is_nan() {
            program_state.cp1.registers[args.fd as usize] = 0;
            return;
        } else if single_value.is_sign_positive() {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_BIG_NAN;
            return;
        } else {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_SMALL_NAN;
            return;
        }
    }

    let rounded_single = mode(single_value);

    if !SignMagnitudeWord::can_represent_single(rounded_single) {
        // The result cannot actually be represented as a word
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
        if rounded_single > 0.0 {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_BIG_NAN;
            return;
        } else {
            program_state.cp1.registers[args.fd as usize] = WORD_TOO_SMALL_NAN;
            return;
        }
    }

    let result = SignMagnitudeWord::from(rounded_single as f32);

    program_state.cp1.registers[args.fd as usize] = result.to_bits();
}

// 0x08.d - round.l.d
pub fn round_l_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_d(program_state, args, f64::round);
}

// 0x08.s - round.l.s
pub fn round_l_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_s(program_state, args, f32::round);
}

// 0x09.d - trunc.l.d
pub fn trunc_l_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_d(program_state, args, f64::trunc);
}

// 0x09.s - trunc.l.s
pub fn trunc_l_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_s(program_state, args, f32::trunc);
}

// 0x0a.d - ceil.l.d
pub fn ceil_l_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_d(program_state, args, f64::ceil);
}

// 0x0a.s - ceil.l.s
pub fn ceil_l_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_s(program_state, args, f32::ceil);
}

// 0x0b.d - floor.l.d
pub fn floor_l_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_d(program_state, args, f64::round);
}

// 0x0b.s - floor.l.s
pub fn floor_l_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_l_s(program_state, args, f32::floor);
}

// 0x0c.d - round.w.d
pub fn round_w_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_d(program_state, args, f64::round);
}

// 0x0c.s - round.w.s
pub fn round_w_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_s(program_state, args, f32::round);
}

// 0x0d.d - trunc.w.d
pub fn trunc_w_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_d(program_state, args, f64::trunc);
}

// 0x0d.s - trunc.w.s
pub fn trunc_w_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_s(program_state, args, f32::trunc);
}

// 0x0e.d - ceil.w.d
pub fn ceil_w_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_d(program_state, args, f64::ceil);
}

// 0x0e.s - ceil.w.s
pub fn ceil_w_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_s(program_state, args, f32::ceil);
}

// 0x0f.d - floor.w.d
pub fn floor_w_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_d(program_state, args, f64::floor);
}

// 0x0f.s - floor.w.s
pub fn floor_w_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    mode_w_s(program_state, args, f32::floor);
}

// 0x11.d - movf.d/movt.d
pub fn mov_conditional_d(_program_state: &mut ProgramState, _args: CondMovCCArgs) -> () {
    todo!("movf.d/movt.d");
}

// 0x11.s - movf.s/movt.s
pub fn mov_conditional_s(_program_state: &mut ProgramState, _args: CondMovCCArgs) -> () {
    todo!("movf.s/movt.s");
}

// 0x12.d - movz.d
pub fn movz_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("movz.d")
}

// 0x12.s - movz.s
pub fn movz_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("movz.s")
}

// 0x13.d - movn.d
pub fn movn_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("movn.d");
}

// 0x13.s - movn.s
pub fn movn_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("movn.s");
}

// 0x15.d - recip.d
pub fn recip_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("recip.d");
}

// 0x15.s - recip.s
pub fn recip_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("recip.s");
}

// 0x16.d - rsqrt.d
pub fn rsqrt_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("rsqrt.d");
}

// 0x16.s - rsqrt.s
pub fn rsqrt_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("rsqrt.s");
}

// 0x20.fmt - cvt.s.fmt

// 0x20.d - cvt.s.d
pub fn cvt_s_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fs);

    let double_value = extract_u64(program_state, args.fs) as f64;
    let single_value = double_value as f32;

    program_state.cp1.registers[args.fd as usize] = single_value.to_bits();
}

// 0x20.l - cvt.s.l
pub fn cvt_s_l(program_state: &mut ProgramState, args: FpRArgs) -> () {
    // Officially, this is supposed to be unpredictable.
    check_alignment!(program_state, args.fs);

    let long_value = SignMagnitudeLong::from_bits(extract_u64(program_state, args.fs));
    let double_value = i64::from(long_value) as f64;
    let single_value = double_value as f32;

    program_state.cp1.registers[args.fd as usize] = single_value.to_bits();
}

// 0x20.w - cvt.s.w
pub fn cvt_s_w(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let word_value = i32::from(SignMagnitudeWord::from_bits(
        program_state.cp1.registers[args.fs as usize],
    ));
    let single_value = word_value as f32;
    program_state.cp1.registers[args.fd as usize] = single_value.to_bits();
}

// 0x21.fmt - cvt.d.fmt

// 0x21.s - cvt.d.l
pub fn cvt_d_l(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs);

    let long_value = SignMagnitudeLong::from_bits(extract_u64(program_state, args.fs));
    let double_value = i64::from(long_value) as f64;

    pack_up_u64(program_state, args.fd, double_value.to_bits());
}

// 0x21.s - cvt.d.s
pub fn cvt_d_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd);

    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);
    let double_value = single_value as f64;

    pack_up_u64(program_state, args.fd, double_value.to_bits());
}

// 0x21.s - cvt.d.w
pub fn cvt_d_w(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd);

    let word_value = SignMagnitudeWord::from_bits(program_state.cp1.registers[args.fs as usize]);
    let single_value = i32::from(word_value) as f32;
    let double_value = single_value as f64;

    pack_up_u64(program_state, args.fd, double_value.to_bits());
}

// 0x24.fmt - cvt.w.fmt

// 0x24.d - cvt.w.d
pub fn cvt_w_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fs);

    let double_value = f64::from_bits(extract_u64(program_state, args.fs));

    if !SignMagnitudeWord::can_represent_double(double_value)
        || double_value.is_nan()
        || double_value.is_infinite()
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
    }

    let rounded_double = apply_fpu_rounding(program_state, double_value);
    let word_value = rounded_double.trunc() as i64 as i32;
    let word = SignMagnitudeWord::from(word_value);

    program_state.cp1.registers[args.fd as usize] = word.to_bits();
}

// 0x24.s - cvt.w.s
pub fn cvt_w_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);

    if !SignMagnitudeWord::can_represent_single(single_value)
        || single_value.is_nan()
        || single_value.is_infinite()
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
    }

    let rounded_single = apply_fpu_rounding(program_state, single_value);
    let word_value = rounded_single.trunc() as i32;
    let word = SignMagnitudeWord::from(word_value);

    program_state.cp1.registers[args.fd as usize] = word.to_bits();
}

// 0x25.fmt - cvt.l.fmt

// 0x25.d - cvt.l.d
pub fn cvt_l_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd, args.fs);

    let double_value = f64::from_bits(extract_u64(program_state, args.fs));

    if !SignMagnitudeLong::can_represent_double(double_value)
        || double_value.is_nan()
        || double_value.is_infinite()
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
    }

    let rounded_double = apply_fpu_rounding(program_state, double_value);
    let long_value = rounded_double.trunc() as i64;
    let long = SignMagnitudeLong::from(long_value);

    pack_up_u64(program_state, args.fd, long.to_bits());
}

// 0x25.s - cvt.l.s
pub fn cvt_l_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    check_alignment!(program_state, args.fd);

    let single_value = f32::from_bits(program_state.cp1.registers[args.fs as usize]);

    if !SignMagnitudeLong::can_represent_single(single_value)
        || single_value.is_nan()
        || single_value.is_infinite()
    {
        program_state.set_exception(ExceptionType::FloatingPoint(
            FpExceptionType::InvalidOperation,
        ));
    }

    let rounded_single = apply_fpu_rounding(program_state, single_value);
    let long_value = rounded_single as i32 as i64;
    let long = SignMagnitudeLong::from(long_value);

    pack_up_u64(program_state, args.fd, long.to_bits());
}

// 0x30.d - c.f.d
pub fn c_f_d(program_state: &mut ProgramState, args: FpCCArgs) -> () {
    program_state.cp1.set_condition_code(args.cc, false);
}

// 0x30.d - c.f.s
pub fn c_f_s(program_state: &mut ProgramState, args: FpCCArgs) -> () {
    program_state.cp1.set_condition_code(args.cc, false);
}

// 0x31.d - c.un.d
pub fn c_un_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.un.d");
}

// 0x31.s - c.un.s
pub fn c_un_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.un.s");
}

// 0x32.d - c.eq.d
pub fn c_eq_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.eq.d");
}

// 0x32.s - c.eq.s
pub fn c_eq_s(program_state: &mut ProgramState, args: FpCCArgs) -> () {
    program_state.cp1.set_condition_code(
        args.cc,
        program_state.cp1.registers[args.ft as usize]
            == program_state.cp1.registers[args.fs as usize],
    );
}

// 0x33.d - c.ueq.d
pub fn c_ueq_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ueq.d");
}

// 0x33.s - c.ueq.s
pub fn c_ueq_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ueq.s");
}

// 0x34.d - c.olt.d
pub fn c_olt_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.olt.d");
}

// 0x34.s - c.olt.s
pub fn c_olt_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.olt.s");
}

// 0x35.d - c.ult.d
pub fn c_ult_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ult.d");
}

// 0x35.s - c.ult.s
pub fn c_ult_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ult.s");
}

// 0x36.d - c.ole.d
pub fn c_ole_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ole.d");
}

// 0x36.s - c.ole.s
pub fn c_ole_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ole.s");
}

// 0x37.d - c.ule.d
pub fn c_ule_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ule.d");
}

// 0x37.s - c.ule.s
pub fn c_ule_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ule.s");
}

// 0x38.d - c.sf.d
pub fn c_sf_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.sf.d");
}

// 0x38.s - c.sf.s
pub fn c_sf_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.sf.s");
}

// 0x39.d - c.ngle.d
pub fn c_ngle_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngle.d");
}

// 0x39.s - c.ngle.s
pub fn c_ngle_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngle.s");
}

// 0x3a.d - c.seq.d
pub fn c_seq_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.seq.d");
}

// 0x3a.s - c.seq.s
pub fn c_seq_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.seq.s");
}

// 0x3b.d - c.ngl.d
pub fn c_ngl_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngl.d");
}

// 0x3b.s - c.ngl.s
pub fn c_ngl_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngl.s");
}

// 0x3c.d - c.lt.d
pub fn c_lt_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.lt.d");
}

// 0x3c.s - c.lt.s
pub fn c_lt_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.lt.s");
}

// 0x3d.d - c.nge.d
pub fn c_nge_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.nge.d");
}

// 0x3d.s - c.nge.s
pub fn c_nge_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.nge.s");
}

// 0x3e.d - c.le.d
pub fn c_le_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.le.d");
}

// 0x3e.s - c.le.s
pub fn c_le_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.le.s");
}

// 0x3f.d - c.ngt.d
pub fn c_ngt_d(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngt.d");
}

// 0x3f.s - c.ngt.s
pub fn c_ngt_s(_program_state: &mut ProgramState, _args: FpCCArgs) -> () {
    todo!("c.ngt.s");
}

/*


   _____ ____  _____  ____   __
  / ____/ __ \|  __ \/_ \ \ / /
 | |   | |  | | |__) || |\ V /
 | |   | |  | |  ___/ | | > <
 | |___| |__| | |     | |/ . \
  \_____\____/|_|     |_/_/ \_\




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

// 0x08 - swxc1
pub fn swxc1(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("swxc1");
}

// 0x09 - sdxc1
pub fn sdxc1(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("sdxc1");
}

// 0x0f - prefx
pub fn prefx(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("prefx");
}

// 0x20.d - madd.d
pub fn madd_d(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("madd.d");
}

// 0x20.s - madd.s
pub fn madd_s(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("madd.s");
}

// 0x28.d - msub.d
pub fn msub_d(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("msub.d");
}

// 0x28.s - msub.s
pub fn msub_s(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("msub.s");
}

// 0x30 - nmadd.d
pub fn nmadd_d(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("nmadd.d");
}

// 0x30 - nmadd.s
pub fn nmadd_s(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("nmadd.s");
}

// 0x38 - nmsub.d
pub fn nmsub_d(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("nmsub.d");
}

// 0x38 - nmsub.s
pub fn nmsub_s(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("nmsub.s");
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

// 0x02 - mul
pub fn mul(program_state: &mut ProgramState, args: RArgs) -> () {
    program_state.cpu.general_purpose_registers[args.rd as usize] =
        (program_state.cpu.general_purpose_registers[args.rs as usize] as i32
            * program_state.cpu.general_purpose_registers[args.rt as usize] as i32) as u32;
}

// 0x04 - msub
pub fn msub(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("msub");
}

// 0x05 - msubu
pub fn msubu(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("msubu");
}

// 0x20 - clz
pub fn clz(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("clz");
}

// 0x21 - clo
pub fn clo(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("clo");
}

// 0x3f - sddbp
pub fn sdbbp(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("sddbp");
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

// 0x20 - BSHFL multiplexing
pub fn bshfl(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("wsbh(0x02) , seb(0x20) , seh(0x30)");
}

// 0x3b - rdhwr (read hardware register)
pub fn rdhwr(_program_state: &mut ProgramState, _args: RArgs) -> () {
    todo!("rdhwr");
}
