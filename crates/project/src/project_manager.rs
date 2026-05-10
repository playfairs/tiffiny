use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProjectManager {
    pub id: String,
    pub name: String,
    pub projects: Arc<RwLock<HashMap<String, super::Project>>>>,
    pub active_project: Arc<RwLock<Option<String>>>>,
    pub event_sender: mpsc::UnboundedSender<ProjectManagerEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProjectManagerEvent>>>>,
    pub settings: Arc<RwLock<ProjectManagerSettings>>,
}

#[derive(Debug, Clone)]
pub enum ProjectManagerEvent {
    ProjectCreated(String),
    ProjectOpened(String),
    ProjectClosed(String),
    ProjectSaved(String),
    ProjectDeleted(String),
    ProjectRenamed(String, String),
    ProjectExported(String, String),
    ProjectImported(String),
    Error(String),
    Warning(String),
}

#[derive(Debug, Clone)]
pub struct ProjectManagerSettings {
    pub auto_save_enabled: bool,
    pub auto_save_interval: std::time::Duration,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub max_recent_projects: usize,
    pub default_project_path: Option<PathBuf>,
    pub auto_create_backup: bool,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl ProjectManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            projects: Arc::new(RwLock::new(HashMap::new())),
            active_project: Arc::new(RwLock::new(None))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
            settings: Arc::new(RwLock::new(ProjectManagerSettings::default())),
        }
    }

    pub async fn create_project(&self, name: String, path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let project = super::Project::new(
            uuid::Uuid::new_v4().to_string(),
            name,
        );

Save project to disk
        let project_path = path.join(format!("{}.tiffiny", project.id));
        self.save_project_to_disk(&project, &project_path).await?;

        {
            let mut projects = self.projects.write();
            projects.insert(project.id.clone(), project);
        }

        let _ = self.event_sender.send(ProjectManagerEvent::ProjectCreated(project.id.clone()));
        Ok(project.id)
    }

    pub async fn open_project(&self, project_id: &str) -> Result<super::Project, Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            let project_path = self.find_project_file(project_id)?;
            let loaded_project = self.load_project_from_disk(&project_path).await?;
            
            {
                let mut project_ref = project.state.write();
                *project_ref = super::ProjectState::Loaded;
            }

            {
                let mut active = self.active_project.write();
                *active = Some(project_id.to_string());
            }

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectOpened(project_id.to_string()));
            Ok(loaded_project)
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn close_project(&self, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            let project_path = self.find_project_file(project_id)?;
            self.save_project_to_disk(project, &project_path).await?;
            
            {
                let mut project_ref = project.state.write();
                *project_ref = super::ProjectState::Closed;
            }

            {
                let mut active = self.active_project.read();
                if let Some(active_id) = *active {
                    if active_id == project_id {
                        *active = None;
                    }
                }
            }

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectClosed(project_id.to_string()));
            Ok(())
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn save_project(&self, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            let project_path = self.find_project_file(project_id)?;
            self.save_project_to_disk(project, &project_path).await?;
            
            {
                let mut project_ref = project.state.write();
                *project_ref = super::ProjectState::Loaded;
            }

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectSaved(project_id.to_string()));
            Ok(())
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn delete_project(&self, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut projects = self.projects.write();
        
        if let Some(project) = projects.remove(project_id) {
            let project_path = self.find_project_file(project_id)?;
            if project_path.exists() {
                tokio::fs::remove_dir_all(&project_path).await?;
            }

            {
                let mut active = self.active_project.write();
                if let Some(active_id) = *active {
                    if active_id == project_id {
                        *active = None;
                    }
                }
            }

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectDeleted(project_id.to_string()));
            Ok(())
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn rename_project(&self, project_id: &str, new_name: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut projects = self.projects.write();
        
        if let Some(project) = projects.get_mut(project_id) {
            project.name = new_name.clone();
            
            let project_path = self.find_project_file(project_id)?;
            self.save_project_to_disk(project, &project_path).await?;

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectRenamed(project_id.to_string(), new_name));
            Ok(())
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub fn get_project(&self, project_id: &str) -> Option<super::Project> {
        let projects = self.projects.read();
        projects.get(project_id).cloned()
    }

    pub fn get_all_projects(&self) -> Vec<super::Project> {
        let projects = self.projects.read();
        projects.values().cloned().collect()
    }

    pub fn get_active_project(&self) -> Option<String> {
        self.active_project.read().clone()
    }

    pub fn set_active_project(&self, project_id: Option<String>) {
        let mut active = self.active_project.write();
        *active = project_id;
    }

    pub fn get_project_count(&self) -> usize {
        self.projects.read().len()
    }

    pub async fn export_project(&self, project_id: &str, export_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            tokio::fs::create_dir_all(&export_path).await?;
            
            let project_file = export_path.join(format!("{}.tiffiny", project_id));
            self.save_project_to_disk(project, &project_file).await?;
            
            let assets_dir = export_path.join("assets");
            tokio::fs::create_dir_all(&assets_dir).await?;
            
            for asset in project.assets.values() {
                if asset.path.exists() {
                    let asset_dest = assets_dir.join(asset.path.file_name().unwrap_or("asset"));
                    tokio::fs::copy(&asset.path, &asset_dest).await?;
                }
            }

            let _ = self.event_sender.send(ProjectManagerEvent::ProjectExported(project_id.to_string(), export_path.to_string_lossy()));
            Ok(())
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn import_project(&self, import_path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let project = self.load_project_from_disk(&import_path).await?;
        
        let project_id = uuid::Uuid::new_v4().to_string();
        
        {
            let mut projects = self.projects.write();
            projects.insert(project_id.clone(), project);
        }

        let _ = self.event_sender.send(ProjectManagerEvent::ProjectImported(project_id.to_string()));
        Ok(project_id)
    }

    pub fn get_recent_projects(&self) -> Vec<super::Project> {
        let projects = self.projects.read();
        let settings = self.settings.read();
        
        let mut recent_projects: Vec<_> = projects.values().collect();
        recent_projects.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        
        recent_projects.into_iter().take(settings.max_recent_projects).collect()
    }

    pub fn search_projects(&self, query: &str) -> Vec<super::Project> {
        let projects = self.projects.read();
        let query_lower = query.to_lowercase();
        
        projects
            .values()
            .filter(|project| {
                project.name.to_lowercase().contains(&query_lower) ||
                project.description.to_lowercase().contains(&query_lower) ||
                project.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) ||
                project.author.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    pub fn get_projects_by_tag(&self, tag: &str) -> Vec<super::Project> {
        let projects = self.projects.read();
        
        projects
            .values()
            .filter(|project| project.has_tag(tag))
            .cloned()
            .collect()
    }

    pub fn get_projects_by_author(&self, author: &str) -> Vec<super::Project> {
        let projects = self.projects.read();
        
        projects
            .values()
            .filter(|project| project.author == author)
            .cloned()
            .collect()
    }

    pub async fn backup_project(&self, project_id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            let settings = self.settings.read();
            
            let backup_dir = self.get_backup_directory().join("backups");
            tokio::fs::create_dir_all(&backup_dir).await?;
            
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let backup_filename = format!("{}_{}_{}.tiffiny", project.name, project_id, timestamp);
            let backup_path = backup_dir.join(backup_filename);
            
            self.save_project_to_disk(project, &backup_path).await?;
            
            self.cleanup_old_backups(project_id).await?;
            
            let _ = self.event_sender.send(ProjectManagerEvent::Warning(format!("Project {} backed up to {:?}", project_id, backup_path)));
            Ok(backup_path)
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub async fn restore_project(&self, backup_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let project = self.load_project_from_disk(backup_path).await?;
        
        let project_id = uuid::Uuid::new_v4().to_string();
        
        {
            let mut projects = self.projects.write();
            projects.insert(project_id.clone(), project);
        }

        let _ = self.event_sender.send(ProjectManagerEvent::Warning(format!("Project {} restored from {:?}", project_id, backup_path)));
        Ok(project_id)
    }

    pub fn get_settings(&self) -> ProjectManagerSettings {
        self.settings.read().clone()
    }

    pub fn set_settings(&self, settings: ProjectManagerSettings) {
        let mut settings_ref = self.settings.write();
        *settings_ref = settings;
    }

    pub async fn get_events(&mut self) -> Vec<ProjectManagerEvent> {
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

    async fn save_project_to_disk(&self, project: &super::Project, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        use super::project_serializer::ProjectSerializer;
        
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let project_data = ProjectSerializer::serialize(project)?;
        
        tokio::fs::write(path, project_data).await?;
        
        Ok(())
    }

    async fn load_project_from_disk(&self, path: &PathBuf) -> Result<super::Project, Box<dyn std::error::Error>> {
        use super::project_serializer::ProjectSerializer;
        
        let project_data = tokio::fs::read_to_string(path).await?;
        
        let project = ProjectSerializer::deserialize(&project_data)?;
        
        Ok(project)
    }

    fn find_project_file(&self, project_id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        
        if let Some(default_path) = &settings.default_project_path {
            let project_path = default_path.join("projects").join(format!("{}.tiffiny", project_id));
            if project_path.exists() {
                return Ok(project_path);
            }
        }
        
        let common_paths = vec![
            dirs::home_dir().map(|h| h.join("TiffinyStudio").join("projects")),
            dirs::document_dir().map(|d| d.join("TiffinyStudio").join("projects")),
            PathBuf::from("./projects"),
        ];
        
        for path in common_paths {
            let project_path = path.join(format!("{}.tiffiny", project_id));
            if project_path.exists() {
                return Ok(project_path);
            }
        }
        
        Err(format!("Project file not found for ID {}", project_id).into())
    }

    fn get_backup_directory(&self) -> PathBuf {
        let settings = self.settings.read();
        
        if let Some(default_path) = &settings.default_project_path {
            default_path.join("backups")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("TiffinyStudio")
                .join("backups")
        }
    }

    async fn cleanup_old_backups(&self, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.read();
        let backup_dir = self.get_backup_directory();
        
        let mut backup_files = tokio::fs::read_dir(&backup_dir).await?;
        let project_backups: Vec<_> = backup_files
            .filter_map(|entry| {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.starts_with(&format!("{}_{}", project_id.split('_').next().unwrap_or(""))) {
                    let path = backup_dir.join(file_name);
                    let metadata = tokio::fs::metadata(&path).await.ok()?;
                    Some((path, metadata.modified().ok()?))
                } else {
                    None
                }
            })
            .collect();
        
        project_backups.sort_by(|a, b| b.1.cmp(&a.1));
        
        let keep_count = settings.backup_count as usize;
        if project_backups.len() > keep_count {
            for (path, _) in project_backups.iter().skip(keep_count) {
                tokio::fs::remove_file(path).await?;
            }
        }
        
        Ok(())
    }

    pub fn get_project_statistics(&self) -> ProjectManagerStatistics {
        let projects = self.projects.read();
        let settings = self.settings.read();
        
        let total_projects = projects.len();
        let active_project_count = if self.active_project.read().is_some() { 1 } else { 0 };
        
        let mut total_size = 0u64;
        let mut total_assets = 0usize;
        let mut projects_by_type = HashMap::new();
        
        for project in projects.values() {
            total_size += project.get_total_size();
            total_assets += project.get_asset_count();
            
            for asset in project.assets.values() {
                let asset_type = format!("{:?}", asset.asset_type);
                projects_by_type.entry(asset_type).or_insert_with(Vec::new()).push(1);
            }
        }
        
        ProjectManagerStatistics {
            total_projects,
            active_project_count,
            total_size,
            total_assets,
            projects_by_type,
            settings: settings.clone(),
        }
    }

    pub fn validate_project(&self, project: &super::Project) -> Vec<String> {
        let mut errors = Vec::new();
        
        if project.name.is_empty() {
            errors.push("Project name cannot be empty".to_string());
        }
        
        if project.id.is_empty() {
            errors.push("Project ID cannot be empty".to_string());
        }
        
        for (asset_id, asset) in &project.assets {
            if asset.name.is_empty() {
                errors.push(format!("Asset {} has empty name", asset_id));
            }
            
            if !asset.path.exists() {
                errors.push(format!("Asset {} path does not exist: {:?}", asset_id, asset.path));
            }
        }
        
        errors
    }

    pub fn optimize_project_storage(&self, project_id: &str) -> Result<StorageOptimizationResult, Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        
        if let Some(project) = projects.get(project_id) {
            let mut suggestions = Vec::new();
            let mut total_savings = 0u64;
            
            let mut seen_paths = std::collections::HashSet::new();
            for asset in project.assets.values() {
                if seen_paths.contains(&asset.path) {
                    suggestions.push(format!("Duplicate asset found: {:?}", asset.path));
                    total_savings += asset.size;
                }
                seen_paths.insert(asset.path.clone());
            }
            
            for asset in project.assets.values() {
                if asset.size > 10 * 1024 * 1024 && asset.asset_type == super::AssetType::Image {
                    suggestions.push(format!("Large image asset could be compressed: {:?}", asset.path));
                }
            }
            
            StorageOptimizationResult {
                original_size: project.get_total_size(),
                optimized_size: project.get_total_size() - total_savings,
                savings_percentage: if project.get_total_size() > 0 {
                    (total_savings as f64 / project.get_total_size() as f64) * 100.0
                } else {
                    0.0
                },
                suggestions,
            }
        } else {
            Err(format!("Project with ID {} not found", project_id).into())
        }
    }

    pub fn clone_project(&self, project: &super::Project) -> super::Project {
        let cloned = super::Project::new(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", project.name),
        );
        
        cloned.description = project.description.clone();
        cloned.version = project.version.clone();
        cloned.author = project.author.clone();
        cloned.tags = project.tags.clone();
        cloned.settings = project.settings.clone();
        cloned.metadata = project.metadata.clone();
        
        for asset in project.assets.values() {
            cloned.add_asset(super::Asset {
                id: uuid::Uuid::new_v4().to_string(),
                name: asset.name.clone(),
                asset_type: asset.asset_type.clone(),
                path: asset.path.clone(),
                size: asset.size,
                created_at: asset.created_at,
                modified_at: asset.modified_at,
                metadata: asset.metadata.clone(),
            });
        }
        
        cloned
    }

    pub fn create_project_from_template(&self, template: &super::ProjectTemplate, name: String) -> Result<super::Project, Box<dyn std::error::Error>> {
        let project = super::Project::new(
            uuid::Uuid::new_v4().to_string(),
            name,
        );
        
        project.settings = template.default_settings.clone();
        project.metadata = template.default_metadata.clone();
        
        for asset_template in &template.asset_templates {
            if asset_template.required {
                let asset = super::Asset {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: asset_template.name.clone(),
                    asset_type: asset_template.asset_type.clone(),
                    path: PathBuf::from(&asset_template.relative_path),
                    size: 0,
                    created_at: chrono::Utc::now(),
                    modified_at: chrono::Utc::now(),
                    metadata: asset_template.metadata_template.clone(),
                };
                project.add_asset(asset);
            }
        }
        
        Ok(project)
    }

    pub fn create_project_from_existing_files(&self, name: String, asset_paths: Vec<PathBuf>) -> Result<super::Project, Box<dyn std::error::Error>> {
        let project = super::Project::new(
            uuid::Uuid::new_v4().to_string(),
            name,
        );
        
        for asset_path in asset_paths {
            if asset_path.exists() {
                let metadata = tokio::fs::metadata(&asset_path).await?;
                let file_size = metadata.len();
                
                let asset_type = self.detect_asset_type(&asset_path);
                
                let asset = super::Asset {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: asset_path.file_name().unwrap_or("unknown").to_string_lossy(),
                    asset_type,
                    path: asset_path,
                    size: file_size,
                    created_at: metadata.created().ok_or_else(|_| chrono::Utc::now()),
                    modified_at: metadata.modified().ok_or_else(|_| chrono::Utc::now()),
                    metadata: super::AssetMetadata::default(),
                };
                
                project.add_asset(asset);
            }
        }
        
        Ok(project)
    }

    fn detect_asset_type(&self, path: &PathBuf) -> super::AssetType {
        let extension = path.extension()
            .and_then(|ext| ext.to_str().to_lowercase())
            .unwrap_or("");
        
        match extension {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => super::AssetType::Image,
            "mp4" | "avi" | "mov" | "mkv" | "webm" => super::AssetType::Video,
            "mp3" | "wav" | "ogg" | "flac" | "aac" => super::AssetType::Audio,
            "txt" | "md" | "rtf" | "doc" | "pdf" => super::AssetType::Text,
            "tiffiny" => super::AssetType::Project,
            _ => super::AssetType::Binary,
        }
    }

    pub fn get_project_path(&self, project_id: &str) -> Option<PathBuf> {
        self.find_project_file(project_id).ok()
    }

    pub fn get_asset_path(&self, project_id: &str, asset_id: &str) -> Option<PathBuf> {
        if let Some(project) = self.get_project(project_id) {
            if let Some(asset) = project.get_asset(asset_id) {
                Some(asset.path.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_project_directory(&self, project_id: &str) -> Option<PathBuf> {
        self.get_project_path(project_id).map(|path| path.parent().unwrap_or_else(|| PathBuf::from(".")))
    }

    pub fn is_project_open(&self, project_id: &str) -> bool {
        if let Some(project) = self.get_project(project_id) {
            project.is_loaded()
        } else {
            false
        }
    }

    pub fn is_project_modified(&self, project_id: &str) -> bool {
        if let Some(project) = self.get_project(project_id) {
            project.is_modified()
        } else {
            false
        }
    }

    pub fn get_project_duration(&self, project_id: &str) -> Option<chrono::Duration> {
        if let Some(project) = self.get_project(project_id) {
            Some(chrono::Utc::now() - project.created_at)
        } else {
            None
        }
    }

    pub fn auto_save_all(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let projects = self.projects.read();
        let settings = self.settings.read();
        
        if !settings.auto_save_enabled {
            return Ok(Vec::new());
        }
        
        let mut saved_projects = Vec::new();
        
        for (project_id, project) in projects.iter() {
            if project.is_modified() {
                if let Err(e) = self.save_project(project_id).await {
                    let _ = self.event_sender.send(ProjectManagerEvent::Error(format!("Failed to save project {}: {}", project_id, e)));
                } else {
                    saved_projects.push(project_id.clone());
                }
            }
        }
        
        Ok(saved_projects)
    }

    pub fn get_project_health(&self, project_id: &str) -> ProjectHealth {
        if let Some(project) = self.get_project(project_id) {
            let mut health_score = 100;
            let mut issues = Vec::new();
            
            for (asset_id, asset) in project.assets.iter() {
                if !asset.path.exists() {
                    health_score -= 10;
                    issues.push(format!("Missing asset: {}", asset_id));
                }
            }
            
            let validation_errors = self.validate_project(project);
            health_score -= validation_errors.len() as i32 * 5;
            issues.extend(validation_errors);
            
            let project_age = chrono::Utc::now() - project.created_at;
            if project_age.num_days() > 365 {
                health_score -= 5;
                issues.push("Project is over a year old".to_string());
            }
            
            let health_level = match health_score {
                90..=100 => ProjectHealthLevel::Excellent,
                70..=89 => ProjectHealthLevel::Good,
                50..=69 => ProjectHealthLevel::Fair,
                30..=49 => ProjectHealthLevel::Poor,
                0..=29 => ProjectHealthLevel::Critical,
                _ => ProjectHealthLevel::Critical,
            };
            
            ProjectHealth {
                score: health_score,
                level: health_level,
                issues,
                last_checked: chrono::Utc::now(),
            }
        } else {
            ProjectHealth {
                score: 0,
                level: ProjectHealthLevel::Critical,
                issues: vec!["Project not found".to_string()],
                last_checked: chrono::Utc::now(),
            }
        }
    }

    pub fn get_manager_health(&self) -> ManagerHealth {
        let projects = self.projects.read();
        let settings = self.settings.read();
        
        let mut health_score = 100;
        let mut issues = Vec::new();
        
        if !settings.backup_enabled {
            health_score -= 20;
            issues.push("Backups are disabled".to_string());
        }
        
        if !settings.auto_save_enabled {
            health_score -= 15;
            issues.push("Auto-save is disabled".to_string());
        }
        
        if projects.len() > 100 {
            health_score -= 10;
            issues.push("Too many projects (over 100)".to_string());
        }
        
        for (project_id, project) in projects.iter() {
            let project_health = self.get_project_health(project_id);
            if project_health.level == ProjectHealthLevel::Critical {
                health_score -= 5;
                issues.push(format!("Project {} has critical health issues", project_id));
            }
        }
        
        let health_level = match health_score {
            90..=100 => ManagerHealthLevel::Excellent,
            70..=89 => ManagerHealthLevel::Good,
            50..=69 => ManagerHealthLevel::Fair,
            30..=49 => ManagerHealthLevel::Poor,
            0..=29 => ManagerHealthLevel::Critical,
            _ => ManagerHealthLevel::Critical,
        };
        
        ManagerHealth {
            score: health_score,
            level: health_level,
            issues,
            last_checked: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectManagerStatistics {
    pub total_projects: usize,
    pub active_project_count: usize,
    pub total_size: u64,
    pub total_assets: usize,
    pub projects_by_type: HashMap<String, usize>,
    pub settings: ProjectManagerSettings,
}

#[derive(Debug, Clone)]
pub struct StorageOptimizationResult {
    pub original_size: u64,
    pub optimized_size: u64,
    pub savings_percentage: f64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectHealth {
    pub score: i32,
    pub level: ProjectHealthLevel,
    pub issues: Vec<String>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectHealthLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ManagerHealth {
    pub score: i32,
    pub level: ManagerHealthLevel,
    pub issues: Vec<String>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManagerHealthLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Project Manager".to_string(),
        )
    }
}

impl Default for ProjectManagerSettings {
    fn default() -> Self {
        Self {
            auto_save_enabled: true,
            auto_save_interval: std::time::Duration::from_secs(300),
            backup_enabled: true,
            backup_count: 5,
            max_recent_projects: 10,
            default_project_path: None,
            auto_create_backup: true,
            compression_enabled: false,
            encryption_enabled: false,
        }
    }
}

impl Default for ProjectManagerStatistics {
    fn default() -> Self {
        Self {
            total_projects: 0,
            active_project_count: 0,
            total_size: 0,
            total_assets: 0,
            projects_by_type: HashMap::new(),
            settings: ProjectManagerSettings::default(),
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

impl Default for ProjectHealth {
    fn default() -> Self {
        Self {
            score: 100,
            level: ProjectHealthLevel::Excellent,
            issues: Vec::new(),
            last_checked: chrono::Utc::now(),
        }
    }
}

impl Default for ManagerHealth {
    fn default() -> Self {
        Self {
            score: 100,
            level: ManagerHealthLevel::Excellent,
            issues: Vec::new(),
            last_checked: chrono::Utc::now(),
        }
    }
}
