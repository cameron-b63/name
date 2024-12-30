use name_ld::args::Cli;
use name_ld::linker::linker;
use std::path::PathBuf;

use name_core::elf_def::Elf;
use name_core::elf_utils::{read_bytes_to_elf, write_elf_to_file};

use clap::Parser;

fn main() {
    // Take in all object files as cli arguments
    let args: Cli = Cli::parse();
    let base_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("[*] FATAL: No parent directory found (did you clone the entire repo?)")
        .to_path_buf();

    // match on arguments provided
    match args.input_filenames.len() {
        0 => {
            // Err
            panic!("[*] FATAL: No input files supplied.");
        }
        _ => {
            // Invoke main linker

            // Collect input filenames
            let multi_module_input_fns: Vec<PathBuf> = args
                .input_filenames
                .iter()
                .map(|filename| base_path.join(filename))
                .collect();

            // Generate output filename
            let multi_module_output_fn = base_path.join(args.output_filename);

            // Deserialize each input file, put into vector for linking
            let mut elf_vector: Vec<Elf> = vec![];
            for filename in multi_module_input_fns {
                // get file contents
                let single_file_contents: Vec<u8> =
                    std::fs::read(filename).expect("Unable to open object file.");

                match read_bytes_to_elf(single_file_contents) {
                    Ok(relocatable) => elf_vector.push(relocatable),
                    Err(e) => panic!("{e}"),
                };
            }

            // Invoke linker on collected Elfs
            let executable_contents: Elf = match linker(elf_vector) {
                Ok(elf) => elf,
                Err(e) => panic!("{e}"),
            };

            // Write output to file
            match write_elf_to_file(&multi_module_output_fn, &executable_contents) {
                Ok(_) => println!("Linking performed successfully."),
                Err(e) => panic!("{e}"),
            }
        }
    }
}
