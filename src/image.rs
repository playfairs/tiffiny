use anyhow::{Result, Context};
use image::{RgbImage, ImageFormat};
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};

pub struct ImageGenerator;

impl ImageGenerator {
    pub fn audio_to_image(input: &str, output: &str, width: u32, height: u32) -> Result<()> {
        let pb = ProgressBar::new(3);
        pb.set_style(
            ProgressStyle::with_template("[{bar:40}] {pos}/{len} {msg}")
                .unwrap()
        );

        pb.set_message("reading audio file");
        let expanded_input = shellexpand::tilde(input).into_owned();
        let audio_data = fs::read(&expanded_input)
            .with_context(|| format!("Failed to read audio file: {}", input))?;
        pb.inc(1);

        pb.set_message("creating image from audio data");
        let pixels = Self::bytes_to_pixels(&audio_data, width, height);
        pb.inc(1);

        pb.set_message("writing TIFF file");
        Self::save_as_tiff(pixels, width, height, output)?;
        pb.inc(1);

        pb.finish_with_message("done");
        Ok(())
    }

    pub fn bytes_to_pixels(audio_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let pixel_count = (width * height * 3) as usize;
        let mut pixels = vec![0u8; pixel_count];
        for i in 0..pixel_count {
            pixels[i] = audio_data[i % audio_data.len()];
        }
        pixels
    }

    pub fn save_as_tiff(pixels: Vec<u8>, width: u32, height: u32, output: &str) -> Result<()> {
        let expanded_output = shellexpand::tilde(output).into_owned();
        let img = RgbImage::from_raw(width, height, pixels)
            .context("Failed to construct image buffer")?;
        img.save_with_format(&expanded_output, ImageFormat::Tiff)
            .with_context(|| format!("Failed to save TIFF file: {}", output))
    }
}