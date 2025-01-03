use std::path::PathBuf;

use name_as::args::Cli;
use name_as::assembler::assembler::Assembler;

use name_core::{
    elf_def::ElfType,
    elf_utils::{create_new_elf, extract_symbol_table_to_sections, write_elf_to_file},
};

use clap::Parser;

fn main() {
    let args = Cli::parse();

    let mut dir_path = PathBuf::from(&args.input_filename);
    dir_path.pop();

    let mut assembler_environment = Assembler::new(dir_path);

    let success = assembler_environment.assemble_file(&args.input_filename);

    if !success {
        std::process::exit(1);
    }

    let (section_dot_symtab, section_dot_strtab) =
        extract_symbol_table_to_sections(assembler_environment.symbol_table);

    let et_rel = create_new_elf(
        vec![
            assembler_environment.section_dot_data,
            assembler_environment.section_dot_text,
            assembler_environment.section_dot_rel, // Placeholder for .rel
            section_dot_symtab,
            section_dot_strtab,
            assembler_environment.section_dot_line,
        ],
        ElfType::Relocatable,
        true,
    );

    match write_elf_to_file(&args.output_filename, &et_rel) {
        Ok(()) => println!(
            "Object file successfuly written to {:?}",
            args.output_filename
        ),
        Err(e) => {
            eprintln!("{}", e);
            panic!();
        }
    }

    println!("Assembly was successful.");
}
