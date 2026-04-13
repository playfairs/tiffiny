use anyhow::{Result, Context};
use std::fs;

pub struct AudioConverter;

impl AudioConverter {
    pub fn mp3_to_raw(input: &str) -> Result<Vec<u8>> {
        let expanded_path = shellexpand::tilde(input).into_owned();
        fs::read(&expanded_path)
            .with_context(|| format!("Failed to read audio file: {}", input))
    }
    
    pub fn get_audio_duration(input: &str) -> Result<f64> {
        let expanded_path = shellexpand::tilde(input).into_owned();
        let data = fs::read(&expanded_path)
            .with_context(|| format!("Failed to read audio file: {}", input))?;
        
        Ok(data.len() as f64 / 44100.0 / 2.0)
    }
}