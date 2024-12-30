use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// List of input files to link
    #[arg(required = true)]
    pub input_filenames: Vec<std::path::PathBuf>,

    /// Output file to generate (required)
    #[arg(short, long, required = true)]
    pub output_filename: std::path::PathBuf,
}
