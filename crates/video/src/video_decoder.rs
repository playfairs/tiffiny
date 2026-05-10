use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoDecoder {
    pub id: String,
    pub name: String,
    pub decoder_type: DecoderType,
    pub codec: Arc<RwLock<VideoCodec>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<DecoderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<DecoderEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecoderType {
    Software,
    Hardware,
    Hybrid,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum DecoderEvent {
    DecodingStarted,
    DecodingProgress(f32),
    FrameDecoded(usize),
    DecodingCompleted,
    Error(String),
    ParameterChanged(String, f32),
}

#[derive(Debug, Clone)]
pub struct DecodingSession {
    pub id: String,
    pub input_path: String,
    pub decoder_config: DecoderConfig,
    pub start_time: std::time::Instant,
    pub frames_decoded: usize,
    pub total_frames: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct DecoderConfig {
    pub codec: VideoCodec,
    pub hardware_acceleration: bool,
    pub threads: Option<u32>,
    pub skip_frames: Option<u32>,
    pub max_frames: Option<u32>,
    pub seek_time: Option<std::time::Duration>,
    pub frame_rate: Option<f32>,
    pub resolution: Option<(u32, u32)>,
    pub pixel_format: Option<crate::video_buffer::PixelFormat>,
    pub deinterlace: bool,
    pub color_correction: bool,
    pub noise_reduction: bool,
    pub sharpening: bool,
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

impl VideoDecoder {
    pub fn new(id: String, name: String, decoder_type: DecoderType, codec: VideoCodec) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            decoder_type,
            codec: Arc::new(RwLock::new(codec)),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn decode(&self, input_path: &str, config: DecoderConfig) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(DecoderEvent::DecodingStarted);
        
        let session = DecodingSession {
            id: uuid::Uuid::new_v4().to_string(),
            input_path: input_path.to_string(),
            decoder_config: config,
            start_time: std::time::Instant::now(),
            frames_decoded: 0,
            total_frames: 0,
            is_active: true,
        };

        self.decode_session(&session).await
    }

    async fn decode_session(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        match self.decoder_type {
            DecoderType::Software => self.decode_software(session).await,
            DecoderType::Hardware => self.decode_hardware(session).await,
            DecoderType::Hybrid => self.decode_hybrid(session).await,
            DecoderType::Custom(_) => self.decode_custom(session).await,
        }
    }

    async fn decode_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let codec = self.codec.read();
        
        match codec {
            VideoCodec::H264 => self.decode_h264_software(session).await,
            VideoCodec::H265 => self.decode_h265_software(session).await,
            VideoCodec::VP9 => self.decode_vp9_software(session).await,
            VideoCodec::AV1 => self.decode_av1_software(session).await,
            VideoCodec::MPEG2 => self.decode_mpeg2_software(session).await,
            VideoCodec::MPEG4 => self.decode_mpeg4_software(session).await,
            VideoCodec::Theora => self.decode_theora_software(session).await,
            VideoCodec::VP8 => self.decode_vp8_software(session).await,
            VideoCodec::Custom(_) => self.decode_custom_software(session).await,
        }
    }

    async fn decode_h264_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
Simulate H.264 software decoding
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_h265_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_vp9_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_av1_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_mpeg2_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(0)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_mpeg4_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(0)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_theora_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_vp8_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_custom_software(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_hardware(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(0)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_hybrid(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(0)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_custom(&self, session: &DecodingSession) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
        self.decode_software(session).await
    }

    fn create_sample_frame(&self, width: u32, height: u32, pixel_format: crate::video_buffer::PixelFormat, frame_number: u32) -> crate::video_buffer::VideoFrame {
        let mut frame = crate::video_buffer::VideoFrame::new(width, height, pixel_format);
        frame.frame_number = frame_number;

        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as f32;
                let g = ((y as f32 / height as f32) * 255.0) as f32;
                let b = ((frame_number % 255) as f32);
                
                let pixel = crate::video_buffer::Pixel::new(r, g, b, 255.0);
                frame.set_pixel(x, y, pixel);
            }
        }

        frame
    }

    async fn get_video_frame_count(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(300)
    }

    pub async fn decode_with_progress<F>(&self, input_path: &str, config: DecoderConfig, progress_callback: F) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(DecoderEvent::DecodingStarted);
        
        let session = DecodingSession {
            id: uuid::Uuid::new_v4().to_string(),
            input_path: input_path.to_string(),
            decoder_config: config,
            start_time: std::time::Instant::now(),
            frames_decoded: 0,
            total_frames: 0,
            is_active: true,
        };

        match self.decoder_type {
            DecoderType::Software => self.decode_software_with_progress(&session, progress_callback).await,
            DecoderType::Hardware => self.decode_hardware_with_progress(&session, progress_callback).await,
            DecoderType::Hybrid => self.decode_hybrid_with_progress(&session, progress_callback).await,
            DecoderType::Custom(_) => self.decode_custom_with_progress(&session, progress_callback).await,
        }
    }

    async fn decode_software_with_progress<F>(&self, session: &DecodingSession, progress_callback: F) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let (width, height) = session.decoder_config.resolution.unwrap_or((1920, 1080));
        let frame_rate = session.decoder_config.frame_rate.unwrap_or(30.0);
        let pixel_format = session.decoder_config.pixel_format.unwrap_or(crate::video_buffer::PixelFormat::Rgba8);
        
        let mut video_buffer = crate::video_buffer::Buffer::new(width, height, pixel_format, frame_rate);
        
        let total_frames = self.get_video_frame_count(&session.input_path).await?;
        let mut current_session = session.clone();
        current_session.total_frames = total_frames;
        
        for frame_index in 0..total_frames {
            let frame = self.create_sample_frame(width, height, pixel_format, frame_index as u32);
            video_buffer.add_frame(frame);
            
            current_session.frames_decoded += 1;
            
            let progress = (current_session.frames_decoded as f32 / total_frames as f32) * 100.0;
            progress_callback(progress);
            
            let _ = self.event_sender.send(DecoderEvent::DecodingProgress(progress));
            let _ = self.event_sender.send(DecoderEvent::FrameDecoded(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        let _ = self.event_sender.send(DecoderEvent::DecodingCompleted);
        Ok(Arc::new(video_buffer))
    }

    async fn decode_hardware_with_progress<F>(&self, session: &DecodingSession, progress_callback: F) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        self.decode_software_with_progress(session, progress_callback).await
    }

    async fn decode_hybrid_with_progress<F>(&self, session: &DecodingSession, progress_callback: F) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        self.decode_software_with_progress(session, progress_callback).await
    }

    async fn decode_custom_with_progress<F>(&self, session: &DecodingSession, progress_callback: F) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        self.decode_software_with_progress(session, progress_callback).await
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
        
        let _ = self.event_sender.send(DecoderEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<DecoderEvent> {
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

    pub fn can_decode_format(&self, codec: &VideoCodec) -> bool {
        self.get_supported_codecs().contains(codec)
    }

    pub fn get_video_info(&self, path: &str) -> Result<crate::video_loader::VideoInfo, Box<dyn std::error::Error>> {
        Ok(crate::video_loader::VideoInfo {
            path: path.to_string(),
            format: "mp4".to_string(),
            duration: std::time::Duration::from_secs(120),
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
            bitrate: 5_000_000,
            codec: "h264".to_string(),
            audio_streams: vec![],
            subtitle_streams: vec![],
            metadata: crate::video_buffer::VideoMetadata::default(),
        })
    }

    pub fn clone_decoder(&self) -> VideoDecoder {
        let mut new_decoder = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.decoder_type.clone(),
            self.get_codec(),
        );
        
        let parameters = self.parameters.read();
        *new_decoder.parameters = parameters.clone();
        
        new_decoder
    }
}

impl Default for VideoDecoder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Decoder".to_string(),
            DecoderType::Software,
            VideoCodec::H264,
        )
    }
}

impl Default for DecoderType {
    fn default() -> Self {
        DecoderType::Software
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            hardware_acceleration: false,
            threads: None,
            skip_frames: None,
            max_frames: None,
            seek_time: None,
            frame_rate: None,
            resolution: None,
            pixel_format: None,
            deinterlace: false,
            color_correction: false,
            noise_reduction: false,
            sharpening: false,
        }
    }
}

impl Default for VideoCodec {
    fn default() -> Self {
        VideoCodec::H264
    }
}

impl Default for DecodingSession {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_path: String::new(),
            decoder_config: DecoderConfig::default(),
            start_time: std::time::Instant::now(),
            frames_decoded: 0,
            total_frames: 0,
            is_active: false,
        }
    }
}
