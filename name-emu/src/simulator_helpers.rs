use name_core::elf_utils::find_target_section_index;

use name_core::elf_def::Elf;
use name_core::structs::LineInfo;

// Extract section .text and section .data from the ELF
pub fn extract_loadable_sections(elf: &Elf) -> (Vec<u8>, Vec<u8>) {
    // Search section header string table for '.text' and '.data'
    let text_section: Vec<u8> = match find_target_section_index(
        &elf.section_header_table,
        &elf.sections[elf.file_header.e_shstrndx as usize],
        ".text",
    ) {
        Some(section_index) => elf.sections[section_index].clone(),
        None => unreachable!(),
    };

    let data_section: Vec<u8> = match find_target_section_index(
        &elf.section_header_table,
        &elf.sections[elf.file_header.e_shstrndx as usize],
        ".data",
    ) {
        Some(section_index) => elf.sections[section_index].clone(),
        None => vec![],
    };

    (text_section, data_section)
}

pub fn generate_err(lineinfo: &Vec<LineInfo>, address: u32, message: &str) -> String {
    // Perform an address-based search for the correct line info
    let found_lineinfo: &LineInfo = match lineinfo
        .iter()
        .find(|li| (li.start_address <= address) && (address < li.end_address))
    {
        Some(info) => info,
        // If no lineinfo was found, just give a general message
        None => return format!("[*] At pc 0x{:x}:\n - {}", address, message),
    };

    // If lineinfo was retrieved, print a well-formed error message
    return format!(
        "[*] At pc 0x{:x}:\n - {}: {}\n - {}",
        address,
        found_lineinfo.line_number,
        found_lineinfo.content.trim(),
        message,
    );
}
