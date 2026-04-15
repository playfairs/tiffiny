use clap::Parser;
use std::fs;
use cli::cli;
use core::{Config, utils};
use image::ImageGenerator;

fn main() -> Result<(), ()> {
    let cli_args = cli::Cli::parse();

    match cli_args.command {
        cli::Commands::Convert { input, output, width, height, encoding, channels, color_scheme, scaling } => {
            let clean_input = utils::normalize(&utils::strip_quotes(&input));
            let config = Config::load();
            
            let clean_output = match output {
                Some(out) => utils::normalize(&utils::strip_quotes(&out)),
                None => {
                    let input_path = std::path::Path::new(&clean_input);
                    let stem = input_path.file_stem().unwrap_or_default().to_str().unwrap_or("output");
                    let filename = format!("{}.png", stem);
                    
                    if let Some(output_dir) = config.get_output_directory() {
                        let output_path = std::path::Path::new(output_dir).join(filename);
                        output_path.to_string_lossy().to_string()
                    } else {
                        filename
                    }
                }
            };
            ImageGenerator::audio_to_image(&clean_input, &clean_output, width, height, &encoding, channels, &color_scheme, &scaling)?;
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

        cli::Commands::PathSet { path } => {
            let mut config = Config::load();
            let clean_path = utils::normalize(&utils::strip_quotes(&path));
            
            if let Err(e) = std::fs::create_dir_all(&clean_path) {
                eprintln!("Error creating directory: {}", e);
                return Err(());
            }
            
            config.set_output_directory(&clean_path);
            if let Err(e) = config.save() {
                eprintln!("Error saving config: {}", e);
                return Err(());
            }
            
            let absolute_path = std::fs::canonicalize(&clean_path).unwrap_or_else(|_| clean_path.clone().into());
            println!("Output directory set to: {}", absolute_path.display());
        }
    }

    Ok(())
}