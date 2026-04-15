use clap::Parser;
use std::fs;
use tiffiny::{cli, ImageGenerator, utils};

fn main() -> Result<(), ()> {
    let cli_args = cli::Cli::parse();

    match cli_args.command {
        cli::Commands::Convert { input, output, width, height } => {
            let clean_input = utils::normalize(&utils::strip_quotes(&input));
            let clean_output = match output {
                Some(out) => utils::normalize(&utils::strip_quotes(&out)),
                None => {
                    let input_path = std::path::Path::new(&clean_input);
                    let stem = input_path.file_stem().unwrap_or_default().to_str().unwrap_or("output");
                    format!("{}.png", stem)
                }
            };
            ImageGenerator::audio_to_image(&clean_input, &clean_output, width, height)?;
            let absolute_path = std::fs::canonicalize(&clean_output).unwrap_or_else(|_| clean_output.clone().into());
            println!("Image written to {}", absolute_path.display());
        }

        cli::Commands::Inspect { input } => {
            let clean_input = utils::normalize(&utils::strip_quotes(&input));
            let metadata = fs::metadata(&clean_input).unwrap();
            let data = fs::read(&clean_input).unwrap();

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