use std::process::exit;

use crate::{
    debug::debug_utils::DebuggerState,
    exception::definitions::{ExceptionType, SourceContext},
    structs::{OperatingSystem, ProgramState},
};

use crate::simulator_helpers::generate_err;

/// The exception handler is invoked whenever an exception has occurred.
/// Some common exceptions include breakpoints, syscalls, and arithmetic overflow.
/// It takes a mutable program state and matches on the exception type - then, it resets state if possible.
pub fn handle_exception(
    program_state: &mut ProgramState,
    os: &mut OperatingSystem,
    source_context: &SourceContext,
    debugger_state: &mut DebuggerState,
) {
    // In order to invoke this function, certain values (like exception_level == 1) are already assumed.

    // Attempt to recognize the exception that occurred
    let exception_type = match ExceptionType::try_from(program_state.cp0.get_exc_code()) {
        Ok(exc_type) => exc_type,
        Err(e) => panic!("{e}"),
    };

    // Retrieve necessary values
    let epc: u32 = program_state.cp0.get_epc();

    // Match on exception type to either error out or handle appropriately
    match exception_type {
        ExceptionType::AddressExceptionLoad => {
            // TODO: Detect difference between instructions like bad lw and bad/misaligned pc
            eprintln!("{}", generate_err(source_context, epc, "Illegal address provided for load/fetch; misaligned, unreachable, or unowned address."));
            exit(0);
        }
        ExceptionType::AddressExceptionStore => {
            eprintln!("{}", generate_err(source_context, epc, "Illegal address provided on store operation; misaligned, unreachable, or unowned address."));
            exit(0);
        }
        ExceptionType::BusFetch => {
            eprintln!("{}", generate_err(
                source_context,
                epc,
                "Failed to interpret instruction as word; Unrecognized bytes in ELF .text space.",
            ));
            exit(0);
        }
        ExceptionType::BusLoadStore => {
            eprintln!(
                "{}",
                generate_err(
                    source_context,
                    epc,
                    "Failed to store data in given address."
                )
            );
            exit(0);
        }
        ExceptionType::Syscall => {
            // Invoke the syscall handler on program state
            if let Err(e) = os.handle_syscall(program_state) {
                panic!(
                    "{}",
                    generate_err(
                        source_context,
                        epc,
                        &format!("Failed to handle a syscall: {e}")
                    )
                )
            }
        }
        ExceptionType::Breakpoint => {
            // Invoke the breakpoint handler on program state and lineinfo
            if program_state.cp0.is_debug_mode() {
                // debugger is running.
                os.handle_breakpoint(program_state, source_context, debugger_state);
            } else {
                panic!("Break not recognized outside of debug mode. To run in debug mode, pass -d as a command line argument.");
            }
        }
        ExceptionType::ReservedInstruction => {
            eprintln!(
                "{}",
                generate_err(
                    source_context,
                    epc,
                    "Reserved instruction encountered; Unrecognized bytes in ELF at program counter.",
                )
            );
            exit(0);
        }
        ExceptionType::CoprocessorUnusable => {
            eprintln!(
                "{}",
                generate_err(
                    source_context,
                    epc,
                    "Attempted to access a coprocessor without correct operating mode.",
                )
            );
            exit(0);
        }
        ExceptionType::ArithmeticOverflow => {
            // TODO: Differentiate between these
            eprintln!(
                "{}",
                generate_err(
                    source_context,
                    epc,
                    "Arithmetic overflow, underflow, or divide by zero detected on instruction.",
                )
            );
            exit(0);
        }
        ExceptionType::Trap => {
            todo!("Not sure how we want trap to work yet.");
        }
        ExceptionType::FloatingPoint => {
            // Will be more useful once cp1 is implemented
            eprintln!(
                "{}",
                generate_err(source_context, epc, "Floating point exception occurred.")
            );
            exit(0);
        }
    }

    // If the exception did not cause a crash, reset program state to reflect that execution will continue as normal
    program_state.recover_from_exception();
}
