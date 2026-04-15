use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    MP3,
    WAV,
    RAW,
    PNG,
    JPEG,
}

impl Format {
    pub fn from_path(path: &str) -> Option<Self> {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "mp3" => Some(Format::MP3),
                "wav" => Some(Format::WAV),
                "raw" => Some(Format::RAW),
                "png" => Some(Format::PNG),
                "jpg" | "jpeg" => Some(Format::JPEG),
                _ => None,
            })
    }
}