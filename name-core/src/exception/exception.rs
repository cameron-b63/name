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
    pub fn set_fp_exception(&mut self, fp_exception_type: FpExceptionType) {
        let current_fcsr_exception_state = self.get_fexr();
        self.set_fexr(current_fcsr_exception_state | fp_exception_type as u32);
    }

    pub fn clear_fp_exception(&mut self) {}
}
