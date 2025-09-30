use clap::Parser;
use name_as::args::Cli;

#[derive(Parser, Debug)]
struct Args {
    pub path: std::path::PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let mut output = args.path.clone();
    let _ = output.set_extension("o");

    let assemble_args = Cli {
        input_filename: args.path.clone(),
        output_filename: output,
        verbose: false,
    };

    let elf = name_as::run_assembler(&assemble_args);

    // Invoke linker on collected Elfs
    let executable_contents = match name_ld::linker::linker(vec![elf]) {
        Ok(elf) => elf,
        Err(e) => panic!("{e}"),
    };

    dbg!(&executable_contents);

    name_emu::simulator::simulate(executable_contents, false, false)
}
