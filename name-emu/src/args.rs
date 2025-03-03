use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    pub input_filename: std::path::PathBuf,

    #[arg(short, long, help = "Enable debug mode")]
    pub debug: bool,

    #[arg(short, long, help = "Send debugger output to stderr and user syscall outputs to stdout")]
    pub separate_io_channels: bool,
}
