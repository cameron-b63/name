// This file contains the helpers for the relocate function

use std::collections::HashSet;

use name_core::{
    elf_def::{Elf, Elf32Sym, ElfType, RelocationEntry},
    elf_utils::{create_new_elf, parse_elf_symbols, parse_rel_info},
};

use crate::constants::{REL, STRTAB, SYMTAB, TEXT};

/// This function is responsible for adjusting link indices for .symtab -> .strtab, .rel -> .symtab, and .rel -> .text.
/// It will also embed information on the scope of origin in the st_other field. It's a surprise tool that will help us later!
pub fn relocate_links(elf: Elf, offsets: &Vec<Vec<u32>>) -> Elf {
    let strtab_offsets: Vec<u32> = offsets.iter().map(|set| set[STRTAB].clone()).collect();
    let mut current_strtab_adjustment: u32 = 0;
    let mut current_offset_idx: usize = 0;
    let mut previous_st_name: u32 = 0xDEADBEEF; // Just had to initalize to an impossible value, so I went with old reliable

    // For each entry in the symbol table, if the symbol has a smaller index than the previous, or it's equal (at 0), it needs to add the offset.
    // IMPORTANT (POSSIBLE EDGE CASE MUST CHECK LATER): I think this can crash at runtime with some really weird files.
    // Note the check for offset range. It doesn't seem watertight.
    let symbol_table = parse_elf_symbols(&elf.sections[SYMTAB]);
    let new_symbol_table: Vec<u8> = symbol_table
        .iter()
        .flat_map(|symbol| {
            let mut new_symbol: Elf32Sym = symbol.clone();
            if symbol.st_name <= previous_st_name && previous_st_name != 0xDEADBEEF {
                // ^ DEBUG POINT IS THIS
                current_strtab_adjustment += strtab_offsets[current_offset_idx + 1];
                current_offset_idx += 1;
            }

            previous_st_name = symbol.st_name;
            new_symbol.st_name = current_strtab_adjustment + symbol.st_name;
            new_symbol.st_other = current_offset_idx as u8;

            new_symbol.to_bytes()
        })
        .collect();

    // Perform the same process as before, but this time two adjustments at once.
    let symtab_offsets: Vec<u32> = offsets.iter().map(|set| set[SYMTAB].clone()).collect();
    let text_offsets: Vec<u32> = offsets.iter().map(|set| set[TEXT].clone()).collect();
    let mut current_symtab_adjustment: u32 = 0;
    let mut current_text_adjustment: u32 = 0;
    current_offset_idx = 0;
    let mut previous_r_offset: u32 = 0x2BADC0DE;

    let rel_section: Vec<RelocationEntry> = parse_rel_info(&elf.sections[REL]);
    let new_rel_section: Vec<u8> = rel_section
        .iter()
        .flat_map(|entry| {
            let mut new_entry: RelocationEntry = entry.clone();

            if entry.r_offset <= previous_r_offset && previous_r_offset != 0x2BADC0DE {
                current_symtab_adjustment += symtab_offsets[current_offset_idx + 1];
                current_text_adjustment += text_offsets[current_offset_idx + 1];
                current_offset_idx += 1;
            }

            previous_r_offset = entry.r_offset;

            // Adjust offset into text section
            new_entry.r_offset = current_text_adjustment + entry.r_offset;

            // Adjust offset into symbol table
            new_entry.r_sym = current_symtab_adjustment + entry.r_sym;

            new_entry.to_bytes()
        })
        .collect();

    // Make a new ELF with the new .rel and .symtab
    let new_sections: Vec<Vec<u8>> = elf
        .sections
        .iter()
        .enumerate()
        .map(|(idx, section)| match idx {
            REL => new_rel_section.clone(),
            SYMTAB => new_symbol_table.clone(),
            _ => section.clone(),
        })
        .collect();

    return create_new_elf(new_sections, ElfType::Relocatable, false);
}

#[test]
fn validate_link_relocation() {
    // I understand there's some pretty hefty mocking going on here, but it's important that this works properly.

    // The situation given is as follows:
    // ELF 1 has an 8-byte text section with two relocation entries; one for each instruction.
    // ELF 2 has a relocation entry which references its first instruction.

    let mock_rel_table: Vec<RelocationEntry> = vec![
        RelocationEntry {
            r_offset: 0,
            r_sym: 0,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
        RelocationEntry {
            r_offset: 4,
            r_sym: 0,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
        RelocationEntry {
            r_offset: 0,
            r_sym: 0,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
    ];

    let mock_rel_section: Vec<u8> = mock_rel_table
        .iter()
        .flat_map(|entry| entry.to_bytes())
        .collect();

    let empty_symtab_entry: Elf32Sym = Elf32Sym {
        st_name: 0,
        st_value: 0,
        st_size: 0,
        st_info: 0,
        st_other: 0,
        st_shndx: 0,
    };

    let mock_symtab: Vec<Elf32Sym> = vec![
        empty_symtab_entry.clone(),
        Elf32Sym {
            st_name: 1,
            st_value: 0xDEADBEEF,
            st_size: 3,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
        empty_symtab_entry.clone(),
        Elf32Sym {
            st_name: 1,
            st_value: 0x8BADF00D,
            st_size: 4,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
    ];
    let mock_sections: Vec<Vec<u8>> = vec![
        vec![0u8; 8],
        vec![0u8; 8],
        mock_rel_section,
        mock_symtab
            .iter()
            .map(|symbol| symbol.to_bytes())
            .flatten()
            .collect(),
        vec![b'\0', b'h', b'i', b'\0', b'\0', b'm', b'o', b'm', b'\0'],
        vec![0u8; 32],
    ];

    let offsets = vec![vec![0, 0, 0, 0, 0, 0], vec![8, 8, 8, 32, 4, 16]];

    let mock_consolidated_elf: Elf = create_new_elf(mock_sections, ElfType::Relocatable, true);

    let tested: Elf = relocate_links(mock_consolidated_elf, &offsets);

    let expected_rel_table: Vec<RelocationEntry> = vec![
        RelocationEntry {
            r_offset: 0,
            r_sym: 0,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
        RelocationEntry {
            r_offset: 4,
            r_sym: 0,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
        RelocationEntry {
            r_offset: 8,
            r_sym: 32,
            r_type: name_core::elf_def::RelocationEntryType::R16,
        },
    ];

    let expected_rel_section: Vec<u8> = expected_rel_table
        .iter()
        .flat_map(|entry| entry.to_bytes())
        .collect();

    let expected_symtab: Vec<Elf32Sym> = vec![
        empty_symtab_entry.clone(),
        Elf32Sym {
            st_name: 1,
            st_value: 0xDEADBEEF,
            st_size: 3,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 4,
            st_value: 0,
            st_size: 0,
            st_info: 0,
            st_other: 1,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 5,
            st_value: 0x8BADF00D,
            st_size: 4,
            st_info: 0,
            st_other: 1,
            st_shndx: 0,
        },
    ];

    let expected_symtab_section: Vec<u8> = expected_symtab
        .iter()
        .map(|symbol| symbol.to_bytes())
        .flatten()
        .collect();

    assert_eq!(tested.sections[SYMTAB], expected_symtab_section);
    assert_eq!(tested.sections[REL], expected_rel_section);
}

/// Error type for check_duplicate_symbols. Contains associated duplicated name
#[derive(Debug, PartialEq, Clone)]
pub enum DuplicateSymbolError {
    Global(String),
    Local(String),
}

/// Pretty print for that error type
impl std::fmt::Display for DuplicateSymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuplicateSymbolError::Global(name) => {
                write!(f, "Duplicate symbol {name} found in global scope")
            }
            DuplicateSymbolError::Local(name) => {
                write!(f, "Duplicate symbol {name} found in local scope")
            }
        }
    }
}

/// This function checks first for duplicate global symbols,
/// then it checks for duplicate local symbols in the same local space (based on calculated offsets).
pub fn check_duplicate_symbols(elf: &Elf) -> Result<(), DuplicateSymbolError> {
    // Collect all symbols.
    let symtab: Vec<Elf32Sym> = parse_elf_symbols(&elf.sections[SYMTAB]);

    // Iterate through all symbols, collecting the linked names. If the name is already in that collection, error out.
    // If the symbol's value is 0, DO NOT COLLECT IT FOR DUPLICATE CHECKING. That's a placeholder.
    let global_names: Vec<String> = symtab
        .iter()
        .filter(|&symbol| symbol.st_value != 0)
        .filter_map(|symbol| match symbol.get_bind() {
            1 => Some(symbol.get_linked_name(&elf.sections[STRTAB])),
            _ => None,
        })
        .collect();

    // Duplicate checking goes on here
    let mut seen_global_names: HashSet<String> = HashSet::new();
    for name in global_names {
        if !seen_global_names.insert(name.clone()) {
            return Err(DuplicateSymbolError::Global(name));
        }
    }

    // Take collected symbols. Break them up by linked scope into a Vec<Vec<Elf32Sym>>.
    // We know what scope each symbol belongs to from the st_other field.
    let num_of_scopes: usize = symtab[symtab.len() - 1].st_other as usize + 1;
    let mut scoped_symbols: Vec<Vec<Elf32Sym>> = vec![vec![]; num_of_scopes];

    symtab
        .iter()
        .for_each(|symbol| scoped_symbols[symbol.st_other as usize].push(symbol.clone()));

    // Iterate through each scope, collecting the linked names. If the name is already in that collection, error out.
    for scope in scoped_symbols {
        // Re-used name collection logic, but remove placeholder check and change match to local symbols
        let scope_names: Vec<String> = scope
            .iter()
            .filter_map(|symbol| match symbol.get_bind() {
                0 => Some(symbol.get_linked_name(&elf.sections[STRTAB])),
                _ => None,
            })
            .collect();

        // Re-use the hashset logic
        let mut seen_local_names: HashSet<String> = HashSet::new();
        for name in scope_names {
            if !seen_local_names.insert(name.clone()) {
                return Err(DuplicateSymbolError::Local(name));
            }
        }
    }

    // If we made it down here, the symbols are ok.

    Ok(())
}

#[test]
fn validate_duplicate_symbol_checker() {
    let mock_symtab: Vec<Elf32Sym> = vec![
        Elf32Sym {
            st_name: 0,
            st_value: 0,
            st_size: 0,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 1,
            st_value: 0xDEADBEEF,
            st_size: 4,
            st_info: 1 << 4,
            st_other: 0,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 5,
            st_value: 0x8BADF00D,
            st_size: 4,
            st_info: 1 << 4,
            st_other: 0,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 9,
            st_value: 0x2EADBEEF,
            st_size: 4,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
        Elf32Sym {
            st_name: 13,
            st_value: 0x9BADF00D,
            st_size: 4,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
        },
    ];

    // In the first case, all global and local symbols have the same name: "cat". Global issues are thrown first.
    let bad_global_strtab: Vec<u8> = vec![
        b'\0', b'c', b'a', b't', b'\0', b'c', b'a', b't', b'\0', b'c', b'a', b't', b'\0', b'c',
        b'a', b't', b'\0',
    ];

    let mock_sections: Vec<Vec<u8>> = vec![
        vec![0; 4],
        vec![0; 16],
        vec![0; 8],
        mock_symtab
            .iter()
            .map(|symbol| symbol.to_bytes())
            .flatten()
            .collect(),
        bad_global_strtab.clone(),
        vec![0; 16],
    ];

    let mut mock_elf: Elf = create_new_elf(mock_sections, ElfType::Relocatable, true);

    let result: Result<(), DuplicateSymbolError> = check_duplicate_symbols(&mock_elf);

    // This should error out due to the duplicate global symbols
    assert_eq!(
        result,
        Err(DuplicateSymbolError::Global(String::from("cat")))
    );

    // Second test - duplicate local names. I fixed my global symbols, now what?
    let bad_local_strtab: Vec<u8> = vec![
        b'\0', b'c', b'a', b't', b'\0', b'd', b'o', b'g', b'\0', b'c', b'a', b't', b'\0', b'c',
        b'a', b't', b'\0',
    ];

    mock_elf.sections[STRTAB] = bad_local_strtab.clone();

    let result: Result<(), DuplicateSymbolError> = check_duplicate_symbols(&mock_elf);

    assert_eq!(
        result,
        Err(DuplicateSymbolError::Local(String::from("cat")))
    );

    // Final test - everything's fine now.
    let fine_strtab: Vec<u8> = vec![
        b'\0', b'c', b'a', b't', b'\0', b'd', b'o', b'g', b'\0', b'c', b'a', b't', b'\0', b'd',
        b'o', b'g', b'\0',
    ];

    mock_elf.sections[STRTAB] = fine_strtab.clone();

    let result: Result<(), DuplicateSymbolError> = check_duplicate_symbols(&mock_elf);

    assert_eq!(result, Ok(()));
}
