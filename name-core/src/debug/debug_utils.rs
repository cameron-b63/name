use std::{collections::HashMap, sync::LazyLock};
// use std::io::{self, Write};

// use crate::debug::debugger_methods::*;

use crate::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_TEXT_START_ADDR},
    debug::{
        exception_handler::handle_exception, fetch::fetch
    },
    exception::definitions::ExceptionType,
    instruction::{information::InstructionInformation, instruction_set::INSTRUCTION_SET, RawInstruction},
    structs::{LineInfo, OperatingSystem, ProgramState},
};

static INSTRUCTION_LOOKUP: LazyLock<HashMap<u32, &'static InstructionInformation>> =
    LazyLock::new(|| {
        INSTRUCTION_SET
            .iter()
            .map(|instr| (instr.lookup_code(), instr))
            .collect()
    });

pub fn single_step(
    _lineinfo: &Vec<LineInfo>,
    program_state: &mut ProgramState,
) -> () {
    if !program_state
        .memory
        .allows_execution_of(program_state.cpu.pc)
    {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // check if there's a breakpoint before instruction on the line is executed
    // TODO: implement break instruction. check after fetch.

    // Fetch
    let raw_instruction = fetch(program_state);
    let instr_info = match INSTRUCTION_LOOKUP.get(&raw_instruction.get_lookup()) {
        Some(info) => info,
        None => {
            program_state.set_exception(ExceptionType::ReservedInstruction);
            return;
        }
    };

    program_state.cpu.pc += MIPS_ADDRESS_ALIGNMENT;

    // Execute the instruction; program_state is modified.
    if false
    /* Allowing for some later verbose mode */
    {
        println!("Executing {}", instr_info.mnemonic);
    }
    let _ = (instr_info.implementation)(program_state, raw_instruction);

    // The $0 register should never have been permanently changed. Don't let it remain changed.

    program_state.cpu.general_purpose_registers[0] = 0;
}

/// Executes only the next line of code. Invoked by "s" in the CLI.
// Also called by continuously_execute
pub fn db_step(
    lineinfo: &Vec<LineInfo>,
    program_state: &mut ProgramState,
    os: &mut OperatingSystem,
    debugger_state: &mut DebuggerState,
) -> Result<(), String> {
    let prev_funct_code = match program_state.memory.read_byte(program_state.cpu.pc - 1) {
        Ok(byte) => byte & 0b00111111,
        Err(e) => {
            if program_state.cpu.pc != MIPS_TEXT_START_ADDR {
                panic!("{e}"); // TODO: once this is back in name-emu, make nicer errors using generate_err
            } else {
                // we're only using the prev_funct_code to check for a breakpoint.
                // so we can set it to 0 if we don't care what it actually is
                0
            }
        }
    };

    // if we just ran past a breakpoint, execute the instruction it replaced.
    if prev_funct_code == 0b001101 {
        let bp_num = match get_bp_num(program_state) {
            Ok(idx) => idx,
            Err(e) => return Err(format!("{e}"))
        };

        let mut bp: &mut Breakpoint = match debugger_state.breakpoints.get(bp_num as usize).clone() {
            Some(bp) => bp,
            None => {
                return Err(format!("Breakpoint {} not found in memory.", bp_num));
            }
        };


        /* This is just copy paste from single_step with edits to make it make sense for our case */

        if bp.already_executed {
            bp.flip_execution_status();
        } else {
            let raw_instruction: RawInstruction;
            let instr_info: &&InstructionInformation;

            // Fetch the instruction replaced by the breakpoint
            raw_instruction = RawInstruction::new(bp.replaced_instruction); // lol
            instr_info = match INSTRUCTION_LOOKUP.get(&raw_instruction.get_lookup()) {
                Some(info) => info,
                None => {
                    program_state.set_exception(ExceptionType::ReservedInstruction);
                    return Err(format!("Reserved instruction reached. (My code is bad so the program state has been changed as a result. Lord help us)"));
                }
            };
                // Execute the instruction; program_state is modified.
            if true
            /* Allowing for some later verbose mode */
            {
                println!("Executing {}", instr_info.mnemonic);
            }
            let _ = (instr_info.implementation)(program_state, raw_instruction);

            // resolve the breakpoint exception
            program_state.recover_from_exception();

            // program_state.cpu.pc += MIPS_ADDRESS_ALIGNMENT;

            bp.flip_execution_status();


            // The $0 register should never have been permanently changed. Don't let it remain changed.
            program_state.cpu.general_purpose_registers[0] = 0;

            return Ok(())
        }
    }

         
    single_step(lineinfo, program_state);
    if program_state.is_exception() {
        // todo!("Handle exception");
        // return Err("exceptionnnnnnnnn".to_string())
        if program_state.cp0.get_exc_code() != ExceptionType::Breakpoint.into() {
            handle_exception(program_state, os, lineinfo, debugger_state);
        } else {
            return Err("Breakpoint reached.".to_string());
        }
    }
    Ok(())
}

/// Returns the breakpoint number of the instruction at $pc.
fn get_bp_num(program_state: &ProgramState) -> Result<u32, String> {
    let bitmask: u32 = 0b00000011111111111111111111000000; // sorry for the magic number. this is filtering out the code field of the break instruction
    let mut bp_num = 0;

    for i in 0..4 {
        let mut word_portion = match program_state.memory.read_byte(dbg!(program_state.cpu.pc - MIPS_ADDRESS_ALIGNMENT + i)){
            Ok(byte) => byte as u32,
            Err(e) => {
                return Err(format!("{e}"))
            }
        };

        word_portion &= (bitmask >> (24 - 8*i)) & 0xff;

        bp_num |= word_portion << 24 - 8*i;
    }

    // we have the break instruction with the code field filtered out.
    // shift it to the right by the number of bits in the funct code to get the actual breakpoint number :^)
    bp_num >>= 6; 

    return Ok(bp_num);
}


#[derive(Debug)]
pub struct Breakpoint {
    // pub bp_num: u16, // why do you have 65535 breakpoints. do better
    pub line_num: u32,
    pub address: u32,
    pub replaced_instruction: u32,
    pub already_executed: bool,
}

pub struct DebuggerState {
    pub global_bp_num: usize, // point to the first available empty space in the breakpoint vector
    pub breakpoints: Vec<Breakpoint>, // indexed by bp_num
    // pub replaced_instructions: Vec<u32>, // also indexed by bp num
    pub global_list_loc: usize, // for the l command; like the center of the output
}

// pub type DebugFn = fn(&Vec<LineInfo>, &mut Memory, &mut Processor, &Vec<Breakpoint>) -> Result<(), String>;