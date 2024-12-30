// This file contains methods responsible for consolidating the sections of the ELF.

use name_core::elf_def::Elf;

use crate::constants::DATA;

/// Consolidate the ELF sections that will appear in the final ET_EXEC.
/// No relocation is performed at this time.
/// No further destructuring is performed at this time.
/// This is a simple concatenation with pad.
pub fn consolidate_sections(elfs: Vec<Elf>, offsets: &Vec<Vec<u32>>) -> Vec<Vec<u8>> {
    // Consolidating sections means creating new Vec<u8>'s for each section,
    // and padding with zeroes for alignment when necessary (I'm looking at you, .data!).

    // Initialize the return vector.
    let mut return_vector: Vec<Vec<u8>> = vec![];

    // For each section, match on the section:
    let mut current_section: usize = 0;

    while current_section < offsets[0].len() {
        match current_section {
            DATA => {
                // Data -> Consolidate the .data bytes. This means that the vector to extend by will be padded with zeros.
                let data_offsets: Vec<u32> = offsets
                    .iter()
                    .map(|elf_offsets| elf_offsets[current_section])
                    .collect();
                let pads: Vec<usize> = elfs
                    .iter()
                    .enumerate()
                    .map(|(idx, elf)| {
                        if idx != elfs.len() - 1 {
                            (data_offsets[idx + 1]
                                - elf.section_header_table[current_section].sh_size)
                                as usize
                        } else {
                            0
                        }
                    })
                    .collect();
                let padded_datas: Vec<u8> = elfs
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, elf)| {
                        vec![elf.sections[current_section].clone(), vec![0u8; pads[idx]]]
                    })
                    .flatten()
                    .collect();
                return_vector.push(padded_datas);
            },
            _ => {
                // Anything else -> one-liner.
                return_vector.push(
                    elfs.iter()
                        .flat_map(|elf| elf.sections[current_section].clone())
                        .collect(),
                );
            }
        }

        current_section += 1;
    }

    return return_vector;
}

#[test]
fn validate_consolidation() {
    let elf1: Elf = name_core::elf_utils::create_new_elf(
        vec![vec![0u8; 37]; 6],
        name_core::elf_def::ElfType::Relocatable,
        true,
    );
    let elf2: Elf = elf1.clone();

    let elfs: Vec<Elf> = vec![elf1, elf2];

    let offsets: Vec<Vec<u32>> = vec![
        vec![0, 37],
        vec![0, 40],
        vec![0, 37],
        vec![0, 37],
        vec![0, 37],
        vec![0, 37],
    ];

    let result: Vec<Vec<u8>> = consolidate_sections(elfs, &offsets);

    // .text, along with any other section that is not .data, will have this property after consolidation.
    assert_eq!(result[0], vec![0u8; 37 * 2]);

    // for n ELF files, the first (n-1) sections in the consolidated form will be padded. The last need not be.
    assert_eq!(result[1], vec![0u8; 40 + 37]);
}
