use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "Convert Audio files into Images using TIFF Headers and RAW Data Manipulation.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Convert {
        #[arg(help = "Input audio file (MP3)")]
        input: String,

        #[arg(short, long, help = "Output image file (defaults to input filename with .png extension)")]
        output: Option<String>,

        #[arg(long, default_value_t = 1024, help = "Image width in pixels")]
        width: u32,

        #[arg(long, default_value_t = 768, help = "Image height in pixels")]
        height: u32,
    },

    Inspect {
        #[arg(short, long, help = "Input file to inspect")]
        input: String,
    },
}
