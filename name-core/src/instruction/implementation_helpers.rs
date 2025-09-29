use crate::{exception::definitions::ExceptionType, structs::ProgramState};

/// Helper function for instructions that operate on register pairs to remove invalid cases.
/// If the register is improperly aligned given current program state, it will trigger
/// an exception in Coprocessor 0.
pub fn is_register_aligned(program_state: &mut ProgramState, reg: u32) -> bool {
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
pub fn extract_u64(program_state: &mut ProgramState, target: u32) -> u64 {
    ((program_state.cp1.registers[target as usize] as u64) << 32)
        | (program_state.cp1.registers[target as usize + 1] as u64)
}

/// This is a helper function for implementations that packs a u64 double-word value
/// back into a register pair given by target.
/// The target register should be the u32 repr of the high (even) register.
/// For instance, extracting the double word inside $f2/$f3 means you should pass $f2's repr.
pub fn pack_up_u64(program_state: &mut ProgramState, target: u32, value: u64) {
    program_state.cp1.registers[target as usize] = f32::from_bits((value >> 32) as u32);
    program_state.cp1.registers[target as usize + 1] = f32::from_bits(value as u32);
}

/// This is a helper function which takes the bitwise representation (u64) of a sign/magnitude long
/// and converts it to the corresponding i64 signed value.
pub fn u64_to_long(long_bits: u64) -> i64 {
    let bitmask = 1 << 63;
    if (long_bits & bitmask) == 0 {
        return long_bits as i64;
    } else {
        return -((long_bits & !bitmask) as i64);
    }
}

/// This is a helper function which takes a signed (i64) value and converts it to
/// the corresponding sign/magnitude long bits (u64).
/// note: potentially lossy.
pub fn i64_to_long_bits(long_value: i64) -> u64 {
    let sign_bit = 1 << 63;
    if long_value > 0 {
        return (long_value as u64) & !sign_bit;
    } else {
        return ((long_value as u64) & !sign_bit) | sign_bit;
    }
}

/// This is a helper function which takes the bitwise representation (u32) of a sign/magnitude word
/// and converts it to the corresponding i32 signed value.
pub fn u32_to_word(word_bits: u32) -> i32 {
    let bitmask = 1 << 31;
    if (word_bits & bitmask) == 0 {
        return word_bits as i32;
    } else {
        return -((word_bits & !bitmask) as i32);
    }
}

/// Some operations will have subnormal results. This essentially means they're super tiny.
/// For details, see the definition of FCSR (FS bit).
/// This function facilitates the subnormal stuff.
pub fn perform_op_with_flush<T>(program_state: &mut ProgramState, result: T) -> T
where
    T: FloatFlush + Copy,
{
    if program_state.cp1.fenr_fs_bit_set() && result.is_subnormal() {
        T::flush_zero_with_sign(result)
    } else {
        result
    }
}

/// This trait facilitates generalizing the subnormal arithmetic issue (see docs).
/// It should be implemented for both f32 and f64.
pub trait FloatFlush {
    fn is_subnormal(self) -> bool;
    fn flush_zero_with_sign(self) -> Self;
}

impl FloatFlush for f32 {
    fn is_subnormal(self) -> bool {
        self.classify() == std::num::FpCategory::Subnormal
    }

    fn flush_zero_with_sign(self) -> Self {
        0.0_f32.copysign(self)
    }
}

impl FloatFlush for f64 {
    fn is_subnormal(self) -> bool {
        self.classify() == std::num::FpCategory::Subnormal
    }

    fn flush_zero_with_sign(self) -> Self {
        0.0_f64.copysign(self)
    }
}
