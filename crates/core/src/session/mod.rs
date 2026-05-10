use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub user_id: Option<String>,
    pub created_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub expires_at: Option<std::time::SystemTime>,
    pub settings: SessionSettings,
    pub state: SessionState,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub auto_save: bool,
    pub auto_save_interval_seconds: u64,
    pub max_history_entries: usize,
    pub theme: String,
    pub language: String,
    pub workspace_layout: WorkspaceLayout,
    pub performance_settings: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub panel_configurations: Vec<PanelConfiguration>,
    pub dock_layout: DockLayout,
    pub window_state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfiguration {
    pub panel_id: String,
    pub panel_type: PanelType,
    pub visible: bool,
    pub docked: bool,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub z_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    Timeline,
    Waveform,
    Spectrogram,
    ProjectExplorer,
    Inspector,
    Preview,
    Console,
    Properties,
    Effects,
    Export,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayout {
    pub main_dock_area: DockArea,
    pub floating_panels: Vec<PanelConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockArea {
    pub orientation: DockOrientation,
    pub children: Vec<DockNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockNode {
    Leaf(PanelConfiguration),
    Container {
        orientation: DockOrientation,
        children: Vec<DockNode>,
        sizes: Vec<f32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub maximized: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub max_concurrent_tasks: usize,
    pub gpu_acceleration: bool,
    pub cache_size_mb: u64,
    pub buffer_size_mb: u64,
    pub thread_pool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub current_project_id: Option<Uuid>,
    pub open_files: Vec<OpenFile>,
    pub recent_projects: Vec<RecentProject>,
    pub clipboard_data: Option<ClipboardData>,
    pub undo_stack: Vec<UndoEntry>,
    pub redo_stack: Vec<UndoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFile {
    pub file_path: String,
    pub file_type: String,
    pub opened_at: std::time::SystemTime,
    pub last_modified: std::time::SystemTime,
    pub cursor_position: Option<(usize, usize)>,
    pub scroll_position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub project_id: Uuid,
    pub name: String,
    pub file_path: String,
    pub last_opened: std::time::SystemTime,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub data_type: ClipboardDataType,
    pub data: serde_json::Value,
    pub copied_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardDataType {
    Text,
    Image,
    Audio,
    Video,
    AssetReference,
    Pipeline,
    Graph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub id: Uuid,
    pub action: String,
    pub timestamp: std::time::SystemTime,
    pub data_before: serde_json::Value,
    pub data_after: serde_json::Value,
    pub description: String,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    current_session: Arc<RwLock<Option<Uuid>>>,
    session_timeout_seconds: u64,
}

impl SessionManager {
    pub fn new(session_timeout_seconds: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            current_session: Arc::new(RwLock::new(None)),
            session_timeout_seconds,
        }
    }

    pub fn create_session(&self, name: String, user_id: Option<String>) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = std::time::SystemTime::now();

        let session = Session {
            id: session_id,
            name,
            user_id,
            created_at: now,
            last_accessed: now,
            expires_at: None,
            settings: SessionSettings::default(),
            state: SessionState::default(),
            metadata: HashMap::new(),
        };

        {
            let mut sessions = self.sessions.write();
            sessions.insert(session_id, session);
        }

        {
            let mut current = self.current_session.write();
            *current = Some(session_id);
        }

        Ok(session_id)
    }

    pub fn open_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_accessed = std::time::SystemTime::now();
            
            let mut current = self.current_session.write();
            *current = Some(session_id);
            
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn close_session(&self, session_id: Uuid) -> Result<()> {
        {
            let mut current = self.current_session.write();
            if *current == Some(session_id) {
                *current = None;
            }
        }

        let mut sessions = self.sessions.write();
        sessions.remove(&session_id);
        Ok(())
    }

    pub fn get_current_session(&self) -> Option<Session> {
        let current = self.current_session.read();
        if let Some(session_id) = *current {
            let sessions = self.sessions.read();
            sessions.get(&session_id).cloned()
        } else {
            None
        }
    }

    pub fn get_session(&self, session_id: Uuid) -> Option<Session> {
        let sessions = self.sessions.read();
        sessions.get(&session_id).cloned()
    }

    pub fn update_session(&self, session_id: Uuid, session: Session) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(_) = sessions.get(&session_id) {
            let mut updated_session = session;
            updated_session.last_accessed = std::time::SystemTime::now();
            sessions.insert(session_id, updated_session);
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn set_current_project(&self, session_id: Uuid, project_id: Option<Uuid>) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.current_project_id = project_id;
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn add_open_file(&self, session_id: Uuid, file: OpenFile) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.open_files.push(file);
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn remove_open_file(&self, session_id: Uuid, file_path: &str) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.open_files.retain(|f| f.file_path != file_path);
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn add_recent_project(&self, session_id: Uuid, project: RecentProject) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.recent_projects.retain(|p| p.project_id != project.project_id);
            session.state.recent_projects.insert(0, project);
            session.state.recent_projects.truncate(10);
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn set_clipboard_data(&self, session_id: Uuid, data: ClipboardData) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.clipboard_data = Some(data);
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn add_undo_entry(&self, session_id: Uuid, entry: UndoEntry) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.state.undo_stack.push(entry);
            session.state.redo_stack.clear();
            
            let max_entries = session.settings.max_history_entries;
            if session.state.undo_stack.len() > max_entries {
                session.state.undo_stack.remove(0);
            }
            
            session.last_accessed = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn undo(&self, session_id: Uuid) -> Result<Option<UndoEntry>> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            if let Some(entry) = session.state.undo_stack.pop() {
                session.state.redo_stack.push(entry.clone());
                session.last_accessed = std::time::SystemTime::now();
                Ok(Some(entry))
            } else {
                Ok(None)
            }
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn redo(&self, session_id: Uuid) -> Result<Option<UndoEntry>> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&session_id) {
            if let Some(entry) = session.state.redo_stack.pop() {
                session.state.undo_stack.push(entry.clone());
                session.last_accessed = std::time::SystemTime::now();
                Ok(Some(entry))
            } else {
                Ok(None)
            }
        } else {
            Err(CoreError::Session(format!("Session {} not found", session_id)))
        }
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read();
        sessions.values().cloned().collect()
    }

    pub fn cleanup_expired_sessions(&self) -> Result<usize> {
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(self.session_timeout_seconds);
        let mut removed_count = 0;

        let expired_sessions: Vec<Uuid> = {
            let sessions = self.sessions.read();
            sessions
                .values()
                .filter(|session| {
                    if let Some(expires_at) = session.expires_at {
                        expires_at < cutoff
                    } else {
                        session.last_accessed < cutoff
                    }
                })
                .map(|s| s.id)
                .collect()
        };

        for session_id in expired_sessions {
            if self.close_session(session_id).is_ok() {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    pub fn export_session(&self, session_id: Uuid, output_path: &str) -> Result<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| CoreError::Session(format!("Session {} not found", session_id)))?;

        let session_data = serde_json::json!({
            "session": session,
            "exported_at": std::time::SystemTime::now()
        });

        std::fs::write(output_path, serde_json::to_string_pretty(&session_data)?)?;
        Ok(())
    }

    pub fn import_session(&self, import_path: &str) -> Result<Uuid> {
        let session_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(import_path)?)?;
        let session: Session = serde_json::from_value(session_data["session"].clone())?;
        let session_id = session.id;

        {
            let mut sessions = self.sessions.write();
            sessions.insert(session_id, session);
        }

        Ok(session_id)
    }
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval_seconds: 300,
            max_history_entries: 100,
            theme: "dark".to_string(),
            language: "en".to_string(),
            workspace_layout: WorkspaceLayout::default(),
            performance_settings: PerformanceSettings::default(),
        }
    }
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self {
            panel_configurations: Vec::new(),
            dock_layout: DockLayout::default(),
            window_state: WindowState::default(),
        }
    }
}

impl Default for DockLayout {
    fn default() -> Self {
        Self {
            main_dock_area: DockArea::default(),
            floating_panels: Vec::new(),
        }
    }
}

impl Default for DockArea {
    fn default() -> Self {
        Self {
            orientation: DockOrientation::Horizontal,
            children: Vec::new(),
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            position: (100, 100),
            size: (1920, 1080),
            maximized: false,
            fullscreen: false,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: num_cpus::get(),
            gpu_acceleration: true,
            cache_size_mb: 1024,
            buffer_size_mb: 512,
            thread_pool_size: num_cpus::get(),
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_project_id: None,
            open_files: Vec::new(),
            recent_projects: Vec::new(),
            clipboard_data: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}
