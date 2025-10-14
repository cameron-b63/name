// This file defines the SignMagnitudeLong and SignMagnitudeWord structs.
// These structs are used to make working with the .l and .w floating-point
// formats more readable in implementation.

use crate::instruction::implementation_helpers::FloatBits;

/// SignMagnitudeLong refers to the .l format. It consists of one sign bit, and 63 value bits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignMagnitudeLong {
    /// To be very clear: sign == true implies NEGATIVE number.
    pub sign: bool,
    pub magnitude: u64,
}

impl SignMagnitudeLong {
    /// Encode into raw bits for packing into registers
    pub fn to_bits(&self) -> u64 {
        let sign_bit = (self.sign as u64) << 63;

        sign_bit | self.magnitude & 0x7FFF_FFFF_FFFF_FFFF
    }

    /// Turn raw bits into a self
    pub fn from_bits(bits: u64) -> Self {
        let sign = (bits >> 63) != 0;
        let magnitude = bits & 0x7FFF_FFFF_FFFF_FFFF;

        Self { sign, magnitude }
    }

    /// Return a boolean respresenting whether SignMagnitudeLong can represent the integer part of this double value.
    pub fn can_represent_double(value: f64) -> bool {
        (value.trunc() as u64) < !0x7FFF_FFFF_FFFF_FFFF
    }

    /// Return a boolean respresenting whether SignMagnitudeLong can represent the integer part of this single value.
    pub fn can_represent_single(value: f32) -> bool {
        (value.trunc() as u64) < !0x7FFF_FFFF
    }
}

impl From<f64> for SignMagnitudeLong {
    /// Convert a ROUNDED f64 (value.fract() == 0) to a SignMagnitudeLong.
    /// Note that rounding is the BURDEN OF THE CALLER.
    /// If you think this isn't working,
    /// you should CONFIRM that the CALLER is ROUNDING.
    fn from(value: f64) -> Self {
        if value.is_nan() {
            return Self {
                sign: false,
                magnitude: 0,
            };
        }

        let sign = value.is_sign_negative();
        // Rounding for magnitude is burden of the caller!
        let magnitude = value.abs() as u64;

        Self { sign, magnitude }
    }
}

impl From<SignMagnitudeLong> for i64 {
    fn from(value: SignMagnitudeLong) -> Self {
        if value.sign {
            -(value.magnitude as i64)
        } else {
            value.magnitude as i64
        }
    }
}

impl From<i64> for SignMagnitudeLong {
    fn from(value: i64) -> Self {
        Self::from_bits(value as u64)
    }
}

// Implement FloatBits for consistent movement to/from registers
impl FloatBits for SignMagnitudeLong {
    type Bits = u64;

    fn from_bits(value: Self::Bits) -> Self {
        Self::from_bits(value)
    }

    fn extract_bits(program_state: &mut crate::structs::ProgramState, target: u32) -> Self::Bits {
        f64::extract_bits(program_state, target)
    }

    fn extract_value(program_state: &mut crate::structs::ProgramState, target: u32) -> Self {
        Self::from_bits(Self::extract_bits(program_state, target))
    }

    fn pack_bits(
        program_state: &mut crate::structs::ProgramState,
        target: u32,
        value: Self::Bits,
    ) -> () {
        f64::pack_bits(program_state, target, value)
    }

    fn pack_value(program_state: &mut crate::structs::ProgramState, destination: u32, value: Self) {
        Self::pack_bits(program_state, destination, Self::to_bits(&value))
    }
}

/// SignMagnitudeWord refers to the .w format. It consists of 1 sign bit, and 31 value bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignMagnitudeWord {
    /// To be very, very clear: sign == true -> NEGATIVE number.
    pub sign: bool,
    pub magnitude: u32,
}

impl SignMagnitudeWord {
    /// Encode into raw bits for packing into registers
    pub fn to_bits(&self) -> u32 {
        let sign_bit = (self.sign as u32) << 31;
        sign_bit | self.magnitude & 0x7FFF_FFFF
    }

    /// Turn raw bits into self
    pub fn from_bits(bits: u32) -> Self {
        let sign = (bits >> 31) != 0;
        let magnitude = bits & 0x7FFF_FFFF;

        Self { sign, magnitude }
    }

    /// Return a boolean representing whether SignMagnitudeWord is capable of representing this double value.
    pub fn can_represent_double(value: f64) -> bool {
        Self::can_represent_long(SignMagnitudeLong::from(value))
    }

    /// Return a boolean representing whether SignMagnitudeWord is capable of representing this long value.
    pub fn can_represent_long(value: SignMagnitudeLong) -> bool {
        value.magnitude < !0x80000000
    }

    /// Return a boolean representing whether SignMagnitudeWord is capable of representing this single value.
    pub fn can_represent_single(value: f32) -> bool {
        (value.trunc() as u32) < !0x7FFF_FFFF
    }
}

impl From<f32> for SignMagnitudeWord {
    /// Convert a ROUNDED f32 (value.fract() == 0) to a SignMagnitudeWord.
    /// Note that rounding is the BURDEN OF THE CALLER.
    /// If you think this isn't working,
    /// you should CONFIRM that the CALLER is ROUNDING.
    fn from(value: f32) -> Self {
        if value.is_nan() {
            return Self {
                sign: false,
                magnitude: 0,
            };
        }

        let sign = value.is_sign_negative();
        // Rounding for magnitude is burden of the caller!
        let magnitude = value.abs() as u32;

        Self { sign, magnitude }
    }
}

impl From<SignMagnitudeWord> for i32 {
    fn from(value: SignMagnitudeWord) -> Self {
        if value.sign {
            -(value.magnitude as i32)
        } else {
            value.magnitude as i32
        }
    }
}

impl From<i32> for SignMagnitudeWord {
    fn from(value: i32) -> Self {
        Self::from_bits(value as u32)
    }
}

// Implement FloatBits for consistent movement to/from registers
impl FloatBits for SignMagnitudeWord {
    type Bits = u32;

    fn from_bits(value: Self::Bits) -> Self {
        Self::from_bits(value)
    }

    fn extract_bits(program_state: &mut crate::structs::ProgramState, target: u32) -> Self::Bits {
        f32::extract_bits(program_state, target)
    }

    fn extract_value(program_state: &mut crate::structs::ProgramState, target: u32) -> Self {
        Self::from_bits(Self::extract_bits(program_state, target))
    }

    fn pack_bits(
        program_state: &mut crate::structs::ProgramState,
        target: u32,
        value: Self::Bits,
    ) -> () {
        f32::pack_bits(program_state, target, value)
    }

    fn pack_value(program_state: &mut crate::structs::ProgramState, destination: u32, value: Self) {
        Self::pack_bits(program_state, destination, Self::to_bits(&value))
    }
}
