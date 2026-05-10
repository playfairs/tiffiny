use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::project::{Project, Asset, ProjectSettings, ProjectMetadata, AssetType};

#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub default_settings: ProjectSettings,
    pub default_metadata: ProjectMetadata,
    pub asset_templates: Vec<AssetTemplate>,
    pub workspace_layout: WorkspaceLayoutTemplate,
    pub project_structure: ProjectStructureTemplate,
    pub template_settings: TemplateSettings,
}

#[derive(Debug, Clone)]
pub struct AssetTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub required: bool,
    pub default_path: String,
    pub relative_path: String,
    pub metadata_template: AssetMetadataTemplate,
    pub creation_settings: AssetCreationSettings,
}

#[derive(Debug, Clone)]
pub struct AssetMetadataTemplate {
    pub format: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub color_space: Option<String>,
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AssetCreationSettings {
    pub create_file: bool,
    pub file_size: Option<u64>,
    pub file_content: Option<String>,
    pub file_extension: String,
    pub compression: Option<String>,
    pub encoding: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceLayoutTemplate {
    pub layout_type: String,
    pub panels: Vec<PanelTemplate>,
    pub default_size: (u32, u32),
    pub resizable: bool,
    pub theme: String,
}

#[derive(Debug, Clone)]
pub struct PanelTemplate {
    pub id: String,
    pub name: String,
    pub panel_type: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub docked: bool,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ProjectStructureTemplate {
    pub directories: Vec<DirectoryTemplate>,
    pub files: Vec<FileTemplate>,
    pub naming_convention: NamingConvention,
    pub organization_rules: Vec<OrganizationRule>,
}

#[derive(Debug, Clone)]
pub struct DirectoryTemplate {
    pub name: String,
    pub path: String,
    pub description: String,
    pub required: bool,
    pub created_automatically: bool,
}

#[derive(Debug, Clone)]
pub struct FileTemplate {
    pub name: String,
    pub path: String,
    pub description: String,
    pub required: bool,
    pub created_automatically: bool,
    pub content_template: Option<String>,
    pub file_type: String,
}

#[derive(Debug, Clone)]
pub struct NamingConvention {
    pub project_name_pattern: String,
    pub asset_name_pattern: String,
    pub directory_name_pattern: String,
    pub file_name_pattern: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub struct OrganizationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: OrganizationRuleType,
    pub pattern: String,
    pub action: OrganizationAction,
    pub priority: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationRuleType {
    FileType,
    FileSize,
    FileName,
    CreationDate,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationAction {
    MoveToDirectory,
    RenameFile,
    AddTag,
    SetMetadata,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TemplateSettings {
    pub auto_create_assets: bool,
    pub auto_create_directories: bool,
    pub apply_workspace_layout: bool,
    pub apply_organization_rules: bool,
    pub validate_before_creation: bool,
    pub allow_modification: bool,
    pub version_control: bool,
    pub backup_original: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateManager {
    pub id: String,
    pub name: String,
    pub templates: HashMap<String, ProjectTemplate>,
    pub categories: HashMap<String, TemplateCategory>,
    pub settings: TemplateManagerSettings,
}

#[derive(Debug, Clone)]
pub struct TemplateCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct TemplateManagerSettings {
    pub default_template_path: PathBuf,
    pub auto_load_templates: bool,
    pub validate_templates: bool,
    pub cache_templates: bool,
    pub max_cached_templates: usize,
    pub template_update_interval: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct TemplateApplication {
    pub id: String,
    pub template_id: String,
    pub project_id: String,
    pub applied_at: DateTime<Utc>,
    pub modifications: Vec<TemplateModification>,
    pub status: ApplicationStatus,
}

#[derive(Debug, Clone)]
pub struct TemplateModification {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub modification_type: ModificationType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModificationType {
    Added,
    Modified,
    Removed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationStatus {
    Pending,
    Applying,
    Completed,
    Failed,
    Partial,
}

impl ProjectTemplate {
    pub fn new(id: String, name: String, category: String) -> Self {
        let now = Utc::now();
        
        Self {
            id,
            name,
            description: String::new(),
            category,
            version: "1.0.0".to_string(),
            author: "Tiffiny Studio".to_string(),
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            default_settings: ProjectSettings::default(),
            default_metadata: ProjectMetadata::default(),
            asset_templates: Vec::new(),
            workspace_layout: WorkspaceLayoutTemplate::default(),
            project_structure: ProjectStructureTemplate::default(),
            template_settings: TemplateSettings::default(),
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

    pub fn add_asset_template(mut self, asset_template: AssetTemplate) -> Self {
        self.asset_templates.push(asset_template);
        self
    }

    pub fn with_workspace_layout(mut self, layout: WorkspaceLayoutTemplate) -> Self {
        self.workspace_layout = layout;
        self
    }

    pub fn with_project_structure(mut self, structure: ProjectStructureTemplate) -> Self {
        self.project_structure = structure;
        self
    }

    pub fn with_template_settings(mut self, settings: TemplateSettings) -> Self {
        self.template_settings = settings;
        self
    }

    pub fn apply_to_project(&self, project_name: String) -> Result<Project, Box<dyn std::error::Error>> {
        let mut project = Project::new(
            uuid::Uuid::new_v4().to_string(),
            project_name,
        );

Apply default settings
        project.settings = self.default_settings.clone();
        
        project.metadata = self.default_metadata.clone();

        if self.template_settings.auto_create_assets {
            for asset_template in &self.asset_templates {
                if asset_template.required {
                    let asset = self.create_asset_from_template(asset_template)?;
                    project.add_asset(asset);
                }
            }
        }

        if self.template_settings.apply_workspace_layout {
            project.settings.thumbnail_size = self.workspace_layout.default_size;
        }

        Ok(project)
    }

    fn create_asset_from_template(&self, template: &AssetTemplate) -> Result<Asset, Box<dyn std::error::Error>> {
        let asset_path = PathBuf::from(&template.relative_path);
        
        if template.creation_settings.create_file {
            if let Some(parent) = asset_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            if !asset_path.exists() {
                let content = template.creation_settings.file_content.as_deref().unwrap_or("");
                std::fs::write(&asset_path, content)?;
            }
        }

        let metadata = std::fs::metadata(&asset_path).unwrap_or_else(|_| {
            std::fs::metadata(".").unwrap()
        });

        let asset = Asset {
            id: uuid::Uuid::new_v4().to_string(),
            name: template.name.clone(),
            asset_type: template.asset_type.clone(),
            path: asset_path,
            size: metadata.len(),
            created_at: metadata.created().ok_or_else(|_| std::time::SystemTime::now())?,
            modified_at: metadata.modified().ok_or_else(|_| std::time::SystemTime::now())?,
            metadata: self.create_asset_metadata_from_template(&template.metadata_template),
        };

        Ok(asset)
    }

    fn create_asset_metadata_from_template(&self, template: &AssetMetadataTemplate) -> crate::project::AssetMetadata {
        crate::project::AssetMetadata {
            format: template.format.clone().unwrap_or("unknown".to_string()),
            dimensions: template.dimensions,
            duration: template.duration.map(|d| std::time::Duration::from_secs(d)),
            sample_rate: template.sample_rate,
            bit_depth: template.bit_depth,
            channels: template.channels,
            color_space: template.color_space.clone(),
            additional: template.additional.clone(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Template name cannot be empty".to_string());
        }

        if self.category.is_empty() {
            errors.push("Template category cannot be empty".to_string());
        }

        for asset_template in &self.asset_templates {
            if asset_template.name.is_empty() {
                errors.push("Asset template name cannot be empty".to_string());
            }

            if asset_template.relative_path.is_empty() {
                errors.push("Asset template relative path cannot be empty".to_string());
            }
        }

        if self.workspace_layout.panels.is_empty() {
            errors.push("Workspace layout must have at least one panel".to_string());
        }

        for directory in &self.project_structure.directories {
            if directory.name.is_empty() {
                errors.push("Directory template name cannot be empty".to_string());
            }
        }

        errors
    }

    pub fn get_required_assets(&self) -> Vec<&AssetTemplate> {
        self.asset_templates.iter().filter(|t| t.required).collect()
    }

    pub fn get_optional_assets(&self) -> Vec<&AssetTemplate> {
        self.asset_templates.iter().filter(|t| !t.required).collect()
    }

    pub fn get_asset_count(&self) -> usize {
        self.asset_templates.len()
    }

    pub fn get_required_asset_count(&self) -> usize {
        self.asset_templates.iter().filter(|t| t.required).count()
    }

    pub fn get_optional_asset_count(&self) -> usize {
        self.asset_templates.iter().filter(|t| !t.required).count()
    }

    pub fn get_estimated_size(&self) -> u64 {
        self.asset_templates
            .iter()
            .map(|t| t.creation_settings.file_size.unwrap_or(0))
            .sum()
    }

    pub fn get_creation_time_estimate(&self) -> std::time::Duration {
        let base_time = std::time::Duration::from_millis(100);
        let asset_time = std::time::Duration::from_millis(50) * self.asset_templates.len() as u32;
        let directory_time = std::time::Duration::from_millis(10) * self.project_structure.directories.len() as u32;
        
        base_time + asset_time + directory_time
    }

    pub fn clone_template(&self, new_name: String) -> Self {
        let mut cloned = self.clone();
        cloned.id = uuid::Uuid::new_v4().to_string();
        cloned.name = new_name;
        cloned.modified_at = Utc::now();
        cloned
    }

    pub fn export_template(&self) -> Result<String, Box<dyn std::error::Error>> {
        serde_json::to_string_pretty(self)
    }

    pub fn import_template(template_json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        serde_json::from_str(template_json)
    }

    pub fn update_modified_at(&mut self) {
        self.modified_at = Utc::now();
    }
}

impl TemplateManager {
    pub fn new(id: String, name: String, settings: TemplateManagerSettings) -> Self {
        Self {
            id,
            name,
            templates: HashMap::new(),
            categories: HashMap::new(),
            settings,
        }
    }

    pub fn add_template(&mut self, template: ProjectTemplate) -> Result<(), Box<dyn std::error::Error>> {
        let errors = template.validate();
        if !errors.is_empty() {
            return Err(format!("Template validation failed: {}", errors.join(", ")).into());
        }

        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    pub fn get_template(&self, template_id: &str) -> Option<&ProjectTemplate> {
        self.templates.get(template_id)
    }

    pub fn get_templates_by_category(&self, category: &str) -> Vec<&ProjectTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn search_templates(&self, query: &str) -> Vec<&ProjectTemplate> {
        let query_lower = query.to_lowercase();
        
        self.templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower) ||
                t.description.to_lowercase().contains(&query_lower) ||
                t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) ||
                t.category.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn remove_template(&mut self, template_id: &str) -> Option<ProjectTemplate> {
        self.templates.remove(template_id)
    }

    pub fn update_template(&mut self, template: ProjectTemplate) -> Result<(), Box<dyn std::error::Error>> {
        let errors = template.validate();
        if !errors.is_empty() {
            return Err(format!("Template validation failed: {}", errors.join(", ")).into());
        }

        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    pub fn add_category(&mut self, category: TemplateCategory) {
        self.categories.insert(category.id.clone(), category);
    }

    pub fn get_category(&self, category_id: &str) -> Option<&TemplateCategory> {
        self.categories.get(category_id)
    }

    pub fn get_all_categories(&self) -> Vec<&TemplateCategory> {
        self.categories.values().collect()
    }

    pub fn get_template_statistics(&self) -> TemplateStatistics {
        let total_templates = self.templates.len();
        let categories_count = self.categories.len();
        
        let templates_by_category = self.templates
            .values()
            .fold(HashMap::new(), |mut map, template| {
                let count = map.entry(template.category.clone()).or_insert(0);
                *count += 1;
                map
            });

        let total_asset_templates = self.templates
            .values()
            .map(|t| t.asset_templates.len())
            .sum();

        let average_assets_per_template = if total_templates > 0 {
            total_asset_templates as f64 / total_templates as f64
        } else {
            0.0
        };

        TemplateStatistics {
            total_templates,
            categories_count,
            templates_by_category,
            total_asset_templates,
            average_assets_per_template,
        }
    }

    pub fn export_templates(&self) -> Result<String, Box<dyn std::error::Error>> {
        serde_json::to_string_pretty(&self.templates)
    }

    pub fn import_templates(&mut self, templates_json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let imported_templates: HashMap<String, ProjectTemplate> = serde_json::from_str(templates_json)?;
        
        for (id, template) in imported_templates {
            let errors = template.validate();
            if errors.is_empty() {
                self.templates.insert(id, template);
            }
        }
        
        Ok(())
    }

    pub fn create_default_templates(&mut self) {
        let image_template = self.create_image_project_template();
        let _ = self.add_template(image_template);

        let video_template = self.create_video_project_template();
        let _ = self.add_template(video_template);

        let audio_template = self.create_audio_project_template();
        let _ = self.add_template(audio_template);

        let mixed_template = self.create_mixed_media_template();
        let _ = self.add_template(mixed_template);
    }

    fn create_image_project_template(&self) -> ProjectTemplate {
        let asset_template = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Image".to_string(),
            description: "Primary image for the project".to_string(),
            asset_type: AssetType::Image,
            required: true,
            default_path: "assets/images/main.png".to_string(),
            relative_path: "assets/images/main.png".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("png".to_string()),
                dimensions: Some((1920, 1080)),
                duration: None,
                sample_rate: None,
                bit_depth: Some(8),
                channels: None,
                color_space: Some("sRGB".to_string()),
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: true,
                file_size: Some(1024 * 1024),
                file_content: None,
                file_extension: "png".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        ProjectTemplate::new(
            uuid::Uuid::new_v4().to_string(),
            "Image Project".to_string(),
            "Image".to_string(),
        )
        .with_description("Template for image-based projects".to_string())
        .with_tags(vec!["image".to_string(), "graphics".to_string()])
        .add_asset_template(asset_template)
    }

    fn create_video_project_template(&self) -> ProjectTemplate {
        let asset_template = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Video".to_string(),
            description: "Primary video for the project".to_string(),
            asset_type: AssetType::Video,
            required: true,
            default_path: "assets/videos/main.mp4".to_string(),
            relative_path: "assets/videos/main.mp4".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("mp4".to_string()),
                dimensions: Some((1920, 1080)),
                duration: Some(60),
                sample_rate: Some(48000),
                bit_depth: None,
                channels: Some(2),
                color_space: Some("Rec.709".to_string()),
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: true,
                file_size: Some(10 * 1024 * 1024),
                file_content: None,
                file_extension: "mp4".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        ProjectTemplate::new(
            uuid::Uuid::new_v4().to_string(),
            "Video Project".to_string(),
            "Video".to_string(),
        )
        .with_description("Template for video-based projects".to_string())
        .with_tags(vec!["video".to_string(), "multimedia".to_string()])
        .add_asset_template(asset_template)
    }

    fn create_audio_project_template(&self) -> ProjectTemplate {
        let asset_template = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Audio".to_string(),
            description: "Primary audio for the project".to_string(),
            asset_type: AssetType::Audio,
            required: true,
            default_path: "assets/audio/main.wav".to_string(),
            relative_path: "assets/audio/main.wav".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("wav".to_string()),
                dimensions: None,
                duration: Some(180),
                sample_rate: Some(44100),
                bit_depth: Some(16),
                channels: Some(2),
                color_space: None,
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: true,
                file_size: Some(5 * 1024 * 1024),
                file_content: None,
                file_extension: "wav".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        ProjectTemplate::new(
            uuid::Uuid::new_v4().to_string(),
            "Audio Project".to_string(),
            "Audio".to_string(),
        )
        .with_description("Template for audio-based projects".to_string())
        .with_tags(vec!["audio".to_string(), "sound".to_string()])
        .add_asset_template(asset_template)
    }

    fn create_mixed_media_template(&self) -> ProjectTemplate {
        let image_asset = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Thumbnail".to_string(),
            description: "Project thumbnail image".to_string(),
            asset_type: AssetType::Image,
            required: true,
            default_path: "assets/images/thumbnail.jpg".to_string(),
            relative_path: "assets/images/thumbnail.jpg".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("jpg".to_string()),
                dimensions: Some((1280, 720)),
                duration: None,
                sample_rate: None,
                bit_depth: Some(8),
                channels: None,
                color_space: Some("sRGB".to_string()),
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: true,
                file_size: Some(512 * 1024),
                file_content: None,
                file_extension: "jpg".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        let video_asset = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Video".to_string(),
            description: "Main video content".to_string(),
            asset_type: AssetType::Video,
            required: false,
            default_path: "assets/videos/main.mp4".to_string(),
            relative_path: "assets/videos/main.mp4".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("mp4".to_string()),
                dimensions: Some((1920, 1080)),
                duration: Some(120),
                sample_rate: Some(48000),
                bit_depth: None,
                channels: Some(2),
                color_space: Some("Rec.709".to_string()),
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: false,
                file_size: None,
                file_content: None,
                file_extension: "mp4".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        let audio_asset = AssetTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Background Music".to_string(),
            description: "Background audio track".to_string(),
            asset_type: AssetType::Audio,
            required: false,
            default_path: "assets/audio/background.mp3".to_string(),
            relative_path: "assets/audio/background.mp3".to_string(),
            metadata_template: AssetMetadataTemplate {
                format: Some("mp3".to_string()),
                dimensions: None,
                duration: Some(180),
                sample_rate: Some(44100),
                bit_depth: Some(16),
                channels: Some(2),
                color_space: None,
                additional: HashMap::new(),
            },
            creation_settings: AssetCreationSettings {
                create_file: false,
                file_size: None,
                file_content: None,
                file_extension: "mp3".to_string(),
                compression: None,
                encoding: "binary".to_string(),
            },
        };

        ProjectTemplate::new(
            uuid::Uuid::new_v4().to_string(),
            "Mixed Media Project".to_string(),
            "Mixed".to_string(),
        )
        .with_description("Template for projects with multiple media types".to_string())
        .with_tags(vec!["mixed".to_string(), "multimedia".to_string(), "video".to_string(), "audio".to_string()])
        .add_asset_template(image_asset)
        .add_asset_template(video_asset)
        .add_asset_template(audio_asset)
    }
}

#[derive(Debug, Clone)]
pub struct TemplateStatistics {
    pub total_templates: usize,
    pub categories_count: usize,
    pub templates_by_category: HashMap<String, usize>,
    pub total_asset_templates: usize,
    pub average_assets_per_template: f64,
}

impl Default for ProjectTemplate {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Template".to_string(),
            "General".to_string(),
        )
    }
}

impl Default for AssetTemplate {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Asset".to_string(),
            description: "Default asset template".to_string(),
            asset_type: AssetType::Binary,
            required: false,
            default_path: "assets/default.bin".to_string(),
            relative_path: "assets/default.bin".to_string(),
            metadata_template: AssetMetadataTemplate::default(),
            creation_settings: AssetCreationSettings::default(),
        }
    }
}

impl Default for AssetMetadataTemplate {
    fn default() -> Self {
        Self {
            format: None,
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

impl Default for AssetCreationSettings {
    fn default() -> Self {
        Self {
            create_file: false,
            file_size: None,
            file_content: None,
            file_extension: "bin".to_string(),
            compression: None,
            encoding: "binary".to_string(),
        }
    }
}

impl Default for WorkspaceLayoutTemplate {
    fn default() -> Self {
        Self {
            layout_type: "default".to_string(),
            panels: vec![
                PanelTemplate {
                    id: "main".to_string(),
                    name: "Main Panel".to_string(),
                    panel_type: "workspace".to_string(),
                    position: (0, 0),
                    size: (800, 600),
                    visible: true,
                    docked: true,
                    config: HashMap::new(),
                }
            ],
            default_size: (800, 600),
            resizable: true,
            theme: "default".to_string(),
        }
    }
}

impl Default for PanelTemplate {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Panel".to_string(),
            panel_type: "generic".to_string(),
            position: (0, 0),
            size: (400, 300),
            visible: true,
            docked: true,
            config: HashMap::new(),
        }
    }
}

impl Default for ProjectStructureTemplate {
    fn default() -> Self {
        Self {
            directories: vec![
                DirectoryTemplate {
                    name: "assets".to_string(),
                    path: "assets".to_string(),
                    description: "Project assets directory".to_string(),
                    required: true,
                    created_automatically: true,
                }
            ],
            files: Vec::new(),
            naming_convention: NamingConvention::default(),
            organization_rules: Vec::new(),
        }
    }
}

impl Default for DirectoryTemplate {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: "default".to_string(),
            description: "Default directory".to_string(),
            required: false,
            created_automatically: false,
        }
    }
}

impl Default for FileTemplate {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: "default".to_string(),
            description: "Default file".to_string(),
            required: false,
            created_automatically: false,
            content_template: None,
            file_type: "text".to_string(),
        }
    }
}

impl Default for NamingConvention {
    fn default() -> Self {
        Self {
            project_name_pattern: "{name}_{timestamp}".to_string(),
            asset_name_pattern: "{name}_{index}".to_string(),
            directory_name_pattern: "{type}_{name}".to_string(),
            file_name_pattern: "{name}.{ext}".to_string(),
            case_sensitive: true,
        }
    }
}

impl Default for OrganizationRule {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Rule".to_string(),
            description: "Default organization rule".to_string(),
            rule_type: OrganizationRuleType::FileType,
            pattern: "*.png".to_string(),
            action: OrganizationAction::MoveToDirectory,
            priority: 1,
        }
    }
}

impl Default for TemplateSettings {
    fn default() -> Self {
        Self {
            auto_create_assets: true,
            auto_create_directories: true,
            apply_workspace_layout: true,
            apply_organization_rules: true,
            validate_before_creation: true,
            allow_modification: true,
            version_control: false,
            backup_original: false,
        }
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Template Manager".to_string(),
            TemplateManagerSettings::default(),
        )
    }
}

impl Default for TemplateCategory {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "General".to_string(),
            description: "General templates".to_string(),
            parent_id: None,
            icon: "folder".to_string(),
            color: "#808080".to_string(),
        }
    }
}

impl Default for TemplateManagerSettings {
    fn default() -> Self {
        Self {
            default_template_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("TiffinyStudio")
                .join("templates"),
            auto_load_templates: true,
            validate_templates: true,
            cache_templates: true,
            max_cached_templates: 50,
            template_update_interval: std::time::Duration::from_secs(3600),
        }
    }
}

impl Default for TemplateApplication {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            template_id: "default".to_string(),
            project_id: "default".to_string(),
            applied_at: Utc::now(),
            modifications: Vec::new(),
            status: ApplicationStatus::Pending,
        }
    }
}

impl Default for TemplateModification {
    fn default() -> Self {
        Self {
            field: "default".to_string(),
            old_value: None,
            new_value: None,
            modification_type: ModificationType::Added,
            timestamp: Utc::now(),
        }
    }
}

impl Default for TemplateStatistics {
    fn default() -> Self {
        Self {
            total_templates: 0,
            categories_count: 0,
            templates_by_category: HashMap::new(),
            total_asset_templates: 0,
            average_assets_per_template: 0.0,
        }
    }
}
