use super::{constants::*, definitions::ExceptionType};
use crate::{
    exception::definitions::FpExceptionType,
    structs::{Coprocessor1, ProgramState},
};

impl ProgramState {
    /// When an exception is triggered in the MIPS architecture,
    /// values in Coprocessor 0 registers are set to indicate the exception state to the operating system.
    /// This impl modifies the ProgramState based on a passed ExceptionType,
    /// filling in the Status (12) and Cause (13) registers appropriately.
    pub fn set_exception(&mut self, exception_type: ExceptionType) -> () {
        // Some values are set no matter what to indicate an exception state:
        //
        // Exceptions are handled in Kernel Mode.
        self.cp0.set_current_mode(KERNEL_MODE);
        // The EXL bit indicates to the OS that an exception is being handled.
        // The EPC register contains the PC of where the exception occurred.
        // If it already contains some other value important to our flow, we do not want to overwrite the address.
        if !self.is_exception() {
            self.cp0.set_epc(self.cpu.pc - 4);
        }
        // Set the EXL bit.
        self.cp0.set_exception_level(EXCEPTION_BEING_HANDLED);
        // Set the ExcCode field of Cause to the proper value
        self.cp0.set_exc_code(u32::from(exception_type));

        // If the exception is floating-point, populate the FCSR fields in Coprocessor 1.
        if let ExceptionType::FloatingPoint(fp_exception_type) = exception_type.clone() {
            self.cp1.set_fp_exception(fp_exception_type);
        }
    }

    /// When an exception was handled without needing to halt, Coprocessor 0 is reset to indicate normal operation.
    pub fn recover_from_exception(&mut self) -> () {
        // Unset the EXL bit to indicate an exception is no longer being handled
        self.cp0.set_exception_level(NO_EXCEPTION);
        // TODO: LEAVE KERNEL MODE
        // Go back to where we were headed before the exception was handled
        self.cpu.pc = self.cp0.get_epc() + 4;
        // Clear EPC
        self.cp0.set_epc(0u32);
        // Clear FPU exception bits
        self.cp1.clear_fp_exception();
    }
}

impl Coprocessor1 {
    /// Set the FEXR register according to the specified exception.
    pub fn set_fp_exception(&mut self, fp_exception_type: FpExceptionType) {
        let current_fcsr_exception_state = self.get_fexr();
        self.set_fexr(current_fcsr_exception_state | fp_exception_type as u32);
    }

    /// Clear all floating-point exceptions.
    pub fn clear_fp_exception(&mut self) {
        self.set_fexr(0u32);
    }

    /// Retrieve all floating-point exceptions that have occurred.
    /// Check both Cause and Flags; the exception handler will check if
    /// explicit traps are enabled.
    pub fn get_floating_point_errors(&self) -> Vec<FpExceptionType> {
        // The following bitmasks are coming straight from FCSR in constants.rs;
        // If you've been keeping track of the fp exception handling code,
        // this follows EVZOUI in FEXR.
        let fexr = self.get_fexr();
        let mut res: Vec<FpExceptionType> = Vec::new();

        // Just so you know: FEXR bitmask is    0b0000_0000_0000_0011_1111_0000_0111_1100

        // Unimplemented Operation
        if (fexr & 0b0000_0000_0000_0010_0000_0000_0000_0000) != 0 {
            res.push(FpExceptionType::UnimplementedOperation);
        }
        // Invalid Operation
        if (fexr & 0b0000_0000_0000_0001_0000_0000_0100_0000) != 0 {
            res.push(FpExceptionType::InvalidOperation);
        }

        // Divide By Zero
        if (fexr & 0b0000_0000_0000_0000_1000_0000_0010_0000) != 0 {
            res.push(FpExceptionType::DivideByZero);
        }

        // Overflow
        if (fexr & 0b0000_0000_0000_0000_0100_0000_0001_0000) != 0 {
            res.push(FpExceptionType::Overflow);
        }

        // Underflow
        if (fexr & 0b0000_0000_0000_0000_0010_0000_0000_1000) != 0 {
            res.push(FpExceptionType::Underflow);
        }

        // Inexact
        if (fexr & 0b0000_0000_0000_0000_0001_0000_0000_0100) != 0 {
            res.push(FpExceptionType::Inexact);
        }

        res
    }
}
