use tiffiny_core::{prelude::*, ProjectManager, SessionManager, TaskManager};
use tiffiny_project::{Project, Asset};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct AppState {
    pub projects: Arc<ProjectManager>,
    pub sessions: Arc<SessionManager>,
    pub tasks: Arc<TaskManager>,
    pub current_session: Option<Uuid>,
    pub current_project: Option<Uuid>,
    pub running_tasks: Vec<TaskExecution>,
    pub recent_files: Vec<String>,
    pub clipboard: Option<ClipboardData>,
}

#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub task_id: Uuid,
    pub execution_id: Uuid,
    pub status: TaskStatus,
    pub progress: f32,
    pub started_at: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub data_type: ClipboardDataType,
    pub data: serde_json::Value,
    pub copied_at: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum ClipboardDataType {
    Text,
    Image,
    Audio,
    Video,
    AssetReference,
    Pipeline,
    Graph,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(ProjectManager::new()),
            sessions: Arc::new(SessionManager::new(3600)),
            tasks: Arc::new(TaskManager::new()),
            current_session: None,
            current_project: None,
            running_tasks: Vec::new(),
            recent_files: Vec::new(),
            clipboard: None,
        }
    }

    pub async fn create_project(&mut self, name: String) -> Result<Uuid> {
        let project_id = self.projects.create_project(name, "New project created from app state".to_string())?;
        
        if let Some(session_id) = self.current_session {
            self.projects.open_project(project_id)?;
            self.sessions.set_current_project(session_id, Some(project_id))?;
        }
        
        self.current_project = Some(project_id);
        
        Ok(project_id)
    }

    pub async fn open_project(&mut self, path: &str) -> Result<Uuid> {
        let project_id = self.projects.import_project(path)?;
        
        if let Some(session_id) = self.current_session {
            self.projects.open_project(project_id)?;
            self.sessions.set_current_project(session_id, Some(project_id))?;
        }
        
        self.current_project = Some(project_id);
        
        self.add_recent_file(path.to_string()).await;
        
        Ok(project_id)
    }

    pub async fn save_project(&self, project_id: Uuid) -> Result<()> {
        let project = self.projects.get_project(project_id)
            .ok_or_else(|| CoreError::Project(format!("Project {} not found", project_id)))?;
        
        let project_dir = dirs::document_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("TiffinyProjects");
        
        std::fs::create_dir_all(&project_dir)?;
        
        let project_file = project_dir.join(format!("{}.tiffiny", project.name));
        self.projects.export_project(project_id, project_file.to_str().unwrap())?;
        
        Ok(())
    }

    pub async fn import_file(&mut self, path: &str) -> Result<Uuid> {
        let asset_id = Uuid::new_v4();
        let file_metadata = std::fs::metadata(path)?;
        
        let asset_type = self.determine_asset_type(path)?;
        let asset_metadata = self.extract_asset_metadata(path, &asset_type).await?;
        
        let asset = Asset {
            id: asset_id,
            name: std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            asset_type,
            file_path: path.to_string(),
            size_bytes: file_metadata.len(),
            format: self.extract_file_format(path),
            metadata: asset_metadata,
            created_at: file_metadata.created().unwrap_or(std::time::SystemTime::now()),
            modified_at: file_metadata.modified().unwrap_or(std::time::SystemTime::now()),
        };
        
        if let Some(project_id) = self.current_project {
            self.projects.add_asset(project_id, asset)?;
        }
        
        self.add_recent_file(path.to_string()).await;
        
        Ok(asset_id)
    }

    pub async fn export_file(&self, path: &str, format: &str) -> Result<Uuid> {
        let export_id = Uuid::new_v4();
        
                
        Ok(export_id)
    }

    pub async fn execute_pipeline(&mut self, pipeline_id: Uuid) -> Result<Uuid> {
        let execution_id = self.tasks.create_execution(pipeline_id)?;
        self.tasks.start_execution(execution_id)?;
        
        let task_execution = TaskExecution {
            task_id: pipeline_id,
            execution_id,
            status: TaskStatus::Running,
            progress: 0.0,
            started_at: std::time::SystemTime::now(),
        };
        
        self.running_tasks.push(task_execution);
        
        Ok(execution_id)
    }

    pub fn get_current_project_id(&self) -> Option<Uuid> {
        self.current_project
    }

    pub fn get_running_tasks(&self) -> &Vec<TaskExecution> {
        &self.running_tasks
    }

    pub fn update_task_progress(&mut self, execution_id: Uuid, progress: f32) -> Result<()> {
        if let Some(task_execution) = self.running_tasks.iter_mut()
            .find(|te| te.execution_id == execution_id) {
            task_execution.progress = progress.clamp(0.0, 1.0);
        }
        
        self.tasks.update_progress(execution_id, progress)?;
        
        Ok(())
    }

    pub fn complete_task(&mut self, execution_id: Uuid, result: Option<serde_json::Value>) -> Result<()> {
        let task_result = tiffiny_core::task::TaskResult {
            success: true,
            output_data: result,
            output_files: Vec::new(),
            metrics: tiffiny_core::task::TaskMetrics {
                duration_ms: 0,
                memory_used_mb: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
            },
        };
        
        self.tasks.complete_execution(execution_id, task_result)?;
        
        self.running_tasks.retain(|te| te.execution_id != execution_id);
        
        Ok(())
    }

    pub fn fail_task(&mut self, execution_id: Uuid, error: String) -> Result<()> {
        self.tasks.fail_execution(execution_id, error)?;
        
        self.running_tasks.retain(|te| te.execution_id != execution_id);
        
        Ok(())
    }

    pub fn set_clipboard(&mut self, data_type: ClipboardDataType, data: serde_json::Value) {
        self.clipboard = Some(ClipboardData {
            data_type,
            data,
            copied_at: std::time::SystemTime::now(),
        });
    }

    pub fn get_clipboard(&self) -> Option<&ClipboardData> {
        self.clipboard.as_ref()
    }

    pub fn clear_clipboard(&mut self) {
        self.clipboard = None;
    }

    async fn add_recent_file(&mut self, file_path: String) {
        self.recent_files.retain(|f| f != &file_path);
        self.recent_files.insert(0, file_path);
        self.recent_files.truncate(20);
    }

    fn determine_asset_type(&self, path: &str) -> Result<tiffiny_project::AssetType> {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        
        match extension.as_deref() {
            Some("wav" | "mp3" | "flac" | "ogg" | "aiff" | "m4a") => {
                Ok(tiffiny_project::AssetType::Audio)
            },
            Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" | "avif") => {
                Ok(tiffiny_project::AssetType::Image)
            },
            Some("mp4" | "avi" | "mov" | "mkv" | "webm" | "flv" | "wmv") => {
                Ok(tiffiny_project::AssetType::Video)
            },
            Some("tiffiny") => {
                Ok(tiffiny_project::AssetType::Project)
            },
            _ => {
                Ok(tiffiny_project::AssetType::Raw)
            }
        }
    }

    async fn extract_asset_metadata(&self, path: &str, asset_type: &tiffiny_project::AssetType) -> Result<tiffiny_project::AssetMetadata> {
        let mut metadata = tiffiny_project::AssetMetadata {
            duration_seconds: None,
            sample_rate: None,
            channels: None,
            width: None,
            height: None,
            frame_rate: None,
            bit_depth: None,
            color_space: None,
            codec: None,
            custom_fields: HashMap::new(),
        };

        match asset_type {
            tiffiny_project::AssetType::Audio => {
                if let Ok(audio_info) = self.extract_audio_metadata(path).await {
                    metadata.duration_seconds = audio_info.duration_seconds;
                    metadata.sample_rate = audio_info.sample_rate;
                    metadata.channels = audio_info.channels;
                    metadata.bit_depth = audio_info.bit_depth;
                    metadata.codec = audio_info.codec;
                }
            },
            tiffiny_project::AssetType::Image => {
                if let Ok(image_info) = self.extract_image_metadata(path).await {
                    metadata.width = image_info.width;
                    metadata.height = image_info.height;
                    metadata.bit_depth = image_info.bit_depth;
                    metadata.color_space = image_info.color_space;
                    metadata.codec = image_info.codec;
                }
            },
            tiffiny_project::AssetType::Video => {
                if let Ok(video_info) = self.extract_video_metadata(path).await {
                    metadata.duration_seconds = video_info.duration_seconds;
                    metadata.width = video_info.width;
                    metadata.height = video_info.height;
                    metadata.frame_rate = video_info.frame_rate;
                    metadata.codec = video_info.codec;
                }
            },
            _ => {}
        }

        Ok(metadata)
    }

    async fn extract_audio_metadata(&self, _path: &str) -> Result<AudioMetadata> {
        Ok(AudioMetadata {
            duration_seconds: Some(180.0),
            sample_rate: Some(44100),
            channels: Some(2),
            bit_depth: Some(16),
            codec: Some("PCM".to_string()),
        })
    }

    async fn extract_image_metadata(&self, _path: &str) -> Result<ImageMetadata> {
        Ok(ImageMetadata {
            width: Some(1920),
            height: Some(1080),
            bit_depth: Some(8),
            color_space: Some("sRGB".to_string()),
            codec: Some("PNG".to_string()),
        })
    }

    async fn extract_video_metadata(&self, _path: &str) -> Result<VideoMetadata> {
        Ok(VideoMetadata {
            duration_seconds: Some(300.0),
            width: Some(1920),
            height: Some(1080),
            frame_rate: Some(30.0),
            codec: Some("H.264".to_string()),
        })
    }

    fn extract_file_format(&self, path: &str) -> String {
        std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_uppercase()
    }
}

#[derive(Debug)]
struct AudioMetadata {
    pub duration_seconds: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub bit_depth: Option<u16>,
    pub codec: Option<String>,
}

#[derive(Debug)]
struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bit_depth: Option<u16>,
    pub color_space: Option<String>,
    pub codec: Option<String>,
}

#[derive(Debug)]
struct VideoMetadata {
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub codec: Option<String>,
}
