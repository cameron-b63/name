use name_core::debug::exception_handler::handle_exception;
use name_core::debug::simulator_helpers::extract_loadable_sections;

use name_core::debug::debug_utils::{single_step, DebuggerState};

use name_core::elf_def::Elf;
use name_core::elf_utils::extract_lineinfo;
use name_core::structs::{LineInfo, Memory, OperatingSystem, Processor, ProgramState};

pub fn simulate(elf: Elf, debug: bool) -> Result<(), String> {
    // Set up simulation environment from information in ELF
    let cpu: Processor = Processor::new(elf.file_header.e_entry);

    let (data, text) = extract_loadable_sections(&elf);

    let lineinfo: Vec<LineInfo> = extract_lineinfo(&elf);

    let memory: Memory = Memory::new(data, text);

    // Create program state
    let mut program_state: ProgramState = ProgramState::new(cpu, memory);
    // Setup a new operating system
    let mut operating_system: OperatingSystem = OperatingSystem::new();

    if debug {
        // Invoke the cli debugger if the user asked for it
        operating_system.cli_debugger(&lineinfo, &mut program_state, &mut DebuggerState::new())
    } else {
        // Begin fetch/decode/execute cycle to run program normally
        while program_state.should_continue_execution {
            // Run the next instruction
            single_step(&lineinfo, &mut program_state);
            // If an exception occurred, handle it
            if program_state.is_exception() {
                handle_exception(&mut program_state, &mut operating_system, &lineinfo, &mut DebuggerState::new());
            }
        }

        Ok(())
    }
}
