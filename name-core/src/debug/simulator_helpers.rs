use crate::elf_utils::find_target_section_index;

use crate::elf_def::Elf;
use crate::structs::LineInfo;

// Extract section .text and section .data from the ELF
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

pub fn generate_err(lineinfo: &Vec<LineInfo>, address: u32, message: &str) -> String {
    // Perform an address-based search for the correct line info
    let found_lineinfo: &LineInfo = match lineinfo
        .iter()
        .find(|li| (li.start_address <= address) && (address < li.end_address))
    {
        Some(info) => info,
        // If no lineinfo was found, just give a general message
        None => return format!("[*] At pc 0x{:8x}:\n - {}", address, message),
    };

    // If lineinfo was retrieved, print a well-formed error message
    return format!(
        "[*] At pc 0x{:x}:\n - {}: {}\n - {}",
        address,
        found_lineinfo.line_number,
        found_lineinfo.get_content().trim(),
        message,
    );
}
