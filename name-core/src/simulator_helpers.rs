use std::collections::HashMap;
use std::sync::LazyLock;

use crate::constants::MIPS_ADDRESS_ALIGNMENT;
use crate::debug::fetch::fetch;
use crate::elf_utils::find_target_section_index;

use crate::elf_def::Elf;
use crate::exception::definitions::{ExceptionType, SourceContext};
use crate::instruction::information::InstructionInformation;
use crate::instruction::instruction_set::INSTRUCTION_SET;
use crate::structs::{LineInfo, ProgramState};

/// Hashmap to lookup instructions based on their lookup code. Used in the "decode"
/// portion of the Von Neumann fetch-decode-execute cycle.
pub static INSTRUCTION_LOOKUP: LazyLock<HashMap<u32, &'static InstructionInformation>> =
    LazyLock::new(|| {
        INSTRUCTION_SET
            .iter()
            .map(|instr| (instr.lookup_code(), instr))
            .collect()
    });

/// Simple single-step operation. Should not require any fancy debugger business.
pub fn single_step(_source_context: &SourceContext, program_state: &mut ProgramState) -> () {
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
        eprintln!("Executing {}", instr_info.mnemonic);
    }
    let _ = (instr_info.implementation)(program_state, raw_instruction);

    // The $0 register should never have been permanently changed. Don't let it remain changed.

    program_state.cpu.general_purpose_registers[0] = 0;
}

/// Extract section .text and section .data from the ELF
pub fn extract_loadable_sections(elf: &Elf) -> (Vec<u8>, Vec<u8>) {
    // Search section header string table for '.text' and '.data'
    let text_section: Vec<u8> = match find_target_section_index(
        &elf.section_header_table,
        &elf.sections[elf.file_header.e_shstrndx as usize - 1],
        ".text",
    ) {
        Some(section_index) => elf.sections[section_index].clone(),
        None => unreachable!(),
    };

    let data_section: Vec<u8> = match find_target_section_index(
        &elf.section_header_table,
        &elf.sections[elf.file_header.e_shstrndx as usize - 1],
        ".data",
    ) {
        Some(section_index) => elf.sections[section_index].clone(),
        None => vec![],
    };

    (data_section, text_section)
}

/// Generate a properly formatted error for the user's reading pleasure.
/// For example:
///
/// \[*\] At pc 0x00000000:
///
///  - An error occurred.
pub fn generate_err(source_context: &SourceContext, address: u32, message: &str) -> String {
    // dbg!(&source_context.lineinfo[0..80]);
    // Perform an address-based search for the correct line info
    let found_lineinfo: &LineInfo = match source_context
        .lineinfo
        .iter()
        .find(|li| (li.start_address <= address) && (address < li.end_address))
    {
        Some(info) => info,
        // If no source_context was found, just give a general message
        None => return format!("[*] At pc 0x{:8x} ({}):\n - {}", address, address, message),
    };

    // If source_context was retrieved, print a well-formed error message
    return format!(
        "[*] At pc 0x{:x} ({}):\n - {}: {}\n - {}",
        address,
        address,
        found_lineinfo.line_number,
        found_lineinfo
            .get_content(&source_context.source_filenames)
            .trim(),
        message,
    );
}
