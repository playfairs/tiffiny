use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "Audio to Image Converter - Transform audio files into visual representations", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Convert {
        #[arg(help = "Input audio file (supports MP3, WAV, FLAC, etc.)")]
        input: String,

        #[arg(short, long, help = "Output image file (defaults to input filename with .png extension)")]
        output: Option<String>,

        #[arg(long, default_value_t = 1024, help = "Image width in pixels")]
        width: u32,

        #[arg(long, default_value_t = 768, help = "Image height in pixels")]
        height: u32,

        #[arg(long, default_value = "raw", help = "Encoding type: raw (direct mapping), waveform (amplitude over time), spectrogram (frequency visualization)")]
        encoding: String,

        #[arg(long, default_value_t = 1, help = "Number of audio channels to process (1=mono, 2=stereo)")]
        channels: u32,

        #[arg(long, default_value = "rgb", help = "Color scheme: rgb (default), grayscale, heat (intensity map), rainbow (frequency spectrum)")]
        color_scheme: String,

        #[arg(long, default_value = "linear", help = "Visualization scaling: linear (direct), log (enhances quiet sounds), sqrt (moderate enhancement)")]
        scaling: String,
    },

    Inspect {
        #[arg(short, long, help = "Input file to inspect")]
        input: String,
    },

    PathSet {
        #[arg(help = "Default output directory for converted images")]
        path: String,
    },
}
