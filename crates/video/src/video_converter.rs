use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoConverter {
    pub id: String,
    pub name: String,
    pub converter_type: ConverterType,
    pub input_format: Arc<RwLock<VideoFormat>>,
    pub output_format: Arc<RwLock<VideoFormat>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<ConverterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ConverterEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConverterType {
    Format,
    Resolution,
    FrameRate,
    Codec,
    Bitrate,
    Quality,
    ColorSpace,
    AspectRatio,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ConverterEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted,
    Error(String),
    ParameterChanged(String, f32),
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub success: bool,
    pub output_video: Option<Arc<crate::video_buffer::Buffer>>,
    pub output_path: Option<String>,
    pub processing_time: std::time::Duration,
    pub frames_processed: usize,
    pub file_size: Option<u64>,
    pub error_message: Option<String>,
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
pub struct ConversionConfig {
    pub input_format: VideoFormat,
    pub output_format: VideoFormat,
    pub input_resolution: Option<(u32, u32)>,
    pub output_resolution: Option<(u32, u32)>,
    pub input_frame_rate: Option<f32>,
    pub output_frame_rate: Option<f32>,
    pub input_bitrate: Option<u32>,
    pub output_bitrate: Option<u32>,
    pub input_codec: Option<String>,
    pub output_codec: Option<String>,
    pub quality: Option<ConversionQuality>,
    pub color_space: Option<String>,
    pub aspect_ratio: Option<String>,
    pub preserve_metadata: bool,
    pub fast_start: bool,
    pub two_pass: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionQuality {
    Low,
    Medium,
    High,
    Ultra,
    Lossless,
    Custom(u32),bitrate in kbps
}

impl VideoConverter {
    pub fn new(id: String, name: String, converter_type: ConverterType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            converter_type,
            input_format: Arc::new(RwLock::new(VideoFormat::Mp4)),
            output_format: Arc::new(RwLock::new(VideoFormat::Mp4)),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn convert(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: ConversionConfig) -> Result<ConversionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        let start_time = std::time::Instant::now();

        let result = match self.converter_type {
            ConverterType::Format => self.convert_format(input, output_path, &config),
            ConverterType::Resolution => self.convert_resolution(input, output_path, &config),
            ConverterType::FrameRate => self.convert_frame_rate(input, output_path, &config),
            ConverterType::Codec => self.convert_codec(input, output_path, &config),
            ConverterType::Bitrate => self.convert_bitrate(input, output_path, &config),
            ConverterType::Quality => self.convert_quality(input, output_path, &config),
            ConverterType::ColorSpace => self.convert_color_space(input, output_path, &config),
            ConverterType::AspectRatio => self.convert_aspect_ratio(input, output_path, &config),
            ConverterType::Custom(_) => self.convert_custom(input, output_path, &config),
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_video) => {
                let _ = self.event_sender.send(ConverterEvent::ConversionCompleted);
                
                let file_size = self.estimate_file_size(&output_video, &config);
                
                Ok(ConversionResult {
                    success: true,
                    output_video: Some(Arc::new(output_video)),
                    output_path: Some(output_path.to_string()),
                    processing_time,
                    frames_processed: input.get_frame_count(),
                    file_size: Some(file_size),
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Conversion failed: {}", e);
                let _ = self.event_sender.send(ConverterEvent::Error(error_msg.clone()));
                
                Ok(ConversionResult {
                    success: false,
                    output_video: None,
                    output_path: Some(output_path.to_string()),
                    processing_time,
                    frames_processed: 0,
                    file_size: None,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    fn convert_format(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output)
    }

    fn convert_resolution(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        if let Some(output_resolution) = config.output_resolution {
            let mut output = crate::video_buffer::Buffer::new(
                output_resolution.0,
                output_resolution.1,
                input.pixel_format,
                input.frame_rate,
            );

            for frame_index in 0..input.get_frame_count() {
                if let Some(input_frame) = input.get_frame(frame_index) {
                    let output_frame = self.resize_frame(input_frame, output_resolution.0, output_resolution.1);
                    output.add_frame(output_frame);
                }

                let progress = (frame_index as f32 / input.get_frame_count() as f32) * 100.0;
                let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            }

            Ok(output)
        } else {
            Err("Output resolution not specified".into())
        }
    }

    fn convert_frame_rate(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        if let Some(output_frame_rate) = config.output_frame_rate {
            let input_frame_rate = input.frame_rate;
            let frame_rate_ratio = output_frame_rate / input_frame_rate;
            
            let mut output = crate::video_buffer::Buffer::new(
                input.width,
                input.height,
                input.pixel_format,
                output_frame_rate,
            );

            let input_frames = input.get_frame_count();
            let output_frames = (input_frames as f32 * frame_rate_ratio).round() as usize;

            for frame_index in 0..output_frames {
                let input_frame_index = (frame_index as f32 / frame_rate_ratio).round() as usize;
                
                if let Some(input_frame) = input.get_frame(input_frame_index) {
                    output.add_frame(input_frame);
                }

                let progress = (frame_index as f32 / output_frames as f32) * 100.0;
                let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            }

            Ok(output)
        } else {
            Err("Output frame rate not specified".into())
        }
    }

    fn convert_codec(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn convert_bitrate(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn convert_quality(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn convert_color_space(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn convert_aspect_ratio(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn convert_custom(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: &ConversionConfig) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for i in 0..=100 {
            let progress = i as f32;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output)
    }

    fn resize_frame(&self, frame: &crate::video_buffer::VideoFrame, new_width: u32, new_height: u32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = crate::video_buffer::VideoFrame::new(new_width, new_height, frame.pixel_format);

        let x_ratio = frame.width as f32 / new_width as f32;
        let y_ratio = frame.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * x_ratio).round() as u32;
                let src_y = (y as f32 * y_ratio).round() as u32;
                
                if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        output_frame
    }

    fn estimate_file_size(&self, video: &crate::video_buffer::Buffer, config: &ConversionConfig) -> u64 {
        let duration = video.get_duration().unwrap_or(std::time::Duration::from_secs(0));
        let bitrate = config.output_bitrate.unwrap_or(5_000_000);
        
        (bitrate as u64 * duration.as_secs()) / 8
    }

    pub async fn convert_with_progress<F>(&self, input: &crate::video_buffer::Buffer, output_path: &str, config: ConversionConfig, progress_callback: F) -> Result<ConversionResult, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        let start_time = std::time::Instant::now();

        let result = match self.converter_type {
            ConverterType::Format => self.convert_format(input, output_path, &config),
            ConverterType::Resolution => self.convert_resolution(input, output_path, &config),
            ConverterType::FrameRate => self.convert_frame_rate(input, output_path, &config),
            ConverterType::Codec => self.convert_codec(input, output_path, &config),
            ConverterType::Bitrate => self.convert_bitrate(input, output_path, &config),
            ConverterType::Quality => self.convert_quality(input, output_path, &config),
            ConverterType::ColorSpace => self.convert_color_space(input, output_path, &config),
            ConverterType::AspectRatio => self.convert_aspect_ratio(input, output_path, &config),
            ConverterType::Custom(_) => self.convert_custom(input, output_path, &config),
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_video) => {
                let _ = self.event_sender.send(ConverterEvent::ConversionCompleted);
                
                let file_size = self.estimate_file_size(&output_video, &config);
                
                Ok(ConversionResult {
                    success: true,
                    output_video: Some(Arc::new(output_video)),
                    output_path: Some(output_path.to_string()),
                    processing_time,
                    frames_processed: input.get_frame_count(),
                    file_size: Some(file_size),
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Conversion failed: {}", e);
                let _ = self.event_sender.send(ConverterEvent::Error(error_msg.clone()));
                
                Ok(ConversionResult {
                    success: false,
                    output_video: None,
                    output_path: Some(output_path.to_string()),
                    processing_time,
                    frames_processed: 0,
                    file_size: None,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    pub async fn convert_batch(&self, inputs: &[&crate::video_buffer::Buffer], output_dir: &str, config: ConversionConfig) -> Result<Vec<ConversionResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (index, input) in inputs.iter().enumerate() {
            let output_path = format!("{}/output_{}.mp4", output_dir, index);
            let result = self.convert(input, &output_path, config).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn set_input_format(&self, format: VideoFormat) {
        let mut input_format = self.input_format.write();
        *input_format = format;
    }

    pub fn set_output_format(&self, format: VideoFormat) {
        let mut output_format = self.output_format.write();
        *output_format = format;
    }

    pub fn get_input_format(&self) -> VideoFormat {
        self.input_format.read().clone()
    }

    pub fn get_output_format(&self) -> VideoFormat {
        self.output_format.read().clone()
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(ConverterEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<ConverterEvent> {
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

    pub fn get_supported_qualities(&self) -> Vec<ConversionQuality> {
        vec![
            ConversionQuality::Low,
            ConversionQuality::Medium,
            ConversionQuality::High,
            ConversionQuality::Ultra,
            ConversionQuality::Lossless,
        ]
    }

    pub fn can_convert_format(&self, from: &VideoFormat, to: &VideoFormat) -> bool {
        let supported_formats = self.get_supported_formats();
        supported_formats.contains(from) && supported_formats.contains(to)
    }

    pub fn estimate_conversion_time(&self, input: &crate::video_buffer::Buffer, config: &ConversionConfig) -> std::time::Duration {
        let frame_count = input.get_frame_count();
        let base_time_per_frame = std::time::Duration::from_millis(10);
        
        let time_multiplier = match self.converter_type {
            ConverterType::Format => 1.0,
            ConverterType::Resolution => 2.0,
            ConverterType::FrameRate => 1.5,
            ConverterType::Codec => 3.0,
            ConverterType::Bitrate => 1.0,
            ConverterType::Quality => 1.2,
            ConverterType::ColorSpace => 1.8,
            ConverterType::AspectRatio => 1.0,
            ConverterType::Custom(_) => 2.5,
        };

        let total_time = base_time_per_frame * frame_count as u32;
        std::time::Duration::from_millis((total_time.as_millis() as f64 * time_multiplier) as u64)
    }

    pub fn validate_config(&self, config: &ConversionConfig) -> Result<(), Box<dyn std::error::Error>> {
        if !self.can_convert_format(&config.input_format, &config.output_format) {
            return Err("Unsupported format conversion".into());
        }

        if let Some((width, height)) = config.output_resolution {
            if width == 0 || height == 0 {
                return Err("Invalid output resolution".into());
            }
        }

        if let Some(frame_rate) = config.output_frame_rate {
            if frame_rate <= 0.0 {
                return Err("Invalid output frame rate".into());
            }
        }

        if let Some(bitrate) = config.output_bitrate {
            if bitrate == 0 {
                return Err("Invalid output bitrate".into());
            }
        }

        Ok(())
    }

    pub fn clone_converter(&self) -> VideoConverter {
        let mut new_converter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.converter_type.clone(),
        );

        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        let parameters = self.parameters.read();

        *new_converter.input_format = input_format.clone();
        *new_converter.output_format = output_format.clone();
        *new_converter.parameters = parameters.clone();

        new_converter
    }
}

impl Default for VideoConverter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Converter".to_string(),
            ConverterType::Format,
        )
    }
}

impl Default for ConverterType {
    fn default() -> Self {
        ConverterType::Format
    }
}

impl Default for VideoFormat {
    fn default() -> Self {
        VideoFormat::Mp4
    }
}

impl Default for ConversionResult {
    fn default() -> Self {
        Self {
            success: false,
            output_video: None,
            output_path: None,
            processing_time: std::time::Duration::from_millis(0),
            frames_processed: 0,
            file_size: None,
            error_message: None,
        }
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            input_format: VideoFormat::Mp4,
            output_format: VideoFormat::Mp4,
            input_resolution: None,
            output_resolution: None,
            input_frame_rate: None,
            output_frame_rate: None,
            input_bitrate: None,
            output_bitrate: None,
            input_codec: None,
            output_codec: None,
            quality: Some(ConversionQuality::Medium),
            color_space: None,
            aspect_ratio: None,
            preserve_metadata: true,
            fast_start: false,
            two_pass: false,
        }
    }
}

impl Default for ConversionQuality {
    fn default() -> Self {
        ConversionQuality::Medium
    }
}
