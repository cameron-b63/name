use crate::exception::register_set::to_register;
use crate::exception::registers::Register;
use crate::structs::Coprocessor0;
use paste::paste;
use std::ops::Range;

#[macro_export]
// The following macro allows us to enter just a couple values to generate an appropriately-named getter and setter for
// each bit field.
// will need more of these getters and setter later
macro_rules! getset {
    // This variant is for when the register is supplied.
    ($name:ident, $reg:expr, $range:expr) => {
        paste! {
            impl Coprocessor0 {
                pub fn [<get_ $name>](&self) -> u32 {
                    self.get_bit_field($reg, $range)
                }

                pub fn [<set_ $name>](&mut self, value: u32) {
                    self.set_bit_field($reg, $range, value)
                }
            }
        }
    };
    // And this variant is when the register shouldn't necessarily be supplied;
    ($name:ident, $start:expr, $end:expr) => {
        paste! {
            impl Coprocessor0 {
                pub fn [<get_ $name>](&self, register: usize) -> u32 {
                    self.get_bit_field(register, $start, $end)
                }

                pub fn [<set_ $name>](&mut self, register: usize, value: u32) {
                    self.set_bit_field(register, $start, $end, value)
                }
            }
        }
    };
}

/// Coprocessor 0 is organized with multiple fields inside any one 4-byte word. It would get old to remember where they all are.
/// For instance, bit 1 of register 12, select 0 (Status) represents the Exception Level.
/// The implementations below are used to perform bit-level accesses.
impl Coprocessor0 {
    // TODO: Implement EJTAG
    pub fn set_debug_mode(&mut self, thing: bool) {
        self.debug_mode = thing;
    }

    pub fn is_debug_mode(&self) -> bool {
        return self.debug_mode;
    }

    fn get_bit_field(&self, register: Register, range: Range<usize>) -> u32 {
        let reg = to_register(register);

        if range.end > 32 {
            panic!(
                "Improper values provided to get_bit_field (Start: {}, End: {})",
                range.start, range.end
            );
        }

        let num_bits = range.len();

        // If num_bits is 32, the mask should cover all 32 bits
        let mask = if num_bits == 32 {
            u32::MAX // All bits set
        } else {
            ((1u32 << num_bits) - 1) << range.start
        };

        // Apply mask and shift the bits back to the right
        (self.registers[reg] & mask) >> range.start
    }

    fn set_bit_field(&mut self, register: Register, range: Range<usize>, value: u32) {
        let reg = to_register(register);

        if range.end > 32 {
            panic!(
                "Improper values provided to set_bit_field (Start: {}, End: {})",
                range.start, range.end
            );
        }

        let num_bits = range.len();

        // If num_bits is 32, the mask should cover all 32 bits
        let mask = if num_bits >= 31 {
            u32::MAX
        } else {
            (1u32 << num_bits) - 1
        };

        // Clear the specified field
        self.registers[reg] &= !mask;

        // Set the specified field
        self.registers[reg] |= (value & mask) << range.start;
    }
}

// Below are all the macro-defined accessors for the small bit fields.
getset!(current_mode, Register::Status, 3..5);
getset!(exception_level, Register::Status, 0..1);
getset!(exc_code, Register::Cause, 2..7);
getset!(epc, Register::EPC, 0..32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bit_field() {
        let mut cop0 = Coprocessor0::default();
        cop0.set_bit_field(Register::Status, 0..32, u32::MAX);
        assert_eq!(cop0.registers[12], u32::MAX);
    }

    #[test]
    fn test_get_bit_field() {
        let mut cop0 = Coprocessor0::default();
        cop0.registers[12] |= 0xABAB << 5;
        assert_eq!(cop0.get_bit_field(Register::Status, 5..(5 + 4 * 4)), 0xABAB);
    }
}
