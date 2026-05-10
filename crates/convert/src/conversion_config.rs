use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    pub input_config: InputConfig,
    pub output_config: OutputConfig,
    pub processing_config: ProcessingConfig,
    pub quality_config: QualityConfig,
    pub performance_config: PerformanceConfig,
    pub metadata_config: MetadataConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub input_path: Option<String>,
    pub input_stream: Option<String>,
    pub input_format: Option<String>,
    pub input_encoding: Option<String>,
    pub input_validation: bool,
    pub auto_detect_format: bool,
    pub chunk_size: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub output_path: Option<String>,
    pub output_stream: Option<String>,
    pub output_format: String,
    pub output_encoding: Option<String>,
    pub output_directory: Option<String>,
    pub create_subdirectories: bool,
    pub overwrite_existing: bool,
    pub create_backup: bool,
    pub preserve_structure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub processing_mode: ProcessingMode,
    pub parallel_processing: bool,
    pub max_threads: Option<usize>,
    pub thread_pool_size: usize,
    pub memory_limit: Option<usize>,
    pub temp_directory: Option<String>,
    pub cleanup_temp: bool,
    pub error_handling: ErrorHandling,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    pub quality_mode: QualityMode,
    pub quality_level: u8,
    pub preserve_quality: bool,
    pub optimize_size: bool,
    pub optimize_speed: bool,
    pub custom_quality_params: std::collections::HashMap<String, String>,
    pub quality_presets: Vec<QualityPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub performance_mode: PerformanceMode,
    pub cpu_priority: CpuPriority,
    pub memory_priority: MemoryPriority,
    pub gpu_acceleration: bool,
    pub hardware_acceleration: bool,
    pub cache_enabled: bool,
    pub cache_size: Option<usize>,
    pub cache_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub preserve_metadata: bool,
    pub strip_metadata: bool,
    pub embed_metadata: bool,
    pub metadata_format: Option<String>,
    pub custom_metadata: std::collections::HashMap<String, String>,
    pub metadata_tags: Vec<MetadataTag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessingMode {
    Sequential,
    Parallel,
    Streaming,
    Batch,
    Adaptive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QualityMode {
    Lossless,
    High,
    Medium,
    Low,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceMode {
    PowerSave,
    Balanced,
    Performance,
    Extreme,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CpuPriority {
    Low,
    Normal,
    High,
    Realtime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorHandling {
    StopOnError,
    SkipOnError,
    RetryOnError,
    ContinueOnError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay: std::time::Duration,
    pub backoff_multiplier: f32,
    pub retry_on_timeout: bool,
    pub retry_on_network_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPreset {
    pub name: String,
    pub description: String,
    pub quality_level: u8,
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataTag {
    pub key: String,
    pub value: String,
    pub tag_type: TagType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TagType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    Binary,
}

impl ConversionConfig {
    pub fn new() -> Self {
        Self {
            input_config: InputConfig::default(),
            output_config: OutputConfig::default(),
            processing_config: ProcessingConfig::default(),
            quality_config: QualityConfig::default(),
            performance_config: PerformanceConfig::default(),
            metadata_config: MetadataConfig::default(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ConversionConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref input_path) = self.input_config.input_path {
            if !std::path::Path::new(input_path).exists() {
                return Err(format!("Input file does not exist: {}", input_path).into());
            }
        }

        if let Some(ref output_path) = self.output_config.output_path {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }

        if self.processing_config.thread_pool_size == 0 {
            return Err("Thread pool size must be greater than 0".into());
        }

        if let Some(memory_limit) = self.processing_config.memory_limit {
            if memory_limit == 0 {
                return Err("Memory limit must be greater than 0".into());
            }
        }

        if self.quality_config.quality_level > 100 {
            return Err("Quality level must be between 0 and 100".into());
        }

        Ok(())
    }

    pub fn merge(&self, other: &ConversionConfig) -> ConversionConfig {
        ConversionConfig {
            input_config: InputConfig {
                input_path: other.input_config.input_path.clone().or(self.input_config.input_path.clone()),
                input_stream: other.input_config.input_stream.clone().or(self.input_config.input_stream.clone()),
                input_format: other.input_config.input_format.clone().or(self.input_config.input_format.clone()),
                input_encoding: other.input_config.input_encoding.clone().or(self.input_config.input_encoding.clone()),
                input_validation: other.input_config.input_validation,
                auto_detect_format: other.input_config.auto_detect_format,
                chunk_size: other.input_config.chunk_size.max(self.input_config.chunk_size),
                buffer_size: other.input_config.buffer_size.max(self.input_config.buffer_size),
            },
            output_config: OutputConfig {
                output_path: other.output_config.output_path.clone().or(self.output_config.output_path.clone()),
                output_stream: other.output_config.output_stream.clone().or(self.output_config.output_stream.clone()),
                output_format: other.output_config.output_format.clone(),
                output_encoding: other.output_config.output_encoding.clone().or(self.output_config.output_encoding.clone()),
                output_directory: other.output_config.output_directory.clone().or(self.output_config.output_directory.clone()),
                create_subdirectories: other.output_config.create_subdirectories || self.output_config.create_subdirectories,
                overwrite_existing: other.output_config.overwrite_existing || self.output_config.overwrite_existing,
                create_backup: other.output_config.create_backup || self.output_config.create_backup,
                preserve_structure: other.output_config.preserve_structure || self.output_config.preserve_structure,
            },
            processing_config: ProcessingConfig {
                processing_mode: other.processing_config.processing_mode.clone(),
                parallel_processing: other.processing_config.parallel_processing || self.processing_config.parallel_processing,
                max_threads: other.processing_config.max_threads.or(self.processing_config.max_threads),
                thread_pool_size: other.processing_config.thread_pool_size.max(self.processing_config.thread_pool_size),
                memory_limit: other.processing_config.memory_limit.or(self.processing_config.memory_limit),
                temp_directory: other.processing_config.temp_directory.clone().or(self.processing_config.temp_directory.clone()),
                cleanup_temp: other.processing_config.cleanup_temp && self.processing_config.cleanup_temp,
                error_handling: other.processing_config.error_handling.clone(),
                retry_config: RetryConfig {
                    max_retries: other.processing_config.retry_config.max_retries.max(self.processing_config.retry_config.max_retries),
                    retry_delay: other.processing_config.retry_config.retry_delay.min(self.processing_config.retry_config.retry_delay),
                    backoff_multiplier: other.processing_config.retry_config.backoff_multiplier.max(self.processing_config.retry_config.backoff_multiplier),
                    retry_on_timeout: other.processing_config.retry_config.retry_on_timeout || self.processing_config.retry_config.retry_on_timeout,
                    retry_on_network_error: other.processing_config.retry_config.retry_on_network_error || self.processing_config.retry_config.retry_on_network_error,
                },
            },
            quality_config: QualityConfig {
                quality_mode: other.quality_config.quality_mode.clone(),
                quality_level: other.quality_config.quality_level.max(self.quality_config.quality_level),
                preserve_quality: other.quality_config.preserve_quality || self.quality_config.preserve_quality,
                optimize_size: other.quality_config.optimize_size || self.quality_config.optimize_size,
                optimize_speed: other.quality_config.optimize_speed || self.quality_config.optimize_speed,
                custom_quality_params: {
                    let mut params = self.quality_config.custom_quality_params.clone();
                    for (key, value) in &other.quality_config.custom_quality_params {
                        params.insert(key.clone(), value.clone());
                    }
                    params
                },
                quality_presets: {
                    let mut presets = self.quality_config.quality_presets.clone();
                    for preset in &other.quality_config.quality_presets {
                        if !presets.iter().any(|p| p.name == preset.name) {
                            presets.push(preset.clone());
                        }
                    }
                    presets
                },
            },
            performance_config: PerformanceConfig {
                performance_mode: other.performance_config.performance_mode.clone(),
                cpu_priority: other.performance_config.cpu_priority.clone(),
                memory_priority: other.performance_config.memory_priority.clone(),
                gpu_acceleration: other.performance_config.gpu_acceleration || self.performance_config.gpu_acceleration,
                hardware_acceleration: other.performance_config.hardware_acceleration || self.performance_config.hardware_acceleration,
                cache_enabled: other.performance_config.cache_enabled || self.performance_config.cache_enabled,
                cache_size: other.performance_config.cache_size.or(self.performance_config.cache_size),
                cache_directory: other.performance_config.cache_directory.clone().or(self.performance_config.cache_directory.clone()),
            },
            metadata_config: MetadataConfig {
                preserve_metadata: other.metadata_config.preserve_metadata || self.metadata_config.preserve_metadata,
                strip_metadata: other.metadata_config.strip_metadata || self.metadata_config.strip_metadata,
                embed_metadata: other.metadata_config.embed_metadata || self.metadata_config.embed_metadata,
                metadata_format: other.metadata_config.metadata_format.clone().or(self.metadata_config.metadata_format.clone()),
                custom_metadata: {
                    let mut metadata = self.metadata_config.custom_metadata.clone();
                    for (key, value) in &other.metadata_config.custom_metadata {
                        metadata.insert(key.clone(), value.clone());
                    }
                    metadata
                },
                metadata_tags: {
                    let mut tags = self.metadata_config.metadata_tags.clone();
                    for tag in &other.metadata_config.metadata_tags {
                        if !tags.iter().any(|t| t.key == tag.key) {
                            tags.push(tag.clone());
                        }
                    }
                    tags
                },
            },
        }
    }

    pub fn optimize_for_speed(&mut self) {
        self.quality_config.optimize_speed = true;
        self.quality_config.optimize_size = false;
        self.performance_config.performance_mode = PerformanceMode::Performance;
        self.processing_config.parallel_processing = true;
        self.processing_config.thread_pool_size = num_cpus::get();
    }

    pub fn optimize_for_quality(&mut self) {
        self.quality_config.optimize_speed = false;
        self.quality_config.optimize_size = false;
        self.quality_config.quality_level = 100;
        self.quality_config.quality_mode = QualityMode::Lossless;
        self.performance_config.performance_mode = PerformanceMode::Balanced;
    }

    pub fn optimize_for_size(&mut self) {
        self.quality_config.optimize_size = true;
        self.quality_config.optimize_speed = false;
        self.quality_config.quality_level = 50;
        self.quality_config.quality_mode = QualityMode::Medium;
        self.performance_config.performance_mode = PerformanceMode::PowerSave;
    }

    pub fn add_quality_preset(&mut self, preset: QualityPreset) {
        self.quality_config.quality_presets.push(preset);
    }

    pub fn remove_quality_preset(&mut self, name: &str) -> bool {
        if let Some(index) = self.quality_config.quality_presets.iter().position(|p| p.name == name) {
            self.quality_config.quality_presets.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_quality_preset(&self, name: &str) -> Option<&QualityPreset> {
        self.quality_config.quality_presets.iter().find(|p| p.name == name)
    }

    pub fn add_custom_metadata(&mut self, key: String, value: String) {
        self.metadata_config.custom_metadata.insert(key, value);
    }

    pub fn remove_custom_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata_config.custom_metadata.remove(key)
    }

    pub fn get_custom_metadata(&self, key: &str) -> Option<&String> {
        self.metadata_config.custom_metadata.get(key)
    }

    pub fn add_metadata_tag(&mut self, tag: MetadataTag) {
        self.metadata_config.metadata_tags.push(tag);
    }

    pub fn remove_metadata_tag(&mut self, key: &str) -> bool {
        if let Some(index) = self.metadata_config.metadata_tags.iter().position(|t| t.key == key) {
            self.metadata_config.metadata_tags.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_metadata_tag(&self, key: &str) -> Option<&MetadataTag> {
        self.metadata_config.metadata_tags.iter().find(|t| t.key == key)
    }

    pub fn estimate_memory_usage(&self) -> usize {
        let base_memory = 1024 * 1024;
        
        let mut memory_usage = base_memory;

        memory_usage += self.input_config.buffer_size;

        memory_usage += self.input_config.buffer_size;

        if self.processing_config.parallel_processing {
            memory_usage *= self.processing_config.thread_pool_size;
        }

        if let Some(cache_size) = self.performance_config.cache_size {
            memory_usage += cache_size;
        }

        memory_usage
    }

    pub fn estimate_processing_time(&self, file_size: u64) -> std::time::Duration {
        let base_time_ms = 1000;
        
        let mut time_multiplier = 1.0;

        match self.quality_config.quality_mode {
            QualityMode::Lossless => time_multiplier *= 2.0,
            QualityMode::High => time_multiplier *= 1.5,
            QualityMode::Medium => time_multiplier *= 1.0,
            QualityMode::Low => time_multiplier *= 0.7,
            QualityMode::Custom => time_multiplier *= 1.2,
        }

        match self.performance_config.performance_mode {
            PerformanceMode::PowerSave => time_multiplier *= 1.5,
            PerformanceMode::Balanced => time_multiplier *= 1.0,
            PerformanceMode::Performance => time_multiplier *= 0.8,
            PerformanceMode::Extreme => time_multiplier *= 0.6,
        }

        if self.processing_config.parallel_processing {
            time_multiplier /= self.processing_config.thread_pool_size as f32;
        }

        let time_ms = (file_size as f64 / (1024.0 * 1024.0)) * base_time_ms as f64 * time_multiplier;
        std::time::Duration::from_millis(time_ms as u64)
    }

    pub fn estimate_output_size(&self, input_size: u64) -> u64 {
        let mut size_multiplier = 1.0;

        match self.quality_config.quality_mode {
            QualityMode::Lossless => size_multiplier *= 1.0,
            QualityMode::High => size_multiplier *= 0.9,
            QualityMode::Medium => size_multiplier *= 0.7,
            QualityMode::Low => size_multiplier *= 0.5,
            QualityMode::Custom => size_multiplier *= (self.quality_config.quality_level as f64 / 100.0),
        }

        if self.quality_config.optimize_size {
            size_multiplier *= 0.8;
        }

        (input_size as f64 * size_multiplier) as u64
    }

    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn clone_config(&self) -> ConversionConfig {
        ConversionConfig {
            input_config: InputConfig {
                input_path: self.input_config.input_path.clone(),
                input_stream: self.input_config.input_stream.clone(),
                input_format: self.input_config.input_format.clone(),
                input_encoding: self.input_config.input_encoding.clone(),
                input_validation: self.input_config.input_validation,
                auto_detect_format: self.input_config.auto_detect_format,
                chunk_size: self.input_config.chunk_size,
                buffer_size: self.input_config.buffer_size,
            },
            output_config: OutputConfig {
                output_path: self.output_config.output_path.clone(),
                output_stream: self.output_config.output_stream.clone(),
                output_format: self.output_config.output_format.clone(),
                output_encoding: self.output_config.output_encoding.clone(),
                output_directory: self.output_config.output_directory.clone(),
                create_subdirectories: self.output_config.create_subdirectories,
                overwrite_existing: self.output_config.overwrite_existing,
                create_backup: self.output_config.create_backup,
                preserve_structure: self.output_config.preserve_structure,
            },
            processing_config: ProcessingConfig {
                processing_mode: self.processing_config.processing_mode.clone(),
                parallel_processing: self.processing_config.parallel_processing,
                max_threads: self.processing_config.max_threads,
                thread_pool_size: self.processing_config.thread_pool_size,
                memory_limit: self.processing_config.memory_limit,
                temp_directory: self.processing_config.temp_directory.clone(),
                cleanup_temp: self.processing_config.cleanup_temp,
                error_handling: self.processing_config.error_handling.clone(),
                retry_config: RetryConfig {
                    max_retries: self.processing_config.retry_config.max_retries,
                    retry_delay: self.processing_config.retry_config.retry_delay,
                    backoff_multiplier: self.processing_config.retry_config.backoff_multiplier,
                    retry_on_timeout: self.processing_config.retry_config.retry_on_timeout,
                    retry_on_network_error: self.processing_config.retry_config.retry_on_network_error,
                },
            },
            quality_config: QualityConfig {
                quality_mode: self.quality_config.quality_mode.clone(),
                quality_level: self.quality_config.quality_level,
                preserve_quality: self.quality_config.preserve_quality,
                optimize_size: self.quality_config.optimize_size,
                optimize_speed: self.quality_config.optimize_speed,
                custom_quality_params: self.quality_config.custom_quality_params.clone(),
                quality_presets: self.quality_config.quality_presets.clone(),
            },
            performance_config: PerformanceConfig {
                performance_mode: self.performance_config.performance_mode.clone(),
                cpu_priority: self.performance_config.cpu_priority.clone(),
                memory_priority: self.performance_config.memory_priority.clone(),
                gpu_acceleration: self.performance_config.gpu_acceleration,
                hardware_acceleration: self.performance_config.hardware_acceleration,
                cache_enabled: self.performance_config.cache_enabled,
                cache_size: self.performance_config.cache_size,
                cache_directory: self.performance_config.cache_directory.clone(),
            },
            metadata_config: MetadataConfig {
                preserve_metadata: self.metadata_config.preserve_metadata,
                strip_metadata: self.metadata_config.strip_metadata,
                embed_metadata: self.metadata_config.embed_metadata,
                metadata_format: self.metadata_config.metadata_format.clone(),
                custom_metadata: self.metadata_config.custom_metadata.clone(),
                metadata_tags: self.metadata_config.metadata_tags.clone(),
            },
        }
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            input_path: None,
            input_stream: None,
            input_format: None,
            input_encoding: None,
            input_validation: true,
            auto_detect_format: true,
            chunk_size: 4096,
            buffer_size: 8192,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_path: None,
            output_stream: None,
            output_format: "mp4".to_string(),
            output_encoding: None,
            output_directory: None,
            create_subdirectories: false,
            overwrite_existing: false,
            create_backup: false,
            preserve_structure: false,
        }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            processing_mode: ProcessingMode::Adaptive,
            parallel_processing: true,
            max_threads: None,
            thread_pool_size: num_cpus::get(),
            memory_limit: None,
            temp_directory: None,
            cleanup_temp: true,
            error_handling: ErrorHandling::RetryOnError,
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            quality_mode: QualityMode::Medium,
            quality_level: 75,
            preserve_quality: true,
            optimize_size: false,
            optimize_speed: false,
            custom_quality_params: std::collections::HashMap::new(),
            quality_presets: Vec::new(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            performance_mode: PerformanceMode::Balanced,
            cpu_priority: CpuPriority::Normal,
            memory_priority: MemoryPriority::Normal,
            gpu_acceleration: true,
            hardware_acceleration: true,
            cache_enabled: true,
            cache_size: Some(100 * 1024 * 1024),
            cache_directory: None,
        }
    }
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            preserve_metadata: true,
            strip_metadata: false,
            embed_metadata: false,
            metadata_format: None,
            custom_metadata: std::collections::HashMap::new(),
            metadata_tags: Vec::new(),
        }
    }
}

impl Default for ProcessingMode {
    fn default() -> Self {
        ProcessingMode::Adaptive
    }
}

impl Default for QualityMode {
    fn default() -> Self {
        QualityMode::Medium
    }
}

impl Default for PerformanceMode {
    fn default() -> Self {
        PerformanceMode::Balanced
    }
}

impl Default for CpuPriority {
    fn default() -> Self {
        CpuPriority::Normal
    }
}

impl Default for MemoryPriority {
    fn default() -> Self {
        MemoryPriority::Normal
    }
}

impl Default for ErrorHandling {
    fn default() -> Self {
        ErrorHandling::RetryOnError
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_network_error: true,
        }
    }
}

impl Default for QualityPreset {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: "Default quality preset".to_string(),
            quality_level: 75,
            parameters: std::collections::HashMap::new(),
        }
    }
}

impl Default for MetadataTag {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            tag_type: TagType::String,
        }
    }
}

impl Default for TagType {
    fn default() -> Self {
        TagType::String
    }
}
