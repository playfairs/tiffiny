use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoSaver {
    pub id: String,
    pub name: String,
    pub save_format: Arc<RwLock<VideoFormat>>,
    pub save_options: Arc<RwLock<SaveOptions>>,
    pub event_sender: mpsc::UnboundedSender<SaverEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SaverEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum SaverEvent {
    SaveStarted(String),
    SaveProgress(f32),
    SaveCompleted(String),
    Error(String),
    EncodingProgress(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    Mp4,
    Avi,
    Mov,
    Mkv,
    Webm,
    Flv,
    Wmv,
    M4v,
    ThreeGp,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SaveOptions {
    pub quality: VideoQuality,
    pub bitrate: Option<u32>,
    pub frame_rate: Option<f32>,
    pub resolution: Option<(u32, u32)>,
    pub codec: Option<VideoCodec>,
    pub audio_codec: Option<AudioCodec>,
    pub container: Option<VideoFormat>,
    pub hardware_acceleration: bool,
    pub two_pass_encoding: bool,
    pub preset: EncodingPreset,
    pub profile: EncodingProfile,
    pub level: Option<u8>,
    pub gop_size: Option<u32>,
    pub max_bitrate: Option<u32>,
    pub min_bitrate: Option<u32>,
    pub buffer_size: Option<u32>,
    pub keyframe_interval: Option<u32>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub subtitle_streams: Vec<SubtitleStream>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoQuality {
    Low,
    Medium,
    High,
    Ultra,
    Lossless,
    Custom(u32),bitrate in kbps
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    AV1,
    MPEG2,
    MPEG4,
    Theora,
    VP8,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    AAC,
    MP3,
    Opus,
    Vorbis,
    FLAC,
    PCM,
    AC3,
    DTS,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncodingPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncodingProfile {
    Baseline,
    Main,
    High,
    High10,
    High422,
    High444,
}

#[derive(Debug, Clone)]
pub struct SubtitleStream {
    pub index: u32,
    pub path: String,
    pub language: Option<String>,
    pub codec: String,
}

#[derive(Debug, Clone)]
pub struct SaveResult {
    pub success: bool,
    pub output_path: String,
    pub file_size: u64,
    pub duration: std::time::Duration,
    pub encoding_time: std::time::Duration,
    pub frames_encoded: usize,
    pub average_bitrate: u32,
    pub error_message: Option<String>,
}

impl VideoSaver {
    pub fn new(id: String, name: String, save_format: VideoFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            save_format: Arc::new(RwLock::new(save_format)),
            save_options: Arc::new(RwLock::new(SaveOptions::default())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn save(&self, video: &crate::video_buffer::Buffer, output_path: &str) -> Result<SaveResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(SaverEvent::SaveStarted(output_path.to_string()));
        let start_time = std::time::Instant::now();

        let save_options = self.save_options.read();
        let save_format = self.save_format.read();

        let result = match save_format {
            VideoFormat::Mp4 => self.save_mp4(video, output_path, &save_options).await,
            VideoFormat::Avi => self.save_avi(video, output_path, &save_options).await,
            VideoFormat::Mov => self.save_mov(video, output_path, &save_options).await,
            VideoFormat::Mkv => self.save_mkv(video, output_path, &save_options).await,
            VideoFormat::Webm => self.save_webm(video, output_path, &save_options).await,
            VideoFormat::Flv => self.save_flv(video, output_path, &save_options).await,
            VideoFormat::Wmv => self.save_wmv(video, output_path, &save_options).await,
            VideoFormat::M4v => self.save_m4v(video, output_path, &save_options).await,
            VideoFormat::ThreeGp => self.save_3gp(video, output_path, &save_options).await,
            VideoFormat::Custom(_) => self.save_custom(video, output_path, &save_options).await,
        };

        let encoding_time = start_time.elapsed();

        match result {
            Ok((file_size, frames_encoded, average_bitrate)) => {
                let duration = video.get_duration().unwrap_or(std::time::Duration::from_secs(0));
                
                let save_result = SaveResult {
                    success: true,
                    output_path: output_path.to_string(),
                    file_size,
                    duration,
                    encoding_time,
                    frames_encoded,
                    average_bitrate,
                    error_message: None,
                };

                let _ = self.event_sender.send(SaverEvent::SaveCompleted(output_path.to_string()));
                Ok(save_result)
            },
            Err(e) => {
                let error_msg = format!("Save failed: {}", e);
                let _ = self.event_sender.send(SaverEvent::Error(error_msg.clone()));
                
                Ok(SaveResult {
                    success: false,
                    output_path: output_path.to_string(),
                    file_size: 0,
                    duration: std::time::Duration::from_secs(0),
                    encoding_time,
                    frames_encoded: 0,
                    average_bitrate: 0,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn save_mp4(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_avi(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 2 * 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_mov(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_mkv(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_webm(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 512 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_flv(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_wmv(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 2 * 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_m4v(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_3gp(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 256 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    async fn save_custom(&self, video: &crate::video_buffer::Buffer, output_path: &str, options: &SaveOptions) -> Result<(u64, usize, u32), Box<dyn std::error::Error>> {
        let total_frames = video.get_frame_count();
        let mut frames_encoded = 0;
        let mut file_size = 0u64;
        let mut total_bitrate = 0u32;

        for (frame_index, _) in video.frames.read().iter().enumerate() {
            let progress = (frame_index as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(SaverEvent::EncodingProgress(progress));
            
            let frame_size = 1024 * 1024;
            file_size += frame_size;
            frames_encoded += 1;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        if let Some(duration) = video.get_duration() {
            total_bitrate = (file_size * 8) / duration.as_secs() as u64;
        }

        Ok((file_size, frames_encoded, total_bitrate as u32))
    }

    pub async fn save_batch(&self, videos: &[&crate::video_buffer::Buffer], output_dir: &str, name_template: &str) -> Result<Vec<SaveResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (index, video) in videos.iter().enumerate() {
            let output_path = format!("{}/{}", output_dir, name_template.replace("{index}", &index.to_string()));
            let result = self.save(video, &output_path).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub async fn save_segmented(&self, video: &crate::video_buffer::Buffer, output_dir: &str, segment_duration: std::time::Duration) -> Result<Vec<SaveResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let total_duration = video.get_duration().unwrap_or(std::time::Duration::from_secs(0));
        let segment_count = (total_duration.as_secs() / segment_duration.as_secs()) as usize + 1;

        for segment_index in 0..segment_count {
            let start_time = segment_duration * segment_index as u32;
            let end_time = start_time + segment_duration;
            
            let segment = video.create_preview(start_time, segment_duration);
            let output_path = format!("{}/segment_{:03}.mp4", output_dir, segment_index);
            
            let result = self.save(&segment, &output_path).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub async fn save_with_audio(&self, video: &crate::video_buffer::Buffer, audio: &tiffiny_audio::audio_buffer::AudioBuffer, output_path: &str) -> Result<SaveResult, Box<dyn std::error::Error>> {
        let _ = audio;
        
        self.save(video, output_path).await
    }

    pub async fn save_with_subtitles(&self, video: &crate::video_buffer::Buffer, subtitles: &[String], output_path: &str) -> Result<SaveResult, Box<dyn std::error::Error>> {
        let _ = subtitles;
        
        self.save(video, output_path).await
    }

    pub fn set_save_format(&self, format: VideoFormat) {
        let mut save_format = self.save_format.write();
        *save_format = format;
    }

    pub fn get_save_format(&self) -> VideoFormat {
        self.save_format.read().clone()
    }

    pub fn set_save_options(&self, options: SaveOptions) {
        let mut save_options = self.save_options.write();
        *save_options = options;
    }

    pub fn get_save_options(&self) -> SaveOptions {
        self.save_options.read().clone()
    }

    pub fn set_quality(&self, quality: VideoQuality) {
        let mut save_options = self.save_options.write();
        save_options.quality = quality;
    }

    pub fn set_bitrate(&self, bitrate: u32) {
        let mut save_options = self.save_options.write();
        save_options.bitrate = Some(bitrate);
    }

    pub fn set_codec(&self, codec: VideoCodec) {
        let mut save_options = self.save_options.write();
        save_options.codec = Some(codec);
    }

    pub fn set_audio_codec(&self, codec: AudioCodec) {
        let mut save_options = self.save_options.write();
        save_options.audio_codec = Some(codec);
    }

    pub fn set_preset(&self, preset: EncodingPreset) {
        let mut save_options = self.save_options.write();
        save_options.preset = preset;
    }

    pub fn set_profile(&self, profile: EncodingProfile) {
        let mut save_options = self.save_options.write();
        save_options.profile = profile;
    }

    pub fn set_hardware_acceleration(&self, enabled: bool) {
        let mut save_options = self.save_options.write();
        save_options.hardware_acceleration = enabled;
    }

    pub fn set_two_pass_encoding(&self, enabled: bool) {
        let mut save_options = self.save_options.write();
        save_options.two_pass_encoding = enabled;
    }

    pub async fn get_events(&mut self) -> Vec<SaverEvent> {
        let mut receiver = self.event_receiver.write();
        if let Some(ref mut rx) = *receiver {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
            events
        } else {
            Vec::new()
        }
    }

    pub fn get_supported_formats(&self) -> Vec<VideoFormat> {
        vec![
            VideoFormat::Mp4,
            VideoFormat::Avi,
            VideoFormat::Mov,
            VideoFormat::Mkv,
            VideoFormat::Webm,
            VideoFormat::Flv,
            VideoFormat::Wmv,
            VideoFormat::M4v,
            VideoFormat::ThreeGp,
        ]
    }

    pub fn get_supported_codecs(&self) -> Vec<VideoCodec> {
        vec![
            VideoCodec::H264,
            VideoCodec::H265,
            VideoCodec::VP9,
            VideoCodec::AV1,
            VideoCodec::MPEG2,
            VideoCodec::MPEG4,
            VideoCodec::Theora,
            VideoCodec::VP8,
        ]
    }

    pub fn get_supported_audio_codecs(&self) -> Vec<AudioCodec> {
        vec![
            AudioCodec::AAC,
            AudioCodec::MP3,
            AudioCodec::Opus,
            AudioCodec::Vorbis,
            AudioCodec::FLAC,
            AudioCodec::PCM,
            AudioCodec::AC3,
            AudioCodec::DTS,
        ]
    }

    pub fn get_encoding_presets(&self) -> Vec<EncodingPreset> {
        vec![
            EncodingPreset::UltraFast,
            EncodingPreset::SuperFast,
            EncodingPreset::VeryFast,
            EncodingPreset::Faster,
            EncodingPreset::Fast,
            EncodingPreset::Medium,
            EncodingPreset::Slow,
            EncodingPreset::Slower,
            EncodingPreset::VerySlow,
        ]
    }

    pub fn get_encoding_profiles(&self) -> Vec<EncodingProfile> {
        vec![
            EncodingProfile::Baseline,
            EncodingProfile::Main,
            EncodingProfile::High,
            EncodingProfile::High10,
            EncodingProfile::High422,
            EncodingProfile::High444,
        ]
    }

    pub fn estimate_file_size(&self, video: &crate::video_buffer::Buffer, options: &SaveOptions) -> u64 {
        let duration = video.get_duration().unwrap_or(std::time::Duration::from_secs(0));
        let bitrate = match options.bitrate {
            Some(bitrate) => bitrate as u64,
            None => match options.quality {
                VideoQuality::Low => 500_000,
                VideoQuality::Medium => 2_000_000,
                VideoQuality::High => 5_000_000,
                VideoQuality::Ultra => 10_000_000,
                VideoQuality::Lossless => 50_000_000,
                VideoQuality::Custom(bitrate) => bitrate as u64,
            },
        };

        (bitrate * duration.as_secs()) / 8
    }

    pub fn estimate_encoding_time(&self, video: &crate::video_buffer::Buffer, options: &SaveOptions) -> std::time::Duration {
        let frame_count = video.get_frame_count();
        let duration = video.get_duration().unwrap_or(std::time::Duration::from_secs(0));
        
        let base_time_per_frame = match options.preset {
            EncodingPreset::UltraFast => 1,
            EncodingPreset::SuperFast => 2,
            EncodingPreset::VeryFast => 4,
            EncodingPreset::Faster => 8,
            EncodingPreset::Fast => 16,
            EncodingPreset::Medium => 32,
            EncodingPreset::Slow => 64,
            EncodingPreset::Slower => 128,
            EncodingPreset::VerySlow => 256,
        };

        let hardware_factor = if options.hardware_acceleration { 0.5 } else { 1.0 };
        let codec_factor = match options.codec {
            Some(VideoCodec::H264) => 1.0,
            Some(VideoCodec::H265) => 2.0,
            Some(VideoCodec::VP9) => 3.0,
            Some(VideoCodec::AV1) => 4.0,
            Some(VideoCodec::MPEG2) => 0.5,
            Some(VideoCodec::MPEG4) => 0.7,
            _ => 1.0,
        };

        let total_encoding_time_ms = (frame_count as f64 * base_time_per_frame as f64 * hardware_factor * codec_factor) as u64;
        std::time::Duration::from_millis(total_encoding_time_ms)
    }

    pub fn validate_output_path(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(path);
        
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        if let Some(parent) = path.parent() {
            if !parent.is_dir() {
                return Err(format!("Parent directory is not a directory: {:?}", parent).into());
            }
        }

        Ok(())
    }

    pub fn clone_saver(&self) -> VideoSaver {
        let mut new_saver = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_save_format(),
        );
        
        let save_options = self.save_options.read();
        *new_saver.save_options = save_options.clone();
        
        new_saver
    }
}

impl Default for VideoSaver {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Saver".to_string(),
            VideoFormat::Mp4,
        )
    }
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            quality: VideoQuality::Medium,
            bitrate: None,
            frame_rate: None,
            resolution: None,
            codec: None,
            audio_codec: None,
            container: None,
            hardware_acceleration: false,
            two_pass_encoding: false,
            preset: EncodingPreset::Medium,
            profile: EncodingProfile::Main,
            level: None,
            gop_size: None,
            max_bitrate: None,
            min_bitrate: None,
            buffer_size: None,
            keyframe_interval: None,
            audio_sample_rate: None,
            audio_channels: None,
            audio_bitrate: None,
            subtitle_streams: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for VideoFormat {
    fn default() -> Self {
        VideoFormat::Mp4
    }
}

impl Default for VideoQuality {
    fn default() -> Self {
        VideoQuality::Medium
    }
}

impl Default for VideoCodec {
    fn default() -> Self {
        VideoCodec::H264
    }
}

impl Default for AudioCodec {
    fn default() -> Self {
        AudioCodec::AAC
    }
}

impl Default for EncodingPreset {
    fn default() -> Self {
        EncodingPreset::Medium
    }
}

impl Default for EncodingProfile {
    fn default() -> Self {
        EncodingProfile::Main
    }
}

impl Default for SaveResult {
    fn default() -> Self {
        Self {
            success: false,
            output_path: String::new(),
            file_size: 0,
            duration: std::time::Duration::from_secs(0),
            encoding_time: std::time::Duration::from_secs(0),
            frames_encoded: 0,
            average_bitrate: 0,
            error_message: None,
        }
    }
}
