mod args;
mod assembler;
mod assembly_utils;
mod assembly_helpers;
mod assemble_instruction;
mod lineinfo;
mod parser;
mod tokens;

use args::Cli;
use assembler::assemble;
use assembly_helpers::extract_symbol_table_to_sections;
use lineinfo::get_lineinfo;

use name_const::structs::LineInfo;
use name_const::elf_utils::{create_new_et_rel, write_et_rel_to_file};

use clap::Parser;

fn main() {
    let args = Cli::parse();
    let file_contents: String = std::fs::read_to_string(args.input_filename).expect("Failed to read input file (likely does not exist).");

    if args.lines {
        let _section_dot_line: Vec<LineInfo> = get_lineinfo(&file_contents).expect("NAME will not assemble an empty file.");
    }

    // Preprocessor would do its work in macro/pseudoinstruction expansion here

    // Allowing assemble to take ownership of the source file contents, because this is the end of its utility in this function.
    let assembled_result = assemble(file_contents);
    match assembled_result {
        Ok((section_dot_text, symbol_table)) => {
            let section_dot_data: Vec<u8> = vec!();
            let (section_dot_symtab, section_dot_strtab) = extract_symbol_table_to_sections(symbol_table);

            let et_rel = create_new_et_rel(section_dot_text, section_dot_data, section_dot_symtab, section_dot_strtab);
            match write_et_rel_to_file(&args.output_filename, &et_rel) {
                Ok(()) => println!("Object file successfuly written to {:?}", args.output_filename),
                Err(e) => {
                    eprintln!("{}", e);
                    panic!();
                }
            }

            println!("Assembly was successful.");
        },
        Err(errors) => {
            eprintln!("Errors were encountered during assembly: \n");
            let joined_errors = errors.join("\n");
            eprintln!("{joined_errors}");
            
            // This exit with a bad exit code tells the vscode extension to not bother with linking or emulation.
            std::process::exit(1);
        }
    }
}

#[test]
fn full_integration_test() {
    let test_file_path = "/home/teqqy/Projects/name/test_files/instruction_demonstration/mips_test.asm";
    let test_output_filename: std::path::PathBuf = std::path::PathBuf::from("/home/teqqy/Projects/name/test_files/instruction_demonstration/mips_test.o");

    let file_contents: String = std::fs::read_to_string(test_file_path).expect("Failed to read input file (likely does not exist).");

    let assembled_output = assemble(file_contents);

    match assembled_output {
        Ok((section_dot_text, symbol_table)) => {
            let section_dot_data: Vec<u8> = vec!();
            let (section_dot_symtab, section_dot_strtab) = extract_symbol_table_to_sections(symbol_table);

            let et_rel = create_new_et_rel(section_dot_text, section_dot_data, section_dot_symtab, section_dot_strtab);
            match write_et_rel_to_file(&test_output_filename, &et_rel) {
                Ok(()) => println!("Object file successfuly written to {:?}.", test_output_filename),
                Err(e) => {
                    eprintln!("{}", e);
                    panic!();
                }
            }

            println!("Assembly was successful.");
        },
        Err(errors) => {
            eprintln!();
            eprintln!("Errors were encountered during assembly: \n");
            let joined_errors = errors.join("\n");
            eprintln!("{joined_errors}");
        },
    }
}