use crate::{
    exception::definitions::{ExceptionType, FpExceptionType},
    structs::ProgramState,
};

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

// Implement traits that allow generalizing operations across doubles and floats.
/// For our model, this is just f32 and f64.
pub trait FloatBits: Sized + Copy {
    type Bits: Copy;
    fn from_bits(value: Self::Bits) -> Self;
    fn extract_bits(program_state: &mut ProgramState, target: u32) -> Self::Bits;
    fn pack_bits(program_state: &mut ProgramState, target: u32, value: Self::Bits) -> ();
    fn extract_value(program_state: &mut ProgramState, target: u32) -> Self;
    fn pack_value(program_state: &mut ProgramState, destination: u32, value: Self);
}

impl FloatBits for f32 {
    type Bits = u32;
    fn from_bits(value: Self::Bits) -> Self {
        let _: u32 = value;
        f32::from_bits(value)
    }

    fn extract_bits(program_state: &mut ProgramState, target: u32) -> Self::Bits {
        program_state.cp1.registers[target as usize]
    }

    fn pack_bits(program_state: &mut ProgramState, destination: u32, value: Self::Bits) -> () {
        let _: u32 = value;
        program_state.cp1.registers[destination as usize] = value;
    }

    fn extract_value(program_state: &mut ProgramState, target: u32) -> Self {
        f32::from_bits(Self::extract_bits(program_state, target))
    }

    fn pack_value(program_state: &mut ProgramState, destination: u32, value: Self) {
        let _: f32 = value;
        Self::pack_bits(program_state, destination, value.to_bits())
    }
}

impl FloatBits for f64 {
    type Bits = u64;
    fn from_bits(value: Self::Bits) -> Self {
        let _: u64 = value;
        f64::from_bits(value)
    }

    /// The passed argument, target, should be the u32 repr of the high register (even register).
    /// For instance, extracting the double word inside $f2/$f3 means you should pass $f2's repr.
    fn extract_bits(program_state: &mut ProgramState, target: u32) -> Self::Bits {
        let _ = is_register_aligned(program_state, target);
        ((program_state.cp1.registers[target as usize] as u64) << 32)
            | (program_state.cp1.registers[target as usize + 1] as u64)
    }

    /// The target register should be the u32 repr of the high (even) register.
    /// For instance, extracting the double word inside $f2/$f3 means you should pass $f2's repr.
    fn pack_bits(program_state: &mut ProgramState, destination: u32, value: Self::Bits) -> () {
        let _ = is_register_aligned(program_state, destination);
        program_state.cp1.registers[destination as usize] = (value >> 32) as u32;
        program_state.cp1.registers[destination as usize + 1] = value as u32;
    }

    fn extract_value(program_state: &mut ProgramState, target: u32) -> Self {
        f64::from_bits(f64::extract_bits(program_state, target))
    }

    fn pack_value(program_state: &mut ProgramState, destination: u32, value: Self) {
        Self::pack_bits(program_state, destination, value.to_bits())
    }
}

/// There are various FPU rounding modes, and the user should be able to use whatever they want.
/// Reference MIPS Volume I-A, page 95, table 6.10
pub fn apply_fpu_rounding<T>(program_state: &mut ProgramState, value: T) -> T
where
    T: Roundable,
{
    match program_state.cp1.get_rounding_mode() {
        0b00 => {
            // nearest even
            value.round()
        }
        0b01 => {
            // toward zero
            value.trunc()
        }
        0b10 => {
            // toward plus inf
            value.ceil()
        }
        0b11 => {
            // toward minus inf
            value.floor()
        }
        _ => unreachable!(), // This is matching on a two-bit value. It's strictly enumerated.
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
        program_state.set_exception(ExceptionType::FloatingPoint(FpExceptionType::Underflow));
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

/// This trait facilitates generalizing FPU rounding.
/// It will make the apply_fpu_rounding<T>(T)->T function generic.
pub trait Roundable {
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn ceil(self) -> Self;
    fn floor(self) -> Self;
}

impl Roundable for f32 {
    fn round(self) -> Self {
        self.round()
    }

    fn trunc(self) -> Self {
        self.trunc()
    }

    fn ceil(self) -> Self {
        self.ceil()
    }

    fn floor(self) -> Self {
        self.floor()
    }
}

impl Roundable for f64 {
    fn round(self) -> Self {
        self.round()
    }

    fn trunc(self) -> Self {
        self.trunc()
    }

    fn ceil(self) -> Self {
        self.ceil()
    }

    fn floor(self) -> Self {
        self.floor()
    }
}

/// Trait to make functions like is_nan() usable in generalized comparisons.
pub trait FloatComparable: PartialEq + PartialOrd + FloatBits {
    fn is_nan(self) -> bool;
    fn is_signaling_nan(value: Self) -> bool;
}

// Please do not mind my evil floating-point bit hacks - consult IEEE 754-2008.

impl FloatComparable for f32 {
    fn is_nan(self) -> bool {
        f32::is_nan(self)
    }

    fn is_signaling_nan(value: Self) -> bool {
        let bits = value.to_bits();
        let exp = (bits >> 23) & 0xFF;
        let frac = bits & 0x7FFFFF;

        // Exponent all ones, fraction nonzero
        if exp == 0xFF && frac != 0 {
            let quiet_bit = (frac >> 22) & 1;
            // signaling NaN has quiet_bit == 0
            quiet_bit == 0
        } else {
            false
        }
    }
}

impl FloatComparable for f64 {
    fn is_nan(self) -> bool {
        f64::is_nan(self)
    }

    fn is_signaling_nan(value: Self) -> bool {
        let bits = value.to_bits();
        let exp = (bits >> 52) & 0x7FF;
        let frac = bits & 0x000F_FFFF_FFFF_FFFF;

        if exp == 0x7FF && frac != 0 {
            let quiet_bit = (frac >> 51) & 1;
            quiet_bit == 0
        } else {
            false
        }
    }
}
