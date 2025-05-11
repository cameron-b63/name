use crate::structs::ProgramState;

use super::{
    implementation_helpers::{extract_u64, is_register_aligned, pack_up_u64, perform_op_with_flush},
    FpCCBranchArgs, FpRArgs,
};

/*

   _____ ____  _____  __
  / ____/ __ \|  __ \/_ |
 | |   | |  | | |__) || |
 | |   | |  | |  ___/ | |
 | |___| |__| | |     | |
  \_____\____/|_|     |_|



*/

// 0x03 - div.fmt

// 0x03.d - div.d
pub fn div_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);
    let _ = is_register_aligned(program_state, args.ft);

    let numerator: f64 = f64::from_bits(extract_u64(program_state, args.fs));
    let denominator: f64 = f64::from_bits(extract_u64(program_state, args.ft));

    let result: f64 = 
        perform_op_with_flush(program_state, {
            numerator
            / denominator 
    });

    pack_up_u64(program_state, args.fd, f64::to_bits(result));
}

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

// 0x06.d - mov.d
pub fn mov_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);

    let temp: u64 = extract_u64(program_state, args.fs);
    let _ = pack_up_u64(program_state, args.fd, temp);
}

// 0x08 (secondary funct code) - bc1<cond>
/// All implementations (t/f, likely/unlikely) are contained in this function.
/// This simplifies the table.
pub fn bc1(_program_state: &mut ProgramState, _args: FpCCBranchArgs) -> () {
    todo!("bc1<cond>");
}

// 0x32.d - c.eq.d
pub fn c_eq_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!();
}
