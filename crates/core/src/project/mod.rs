use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: std::time::SystemTime,
    pub modified_at: std::time::SystemTime,
    pub settings: ProjectSettings,
    pub assets: HashMap<Uuid, Asset>,
    pub pipelines: HashMap<Uuid, crate::pipeline::Pipeline>,
    pub graphs: HashMap<Uuid, crate::graph::ProcessingGraph>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub auto_save_interval_seconds: u64,
    pub max_undo_steps: usize,
    pub default_sample_rate: u32,
    pub default_bit_depth: u16,
    pub default_color_space: String,
    pub cache_size_mb: u64,
    pub gpu_acceleration: bool,
    pub parallel_processing: bool,
    pub temp_dir: Option<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_save_interval_seconds: 300,
            max_undo_steps: 50,
            default_sample_rate: 44100,
            default_bit_depth: 16,
            default_color_space: "sRGB".to_string(),
            cache_size_mb: 1024,
            gpu_acceleration: true,
            parallel_processing: true,
            temp_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub file_path: String,
    pub size_bytes: u64,
    pub format: String,
    pub metadata: AssetMetadata,
    pub created_at: std::time::SystemTime,
    pub modified_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Audio,
    Image,
    Video,
    Raw,
    Project,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub duration_seconds: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub bit_depth: Option<u16>,
    pub color_space: Option<String>,
    pub codec: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHistory {
    pub id: Uuid,
    pub project_id: Uuid,
    pub version: u32,
    pub timestamp: std::time::SystemTime,
    pub description: String,
    pub changes: Vec<ProjectChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectChange {
    pub change_type: ChangeType,
    pub target_id: Uuid,
    pub target_type: String,
    pub description: String,
    pub data_before: Option<serde_json::Value>,
    pub data_after: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Moved,
    Renamed,
}

pub struct ProjectManager {
    projects: Arc<RwLock<HashMap<Uuid, Project>>>,
    history: Arc<RwLock<HashMap<Uuid, Vec<ProjectHistory>>>>,
    current_project: Arc<RwLock<Option<Uuid>>>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
            current_project: Arc::new(RwLock::new(None)),
        }
    }

    pub fn create_project(&self, name: String, description: String) -> Result<Uuid> {
        let project_id = Uuid::new_v4();
        let now = std::time::SystemTime::now();

        let project = Project {
            id: project_id,
            name,
            description,
            version: "1.0.0".to_string(),
            created_at: now,
            modified_at: now,
            settings: ProjectSettings::default(),
            assets: HashMap::new(),
            pipelines: HashMap::new(),
            graphs: HashMap::new(),
            metadata: HashMap::new(),
        };

        {
            let mut projects = self.projects.write();
            projects.insert(project_id, project);
        }

        {
            let mut current = self.current_project.write();
            *current = Some(project_id);
        }

        self.record_history(project_id, "Project created".to_string(), vec![])?;

        Ok(project_id)
    }

    pub fn open_project(&self, project_id: Uuid) -> Result<()> {
        let projects = self.projects.read();
        if !projects.contains_key(&project_id) {
            return Err(CoreError::Project(format!("Project {} not found", project_id)));
        }

        let mut current = self.current_project.write();
        *current = Some(project_id);

        Ok(())
    }

    pub fn close_project(&self) -> Result<()> {
        let mut current = self.current_project.write();
        *current = None;
        Ok(())
    }

    pub fn get_current_project(&self) -> Option<Project> {
        let current = self.current_project.read();
        if let Some(project_id) = *current {
            let projects = self.projects.read();
            projects.get(&project_id).cloned()
        } else {
            None
        }
    }

    pub fn get_project(&self, project_id: Uuid) -> Option<Project> {
        let projects = self.projects.read();
        projects.get(&project_id).cloned()
    }

    pub fn update_project(&self, project_id: Uuid, project: Project) -> Result<()> {
        let old_project = {
            let projects = self.projects.read();
            projects.get(&project_id).cloned()
        };

        if let Some(old_project) = old_project {
            let mut projects = self.projects.write();
            let mut updated_project = project;
            updated_project.modified_at = std::time::SystemTime::now();
            projects.insert(project_id, updated_project.clone());

            let changes = vec![
                ProjectChange {
                    change_type: ChangeType::Modified,
                    target_id: project_id,
                    target_type: "project".to_string(),
                    description: "Project updated".to_string(),
                    data_before: Some(serde_json::to_value(old_project)?),
                    data_after: Some(serde_json::to_value(updated_project)?),
                }
            ];

            self.record_history(project_id, "Project updated".to_string(), changes)?;

            Ok(())
        } else {
            Err(CoreError::Project(format!("Project {} not found", project_id)))
        }
    }

    pub fn delete_project(&self, project_id: Uuid) -> Result<()> {
        let project = {
            let mut projects = self.projects.write();
            projects.remove(&project_id)
        };

        if project.is_some() {
            {
                let mut history = self.history.write();
                history.remove(&project_id);
            }

            {
                let mut current = self.current_project.write();
                if *current == Some(project_id) {
                    *current = None;
                }
            }

            Ok(())
        } else {
            Err(CoreError::Project(format!("Project {} not found", project_id)))
        }
    }

    pub fn add_asset(&self, project_id: Uuid, asset: Asset) -> Result<()> {
        let mut projects = self.projects.write();
        if let Some(project) = projects.get_mut(&project_id) {
            project.assets.insert(asset.id, asset.clone());
            project.modified_at = std::time::SystemTime::now();

            let changes = vec![
                ProjectChange {
                    change_type: ChangeType::Created,
                    target_id: asset.id,
                    target_type: "asset".to_string(),
                    description: format!("Asset {} added", asset.name),
                    data_before: None,
                    data_after: Some(serde_json::to_value(asset)?),
                }
            ];

            drop(projects);
            self.record_history(project_id, "Asset added".to_string(), changes)?;

            Ok(())
        } else {
            Err(CoreError::Project(format!("Project {} not found", project_id)))
        }
    }

    pub fn remove_asset(&self, project_id: Uuid, asset_id: Uuid) -> Result<()> {
        let asset = {
            let mut projects = self.projects.write();
            if let Some(project) = projects.get_mut(&project_id) {
                let asset = project.assets.remove(&asset_id);
                project.modified_at = std::time::SystemTime::now();
                asset
            } else {
                return Err(CoreError::Project(format!("Project {} not found", project_id)));
            }
        };

        if let Some(asset) = asset {
            let changes = vec![
                ProjectChange {
                    change_type: ChangeType::Deleted,
                    target_id: asset_id,
                    target_type: "asset".to_string(),
                    description: format!("Asset {} removed", asset.name),
                    data_before: Some(serde_json::to_value(asset)?),
                    data_after: None,
                }
            ];

            self.record_history(project_id, "Asset removed".to_string(), changes)?;
            Ok(())
        } else {
            Err(CoreError::Project(format!("Asset {} not found", asset_id)))
        }
    }

    pub fn add_pipeline(&self, project_id: Uuid, pipeline: crate::pipeline::Pipeline) -> Result<()> {
        let mut projects = self.projects.write();
        if let Some(project) = projects.get_mut(&project_id) {
            project.pipelines.insert(pipeline.id, pipeline.clone());
            project.modified_at = std::time::SystemTime::now();

            let changes = vec![
                ProjectChange {
                    change_type: ChangeType::Created,
                    target_id: pipeline.id,
                    target_type: "pipeline".to_string(),
                    description: format!("Pipeline {} added", pipeline.name),
                    data_before: None,
                    data_after: Some(serde_json::to_value(pipeline)?),
                }
            ];

            drop(projects);
            self.record_history(project_id, "Pipeline added".to_string(), changes)?;

            Ok(())
        } else {
            Err(CoreError::Project(format!("Project {} not found", project_id)))
        }
    }

    pub fn add_graph(&self, project_id: Uuid, graph: crate::graph::ProcessingGraph) -> Result<()> {
        let mut projects = self.projects.write();
        if let Some(project) = projects.get_mut(&project_id) {
            project.graphs.insert(graph.id, graph.clone());
            project.modified_at = std::time::SystemTime::now();

            let changes = vec![
                ProjectChange {
                    change_type: ChangeType::Created,
                    target_id: graph.id,
                    target_type: "graph".to_string(),
                    description: format!("Graph {} added", graph.name),
                    data_before: None,
                    data_after: Some(serde_json::to_value(graph)?),
                }
            ];

            drop(projects);
            self.record_history(project_id, "Graph added".to_string(), changes)?;

            Ok(())
        } else {
            Err(CoreError::Project(format!("Project {} not found", project_id)))
        }
    }

    pub fn list_projects(&self) -> Vec<Project> {
        let projects = self.projects.read();
        projects.values().cloned().collect()
    }

    pub fn get_project_history(&self, project_id: Uuid) -> Vec<ProjectHistory> {
        let history = self.history.read();
        history.get(&project_id).cloned().unwrap_or_default()
    }

    pub fn undo_last_change(&self, project_id: Uuid) -> Result<bool> {
        let mut history = self.history.write();
        if let Some(project_history) = history.get_mut(&project_id) {
            if !project_history.is_empty() {
                project_history.pop();
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn record_history(&self, project_id: Uuid, description: String, changes: Vec<ProjectChange>) -> Result<()> {
        let history_entry = ProjectHistory {
            id: Uuid::new_v4(),
            project_id,
            version: self.get_next_version(project_id),
            timestamp: std::time::SystemTime::now(),
            description,
            changes,
        };

        let mut history = self.history.write();
        let project_history = history.entry(project_id).or_insert_with(Vec::new);
        project_history.push(history_entry);

        Ok(())
    }

    fn get_next_version(&self, project_id: Uuid) -> u32 {
        let history = self.history.read();
        if let Some(project_history) = history.get(&project_id) {
            project_history.len() as u32 + 1
        } else {
            1
        }
    }

    pub fn export_project(&self, project_id: Uuid, output_path: &str) -> Result<()> {
        let project = self.get_project(project_id)
            .ok_or_else(|| CoreError::Project(format!("Project {} not found", project_id)))?;

        let project_data = serde_json::json!({
            "project": project,
            "history": self.get_project_history(project_id),
            "exported_at": std::time::SystemTime::now()
        });

        std::fs::write(output_path, serde_json::to_string_pretty(&project_data)?)?;
        Ok(())
    }

    pub fn import_project(&self, import_path: &str) -> Result<Uuid> {
        let project_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(import_path)?)?;
        
        let project: Project = serde_json::from_value(project_data["project"].clone())?;
        let project_id = project.id;

        {
            let mut projects = self.projects.write();
            projects.insert(project_id, project);
        }

        if let Ok(history_data) = serde_json::from_value::<Vec<ProjectHistory>>(project_data["history"].clone()) {
            let mut history = self.history.write();
            history.insert(project_id, history_data);
        }

        Ok(project_id)
    }
}
