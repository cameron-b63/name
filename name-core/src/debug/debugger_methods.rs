// use std::os;

use crate::{
    constants::{/*MIPS_ADDRESS_ALIGNMENT,*/ REGISTERS},
    // structs::Register,
    structs::{LineInfo, OperatingSystem, ProgramState},
    // exception::definitions::ExceptionType,
};

use crate::debug::debug_utils::{db_step, DebuggerState};
// use crate::debug::exception_handler::handle_exception;
// use crate::debug::fetch::fetch;

//
// Everything below this point is function spam for the cli debugger.
// Autocollapse is your best friend...
//

/// Executes program normally until otherwise noted. Invoked by "r" or "c" in the CLI.
pub fn continuously_execute(
    lineinfo: &Vec<LineInfo>,
    program_state: &mut ProgramState,
    os: &mut OperatingSystem,
    debugger_state: &mut DebuggerState,
) -> Result<(), String> {
    // TODO: make a reinitializer
    while program_state.should_continue_execution {
        match db_step(lineinfo, program_state, os, debugger_state) {
            Ok(_) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Lists the text surrounding a given line number. Invoked by "l" in the CLI.
pub fn list_text(
    lineinfo: &Vec<LineInfo>,
    debugger_state: &mut DebuggerState,
    db_args: &Vec<String>,
) -> Result<(), String> {
    if db_args.len() == 1 {
        debugger_state.list_lines(lineinfo, debugger_state.global_list_loc);
        Ok(())
    } else if db_args.len() == 2 {
        if db_args[1] == "all" {
            for line in lineinfo {
                println!(
                    "{:>3} #{:08x}  {}",
                    line.line_number, line.start_address, line.content
                );
            }
            Ok(())
        } else {
            match db_args[1].parse::<usize>() {
                Err(_) => {
                    return Err(format!(
                        "l expects an unsigned int or \"all\" as an argument"
                    ));
                }
                Ok(lnum) => {
                    if lnum > lineinfo.len() {
                        return Err(format!("{} out of bounds of program.", lnum));
                    } else {
                        debugger_state.list_lines(lineinfo, lnum);
                    }
                }
            };
            Ok(())
        }
    } else {
        Err(format!(
            "l expects 0 or 1 arguments, received {}",
            db_args.len() - 1
        ))
    }
}

/// Prints the value at a given register. Invoked by "p" in the CLI.
pub fn print_register(
    program_state: &mut ProgramState,
    db_args: &Vec<String>,
) -> Result<(), String> {
    if db_args.len() < 2 {
        return Err(format!(
            "p expects a non-zero argument, received {}",
            db_args.len() - 1
        ));
    }

    // if the first character of the argument isn't a dollar sign,
    // assume the user isn't referring to a register
    // (eventually expand the functionality of this method to also be able to print the value of memory addresses
    // and stuff like that)

    for arg in db_args[1..].to_vec() {
        if arg.chars().nth(0) == Some('$') {
            let register = arg;
            if register == "$pc" {
                println!(
                    "Value in register {} is {:08x}",
                    register, program_state.cpu.pc
                );
                continue;
            }

            match REGISTERS.iter().position(|&x| x == register) {
                Some(found_register) => {
                    // should we continue printing the actual number of the register?
                    // this will all eventually be a table or something anyways :^)
                    println!(
                        "Value in register {} is {:08x}",
                        found_register, program_state.cpu.general_purpose_registers[found_register]
                    );
                }
                None => {
                    return Err(format!("{} is not a valid register.", db_args[1]));
                }
            }
        } else if arg.chars().nth(0) == Some('#') {
            // there's a method in the assembler to convert a word into a line
            let address = match u32::from_str_radix(&arg[1..], 16) {
                Ok(addy) => addy,
                Err(e) => return Err(format!("{e}")),
            };

            let value = match program_state.memory.read_byte(address) {
                Ok(v) => v,
                Err(e) => return Err(format!("{e}")),
            };

            println!("Value in address {:08x} is {:08b}", address, value);
        } else {
            return Err(format!(
                "Congrats! You discovered an unimplemented feature... or you forgot the dollar sign on your register, or the hashtag in your memory address."
            ));
        }
    }
    Ok(())
}

/// Modifies the value inside a given register. Invoked by "m" in the CLI.
pub fn modify_register(
    program_state: &mut ProgramState,
    db_args: &Vec<String>,
) -> Result<(), String> {
    if db_args.len() != 3 {
        return Err(format!(
            "m expects 2 arguments, received {}",
            db_args.len() - 1
        ));
    }

    // grab the register we want to modify
    let register = match REGISTERS.iter().position(|&x| x == db_args[1]) {
        Some(found_register) => found_register,
        None => {
            return Err(format!(
                "First argument to m must be a register. (Did you include the dollar sign?)"
            ));
        }
    };

    // grab the value we want to change the register to
    let parsed_u32 = match db_args[2].parse::<u32>() {
        Ok(found) => found,
        Err(e) => {
            return Err(format!("{e}"));
        }
    };

    let original_val = program_state.cpu.general_purpose_registers[register];
    program_state.cpu.general_purpose_registers[register] = parsed_u32;
    println!(
        "Successfully modified value in register {} from {} to {}.",
        db_args[1], original_val, parsed_u32
    );
    Ok(())
}

pub fn help_menu(db_args: Vec<String>) -> Result<(), String> {
    if db_args.len() == 1 {
        println!("help - Display this menu.");
        println!("help [CMD] - Get more information about a specific db command CMD.");
        println!("r - Begin execution of program.");
        println!("c - Continue program execution until the next breakpoint.");
        println!("s - Execute only the next instruction.");
        println!("l - Print the entire program. (this functionality will be much improved later)");
        println!("p - Print the value of a register (or registers) at the current place in program execution (please include the dollar sign).");
        println!("pa - Print value of ALL registers at once.");
        println!("pb - Print all breakpoints.");
        println!("b [N] - Insert a breakpoint at line number N.");
        println!("del [N] - Delete breakpoint number N.");
        println!("q - Exit (quit) debugger.");
    } else if db_args.len() == 2 {
        match &db_args[1] as &str {
            "r" => {
                println!("Begin execution of program.");
            }
            "c" => {
                println!("Continue program execution until the next breakpoint.");
            }
            "s" => {
                println!("Execute only the next instruction.");
            }
            "l" => {
                println!("When provided no arguments: print the first ten lines of the program. Then, print the next 10, and so forth.");
                println!("When provided a line number (positive integer): print 9 lines around the given line number.");
                println!("When provided the argument \"all\": print the entire program.");
            }
            "p" => {
                println!("Print the value stored in the provided register.");
            }
            "pa" => {
                println!("Print each register and the value stored therein.");
            }
            "pb" => {
                println!("Print all user-created breakpoints.");
            }
            "b" => {
                println!("Insert a breakpoint at the line number provided. Note that this line will be executed before the break occurs.");
            }
            "del" => {
                println!("Delete the breakpoint with the associated number. (run pb to find out which number the desired breakpoint has)");
            }
            "help" => {
                println!("you're funny");
            }
            "q" => {
                println!("please work :wq please work :wq plea");
            }
            _ => {
                eprintln!("{} is either not recognized as a valid command or the help menu for it was neglected to be implemented.", db_args[1]);
            }
        };
    }
    Ok(())
}
