use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoStream {
    pub id: String,
    pub name: String,
    pub stream_type: StreamType,
    pub format: Arc<RwLock<StreamFormat>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<StreamEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Input,
    Output,
    Bidirectional,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    StreamStarted,
    StreamStopped,
    FrameReceived(usize),
    FrameSent(usize),
    Error(String),
    ParameterChanged(String, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamFormat {
    Rgba8,
    Rgb8,
    Yuv420,
    Nv12,
    Nv21,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub bitrate: Option<u32>,
    pub codec: Option<String>,
    pub buffer_size: usize,
    pub latency: std::time::Duration,
    pub auto_reconnect: bool,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct StreamStats {
    pub frames_received: usize,
    pub frames_sent: usize,
    pub frames_dropped: usize,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub current_bitrate: f32,
    pub average_bitrate: f32,
    pub uptime: std::time::Duration,
    pub last_frame_time: Option<std::time::Instant>,
}

impl VideoStream {
    pub fn new(id: String, name: String, stream_type: StreamType, format: StreamFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            format: Arc::new(RwLock::new(format)),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn start(&self, config: StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(StreamEvent::StreamStarted);
        
        match self.stream_type {
            StreamType::Input => self.start_input_stream(config).await,
            StreamType::Output => self.start_output_stream(config).await,
            StreamType::Bidirectional => self.start_bidirectional_stream(config).await,
            StreamType::Custom(_) => self.start_custom_stream(config).await,
        }
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(StreamEvent::StreamStopped);
        
Simulate stream stopping
        Ok(())
    }

    pub async fn send_frame(&self, frame: &crate::video_buffer::VideoFrame) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(StreamEvent::FrameSent(frame.frame_number as usize));
        
        Ok(())
    }

    pub async fn receive_frame(&self) -> Result<Option<crate::video_buffer::VideoFrame>, Box<dyn std::error::Error>> {
        let frame = self.create_sample_frame(1920, 1080, crate::video_buffer::PixelFormat::Rgba8, 0);
        let _ = self.event_sender.send(StreamEvent::FrameReceived(0));
        
        Ok(Some(frame))
    }

    async fn start_input_stream(&self, config: StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }

    async fn start_output_stream(&self, config: StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }

    async fn start_bidirectional_stream(&self, config: StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        Ok(())
    }

    async fn start_custom_stream(&self, config: StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
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

    pub fn set_format(&self, format: StreamFormat) {
        let mut current_format = self.format.write();
        *current_format = format;
    }

    pub fn get_format(&self) -> StreamFormat {
        self.format.read().clone()
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(StreamEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<StreamEvent> {
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

    pub fn get_stats(&self) -> StreamStats {
        StreamStats {
            frames_received: 0,
            frames_sent: 0,
            frames_dropped: 0,
            bytes_received: 0,
            bytes_sent: 0,
            current_bitrate: 0.0,
            average_bitrate: 0.0,
            uptime: std::time::Duration::from_secs(0),
            last_frame_time: None,
        }
    }

    pub fn get_supported_formats(&self) -> Vec<StreamFormat> {
        vec![
            StreamFormat::Rgba8,
            StreamFormat::Rgb8,
            StreamFormat::Yuv420,
            StreamFormat::Nv12,
            StreamFormat::Nv21,
        ]
    }

    pub fn can_handle_format(&self, format: &StreamFormat) -> bool {
        self.get_supported_formats().contains(format)
    }

    pub fn clone_stream(&self) -> VideoStream {
        let mut new_stream = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.stream_type.clone(),
            self.get_format(),
        );

        let parameters = self.parameters.read();
        *new_stream.parameters = parameters.clone();

        new_stream
    }
}

impl Default for VideoStream {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Stream".to_string(),
            StreamType::Input,
            StreamFormat::Rgba8,
        )
    }
}

impl Default for StreamType {
    fn default() -> Self {
        StreamType::Input
    }
}

impl Default for StreamFormat {
    fn default() -> Self {
        StreamFormat::Rgba8
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
            bitrate: None,
            codec: None,
            buffer_size: 1024 * 1024,
            latency: std::time::Duration::from_millis(100),
            auto_reconnect: true,
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        Self {
            frames_received: 0,
            frames_sent: 0,
            frames_dropped: 0,
            bytes_received: 0,
            bytes_sent: 0,
            current_bitrate: 0.0,
            average_bitrate: 0.0,
            uptime: std::time::Duration::from_secs(0),
            last_frame_time: None,
        }
    }
}
