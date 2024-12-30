use name_core::elf_def::*;

/// A wrapper over relocatable_conformity_check to simplify main()
pub fn conformity_check(elfs: &Vec<Elf>) -> Result<(), String> {
    let conformity_results: Vec<Result<(), String>> = elfs
        .iter()
        .map(|elf| relocatable_conformity_check(elf))
        .collect();

    let mut errors: Vec<String> = vec![];

    for res in conformity_results {
        match res {
            Ok(_) => {}
            Err(e) => errors.push(e),
        }
    }

    if errors.len() > 0 {
        let error_string: String = errors.iter().map(|err| format!("- {err}\n")).collect();
        return Err(format!("Conformity checks failed:\n {error_string}"));
    } else {
        return Ok(());
    }
}

fn relocatable_conformity_check(et_rel: &Elf) -> Result<(), String> {
    let fh = &et_rel.file_header;

    // Check everything in E_IDENT first
    if let Err(e) = check_e_ident(&et_rel.file_header.e_ident) {
        return Err(e);
    };

    if fh.e_type != ET_REL {
        return Err(format!(
            "Linker expected a relocatable object file ({}), found e_type field {}.",
            E_TYPE_DEFAULT, fh.e_type
        ));
    }

    if fh.e_machine != E_MACHINE_DEFAULT {
        return Err(format!(
            "NAME only supports big-endian MIPS ({}), received {}.",
            E_MACHINE_DEFAULT, fh.e_machine
        ));
    }

    if fh.e_phoff != E_PHOFF_DEFAULT {
        return Err(format!(
            "Non-compliant file header size detected. Ensure 32-bit format."
        ));
    }

    if fh.e_flags != E_FLAGS_DEFAULT {
        return Err(format!(
            "Linker expected flags {}, received {}.",
            E_FLAGS_DEFAULT, fh.e_flags
        ));
    }

    if fh.e_ehsize != E_EHSIZE_DEFAULT {
        return Err(format!(
            "Non-compliant file header size detected. Ensure 32-bit format."
        ));
    }

    if fh.e_phentsize != E_PHENTSIZE_DEFAULT {
        return Err(format!(
            "Non-compliant program header entry size detected. Ensure 32-bit format."
        ));
    }

    if fh.e_phnum != E_PHNUM_DEFAULT {
        return Err(format!("Linker expected a very specific section layout. If you're making your own ELF, refer to documentation (bad e_phnum field, expected {}).", E_PHNUM_DEFAULT));
    }

    if fh.e_shentsize != E_SHENTSIZE_DEFAULT {
        return Err(format!(
            "Non-compliant section header entry size detected. Ensure 32-bit format."
        ));
    }

    if fh.e_shnum != E_SHNUM_DEFAULT_REL {
        return Err(format!("Linker expected a very specific section layout. If you're making your own ELF, refer to documentation (bad e_shnum field, expected {}).", E_SHNUM_DEFAULT_REL));
    }

    if fh.e_shstrndx != E_SHSTRNDX_DEFAULT_REL {
        return Err(format!("Linker expected a very specific section layout. If you're making your own ELF, refer to documentation (bad e_shstrndx field, expected {}).", E_SHSTRNDX_DEFAULT_REL));
    }

    Ok(())
}

fn check_e_ident(e_ident: &[u8]) -> Result<(), String> {
    if e_ident[0..4] != E_IDENT_DEFAULT[0..4] {
        return Err(format!(
            "Magic bytes did not match for ELF format (EI_MAG field expected {:x}).",
            u32::from_be_bytes(E_IDENT_DEFAULT[0..4].try_into().unwrap())
        ));
    }

    if e_ident[5] != E_IDENT_DEFAULT[5] {
        return Err(format!(
            "NAME currently only supports 32-bit binaries (EI_CLASS field expected {}).",
            EI_CLASS
        ));
    }

    if e_ident[6] != E_IDENT_DEFAULT[6] {
        return Err(format!(
            "NAME only supports big-endian data (EI_DATA field expected {}).",
            EI_DATA
        ));
    }

    // Version field don't matter

    if e_ident[8] != E_IDENT_DEFAULT[8] {
        return Err(format!("OSABI expected {}", EI_OSABI));
    }

    // abi version is unused by NAME
    // pad doesn't need to be empty so steganography possibility i guess

    Ok(())
}
