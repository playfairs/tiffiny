use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProjectLoader {
    pub id: String,
    pub name: String,
    pub event_sender: mpsc::UnboundedSender<LoaderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<LoaderEvent>>>>,
    pub settings: Arc<RwLock<LoaderSettings>>,
    pub cache: Arc<RwLock<ProjectCache>>,
}

#[derive(Debug, Clone)]
pub enum LoaderEvent {
    LoadingStarted(String),
    LoadingProgress(String, f32),
    LoadingCompleted(String, super::Project),
    LoadingFailed(String, String),
    CacheHit(String),
    CacheMiss(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct LoaderSettings {
    pub enable_cache: bool,
    pub cache_size: usize,
    pub cache_ttl: std::time::Duration,
    pub parallel_loading: bool,
    pub max_concurrent_loads: usize,
    pub validate_on_load: bool,
    pub auto_backup: bool,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ProjectCache {
    pub entries: HashMap<String, CacheEntry>,
    pub max_size: usize,
    pub current_size: usize,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub project: super::Project,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub file_path: PathBuf,
    pub file_hash: String,
    pub compressed: bool,
    pub encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub project_id: String,
    pub stage: LoadingStage,
    pub progress: f32,
    pub message: String,
    pub bytes_loaded: u64,
    pub total_bytes: u64,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStage {
    Initializing,
    ReadingFile,
    ParsingMetadata,
    LoadingAssets,
    Validating,
    Finalizing,
    Completed,
    Error,
}

#[derive(Debug, Clone)]
pub struct LoadingResult {
    pub success: bool,
    pub project: Option<super::Project>,
    pub loading_time: std::time::Duration,
    pub cache_hit: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ProjectLoader {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            settings: Arc::new(RwLock::new(LoaderSettings::default())),
            cache: Arc::new(RwLock::new(ProjectCache::default())),
        }
    }

    pub async fn load_project(&self, path: &Path) -> Result<LoadingResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let project_id = self.extract_project_id(path)?;
        
        let _ = self.event_sender.send(LoaderEvent::LoadingStarted(project_id.clone()));
        
Check cache first
        if self.is_cache_enabled() {
            if let Some(cached_project) = self.check_cache(&project_id, path).await {
                let _ = self.event_sender.send(LoaderEvent::CacheHit(project_id.clone()));
                return Ok(LoadingResult {
                    success: true,
                    project: Some(cached_project),
                    loading_time: start_time.elapsed(),
                    cache_hit: true,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                });
            } else {
                let _ = self.event_sender.send(LoaderEvent::CacheMiss(project_id.clone()));
            }
        }

        let result = self.load_from_disk(&project_id, path).await;
        
        let loading_time = start_time.elapsed();
        
        match result {
            Ok(project) => {
                if self.is_cache_enabled() {
                    self.cache_project(&project_id, &project, path).await;
                }
                
                let _ = self.event_sender.send(LoaderEvent::LoadingCompleted(project_id.clone(), project.clone()));
                
                Ok(LoadingResult {
                    success: true,
                    project: Some(project),
                    loading_time,
                    cache_hit: false,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                })
            },
            Err(e) => {
                let error_msg = format!("Failed to load project: {}", e);
                let _ = self.event_sender.send(LoaderEvent::LoadingFailed(project_id.clone(), error_msg.clone()));
                
                Ok(LoadingResult {
                    success: false,
                    project: None,
                    loading_time,
                    cache_hit: false,
                    warnings: Vec::new(),
                    errors: vec![error_msg],
                })
            },
        }
    }

    async fn load_from_disk(&self, project_id: &str, path: &Path) -> Result<super::Project, Box<dyn std::error::Error>> {
        self.update_progress(project_id, LoadingStage::ReadingFile, 0.0, "Reading project file").await;
        
        let file_content = tokio::fs::read(path).await?;
        let total_size = file_content.len();
        
        self.update_progress(project_id, LoadingStage::ParsingMetadata, 20.0, "Parsing project metadata").await;
        
        let project = self.parse_project_file(&file_content, project_id)?;
        
        self.update_progress(project_id, LoadingStage::LoadingAssets, 40.0, "Loading project assets").await;
        
        let loaded_project = self.load_project_assets(project, project_id, total_size).await?;
        
        self.update_progress(project_id, LoadingStage::Validating, 80.0, "Validating project").await;
        
        if self.should_validate() {
            self.validate_project(&loaded_project, project_id).await?;
        }
        
        self.update_progress(project_id, LoadingStage::Finalizing, 95.0, "Finalizing project").await;
        
        self.update_progress(project_id, LoadingStage::Completed, 100.0, "Project loaded successfully").await;
        
        Ok(loaded_project)
    }

    fn parse_project_file(&self, content: &[u8], project_id: &str) -> Result<super::Project, Box<dyn std::error::Error>> {
        let content_str = String::from_utf8(content)?;
        
        if content_str.trim_start().starts_with('{') {
            self.parse_json_project(&content_str, project_id)
        } else if content_str.trim_start().starts_with('#') {
            self.parse_toml_project(&content_str, project_id)
        } else {
            self.parse_binary_project(content, project_id)
        }
    }

    fn parse_json_project(&self, content: &str, project_id: &str) -> Result<super::Project, Box<dyn std::error::Error>> {
        let json_value: serde_json::Value = serde_json::from_str(content)?;
        
        let project = super::Project {
            id: project_id.to_string(),
            name: json_value["name"].as_str().unwrap_or("Unknown").to_string(),
            description: json_value["description"].as_str().unwrap_or("").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(json_value["created_at"].as_str().unwrap_or(""))
                .unwrap_or_else(|_| chrono::Utc::now()),
            modified_at: chrono::DateTime::parse_from_rfc3339(json_value["modified_at"].as_str().unwrap_or(""))
                .unwrap_or_else(|_| chrono::Utc::now()),
            version: json_value["version"].as_str().unwrap_or("1.0.0").to_string(),
            author: json_value["author"].as_str().unwrap_or("Unknown").to_string(),
            tags: json_value["tags"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            settings: serde_json::from_value(json_value["settings"].clone()).unwrap_or_default(),
            assets: self.parse_assets_from_json(&json_value["assets"])?,
            metadata: serde_json::from_value(json_value["metadata"].clone()).unwrap_or_default(),
            state: Arc::new(RwLock::new(super::ProjectState::Loaded))),
        };
        
        Ok(project)
    }

    fn parse_toml_project(&self, content: &str, project_id: &str) -> Result<super::Project, Box<dyn std::error::Error>> {
        let toml_value: toml::Value = toml::from_str(content)?;
        
        let project = super::Project {
            id: project_id.to_string(),
            name: toml_value["name"].as_str().unwrap_or("Unknown").to_string(),
            description: toml_value["description"].as_str().unwrap_or("").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(toml_value["created_at"].as_str().unwrap_or(""))
                .unwrap_or_else(|_| chrono::Utc::now()),
            modified_at: chrono::DateTime::parse_from_rfc3339(toml_value["modified_at"].as_str().unwrap_or(""))
                .unwrap_or_else(|_| chrono::Utc::now()),
            version: toml_value["version"].as_str().unwrap_or("1.0.0").to_string(),
            author: toml_value["author"].as_str().unwrap_or("Unknown").to_string(),
            tags: toml_value["tags"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            settings: toml::from_str(&toml_value["settings"].to_string()).unwrap_or_default(),
            assets: self.parse_assets_from_toml(&toml_value["assets"])?,
            metadata: toml::from_str(&toml_value["metadata"].to_string()).unwrap_or_default(),
            state: Arc::new(RwLock::new(super::ProjectState::Loaded))),
        };
        
        Ok(project)
    }

    fn parse_binary_project(&self, content: &[u8], project_id: &str) -> Result<super::Project, Box<dyn std::error::Error>> {
        let project: super::Project = bincode::deserialize(content)?;
        
        let mut project = project;
        project.id = project_id.to_string();
        
        Ok(project)
    }

    fn parse_assets_from_json(&self, assets_value: &serde_json::Value) -> Result<std::collections::HashMap<String, super::Asset>, Box<dyn std::error::Error>> {
        let mut assets = std::collections::HashMap::new();
        
        if let Some(assets_array) = assets_value.as_array() {
            for asset_value in assets_array {
                if let Some(asset_obj) = asset_value.as_object() {
                    let asset = super::Asset {
                        id: asset_obj.get("id").and_then(|v| v.as_str()).unwrap_or(&uuid::Uuid::new_v4().to_string()).to_string(),
                        name: asset_obj.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        asset_type: self.parse_asset_type(asset_obj.get("asset_type").and_then(|v| v.as_str()).unwrap_or("Binary"))),
                        path: PathBuf::from(asset_obj.get("path").and_then(|v| v.as_str()).unwrap_or("")),
                        size: asset_obj.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                        created_at: chrono::DateTime::parse_from_rfc3339(asset_obj.get("created_at").and_then(|v| v.as_str()).unwrap_or(""))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        modified_at: chrono::DateTime::parse_from_rfc3339(asset_obj.get("modified_at").and_then(|v| v.as_str()).unwrap_or(""))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        metadata: self.parse_asset_metadata_from_json(asset_obj.get("metadata").unwrap_or(&serde_json::Value::Null))),
                    };
                    
                    assets.insert(asset.id.clone(), asset);
                }
            }
        }
        
        Ok(assets)
    }

    fn parse_assets_from_toml(&self, assets_value: &toml::Value) -> Result<std::collections::HashMap<String, super::Asset>, Box<dyn std::error::Error>> {
        let mut assets = std::collections::HashMap::new();
        
        if let Some(assets_table) = assets_value.as_table() {
            for (asset_id, asset_value) in assets_table {
                if let Some(asset_table) = asset_value.as_table() {
                    let asset = super::Asset {
                        id: asset_id.clone(),
                        name: asset_table.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        asset_type: self.parse_asset_type(asset_table.get("asset_type").and_then(|v| v.as_str()).unwrap_or("Binary")),
                        path: PathBuf::from(asset_table.get("path").and_then(|v| v.as_str()).unwrap_or("")),
                        size: asset_table.get("size").and_then(|v| v.as_integer()).unwrap_or(0) as u64,
                        created_at: chrono::DateTime::parse_from_rfc3339(asset_table.get("created_at").and_then(|v| v.as_str()).unwrap_or(""))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        modified_at: chrono::DateTime::parse_from_rfc3339(asset_table.get("modified_at").and_then(|v| v.as_str()).unwrap_or(""))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        metadata: self.parse_asset_metadata_from_toml(asset_table.get("metadata").unwrap_or(&toml::Value::Table(toml::value::Table::new()))),
                    };
                    
                    assets.insert(asset_id.clone(), asset);
                }
            }
        }
        
        Ok(assets)
    }

    fn parse_asset_type(&self, type_str: &str) -> super::AssetType {
        match type_str {
            "Image" => super::AssetType::Image,
            "Video" => super::AssetType::Video,
            "Audio" => super::AssetType::Audio,
            "Text" => super::AssetType::Text,
            "Project" => super::AssetType::Project,
            _ => super::AssetType::Custom(type_str.to_string()),
        }
    }

    fn parse_asset_metadata_from_json(&self, metadata_value: &serde_json::Value) -> super::AssetMetadata {
        super::AssetMetadata {
            format: metadata_value["format"].as_str().unwrap_or("unknown").to_string(),
            dimensions: metadata_value["dimensions"].as_array().and_then(|arr| {
                if arr.len() >= 2 {
                    Some((arr[0].as_u64().unwrap_or(0) as u32, arr[1].as_u64().unwrap_or(0) as u32))
                } else {
                    None
                }
            }),
            duration: metadata_value["duration"].as_u64().map(|d| std::time::Duration::from_secs(d)),
            sample_rate: metadata_value["sample_rate"].as_u64().map(|s| s as u32),
            bit_depth: metadata_value["bit_depth"].as_u64().map(|b| b as u8),
            channels: metadata_value["channels"].as_u64().map(|c| c as u8),
            color_space: metadata_value["color_space"].as_str().map(|s| s.to_string()),
            additional: metadata_value["additional"].as_object()
                .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                .unwrap_or_default(),
        }
    }

    fn parse_asset_metadata_from_toml(&self, metadata_value: &toml::Value) -> super::AssetMetadata {
        super::AssetMetadata {
            format: metadata_value["format"].as_str().unwrap_or("unknown").to_string(),
            dimensions: metadata_value["dimensions"].as_array().and_then(|arr| {
                if arr.len() >= 2 {
                    Some((arr[0].as_integer().unwrap_or(0) as u32, arr[1].as_integer().unwrap_or(0) as u32))
                } else {
                    None
                }
            }),
            duration: metadata_value["duration"].as_integer().map(|d| std::time::Duration::from_secs(d as u64)),
            sample_rate: metadata_value["sample_rate"].as_integer().map(|s| s as u32),
            bit_depth: metadata_value["bit_depth"].as_integer().map(|b| b as u8),
            channels: metadata_value["channels"].as_integer().map(|c| c as u8),
            color_space: metadata_value["color_space"].as_str().map(|s| s.to_string()),
            additional: metadata_value["additional"].as_table()
                .map(|table| table.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                .unwrap_or_default(),
        }
    }

    async fn load_project_assets(&self, mut project: super::Project, project_id: &str, total_size: usize) -> Result<super::Project, Box<dyn std::error::Error>> {
        let mut assets_loaded = 0;
        let total_assets = project.assets.len();
        
        let settings = self.settings.read();
        let use_parallel = settings.parallel_loading && total_assets > 1;
        
        if use_parallel {
            let mut tasks = Vec::new();
            let assets: Vec<_> = project.assets.iter().collect();
            
            for (asset_id, asset) in assets {
                let project_id = project_id.to_string();
                let asset_id = asset_id.clone();
                let asset = asset.clone();
                
                let task = tokio::spawn(async move {
                    self.load_single_asset(&project_id, &asset_id, &asset).await
                });
                
                tasks.push(task);
            }
            
            let results = futures::future::join_all(tasks).await;
            
            for result in results {
                match result {
                    Ok(loaded_asset) => {
                        project.assets.insert(loaded_asset.id.clone(), loaded_asset);
                        assets_loaded += 1;
                        
                        let progress = 40.0 + (60.0 * assets_loaded as f32 / total_assets as f32);
                        self.update_progress(project_id, LoadingStage::LoadingAssets, progress, &format!("Loaded {} of {} assets", assets_loaded, total_assets)).await;
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(LoaderEvent::Error(format!("Failed to load asset: {}", e)));
                    },
                }
            }
        } else {
            for (asset_id, asset) in project.assets.iter() {
                match self.load_single_asset(project_id, asset_id, asset).await {
                    Ok(loaded_asset) => {
                        project.assets.insert(loaded_asset.id.clone(), loaded_asset);
                        assets_loaded += 1;
                        
                        let progress = 40.0 + (60.0 * assets_loaded as f32 / total_assets as f32);
                        self.update_progress(project_id, LoadingStage::LoadingAssets, progress, &format!("Loaded {} of {} assets", assets_loaded, total_assets)).await;
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(LoaderEvent::Error(format!("Failed to load asset: {}", e)));
                    },
                }
            }
        }
        
        Ok(project)
    }

    async fn load_single_asset(&self, project_id: &str, asset_id: &str, asset: &super::Asset) -> Result<super::Asset, Box<dyn std::error::Error>> {
        if !asset.path.exists() {
            return Err(format!("Asset file not found: {:?}", asset.path).into());
        }
        
        let metadata = tokio::fs::metadata(&asset.path).await?;
        let file_size = metadata.len();
        let modified_time = metadata.modified().ok_or_else(|_| std::time::SystemTime::now())?;
        let created_time = metadata.created().ok_or_else(|_| std::time::SystemTime::now())?;
        
        let loaded_asset = super::Asset {
            id: asset_id.to_string(),
            name: asset.name.clone(),
            asset_type: asset.asset_type.clone(),
            path: asset.path.clone(),
            size: file_size,
            created_at: chrono::DateTime::from(created_time),
            modified_at: chrono::DateTime::from(modified_time),
            metadata: asset.metadata.clone(),
        };
        
        Ok(loaded_asset)
    }

    async fn validate_project(&self, project: &super::Project, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let validation_errors = project.validate();
        
        if !validation_errors.is_empty() {
            return Err(format!("Project validation failed: {}", validation_errors.join(", ")).into());
        }
        
        if project.name.is_empty() {
            return Err("Project name cannot be empty".into());
        }
        
        if project.id != project_id {
            return Err("Project ID mismatch".into());
        }
        
        Ok(())
    }

    async fn update_progress(&self, project_id: &str, stage: LoadingStage, progress: f32, message: &str) {
        let _ = self.event_sender.send(LoaderEvent::LoadingProgress(
            project_id.to_string(),
            progress,
        ));
    }

    fn extract_project_id(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            
            if let Some(id_part) = file_name_str.strip_suffix(".tiffiny") {
                Ok(id_part.to_string())
            } else {
                Ok(uuid::Uuid::new_v4().to_string())
            }
        } else {
            Err("Invalid project path".into())
        }
    }

    async fn check_cache(&self, project_id: &str, path: &Path) -> Option<super::Project> {
        let cache = self.cache.read();
        
        if let Some(entry) = cache.entries.get(project_id) {
            let now = chrono::Utc::now();
            let age = now - entry.timestamp;
            let settings = self.settings.read();
            
            if age < settings.cache_ttl && entry.file_path == path {
                if let Ok(current_hash) = self.calculate_file_hash(path).await {
                    if current_hash == entry.file_hash {
                        return Some(entry.project.clone());
                    }
                }
            }
        }
        
        None
    }

    async fn cache_project(&self, project_id: &str, project: &super::Project, path: &Path) {
        let settings = self.settings.read();
        
        if !settings.enable_cache {
            return;
        }
        
        if let Ok(file_hash) = self.calculate_file_hash(path).await {
            let cache_entry = CacheEntry {
                project: project.clone(),
                timestamp: chrono::Utc::now(),
                access_count: 1,
                file_path: path.to_path_buf(),
                file_hash,
                compressed: settings.compression_enabled,
                encrypted: settings.encryption_enabled,
            };
            
            {
                let mut cache = self.cache.write();
                cache.entries.insert(project_id.to_string(), cache_entry);
                cache.current_size += 1;
                
                if cache.current_size > cache.max_size {
                    self.cleanup_cache(&mut cache);
                }
            }
        }
    }

    async fn calculate_file_hash(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = tokio::fs::read(path).await?;
        let hash = sha2::Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }

    fn cleanup_cache(&self, cache: &mut ProjectCache) {
        let mut entries: Vec<_> = cache.entries.iter().collect();
        
        entries.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
        
        while cache.current_size > cache.max_size && !entries.is_empty() {
            if let Some((oldest_id, _)) = entries.remove(0) {
                cache.entries.remove(oldest_id);
                cache.current_size -= 1;
            }
        }
        
        cache.last_cleanup = chrono::Utc::now();
    }

    fn is_cache_enabled(&self) -> bool {
        self.settings.read().enable_cache
    }

    fn should_validate(&self) -> bool {
        self.settings.read().validate_on_load
    }

    pub async fn get_events(&mut self) -> Vec<LoaderEvent> {
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

    pub fn get_settings(&self) -> LoaderSettings {
        self.settings.read().clone()
    }

    pub fn set_settings(&self, settings: LoaderSettings) {
        let mut settings_ref = self.settings.write();
        *settings_ref = settings;
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        
        CacheStats {
            entries: cache.entries.len(),
            max_size: cache.max_size,
            current_size: cache.current_size,
            last_cleanup: cache.last_cleanup,
            hit_rate: self.calculate_hit_rate(),
        }
    }

    fn calculate_hit_rate(&self) -> f32 {
        0.0
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.entries.clear();
        cache.current_size = 0;
        cache.last_cleanup = chrono::Utc::now();
    }

    pub async fn preload_projects(&self, paths: &[PathBuf]) -> Vec<LoadingResult> {
        let settings = self.settings.read();
        let max_concurrent = settings.max_concurrent_loads;
        
        let mut results = Vec::new();
        
        if settings.parallel_loading && paths.len() > 1 {
            let mut tasks = Vec::new();
            
            for path in paths {
                let path = path.clone();
                let task = tokio::spawn(async move {
                    self.load_project(&path).await
                });
                
                tasks.push(task);
            }
            
            let task_results = futures::future::join_all(tasks).await;
            results.extend(task_results);
        } else {
            for path in paths {
                match self.load_project(path).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        results.push(LoadingResult {
                            success: false,
                            project: None,
                            loading_time: std::time::Duration::from_millis(0),
                            cache_hit: false,
                            warnings: Vec::new(),
                            errors: vec![e.to_string()],
                        });
                    },
                }
            }
        }
        
        results
    }

    pub fn get_supported_formats(&self) -> Vec<String> {
        vec![
            "tiffiny".to_string(),
            "json".to_string(),
            "toml".to_string(),
            "bin".to_string(),
        ]
    }

    pub fn can_load_format(&self, format: &str) -> bool {
        self.get_supported_formats().contains(&format.to_lowercase())
    }

    pub fn estimate_loading_time(&self, path: &Path) -> std::time::Duration {
        if let Ok(metadata) = std::fs::metadata(path) {
            let file_size = metadata.len();
            let base_time_ms = 100;
            let size_factor = (file_size as f64 / 1024.0 / 1024.0).max(1.0);
            
            std::time::Duration::from_millis((base_time_ms as f64 * size_factor) as u64)
        } else {
            std::time::Duration::from_millis(100)
        }
    }

    pub fn get_loading_info(&self, path: &Path) -> LoadingInfo {
        LoadingInfo {
            path: path.to_path_buf(),
            format: self.detect_format(path),
            estimated_size: self.estimate_file_size(path),
            estimated_time: self.estimate_loading_time(path),
            supported: self.can_load_format(&self.detect_format(path)),
        }
    }

    fn detect_format(&self, path: &Path) -> String {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("tiffiny") => "tiffiny".to_string(),
                Some("json") => "json".to_string(),
                Some("toml") => "toml".to_string(),
                Some("bin") => "bin".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }

    fn estimate_file_size(&self, path: &Path) -> u64 {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    }

    pub fn reset(&self) {
        self.clear_cache();
        
        let mut settings = self.settings.write();
        *settings = LoaderSettings::default();
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_size: usize,
    pub current_size: usize,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
    pub hit_rate: f32,
}

#[derive(Debug, Clone)]
pub struct LoadingInfo {
    pub path: PathBuf,
    pub format: String,
    pub estimated_size: u64,
    pub estimated_time: std::time::Duration,
    pub supported: bool,
}

impl Default for ProjectLoader {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Project Loader".to_string(),
        )
    }
}

impl Default for LoaderSettings {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_size: 10,
            cache_ttl: std::time::Duration::from_secs(3600),
            parallel_loading: true,
            max_concurrent_loads: 4,
            validate_on_load: true,
            auto_backup: false,
            compression_enabled: false,
            encryption_enabled: false,
        }
    }
}

impl Default for ProjectCache {
    fn default() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            max_size: 10,
            current_size: 0,
            last_cleanup: chrono::Utc::now(),
        }
    }
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self {
            project: super::Project::default(),
            timestamp: chrono::Utc::now(),
            access_count: 0,
            file_path: PathBuf::new(),
            file_hash: String::new(),
            compressed: false,
            encrypted: false,
        }
    }
}

impl Default for LoadingProgress {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            stage: LoadingStage::Initializing,
            progress: 0.0,
            message: String::new(),
            bytes_loaded: 0,
            total_bytes: 0,
            estimated_time_remaining: None,
        }
    }
}

impl Default for LoadingResult {
    fn default() -> Self {
        Self {
            success: false,
            project: None,
            loading_time: std::time::Duration::from_millis(0),
            cache_hit: false,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            entries: 0,
            max_size: 10,
            current_size: 0,
            last_cleanup: chrono::Utc::now(),
            hit_rate: 0.0,
        }
    }
}

impl Default for LoadingInfo {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            format: "unknown".to_string(),
            estimated_size: 0,
            estimated_time: std::time::Duration::from_millis(0),
            supported: false,
        }
    }
}
