// This file contains the logic for calculating the offsets associated with each section in the resulting ELF.

use name_core::elf_def::Elf;

use crate::constants::DATA;

/// Calculate the offsets of each ELF's section in the final ET_EXEC executable.
/// The return vector is essentially just ELF -> section -> offset.
/// The return vector's sections (the inner vector) are going to be formatted as follows:
/// [.text, .data, .rel .symtab, .strtab, .line <.shstrtab handled later>]
pub fn calculate_offsets(elfs: &Vec<Elf>) -> Vec<Vec<u32>> {
    // Efficient approach:
    // Initialize the final Vec<Vec<u32>> with a Vec<u32> containing the starting values for the first ELF.
    let mut return_data: Vec<Vec<u32>> = vec![vec![
        0, // .text
        0, // .data
        0, // .rel
        0, // .symtab
        0, // .strtab
        0, // .line
    ]];

    // for each ELF that is not the last, add the size of the current ELF's section to the current ELF's offset to discover the next offset.
    for (idx, elf) in elfs.into_iter().enumerate() {
        if idx != elfs.len() - 1 {
            let mut next_offsets: Vec<u32> = vec![];
            let mut j: usize = 0;

            while j < return_data[0].len() {
                // j+1 is due to the NULL section existing in the section table but being disregarded by the Elf deserializer.
                next_offsets.push(
                    // Match on the section, as .data must remain aligned.
                    match j {
                        DATA => {
                            // Word-align .data's offset
                            ((return_data[idx][j] + elf.section_header_table[j + 1].sh_size + 4)
                                >> 3)
                                << 3
                        }
                        _ => return_data[idx][j] + elf.section_header_table[j + 1].sh_size,
                    },
                );
                j += 1;
            }

            return_data.push(next_offsets);
        }
    }

    // return the final Vec<Vec<u32>>
    return return_data;
}

// #[test]
// fn verify_calculate_offsets() {
//     // Mock sections are just test_num 0's per section, causing the new offsets to be test_num.
//     let test_num: usize = 63;
//     // Data must be aligned.
//     let data_num: usize = (test_num + 4) >> 3 << 3;
//
//     let mock_sections: Vec<Vec<u8>> = vec![vec![0u8; test_num]; 7];
//     let elf1: Elf = name_core::elf_utils::create_new_elf(
//         mock_sections,
//         name_core::elf_def::ElfType::Relocatable,
//         true,
//     );
//     let elf2: Elf = elf1.clone();
//
//     let res = calculate_offsets(&vec![elf1, elf2]);
//
//     // Offsets calculated properly
//     assert_eq!(
//         res,
//         vec![
//             vec![0u32; 6],
//             vec![
//                 test_num as u32,
//                 data_num as u32,
//                 test_num as u32,
//                 test_num as u32,
//                 test_num as u32,
//                 test_num as u32
//             ]
//         ]
//     );
//
//     // Data alignment performed properly
//     assert_eq!(data_num, 64);
// }
