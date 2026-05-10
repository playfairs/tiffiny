use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub settings: ProjectSettings,
    pub assets: HashMap<String, Asset>,
    pub metadata: ProjectMetadata,
    pub state: Arc<RwLock<ProjectState>>,
}

#[derive(Debug, Clone)]
pub struct ProjectSettings {
    pub auto_save: bool,
    pub auto_save_interval: std::time::Duration,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub compression_enabled: bool,
    pub compression_level: u8,
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    pub thumbnail_size: (u32, u32),
    pub preview_quality: u8,
    pub workspace_layout: WorkspaceLayout,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Text,
    Binary,
    Project,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub format: String,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<std::time::Duration>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub color_space: Option<String>,
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub genre: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub rating: Option<f32>,
    pub language: String,
    pub software: String,
    pub notes: String,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectState {
    New,
    Loading,
    Loaded,
    Modified,
    Saving,
    Error(String),
    Closed,
}

#[derive(Debug, Clone)]
pub struct WorkspaceLayout {
    pub panels: Vec<PanelInfo>,
    pub layout_type: LayoutType,
    pub saved_positions: HashMap<String, (i32, i32)>,
    pub panel_sizes: HashMap<String, (u32, u32)>,
}

#[derive(Debug, Clone)]
pub struct PanelInfo {
    pub id: String,
    pub name: String,
    pub panel_type: PanelType,
    pub is_visible: bool,
    pub is_docked: bool,
    pub position: (i32, i32),
    pub size: (u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PanelType {
    ProjectExplorer,
    AssetBrowser,
    Timeline,
    Properties,
    Preview,
    Effects,
    Export,
    Console,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutType {
    Default,
    Compact,
    Detailed,
    Custom(String),
}

impl Project {
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now();
        
        Self {
            id,
            name,
            description: String::new(),
            created_at: now,
            modified_at: now,
            version: "1.0.0".to_string(),
            author: "Tiffiny Studio User".to_string(),
            tags: Vec::new(),
            settings: ProjectSettings::default(),
            assets: HashMap::new(),
            metadata: ProjectMetadata::default(),
            state: Arc::new(RwLock::new(ProjectState::New))),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_settings(mut self, settings: ProjectSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn add_asset(&self, asset: Asset) {
        let mut assets = self.assets;
        assets.insert(asset.id.clone(), asset);
    }

    pub fn remove_asset(&self, asset_id: &str) -> Option<Asset> {
        let mut assets = self.assets;
        assets.remove(asset_id)
    }

    pub fn get_asset(&self, asset_id: &str) -> Option<&Asset> {
        self.assets.get(asset_id)
    }

    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<&Asset> {
        self.assets
            .values()
            .filter(|asset| asset.asset_type == asset_type)
            .collect()
    }

    pub fn get_asset_count(&self) -> usize {
        self.assets.len()
    }

    pub fn get_total_size(&self) -> u64 {
        self.assets.values().map(|asset| asset.size).sum()
    }

    pub fn set_state(&self, state: ProjectState) {
        let mut project_state = self.state.write();
        *project_state = state;
    }

    pub fn get_state(&self) -> ProjectState {
        self.state.read().clone()
    }

    pub fn is_modified(&self) -> bool {
        matches!(self.get_state(), ProjectState::Modified)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.get_state(), ProjectState::Loaded)
    }

    pub fn is_saving(&self) -> bool {
        matches!(self.get_state(), ProjectState::Saving)
    }

    pub fn has_error(&self) -> bool {
        matches!(self.get_state(), ProjectState::Error(_))
    }

    pub fn get_error_message(&self) -> Option<String> {
        if let ProjectState::Error(message) = self.get_state() {
            Some(message)
        } else {
            None
        }
    }

    pub fn mark_modified(&self) {
        self.set_state(ProjectState::Modified);
        self.update_modified_at();
    }

    pub fn mark_saving(&self) {
        self.set_state(ProjectState::Saving);
    }

    pub fn mark_loaded(&self) {
        self.set_state(ProjectState::Loaded);
    }

    pub fn mark_error(&self, error: String) {
        self.set_state(ProjectState::Error(error));
    }

    pub fn mark_closed(&self) {
        self.set_state(ProjectState::Closed);
    }

    fn update_modified_at(&self) {
This would need to be implemented with a mutable reference
    }

    pub fn add_tag(&self, tag: String) {
        let mut tags = self.tags;
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    pub fn remove_tag(&self, tag: &str) -> bool {
        let mut tags = self.tags;
        if let Some(pos) = tags.iter().position(|t| t == tag) {
            tags.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag)
    }

    pub fn get_tag_count(&self) -> usize {
        self.tags.len()
    }

    pub fn get_project_info(&self) -> ProjectInfo {
        ProjectInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            created_at: self.created_at,
            modified_at: self.modified_at,
            version: self.version.clone(),
            author: self.author.clone(),
            tags: self.tags.clone(),
            asset_count: self.get_asset_count(),
            total_size: self.get_total_size(),
            state: self.get_state(),
            settings: self.settings.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Project name cannot be empty".to_string());
        }

        if self.id.is_empty() {
            errors.push("Project ID cannot be empty".to_string());
        }

        if !self.version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            errors.push("Version must contain only digits and dots".to_string());
        }

        for (asset_id, asset) in &self.assets {
            if asset.name.is_empty() {
                errors.push(format!("Asset {} has empty name", asset_id));
            }
            
            if !asset.path.exists() {
                errors.push(format!("Asset {} path does not exist: {:?}", asset_id, asset.path));
            }
        }

        errors
    }

    pub fn clone_project(&self) -> Project {
        let mut cloned = Self::new(self.id.clone(), self.name.clone());
        cloned.description = self.description.clone();
        cloned.created_at = self.created_at;
        cloned.modified_at = self.modified_at;
        cloned.version = self.version.clone();
        cloned.author = self.author.clone();
        cloned.tags = self.tags.clone();
        cloned.settings = self.settings.clone();
        cloned.assets = self.assets.clone();
        cloned.metadata = self.metadata.clone();
        cloned
    }

    pub fn create_snapshot(&self) -> ProjectSnapshot {
        let now = chrono::Utc::now();
        
        ProjectSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: self.id.clone(),
            timestamp: now,
            name: format!("{} - {}", self.name, now.format("%Y-%m-%d %H:%M:%S")),
            description: format!("Snapshot of project {} at {}", self.name, now.format("%Y-%m-%d %H:%M:%S")),
            project_data: self.clone(),
            metadata: SnapshotMetadata::default(),
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: &ProjectSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        self.id = snapshot.project_id.clone();
        self.name = snapshot.project_data.name.clone();
        self.description = snapshot.project_data.description.clone();
        self.created_at = snapshot.project_data.created_at;
        self.modified_at = snapshot.project_data.modified_at;
        self.version = snapshot.project_data.version.clone();
        self.author = snapshot.project_data.author.clone();
        self.tags = snapshot.project_data.tags.clone();
        self.settings = snapshot.project_data.settings.clone();
        self.assets = snapshot.project_data.assets.clone();
        self.metadata = snapshot.project_data.metadata.clone();
        
        Ok(())
    }

    pub fn export_metadata(&self) -> ProjectExportMetadata {
        ProjectExportMetadata {
            project_info: self.get_project_info(),
            export_timestamp: chrono::Utc::now(),
            export_version: "1.0.0".to_string(),
            software_info: "Tiffiny Studio".to_string(),
            checksum: self.calculate_checksum(),
        }
    }

    fn calculate_checksum(&self) -> String {
        format!("{:x}", self.id.len() + self.name.len())
    }

    pub fn get_recent_assets(&self, limit: usize) -> Vec<&Asset> {
        let mut assets: Vec<_> = self.assets.values().collect();
        assets.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        assets.into_iter().take(limit).collect()
    }

    pub fn get_assets_by_size_range(&self, min_size: u64, max_size: u64) -> Vec<&Asset> {
        self.assets
            .values()
            .filter(|asset| asset.size >= min_size && asset.size <= max_size)
            .collect()
    }

    pub fn get_assets_by_date_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Vec<&Asset> {
        self.assets
            .values()
            .filter(|asset| asset.created_at >= start && asset.created_at <= end)
            .collect()
    }

    pub fn search_assets(&self, query: &str) -> Vec<&Asset> {
        let query_lower = query.to_lowercase();
        self.assets
            .values()
            .filter(|asset| {
                asset.name.to_lowercase().contains(&query_lower) ||
                asset.path.to_string_lossy().to_lowercase().contains(&query_lower) ||
                asset.metadata.additional.values().any(|v| v.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_statistics(&self) -> ProjectStatistics {
        let assets_by_type = self.assets
            .values()
            .fold(HashMap::new(), |mut map, asset| {
                map.entry(asset.asset_type.clone()).or_insert_with(Vec::new).push(asset);
                map
            });

        let mut total_size = 0u64;
        let mut asset_count = 0usize;
        
        for asset in self.assets.values() {
            total_size += asset.size;
            asset_count += 1;
        }

        ProjectStatistics {
            total_assets: asset_count,
            total_size,
            assets_by_type,
            average_asset_size: if asset_count > 0 { total_size / asset_count as u64 } else { 0 },
            oldest_asset: self.assets.values().min_by_key(|asset| asset.created_at).map(|(_, a)| a),
            newest_asset: self.assets.values().max_by_key(|asset| asset.created_at).map(|_, a)| a),
            project_duration: chrono::Utc::now() - self.created_at,
        }
    }

    pub fn optimize_storage(&self) -> StorageOptimizationResult {
        let mut suggestions = Vec::new();
        let mut total_savings = 0u64;

        let mut seen_paths = std::collections::HashSet::new();
        for asset in self.assets.values() {
            if seen_paths.contains(&asset.path) {
                suggestions.push(format!("Duplicate asset found: {:?}", asset.path));
                total_savings += asset.size;
            }
            seen_paths.insert(asset.path.clone());
        }

        for asset in self.assets.values() {
            if asset.size > 10 * 1024 * 1024 && asset.asset_type == AssetType::Image {
                suggestions.push(format!("Large image asset could be compressed: {:?}", asset.path));
            }
        }

        StorageOptimizationResult {
            original_size: self.get_total_size(),
            optimized_size: self.get_total_size() - total_savings,
            savings_percentage: if self.get_total_size() > 0 {
                (total_savings as f64 / self.get_total_size() as f64) * 100.0
            } else {
                0.0
            },
            suggestions,
        }
    }

    pub fn create_template(&self) -> ProjectTemplate {
        ProjectTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Template", self.name),
            description: format!("Template based on project: {}", self.name),
            project_structure: self.create_project_structure(),
            default_settings: self.settings.clone(),
            default_metadata: self.metadata.clone(),
            asset_templates: self.create_asset_templates(),
        }
    }

    fn create_project_structure(&self) -> ProjectStructure {
        ProjectStructure {
            folders: vec![
                "assets/images".to_string(),
                "assets/audio".to_string(),
                "assets/video".to_string(),
                "assets/text".to_string(),
                "exports".to_string(),
                "backups".to_string(),
            ],
            file_patterns: vec![
                "*.tiffiny".to_string(),
                "*.png".to_string(),
                "*.jpg".to_string(),
                "*.mp4".to_string(),
                "*.wav".to_string(),
            ],
            settings_files: vec![
                "project.json".to_string(),
                "settings.toml".to_string(),
            ],
        }
    }

    fn create_asset_templates(&self) -> Vec<AssetTemplate> {
        self.assets
            .values()
            .map(|asset| AssetTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: asset.name.clone(),
                asset_type: asset.asset_type.clone(),
                relative_path: asset.path.strip_prefix(Path::new("")).unwrap_or(&asset.path).to_string_lossy(),
                metadata_template: asset.metadata.clone(),
                required: true,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub asset_count: usize,
    pub total_size: u64,
    pub state: ProjectState,
    pub settings: ProjectSettings,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
    pub id: String,
    pub project_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub name: String,
    pub description: String,
    pub project_data: Project,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    pub created_by: String,
    pub software_version: String,
    pub compression_method: Option<String>,
    pub checksum: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectExportMetadata {
    pub project_info: ProjectInfo,
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
    pub export_version: String,
    pub software_info: String,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct ProjectStatistics {
    pub total_assets: usize,
    pub total_size: u64,
    pub assets_by_type: HashMap<AssetType, Vec<Asset>>,
    pub average_asset_size: u64,
    pub oldest_asset: Option<&Asset>,
    pub newest_asset: Option<&Asset>,
    pub project_duration: chrono::Duration,
}

#[derive(Debug, Clone)]
pub struct StorageOptimizationResult {
    pub original_size: u64,
    pub optimized_size: u64,
    pub savings_percentage: f64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub project_structure: ProjectStructure,
    pub default_settings: ProjectSettings,
    pub default_metadata: ProjectMetadata,
    pub asset_templates: Vec<AssetTemplate>,
}

#[derive(Debug, Clone)]
pub struct ProjectStructure {
    pub folders: Vec<String>,
    pub file_patterns: Vec<String>,
    pub settings_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AssetTemplate {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub relative_path: String,
    pub metadata_template: AssetMetadata,
    pub required: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self::new("default".to_string(), "Default Project".to_string())
    }
}

impl Default for ProjectSettings {
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
            thumbnail_size: (256, 256),
            preview_quality: 80,
            workspace_layout: WorkspaceLayout::default(),
        }
    }
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Asset".to_string(),
            asset_type: AssetType::Binary,
            path: PathBuf::from("default.bin"),
            size: 0,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            metadata: AssetMetadata::default(),
        }
    }
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            format: "unknown".to_string(),
            dimensions: None,
            duration: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            color_space: None,
            additional: HashMap::new(),
        }
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            genre: "General".to_string(),
            category: "Uncategorized".to_string(),
            keywords: Vec::new(),
            rating: None,
            language: "English".to_string(),
            software: "Tiffiny Studio".to_string(),
            notes: String::new(),
            custom_fields: HashMap::new(),
        }
    }
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self {
            panels: Vec::new(),
            layout_type: LayoutType::Default,
            saved_positions: HashMap::new(),
            panel_sizes: HashMap::new(),
        }
    }
}

impl Default for PanelInfo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Panel".to_string(),
            panel_type: PanelType::ProjectExplorer,
            is_visible: true,
            is_docked: false,
            position: (0, 0),
            size: (200, 300),
        }
    }
}

impl Default for ProjectTemplate {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Template".to_string(),
            description: "Default project template".to_string(),
            project_structure: ProjectStructure::default(),
            default_settings: ProjectSettings::default(),
            default_metadata: ProjectMetadata::default(),
            asset_templates: Vec::new(),
        }
    }
}

impl Default for ProjectStructure {
    fn default() -> Self {
        Self {
            folders: vec![
                "assets".to_string(),
                "exports".to_string(),
            ],
            file_patterns: vec![
                "*.tiffiny".to_string(),
            ],
            settings_files: vec![
                "project.json".to_string(),
            ],
        }
    }
}

impl Default for AssetTemplate {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Asset Template".to_string(),
            asset_type: AssetType::Binary,
            relative_path: "assets/default.bin".to_string(),
            metadata_template: AssetMetadata::default(),
            required: false,
        }
    }
}

impl Default for SnapshotMetadata {
    fn default() -> Self {
        Self {
            created_by: "Tiffiny Studio".to_string(),
            software_version: "1.0.0".to_string(),
            compression_method: None,
            checksum: String::new(),
            tags: Vec::new(),
        }
    }
}

impl Default for StorageOptimizationResult {
    fn default() -> Self {
        Self {
            original_size: 0,
            optimized_size: 0,
            savings_percentage: 0.0,
            suggestions: Vec::new(),
        }
    }
}
