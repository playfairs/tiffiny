use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioFormat {
    F32,
    F64,
    I16,
    I24,
    I32,
    U8,
    U16,
    U24,
    U32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioCodec {
    PCM,
    MP3,
    AAC,
    FLAC,
    OGG,
    WAV,
    AIFF,
    WMA,
    AC3,
    DTS,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioContainer {
    WAV,
    AIFF,
    FLAC,
    OGG,
    MP3,
    AAC,
    M4A,
    WMA,
    RAW,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFormatInfo {
    pub sample_format: AudioFormat,
    pub codec: AudioCodec,
    pub container: AudioContainer,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub bit_rate: Option<u32>,
    pub duration: Option<f64>,
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub copyright: Option<String>,
    pub encoder: Option<String>,
    pub tags: std::collections::HashMap<String, String>,
}

impl AudioFormat {
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            AudioFormat::F32 | AudioFormat::I32 | AudioFormat::U32 => 4,
            AudioFormat::F64 => 8,
            AudioFormat::I16 | AudioFormat::U16 => 2,
            AudioFormat::I24 | AudioFormat::U24 => 3,
            AudioFormat::U8 => 1,
        }
    }

    pub fn bits_per_sample(&self) -> u16 {
        match self {
            AudioFormat::F32 | AudioFormat::I32 | AudioFormat::U32 => 32,
            AudioFormat::F64 => 64,
            AudioFormat::I16 | AudioFormat::U16 => 16,
            AudioFormat::I24 | AudioFormat::U24 => 24,
            AudioFormat::U8 => 8,
        }
    }

    pub fn is_floating_point(&self) -> bool {
        matches!(self, AudioFormat::F32 | AudioFormat::F64)
    }

    pub fn is_signed(&self) -> bool {
        !matches!(self, AudioFormat::U8 | AudioFormat::U16 | AudioFormat::U24 | AudioFormat::U32)
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::F32 | AudioFormat::F64 | AudioFormat::I16 | AudioFormat::I24 | AudioFormat::I32)
    }

    pub fn get_dynamic_range(&self) -> f32 {
        match self {
            AudioFormat::F32 | AudioFormat::F64 => f32::INFINITY,
            AudioFormat::I16 => 65535.0,
            AudioFormat::I24 => 16777215.0,
            AudioFormat::I32 => 4294967295.0,
            AudioFormat::U8 => 255.0,
            AudioFormat::U16 => 65535.0,
            AudioFormat::U24 => 16777215.0,
            AudioFormat::U32 => 4294967295.0,
        }
    }

    pub fn get_min_value(&self) -> f32 {
        match self {
            AudioFormat::F32 => f32::MIN,
            AudioFormat::F64 => f32::MIN,
            AudioFormat::I16 => i16::MIN as f32,
            AudioFormat::I24 => -(1 << 23) as f32,
            AudioFormat::I32 => i32::MIN as f32,
            AudioFormat::U8 => 0.0,
            AudioFormat::U16 => 0.0,
            AudioFormat::U24 => 0.0,
            AudioFormat::U32 => 0.0,
        }
    }

    pub fn get_max_value(&self) -> f32 {
        match self {
            AudioFormat::F32 => f32::MAX,
            AudioFormat::F64 => f32::MAX,
            AudioFormat::I16 => i16::MAX as f32,
            AudioFormat::I24 => ((1 << 23) - 1) as f32,
            AudioFormat::I32 => i32::MAX as f32,
            AudioFormat::U8 => u8::MAX as f32,
            AudioFormat::U16 => u16::MAX as f32,
            AudioFormat::U24 => ((1 << 24) - 1) as f32,
            AudioFormat::U32 => u32::MAX as f32,
        }
    }

    pub fn convert_to_f32(&self, value: f32) -> f32 {
        match self {
            AudioFormat::F32 => value,
            AudioFormat::F64 => value as f32,
            AudioFormat::I16 => (value as i16) as f32,
            AudioFormat::I24 => (value as i32) as f32,
            AudioFormat::I32 => (value as i32) as f32,
            AudioFormat::U8 => (value as u8) as f32,
            AudioFormat::U16 => (value as u16) as f32,
            AudioFormat::U24 => (value as u32) as f32,
            AudioFormat::U32 => (value as u32) as f32,
        }
    }

    pub fn convert_from_f32(&self, value: f32) -> f32 {
        match self {
            AudioFormat::F32 => value,
            AudioFormat::F64 => value as f64 as f32,
            AudioFormat::I16 => (value.clamp(i16::MIN as f32, i16::MAX as f32) as i16) as f32,
            AudioFormat::I24 => (value.clamp(-(1 << 23) as f32, ((1 << 23) - 1) as f32) as i32) as f32,
            AudioFormat::I32 => (value.clamp(i32::MIN as f32, i32::MAX as f32) as i32) as f32,
            AudioFormat::U8 => (value.clamp(0.0, u8::MAX as f32) as u8) as f32,
            AudioFormat::U16 => (value.clamp(0.0, u16::MAX as f32) as u16) as f32,
            AudioFormat::U24 => (value.clamp(0.0, ((1 << 24) - 1) as f32) as u32) as f32,
            AudioFormat::U32 => (value.clamp(0.0, u32::MAX as f32) as u32) as f32,
        }
    }
}

impl AudioCodec {
    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioCodec::PCM | AudioCodec::FLAC | AudioCodec::WAV | AudioCodec::AIFF)
    }

    pub fn is_lossy(&self) -> bool {
        !self.is_lossless()
    }

    pub fn get_quality_levels(&self) -> Vec<&'static str> {
        match self {
            AudioCodec::MP3 => vec!["64kbps", "128kbps", "192kbps", "256kbps", "320kbps"],
            AudioCodec::AAC => vec!["64kbps", "96kbps", "128kbps", "160kbps", "192kbps", "256kbps"],
            AudioCodec::OGG => vec!["64kbps", "96kbps", "128kbps", "160kbps", "192kbps", "256kbps"],
            AudioCodec::FLAC => vec!["0", "1", "2", "3", "4", "5", "6", "7", "8"],
            _ => vec!["Lossless"],
        }
    }

    pub fn get_default_bitrate(&self) -> Option<u32> {
        match self {
            AudioCodec::MP3 => Some(128000),
            AudioCodec::AAC => Some(128000),
            AudioCodec::OGG => Some(128000),
            _ => None,
        }
    }

    pub fn get_compression_ratio(&self) -> Option<f32> {
        match self {
            AudioCodec::MP3 => Some(10.0),
            AudioCodec::AAC => Some(8.0),
            AudioCodec::OGG => Some(6.0),
            AudioCodec::FLAC => Some(2.0),
            _ => None,
        }
    }
}

impl AudioContainer {
    pub fn supports_codec(&self, codec: &AudioCodec) -> bool {
        match (self, codec) {
            (AudioContainer::WAV, AudioCodec::PCM) => true,
            (AudioContainer::AIFF, AudioCodec::PCM) => true,
            (AudioContainer::FLAC, AudioCodec::FLAC) => true,
            (AudioContainer::OGG, AudioCodec::OGG) => true,
            (AudioContainer::MP3, AudioCodec::MP3) => true,
            (AudioContainer::AAC | AudioContainer::M4A, AudioCodec::AAC) => true,
            (AudioContainer::WMA, AudioCodec::WMA) => true,
            _ => false,
        }
    }

    pub fn get_supported_codecs(&self) -> Vec<AudioCodec> {
        match self {
            AudioContainer::WAV | AudioContainer::AIFF => vec![AudioCodec::PCM],
            AudioContainer::FLAC => vec![AudioCodec::FLAC],
            AudioContainer::OGG => vec![AudioCodec::OGG],
            AudioContainer::MP3 => vec![AudioCodec::MP3],
            AudioContainer::AAC | AudioContainer::M4A => vec![AudioCodec::AAC],
            AudioContainer::WMA => vec![AudioCodec::WMA],
            AudioContainer::RAW => vec![AudioCodec::PCM],
        }
    }

    pub fn get_file_extensions(&self) -> Vec<&'static str> {
        match self {
            AudioContainer::WAV => vec!["wav"],
            AudioContainer::AIFF => vec!["aiff", "aif"],
            AudioContainer::FLAC => vec!["flac"],
            AudioContainer::OGG => vec!["ogg"],
            AudioContainer::MP3 => vec!["mp3"],
            AudioContainer::AAC => vec!["aac"],
            AudioContainer::M4A => vec!["m4a"],
            AudioContainer::WMA => vec!["wma"],
            AudioContainer::RAW => vec!["raw", "pcm"],
        }
    }

    pub fn get_mime_type(&self) -> &'static str {
        match self {
            AudioContainer::WAV => "audio/wav",
            AudioContainer::AIFF => "audio/aiff",
            AudioContainer::FLAC => "audio/flac",
            AudioContainer::OGG => "audio/ogg",
            AudioContainer::MP3 => "audio/mpeg",
            AudioContainer::AAC => "audio/aac",
            AudioContainer::M4A => "audio/mp4",
            AudioContainer::WMA => "audio/x-ms-wma",
            AudioContainer::RAW => "audio/raw",
        }
    }
}

impl AudioFormatInfo {
    pub fn new() -> Self {
        Self {
            sample_format: AudioFormat::F32,
            codec: AudioCodec::PCM,
            container: AudioContainer::WAV,
            sample_rate: 44100,
            channels: 2,
            bit_depth: 32,
            bit_rate: None,
            duration: None,
            metadata: AudioMetadata::new(),
        }
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }

    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.sample_format = format.clone();
        self.bit_depth = format.bits_per_sample();
        self
    }

    pub fn with_codec(mut self, codec: AudioCodec) -> Self {
        self.codec = codec;
        self
    }

    pub fn with_container(mut self, container: AudioContainer) -> Self {
        self.container = container;
        self
    }

    pub fn with_bit_rate(mut self, bit_rate: u32) -> Self {
        self.bit_rate = Some(bit_rate);
        self
    }

    pub fn with_duration(mut self, duration: f64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_metadata(mut self, metadata: AudioMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn get_bytes_per_second(&self) -> Option<u32> {
        if let Some(bit_rate) = self.bit_rate {
            Some(bit_rate / 8)
        } else {
            Some(self.sample_rate * self.channels as u32 * self.sample_format.bytes_per_sample() as u32)
        }
    }

    pub fn get_file_size(&self) -> Option<u64> {
        if let (Some(bit_rate), Some(duration)) = (self.bit_rate, self.duration) {
            Some((bit_rate as f64 * duration / 8.0) as u64)
        } else if let Some(duration) = self.duration {
            Some((self.sample_rate as f64 * self.channels as f64 * self.sample_format.bytes_per_sample() as f64 * duration) as u64)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.container.supports_codec(&self.codec) &&
        self.sample_rate > 0 &&
        self.channels > 0 &&
        self.bit_depth > 0
    }

    pub fn get_quality_score(&self) -> f32 {
        let mut score = 0.0;
        
        if self.sample_rate >= 96000 {
            score += 4.0;
        } else if self.sample_rate >= 48000 {
            score += 3.0;
        } else if self.sample_rate >= 44100 {
            score += 2.0;
        } else {
            score += 1.0;
        }
        
        if self.bit_depth >= 24 {
            score += 3.0;
        } else if self.bit_depth >= 16 {
            score += 2.0;
        } else {
            score += 1.0;
        }
        
        if self.codec.is_lossless() {
            score += 3.0;
        } else if let Some(bit_rate) = self.bit_rate {
            if bit_rate >= 256000 {
                score += 2.0;
            } else if bit_rate >= 128000 {
                score += 1.0;
            }
        }
        
        if self.channels >= 6 {
            score += 2.0;
        } else if self.channels >= 2 {
            score += 1.0;
        }
        
        score
    }

    pub fn clone_with_changes(&self) -> AudioFormatInfo {
        Self {
            sample_format: self.sample_format.clone(),
            codec: self.codec.clone(),
            container: self.container.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            bit_depth: self.bit_depth,
            bit_rate: self.bit_rate,
            duration: self.duration,
            metadata: self.metadata.clone(),
        }
    }
}

impl AudioMetadata {
    pub fn new() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            genre: None,
            year: None,
            track_number: None,
            album_artist: None,
            composer: None,
            comment: None,
            copyright: None,
            encoder: None,
            tags: std::collections::HashMap::new(),
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_artist(mut self, artist: String) -> Self {
        self.artist = Some(artist);
        self
    }

    pub fn with_album(mut self, album: String) -> Self {
        self.album = Some(album);
        self
    }

    pub fn with_genre(mut self, genre: String) -> Self {
        self.genre = Some(genre);
        self
    }

    pub fn with_year(mut self, year: u32) -> Self {
        self.year = Some(year);
        self
    }

    pub fn with_track_number(mut self, track_number: u32) -> Self {
        self.track_number = Some(track_number);
        self
    }

    pub fn add_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    pub fn get_tag(&self, key: &str) -> Option<&String> {
        self.tags.get(key)
    }

    pub fn has_metadata(&self) -> bool {
        self.title.is_some() ||
        self.artist.is_some() ||
        self.album.is_some() ||
        self.genre.is_some() ||
        self.year.is_some() ||
        self.track_number.is_some() ||
        !self.tags.is_empty()
    }

    pub fn get_display_string(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref title) = self.title {
            parts.push(title.clone());
        }
        
        if let Some(ref artist) = self.artist {
            parts.push(format!("by {}", artist));
        }
        
        if let Some(ref album) = self.album {
            parts.push(format!("from {}", album));
        }
        
        if let Some(year) = self.year {
            parts.push(format!("({})", year));
        }
        
        parts.join(" ")
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::F32
    }
}

impl Default for AudioCodec {
    fn default() -> Self {
        AudioCodec::PCM
    }
}

impl Default for AudioContainer {
    fn default() -> Self {
        AudioContainer::WAV
    }
}

impl Default for AudioFormatInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AudioMetadata {
    fn default() -> Self {
        Self::new()
    }
}
