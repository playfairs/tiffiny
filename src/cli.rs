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

        #[arg(help = "Output image file")]
        output: String,

        #[arg(short, long, default_value_t = 512, help = "Image width in pixels")]
        width: u32,

        #[arg(short = 'H', long, default_value_t = 512, help = "Image height in pixels")]
        height: u32,

        #[arg(long, default_value = "rgb", help = "Output format: rgb, rgba")]
        format: String,
    },

    Inspect {
        #[arg(short, long, help = "Input file to inspect")]
        input: String,
    },
}
