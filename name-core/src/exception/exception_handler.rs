use std::process::exit;

use crate::{
    constants::fpu_control::FCSR_INDEX,
    debug::debug_utils::DebuggerState,
    exception::definitions::{ExceptionType, FpExceptionType, SourceContext},
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
        ExceptionType::FloatingPoint(_) => {
            // This scope is going to refer to "explicit trap" quite a bit.
            // Check MIPS Volume I-A, of course, but essentially, all IEEE-defined floating-point
            // exceptions have the capability to be caught or replaced with sensible values.
            // specifics are defined in I-A, but simply put, explicit trap for us probably means crash.
            // magic numbers for the enabled-bit check are coming from FCSR's bit field breakdown.
            let fp_exceptions: Vec<FpExceptionType> = program_state.cp1.get_floating_point_errors();
            for fp_exception_type in fp_exceptions {
                match fp_exception_type {
                    FpExceptionType::None => {
                        // Catch-all. Please don't have this triggering!
                        eprintln!(
                            "{}",
                            generate_err(
                                source_context,
                                epc,
                                "An unspecified floating point exception occurred."
                            )
                        );
                        exit(0);
                    }
                    FpExceptionType::UnimplementedOperation => {
                        // This must be an explicit trap. If it's happening, it should crash.
                        eprintln!(
                            "{}",
                            generate_err(
                                source_context,
                                epc,
                                "The target floating-point operation is not implemented."
                            )
                        );
                        exit(0);
                    }
                    FpExceptionType::InvalidOperation => {
                        // If explicit trap for invalid operation is enabled, this should crash.
                        if (program_state.cp1.control_registers[FCSR_INDEX]
                            & 0b0000_0000_0000_0000_0000_1000_0000_0000)
                            != 0
                        {
                            eprintln!(
                                "{}",
                                generate_err(
                                    source_context,
                                    epc,
                                    "An invalid operation occurred in a floating-point instruction; NaN, Infinity, or indeterminate forms may trigger this exception."
                                )
                            );
                            exit(0);
                        }
                        // If explicit trap is not enabled, this should supply a QNan.
                        // That's the burden of the caller;
                        // the right thing to do is go back to where we came and clear exception state.
                    }
                    FpExceptionType::DivideByZero => {
                        // If explicit trap for divide by zero is enabled, this should crash.
                        if (program_state.cp1.control_registers[FCSR_INDEX]
                            & 0b0000_0000_0000_0000_0000_0100_0000_0000)
                            != 0
                        {
                            eprintln!(
                                "{}",
                                generate_err(
                                    source_context,
                                    epc,
                                    "Division by zero in a floating-point division instruction occurred."
                                )
                            );
                            exit(0);
                        }

                        // If explicit trap is not enabled, it should supply a properly signed infinity. This is the burden of the caller.
                    }
                    FpExceptionType::Overflow => {
                        // If explicit trap for overflow is enabled, this should crash.
                        if (program_state.cp1.control_registers[FCSR_INDEX]
                            & 0b0000_0000_0000_0000_0000_0010_0000_0000)
                            != 0
                        {
                            eprintln!(
                                "{}",
                                generate_err(
                                    source_context,
                                    epc,
                                    "Floating-point overflow occurred."
                                )
                            );
                            exit(0);
                        }

                        // If explicit trap is not enabled, the instruction should apply the rounding mode indicated in FCSR.
                    }
                    FpExceptionType::Underflow => {
                        // If explicit trap for underflow is enabled, this should crash.
                        if (program_state.cp1.control_registers[FCSR_INDEX]
                            & 0b0000_0000_0000_0000_0000_0001_0000_0000)
                            != 0
                        {
                            eprintln!(
                                "{}",
                                generate_err(
                                    source_context,
                                    epc,
                                    "Subnormal results occurred (floating-point underflow)."
                                )
                            );
                            exit(0);
                        }

                        // If explicit trap is disabled, flush to zero on subnormal.
                    }
                    FpExceptionType::Inexact => {
                        // If explicit trap for inexact is enabled, this should crash.
                        if (program_state.cp1.control_registers[FCSR_INDEX]
                            & 0b0000_0000_0000_0000_0000_0000_1000_0000)
                            != 0
                        {
                            eprintln!(
                                "{}",
                                generate_err(
                                    source_context,
                                    epc,
                                    "Floating-point arithmetic was inexact."
                                )
                            );
                            exit(0);
                        }

                        // If explicit trap is disabled, supply a rounded result (if this came from overflow, supply overflowed).
                    }
                }
            }
        }
    }

    // If the exception did not cause a crash, reset program state to reflect that execution will continue as normal
    program_state.recover_from_exception();
}
