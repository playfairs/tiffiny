use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct FormatConverter {
    pub id: String,
    pub name: String,
    pub source_format: Arc<RwLock<MediaFormat>>,
    pub target_format: Arc<RwLock<MediaFormat>>,
    pub event_sender: mpsc::UnboundedSender<FormatConverterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<FormatConverterEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaFormat {
    Audio(AudioFormat),
    Image(ImageFormat),
    Video(VideoFormat),
    Document(DocumentFormat),
    Archive(ArchiveFormat),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    MP3,
    WAV,
    FLAC,
    AAC,
    OGG,
    WMA,
    M4A,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    JPEG,
    PNG,
    GIF,
    BMP,
    TIFF,
    WEBP,
    ICO,
    SVG,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    MP4,
    AVI,
    MOV,
    MKV,
    WEBM,
    FLV,
    WMV,
    M4V,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentFormat {
    PDF,
    DOC,
    DOCX,
    TXT,
    RTF,
    HTML,
    ODT,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveFormat {
    ZIP,
    RAR,
    SEVEN_Z,
    TAR,
    GZIP,
    BZIP2,
    XZ,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum FormatConverterEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted(FormatConversionResult),
    Error(String),
    FormatDetected(MediaFormat),
}

#[derive(Debug, Clone)]
pub struct FormatConversionResult {
    pub success: bool,
    pub source_format: MediaFormat,
    pub target_format: MediaFormat,
    pub output_path: String,
    pub file_size_before: u64,
    pub file_size_after: u64,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FormatConversionConfig {
    pub source_format: MediaFormat,
    pub target_format: MediaFormat,
    pub quality: Option<ConversionQuality>,
    pub preserve_metadata: bool,
    pub optimize_size: bool,
    pub custom_parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionQuality {
    Low,
    Medium,
    High,
    Ultra,
    Lossless,
    Custom(u8),1-100
}

#[derive(Debug, Clone)]
pub struct FormatDetectionResult {
    pub detected_format: Option<MediaFormat>,
    pub confidence: f32,
    pub file_size: u64,
    pub mime_type: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl FormatConverter {
    pub fn new(id: String, name: String, source_format: MediaFormat, target_format: MediaFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            source_format: Arc::new(RwLock::new(source_format)),
            target_format: Arc::new(RwLock::new(target_format)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn convert_file(&self, input_path: &str, output_path: &str, config: FormatConversionConfig) -> Result<FormatConversionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(FormatConverterEvent::ConversionStarted);
        let start_time = std::time::Instant::now();

        let metadata = std::fs::metadata(input_path)?;
        let file_size_before = metadata.len();

        let result = match (&config.source_format, &config.target_format) {
            (MediaFormat::Audio(_), MediaFormat::Audio(_)) => {
                self.convert_audio_to_audio(input_path, output_path, &config).await
            },
            (MediaFormat::Image(_), MediaFormat::Image(_)) => {
                self.convert_image_to_image(input_path, output_path, &config).await
            },
            (MediaFormat::Video(_), MediaFormat::Video(_)) => {
                self.convert_video_to_video(input_path, output_path, &config).await
            },
            (MediaFormat::Document(_), MediaFormat::Document(_)) => {
                self.convert_document_to_document(input_path, output_path, &config).await
            },
            (MediaFormat::Archive(_), MediaFormat::Archive(_)) => {
                self.convert_archive_to_archive(input_path, output_path, &config).await
            },
            (MediaFormat::Image(_), MediaFormat::Document(_)) => {
                self.convert_image_to_document(input_path, output_path, &config).await
            },
            (MediaFormat::Document(_), MediaFormat::Image(_)) => {
                self.convert_document_to_image(input_path, output_path, &config).await
            },
            (MediaFormat::Audio(_), MediaFormat::Video(_)) => {
                self.convert_audio_to_video(input_path, output_path, &config).await
            },
            (MediaFormat::Video(_), MediaFormat::Audio(_)) => {
                self.convert_video_to_audio(input_path, output_path, &config).await
            },
            (MediaFormat::Image(_), MediaFormat::Video(_)) => {
                self.convert_image_to_video(input_path, output_path, &config).await
            },
            (MediaFormat::Video(_), MediaFormat::Image(_)) => {
                self.convert_video_to_image(input_path, output_path, &config).await
            },
            _ => {
                self.convert_generic(input_path, output_path, &config).await
            },
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_path) => {
                let output_metadata = std::fs::metadata(&output_path)?;
                let file_size_after = output_metadata.len();

                let conversion_result = FormatConversionResult {
                    success: true,
                    source_format: config.source_format.clone(),
                    target_format: config.target_format.clone(),
                    output_path,
                    file_size_before,
                    file_size_after,
                    processing_time,
                    error_message: None,
                };

                let _ = self.event_sender.send(FormatConverterEvent::ConversionCompleted(conversion_result.clone()));
                Ok(conversion_result)
            },
            Err(e) => {
                let error_msg = format!("Conversion failed: {}", e);
                let _ = self.event_sender.send(FormatConverterEvent::Error(error_msg.clone()));

                Ok(FormatConversionResult {
                    success: false,
                    source_format: config.source_format.clone(),
                    target_format: config.target_format.clone(),
                    output_path: output_path.to_string(),
                    file_size_before,
                    file_size_after: 0,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn convert_audio_to_audio(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_image_to_image(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_video_to_video(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_document_to_document(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_archive_to_archive(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_image_to_document(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_document_to_image(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(350)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_audio_to_video(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_video_to_audio(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(450)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_image_to_video(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_video_to_image(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    async fn convert_generic(&self, input_path: &str, output_path: &str, config: &FormatConversionConfig) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        std::fs::copy(input_path, output_path)?;
        
        Ok(output_path.to_string())
    }

    pub async fn detect_format(&self, file_path: &str) -> Result<FormatDetectionResult, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();
        let path_obj = std::path::Path::new(file_path);
        
        let extension = path_obj
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        let (detected_format, confidence, mime_type) = match extension {
            Some("mp3") => (Some(MediaFormat::Audio(AudioFormat::MP3)), 0.9, Some("audio/mpeg".to_string())),
            Some("wav") => (Some(MediaFormat::Audio(AudioFormat::WAV)), 0.95, Some("audio/wav".to_string())),
            Some("flac") => (Some(MediaFormat::Audio(AudioFormat::FLAC)), 0.95, Some("audio/flac".to_string())),
            Some("aac") => (Some(MediaFormat::Audio(AudioFormat::AAC)), 0.9, Some("audio/aac".to_string())),
            Some("ogg") => (Some(MediaFormat::Audio(AudioFormat::OGG)), 0.9, Some("audio/ogg".to_string())),
            Some("jpg") | Some("jpeg") => (Some(MediaFormat::Image(ImageFormat::JPEG)), 0.95, Some("image/jpeg".to_string())),
            Some("png") => (Some(MediaFormat::Image(ImageFormat::PNG)), 0.95, Some("image/png".to_string())),
            Some("gif") => (Some(MediaFormat::Image(ImageFormat::GIF)), 0.9, Some("image/gif".to_string())),
            Some("bmp") => (Some(MediaFormat::Image(ImageFormat::BMP)), 0.9, Some("image/bmp".to_string())),
            Some("tiff") | Some("tif") => (Some(MediaFormat::Image(ImageFormat::TIFF)), 0.9, Some("image/tiff".to_string())),
            Some("webp") => (Some(MediaFormat::Image(ImageFormat::WEBP)), 0.9, Some("image/webp".to_string())),
            Some("mp4") => (Some(MediaFormat::Video(VideoFormat::MP4)), 0.95, Some("video/mp4".to_string())),
            Some("avi") => (Some(MediaFormat::Video(VideoFormat::AVI)), 0.9, Some("video/avi".to_string())),
            Some("mov") => (Some(MediaFormat::Video(VideoFormat::MOV)), 0.9, Some("video/quicktime".to_string())),
            Some("mkv") => (Some(MediaFormat::Video(VideoFormat::MKV)), 0.9, Some("video/x-matroska".to_string())),
            Some("webm") => (Some(MediaFormat::Video(VideoFormat::WEBM)), 0.9, Some("video/webm".to_string())),
            Some("flv") => (Some(MediaFormat::Video(VideoFormat::FLV)), 0.9, Some("video/x-flv".to_string())),
            Some("pdf") => (Some(MediaFormat::Document(DocumentFormat::PDF)), 0.95, Some("application/pdf".to_string())),
            Some("doc") => (Some(MediaFormat::Document(DocumentFormat::DOC)), 0.9, Some("application/msword".to_string())),
            Some("docx") => (Some(MediaFormat::Document(DocumentFormat::DOCX)), 0.9, Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string())),
            Some("txt") => (Some(MediaFormat::Document(DocumentFormat::TXT)), 0.9, Some("text/plain".to_string())),
            Some("rtf") => (Some(MediaFormat::Document(DocumentFormat::RTF)), 0.9, Some("application/rtf".to_string())),
            Some("html") | Some("htm") => (Some(MediaFormat::Document(DocumentFormat::HTML)), 0.9, Some("text/html".to_string())),
            Some("zip") => (Some(MediaFormat::Archive(ArchiveFormat::ZIP)), 0.95, Some("application/zip".to_string())),
            Some("rar") => (Some(MediaFormat::Archive(ArchiveFormat::RAR)), 0.9, Some("application/x-rar-compressed".to_string())),
            Some("7z") => (Some(MediaFormat::Archive(ArchiveFormat::SEVEN_Z)), 0.9, Some("application/x-7z-compressed".to_string())),
            Some("tar") => (Some(MediaFormat::Archive(ArchiveFormat::TAR)), 0.9, Some("application/x-tar".to_string())),
            Some("gz") => (Some(MediaFormat::Archive(ArchiveFormat::GZIP)), 0.9, Some("application/gzip".to_string())),
            _ => (None, 0.0, None),
        };

        let _ = self.event_sender.send(FormatConverterEvent::FormatDetected(detected_format.clone().unwrap_or(MediaFormat::Custom("unknown".to_string()))));

        Ok(FormatDetectionResult {
            detected_format,
            confidence,
            file_size,
            mime_type,
            metadata: std::collections::HashMap::new(),
        })
    }

    pub fn set_source_format(&self, format: MediaFormat) {
        let mut source_format = self.source_format.write();
        *source_format = format;
    }

    pub fn set_target_format(&self, format: MediaFormat) {
        let mut target_format = self.target_format.write();
        *target_format = format;
    }

    pub fn get_source_format(&self) -> MediaFormat {
        self.source_format.read().clone()
    }

    pub fn get_target_format(&self) -> MediaFormat {
        self.target_format.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<FormatConverterEvent> {
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

    pub fn get_supported_conversions(&self) -> Vec<(MediaFormat, MediaFormat)> {
        let mut conversions = Vec::new();
        
        let audio_formats = vec![
            MediaFormat::Audio(AudioFormat::MP3),
            MediaFormat::Audio(AudioFormat::WAV),
            MediaFormat::Audio(AudioFormat::FLAC),
            MediaFormat::Audio(AudioFormat::AAC),
            MediaFormat::Audio(AudioFormat::OGG),
        ];
        
        let image_formats = vec![
            MediaFormat::Image(ImageFormat::JPEG),
            MediaFormat::Image(ImageFormat::PNG),
            MediaFormat::Image(ImageFormat::GIF),
            MediaFormat::Image(ImageFormat::BMP),
            MediaFormat::Image(ImageFormat::TIFF),
            MediaFormat::Image(ImageFormat::WEBP),
        ];
        
        let video_formats = vec![
            MediaFormat::Video(VideoFormat::MP4),
            MediaFormat::Video(VideoFormat::AVI),
            MediaFormat::Video(VideoFormat::MOV),
            MediaFormat::Video(VideoFormat::MKV),
            MediaFormat::Video(VideoFormat::WEBM),
            MediaFormat::Video(VideoFormat::FLV),
        ];
        
        let document_formats = vec![
            MediaFormat::Document(DocumentFormat::PDF),
            MediaFormat::Document(DocumentFormat::DOC),
            MediaFormat::Document(DocumentFormat::DOCX),
            MediaFormat::Document(DocumentFormat::TXT),
            MediaFormat::Document(DocumentFormat::RTF),
            MediaFormat::Document(DocumentFormat::HTML),
        ];
        
        let archive_formats = vec![
            MediaFormat::Archive(ArchiveFormat::ZIP),
            MediaFormat::Archive(ArchiveFormat::RAR),
            MediaFormat::Archive(ArchiveFormat::SEVEN_Z),
            MediaFormat::Archive(ArchiveFormat::TAR),
            MediaFormat::Archive(ArchiveFormat::GZIP),
        ];
        
        for format in &audio_formats {
            for target in &audio_formats {
                if format != target {
                    conversions.push((format.clone(), target.clone()));
                }
            }
        }
        
        for format in &image_formats {
            for target in &image_formats {
                if format != target {
                    conversions.push((format.clone(), target.clone()));
                }
            }
        }
        
        for format in &video_formats {
            for target in &video_formats {
                if format != target {
                    conversions.push((format.clone(), target.clone()));
                }
            }
        }
        
        for format in &document_formats {
            for target in &document_formats {
                if format != target {
                    conversions.push((format.clone(), target.clone()));
                }
            }
        }
        
        for format in &archive_formats {
            for target in &archive_formats {
                if format != target {
                    conversions.push((format.clone(), target.clone()));
                }
            }
        }
        
        for image in &image_formats {
            for document in &document_formats {
                conversions.push((image.clone(), document.clone()));
                conversions.push((document.clone(), image.clone()));
            }
        }
        
        for video in &video_formats {
            for audio in &audio_formats {
                conversions.push((video.clone(), audio.clone()));
                conversions.push((audio.clone(), video.clone()));
            }
        }
        
        for image in &image_formats {
            for video in &video_formats {
                conversions.push((image.clone(), video.clone()));
                conversions.push((video.clone(), image.clone()));
            }
        }
        
        conversions
    }

    pub fn can_convert(&self, source: &MediaFormat, target: &MediaFormat) -> bool {
        let supported_conversions = self.get_supported_conversions();
        supported_conversions.contains(&(source.clone(), target.clone()))
    }

    pub fn estimate_conversion_time(&self, input_path: &str, config: &FormatConversionConfig) -> std::time::Duration {
        match std::fs::metadata(input_path) {
            Ok(metadata) => {
                let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                
                let base_time_ms = match (&config.source_format, &config.target_format) {
                    (MediaFormat::Audio(_), MediaFormat::Audio(_)) => 50.0,
                    (MediaFormat::Image(_), MediaFormat::Image(_)) => 100.0,
                    (MediaFormat::Video(_), MediaFormat::Video(_)) => 500.0,
                    (MediaFormat::Document(_), MediaFormat::Document(_)) => 200.0,
                    (MediaFormat::Archive(_), MediaFormat::Archive(_)) => 150.0,
                    (MediaFormat::Image(_), MediaFormat::Document(_)) => 300.0,
                    (MediaFormat::Document(_), MediaFormat::Image(_)) => 250.0,
                    (MediaFormat::Audio(_), MediaFormat::Video(_)) => 600.0,
                    (MediaFormat::Video(_), MediaFormat::Audio(_)) => 400.0,
                    (MediaFormat::Image(_), MediaFormat::Video(_)) => 700.0,
                    (MediaFormat::Video(_), MediaFormat::Image(_)) => 200.0,
                    _ => 300.0,
                };

                let time_ms = file_size_mb * base_time_ms;
                std::time::Duration::from_millis(time_ms as u64)
            },
            Err(_) => std::time::Duration::from_secs(1),
        }
    }

    pub fn estimate_output_size(&self, input_path: &str, config: &FormatConversionConfig) -> Result<u64, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(input_path)?;
        let input_size = metadata.len();

        let compression_factor = match (&config.source_format, &config.target_format) {
            (MediaFormat::Image(_), MediaFormat::Image(_)) => {
                match config.quality {
                    Some(ConversionQuality::Low) => 0.3,
                    Some(ConversionQuality::Medium) => 0.6,
                    Some(ConversionQuality::High) => 0.8,
                    Some(ConversionQuality::Ultra) => 0.9,
                    Some(ConversionQuality::Lossless) => 1.0,
                    Some(ConversionQuality::Custom(q)) => q as f32 / 100.0,
                    None => 0.7,
                }
            },
            (MediaFormat::Audio(_), MediaFormat::Audio(_)) => {
                match config.quality {
                    Some(ConversionQuality::Low) => 0.2,
                    Some(ConversionQuality::Medium) => 0.5,
                    Some(ConversionQuality::High) => 0.7,
                    Some(ConversionQuality::Ultra) => 0.9,
                    Some(ConversionQuality::Lossless) => 1.0,
                    Some(ConversionQuality::Custom(q)) => q as f32 / 100.0,
                    None => 0.6,
                }
            },
            (MediaFormat::Video(_), MediaFormat::Video(_)) => {
                match config.quality {
                    Some(ConversionQuality::Low) => 0.4,
                    Some(ConversionQuality::Medium) => 0.6,
                    Some(ConversionQuality::High) => 0.8,
                    Some(ConversionQuality::Ultra) => 0.9,
                    Some(ConversionQuality::Lossless) => 1.0,
                    Some(ConversionQuality::Custom(q)) => q as f32 / 100.0,
                    None => 0.7,
                }
            },
            _ => 1.0,
        };

        Ok((input_size as f64 * compression_factor) as u64)
    }

    pub fn clone_converter(&self) -> FormatConverter {
        let mut new_converter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_source_format(),
            self.get_target_format(),
        );

        new_converter
    }

    pub fn reset(&self) {
        let _ = self.event_sender.send(FormatConverterEvent::Error("Converter reset".to_string()));
    }
}

impl Default for FormatConverter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Format Converter".to_string(),
            MediaFormat::Image(ImageFormat::JPEG),
            MediaFormat::Image(ImageFormat::PNG),
        )
    }
}

impl Default for MediaFormat {
    fn default() -> Self {
        MediaFormat::Image(ImageFormat::JPEG)
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::MP3
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        ImageFormat::JPEG
    }
}

impl Default for VideoFormat {
    fn default() -> Self {
        VideoFormat::MP4
    }
}

impl Default for DocumentFormat {
    fn default() -> Self {
        DocumentFormat::PDF
    }
}

impl Default for ArchiveFormat {
    fn default() -> Self {
        ArchiveFormat::ZIP
    }
}

impl Default for FormatConverterEvent {
    fn default() -> Self {
        FormatConverterEvent::ConversionStarted
    }
}

impl Default for FormatConversionResult {
    fn default() -> Self {
        Self {
            success: false,
            source_format: MediaFormat::default(),
            target_format: MediaFormat::default(),
            output_path: String::new(),
            file_size_before: 0,
            file_size_after: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for FormatConversionConfig {
    fn default() -> Self {
        Self {
            source_format: MediaFormat::default(),
            target_format: MediaFormat::default(),
            quality: Some(ConversionQuality::Medium),
            preserve_metadata: true,
            optimize_size: false,
            custom_parameters: std::collections::HashMap::new(),
        }
    }
}

impl Default for ConversionQuality {
    fn default() -> Self {
        ConversionQuality::Medium
    }
}

impl Default for FormatDetectionResult {
    fn default() -> Self {
        Self {
            detected_format: None,
            confidence: 0.0,
            file_size: 0,
            mime_type: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}
