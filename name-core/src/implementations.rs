use crate::constants::{
    fpu_control::{FCSR_DEFAULT_VALUES, FIR_DEFAULT_VALUES},
    {MIPS_TEXT_START_ADDR, REGISTERS},
};
use crate::exception::{definitions::ExceptionType, register_set::Cp0Register};
use crate::structs::{Coprocessor0, Coprocessor1, Processor, ProgramState, /*, OperatingSystem*/};

const CP0_REGISTER_COUNT: usize = 32;
const CP0_SELECT_COUNT: usize = 8;
const CP0_STATUS_EXL_BIT: u32 = 1 << 1;
const CP0_STATUS_ERL_BIT: u32 = 1 << 2;
const CP0_STATUS_KSU_SHIFT: u32 = 3;
const CP0_STATUS_KSU_MASK: u32 = 0b11 << CP0_STATUS_KSU_SHIFT;
const CP0_CAUSE_EXC_CODE_SHIFT: u32 = 2;
const CP0_CAUSE_EXC_CODE_MASK: u32 = 0x1f << CP0_CAUSE_EXC_CODE_SHIFT;

impl Default for Processor {
    fn default() -> Self {
        Self {
            pc: MIPS_TEXT_START_ADDR,
            general_purpose_registers: [0u32; 32],
            hi: 0,
            lo: 0,
        }
    }
}

impl Processor {
    pub fn new(entry: u32) -> Self {
        Processor {
            pc: entry,
            general_purpose_registers: [0; 32],
            hi: 0,
            lo: 0,
        }
    }
}

// TODO: Fill any default values for cp0 fields
impl Coprocessor0 {
    pub fn new() -> Self {
        let mut cp0 = Coprocessor0 {
            registers: [[0; CP0_SELECT_COUNT]; CP0_REGISTER_COUNT],
            debug_mode: false,
        };

        for register in [
            Cp0Register::BadVAddr,
            Cp0Register::Count,
            Cp0Register::Compare,
            Cp0Register::Status,
            Cp0Register::Cause,
            Cp0Register::EPC,
            Cp0Register::PRId,
            Cp0Register::Debug,
            Cp0Register::DEPC,
            Cp0Register::ErrorEPC,
        ] {
            let (rd, sel) = register.index();
            cp0.registers[rd][sel] = register.reset_value();
        }

        cp0
    }

    fn validate_index(rd: usize, sel: usize) -> Result<(), ExceptionType> {
        if rd < CP0_REGISTER_COUNT && sel < CP0_SELECT_COUNT {
            Ok(())
        } else {
            Err(ExceptionType::ReservedInstruction)
        }
    }

    fn resolve_register(rd: usize, sel: usize) -> Result<Cp0Register, ExceptionType> {
        Self::validate_index(rd, sel)?;
        Cp0Register::from_index(rd, sel).ok_or(ExceptionType::ReservedInstruction)
    }

    pub fn get_cp0_register(&self, rd: usize, sel: usize) -> Result<u32, ExceptionType> {
        Self::resolve_register(rd, sel)?;
        Ok(self.registers[rd][sel])
    }

    pub fn set_cp0_register(
        &mut self,
        rd: usize,
        sel: usize,
        value: u32,
    ) -> Result<(), ExceptionType> {
        let register = Self::resolve_register(rd, sel)?;
        let write_mask = register.write_mask();
        let preserved = self.registers[rd][sel] & !write_mask;
        self.registers[rd][sel] = preserved | (value & write_mask);

        if register == Cp0Register::Compare {
            self.clear_cause_ip7();
        }

        Ok(())
    }

    fn get_named_register(&self, register: Cp0Register) -> u32 {
        let (rd, sel) = register.index();
        self.registers[rd][sel]
    }

    fn set_named_register(&mut self, register: Cp0Register, value: u32) {
        let (rd, sel) = register.index();
        let write_mask = register.write_mask();
        let preserved = self.registers[rd][sel] & !write_mask;
        self.registers[rd][sel] = preserved | (value & write_mask);
    }

    pub fn get_status(&self) -> u32 {
        self.get_named_register(Cp0Register::Status)
    }

    pub fn set_status(&mut self, value: u32) {
        self.set_named_register(Cp0Register::Status, value);
    }

    pub fn get_cause(&self) -> u32 {
        self.get_named_register(Cp0Register::Cause)
    }

    pub fn set_cause(&mut self, value: u32) {
        self.set_named_register(Cp0Register::Cause, value);
    }

    pub fn get_epc(&self) -> u32 {
        self.get_named_register(Cp0Register::EPC)
    }

    pub fn set_epc(&mut self, value: u32) {
        self.set_named_register(Cp0Register::EPC, value);
    }

    pub fn get_depc(&self) -> u32 {
        self.get_named_register(Cp0Register::DEPC)
    }

    pub fn set_depc(&mut self, value: u32) {
        self.set_named_register(Cp0Register::DEPC, value);
    }

    pub fn get_error_epc(&self) -> u32 {
        self.get_named_register(Cp0Register::ErrorEPC)
    }

    pub fn set_error_epc(&mut self, value: u32) {
        self.set_named_register(Cp0Register::ErrorEPC, value);
    }

    pub fn get_exception_level(&self) -> u32 {
        ((self.get_status() & CP0_STATUS_EXL_BIT) != 0) as u32
    }

    pub fn set_exception_level(&mut self, value: u32) {
        let status = (self.get_status() & !CP0_STATUS_EXL_BIT) | ((value & 0x1) << 1);
        self.set_status(status);
    }

    pub fn get_error_level(&self) -> u32 {
        ((self.get_status() & CP0_STATUS_ERL_BIT) != 0) as u32
    }

    pub fn set_error_level(&mut self, value: u32) {
        let status = (self.get_status() & !CP0_STATUS_ERL_BIT) | ((value & 0x1) << 2);
        self.set_status(status);
    }

    pub fn get_current_mode(&self) -> u32 {
        (self.get_status() & CP0_STATUS_KSU_MASK) >> CP0_STATUS_KSU_SHIFT
    }

    pub fn set_current_mode(&mut self, value: u32) {
        let status = (self.get_status() & !CP0_STATUS_KSU_MASK)
            | ((value & 0b11) << CP0_STATUS_KSU_SHIFT);
        self.set_status(status);
    }

    pub fn get_exc_code(&self) -> u32 {
        (self.get_cause() & CP0_CAUSE_EXC_CODE_MASK) >> CP0_CAUSE_EXC_CODE_SHIFT
    }

    pub fn set_exc_code(&mut self, value: u32) {
        let cause = (self.get_cause() & !CP0_CAUSE_EXC_CODE_MASK)
            | ((value & 0x1f) << CP0_CAUSE_EXC_CODE_SHIFT);
        let (rd, sel) = Cp0Register::Cause.index();
        self.registers[rd][sel] = cause;
    }

    fn clear_cause_ip7(&mut self) {
        let cause = self.get_cause() & !(1 << 15);
        self.set_cause(cause);
    }

    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp0_rejects_unmapped_register_pairs() {
        let cp0 = Coprocessor0::new();

        assert!(matches!(
            cp0.get_cp0_register(20, 0),
            Err(ExceptionType::ReservedInstruction)
        ));
    }

    #[test]
    fn cp0_compare_write_clears_timer_interrupt_pending() {
        let mut cp0 = Coprocessor0::new();
        cp0.set_cause(1 << 15);

        assert!(matches!(cp0.set_cp0_register(11, 0, 0x1234), Ok(())));
        assert_eq!(cp0.get_cause() & (1 << 15), 0);
    }

    #[test]
    fn cp0_tracks_error_level_bit() {
        let mut cp0 = Coprocessor0::new();

        cp0.set_error_level(1);
        assert_eq!(cp0.get_error_level(), 1);

        cp0.set_error_level(0);
        assert_eq!(cp0.get_error_level(), 0);
    }

    #[test]
    fn cp0_exc_code_round_trips_for_internal_exception_updates() {
        let mut cp0 = Coprocessor0::new();

        cp0.set_exc_code(8);
        assert_eq!(cp0.get_exc_code(), 8);
    }
}

impl Coprocessor1 {
    pub fn new() -> Self {
        Coprocessor1 {
            registers: [0; 32],
            control_registers: [FIR_DEFAULT_VALUES, FCSR_DEFAULT_VALUES],
        }
    }
}

impl ProgramState {
    pub fn insert_breakpoint(&mut self, address: u32, bp_num: usize) -> Result<u32, String> {
        // least vulnerable code ever

        if !self.memory.allows_execution_of(address) {
            return Err(format!(" - Address 0x{:x} is out of bounds.", address));
        }

        // get offset from address so we can manipulate the data in memory

        let mut old_instruction_word: [u8; 4] = [0; 4];

        // craft the break instruction (i.e. stick what we need to into `code`)
        // TODO: for the love of God make this fit more with the codebase
        let break_inst: u32 = ((bp_num as u32) << 6) | 0b001101;

        // stick the crafted break instruction into memory
        // and in the process, grab the old instruction
        for i in 0..4 {
            // this looks like gobbledygook, but here's what it's doing:
            // take the last 8 bits in the break instruction.
            // make that into a byte and store it in the data.
            // shift the instruction to the right so that we can take the second-to-last byte.
            // so on so forth
            let break_inst_byte: u8 = (break_inst >> (24 - 8 * i)) as u8;
            // nab the original instruction that was there before to be returned
            old_instruction_word[i] = match self.memory.read_byte(address + i as u32) {
                Ok(byte) => byte,
                Err(e) => {
                    return Err(format!("{e}"));
                }
            };
            // replace it with the break instruction
            match self.memory.set_byte(address + i as u32, break_inst_byte) {
                Ok(_) => continue,
                Err(e) => {
                    return Err(format!("{e}"));
                }
            };
        }

        let mut old_instruction: u32 = 0;
        for i in 0..4 {
            old_instruction |= ((old_instruction_word[i] as u32) << (24 - 8 * i)) as u32;
            // println!("{:x}", old_instruction_word[i]);
        }

        Ok(old_instruction)
    }

    /// Prints the values of all registers at once. Invoked by "pa" in the CLI.
    pub fn print_all_registers(&mut self, db_args: &Vec<String>) -> Result<(), String> {
        if db_args.len() > 1 {
            // this outputs a lot so make sure the user actually meant to type pa and not pb or p or something
            // made it > so we can use this function to do register_dump()
            return Err(format!(
                "pa expects 0 arguments, received {}",
                db_args.len() - 1
            ));
        }

        println!("{:>5}: {:08x}", "$pc", self.cpu.pc);

        // for register in Register.values() {
        for register in REGISTERS {
            // change this to loop through the enum in name-core::structs instead?
            let idx: usize = REGISTERS.iter().position(|&x| x == register).unwrap();
            println!(
                "{:>5}: {:08x}",
                register,
                self.cpu.general_purpose_registers[idx] // register, program_state.cpu.general_purpose_registers[register]
            );
        }
        Ok(())
    }

    pub fn register_dump(&mut self) {
        match self.print_all_registers(&Vec::new()) {
            Ok(_) => {}
            Err(e) => eprintln!("{e}"),
        };
    }
}
