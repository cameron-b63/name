use name_emu::args::Cli;
use name_emu::simulator::simulate;

use name_core::elf_def::Elf;
use name_core::elf_utils::read_bytes_to_elf;

use clap::Parser;

use std::fs::read;

pub fn main() {
    let args = Cli::parse();

    let elf_contents: Vec<u8> = read(args.input_filename).expect("File not found");

    let executable: Elf = match read_bytes_to_elf(elf_contents) {
        Ok(elf) => elf,
        Err(e) => panic!("{}", e),
    };

    let _ = simulate(executable, args.debug);
}
