use crate::{exception::definitions::ExceptionType, structs::ProgramState};

use super::FpRArgs;

/// Helper function for instructions that operate on register pairs to remove invalid cases.
/// If the register is improperly aligned given current program state, it will trigger
/// an exception in Coprocessor 0.
fn is_register_aligned(program_state: &mut ProgramState, reg: u32) -> bool {
    if reg % 2 == 0 {
        return true;
    } else {
        program_state.set_exception(ExceptionType::ReservedInstruction);
        return false;
    }
}

/// This is a helper function for implementations that extracts a register pair's value as bits.
/// The passed argument, target, should be the u32 repr of the high register (even register).
/// For instance, extracting the double word inside $f2/$f3 means you should pass $f2's repr.
fn extract_u64(program_state: &mut ProgramState, target: u32) -> u64 {
    ((program_state.cp1.registers[target as usize] as u64) << 32)
        | (program_state.cp1.registers[target as usize + 1] as u64)
}

/// This is a helper function for implementations that packs a u64 double-word value
/// back into a register pair given by target.
/// The target register should be the u32 repr of the high (even) register.
/// For instance, extracting the double word inside $f2/$f3 means you should pass $f2's repr.
fn pack_up_u64(program_state: &mut ProgramState, target: u32, value: u64) {
    program_state.cp1.registers[target as usize] = f32::from_bits((value >> 32) as u32);
    program_state.cp1.registers[target as usize + 1] = f32::from_bits(value as u32);
}

/*

   _____ ____  _____  __
  / ____/ __ \|  __ \/_ |
 | |   | |  | | |__) || |
 | |   | |  | |  ___/ | |
 | |___| |__| | |     | |
  \_____\____/|_|     |_|



*/

// 0x05 - abs.fmt

// 0x05.d - abs.d
pub fn abs_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);

    let temp: u64 = extract_u64(program_state, args.fs);

    // Simply clear the sign bit.
    // Refer to IEEE 754-2008 documentation if this appears non-sensical.
    let mask: u64 = 0x7FFF_FFFF_FFFF_FFFF;
    let result: u64 = temp & mask;

    let _ = pack_up_u64(program_state, args.fd, result);
}

// 0x05.s - abs.s
pub fn abs_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    program_state.cp1.registers[args.fd as usize] =
        f32::abs(program_state.cp1.registers[args.fs as usize]);
}
