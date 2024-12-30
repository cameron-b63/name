use std::fs::read_to_string;
use std::path::PathBuf;

use name_as::assembler::assemble_file::assemble;

use name_core::{
    elf_def::ElfType,
    elf_utils::{create_new_elf, extract_symbol_table_to_sections, write_elf_to_file},
};

#[test]
fn fib_as_test() {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("[*] FATAL: No parent directory found (did you clone the entire repo?)")
        .join("tests")
        .join("samples");

    let test_file_path: PathBuf = base_path.join("fib.asm");
    let test_output_filename: PathBuf = base_path.join("fib.o");

    let file_contents: String =
        read_to_string(test_file_path).expect("Failed to read input file (likely does not exist).");

    let assembled_output = assemble(file_contents, base_path, None);

    match assembled_output {
        Ok(assembler_environment) => {
            let (section_dot_symtab, section_dot_strtab) =
                extract_symbol_table_to_sections(assembler_environment.symbol_table);

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
            match write_elf_to_file(&test_output_filename, &et_rel) {
                Ok(()) => println!(
                    "Object file successfuly written to {:?}.",
                    test_output_filename
                ),
                Err(e) => {
                    eprintln!("{}", e);
                    panic!();
                }
            }

            println!("Assembly was successful.");
        }
        Err(errors) => {
            eprintln!();
            eprintln!(
                "The following errors were encountered during assembly (others slipped by): \n"
            );
            let joined_errors = errors.join("\n");
            eprintln!("{joined_errors}");

            println!("\nAssembly was unsuccessful.");
        }
    }
}
