use std::path::PathBuf;

use name_as::args::Cli;
use name_as::assembler::assemble_file::assemble_file;

use name_core::{
    elf_def::ElfType,
    elf_utils::{create_new_elf, extract_symbol_table_to_sections, write_elf_to_file},
    parse::session::Session,
};

use bumpalo::Bump;

use clap::Parser;

fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    // Grab the input file from args and get the parent directory
    let mut dir_path = PathBuf::from(&args.input_filename);
    dir_path.pop();

    // Bump allocation for strings in the parsing session.
    // Using bump allocation solves so many issues with lifetimes.
    // Using an area of memory as our own sandbox.
    // Ownership is tied to the parser session.
    let bump = Bump::new();
    // Create a new parser session to own the bump.
    let mut session = Session::new(&bump, dir_path);

    // Assembler entry point
    let assembler_environment =
        assemble_file(&mut session, args.input_filename).unwrap_or_else(|_e| {
            std::process::exit(1);
        });

    // Structure the assembler symbol table into ELF sections.
    let (section_dot_symtab, section_dot_strtab) =
        extract_symbol_table_to_sections(assembler_environment.symbol_table);

    // Structure the assembler output into proper ELF sections.
    let et_rel = create_new_elf(
        vec![
            assembler_environment.section_dot_data,
            assembler_environment.section_dot_text,
            assembler_environment.section_dot_rel,
            section_dot_symtab,
            section_dot_strtab,
            assembler_environment.section_dot_line,
        ],
        ElfType::Relocatable,
        true,
    );

    // Write ELF object file to disk given passed filename.
    match write_elf_to_file(&args.output_filename, &et_rel) {
        Ok(()) => println!(
            "[+] Object file successfuly written to {:?}",
            args.output_filename
        ),
        Err(e) => {
            eprintln!("{}", e);
            panic!();
        }
    }

    println!("[+] Assembly was successful.");
}
