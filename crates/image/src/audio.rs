use std::fs;

pub struct AudioConverter;

impl AudioConverter {
    pub fn mp3_to_raw(input: &str) -> Result<Vec<u8>, ()> {
        Ok(fs::read(&input)
            .expect(&format!("Failed to read audio file: {}", input)))
    }
    
    pub fn get_audio_duration(input: &str) -> Result<f64, ()> {
        let data = fs::read(&input)
            .expect(&format!("Failed to read audio file: {}", input));
        
        Ok(data.len() as f64 / 44100.0 / 2.0)
    }
}