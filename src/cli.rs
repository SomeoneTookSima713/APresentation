use clap::{ Parser, Subcommand };

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Command,
    /// Path to the file to operate on
    pub file: String,
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Opens and presents a presentation
    Present,
    /// Generates a new presentation using a template
    Generate
}