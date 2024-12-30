// This file is responsible for the heavy lifting - performing the actual relocation process (and construction of a new ELF).
// Note that it's almost impossible to create a mock test that's more useful than

use name_core::{
    elf_def::{Elf, ElfType},
    elf_utils::create_new_elf,
};

use crate::{
    relocate_helpers::{check_duplicate_symbols, relocate_links},
    text_relocation::relocate_text_entries,
};

pub fn relocate(sections: Vec<Vec<u8>>, offsets: Vec<Vec<u32>>) -> Result<Elf, String> {
    // Relocation is the process of putting things where they ought to go.
    // The first thing to do is create a mutable ELF (it will still be relocatable). Now that the sections have been consolidated, we just need to operate on the structured data.
    let new_elf: Elf = create_new_elf(sections, ElfType::Relocatable, true);

    // Now that the ELF is mutable, the first thing to do is fix the offsets for the .symtab links into .strtab.
    // We'll also need to fix the .rel links into .text - as well as the .rel links into .symtab.
    // Each symbol needs to have the appropriate offset added to fix the indexing.
    // We also need to embed information about the scoping of symbols (i.e. which executable they're from).
    // This has been extracted to a function.
    let relocated_symtab: Elf = relocate_links(new_elf, &offsets);

    // Next, duplicate global symbols need to be checked for, as well as duplicate, same-scope local symbols.
    // This has also been extracted to a function.
    check_duplicate_symbols(&relocated_symtab).map_err(|e| e.to_string())?;

    // Now, each entry in .rel needs to be reconciled.
    // This is a complex process, so it's also been extracted to a function.
    let relocated: Elf =
        relocate_text_entries(relocated_symtab, &offsets).map_err(|e| e.to_string())?;

    // Now that each entry in .rel has been reconciled, there's nothing left to do.
    Ok(relocated)
}
