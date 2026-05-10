use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct StreamConverter {
    pub id: String,
    pub name: String,
    pub input_stream: Arc<RwLock<Option<super::data_stream::DataStream>>>,
    pub output_stream: Arc<RwLock<Option<super::data_stream::DataStream>>>,
    pub status: Arc<RwLock<StreamStatus>>,
    pub event_sender: mpsc::UnboundedSender<StreamEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Idle,
    Connecting,
    Connected,
    Converting,
    Paused,
    Completed,
    Failed(String),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    StreamStarted,
    StreamProgress(f32),
    StreamCompleted(StreamResult),
    StreamFailed(String),
    DataReceived(Vec<u8>),
    DataSent(Vec<u8>),
    Error(String),
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct StreamResult {
    pub success: bool,
    pub bytes_processed: u64,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub chunk_size: usize,
    pub timeout: std::time::Duration,
    pub auto_reconnect: bool,
    pub max_retries: u32,
    pub retry_delay: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct StreamStats {
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub errors: u64,
    pub uptime: std::time::Duration,
    pub current_rate: f64,
}

impl StreamConverter {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            input_stream: Arc::new(RwLock::new(None))),
            output_stream: Arc::new(RwLock::new(None))),
            status: Arc::new(RwLock::new(StreamStatus::Idle))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn with_streams(id: String, name: String, input_stream: super::data_stream::DataStream, output_stream: super::data_stream::DataStream) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            input_stream: Arc::new(RwLock::new(Some(input_stream)))),
            output_stream: Arc::new(RwLock::new(Some(output_stream)))),
            status: Arc::new(RwLock::new(StreamStatus::Idle))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn start_conversion(&self, config: StreamConfig) -> Result<StreamResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(StreamEvent::StreamStarted);
        let start_time = std::time::Instant::now();

Set status to connecting
        let mut status = self.status.write();
        *status = StreamStatus::Connecting;

        self.connect_streams(&config).await?;

        *status = StreamStatus::Converting;

        let result = self.perform_conversion(&config).await;

        let processing_time = start_time.elapsed();

        match result {
            Ok(bytes_processed) => {
                *status = StreamStatus::Completed;

                let stream_result = StreamResult {
                    success: true,
                    bytes_processed,
                    processing_time,
                    error_message: None,
                };

                let _ = self.event_sender.send(StreamEvent::StreamCompleted(stream_result.clone()));
                Ok(stream_result)
            },
            Err(e) => {
                let error_msg = format!("Stream conversion failed: {}", e);
                
                *status = StreamStatus::Failed(error_msg.clone());
                
                let _ = self.event_sender.send(StreamEvent::StreamFailed(error_msg.clone()));
                
                Ok(StreamResult {
                    success: false,
                    bytes_processed: 0,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn connect_streams(&self, config: &StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let _ = self.event_sender.send(StreamEvent::Connected);
        Ok(())
    }

    async fn perform_conversion(&self, config: &StreamConfig) -> Result<u64, Box<dyn std::error::Error>> {
        let input_stream = self.input_stream.read();
        let output_stream = self.output_stream.read();

        if input_stream.is_none() || output_stream.is_none() {
            return Err("Input or output stream not available".into());
        }

        let input = input_stream.as_ref().unwrap().clone();
        let output = output_stream.as_ref().unwrap().clone();

        let mut total_bytes = 0u64;
        let mut processed_bytes = 0u64;

        loop {
            let chunk = input.read(config.chunk_size).await?;
            
            if chunk.is_empty() {
                break;
            }

            total_bytes += chunk.len() as u64;
            processed_bytes += chunk.len() as u64;

            output.write(&chunk).await?;

            let progress = if total_bytes > 0 {
                (processed_bytes as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            let _ = self.event_sender.send(StreamEvent::StreamProgress(progress));

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(processed_bytes)
    }

    pub async fn start_conversion_with_progress<F>(&self, config: StreamConfig, progress_callback: F) -> Result<StreamResult, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(StreamEvent::StreamStarted);
        let start_time = std::time::Instant::now();

        let mut status = self.status.write();
        *status = StreamStatus::Connecting;

        self.connect_streams(&config).await?;

        *status = StreamStatus::Converting;

        let result = self.perform_conversion_with_progress(&config, &progress_callback).await;

        let processing_time = start_time.elapsed();

        match result {
            Ok(bytes_processed) => {
                *status = StreamStatus::Completed;

                let stream_result = StreamResult {
                    success: true,
                    bytes_processed,
                    processing_time,
                    error_message: None,
                };

                let _ = self.event_sender.send(StreamEvent::StreamCompleted(stream_result.clone()));
                Ok(stream_result)
            },
            Err(e) => {
                let error_msg = format!("Stream conversion failed: {}", e);
                
                *status = StreamStatus::Failed(error_msg.clone());
                
                let _ = self.event_sender.send(StreamEvent::StreamFailed(error_msg.clone()));
                
                Ok(StreamResult {
                    success: false,
                    bytes_processed: 0,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn perform_conversion_with_progress<F>(&self, config: &StreamConfig, progress_callback: &F) -> Result<u64, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let input_stream = self.input_stream.read();
        let output_stream = self.output_stream.read();

        if input_stream.is_none() || output_stream.is_none() {
            return Err("Input or output stream not available".into());
        }

        let input = input_stream.as_ref().unwrap().clone();
        let output = output_stream.as_ref().unwrap().clone();

        let mut total_bytes = 0u64;
        let mut processed_bytes = 0u64;

        loop {
            let chunk = input.read(config.chunk_size).await?;
            
            if chunk.is_empty() {
                break;
            }

            total_bytes += chunk.len() as u64;
            processed_bytes += chunk.len() as u64;

            output.write(&chunk).await?;

            let progress = if total_bytes > 0 {
                (processed_bytes as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            progress_callback(progress);
            let _ = self.event_sender.send(StreamEvent::StreamProgress(progress));

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(processed_bytes)
    }

    pub fn set_input_stream(&self, stream: super::data_stream::DataStream) {
        let mut input_stream = self.input_stream.write();
        *input_stream = Some(stream);
    }

    pub fn set_output_stream(&self, stream: super::data_stream::DataStream) {
        let mut output_stream = self.output_stream.write();
        *output_stream = Some(stream);
    }

    pub fn get_input_stream(&self) -> Option<super::data_stream::DataStream> {
        self.input_stream.read().clone()
    }

    pub fn get_output_stream(&self) -> Option<super::data_stream::DataStream> {
        self.output_stream.read().clone()
    }

    pub fn get_status(&self) -> StreamStatus {
        self.status.read().clone()
    }

    pub fn pause(&self) {
        let mut status = self.status.write();
        *status = StreamStatus::Paused;
    }

    pub fn resume(&self) {
        let mut status = self.status.write();
        *status = StreamStatus::Converting;
    }

    pub fn stop(&self) {
        let mut status = self.status.write();
        *status = StreamStatus::Disconnected;
        
        let _ = self.event_sender.send(StreamEvent::Disconnected);
    }

    pub fn reset(&self) {
        let mut status = self.status.write();
        *status = StreamStatus::Idle;
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
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            errors: 0,
            uptime: std::time::Duration::from_secs(0),
            current_rate: 0.0,
        }
    }

    pub fn clone_converter(&self) -> StreamConverter {
        let mut new_converter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
        );

        if let Some(input_stream) = self.get_input_stream() {
            new_converter.set_input_stream(input_stream);
        }

        if let Some(output_stream) = self.get_output_stream() {
            new_converter.set_output_stream(output_stream);
        }

        new_converter
    }

    pub async fn convert_with_timeout(&self, config: StreamConfig, timeout: std::time::Duration) -> Result<StreamResult, Box<dyn std::error::Error>> {
        let conversion_task = tokio::spawn(self.start_conversion(config));
        
        match tokio::time::timeout(timeout, conversion_task).await {
            Ok(result) => result?,
            Err(_) => {
                let error_msg = "Stream conversion timed out".to_string();
                let _ = self.event_sender.send(StreamEvent::StreamFailed(error_msg.clone()));
                
                Ok(StreamResult {
                    success: false,
                    bytes_processed: 0,
                    processing_time: timeout,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    pub async fn convert_with_retry(&self, config: StreamConfig, max_retries: u32) -> Result<StreamResult, Box<dyn std::error::Error>> {
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            match self.start_conversion(config.clone()).await {
                Ok(result) => {
                    if result.success {
                        return Ok(result);
                    } else {
                        last_error = result.error_message;
                    }
                },
                Err(e) => {
                    last_error = Some(e.to_string());
                },
            }
            
            if attempt < max_retries {
                tokio::time::sleep(config.retry_delay).await;
            }
        }

        let error_msg = last_error.unwrap_or_else(|| "Stream conversion failed after retries".to_string());
        let _ = self.event_sender.send(StreamEvent::StreamFailed(error_msg.clone()));
        
        Ok(StreamResult {
            success: false,
            bytes_processed: 0,
            processing_time: std::time::Duration::from_secs(0),
            error_message: Some(error_msg),
        })
    }

    pub fn estimate_conversion_time(&self, config: &StreamConfig, estimated_bytes: u64) -> std::time::Duration {
        let chunks = (estimated_bytes + config.chunk_size as u64 - 1) / config.chunk_size as u64;
        let processing_time_per_chunk = std::time::Duration::from_millis(10);
        let total_time = processing_time_per_chunk * chunks as u32;
        
        total_time
    }

    pub fn estimate_throughput(&self, config: &StreamConfig) -> f64 {
        let chunk_size = config.chunk_size as f64;
        let processing_time_ms = 10.0;
        let throughput = chunk_size / (processing_time_ms / 1000.0);
        
        throughput
    }
}

impl Default for StreamConverter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Stream Converter".to_string(),
        )
    }
}

impl Default for StreamStatus {
    fn default() -> Self {
        StreamStatus::Idle
    }
}

impl Default for StreamEvent {
    fn default() -> Self {
        StreamEvent::StreamStarted
    }
}

impl Default for StreamResult {
    fn default() -> Self {
        Self {
            success: false,
            bytes_processed: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            chunk_size: 1024,
            timeout: std::time::Duration::from_secs(30),
            auto_reconnect: true,
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(1000),
        }
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            errors: 0,
            uptime: std::time::Duration::from_secs(0),
            current_rate: 0.0,
        }
    }
}
