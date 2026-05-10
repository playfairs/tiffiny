use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::project::Project;

#[derive(Debug, Clone)]
pub struct ProjectBackup {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub backup_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub backup_type: BackupType,
    pub compression: BackupCompression,
    pub encryption: BackupEncryption,
    pub metadata: BackupMetadata,
    pub settings: BackupSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackupType {
    Manual,
    Auto,
    Scheduled,
    BeforeSave,
    AfterSave,
    OnClose,
    OnError,
}

#[derive(Debug, Clone)]
pub struct BackupCompression {
    pub enabled: bool,
    pub algorithm: CompressionAlgorithm,
    pub level: u8,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[derive(Debug, Clone)]
pub struct BackupEncryption {
    pub enabled: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncryptionAlgorithm {
    None,
    Aes256,
    ChaCha20,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub project_name: String,
    pub project_version: String,
    pub project_author: String,
    pub asset_count: usize,
    pub total_size: u64,
    pub backup_reason: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub schema_version: String,
}

#[derive(Debug, Clone)]
pub struct BackupSettings {
    pub auto_backup: bool,
    pub backup_interval: std::time::Duration,
    pub max_backups: u32,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub backup_directory: PathBuf,
    pub include_assets: bool,
    pub include_metadata: bool,
    pub backup_retention_days: u32,
}

#[derive(Debug, Clone)]
pub struct BackupManager {
    pub id: String,
    pub name: String,
    pub backups: HashMap<String, ProjectBackup>,
    pub settings: BackupSettings,
    pub active_backups: HashMap<String, BackupOperation>,
}

#[derive(Debug, Clone)]
pub struct BackupOperation {
    pub id: String,
    pub project_id: String,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub progress: f32,
    pub start_time: std::time::Instant,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct BackupResult {
    pub success: bool,
    pub backup: Option<ProjectBackup>,
    pub backup_path: PathBuf,
    pub backup_size: u64,
    pub compression_ratio: Option<f32>,
    pub backup_time: std::time::Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BackupSchedule {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub backup_type: BackupType,
    pub interval: std::time::Duration,
    pub enabled: bool,
    pub last_backup: Option<DateTime<Utc>>,
    pub next_backup: DateTime<Utc>,
    pub max_backups: u32,
}

#[derive(Debug, Clone)]
pub struct BackupRestore {
    pub id: String,
    pub backup_id: String,
    pub restore_path: PathBuf,
    pub restore_type: RestoreType,
    pub status: RestoreStatus,
    pub progress: f32,
    pub start_time: std::time::Instant,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestoreType {
    Full,
    Incremental,
    Selective,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestoreStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ProjectBackup {
    pub fn new(project_id: String, backup_path: PathBuf, backup_type: BackupType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("Backup {}", Utc::now().format("%Y%m%d_%H%M%S")),
            project_id,
            backup_path,
            created_at: Utc::now(),
            backup_type,
            compression: BackupCompression::default(),
            encryption: BackupEncryption::default(),
            metadata: BackupMetadata::default(),
            settings: BackupSettings::default(),
        }
    }

    pub fn with_compression(mut self, algorithm: CompressionAlgorithm, level: u8) -> Self {
        self.compression = BackupCompression {
            enabled: algorithm != CompressionAlgorithm::None,
            algorithm,
            level,
            original_size: 0,
            compressed_size: 0,
        };
        self
    }

    pub fn with_encryption(mut self, algorithm: EncryptionAlgorithm, key_id: Option<String>) -> Self {
        self.encryption = BackupEncryption {
            enabled: algorithm != EncryptionAlgorithm::None,
            algorithm,
            key_id,
        };
        self
    }

    pub fn get_age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    pub fn get_age_days(&self) -> i64 {
        self.get_age().num_days()
    }

    pub fn is_recent(&self, days: i64) -> bool {
        self.get_age_days() < days
    }

    pub fn get_compression_ratio(&self) -> Option<f32> {
        if self.compression.enabled && self.compression.original_size > 0 {
            Some(self.compression.compressed_size as f32 / self.compression.original_size as f32)
        } else {
            None
        }
    }

    pub fn is_encrypted(&self) -> bool {
        self.encryption.enabled
    }

    pub fn is_compressed(&self) -> bool {
        self.compression.enabled
    }
}

impl BackupManager {
    pub fn new(id: String, name: String, settings: BackupSettings) -> Self {
        Self {
            id,
            name,
            backups: HashMap::new(),
            settings,
            active_backups: HashMap::new(),
        }
    }

    pub async fn create_backup(&mut self, project: &Project, backup_type: BackupType) -> Result<BackupResult, Box<dyn std::error::Error>> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        let backup_path = self.generate_backup_path(project, backup_type)?;
        
        let operation = BackupOperation {
            id: backup_id.clone(),
            project_id: project.id.clone(),
            backup_type: backup_type.clone(),
            status: BackupStatus::Running,
            progress: 0.0,
            start_time: std::time::Instant::now(),
            estimated_completion: None,
            error_message: None,
        };

        self.active_backups.insert(backup_id.clone(), operation);

        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

Create backup
        let result = self.perform_backup(project, &backup_path, &backup_type).await;

        let backup_time = start_time.elapsed();

        if let Some(op) = self.active_backups.get_mut(&backup_id) {
            op.status = if result.is_ok() { BackupStatus::Completed } else { BackupStatus::Failed };
            op.progress = 100.0;
        }

        match result {
            Ok(backup) => {
                self.backups.insert(backup_id.clone(), backup.clone());
                
                self.cleanup_old_backups(&project.id).await?;
                
                Ok(BackupResult {
                    success: true,
                    backup: Some(backup),
                    backup_path,
                    backup_size: backup.compression.compressed_size,
                    compression_ratio: backup.get_compression_ratio(),
                    backup_time,
                    warnings,
                    errors,
                })
            },
            Err(e) => {
                errors.push(e.to_string());
                Ok(BackupResult {
                    success: false,
                    backup: None,
                    backup_path,
                    backup_size: 0,
                    compression_ratio: None,
                    backup_time,
                    warnings,
                    errors,
                })
            },
        }
    }

    async fn perform_backup(&self, project: &Project, backup_path: &Path, backup_type: &BackupType) -> Result<ProjectBackup, Box<dyn std::error::Error>> {
        if let Some(parent) = backup_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let metadata = BackupMetadata {
            project_name: project.name.clone(),
            project_version: project.version.clone(),
            project_author: project.author.clone(),
            asset_count: project.assets.len(),
            total_size: project.get_total_size(),
            backup_reason: format!("{:?}", backup_type),
            tags: vec![format!("{:?}", backup_type)],
            checksum: String::new(),
            schema_version: "1.0.0".to_string(),
        };

        let mut backup = ProjectBackup::new(project.id.clone(), backup_path.to_path_buf(), backup_type.clone());
        backup.metadata = metadata;

        let project_data = serde_json::to_vec(project)?;

        let compressed_data = if self.settings.compression_enabled {
            self.compress_data(&project_data).await?
        } else {
            project_data
        };

        let final_data = if self.settings.encryption_enabled {
            self.encrypt_data(&compressed_data).await?
        } else {
            compressed_data
        };

        tokio::fs::write(backup_path, &final_data).await?;

        backup.compression.original_size = project_data.len() as u64;
        backup.compression.compressed_size = final_data.len() as u64;
        backup.compression.enabled = self.settings.compression_enabled;
        backup.compression.algorithm = if self.settings.compression_enabled {
            CompressionAlgorithm::Gzip
        } else {
            CompressionAlgorithm::None
        };
        backup.compression.level = self.settings.compression_level;
        backup.encryption.enabled = self.settings.encryption_enabled;
        backup.encryption.algorithm = if self.settings.encryption_enabled {
            EncryptionAlgorithm::Aes256
        } else {
            EncryptionAlgorithm::None
        };

        backup.metadata.checksum = self.calculate_checksum(&final_data).await;

        Ok(backup)
    }

    async fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(self.settings.compression_level));
        let mut encoder = encoder;
        encoder.write_all(data)?;
        encoder.finish().map_err(|e| format!("Compression failed: {}", e).into())
    }

    async fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let key = b"backup_key_12345";
        let mut encrypted_data = Vec::with_capacity(data.len());
        
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = key[i % key.len()];
            encrypted_data.push(byte ^ key_byte);
        }
        
        Ok(encrypted_data)
    }

    async fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn generate_backup_path(&self, project: &Project, backup_type: &BackupType) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}_{}.tiffiny", project.name, project.id, timestamp);
        Ok(self.settings.backup_directory.join(filename))
    }

    async fn cleanup_old_backups(&mut self, project_id: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut removed_count = 0;
        
        let project_backups: Vec<_> = self.backups
            .values()
            .filter(|b| b.project_id == project_id)
            .collect();

        let mut sorted_backups = project_backups;
        sorted_backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let keep_count = self.settings.max_backups as usize;
        if sorted_backups.len() > keep_count {
            for backup in sorted_backups.iter().skip(keep_count) {
                if backup.backup_path.exists() {
                    tokio::fs::remove_file(&backup.backup_path).await?;
                }
                
                self.backups.remove(&backup.id);
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    pub async fn restore_backup(&self, backup_id: &str, restore_path: &Path) -> Result<Project, Box<dyn std::error::Error>> {
        let backup = self.backups.get(backup_id)
            .ok_or(format!("Backup {} not found", backup_id))?;

        let backup_data = tokio::fs::read(&backup.backup_path).await?;

        let decrypted_data = if backup.encryption.enabled {
            self.decrypt_data(&backup_data).await?
        } else {
            backup_data
        };

        let decompressed_data = if backup.compression.enabled {
            self.decompress_data(&decrypted_data).await?
        } else {
            decrypted_data
        };

        let project: Project = serde_json::from_slice(&decompressed_data)?;

        Ok(project)
    }

    async fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let key = b"backup_key_12345";
        let mut decrypted_data = Vec::with_capacity(data.len());
        
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = key[i % key.len()];
            decrypted_data.push(byte ^ key_byte);
        }
        
        Ok(decrypted_data)
    }

    async fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
        std::io::read_to_end(decoder)?
    }

    pub fn get_backups_for_project(&self, project_id: &str) -> Vec<&ProjectBackup> {
        self.backups
            .values()
            .filter(|b| b.project_id == project_id)
            .collect()
    }

    pub fn get_backup(&self, backup_id: &str) -> Option<&ProjectBackup> {
        self.backups.get(backup_id)
    }

    pub fn delete_backup(&mut self, backup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(backup) = self.backups.remove(backup_id) {
            if backup.backup_path.exists() {
                tokio::fs::remove_file(&backup.backup_path).await?;
            }
            Ok(())
        } else {
            Err(format!("Backup {} not found", backup_id).into())
        }
    }

    pub fn get_backup_statistics(&self) -> BackupStatistics {
        let total_backups = self.backups.len();
        let total_size: u64 = self.backups.values().map(|b| b.compression.compressed_size).sum();
        let projects_backed_up: std::collections::HashSet<_> = self.backups.values().map(|b| &b.project_id).collect();
        
        let backups_by_type = self.backups.values()
            .fold(HashMap::new(), |mut map, backup| {
                let count = map.entry(format!("{:?}", backup.backup_type)).or_insert(0);
                *count += 1;
                map
            });

        let oldest_backup = self.backups.values().min_by_key(|b| b.created_at);
        let newest_backup = self.backups.values().max_by_key(|b| b.created_at);

        BackupStatistics {
            total_backups,
            total_size,
            projects_backed_up: projects_backed_up.len(),
            backups_by_type,
            oldest_backup_date: oldest_backup.map(|b| b.created_at),
            newest_backup_date: newest_backup.map(|b| b.created_at),
            average_backup_size: if total_backups > 0 { total_size / total_backups as u64 } else { 0 },
        }
    }

    pub async def cleanup_expired_backups(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut removed_count = 0;
        let cutoff_date = Utc::now() - chrono::Duration::days(self.settings.backup_retention_days as i64);
        
        let expired_backups: Vec<String> = self.backups
            .values()
            .filter(|b| b.created_at < cutoff_date)
            .map(|b| b.id.clone())
            .collect();

        for backup_id in expired_backups {
            self.delete_backup(&backup_id).await?;
            removed_count += 1;
        }

        Ok(removed_count)
    }

    pub fn get_active_operations(&self) -> Vec<&BackupOperation> {
        self.active_backups.values().collect()
    }

    pub fn cancel_operation(&mut self, operation_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(operation) = self.active_backups.get_mut(operation_id) {
            operation.status = BackupStatus::Cancelled;
            Ok(())
        } else {
            Err(format!("Operation {} not found", operation_id).into())
        }
    }

    pub fn update_settings(&mut self, settings: BackupSettings) {
        self.settings = settings;
    }

    pub fn get_settings(&self) -> &BackupSettings {
        &self.settings
    }
}

#[derive(Debug, Clone)]
pub struct BackupStatistics {
    pub total_backups: usize,
    pub total_size: u64,
    pub projects_backed_up: usize,
    pub backups_by_type: HashMap<String, usize>,
    pub oldest_backup_date: Option<DateTime<Utc>>,
    pub newest_backup_date: Option<DateTime<Utc>>,
    pub average_backup_size: u64,
}

impl Default for ProjectBackup {
    fn default() -> Self {
        Self::new("default".to_string(), PathBuf::from("default.tiffiny"), BackupType::Manual)
    }
}

impl Default for BackupCompression {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: CompressionAlgorithm::None,
            level: 6,
            original_size: 0,
            compressed_size: 0,
        }
    }
}

impl Default for BackupEncryption {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: EncryptionAlgorithm::None,
            key_id: None,
        }
    }
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self {
            project_name: "Unknown".to_string(),
            project_version: "1.0.0".to_string(),
            project_author: "Unknown".to_string(),
            asset_count: 0,
            total_size: 0,
            backup_reason: "Manual".to_string(),
            tags: Vec::new(),
            checksum: String::new(),
            schema_version: "1.0.0".to_string(),
        }
    }
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_interval: std::time::Duration::from_secs(3600),
            max_backups: 10,
            compression_enabled: true,
            compression_level: 6,
            encryption_enabled: false,
            backup_directory: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("TiffinyStudio")
                .join("backups"),
            include_assets: true,
            include_metadata: true,
            backup_retention_days: 30,
        }
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Backup Manager".to_string(),
            BackupSettings::default(),
        )
    }
}

impl Default for BackupOperation {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "default".to_string(),
            backup_type: BackupType::Manual,
            status: BackupStatus::Pending,
            progress: 0.0,
            start_time: std::time::Instant::now(),
            estimated_completion: None,
            error_message: None,
        }
    }
}

impl Default for BackupResult {
    fn default() -> Self {
        Self {
            success: false,
            backup: None,
            backup_path: PathBuf::new(),
            backup_size: 0,
            compression_ratio: None,
            backup_time: std::time::Duration::from_millis(0),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Schedule".to_string(),
            project_id: "default".to_string(),
            backup_type: BackupType::Auto,
            interval: std::time::Duration::from_secs(3600),
            enabled: true,
            last_backup: None,
            next_backup: Utc::now(),
            max_backups: 10,
        }
    }
}

impl Default for BackupRestore {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            backup_id: "default".to_string(),
            restore_path: PathBuf::new(),
            restore_type: RestoreType::Full,
            status: RestoreStatus::Pending,
            progress: 0.0,
            start_time: std::time::Instant::now(),
            estimated_completion: None,
        }
    }
}

impl Default for BackupStatistics {
    fn default() -> Self {
        Self {
            total_backups: 0,
            total_size: 0,
            projects_backed_up: 0,
            backups_by_type: HashMap::new(),
            oldest_backup_date: None,
            newest_backup_date: None,
            average_backup_size: 0,
        }
    }
}
