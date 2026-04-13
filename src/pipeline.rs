use crate::{audio, image};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

pub fn convert_audio_to_image(
    input: &str,
    output: &str,
    width: u32,
    height: u32,
) -> Result<()> {
    println!("tiffiny::convert");
    println!("  input:  {}", input);
    println!("  output: {}", output);
    println!("  size:   {}x{}", width, height);

    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::with_template("[{bar:40}] {pos}/{len} {msg}")
            .unwrap()
    );

    pb.set_message("reading audio file");
    let raw_audio = audio::AudioConverter::mp3_to_raw(input)?;
    pb.inc(1);

    pb.set_message("creating image from audio data");
    let pixels = image::ImageGenerator::bytes_to_pixels(&raw_audio, width, height);
    pb.inc(1);

    pb.set_message("writing TIFF file");
    image::ImageGenerator::save_as_tiff(pixels, width, height, output)?;
    pb.inc(1);

    pb.finish_with_message("done");
    println!("Image written to {}", output);

    Ok(())
}
