use std::{collections::HashMap, sync::LazyLock};

use name_core::{
    constants::MIPS_ADDRESS_ALIGNMENT,
    exception::definitions::ExceptionType,
    instruction::{information::InstructionInformation, instruction_set::INSTRUCTION_SET},
    structs::{LineInfo, ProgramState},
};

use crate::debug::debugger_methods::*;

static INSTRUCTION_LOOKUP: LazyLock<HashMap<u32, &'static InstructionInformation>> =
    LazyLock::new(|| {
        INSTRUCTION_SET
            .iter()
            .map(|instr| (instr.lookup_code(), instr))
            .collect()
    });

use crate::fetch::fetch;

use std::io::{self, Write};

pub fn handle_breakpoint(_program_state: &mut ProgramState, _lineinfo: &Vec<LineInfo>) -> () {
    todo!("Finish breakpoint handler implementation @Nick");
}

pub fn single_step(
    _lineinfo: &Vec<LineInfo>,
    program_state: &mut ProgramState,
    debugger_state: &DebuggerState,
) -> () {
    // passing a breakpoints vector into this function is a very messy way of doing this, i'm aware,,,
    // ideally, a break instruction is physically injected into the code and everything works politely from there without extra shenaniganery.
    // however, for now, this will have to do
    if !program_state
        .memory
        .allows_execution_of(program_state.cpu.pc)
    {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
        return;
    }

    // println!("{}", program_state.cpu.pc);

    // check if there's a breakpoint after instruction on the line is executed
    for bp in &debugger_state.breakpoints {
        if program_state.cpu.pc == bp.address {
            // println!("Breakpoint at line {} reached. (This ran in single_step())", bp.line_num);
            program_state.set_exception(ExceptionType::Breakpoint);
        }
    }

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

#[derive(Debug)]
pub struct Breakpoint {
    pub bp_num: u16, // why do you have 65535 breakpoints. do better
    pub line_num: u32,
    pub address: u32,
}

pub struct DebuggerState {
    pub breakpoints: Vec<Breakpoint>,
    pub global_bp_num: u16,

    pub global_list_loc: usize, // for the l command
}

// pub type DebugFn = fn(&Vec<LineInfo>, &mut Memory, &mut Processor, &Vec<Breakpoint>) -> Result<(), String>;

// This is the name debugger. Have fun...
pub fn debugger(lineinfo: &Vec<LineInfo>, program_state: &mut ProgramState) -> Result<(), String> {
    let mut debugger_state = DebuggerState::new();

    println!("Welcome to the NAME debugger.");
    println!("For a list of commands, type \"help\".");

    loop {
        print!("(name-db) ");
        io::stdout().flush().expect("Failed to flush stdout");

        // take in the command and split it up into arguments
        let mut user_input = String::new();
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => {}
            Err(e) => eprintln!("stdin error: {e}"),
        };
        let db_args: Vec<String> = user_input
            .trim()
            .split(" ")
            .map(|s| s.to_string())
            .collect();

        match db_args[0].as_str() {
            "help" => match help_menu(db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "q" => return Ok(()),
            "exit" => return Ok(()),
            "quit" => return Ok(()),
            "r" => match continuously_execute(lineinfo, program_state, &mut debugger_state) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "c" => match continuously_execute(lineinfo, program_state, &mut debugger_state) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "s" => match db_step(lineinfo, program_state, &mut debugger_state) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "l" => match list_text(lineinfo, &mut debugger_state, &db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "p" => match print_register(program_state, &db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "pa" => match print_all_registers(program_state, &db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "m" => match modify_register(program_state, &db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "pb" => match debugger_state.print_all_breakpoints() {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "b" => match debugger_state.add_breakpoint(lineinfo, &db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            "del" => match debugger_state.remove_breakpoint(&db_args) {
                Ok(_) => continue,
                Err(e) => eprintln!("{e}"),
            },
            _ => eprintln!("Option not recognized. Type \"help\" to view accepted options."),
        };
    }
}
