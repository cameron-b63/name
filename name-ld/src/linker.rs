/// This file contains the linker logic. If only one file was provided, it will invoke the far simpler single module linker
use name_core::elf_def::Elf;

use crate::{
    conformity::conformity_check, consolidate::consolidate_sections, offsets::calculate_offsets,
    relocate::relocate,
};

/// The driver code for the linker is as follows:
/// Check the ELFs, calculate section offsets for each ELF, consolidate the sections, then perform relocation.
/// Details inside.
pub fn linker(elfs: Vec<Elf>) -> Result<Elf, String> {
    // Ensure each ELF conforms to the correct standard
    conformity_check(&elfs)?;

    // Now that we know each ELF conforms to standard, we can try to do some work:

    // Retrieve offsets for each executable, each section.
    // Takes in the list of checked ELFs
    // Returns a Vec<Vec<u32>> representing the offsets for each section of each ELF in the resulting executable.
    // This should be infallible due to the conformity checks, but I haven't verified this.
    let offsets: Vec<Vec<u32>> = calculate_offsets(&elfs);

    // Using offsets, consolidate each section. Update appropriate indices (think .strtab, etc)
    // Takes in the list of checked ELFs (consumed) along with the start offsets for each section (borrowed)
    // Returns a Result<Vec<Vec<u8>>>, with Ok(_) representing the consolidated sections for use in relocation and construction of the final executable.
    // Infallible because this stage represents a naive concatenation with pad.
    let consolidated_sections: Vec<Vec<u8>> = consolidate_sections(elfs, &offsets);

    // Now that final segments are constructed, use relocation information in .rel to complete the relocation process.
    // Takes in the consolidated sections (consumed) and offsets (consumed)
    // Returns a Result<Elf>, with Ok(_) representing the relocated ELF.
    // Fallible due to issues like a label being undefined in scope.
    return relocate(consolidated_sections, offsets);
}
