/// These are the key structs on which NAME operates along with their associated implementations.
/// It's gonna be quite a few definitions, so buckle up.
use std::{
    fmt,
    io::{/*self,*/ stdin, stdout, Stdin, Stdout, Write},
};

use crate::{
    constants::{
        MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_HEAP_START_ADDR, MIPS_STACK_END_ADDR,
        MIPS_TEXT_START_ADDR,
    }, 
    debug::{debug_utils::*, debugger_methods::* /* implementations::* */}, exception::constants::EXCEPTION_BEING_HANDLED, syscalls::*,
    dbprint,
};

/// Symbol is used for assembly -> ELF, ET_REL -> ET_EXEC, and ELF -> ProgramState construction.
/// Its definition is provided in the ELF TIS: https://refspecs.linuxfoundation.org/elf/elf.pdf
#[derive(Debug)]
pub struct Symbol {
    pub symbol_type: u8,
    pub identifier: String,
    pub value: u32,
    pub size: u32,
    pub visibility: Visibility,
    pub section: Section,
}

/// The processor holds the general-purpose registers along with $sp, $gp, etc.
#[derive(Debug)]
pub struct Processor {
    pub pc: u32,
    pub general_purpose_registers: [u32; 32],
}

/// Coprocessor 0 is for communication with the OS. Look in name-core/exception for more.
#[derive(Debug, Default)]
pub struct Coprocessor0 {
    pub registers: [u32; 32],
    pub debug_mode: bool, // TODO: implement EJTAG
}

/// Memory is a conglomerate of program text, program data, the heap, the stack, and other segments.
/// There exist predefined offsets for each of these segments in 32-bit MIPS:
///  - reserved space from 0x00000000 to 0x3fffffff;
///  - section .text begins at 0x40000000 in memory;
///  - section .data begins at 0x10010000 in memory;
///  - heap begins at 0x10040000 in memory;
///  - stack begins at 0x7ffffe00 in memory (and grows downward);
///  - kernel data begins at 0x90000000 in memory;
///  - mem-mapped I/O begins at 0xffff0000 in memory.
/// The Memory struct relies on address translation for proper use. Each segment is represented as a Vec<u8>.
#[derive(Debug)]
pub struct Memory {
    pub text: Vec<u8>,
    pub data: Vec<u8>,
    pub heap: Vec<u8>,
    pub stack: Vec<u8>,
    // These fields may be used later:
    // pub kernel: Vec<u8>,
    // pub mmio: Vec<u8>,
}

/// Error type for reading from memory / address translation
#[derive(Debug)]
pub enum MemoryError {
    TextTranslationError,
    DataTranslationError,
    HeapTranslationError,
    StackTranslationError,
    ReservedSpaceReferenced,
}

/// Pretty print for said errors
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::TextTranslationError => write!(f, "Address translation error occurred: Detected an address in .text"),
            MemoryError::DataTranslationError => write!(f, "Address translation error occurred: Detected an address in .data"),
            MemoryError::HeapTranslationError => write!(f, "Address translation error occurred: Detected a heap address"),
            MemoryError::StackTranslationError => write!(f, "Address translation error occurred: Detected a stack address (did you decrement $sp?)"),
            MemoryError::ReservedSpaceReferenced => write!(f, "Attempted to reference reserved space; read not permitted"),
        }
    }
}

/// Default constructor just creates empty segments.
impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: Vec::new(),
            text: Vec::new(),
            heap: Vec::new(),
            stack: Vec::new(),
        }
    }
}

/// The heavy lifting of the Memory struct comes in the impl block in the form of address translation.
impl Memory {
    /// Simple constructor - makes blank heap and stack
    pub fn new(data: Vec<u8>, text: Vec<u8>) -> Self {
        Memory {
            text: text,
            data: data,
            heap: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// The burden of alignment checking rests on each read_<type> function.
    /// read_byte reads a byte, performing address translation.
    pub fn read_byte(&self, address: u32) -> Result<u8, MemoryError> {
        // Obtain values for segment boundaries:
        let text_end: u32 = MIPS_TEXT_START_ADDR + self.text.len() as u32;
        let data_end: u32 = MIPS_DATA_START_ADDR + self.data.len() as u32;
        let heap_end: u32 = MIPS_HEAP_START_ADDR + self.heap.len() as u32; // This one MUST be calculated on the fly
        let stack_start: u32 = MIPS_STACK_END_ADDR - self.stack.len() as u32; // Similarly, this must be calculated on the fly
                                                                              // Match on the address to find the correct segment to read from and ensure the offset is within proper bounds of segment
        match address {
            addr if MIPS_TEXT_START_ADDR <= addr && addr <= text_end => {
                // This pipeline either returns the obtained byte or an appropriate error.
                // It works the same way for every match arm.
                return self
                    .text
                    .get((address - MIPS_TEXT_START_ADDR) as usize)
                    .copied()
                    .ok_or_else(|| MemoryError::TextTranslationError);
            }
            addr if MIPS_DATA_START_ADDR <= addr && addr <= data_end => {
                return self
                    .data
                    .get((address - MIPS_DATA_START_ADDR) as usize)
                    .copied()
                    .ok_or_else(|| MemoryError::DataTranslationError);
            }
            addr if MIPS_HEAP_START_ADDR <= addr && addr <= heap_end => {
                return self
                    .heap
                    .get((address - MIPS_HEAP_START_ADDR) as usize)
                    .copied()
                    .ok_or_else(|| MemoryError::HeapTranslationError);
            }
            // Note that the stack is most likely to be error-prone as it works differently. (i.e. look here first)
            addr if stack_start <= addr && addr <= MIPS_STACK_END_ADDR => {
                return self
                    .stack
                    .get((MIPS_STACK_END_ADDR - address) as usize)
                    .copied()
                    .ok_or_else(|| MemoryError::StackTranslationError);
            }
            // Other areas in memory are not yet necessary to match on but can be added quickly
            _ => {
                // If the address provided was out of bounds it should generate the following error:
                return Err(MemoryError::ReservedSpaceReferenced);
            }
        }
    }

    /// The burden of alignment checking rests on each set_<type> function.
    /// set_byte performs address translation on the provided address and sets the value at that address to value.
    pub fn set_byte(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        // Obtain values for segment boundaries:
        let text_end: u32 = MIPS_TEXT_START_ADDR + self.text.len() as u32;
        let data_end: u32 = MIPS_DATA_START_ADDR + self.data.len() as u32;
        let heap_end: u32 = MIPS_HEAP_START_ADDR + self.heap.len() as u32;
        let stack_start: u32 = MIPS_STACK_END_ADDR - self.stack.len() as u32;
        // Match on the address to find the correct segment to read from and ensure the offset is within proper bounds of segment
        match address {
            addr if MIPS_TEXT_START_ADDR <= addr && addr <= text_end => {
                // This offset check is redundant.
                let offset: usize = (address - MIPS_TEXT_START_ADDR) as usize;
                if offset < self.text.len() {
                    self.text[offset] = value;
                    return Ok(());
                } else {
                    return Err(MemoryError::TextTranslationError);
                }
            }
            addr if MIPS_DATA_START_ADDR <= addr && addr <= data_end => {
                let offset: usize = (address - MIPS_DATA_START_ADDR) as usize;
                if offset < self.data.len() {
                    self.data[offset] = value;
                    return Ok(());
                } else {
                    return Err(MemoryError::TextTranslationError);
                }
            }
            addr if MIPS_HEAP_START_ADDR <= addr && addr <= heap_end => {
                let offset: usize = (address - MIPS_HEAP_START_ADDR) as usize;
                if offset < self.heap.len() {
                    self.heap[offset] = value;
                    return Ok(());
                } else {
                    return Err(MemoryError::TextTranslationError);
                }
            }
            // Note that the stack is most likely to be error-prone as it works differently. (i.e. look here first)
            addr if stack_start <= addr && addr <= MIPS_STACK_END_ADDR => {
                let offset: usize = (MIPS_STACK_END_ADDR - address) as usize;
                if offset < self.stack.len() {
                    self.stack[offset] = value;
                    return Ok(());
                } else {
                    return Err(MemoryError::TextTranslationError);
                }
            }
            // Other areas in memory are not yet necessary to match on but can be added quickly
            _ => {
                // If the address provided was out of bounds it should generate the following error:
                return Err(MemoryError::ReservedSpaceReferenced);
            }
        }
    }

    /// This function checks that the provided address falls within a section that allows execution.
    pub fn allows_execution_of(&self, address: u32) -> bool {
        let text_end: u32 = MIPS_TEXT_START_ADDR + self.text.len() as u32;

        // This will require more sophisticated checks in the future when self-modifying code is optionally allowed.
        return MIPS_TEXT_START_ADDR <= address && address < text_end;
    }

    /// This function checks if the provided address can be written to.
    pub fn allows_write_to(&self, address: u32) -> bool {
        let data_end: u32 = MIPS_DATA_START_ADDR + self.text.len() as u32;
        let heap_end: u32 = MIPS_HEAP_START_ADDR + self.heap.len() as u32;
        let stack_start: u32 = MIPS_STACK_END_ADDR - self.stack.len() as u32;

        return (MIPS_DATA_START_ADDR <= address && address < data_end)
            || (MIPS_HEAP_START_ADDR <= address && address < heap_end)
            || (stack_start <= address && address < MIPS_STACK_END_ADDR);
    }

    /// This function checks if the provided address can be read from.
    pub fn allows_read_from(&self, address: u32) -> bool {
        let text_end: u32 = MIPS_TEXT_START_ADDR + self.text.len() as u32;
        let data_end: u32 = MIPS_DATA_START_ADDR + self.data.len() as u32;
        let heap_end: u32 = MIPS_HEAP_START_ADDR + self.heap.len() as u32;
        let stack_start: u32 = MIPS_STACK_END_ADDR - self.stack.len() as u32;

        return (MIPS_TEXT_START_ADDR <= address && address < text_end)
            || (MIPS_DATA_START_ADDR <= address && address < data_end)
            || (MIPS_HEAP_START_ADDR <= address && address < heap_end)
            || (stack_start <= address && address < MIPS_STACK_END_ADDR);
    }
}

#[derive(Debug, Default)]
pub struct ProgramState {
    pub should_continue_execution: bool,
    pub cpu: Processor,
    pub cp0: Coprocessor0,
    pub memory: Memory,
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
}

/// Enumeration of general-purpose register set for simplicity in instructions
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    Gp,
    Sp,
    Fp,
    Ra,
}

/// Visibility - for use in Symbol. Enumerated version of needed variants.
#[derive(Debug, Default)]
pub enum Visibility {
    #[default]
    Local,
    Global,
    Weak,
}

/// Section - enumerated for checks in assembler and referenced in Symbol construction
#[derive(Debug, Clone)]
pub enum Section {
    Null,
    Text,
    Data,
}

/// The definition for section .line
#[derive(Debug)]
pub struct LineInfo {
    pub content: String,
    pub line_number: u32,
    pub start_address: u32,
    pub end_address: u32,
}

/// For serializing lineinfo to ELF
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

/// Handler for outside world. Operating System interprets syscalls.
/// Still WIP, will grow to include other non processor peripheries (which can interact through MMIO)
#[derive(Debug)]
pub struct OperatingSystem {
    stdin: Stdin,
    stdout: Stdout,
}

impl OperatingSystem {
    pub fn new() -> OperatingSystem {
        OperatingSystem {
            stdin: stdin(),
            stdout: stdout(),
        }
    }

    /// Contains the logic for handling syscalls.
    /// Invoked by the exception handler.
    pub fn handle_syscall(&mut self, program_state: &mut ProgramState) -> Result<(), String> {
        let syscall_num: usize =
            program_state.cpu.general_purpose_registers[Register::V0 as usize] as usize;

        match syscall_num {
            0x01 => sys_print_int(program_state, &mut self.stdout.lock()),
            0x04 => sys_print_string(program_state, &mut self.stdout.lock()),
            0x05 => sys_read_int(program_state, &mut self.stdin.lock()),
            0x0A => sys_exit(program_state),
            0x0B => sys_print_char(program_state, &mut self.stdout.lock()),
            0x0C => sys_read_char(program_state, &mut self.stdin.lock()),
            _ => Err(format!("{} is not a recognized syscall.", syscall_num)),
        }
    }

    pub fn handle_breakpoint(
        &mut self,
        program_state: &mut ProgramState,
        lineinfo: &Vec<LineInfo>,
        debugger_state: &mut DebuggerState,
    ) -> () {
        /* Needs to do the following:
         * Transfer control to the user
         *      Register dump (pretty pa)
         *      Type in a letter to get a hex dump of .data
         * Note that cp0 should have flags for whether user ran c or s
         * Idea: simply replace the instruction on bp.line_num with break
         *      when done, rereplace the instruction and decrement pc by 4 :jadCensored:
         * Use the code in the break instruction to match injectively (:nerd:) to the instruction you replaced
         */

        // program_state.cpu.pc -= MIPS_ADDRESS_ALIGNMENT;

        // grab the breakpoint number and other breakpoint information in one fell swoop
        let bp_tuple = match debugger_state
            .breakpoints
            .iter()
            .enumerate()
            .find(|bp| bp.1.address == program_state.cpu.pc - MIPS_ADDRESS_ALIGNMENT)
        {
            Some(toupee) => toupee,
            None => {
                panic!("Breakpoint not found in breakpoint vector. (How.)");
            }
        };

        println!(
            "Breakpoint {} at line {} reached.",
            bp_tuple.0, bp_tuple.1.line_num
        );
        program_state.register_dump();

        // program counter is now pointing to the instruction AFTER the breakpoint
        // once we begin executing code again, execute the breakpoint's replaced_instruction instead of the actual breakpoint

        if program_state.cp0.is_debug_mode() {
            // terminate existing debugger process????
            // what
            match self.cli_debugger(lineinfo, program_state, debugger_state) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{e}");
                }
            }
            // return;
        } else {
            return;
        }
        //TODO: ("Finish breakpoint handler implementation @Nick");
    }

    // Pass control to the user upon hitting a breakpoint
    pub fn cli_debugger(
        &mut self,
        lineinfo: &Vec<LineInfo>,
        program_state: &mut ProgramState,
        debugger_state: &mut DebuggerState,
    ) -> Result<(), String> {
        println!("Welcome to the NAME CLI debugger.");
        println!("For a list of commands, type \"help\".");

        loop {
            dbprint!(debugger_state.sioc, "(name-db) ");
            self.stdout.flush().expect("Failed to flush stdout");

            // take in the command and split it up into arguments
            let mut user_input = String::new();
            match self.stdin.read_line(&mut user_input) {
                Ok(_) => {}
                Err(e) => eprintln!("stdin error: {e}"),
            };
            let db_args: Vec<String> = user_input
                .trim()
                .split(" ")
                .map(|s| s.to_string())
                .collect();

            match db_args[0].as_str() {
                "help" => match help_menu(db_args, debugger_state) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "q" => return Ok(()),
                "exit" => return Ok(()),
                "quit" => return Ok(()),
                "r" => match continuously_execute(lineinfo, program_state, self, debugger_state) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "c" => match continuously_execute(lineinfo, program_state, self, debugger_state) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "s" => match db_step(lineinfo, program_state, self, debugger_state) {
                    Ok(_) => continue,
                    Err(e) => {
                        if e == "Breakpoint reached." {
                            continue;
                        } else {
                            eprintln!("{e}");
                        }
                    }
                },
                "l" => match list_text(lineinfo, debugger_state, &db_args) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "p" => match print_register(program_state, debugger_state, &db_args) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "pa" => match program_state.print_all_registers(&db_args) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "m" => match modify_register(program_state, debugger_state, &db_args) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "pb" => match debugger_state.print_all_breakpoints() {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "b" => match debugger_state.add_breakpoint(lineinfo, &db_args, program_state) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                "del" => match debugger_state.remove_breakpoint(&db_args, program_state) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{e}"),
                },
                _ => eprintln!("Option not recognized. Type \"help\" to view accepted options."),
            };
        }
    }
}
