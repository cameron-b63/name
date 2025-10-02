use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
struct Args {
    pub path: Option<std::path::PathBuf>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let path = args
        .path
        .unwrap_or(env::current_dir().expect("Could not find a path to assemble"));

    let mut input_files = vec![];

    if path.is_dir() {
        for entry in path.read_dir().expect("Could not read dir") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "asm") {
                    input_files.push(path);
                }
            }
        }
    } else if path.is_file() && path.extension().is_some_and(|ext| ext == "asm") {
        input_files.push(path);
    }

    if input_files.is_empty() {
        eprintln!("Could not find files to asemble");
        std::process::exit(1);
    }

    let elfs = input_files
        .into_iter()
        .map(|file| name_as::run_assembler(file))
        .collect();

    let executable_contents = match name_ld::linker::linker(elfs) {
        Ok(elf) => elf,
        Err(e) => panic!("{e}"),
    };

    name_emu::simulator::simulate(executable_contents, false, false)
}
