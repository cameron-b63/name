use crate::conformity::relocatable_conformity_check;

use name_core::constants::MIPS_TEXT_START_ADDR;
use name_core::elf_def::{Elf, ET_EXEC};
use name_core::elf_utils::{find_global_symbol_address, parse_elf_symbols};

pub fn one_module_linker(et_rel: Elf) -> Result<Elf, String> {
    let mut et_exec: Elf = match relocatable_conformity_check(&et_rel) {
        Ok(_) => et_rel.clone(),
        Err(e) => return Err(e),
    };

    // Update executable type (needed for name-emu to recognize file)
    et_exec.file_header.e_type = ET_EXEC;

    // Increment executable version
    et_exec.file_header.e_version += 1;

    // Update entry address
    et_exec.file_header.e_entry = MIPS_TEXT_START_ADDR;

    // If the global symbol "main" exists, that should be the entry address. It still needs to be global even in one module by convention.

    // Construct symbol table and corresponding string table from what was given
    let symbol_table = parse_elf_symbols(&et_exec.sections[3]);
    let str_table = &et_exec.sections[4];

    et_exec.file_header.e_entry = match find_global_symbol_address(&symbol_table, str_table, "main")
    {
        Some(new_address) => new_address,
        None => MIPS_TEXT_START_ADDR,
    };

    Ok(et_exec)
}
