use name_core::debug::simulator_helpers::extract_loadable_sections;

use name_core::debug::debug_utils::DebuggerState;

use name_core::elf_def::Elf;
use name_core::elf_utils::extract_lineinfo;
use name_core::exception::exception_handler::handle_exception;
use name_core::structs::{LineInfo, Memory, OperatingSystem, Processor, ProgramState};

pub fn simulate(elf: Elf, debug: bool, separate_io_channels: bool) -> Result<(), String> {
    // Set up simulation environment from information in ELF
    let cpu: Processor = Processor::new(elf.file_header.e_entry);

    let (data, text) = extract_loadable_sections(&elf);

    let source_context: SourceContext = extract_lineinfo(&elf);

    let memory: Memory = Memory::new(data, text);

    // Create program state
    let mut program_state: ProgramState = ProgramState::new(cpu, memory);

    // Setup a new operating system
    let mut operating_system: OperatingSystem = OperatingSystem::new();

    program_state.cp0.set_debug_mode(debug);
    if program_state.cp0.is_debug_mode() {
        // Invoke the cli debugger if the user asked for it
        // When VSCode extension is implemented, add a flag here to determine whether to
        // run the CLI debugger right away or to engage in soon-to-be-defined behavior
        // depending on whether the user ran this from the command line or from the nice little VSCode button
        return operating_system.cli_debugger(
            &source_context,
            &mut program_state,
            &mut DebuggerState::new(separate_io_channels),
        );
    } else {
        // Begin fetch/decode/execute cycle to run program normally
        while program_state.should_continue_execution {
            // Run the next instruction
            single_step(&source_context, &mut program_state);
            // If an exception occurred, handle it
            if program_state.is_exception() {
                handle_exception(
                    &mut program_state,
                    &mut operating_system,
                    &source_context,
                    &mut DebuggerState::new(separate_io_channels),
                );
                if program_state.cp0.is_debug_mode() {} // ooops you have to put the cd in the computer
            }
        }
    }

    Ok(())
}
