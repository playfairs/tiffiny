use image::{RgbImage, ImageFormat};
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};

pub struct ImageGenerator;

impl ImageGenerator {
    pub fn audio_to_image(input: &str, output: &str, width: u32, height: u32, encoding: &str, channels: u32, color_scheme: &str, scaling: &str) -> Result<(), ()> {
        let pb = ProgressBar::new(3);
        pb.set_style(
            ProgressStyle::with_template("[{bar:40}] {pos}/{len} {msg}")
                .unwrap()
        );

        pb.set_message("reading audio file");
        let audio_data = fs::read(&input)
            .expect(&format!("Failed to read audio file: {}", input));
        pb.inc(1);

        pb.set_message("creating image from audio data");
        let pixels = Self::bytes_to_pixels(&audio_data, width, height, encoding, channels, color_scheme, scaling);
        pb.inc(1);

        pb.set_message("writing PNG file");
        Self::save_as_png(pixels, width, height, output)?;
        pb.inc(1);

        pb.finish_with_message("done");
        Ok(())
    }

    pub fn bytes_to_pixels(audio_data: &[u8], width: u32, height: u32, encoding: &str, channels: u32, color_scheme: &str, scaling: &str) -> Vec<u8> {
        let pixel_count = (width * height * 3) as usize;
        let mut pixels = vec![0u8; pixel_count];
        
        match encoding {
            "raw" => Self::raw_encoding(&audio_data, &mut pixels, width, height, channels, color_scheme, scaling),
            "waveform" => Self::waveform_encoding(&audio_data, &mut pixels, width, height, channels, color_scheme, scaling),
            "spectrogram" => Self::spectrogram_encoding(&audio_data, &mut pixels, width, height, channels, color_scheme, scaling),
            _ => Self::raw_encoding(&audio_data, &mut pixels, width, height, channels, color_scheme, scaling),
        }
        
        pixels
    }

    fn raw_encoding(audio_data: &[u8], pixels: &mut [u8], width: u32, height: u32, _channels: u32, _color_scheme: &str, _scaling: &str) {
        let pixel_count = (width * height * 3) as usize;
        for i in 0..pixel_count {
            pixels[i] = audio_data[i % audio_data.len()];
        }
    }

    fn waveform_encoding(audio_data: &[u8], pixels: &mut [u8], width: u32, height: u32, _channels: u32, color_scheme: &str, _scaling: &str) {
        let pixel_count = (width * height * 3) as usize;
        let samples_per_pixel = (audio_data.len() / pixel_count).max(1);
        
        for (i, pixel_chunk) in pixels.chunks_exact_mut(3).enumerate() {
            let sample_idx = (i * samples_per_pixel) % audio_data.len();
            let sample_value = audio_data[sample_idx];
            
            match color_scheme {
                "grayscale" => {
                    pixel_chunk[0] = sample_value;
                    pixel_chunk[1] = sample_value;
                    pixel_chunk[2] = sample_value;
                },
                "heat" => {
                    let intensity = sample_value as f32 / 255.0;
                    pixel_chunk[0] = (intensity * 255.0) as u8;
                    pixel_chunk[1] = (intensity * 128.0) as u8;
                    pixel_chunk[2] = 0;
                },
                "rainbow" => {
                    let hue = (sample_value as f32 / 255.0) * 360.0;
                    let rgb = Self::hsv_to_rgb(hue, 1.0, 1.0);
                    pixel_chunk[0] = rgb.0;
                    pixel_chunk[1] = rgb.1;
                    pixel_chunk[2] = rgb.2;
                },
                _ => {
                    pixel_chunk[0] = sample_value;
                    pixel_chunk[1] = sample_value.wrapping_add(85);
                    pixel_chunk[2] = sample_value.wrapping_add(170);
                }
            }
        }
    }

    fn spectrogram_encoding(audio_data: &[u8], pixels: &mut [u8], width: u32, height: u32, _channels: u32, color_scheme: &str, scaling: &str) {
        let pixel_count = (width * height * 3) as usize;
        let freq_resolution = width as usize / 8;
        let time_resolution = height;
        
        for y in 0..time_resolution {
            for x in 0..width {
                let freq_band = (x / (width / freq_resolution as u32)) as usize;
                let time_idx = (y as usize * audio_data.len() / time_resolution as usize) % audio_data.len();
                
                let sample_value = audio_data[time_idx];
                let freq_intensity = if freq_band < audio_data.len() {
                    audio_data[(time_idx + freq_band) % audio_data.len()]
                } else {
                    sample_value
                };
                
                let pixel_idx = ((y as usize * width as usize + x as usize) * 3) as usize;
                if pixel_idx + 2 < pixel_count {
                    match color_scheme {
                        "grayscale" => {
                            let intensity = Self::apply_scaling(freq_intensity as f32, scaling);
                            pixels[pixel_idx] = intensity;
                            pixels[pixel_idx + 1] = intensity;
                            pixels[pixel_idx + 2] = intensity;
                        },
                        "heat" => {
                            let intensity = Self::apply_scaling(freq_intensity as f32, scaling) as f32 / 255.0;
                            pixels[pixel_idx] = (intensity * 255.0) as u8;
                            pixels[pixel_idx + 1] = (intensity * 128.0) as u8;
                            pixels[pixel_idx + 2] = 0;
                        },
                        "rainbow" => {
                            let hue = (Self::apply_scaling(freq_intensity as f32, scaling) as f32 / 255.0) * 360.0;
                            let rgb = Self::hsv_to_rgb(hue, 1.0, 1.0);
                            pixels[pixel_idx] = rgb.0;
                            pixels[pixel_idx + 1] = rgb.1;
                            pixels[pixel_idx + 2] = rgb.2;
                        },
                        _ => {
                            let intensity = Self::apply_scaling(freq_intensity as f32, scaling);
                            pixels[pixel_idx] = intensity;
                            pixels[pixel_idx + 1] = intensity.wrapping_add(100);
                            pixels[pixel_idx + 2] = intensity.wrapping_add(200);
                        }
                    }
                }
            }
        }
    }

    fn apply_scaling(value: f32, scaling: &str) -> u8 {
        match scaling {
            "log" => ((value.ln() + 1.0) * 127.0 / 6.0).max(0.0).min(255.0) as u8,
            "sqrt" => (value.sqrt() * 16.0).max(0.0).min(255.0) as u8,
            _ => value as u8,
        }
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0 % 2.0) - 1.0).abs());
        let m = v - c;
        
        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        (
            ((r_prime + m) * 255.0) as u8,
            ((g_prime + m) * 255.0) as u8,
            ((b_prime + m) * 255.0) as u8,
        )
    }

    pub fn save_as_png(pixels: Vec<u8>, width: u32, height: u32, output: &str) -> Result<(), ()> {
        let img = RgbImage::from_raw(width, height, pixels)
            .expect("Failed to construct image buffer");
        img.save_with_format(&output, ImageFormat::Png)
            .expect(&format!("Failed to save PNG file: {}", output));
        Ok(())
    }
}