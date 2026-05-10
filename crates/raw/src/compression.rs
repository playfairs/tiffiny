use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CompressionEngine {
    pub id: String,
    pub name: String,
    pub algorithm: Arc<RwLock<CompressionAlgorithm>>,
    pub level: Arc<RwLock<CompressionLevel>>,
    pub event_sender: mpsc::UnboundedSender<CompressionEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<CompressionEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Deflate,
    Brotli,
    LZ4,
    Zstd,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionLevel {
    None,
    Fast,
    Default,
    Best,
    Custom(u8),
}

#[derive(Debug, Clone)]
pub enum CompressionEvent {
    CompressionStarted,
    CompressionProgress(f32),
    CompressionCompleted(CompressionResult),
    DecompressionStarted,
    DecompressionProgress(f32),
    DecompressionCompleted(DecompressionResult),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub success: bool,
    pub compressed_size: usize,
    pub original_size: usize,
    pub compression_ratio: f32,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DecompressionResult {
    pub success: bool,
    pub decompressed_size: usize,
    pub compressed_size: usize,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: CompressionLevel,
    pub chunk_size: usize,
    pub use_dictionary: bool,
    pub dictionary_size: Option<usize>,
    pub window_size: Option<usize>,
    pub worker_threads: Option<u32>,
}

impl CompressionEngine {
    pub fn new(id: String, name: String, algorithm: CompressionAlgorithm) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            algorithm: Arc::new(RwLock::new(algorithm)),
            level: Arc::new(RwLock::new(CompressionLevel::Default))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn compress(&self, data: &[u8], config: CompressionConfig) -> Result<CompressionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(CompressionEvent::CompressionStarted);
        let start_time = std::time::Instant::now();

        let result = match config.algorithm {
            CompressionAlgorithm::None => self.compress_none(data, &config),
            CompressionAlgorithm::Gzip => self.compress_gzip(data, &config),
            CompressionAlgorithm::Deflate => self.compress_deflate(data, &config),
            CompressionAlgorithm::Brotli => self.compress_brotli(data, &config),
            CompressionAlgorithm::LZ4 => self.compress_lz4(data, &config),
            CompressionAlgorithm::Zstd => self.compress_zstd(data, &config),
            CompressionAlgorithm::Custom(_) => self.compress_custom(data, &config),
        };

        let processing_time = start_time.elapsed();
        let original_size = data.len();

        match result {
            Ok(compressed_data) => {
                let compression_ratio = compressed_data.len() as f32 / original_size as f32;
                
                let _ = self.event_sender.send(CompressionEvent::CompressionCompleted(CompressionResult {
                    success: true,
                    compressed_size: compressed_data.len(),
                    original_size,
                    compression_ratio,
                    processing_time,
                    error_message: None,
                }));
                
                Ok(CompressionResult {
                    success: true,
                    compressed_size: compressed_data.len(),
                    original_size,
                    compression_ratio,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Compression failed: {}", e);
                let _ = self.event_sender.send(CompressionEvent::Error(error_msg.clone()));
                
                Ok(CompressionResult {
                    success: false,
                    compressed_size: 0,
                    original_size,
                    compression_ratio: 0.0,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    pub async fn decompress(&self, compressed_data: &[u8], config: CompressionConfig) -> Result<DecompressionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(CompressionEvent::DecompressionStarted);
        let start_time = std::time::Instant::now();

        let result = match config.algorithm {
            CompressionAlgorithm::None => self.decompress_none(compressed_data, &config),
            CompressionAlgorithm::Gzip => self.decompress_gzip(compressed_data, &config),
            CompressionAlgorithm::Deflate => self.decompress_deflate(compressed_data, &config),
            CompressionAlgorithm::Brotli => self.decompress_brotli(compressed_data, &config),
            CompressionAlgorithm::LZ4 => self.decompress_lz4(compressed_data, &config),
            CompressionAlgorithm::Zstd => self.decompress_zstd(compressed_data, &config),
            CompressionAlgorithm::Custom(_) => self.decompress_custom(compressed_data, &config),
        };

        let processing_time = start_time.elapsed();
        let compressed_size = compressed_data.len();

        match result {
            Ok(decompressed_data) => {
                let _ = self.event_sender.send(CompressionEvent::DecompressionCompleted(DecompressionResult {
                    success: true,
                    decompressed_size: decompressed_data.len(),
                    compressed_size,
                    processing_time,
                    error_message: None,
                }));
                
                Ok(DecompressionResult {
                    success: true,
                    decompressed_size: decompressed_data.len(),
                    compressed_size,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Decompression failed: {}", e);
                let _ = self.event_sender.send(CompressionEvent::Error(error_msg.clone()));
                
                Ok(DecompressionResult {
                    success: false,
                    decompressed_size: 0,
                    compressed_size,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    fn compress_none(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(data.to_vec())
    }

    fn compress_gzip(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let level = match config.level {
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Default => Compression::default(),
            CompressionLevel::Best => Compression::best(),
            CompressionLevel::Custom(l) => Compression::new(l),
            CompressionLevel::None => Compression::none(),
        };

        let mut encoder = GzEncoder::new(Vec::new(), level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn compress_deflate(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let level = match config.level {
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Default => Compression::default(),
            CompressionLevel::Best => Compression::best(),
            CompressionLevel::Custom(l) => Compression::new(l),
            CompressionLevel::None => Compression::none(),
        };

        let mut encoder = DeflateEncoder::new(Vec::new(), level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn compress_brotli(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use brotli::enc::{BrotliCompress, BrotliEncoderParams};
        
        let level = match config.level {
            CompressionLevel::Fast => 1,
            CompressionLevel::Default => 4,
            CompressionLevel::Best => 11,
            CompressionLevel::Custom(l) => l as u32,
            CompressionLevel::None => 0,
        };

        let params = BrotliEncoderParams {
            quality: level,
            lgwin: config.window_size.unwrap_or(22) as u32,
            lgblock: config.chunk_size as u32,
            mode: brotli::enc::BrotliEncoderMode::GENERIC,
            size_hint: data.len(),
            ..Default::default()
        };

        let mut output = Vec::new();
        brotli::enc::BrotliCompress(&mut std::io::Cursor::new(data), &params, &mut output)?;
        Ok(output)
    }

    fn compress_lz4(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use lz4::{EncoderBuilder, CompressionLevel as LZ4CompressionLevel};
        
        let level = match config.level {
            CompressionLevel::Fast => LZ4CompressionLevel::Fast,
            CompressionLevel::Default => LZ4CompressionLevel::Default,
            CompressionLevel::Best => LZ4CompressionLevel::High,
            CompressionLevel::Custom(l) => LZ4CompressionLevel::new(l),
            CompressionLevel::None => LZ4CompressionLevel::default(),
        };

        let mut encoder = EncoderBuilder::new()
            .level(level)
            .build(Vec::new())?;
        
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn compress_zstd(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use zstd;
        
        let level = match config.level {
            CompressionLevel::Fast => 1,
            CompressionLevel::Default => 3,
            CompressionLevel::Best => 22,
            CompressionLevel::Custom(l) => l as i32,
            CompressionLevel::None => 0,
        };

        let compressed_data = zstd::encode_all(data, level)?;
        Ok(compressed_data)
    }

    fn compress_custom(&self, data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
Custom compression implementation
        Ok(data.to_vec())
    }

    fn decompress_none(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(compressed_data.to_vec())
    }

    fn decompress_gzip(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn decompress_deflate(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::read::DeflateDecoder;
        use std::io::Read;

        let mut decoder = DeflateDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn decompress_brotli(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use brotli::dec::BrotliDecompress;
        
        let mut output = Vec::new();
        BrotliDecompress(&mut std::io::Cursor::new(compressed_data), compressed_data.len(), &mut output)?;
        Ok(output)
    }

    fn decompress_lz4(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use lz4::{Decoder, DecoderBuilder};
        
        let mut decoder = DecoderBuilder::new()
            .build()?;
        
        let decompressed = decoder.decompress(compressed_data, None)?;
        Ok(decompressed)
    }

    fn decompress_zstd(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use zstd;
        
        let decompressed = zstd::decode_all(compressed_data)?;
        Ok(decompressed)
    }

    fn decompress_custom(&self, compressed_data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(compressed_data.to_vec())
    }

    pub async fn compress_with_progress<F>(&self, data: &[u8], config: CompressionConfig, progress_callback: F) -> Result<CompressionResult, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(CompressionEvent::CompressionStarted);
        let start_time = std::time::Instant::now();

        let chunk_size = config.chunk_size;
        let total_chunks = (data.len() + chunk_size - 1) / chunk_size;
        
        let mut compressed_data = Vec::new();
        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            let progress = (i as f32 / total_chunks as f32) * 100.0;
            progress_callback(progress);
            
            let chunk_compressed = match config.algorithm {
                CompressionAlgorithm::Gzip => self.compress_gzip(chunk, &config)?,
                CompressionAlgorithm::Deflate => self.compress_deflate(chunk, &config)?,
                CompressionAlgorithm::Brotli => self.compress_brotli(chunk, &config)?,
                CompressionAlgorithm::LZ4 => self.compress_lz4(chunk, &config)?,
                CompressionAlgorithm::Zstd => self.compress_zstd(chunk, &config)?,
                _ => chunk.to_vec(),
            };
            
            compressed_data.extend_from_slice(&chunk_compressed);
        }

        let processing_time = start_time.elapsed();
        let original_size = data.len();
        let compression_ratio = compressed_data.len() as f32 / original_size as f32;

        let _ = self.event_sender.send(CompressionEvent::CompressionCompleted(CompressionResult {
            success: true,
            compressed_size: compressed_data.len(),
            original_size,
            compression_ratio,
            processing_time,
            error_message: None,
        }));

        Ok(CompressionResult {
            success: true,
            compressed_size: compressed_data.len(),
            original_size,
            compression_ratio,
            processing_time,
            error_message: None,
        })
    }

    pub async fn decompress_with_progress<F>(&self, compressed_data: &[u8], config: CompressionConfig, progress_callback: F) -> Result<DecompressionResult, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(CompressionEvent::DecompressionStarted);
        let start_time = std::time::Instant::now();

        let chunk_size = config.chunk_size;
        let total_chunks = (compressed_data.len() + chunk_size - 1) / chunk_size;
        
        let mut decompressed_data = Vec::new();
        for (i, chunk) in compressed_data.chunks(chunk_size).enumerate() {
            let progress = (i as f32 / total_chunks as f32) * 100.0;
            progress_callback(progress);
            
            let chunk_decompressed = match config.algorithm {
                CompressionAlgorithm::Gzip => self.decompress_gzip(chunk, &config)?,
                CompressionAlgorithm::Deflate => self.decompress_deflate(chunk, &config)?,
                CompressionAlgorithm::Brotli => self.decompress_brotli(chunk, &config)?,
                CompressionAlgorithm::LZ4 => self.decompress_lz4(chunk, &config)?,
                CompressionAlgorithm::Zstd => self.decompress_zstd(chunk, &config)?,
                _ => chunk.to_vec(),
            };
            
            decompressed_data.extend_from_slice(&chunk_decompressed);
        }

        let processing_time = start_time.elapsed();
        let compressed_size = compressed_data.len();

        let _ = self.event_sender.send(CompressionEvent::DecompressionCompleted(DecompressionResult {
            success: true,
            decompressed_size: decompressed_data.len(),
            compressed_size,
            processing_time,
            error_message: None,
        }));

        Ok(DecompressionResult {
            success: true,
            decompressed_size: decompressed_data.len(),
            compressed_size,
            processing_time,
            error_message: None,
        })
    }

    pub fn set_algorithm(&self, algorithm: CompressionAlgorithm) {
        let mut current_algorithm = self.algorithm.write();
        *current_algorithm = algorithm;
    }

    pub fn get_algorithm(&self) -> CompressionAlgorithm {
        self.algorithm.read().clone()
    }

    pub fn set_level(&self, level: CompressionLevel) {
        let mut current_level = self.level.write();
        *current_level = level;
    }

    pub fn get_level(&self) -> CompressionLevel {
        self.level.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<CompressionEvent> {
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

    pub fn get_supported_algorithms(&self) -> Vec<CompressionAlgorithm> {
        vec![
            CompressionAlgorithm::None,
            CompressionAlgorithm::Gzip,
            CompressionAlgorithm::Deflate,
            CompressionAlgorithm::Brotli,
            CompressionAlgorithm::LZ4,
            CompressionAlgorithm::Zstd,
        ]
    }

    pub fn can_compress_algorithm(&self, algorithm: &CompressionAlgorithm) -> bool {
        self.get_supported_algorithms().contains(algorithm)
    }

    pub fn estimate_compression_ratio(&self, data: &[u8], algorithm: CompressionAlgorithm) -> f32 {
        if data.is_empty() {
            return 1.0;
        }

        let entropy = self.calculate_entropy(data);
        match algorithm {
            CompressionAlgorithm::None => 1.0,
            CompressionAlgorithm::Gzip => 0.7 + entropy * 0.3,
            CompressionAlgorithm::Deflate => 0.8 + entropy * 0.2,
            CompressionAlgorithm::Brotli => 0.6 + entropy * 0.4,
            CompressionAlgorithm::LZ4 => 0.85 + entropy * 0.15,
            CompressionAlgorithm::Zstd => 0.5 + entropy * 0.5,
            CompressionAlgorithm::Custom(_) => 1.0,
        }
    }

    fn calculate_entropy(&self, data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let mut frequency = [0u32; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let mut entropy = 0.0;
        let data_len = data.len() as f32;
        
        for &freq in &frequency {
            if freq > 0 {
                let probability = freq as f32 / data_len;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    pub fn clone_engine(&self) -> CompressionEngine {
        let mut new_engine = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_algorithm(),
        );

        let level = self.get_level();
        new_engine.set_level(level);

        new_engine
    }

    pub fn reset(&self) {
        let _ = self.event_sender.send(CompressionEvent::Error("Engine reset".to_string()));
    }
}

impl Default for CompressionEngine {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Compression Engine".to_string(),
            CompressionAlgorithm::Gzip,
        )
    }
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        CompressionAlgorithm::Gzip
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel::Default
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Gzip,
            level: CompressionLevel::Default,
            chunk_size: 4096,
            use_dictionary: false,
            dictionary_size: None,
            window_size: None,
            worker_threads: None,
        }
    }
}

impl Default for CompressionResult {
    fn default() -> Self {
        Self {
            success: false,
            compressed_size: 0,
            original_size: 0,
            compression_ratio: 0.0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for DecompressionResult {
    fn default() -> Self {
        Self {
            success: false,
            decompressed_size: 0,
            compressed_size: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}
