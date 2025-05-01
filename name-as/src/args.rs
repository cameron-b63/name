use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    pub input_filename: std::path::PathBuf,
    pub output_filename: std::path::PathBuf,

    #[arg(short, long)]
    pub verbose: bool,
}

// Test change 