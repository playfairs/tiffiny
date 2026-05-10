use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoEncoder {
    pub id: String,
    pub name: String,
    pub encoder_type: EncoderType,
    pub codec: Arc<RwLock<VideoCodec>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<EncoderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<EncoderEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncoderType {
    Software,
    Hardware,
    Hybrid,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum EncoderEvent {
    EncodingStarted,
    EncodingProgress(f32),
    FrameEncoded(usize),
    EncodingCompleted,
    Error(String),
    ParameterChanged(String, f32),
}

#[derive(Debug, Clone)]
pub struct EncodingSession {
    pub id: String,
    pub input_buffer: Arc<crate::video_buffer::Buffer>,
    pub output_path: String,
    pub encoder_config: EncoderConfig,
    pub start_time: std::time::Instant,
    pub frames_encoded: usize,
    pub total_frames: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub codec: VideoCodec,
    pub preset: EncodingPreset,
    pub profile: EncodingProfile,
    pub level: Option<u8>,
    pub bitrate: Option<u32>,
    pub max_bitrate: Option<u32>,
    pub min_bitrate: Option<u32>,
    pub buffer_size: Option<u32>,
    pub gop_size: Option<u32>,
    pub keyframe_interval: Option<u32>,
    pub b_frames: Option<u32>,
    pub reference_frames: Option<u32>,
    pub threads: Option<u32>,
    pub hardware_acceleration: bool,
    pub two_pass: bool,
    pub crf: Option<u8>,Constant Rate Factor
    pub tune: Option<String>,
    pub x264_params: std::collections::HashMap<String, String>,
    pub x265_params: std::collections::HashMap<String, String>,
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
    Placebo,
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

impl VideoEncoder {
    pub fn new(id: String, name: String, encoder_type: EncoderType, codec: VideoCodec) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            encoder_type,
            codec: Arc::new(RwLock::new(codec)),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn encode(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(EncoderEvent::EncodingStarted);
        
        let session = EncodingSession {
            id: uuid::Uuid::new_v4().to_string(),
            input_buffer: Arc::new(input.clone()),
            output_path: output_path.to_string(),
            encoder_config: config,
            start_time: std::time::Instant::now(),
            frames_encoded: 0,
            total_frames: input.get_frame_count(),
            is_active: true,
        };

        self.encode_session(&session).await
    }

    async fn encode_session(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        match self.encoder_type {
            EncoderType::Software => self.encode_software(session).await,
            EncoderType::Hardware => self.encode_hardware(session).await,
            EncoderType::Hybrid => self.encode_hybrid(session).await,
            EncoderType::Custom(_) => self.encode_custom(session).await,
        }
    }

    async fn encode_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let codec = self.codec.read();
        
        match codec {
            VideoCodec::H264 => self.encode_h264_software(session).await,
            VideoCodec::H265 => self.encode_h265_software(session).await,
            VideoCodec::VP9 => self.encode_vp9_software(session).await,
            VideoCodec::AV1 => self.encode_av1_software(session).await,
            VideoCodec::MPEG2 => self.encode_mpeg2_software(session).await,
            VideoCodec::MPEG4 => self.encode_mpeg4_software(session).await,
            VideoCodec::Theora => self.encode_theora_software(session).await,
            VideoCodec::VP8 => self.encode_vp8_software(session).await,
            VideoCodec::Custom(_) => self.encode_custom_software(session).await,
        }
    }

    async fn encode_h264_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config);
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_h265_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 2.0;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_vp9_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 1.5;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_av1_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 3.0;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_mpeg2_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 0.5;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_mpeg4_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 0.8;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_theora_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 1.2;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_vp8_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 1.0;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_custom_software(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config);
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_hardware(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 0.3;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_hybrid(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config) * 0.6;
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    async fn encode_custom(&self, session: &EncodingSession) -> Result<(), Box<dyn std::error::Error>> {
        let frames = session.input_buffer.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &session.encoder_config);
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    fn simulate_frame_encoding_time(&self, frame: &crate::video_buffer::VideoFrame, config: &EncoderConfig) -> std::time::Duration {
        let base_time = std::time::Duration::from_millis(10);
        
        let size_factor = (frame.width * frame.height) as f64 / (1920.0 * 1080.0);
        
        let preset_factor = match config.preset {
            EncodingPreset::UltraFast => 0.1,
            EncodingPreset::SuperFast => 0.2,
            EncodingPreset::VeryFast => 0.3,
            EncodingPreset::Faster => 0.4,
            EncodingPreset::Fast => 0.5,
            EncodingPreset::Medium => 1.0,
            EncodingPreset::Slow => 2.0,
            EncodingPreset::Slower => 3.0,
            EncodingPreset::VerySlow => 5.0,
            EncodingPreset::Placebo => 10.0,
        };
        
        let quality_factor = if let Some(bitrate) = config.bitrate {
            (bitrate as f64 / 5_000_000.0).max(0.1)
        } else {
            1.0
        };
        
        let total_factor = size_factor * preset_factor * quality_factor;
        let adjusted_time = std::time::Duration::from_millis((base_time.as_millis() as f64 * total_factor) as u64);
        
        adjusted_time
    }

    pub async fn encode_with_progress<F>(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: EncoderConfig, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(EncoderEvent::EncodingStarted);
        
        let frames = input.frames.read();
        
        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &config);
            tokio::time::sleep(encoding_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 100.0;
            progress_callback(progress);
            
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    pub async fn encode_two_pass(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(EncoderEvent::EncodingStarted);
        
        let first_pass_config = EncoderConfig {
            two_pass: false,
            ..config.clone()
        };

        let frames = input.frames.read();
        for (frame_index, frame) in frames.iter().enumerate() {
            let analysis_time = self.simulate_frame_encoding_time(frame, &first_pass_config) * 0.5;
            tokio::time::sleep(analysis_time).await;
            
            let progress = (frame_index as f32 / frames.len() as f32) * 50.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
        }

        let second_pass_config = EncoderConfig {
            two_pass: false,
            ..config
        };

        for (frame_index, frame) in frames.iter().enumerate() {
            let encoding_time = self.simulate_frame_encoding_time(frame, &second_pass_config);
            tokio::time::sleep(encoding_time).await;
            
            let progress = 50.0 + (frame_index as f32 / frames.len() as f32) * 50.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
            let _ = self.event_sender.send(EncoderEvent::FrameEncoded(frame_index));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    pub async fn encode_stream(&self, input_stream: &mut dyn std::io::Read, output_path: &str, config: EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(EncoderEvent::EncodingStarted);
        
        for i in 0..100 {
            let encoding_time = std::time::Duration::from_millis(10);
            tokio::time::sleep(encoding_time).await;
            
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(EncoderEvent::EncodingProgress(progress));
        }

        let _ = self.event_sender.send(EncoderEvent::EncodingCompleted);
        Ok(())
    }

    pub fn set_codec(&self, codec: VideoCodec) {
        let mut current_codec = self.codec.write();
        *current_codec = codec;
    }

    pub fn get_codec(&self) -> VideoCodec {
        self.codec.read().clone()
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(EncoderEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<EncoderEvent> {
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

    pub fn get_supported_presets(&self) -> Vec<EncodingPreset> {
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
            EncodingPreset::Placebo,
        ]
    }

    pub fn get_supported_profiles(&self) -> Vec<EncodingProfile> {
        vec![
            EncodingProfile::Baseline,
            EncodingProfile::Main,
            EncodingProfile::High,
            EncodingProfile::High10,
            EncodingProfile::High422,
            EncodingProfile::High444,
        ]
    }

    pub fn can_encode_format(&self, codec: &VideoCodec) -> bool {
        self.get_supported_codecs().contains(codec)
    }

    pub fn estimate_encoding_time(&self, input: &crate::video_buffer::Buffer, config: &EncoderConfig) -> std::time::Duration {
        let frames = input.get_frame_count();
        let total_time = if let Some(first_frame) = input.get_frame(0) {
            let frame_time = self.simulate_frame_encoding_time(first_frame, config);
            frame_time * frames as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        total_time
    }

    pub fn estimate_file_size(&self, input: &crate::video_buffer::Buffer, config: &EncoderConfig) -> u64 {
        let duration = input.get_duration().unwrap_or(std::time::Duration::from_secs(0));
        let bitrate = config.bitrate.unwrap_or(5_000_000);
        
        (bitrate as u64 * duration.as_secs()) / 8
    }

    pub fn validate_config(&self, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        match (&config.codec, &config.profile) {
            (VideoCodec::H264, EncodingProfile::High10) => return Err("H.264 High10 profile not supported".into()),
            (VideoCodec::H264, EncodingProfile::High422) => return Err("H.264 High422 profile not supported".into()),
            (VideoCodec::H264, EncodingProfile::High444) => return Err("H.264 High444 profile not supported".into()),
            (VideoCodec::MPEG2, EncodingProfile::High10) => return Err("MPEG2 High10 profile not supported".into()),
            (VideoCodec::MPEG2, EncodingProfile::High422) => return Err("MPEG2 High422 profile not supported".into()),
            (VideoCodec::MPEG2, EncodingProfile::High444) => return Err("MPEG2 High444 profile not supported".into()),
            _ => (),
        }

        if let Some(bitrate) = config.bitrate {
            if bitrate < 100_000 || bitrate > 100_000_000 {
                return Err("Bitrate must be between 100kbps and 100Mbps".into());
            }
        }

        if let Some(level) = config.level {
            match config.codec {
                VideoCodec::H264 => {
                    if level > 51 {
                        return Err("H.264 level must be <= 5.1".into());
                    }
                },
                VideoCodec::H265 => {
                    if level > 62 {
                        return Err("H.265 level must be <= 6.2".into());
                    }
                },
                _ => (),
            }
        }

        Ok(())
    }

    pub fn clone_encoder(&self) -> VideoEncoder {
        let mut new_encoder = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.encoder_type.clone(),
            self.get_codec(),
        );
        
        let parameters = self.parameters.read();
        *new_encoder.parameters = parameters.clone();
        
        new_encoder
    }
}

impl Default for VideoEncoder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Encoder".to_string(),
            EncoderType::Software,
            VideoCodec::H264,
        )
    }
}

impl Default for EncoderType {
    fn default() -> Self {
        EncoderType::Software
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            preset: EncodingPreset::Medium,
            profile: EncodingProfile::Main,
            level: None,
            bitrate: None,
            max_bitrate: None,
            min_bitrate: None,
            buffer_size: None,
            gop_size: None,
            keyframe_interval: None,
            b_frames: None,
            reference_frames: None,
            threads: None,
            hardware_acceleration: false,
            two_pass: false,
            crf: None,
            tune: None,
            x264_params: std::collections::HashMap::new(),
            x265_params: std::collections::HashMap::new(),
        }
    }
}

impl Default for VideoCodec {
    fn default() -> Self {
        VideoCodec::H264
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
