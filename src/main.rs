use anyhow::Result;
use clap::Parser;
use std::fs;
use tiffiny::{cli, ImageGenerator, utils};

fn main() -> Result<()> {
    let cli_args = cli::Cli::parse();

    match cli_args.command {
        cli::Commands::Convert { input, output, width, height, format: _ } => {
            let clean_input = utils::strip_quotes(&input);
            let clean_output = utils::strip_quotes(&output);
            ImageGenerator::audio_to_image(&clean_input, &clean_output, width, height)?;
            println!("Image written to {}", clean_output);
        }

        cli::Commands::Inspect { input } => {
            let clean_input = utils::strip_quotes(&input);
            let metadata = fs::metadata(&clean_input)?;
            let data = fs::read(&clean_input)?;

            println!("tiffiny::inspect");
            println!("  file: {}", clean_input);
            println!("  size: {} bytes", metadata.len());

            let entropy = utils::calculate_entropy(&data);
            println!("  entropy: {:.4}", entropy);
            println!("  preview (first 16 bytes):");

            for byte in data.iter().take(16) {
                print!("{:02X} ", byte);
            }
            println!();
        }
    }

    Ok(())
}