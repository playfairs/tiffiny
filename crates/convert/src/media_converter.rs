use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct MediaConverter {
    pub id: String,
    pub name: String,
    pub converter_type: Arc<RwLock<ConverterType>>,
    pub event_sender: mpsc::UnboundedSender<ConverterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ConverterEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConverterType {
    Audio,
    Image,
    Video,
    Document,
    Archive,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ConverterEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted(ConversionResult),
    Error(String),
    FileProcessed(String),
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub success: bool,
    pub input_file: String,
    pub output_file: String,
    pub input_format: String,
    pub output_format: String,
    pub file_size_before: u64,
    pub file_size_after: u64,
    pub compression_ratio: f32,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub path: String,
    pub name: String,
    pub format: String,
    pub size: u64,
    pub created_time: Option<std::time::SystemTime>,
    pub modified_time: Option<std::time::SystemTime>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ConversionOptions {
    pub preserve_metadata: bool,
    pub preserve_quality: bool,
    pub optimize_size: bool,
    pub overwrite_existing: bool,
    pub create_backup: bool,
    pub output_directory: Option<String>,
    pub custom_parameters: std::collections::HashMap<String, String>,
}

impl MediaConverter {
    pub fn new(id: String, name: String, converter_type: ConverterType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            converter_type: Arc::new(RwLock::new(converter_type)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn convert_file(&self, input_path: &str, output_path: &str, options: ConversionOptions) -> Result<ConversionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        let start_time = std::time::Instant::now();

Get input file info
        let input_file = self.get_file_info(input_path).await?;
        let file_size_before = input_file.size;

        let result = match *self.converter_type.read() {
            ConverterType::Audio => self.convert_audio_file(input_path, output_path, &options).await,
            ConverterType::Image => self.convert_image_file(input_path, output_path, &options).await,
            ConverterType::Video => self.convert_video_file(input_path, output_path, &options).await,
            ConverterType::Document => self.convert_document_file(input_path, output_path, &options).await,
            ConverterType::Archive => self.convert_archive_file(input_path, output_path, &options).await,
            ConverterType::Custom(_) => self.convert_custom_file(input_path, output_path, &options).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_file) => {
                let output_info = self.get_file_info(&output_file).await?;
                let file_size_after = output_info.size;
                let compression_ratio = file_size_after as f32 / file_size_before as f32;

                let conversion_result = ConversionResult {
                    success: true,
                    input_file: input_path.to_string(),
                    output_file: output_file,
                    input_format: input_file.format.clone(),
                    output_format: output_info.format,
                    file_size_before,
                    file_size_after,
                    compression_ratio,
                    processing_time,
                    error_message: None,
                };

                let _ = self.event_sender.send(ConverterEvent::ConversionCompleted(conversion_result.clone()));
                let _ = self.event_sender.send(ConverterEvent::FileProcessed(output_file));

                Ok(conversion_result)
            },
            Err(e) => {
                let error_msg = format!("Conversion failed: {}", e);
                let _ = self.event_sender.send(ConverterEvent::Error(error_msg.clone()));

                Ok(ConversionResult {
                    success: false,
                    input_file: input_path.to_string(),
                    output_file: output_path.to_string(),
                    input_format: String::new(),
                    output_format: String::new(),
                    file_size_before,
                    file_size_after: 0,
                    compression_ratio: 0.0,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    pub async fn convert_files(&self, input_files: &[String], output_directory: &str, options: ConversionOptions) -> Result<Vec<ConversionResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (index, input_file) in input_files.iter().enumerate() {
            let file_name = std::path::Path::new(input_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            let output_extension = self.get_output_extension(&options);
            let output_path = format!("{}/{}.{}", output_directory, file_name, output_extension);

            let result = self.convert_file(input_file, &output_path, options).await?;
            results.push(result);

            let progress = ((index + 1) as f32 / input_files.len() as f32) * 100.0;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
        }

        Ok(results)
    }

    pub async fn convert_directory(&self, input_dir: &str, output_dir: &str, options: ConversionOptions, recursive: bool) -> Result<Vec<ConversionResult>, Box<dyn std::error::Error>> {
        let mut input_files = Vec::new();

        if recursive {
            for entry in walkdir::WalkDir::new(input_dir) {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            let path = entry.path().to_string_lossy().to_string();
                            if self.can_convert_file(&path) {
                                input_files.push(path);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(ConverterEvent::Error(format!("Error reading directory: {}", e)));
                    },
                }
            }
        } else {
            for entry in std::fs::read_dir(input_dir)? {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            let path_str = path.to_string_lossy().to_string();
                            if self.can_convert_file(&path_str) {
                                input_files.push(path_str);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(ConverterEvent::Error(format!("Error reading directory: {}", e)));
                    },
                }
            }
        }

        self.convert_files(&input_files, output_dir, options).await
    }

    async fn convert_audio_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_image_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_video_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_document_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_archive_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_custom_file(&self, input_path: &str, output_path: &str, options: &ConversionOptions) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn get_file_info(&self, path: &str) -> Result<MediaFile, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(path)?;
        let path_obj = std::path::Path::new(path);
        
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let format = path_obj
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let size = metadata.len();
        let created_time = metadata.created().ok();
        let modified_time = metadata.modified().ok();

        Ok(MediaFile {
            path: path.to_string(),
            name,
            format,
            size,
            created_time,
            modified_time,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn can_convert_file(&self, path: &str) -> bool {
        let path_obj = std::path::Path::new(path);
        let extension = path_obj
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match *self.converter_type.read() {
            ConverterType::Audio => {
                matches!(extension, Some("mp3") | Some("wav") | Some("flac") | Some("aac") | Some("ogg"))
            },
            ConverterType::Image => {
                matches!(extension, Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("tiff") | Some("webp"))
            },
            ConverterType::Video => {
                matches!(extension, Some("mp4") | Some("avi") | Some("mov") | Some("mkv") | Some("webm") | Some("flv"))
            },
            ConverterType::Document => {
                matches!(extension, Some("pdf") | Some("doc") | Some("docx") | Some("txt") | Some("rtf"))
            },
            ConverterType::Archive => {
                matches!(extension, Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz"))
            },
            ConverterType::Custom(_) => true,
        }
    }

    fn get_output_extension(&self, options: &ConversionOptions) -> &str {
        match *self.converter_type.read() {
            ConverterType::Audio => "mp3",
            ConverterType::Image => "png",
            ConverterType::Video => "mp4",
            ConverterType::Document => "pdf",
            ConverterType::Archive => "zip",
            ConverterType::Custom(_) => "converted",
        }
    }

    pub async fn batch_convert(&self, input_files: &[String], output_dir: &str, options: ConversionOptions, max_concurrent: usize) -> Result<Vec<ConversionResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let mut tasks = Vec::new();

        std::fs::create_dir_all(output_dir)?;

        for chunk in input_files.chunks(max_concurrent) {
            let mut chunk_tasks = Vec::new();

            for input_file in chunk {
                let converter = self.clone_converter();
                let file_name = std::path::Path::new(input_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                let output_extension = self.get_output_extension(&options);
                let output_path = format!("{}/{}.{}", output_dir, file_name, output_extension);
                let options_clone = options.clone();

                let task = tokio::spawn(async move {
                    converter.convert_file(input_file, &output_path, options_clone).await
                });

                chunk_tasks.push(task);
            }

            for task in chunk_tasks {
                match task.await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        let _ = self.event_sender.send(ConverterEvent::Error(format!("Batch conversion error: {}", e)));
                    },
                }
            }

            let progress = (results.len() as f32 / input_files.len() as f32) * 100.0;
            let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress));
        }

        Ok(results)
    }

    pub fn set_converter_type(&self, converter_type: ConverterType) {
        let mut current_type = self.converter_type.write();
        *current_type = converter_type;
    }

    pub fn get_converter_type(&self) -> ConverterType {
        self.converter_type.read().clone()
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

    pub fn get_supported_formats(&self) -> Vec<String> {
        match *self.converter_type.read() {
            ConverterType::Audio => vec![
                "mp3".to_string(),
                "wav".to_string(),
                "flac".to_string(),
                "aac".to_string(),
                "ogg".to_string(),
            ],
            ConverterType::Image => vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "tiff".to_string(),
                "webp".to_string(),
            ],
            ConverterType::Video => vec![
                "mp4".to_string(),
                "avi".to_string(),
                "mov".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
                "flv".to_string(),
            ],
            ConverterType::Document => vec![
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "txt".to_string(),
                "rtf".to_string(),
            ],
            ConverterType::Archive => vec![
                "zip".to_string(),
                "rar".to_string(),
                "7z".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
            ConverterType::Custom(_) => vec!["*".to_string()],
        }
    }

    pub fn can_convert_format(&self, format: &str) -> bool {
        let supported_formats = self.get_supported_formats();
        supported_formats.contains(&format.to_lowercase())
    }

    pub fn estimate_conversion_time(&self, input_file: &str, options: &ConversionOptions) -> std::time::Duration {
        match std::fs::metadata(input_file) {
            Ok(metadata) => {
                let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                let base_time_ms = match *self.converter_type.read() {
                    ConverterType::Audio => 50.0,
                    ConverterType::Image => 20.0,
                    ConverterType::Video => 200.0,
                    ConverterType::Document => 30.0,
                    ConverterType::Archive => 40.0,
                    ConverterType::Custom(_) => 100.0,
                };

                let time_ms = file_size_mb * base_time_ms;
                std::time::Duration::from_millis(time_ms as u64)
            },
            Err(_) => std::time::Duration::from_secs(1),
        }
    }

    pub fn estimate_output_size(&self, input_file: &str, options: &ConversionOptions) -> Result<u64, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(input_file)?;
        let input_size = metadata.len();

        let compression_factor = match *self.converter_type.read() {
            ConverterType::Audio => 0.8,
            ConverterType::Image => if options.optimize_size { 0.6 } else { 1.0 },
            ConverterType::Video => 0.7,
            ConverterType::Document => 0.5,
            ConverterType::Archive => 0.3,
            ConverterType::Custom(_) => 1.0,
        };

        Ok((input_size as f64 * compression_factor) as u64)
    }

    pub fn clone_converter(&self) -> MediaConverter {
        let mut new_converter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_converter_type(),
        );

        new_converter
    }

    pub fn reset(&self) {
        let _ = self.event_sender.send(ConverterEvent::Error("Converter reset".to_string()));
    }

    pub fn get_conversion_stats(&self) -> ConversionStats {
        ConversionStats {
            total_conversions: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            average_processing_time: std::time::Duration::from_secs(0),
            total_bytes_processed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub total_conversions: u64,
    pub successful_conversions: u64,
    pub failed_conversions: u64,
    pub average_processing_time: std::time::Duration,
    pub total_bytes_processed: u64,
}

impl Default for MediaConverter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Media Converter".to_string(),
            ConverterType::Image,
        )
    }
}

impl Default for ConverterType {
    fn default() -> Self {
        ConverterType::Image
    }
}

impl Default for ConverterEvent {
    fn default() -> Self {
        ConverterEvent::ConversionStarted
    }
}

impl Default for ConversionResult {
    fn default() -> Self {
        Self {
            success: false,
            input_file: String::new(),
            output_file: String::new(),
            input_format: String::new(),
            output_format: String::new(),
            file_size_before: 0,
            file_size_after: 0,
            compression_ratio: 0.0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for MediaFile {
    fn default() -> Self {
        Self {
            path: String::new(),
            name: String::new(),
            format: String::new(),
            size: 0,
            created_time: None,
            modified_time: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preserve_metadata: true,
            preserve_quality: true,
            optimize_size: false,
            overwrite_existing: false,
            create_backup: false,
            output_directory: None,
            custom_parameters: std::collections::HashMap::new(),
        }
    }
}

impl Default for ConversionStats {
    fn default() -> Self {
        Self {
            total_conversions: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            average_processing_time: std::time::Duration::from_secs(0),
            total_bytes_processed: 0,
        }
    }
}
