use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub id: String,
    pub conversion_type: Arc<RwLock<ConversionType>>,
    pub success: Arc<RwLock<bool>>,
    pub input_source: Arc<RwLock<InputSource>>,
    pub output_destination: Arc<RwLock<OutputDestination>>,
    pub metadata: Arc<RwLock<ConversionMetadata>>,
    pub performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    pub error_info: Arc<RwLock<Option<ErrorInfo>>>>,
    pub event_sender: mpsc::UnboundedSender<ResultEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ResultEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionType {
    Audio,
    Image,
    Video,
    Document,
    Archive,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum InputSource {
    FilePath(String),
    DataStream(String),
    Buffer(Vec<u8>),
    Url(String),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum OutputDestination {
    FilePath(String),
    DataStream(String),
    Buffer,
    Url(String),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ResultEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted,
    ConversionFailed(String),
    InputProcessed(u64),
    OutputWritten(u64),
    ErrorOccurred(String),
    Warning(String),
}

#[derive(Debug, Clone)]
pub struct ConversionMetadata {
    pub input_format: Option<String>,
    pub output_format: Option<String>,
    pub input_size: u64,
    pub output_size: u64,
    pub compression_ratio: f32,
    pub processing_time: std::time::Duration,
    pub start_time: std::time::Instant,
    pub end_time: std::time::Instant,
    pub quality_settings: std::collections::HashMap<String, String>,
    pub custom_properties: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub disk_io_read: u64,
    pub disk_io_write: u64,
    pub network_io: u64,
    pub throughput: f64,
    pub processing_rate: f64,
    pub efficiency: f32,
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: ErrorType,
    pub error_code: Option<String>,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub context: std::collections::HashMap<String, String>,
    pub recoverable: bool,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    InputError,
    OutputError,
    ProcessingError,
    NetworkError,
    TimeoutError,
    MemoryError,
    DiskError,
    PermissionError,
    FormatError,
    ConfigurationError,
    UnknownError,
}

#[derive(Debug, Clone)]
pub struct ConversionSummary {
    pub total_conversions: u64,
    pub successful_conversions: u64,
    pub failed_conversions: u64,
    pub total_input_bytes: u64,
    pub total_output_bytes: u64,
    pub average_processing_time: std::time::Duration,
    pub average_throughput: f64,
    pub error_rate: f32,
}

impl ConversionResult {
    pub fn new(id: String, conversion_type: ConversionType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            conversion_type: Arc::new(RwLock::new(conversion_type))),
            success: Arc::new(RwLock::new(false))),
            input_source: Arc::new(RwLock::new(InputSource::Buffer(Vec::new())))),
            output_destination: Arc::new(RwLock::new(OutputDestination::Buffer))),
            metadata: Arc::new(RwLock::new(ConversionMetadata::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            error_info: Arc::new(RwLock::new(None))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_file(id: String, conversion_type: ConversionType, input_path: &str, output_path: &str) -> Self {
        let mut result = Self::new(id, conversion_type);
        
        let mut input_source = result.input_source.write();
        *input_source = InputSource::FilePath(input_path.to_string());
        
        let mut output_destination = result.output_destination.write();
        *output_destination = OutputDestination::FilePath(output_path.to_string());
        
        result
    }

    pub fn from_stream(id: String, conversion_type: ConversionType, input_stream: &str, output_stream: &str) -> Self {
        let mut result = Self::new(id, conversion_type);
        
        let mut input_source = result.input_source.write();
        *input_source = InputSource::DataStream(input_stream.to_string());
        
        let mut output_destination = result.output_destination.write();
        *output_destination = OutputDestination::DataStream(output_stream.to_string());
        
        result
    }

    pub fn from_buffer(id: String, conversion_type: ConversionType, input_data: Vec<u8>, output_buffer: bool) -> Self {
        let mut result = Self::new(id, conversion_type);
        
        let mut input_source = result.input_source.write();
        *input_source = InputSource::Buffer(input_data);
        
        let mut output_destination = result.output_destination.write();
        *output_destination = if output_buffer {
            OutputDestination::Buffer
        } else {
            OutputDestination::Buffer(Vec::new())
        };
        
        result
    }

    pub fn from_url(id: String, conversion_type: ConversionType, input_url: &str, output_url: &str) -> Self {
        let mut result = Self::new(id, conversion_type);
        
        let mut input_source = result.input_source.write();
        *input_source = InputSource::Url(input_url.to_string());
        
        let mut output_destination = result.output_destination.write();
        *output_destination = OutputDestination::Url(output_url.to_string());
        
        result
    }

    pub async fn start_conversion(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ResultEvent::ConversionStarted);
        
Set start time
        let mut metadata = self.metadata.write();
        metadata.start_time = std::time::Instant::now();
        
        let result = match *self.conversion_type.read() {
            ConversionType::Audio => self.convert_audio().await,
            ConversionType::Image => self.convert_image().await,
            ConversionType::Video => self.convert_video().await,
            ConversionType::Document => self.convert_document().await,
            ConversionType::Archive => self.convert_archive().await,
            ConversionType::Custom(_) => self.convert_custom().await,
        };

        metadata.end_time = std::time::Instant::now();
        metadata.processing_time = metadata.end_time.duration_since(metadata.start_time);

        match result {
            Ok(output_size) => {
                metadata.output_size = output_size;
                metadata.compression_ratio = if metadata.input_size > 0 {
                    output_size as f32 / metadata.input_size as f32
                } else {
                    1.0
                };

                let mut metrics = self.performance_metrics.write();
                metrics.efficiency = 1.0;
                metrics.throughput = output_size as f64 / metadata.processing_time.as_secs_f64();

                let mut success = self.success.write();
                *success = true;

                let _ = self.event_sender.send(ResultEvent::ConversionCompleted);
                let _ = self.event_sender.send(ResultEvent::OutputWritten(output_size));
                
                Ok(())
            },
            Err(e) => {
                let mut error_info = self.error_info.write();
                *error_info = Some(ErrorInfo {
                    error_type: ErrorType::ProcessingError,
                    error_code: None,
                    error_message: e.to_string(),
                    stack_trace: None,
                    context: std::collections::HashMap::new(),
                    recoverable: false,
                    retry_count: 0,
                });

                let mut success = self.success.write();
                *success = false;

                let _ = self.event_sender.send(ResultEvent::ConversionFailed(e.to_string()));
                let _ = self.event_sender.send(ResultEvent::ErrorOccurred(e.to_string()));
                
                Ok(())
            },
        }
    }

    async fn convert_audio(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("mp3".to_string());
        metadata.output_format = Some("wav".to_string());
        metadata.input_size = input_size;

        let total_steps = 100;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(input_size)
    }

    async fn convert_image(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("jpg".to_string());
        metadata.output_format = Some("png".to_string());
        metadata.input_size = input_size;

        let total_steps = 100;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok((input_size as f64 * 1.2) as u64)
    }

    async fn convert_video(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("mp4".to_string());
        metadata.output_format = Some("webm".to_string());
        metadata.input_size = input_size;

        let total_steps = 200;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok((input_size as f64 * 0.8) as u64)
    }

    async fn convert_document(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("docx".to_string());
        metadata.output_format = Some("pdf".to_string());
        metadata.input_size = input_size;

        let total_steps = 150;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        Ok((input_size as f64 * 0.6) as u64)
    }

    async fn convert_archive(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("rar".to_string());
        metadata.output_format = Some("zip".to_string());
        metadata.input_size = input_size;

        let total_steps = 80;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok((input_size as f64 * 1.1) as u64)
    }

    async fn convert_custom(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let input_source = self.input_source.read();
        let input_size = self.get_input_size(&input_source);
        
        let mut metadata = self.metadata.write();
        metadata.input_format = Some("custom".to_string());
        metadata.output_format = Some("custom".to_string());
        metadata.input_size = input_size;

        let total_steps = 120;
        for step in 0..=total_steps {
            let progress = (step as f32 / total_steps as f32) * 100.0;
            let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
            tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        }

        Ok(input_size)
    }

    fn get_input_size(&self, input_source: &InputSource) -> u64 {
        match input_source {
            InputSource::FilePath(path) => {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            },
            InputSource::Buffer(data) => data.len() as u64,
            InputSource::DataStream(_) => 0,
            InputSource::Url(_) => 0,
            InputSource::Custom(_) => 0,
        }
    }

    pub fn set_input_source(&self, source: InputSource) {
        let mut input_source = self.input_source.write();
        *input_source = source;
    }

    pub fn set_output_destination(&self, destination: OutputDestination) {
        let mut output_destination = self.output_destination.write();
        *output_destination = destination;
    }

    pub fn get_conversion_type(&self) -> ConversionType {
        self.conversion_type.read().clone()
    }

    pub fn is_successful(&self) -> bool {
        *self.success.read()
    }

    pub fn get_metadata(&self) -> ConversionMetadata {
        self.metadata.read().clone()
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().clone()
    }

    pub fn get_error_info(&self) -> Option<ErrorInfo> {
        self.error_info.read().clone()
    }

    pub fn update_progress(&self, progress: f32) {
        let _ = self.event_sender.send(ResultEvent::ConversionProgress(progress));
    }

    pub fn report_warning(&self, warning: &str) {
        let _ = self.event_sender.send(ResultEvent::Warning(warning.to_string()));
    }

    pub async fn get_events(&mut self) -> Vec<ResultEvent> {
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

    pub fn reset(&self) {
        let mut success = self.success.write();
        *success = false;

        let mut metadata = self.metadata.write();
        *metadata = ConversionMetadata::default();

        let mut performance_metrics = self.performance_metrics.write();
        *performance_metrics = PerformanceMetrics::default();

        let mut error_info = self.error_info.write();
        *error_info = None;
    }

    pub fn clone_result(&self) -> ConversionResult {
        let mut new_result = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.get_conversion_type(),
        );

        let input_source = self.input_source.read();
        new_result.set_input_source(input_source.clone());

        let output_destination = self.output_destination.read();
        new_result.set_output_destination(output_destination.clone());

        let metadata = self.metadata.read();
        *new_result.metadata = Arc::new(RwLock::new(metadata.clone()));

        let performance_metrics = self.performance_metrics.read();
        *new_result.performance_metrics = Arc::new(RwLock::new(performance_metrics.clone()));

        let error_info = self.error_info.read();
        *new_result.error_info = Arc::new(RwLock::new(error_info.clone()));

        new_result
    }

    pub fn create_summary(&self, results: &[ConversionResult]) -> ConversionSummary {
        let total_conversions = results.len() as u64;
        let successful_conversions = results.iter().filter(|r| r.is_successful()).count() as u64;
        let failed_conversions = total_conversions - successful_conversions;

        let total_input_bytes: u64 = results.iter().map(|r| r.get_metadata().input_size).sum();
        let total_output_bytes: u64 = results.iter().map(|r| r.get_metadata().output_size).sum();

        let average_processing_time = if !results.is_empty() {
            let total_time: std::time::Duration = results.iter()
                .map(|r| r.get_metadata().processing_time)
                .sum();
            total_time / results.len() as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        let average_throughput = if average_processing_time.as_secs_f64() > 0.0 {
            total_output_bytes as f64 / average_processing_time.as_secs_f64()
        } else {
            0.0
        };

        let error_rate = if total_conversions > 0 {
            failed_conversions as f32 / total_conversions as f32
        } else {
            0.0
        };

        ConversionSummary {
            total_conversions,
            successful_conversions,
            failed_conversions,
            total_input_bytes,
            total_output_bytes,
            average_processing_time,
            average_throughput,
            error_rate,
        }
    }

    pub fn export_result(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let metadata = self.get_metadata();
        let performance_metrics = self.get_performance_metrics();
        let error_info = self.get_error_info();

        Ok(serde_json::json!({
            "id": self.id,
            "conversion_type": format!("{:?}", self.get_conversion_type()),
            "success": self.is_successful(),
            "input_source": format!("{:?}", self.input_source.read()),
            "output_destination": format!("{:?}", self.output_destination.read()),
            "metadata": {
                "input_format": metadata.input_format,
                "output_format": metadata.output_format,
                "input_size": metadata.input_size,
                "output_size": metadata.output_size,
                "compression_ratio": metadata.compression_ratio,
                "processing_time": format!("{:?}", metadata.processing_time),
                "start_time": format!("{:?}", metadata.start_time),
                "end_time": format!("{:?}", metadata.end_time),
                "quality_settings": metadata.quality_settings,
                "custom_properties": metadata.custom_properties,
            },
            "performance_metrics": {
                "cpu_usage": performance_metrics.cpu_usage,
                "memory_usage": performance_metrics.memory_usage,
                "disk_io_read": performance_metrics.disk_io_read,
                "disk_io_write": performance_metrics.disk_io_write,
                "network_io": performance_metrics.network_io,
                "throughput": performance_metrics.throughput,
                "processing_rate": performance_metrics.processing_rate,
                "efficiency": performance_metrics.efficiency,
            },
            "error_info": error_info.map(|e| serde_json::json!({
                "error_type": format!("{:?}", e.error_type),
                "error_code": e.error_code,
                "error_message": e.error_message,
                "stack_trace": e.stack_trace,
                "context": e.context,
                "recoverable": e.recoverable,
                "retry_count": e.retry_count,
            })),
        }))
    }

    pub fn import_result(&self, data: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let id = data["id"].as_str().unwrap_or("").to_string();
        let success = data["success"].as_bool().unwrap_or(false);
        
        let mut current_success = self.success.write();
        *current_success = success;
        
        Ok(())
    }
}

impl Default for ConversionResult {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            ConversionType::Image,
        )
    }
}

impl Default for ConversionType {
    fn default() -> Self {
        ConversionType::Image
    }
}

impl Default for InputSource {
    fn default() -> Self {
        InputSource::Buffer(Vec::new())
    }
}

impl Default for OutputDestination {
    fn default() -> Self {
        OutputDestination::Buffer
    }
}

impl Default for ResultEvent {
    fn default() -> Self {
        ResultEvent::ConversionStarted
    }
}

impl Default for ConversionMetadata {
    fn default() -> Self {
        Self {
            input_format: None,
            output_format: None,
            input_size: 0,
            output_size: 0,
            compression_ratio: 1.0,
            processing_time: std::time::Duration::from_secs(0),
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            quality_settings: std::collections::HashMap::new(),
            custom_properties: std::collections::HashMap::new(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            disk_io_read: 0,
            disk_io_write: 0,
            network_io: 0,
            throughput: 0.0,
            processing_rate: 0.0,
            efficiency: 0.0,
        }
    }
}

impl Default for ErrorInfo {
    fn default() -> Self {
        Self {
            error_type: ErrorType::UnknownError,
            error_code: None,
            error_message: String::new(),
            stack_trace: None,
            context: std::collections::HashMap::new(),
            recoverable: false,
            retry_count: 0,
        }
    }
}

impl Default for ConversionSummary {
    fn default() -> Self {
        Self {
            total_conversions: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            total_input_bytes: 0,
            total_output_bytes: 0,
            average_processing_time: std::time::Duration::from_secs(0),
            average_throughput: 0.0,
            error_rate: 0.0,
        }
    }
}
