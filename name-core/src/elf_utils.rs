// Utilities to assemble to ELF.

// Imports
use std::{fs, io::Write, path::PathBuf, vec::Vec};

use crate::constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR};
use crate::elf_def::*;
use crate::structs::{LineInfo, Section, Symbol, Visibility}; // Used for ELF sections

// Macros - had to learn somehow!

macro_rules! ELF32_ST_INFO {
    ($bind:expr, $type:expr) => {
        (($bind << 4) + ($type & 0xf)) as u8
    };
}

/*

 __          _______  _____ _______ ______
 \ \        / /  __ \|_   _|__   __|  ____|
  \ \  /\  / /| |__) | | |    | |  | |__
   \ \/  \/ / |  _  /  | |    | |  |  __|
    \  /\  /  | | \ \ _| |_   | |  | |____
     \/  \/   |_|  \_\_____|  |_|  |______|



*/

// Create a new ET_REL ELF file header with default values
// takes parameters passed_e_entry, the entry point of the program,
// and passed_e_shoff, the section header offset calculated in a separate method.
fn create_new_et_rel_file_header(passed_e_shoff: u32) -> Elf32Header {
    Elf32Header {
        e_ident: E_IDENT_DEFAULT,
        e_type: E_TYPE_DEFAULT,
        e_machine: E_MACHINE_DEFAULT,
        e_version: E_VERSION_DEFAULT,
        e_entry: E_ENTRY_DEFAULT,
        e_phoff: E_PHOFF_DEFAULT,
        e_shoff: passed_e_shoff,
        e_flags: E_FLAGS_DEFAULT,
        e_ehsize: E_EHSIZE_DEFAULT,
        e_phentsize: E_PHENTSIZE_DEFAULT,
        e_phnum: E_PHNUM_DEFAULT,
        e_shentsize: E_SHENTSIZE_DEFAULT,
        e_shnum: E_SHNUM_DEFAULT,
        e_shstrndx: E_SHSTRNDX_DEFAULT,
    }
}

// This function combines all the previous to actually create a new object file.
pub fn create_new_et_rel(
    data_section: Vec<u8>,
    text_section: Vec<u8>,
    symtab_section: Vec<u8>,
    strtab_section: Vec<u8>,
    line_section: Vec<u8>,
) -> Elf {
    // The section header string table entry requires some calculations.
    // Here we get the shstrtab as bytes from the constant defined at the top of the file.
    // We also get the size of the shstrtab.
    let mut shstrtab_section: Vec<u8> = vec![];
    for item in SECTIONS {
        shstrtab_section.extend_from_slice(item.as_bytes());
        shstrtab_section.extend_from_slice(&[b'\0']);
    }
    let shstrtab_size: u32 = shstrtab_section.len() as u32;

    // Get size of each section to properly calculate offsets in result file
    let data_size: u32 = data_section.len() as u32;
    let text_size: u32 = text_section.len() as u32;
    let symtab_size: u32 = symtab_section.len() as u32;
    let strtab_size: u32 = strtab_section.len() as u32;
    let line_size: u32 = line_section.len() as u32;

    // Calculate offsets using sizes
    let data_offset: u32 = E_PHOFF_DEFAULT + (E_PHNUM_DEFAULT * E_PHENTSIZE_DEFAULT) as u32;
    let text_offset: u32 = data_offset + data_size; // The program header entries are for the two loadable segments, .text and .data
    let symtab_offset: u32 = text_offset + text_size;
    let strtab_offset: u32 = symtab_offset + symtab_size;
    let line_offset: u32 = strtab_offset + strtab_size;
    let shstrtab_offset: u32 = line_offset + line_size;
    let sh_offset: u32 = shstrtab_offset + shstrtab_size;

    // Construct the ELF file header
    let elf_file_header: Elf32Header = create_new_et_rel_file_header(sh_offset);

    // Populate the program headers - by MIPS convention, section .text should be at 0x00400000 and section .data at 0x10000000
    let data_ph: Elf32ProgramHeader = Elf32ProgramHeader {
        p_type: PT_LOAD,
        p_offset: data_offset,
        p_vaddr: MIPS_DATA_START_ADDR,
        p_paddr: MIPS_DATA_START_ADDR,
        p_filesz: data_size,
        p_memsz: data_size,
        p_flags: PF_R | PF_W, // section .data should not be executable
        p_align: MIPS_ALIGNMENT,
    };

    let text_ph: Elf32ProgramHeader = Elf32ProgramHeader {
        p_type: PT_LOAD,
        p_offset: text_offset,
        p_vaddr: MIPS_TEXT_START_ADDR,
        p_paddr: MIPS_TEXT_START_ADDR,
        p_filesz: text_size,
        p_memsz: text_size,
        p_flags: PF_R | PF_X, // section .text should not be writable
        p_align: MIPS_ALIGNMENT,
    };

    // Construct program header table
    let complete_program_header_table: Vec<Elf32ProgramHeader> = vec![data_ph, text_ph];

    // Populate the section headers - indexes are in the same order as the struct (.data, .text, .debug, .line)
    // First field is SHT_NULL and reserved, but must be included.
    let null_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: 0, // This is a byte index
        sh_type: SHT_NULL,
        sh_flags: 0,
        sh_addr: 0,
        sh_offset: 0,
        sh_size: 0,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: 0,
    };

    let data_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: 1,
        sh_type: SHT_PROGBITS,
        sh_flags: SHF_ALLOC | SHF_WRITE, // Allocated and writeable
        sh_addr: MIPS_DATA_START_ADDR,
        sh_offset: data_offset,
        sh_size: data_size,
        sh_link: 0, // Unused
        sh_info: 0, // Unused
        sh_addralign: MIPS_ADDRESS_ALIGNMENT,
        sh_entsize: 0, // Unused in this section
    };

    let text_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: data_sh.sh_name + SECTIONS[1].len() as u32 + 1,
        sh_type: SHT_PROGBITS,
        sh_flags: SHF_ALLOC | SHF_EXECINSTR, // Allocated and executable
        sh_addr: MIPS_TEXT_START_ADDR,       // Implicit virtual address
        sh_offset: text_offset,
        sh_size: text_size,
        sh_link: 0, // Unused
        sh_info: 0, // Unused
        sh_addralign: MIPS_ADDRESS_ALIGNMENT,
        sh_entsize: 0, // Unused in this section
    };

    let symtab_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: text_sh.sh_name + SECTIONS[2].len() as u32 + 1,
        sh_type: SHT_SYMTAB,
        sh_flags: 0, // The symtab does not have any flags associated
        sh_addr: 0,
        sh_offset: symtab_offset,
        sh_size: symtab_size,
        sh_link: 4, // Link to appropriate string table
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: SH_ENTSIZE_SYMTAB,
    };

    let strtab_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: symtab_sh.sh_name + SECTIONS[3].len() as u32 + 1,
        sh_type: SHT_STRTAB,
        sh_flags: SHF_STRINGS,
        sh_addr: 0,
        sh_offset: strtab_offset,
        sh_size: strtab_size,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: 0,
    };

    let line_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: strtab_sh.sh_name + SECTIONS[4].len() as u32 + 1,
        sh_type: SHT_PROGBITS,
        sh_flags: 0,
        sh_addr: 0,
        sh_offset: line_offset,
        sh_size: line_size,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: 0,
    };

    let shstrtab_sh: Elf32SectionHeader = Elf32SectionHeader {
        sh_name: line_sh.sh_name + SECTIONS[5].len() as u32 + 1,
        sh_type: SHT_STRTAB,
        sh_flags: SHF_STRINGS,
        sh_addr: 0,
        sh_offset: shstrtab_offset,
        sh_size: shstrtab_size,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: 0,
    };

    // Collect all sections into sections Vec
    let sections: Vec<Vec<u8>> = vec![
        data_section,
        text_section,
        symtab_section,
        strtab_section,
        line_section,
        shstrtab_section,
    ];

    // Collect all previously defined section headers into the section header table
    let complete_section_header_table: Vec<Elf32SectionHeader> = vec![
        null_sh,
        data_sh,
        text_sh,
        symtab_sh,
        strtab_sh,
        line_sh,
        shstrtab_sh,
    ];

    // Final step is to create the final Elf struct
    return Elf {
        file_header: elf_file_header,
        program_header_table: complete_program_header_table,
        sections: sections,
        section_header_table: complete_section_header_table,
    };
}

// Used in et_rel construction process to create .symbtab and .strtab
pub fn convert_symbol_to_elf32sym(symbol: &Symbol, strtab_index: u32) -> Elf32Sym {
    Elf32Sym {
        st_name: strtab_index,
        st_value: symbol.value,
        st_size: symbol.size,
        st_info: match symbol.visibility {
            Visibility::Local => ELF32_ST_INFO!(0, symbol.symbol_type),
            Visibility::Global => ELF32_ST_INFO!(1, symbol.symbol_type),
            Visibility::Weak => ELF32_ST_INFO!(2, symbol.symbol_type),
        },
        st_other: match symbol.visibility {
            Visibility::Local => 2,
            Visibility::Global => 0,
            Visibility::Weak => 0,
        },
        st_shndx: match symbol.section {
            Section::Text => 1,
            Section::Data => 2,
            _ => 0,
        },
    }
}

// This function creates a new file with the passed name and writes all bytes in a RelocatableElf object
pub fn write_elf_to_file(file_name: &PathBuf, et_rel: &Elf) -> Result<(), String> {
    // Declare file_bytes vector to push all these file bytes onto
    // Concatenate all bytes in file header
    let mut file_bytes: Vec<u8> = et_rel.file_header.to_bytes().to_vec();

    // Get all bytes in program header table
    for entry in &et_rel.program_header_table {
        file_bytes.extend(&entry.to_bytes());
    }

    // Add all sections
    for section in &et_rel.sections {
        file_bytes.extend(section);
    }

    // Section header table
    for entry in &et_rel.section_header_table {
        file_bytes.extend_from_slice(&entry.to_bytes());
    }

    // Write file bytes to output file
    let mut f: fs::File = fs::File::create(file_name).expect("Unable to write file"); // This is really bad and insecure for right now - path MUST be checked before this gets out of alpha
                                                                                      // FIXME ^ ?
    f.write_all(&file_bytes).expect("Unable to write data.");

    Ok(())
}

/*

  _____  ______          _____
 |  __ \|  ____|   /\   |  __ \
 | |__) | |__     /  \  | |  | |
 |  _  /|  __|   / /\ \ | |  | |
 | | \ \| |____ / ____ \| |__| |
 |_|  \_\______/_/    \_\_____/



*/

// read input byte vector in as ELF.
pub fn read_bytes_to_elf(file_contents: Vec<u8>) -> Result<Elf, String> {
    if file_contents.len() < E_EHSIZE_DEFAULT as usize {
        return Err(format!("Incomplete ELF file provided. Please include complete file header. Only {} bytes provided", file_contents.len()));
    }

    let elf_header: Elf32Header = match parse_elf_header(&file_contents[0..52]) {
        Ok(parsed_header) => parsed_header,
        Err(e) => return Err(e),
    };

    let num_of_ph_sections: u16 = elf_header.e_phnum;

    let program_header_table_end: usize =
        (E_EHSIZE_DEFAULT + (E_PHENTSIZE_DEFAULT * num_of_ph_sections)) as usize;
    if file_contents.len() < program_header_table_end {
        return Err(format!("Incomplete ELF file provided. Please include program header entries for {num_of_ph_sections} section(s)."));
    }

    let program_header_table_bytes =
        &file_contents[E_EHSIZE_DEFAULT as usize..program_header_table_end];
    let program_header_table: Vec<Elf32ProgramHeader> = parse_ph_table(program_header_table_bytes);

    let section_header_table_bytes =
        &file_contents[(elf_header.e_shoff as usize)..file_contents.len()];
    let section_header_table: Vec<Elf32SectionHeader> =
        parse_sh_table_bytes(section_header_table_bytes);

    let mut sections: Vec<Vec<u8>> = vec![];
    for sh in &section_header_table {
        sections.push(
            file_contents[(sh.sh_offset) as usize..(sh.sh_offset + sh.sh_size as u32) as usize]
                .to_owned(),
        );
    }

    Ok(Elf {
        file_header: elf_header,
        program_header_table: program_header_table,
        sections: sections,
        section_header_table: section_header_table,
    })
}

fn parse_elf_header(expected_bytes: &[u8]) -> Result<Elf32Header, String> {
    Ok(Elf32Header {
        e_ident: match &expected_bytes[0..16].try_into().unwrap() {
            &E_IDENT_DEFAULT => E_IDENT_DEFAULT,
            _ => return Err("E_IDENT field did not match expected format.".to_string()),
        },
        e_type: u16::from_be_bytes(expected_bytes[16..18].try_into().unwrap()),
        e_machine: match u16::from_be_bytes(expected_bytes[18..20].try_into().unwrap()) {
            E_MACHINE_DEFAULT => E_MACHINE_DEFAULT,
            _ => return Err(format!("Unexpected machine type in ELF header (expected {E_MACHINE_DEFAULT})")),
        },
        e_version: u32::from_be_bytes(expected_bytes[20..24].try_into().unwrap()),
        e_entry: u32::from_be_bytes(expected_bytes[24..28].try_into().unwrap()),
        e_phoff: match u32::from_be_bytes(expected_bytes[28..32].try_into().unwrap()) {
            E_PHOFF_DEFAULT => E_PHOFF_DEFAULT,
            _ => return Err(format!("Unexpected program header offset discovered in ELF header (expected {E_PHOFF_DEFAULT}).")),
        },
        e_shoff: u32::from_be_bytes(expected_bytes[32..36].try_into().unwrap()),
        e_flags: u32::from_be_bytes(expected_bytes[36..40].try_into().unwrap()),
        e_ehsize: match u16::from_be_bytes(expected_bytes[40..42].try_into().unwrap()) {
            E_EHSIZE_DEFAULT => E_EHSIZE_DEFAULT,
            _ => return Err(format!("Unexpected ELF header size discovered in ELF header (expected {E_EHSIZE_DEFAULT}).")),
        },
        e_phentsize: match u16::from_be_bytes(expected_bytes[42..44].try_into().unwrap()) {
            E_PHENTSIZE_DEFAULT => E_PHENTSIZE_DEFAULT,
            _ => return Err(format!("Unexpected program header entry size discovered in ELF header (expected {E_PHENTSIZE_DEFAULT})")),
        },
        e_phnum: u16::from_be_bytes(expected_bytes[44..46].try_into().unwrap()),
        e_shentsize: match u16::from_be_bytes(expected_bytes[46..48].try_into().unwrap()) {
            E_SHENTSIZE_DEFAULT => E_SHENTSIZE_DEFAULT,
            _ => return Err(format!("Unexpected section header entry size discovered in ELF header (expected {E_SHENTSIZE_DEFAULT}).")),
        },
        e_shnum: u16::from_be_bytes(expected_bytes[48..50].try_into().unwrap()),
        e_shstrndx: u16::from_be_bytes(expected_bytes[50..52].try_into().unwrap()),
    })
}

fn parse_ph_table(program_header_table_bytes: &[u8]) -> Vec<Elf32ProgramHeader> {
    program_header_table_bytes
        .chunks(E_PHENTSIZE_DEFAULT as usize)
        .map(|entry| Elf32ProgramHeader {
            p_type: u32::from_be_bytes(entry[0..4].try_into().unwrap()),
            p_offset: u32::from_be_bytes(entry[4..8].try_into().unwrap()),
            p_vaddr: u32::from_be_bytes(entry[8..12].try_into().unwrap()),
            p_paddr: u32::from_be_bytes(entry[12..16].try_into().unwrap()),
            p_filesz: u32::from_be_bytes(entry[16..20].try_into().unwrap()),
            p_memsz: u32::from_be_bytes(entry[20..24].try_into().unwrap()),
            p_flags: u32::from_be_bytes(entry[24..28].try_into().unwrap()),
            p_align: u32::from_be_bytes(entry[28..32].try_into().unwrap()),
        })
        .collect()
}

fn parse_sh_table_bytes(section_header_table_bytes: &[u8]) -> Vec<Elf32SectionHeader> {
    section_header_table_bytes
        .chunks(E_SHENTSIZE_DEFAULT as usize)
        .map(|entry| Elf32SectionHeader {
            sh_name: u32::from_be_bytes(entry[0..4].try_into().unwrap()),
            sh_type: u32::from_be_bytes(entry[4..8].try_into().unwrap()),
            sh_flags: u32::from_be_bytes(entry[8..12].try_into().unwrap()),
            sh_addr: u32::from_be_bytes(entry[12..16].try_into().unwrap()),
            sh_offset: u32::from_be_bytes(entry[16..20].try_into().unwrap()),
            sh_size: u32::from_be_bytes(entry[20..24].try_into().unwrap()),
            sh_link: u32::from_be_bytes(entry[24..28].try_into().unwrap()),
            sh_info: u32::from_be_bytes(entry[28..32].try_into().unwrap()),
            sh_addralign: u32::from_be_bytes(entry[32..36].try_into().unwrap()),
            sh_entsize: u32::from_be_bytes(entry[36..40].try_into().unwrap()),
        })
        .collect()
}

pub fn parse_elf_symbols(symbol_table: &Vec<u8>) -> Vec<Elf32Sym> {
    symbol_table
        .chunks(16)
        .map(|entry| Elf32Sym {
            st_name: u32::from_be_bytes(entry[0..4].try_into().unwrap()),
            st_value: u32::from_be_bytes(entry[4..8].try_into().unwrap()),
            st_size: u32::from_be_bytes(entry[8..12].try_into().unwrap()),
            st_info: entry[12],
            st_other: entry[13],
            st_shndx: u16::from_be_bytes(entry[14..16].try_into().unwrap()),
        })
        .collect()
}

fn get_string_from_strtab(strtab: &Vec<u8>, offset: u32) -> Option<&str> {
    let start = offset as usize;
    if start >= strtab.len() {
        return None;
    }
    let end = strtab[start..].iter().position(|&c| c == 0)?;
    std::str::from_utf8(&strtab[start..start + end]).ok()
}

pub fn extract_lineinfo(elf: &Elf) -> Vec<LineInfo> {
    let shstrtab = &elf.sections[elf.file_header.e_shstrndx as usize];
    let idx = match find_target_section_index(&elf.section_header_table, shstrtab, ".line") {
        Some(i) => i,
        None => unreachable!(),
    };

    deserialize_line_info(&elf.sections[idx])
}

pub fn find_global_symbol_address(
    symbols: &[Elf32Sym],
    strtab: &Vec<u8>,
    target: &str,
) -> Option<u32> {
    const STB_GLOBAL: u8 = 1;
    for symbol in symbols {
        let binding = symbol.st_info >> 4;
        if binding == STB_GLOBAL {
            if let Some(name) = get_string_from_strtab(strtab, symbol.st_name) {
                if name == target {
                    return Some(symbol.st_value);
                }
            }
        }
    }
    None
}

pub fn find_target_section_index(
    section_header_table: &Vec<Elf32SectionHeader>,
    strtab: &Vec<u8>,
    target: &str,
) -> Option<usize> {
    for (i, section) in section_header_table.iter().enumerate() {
        if let Some(name) = get_string_from_strtab(strtab, section.sh_name) {
            if name == target {
                return Some(i);
            }
        }
    }
    None
}

fn deserialize_line_info(data: &Vec<u8>) -> Vec<LineInfo> {
    let mut result = Vec::new();
    let mut cursor = &data[..];

    while !cursor.is_empty() {
        // Find the null terminator (0 byte) to extract the string
        if let Some(pos) = cursor.iter().position(|&c| c == 0) {
            let content_bytes = &cursor[..pos];
            let content = String::from_utf8_lossy(content_bytes).to_string();

            // Move cursor past the null terminator and string
            cursor = &cursor[pos + 1..];

            // Ensure we have at least 12 bytes remaining for three u32 values
            if cursor.len() < 12 {
                break;
            }

            // Read the u32 values
            let line_number = u32::from_be_bytes(cursor[0..4].try_into().unwrap());
            let start_address = u32::from_be_bytes(cursor[4..8].try_into().unwrap());
            let end_address = u32::from_be_bytes(cursor[8..12].try_into().unwrap());

            // Move cursor past the u32 values
            cursor = &cursor[12..];

            // Add the deserialized LineInfo to the result
            result.push(LineInfo {
                content,
                line_number,
                start_address,
                end_address,
            });
        } else {
            // If there's no null terminator found, stop processing
            break;
        }
    }

    result
}
