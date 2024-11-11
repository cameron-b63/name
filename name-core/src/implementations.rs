use crate::constants::REGISTERS;
use crate::exception::constants::*;
use crate::structs::{Coprocessor0, LineInfo, Memory, Processor, ProgramState};
// use crate::debug::{
//     // debug_utils::*,
//     debugger_methods::*,
// };

impl Processor {
    pub fn new(entry: u32) -> Self {
        Processor {
            pc: entry,
            general_purpose_registers: [0; 32],
        }
    }
}

// TODO: Fill any default values for cp0 fields
impl Coprocessor0 {
    pub fn new() -> Self {
        Coprocessor0 { registers: [0; 32] }
    }
}

impl ProgramState {
    pub fn new(cpu: Processor, memory: Memory) -> Self {
        ProgramState {
            should_continue_execution: true,
            cpu: cpu,
            cp0: Coprocessor0::new(),
            memory: memory,
        }
    }

    pub fn is_exception(&self) -> bool {
        return self.cp0.get_exception_level() == EXCEPTION_BEING_HANDLED;
    }

    pub fn insert_breakpoint(&mut self, address: u32, bp_num: usize) -> Result<u32, String> {
        // least vulnerable code ever

        if !self.memory.allows_execution_of(address){
            return Err(format!(" - Address 0x{:x} is out of bounds.", address));
        }

        // get offset from address so we can manipulate the data in memory

        let mut old_instruction_word: [u8; 4] = [0; 4];

        // craft the break instruction (i.e. stick what we need to into `code`)
        // TODO: for the love of God make this fit more with the codebase
        let break_inst: u32 = ((bp_num as u32) >> 6) | 0b001101;

        // stick the crafted break instruction into memory
        // and in the process, grab the old instruction
        for i in 0..4 {
            // this looks like gobbledygook, but here's what it's doing:
            // take the last 8 bits in the break instruction.
            // make that into a byte and store it in the data.
            // shift the instruction to the right so that we can take the second-to-last byte.
            // so on so forth
            let break_inst_byte: u8 = ((break_inst >> (24 - 8*i)) & 0x000000ff) as u8;
            // nab the original instruction that was there before to be returned
            old_instruction_word[i] = match self.memory.read_byte(address + 8*i as u32) {
                Ok(byte) => byte,
                Err(e) => { return Err(format!("{e}")); }
            }; 
            // replace it with the break instruction 
            match self.memory.set_byte(address + 8*i as u32, break_inst_byte) {
                Ok(_) => continue,
                Err(e) => { return Err(format!("{e}")); }
            };
        }

        let mut old_instruction: u32 = 0;
        for i in 0..4 {
            old_instruction |= (old_instruction_word[i] << (24 - 8*i)) as u32;
        }
        
        Ok(old_instruction)
    }

// "pa"
    pub fn print_all_registers(
        &mut self,
        db_args: &Vec<String>,
    ) -> Result<(), String> {
        if db_args.len() > 1 {
            // this outputs a lot so make sure the user actually meant to type pa and not pb or p or something
            // made it > so we can use this function to do register_dump()
            return Err(format!(
                "pa expects 0 arguments, received {}",
                db_args.len() - 1
            ));
        }

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

impl LineInfo {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.content.as_bytes().to_vec();
        bytes.push(b'\0');

        bytes.extend_from_slice(&self.line_number.to_be_bytes());
        bytes.extend_from_slice(&self.start_address.to_be_bytes());
        bytes.extend_from_slice(&self.end_address.to_be_bytes());

        bytes
    }
}
