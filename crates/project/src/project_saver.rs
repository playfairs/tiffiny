use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProjectSaver {
    pub id: String,
    pub name: String,
    pub event_sender: mpsc::UnboundedSender<SaverEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SaverEvent>>>>,
    pub settings: Arc<RwLock<SaverSettings>>,
    pub active_saves: Arc<RwLock<HashMap<String, SaveOperation>>>>,
}

#[derive(Debug, Clone)]
pub enum SaverEvent {
    SaveStarted(String),
    SaveProgress(String, f32),
    SaveCompleted(String, SaveResult),
    SaveFailed(String, String),
    BackupCreated(String, PathBuf),
    BackupFailed(String, String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct SaverSettings {
    pub auto_save: bool,
    pub auto_save_interval: std::time::Duration,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    pub save_format: SaveFormat,
    pub max_concurrent_saves: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveFormat {
    Json,
    Toml,
    Binary,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SaveOperation {
    pub id: String,
    pub project_id: String,
    pub save_path: PathBuf,
    pub format: SaveFormat,
    pub compression: bool,
    pub encryption: bool,
    pub progress: Arc<RwLock<SaveProgress>>,
    pub start_time: std::time::Instant,
    pub status: Arc<RwLock<SaveStatus>>,
}

#[derive(Debug, Clone)]
pub struct SaveProgress {
    pub stage: SaveStage,
    pub progress: f32,
    pub message: String,
    pub bytes_saved: u64,
    pub total_bytes: u64,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveStage {
    Initializing,
    Preparing,
    Serializing,
    Compressing,
    Encrypting,
    Writing,
    Validating,
    Completed,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct SaveResult {
    pub success: bool,
    pub save_path: PathBuf,
    pub file_size: u64,
    pub compression_ratio: Option<f32>,
    pub checksum: String,
    pub save_time: std::time::Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub backup_path: PathBuf,
    pub backup_count: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub include_metadata: bool,
    pub timestamp_format: String,
}

impl ProjectSaver {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            settings: Arc::new(RwLock::new(SaverSettings::default())),
            active_saves: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_project(&self, project: &super::Project, path: &Path) -> Result<SaveResult, Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        let save_id = uuid::Uuid::new_v4().to_string();
        
        let _ = self.event_sender.send(SaverEvent::SaveStarted(save_id.clone()));
        
Create save operation
        let save_operation = SaveOperation {
            id: save_id.clone(),
            project_id: project.id.clone(),
            save_path: path.to_path_buf(),
            format: settings.save_format.clone(),
            compression: settings.compression_enabled,
            encryption: settings.encryption_enabled,
            progress: Arc::new(RwLock::new(SaveProgress::default())),
            start_time: std::time::Instant::now(),
            status: Arc::new(RwLock::new(SaveStatus::Running)),
        };

        {
            let mut active_saves = self.active_saves.write();
            active_saves.insert(save_id.clone(), save_operation.clone());
        }

        let result = self.perform_save(&save_operation, project).await;

        {
            let mut active_saves = self.active_saves.write();
            active_saves.remove(&save_id);
        }

        let _ = self.event_sender.send(SaverEvent::SaveCompleted(save_id.clone(), result.clone()));

        result
    }

    async fn perform_save(&self, operation: &SaveOperation, project: &super::Project) -> Result<SaveResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        self.update_progress(&operation, SaveStage::Initializing, 0.0, "Initializing save").await;

        self.update_progress(&operation, SaveStage::Preparing, 10.0, "Preparing save directory").await;
        if let Some(parent) = operation.save_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        self.update_progress(&operation, SaveStage::Serializing, 30.0, "Serializing project data").await;
        let serialized_data = self.serialize_project(project, &operation.format).await?;
        let original_size = serialized_data.len() as u64;

        let mut compressed_data = serialized_data;
        let mut compression_ratio = None;
        
        if operation.compression {
            self.update_progress(&operation, SaveStage::Compressing, 50.0, "Compressing data").await;
            compressed_data = self.compress_data(&serialized_data, &operation).await?;
            compression_ratio = Some(compressed_data.len() as f32 / original_size as f32);
        }

        let mut final_data = compressed_data;
        if operation.encryption {
            self.update_progress(&operation, SaveStage::Encrypting, 70.0, "Encrypting data").await;
            final_data = self.encrypt_data(&compressed_data, &operation).await?;
        }

        self.update_progress(&operation, SaveStage::Writing, 80.0, "Writing to file").await;
        self.write_data_to_file(&final_data, &operation.save_path, &operation).await?;

        self.update_progress(&operation, SaveStage::Validating, 90.0, "Validating saved file").await;
        if let Err(e) = self.validate_saved_file(&operation.save_path, project).await {
            errors.push(format!("Validation failed: {}", e));
        }

        let checksum = self.calculate_checksum(&final_data).await;

        let save_time = start_time.elapsed();
        let file_size = tokio::fs::metadata(&operation.save_path).await?.len();

        self.update_progress(&operation, SaveStage::Completed, 100.0, "Save completed").await;

        Ok(SaveResult {
            success: errors.is_empty(),
            save_path: operation.save_path.clone(),
            file_size,
            compression_ratio,
            checksum,
            save_time,
            warnings,
            errors,
        })
    }

    async fn serialize_project(&self, project: &super::Project, format: &SaveFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match format {
            SaveFormat::Json => {
                let json_data = serde_json::to_string_pretty(project)?;
                Ok(json_data.into_bytes())
            },
            SaveFormat::Toml => {
                let toml_data = toml::to_string_pretty(project)?;
                Ok(toml_data.into_bytes())
            },
            SaveFormat::Binary => {
                let binary_data = bincode::serialize(project)?;
                Ok(binary_data)
            },
            SaveFormat::Custom(format_name) => {
                match format_name.as_str() {
                    "yaml" => {
                        Err("YAML format not yet implemented".into())
                    },
                    "xml" => {
                        Err("XML format not yet implemented".into())
                    },
                    _ => Err(format!("Unsupported custom format: {}", format_name)).into(),
                }
            },
        }
    }

    async fn compress_data(&self, data: &[u8], operation: &SaveOperation) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        let compression_level = settings.compression_level;
        
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(compression_level));
        
        let mut encoder = encoder;
        encoder.write_all(data)?;
        encoder.finish().map_err(|e| format!("Compression failed: {}", e).into())
    }

    async fn encrypt_data(&self, data: &[u8], operation: &SaveOperation) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        
        if let Some(ref key) = settings.encryption_key {
            let key_bytes = key.as_bytes();
            let mut encrypted_data = Vec::with_capacity(data.len());
            
            for (i, &byte) in data.iter().enumerate() {
                let key_byte = key_bytes[i % key_bytes.len()];
                encrypted_data.push(byte ^ key_byte);
            }
            
            Ok(encrypted_data)
        } else {
            Err("No encryption key provided".into())
        }
    }

    async fn write_data_to_file(&self, data: &[u8], path: &Path, operation: &SaveOperation) -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, data).await?;
        
        let written_data = tokio::fs::read(&temp_path).await?;
        if written_data != data {
            tokio::fs::remove_file(&temp_path).await?;
            return Err("Data verification failed".into());
        }
        
        tokio::fs::rename(&temp_path, path).await?;
        
        Ok(())
    }

    async fn validate_saved_file(&self, path: &Path, original_project: &super::Project) -> Result<(), Box<dyn std::error::Error>> {
        let saved_data = tokio::fs::read(path).await?;
        
        let loaded_project: super::Project = match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::from_slice(&saved_data)?,
            Some("toml") => toml::from_slice(&saved_data)?,
            Some("bin") => bincode::deserialize(&saved_data)?,
            _ => return Err("Unknown file format".into()),
        };

        if loaded_project.id != original_project.id {
            return Err("Project ID mismatch".into());
        }

        if loaded_project.name != original_project.name {
            return Err("Project name mismatch".into());
        }

        if loaded_project.assets.len() != original_project.assets.len() {
            return Err("Asset count mismatch".into());
        }

        Ok(())
    }

    async fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    async fn update_progress(&self, operation: &SaveOperation, stage: SaveStage, progress: f32, message: &str) {
        let mut save_progress = operation.progress.write();
        save_progress.stage = stage.clone();
        save_progress.progress = progress;
        save_progress.message = message.to_string();
        
        let _ = self.event_sender.send(SaverEvent::SaveProgress(
            operation.id.clone(),
            progress,
        ));
    }

    pub async fn create_backup(&self, project: &super::Project, config: BackupConfig) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        
        let timestamp = chrono::Utc::now().format(&config.timestamp_format);
        let backup_filename = format!("{}_{}_{}", project.name, project.id, timestamp);
        let backup_path = config.backup_path.join(format!("{}.tiffiny", backup_filename));

        let result = self.save_project(project, &backup_path).await?;

        if result.success {
            let _ = self.event_sender.send(SaverEvent::BackupCreated(backup_id.clone(), backup_path.clone()));
            Ok(backup_path)
        } else {
            let error_msg = format!("Backup failed: {:?}", result.errors);
            let _ = self.event_sender.send(SaverEvent::BackupFailed(backup_id.clone(), error_msg.clone()));
            Err(error_msg.into())
        }
    }

    pub async fn auto_save_project(&self, project: &super::Project) -> Result<Option<SaveResult>, Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        
        if !settings.auto_save {
            return Ok(None);
        }

        let auto_save_path = self.get_auto_save_path(project)?;
        
        let result = self.save_project(project, &auto_save_path).await?;
        
        Ok(Some(result))
    }

    fn get_auto_save_path(&self, project: &super::Project) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let auto_save_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TiffinyStudio")
            .join("auto_saves");

        std::fs::create_dir_all(&auto_save_dir)?;

        let auto_save_filename = format!("{}_autosave.{}", project.id, "tiffiny");
        Ok(auto_save_dir.join(auto_save_filename))
    }

    pub async fn restore_backup(&self, backup_path: &Path) -> Result<super::Project, Box<dyn std::error::Error>> {
        let backup_data = tokio::fs::read(backup_path).await?;
        
        let project: super::Project = match backup_path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::from_slice(&backup_data)?,
            Some("toml") => toml::from_slice(&backup_data)?,
            Some("bin") => bincode::deserialize(&backup_data)?,
            _ => return Err("Unknown backup format".into()),
        };

        Ok(project)
    }

    pub async fn get_events(&mut self) -> Vec<SaverEvent> {
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

    pub fn get_settings(&self) -> SaverSettings {
        self.settings.read().clone()
    }

    pub fn set_settings(&self, settings: SaverSettings) {
        let mut settings_ref = self.settings.write();
        *settings_ref = settings;
    }

    pub fn get_active_saves(&self) -> HashMap<String, SaveOperation> {
        self.active_saves.read().clone()
    }

    pub fn get_save_operation(&self, save_id: &str) -> Option<SaveOperation> {
        let active_saves = self.active_saves.read();
        active_saves.get(save_id).cloned()
    }

    pub fn cancel_save(&self, save_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_saves = self.active_saves.write();
        
        if let Some(operation) = active_saves.get_mut(save_id) {
            let mut status = operation.status.write();
            *status = SaveStatus::Cancelled;
            
            if operation.save_path.exists() {
                tokio::fs::remove_file(&operation.save_path).await?;
            }
            
            Ok(())
        } else {
            Err(format!("Save operation {} not found", save_id).into())
        }
    }

    pub fn get_save_progress(&self, save_id: &str) -> Option<SaveProgress> {
        let active_saves = self.active_saves.read();
        
        if let Some(operation) = active_saves.get(save_id) {
            Some(operation.progress.read().clone())
        } else {
            None
        }
    }

    pub fn get_save_status(&self, save_id: &str) -> Option<SaveStatus> {
        let active_saves = self.active_saves.read();
        
        if let Some(operation) = active_saves.get(save_id) {
            Some(operation.status.read().clone())
        } else {
            None
        }
    }

    pub async fn cleanup_old_backups(&self, project: &super::Project, config: &BackupConfig) -> Result<usize, Box<dyn std::error::Error>> {
        let mut removed_count = 0;
        
        if config.backup_path.exists() {
            let mut entries = tokio::fs::read_dir(&config.backup_path).await?;
            
            let mut backup_files = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.contains(&project.id) && file_name.ends_with(".tiffiny") {
                    let metadata = entry.metadata().await?;
                    let modified_time = metadata.modified().ok_or_else(|_| std::time::SystemTime::now())?;
                    backup_files.push((entry.path(), modified_time));
                }
            }

            backup_files.sort_by(|a, b| b.1.cmp(&a.1));

            let keep_count = config.backup_count as usize;
            if backup_files.len() > keep_count {
                for (path, _) in backup_files.iter().skip(keep_count) {
                    tokio::fs::remove_file(path).await?;
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    pub fn estimate_save_time(&self, project: &super::Project) -> std::time::Duration {
        let base_time_ms = 100;
        let asset_count = project.get_asset_count();
        let total_size = project.get_total_size();
        
        let size_factor = (total_size / (1024 * 1024)).max(1) as f64;
        let asset_factor = asset_count as f64;
        
        let estimated_time_ms = base_time_ms as f64 * (1.0 + size_factor * 0.1 + asset_factor * 0.05);
        
        std::time::Duration::from_millis(estimated_time_ms as u64)
    }

    pub fn get_save_info(&self, project: &super::Project) -> SaveInfo {
        let settings = self.settings.read();
        
        SaveInfo {
            project_id: project.id.clone(),
            project_name: project.name.clone(),
            estimated_size: self.estimate_save_size(project),
            estimated_time: self.estimate_save_time(project),
            save_format: settings.save_format.clone(),
            compression_enabled: settings.compression_enabled,
            compression_level: settings.compression_level,
            encryption_enabled: settings.encryption_enabled,
            backup_enabled: settings.backup_enabled,
            auto_save_enabled: settings.auto_save,
            auto_save_interval: settings.auto_save_interval,
        }
    }

    fn estimate_save_size(&self, project: &super::Project) -> u64 {
        let base_size = 1024;
        let asset_size = project.get_total_size();
        let metadata_size = project.get_asset_count() as u64 * 256;
        
        base_size + asset_size + metadata_size
    }

    pub fn validate_save_settings(&self, settings: &SaverSettings) -> Vec<String> {
        let mut errors = Vec::new();

        if settings.compression_level > 9 {
            errors.push("Compression level must be between 0 and 9".to_string());
        }

        if settings.backup_count == 0 {
            errors.push("Backup count must be at least 1".to_string());
        }

        if settings.auto_save_interval.as_secs() < 60 {
            errors.push("Auto-save interval must be at least 60 seconds".to_string());
        }

        if settings.encryption_enabled && settings.encryption_key.is_none() {
            errors.push("Encryption key is required when encryption is enabled".to_string());
        }

        errors
    }

    pub fn create_save_template(&self, template_name: &str) -> SaveTemplate {
        SaveTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: template_name.to_string(),
            settings: match template_name {
                "minimal" => SaverSettings {
                    auto_save: false,
                    auto_save_interval: std::time::Duration::from_secs(300),
                    backup_enabled: false,
                    backup_count: 1,
                    compression_enabled: false,
                    compression_level: 0,
                    encryption_enabled: false,
                    encryption_key: None,
                    save_format: SaveFormat::Json,
                    max_concurrent_saves: 1,
                },
                "standard" => SaverSettings {
                    auto_save: true,
                    auto_save_interval: std::time::Duration::from_secs(300),
                    backup_enabled: true,
                    backup_count: 5,
                    compression_enabled: true,
                    compression_level: 6,
                    encryption_enabled: false,
                    encryption_key: None,
                    save_format: SaveFormat::Json,
                    max_concurrent_saves: 2,
                },
                "secure" => SaverSettings {
                    auto_save: true,
                    auto_save_interval: std::time::Duration::from_secs(180),
                    backup_enabled: true,
                    backup_count: 10,
                    compression_enabled: true,
                    compression_level: 9,
                    encryption_enabled: true,
                    encryption_key: Some("default-encryption-key".to_string()),
                    save_format: SaveFormat::Binary,
                    max_concurrent_saves: 1,
                },
                _ => SaverSettings::default(),
            },
        }
    }

    pub fn get_save_templates(&self) -> Vec<String> {
        vec![
            "minimal".to_string(),
            "standard".to_string(),
            "secure".to_string(),
        ]
    }

    pub fn apply_save_template(&self, template_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let template = self.create_save_template(template_name);
        
        let errors = self.validate_save_settings(&template.settings);
        if !errors.is_empty() {
            return Err(format!("Invalid template settings: {}", errors.join(", ")).into());
        }

        self.set_settings(template.settings);
        Ok(())
    }

    pub async fn batch_save_projects(&self, projects: &[(super::Project, PathBuf)]) -> Vec<SaveResult> {
        let settings = self.settings.read();
        let max_concurrent = settings.max_concurrent_saves;
        
        let mut results = Vec::new();
        
        if projects.len() <= max_concurrent {
            let mut tasks = Vec::new();
            
            for (project, path) in projects {
                let project = project.clone();
                let path = path.clone();
                let task = tokio::spawn(async move {
                    self.save_project(&project, &path).await
                });
                
                tasks.push(task);
            }
            
            let task_results = futures::future::join_all(tasks).await;
            results.extend(task_results);
        } else {
            let chunks: Vec<_> = projects.chunks(max_concurrent).collect();
            
            for chunk in chunks {
                let mut chunk_results = Vec::new();
                
                for (project, path) in chunk {
                    let result = self.save_project(project, path).await;
                    chunk_results.push(result);
                }
                
                results.extend(chunk_results);
            }
        }

        results
    }

    pub fn reset(&self) {
        let active_saves = self.active_saves.read();
        for save_id in active_saves.keys() {
            let _ = self.cancel_save(save_id);
        }

        let mut settings = self.settings.write();
        *settings = SaverSettings::default();
    }
}

#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub project_id: String,
    pub project_name: String,
    pub estimated_size: u64,
    pub estimated_time: std::time::Duration,
    pub save_format: SaveFormat,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub backup_enabled: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct SaveTemplate {
    pub id: String,
    pub name: String,
    pub settings: SaverSettings,
}

impl Default for ProjectSaver {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Project Saver".to_string(),
        )
    }
}

impl Default for SaverSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval: std::time::Duration::from_secs(300),
            backup_enabled: true,
            backup_count: 5,
            compression_enabled: false,
            compression_level: 6,
            encryption_enabled: false,
            encryption_key: None,
            save_format: SaveFormat::Json,
            max_concurrent_saves: 2,
        }
    }
}

impl Default for SaveOperation {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: String::new(),
            save_path: PathBuf::new(),
            format: SaveFormat::Json,
            compression: false,
            encryption: false,
            progress: Arc::new(RwLock::new(SaveProgress::default())),
            start_time: std::time::Instant::now(),
            status: Arc::new(RwLock::new(SaveStatus::Pending)),
        }
    }
}

impl Default for SaveProgress {
    fn default() -> Self {
        Self {
            stage: SaveStage::Initializing,
            progress: 0.0,
            message: String::new(),
            bytes_saved: 0,
            total_bytes: 0,
            estimated_time_remaining: None,
        }
    }
}

impl Default for SaveResult {
    fn default() -> Self {
        Self {
            success: false,
            save_path: PathBuf::new(),
            file_size: 0,
            compression_ratio: None,
            checksum: String::new(),
            save_time: std::time::Duration::from_millis(0),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("TiffinyStudio")
                .join("backups"),
            backup_count: 5,
            compression_enabled: true,
            encryption_enabled: false,
            include_metadata: true,
            timestamp_format: "%Y%m%d_%H%M%S".to_string(),
        }
    }
}

impl Default for SaveInfo {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            project_name: String::new(),
            estimated_size: 0,
            estimated_time: std::time::Duration::from_millis(0),
            save_format: SaveFormat::Json,
            compression_enabled: false,
            compression_level: 6,
            encryption_enabled: false,
            backup_enabled: false,
            auto_save_enabled: false,
            auto_save_interval: std::time::Duration::from_secs(300),
        }
    }
}

impl Default for SaveTemplate {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Template".to_string(),
            settings: SaverSettings::default(),
        }
    }
}
