use clap::Parser;
use name_as::args::Cli;
use name_as::run_assembler;
use name_core::elf_utils::write_elf_to_file;

fn main() {
    // Parse command line arguments
    let args = Cli::parse();
    let et_rel = run_assembler(args.input_filename);
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
