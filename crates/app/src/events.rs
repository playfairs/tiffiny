use tiffiny_core::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    NewProject,
    OpenProject(String),
    SaveProject,
    SaveProjectAs(String),
    CloseProject,
    ImportFile(String),
    ExportFile(String, String),
    FileImported { path: String, asset_id: Uuid },
    FileExported { path: String, format: String, export_id: Uuid },
    
    ProjectCreated { project_id: Uuid },
    ProjectOpened { project_id: Uuid },
    ProjectSaved { project_id: Uuid },
    ProjectClosed { project_id: Uuid },
    
    PipelineCreated { pipeline_id: Uuid },
    PipelineDeleted { pipeline_id: Uuid },
    PipelineExecutionStarted { pipeline_id: Uuid, execution_id: Uuid },
    PipelineExecutionCompleted { pipeline_id: Uuid, execution_id: Uuid },
    
    GraphCreated { graph_id: Uuid },
    GraphDeleted { graph_id: Uuid },
    GraphExecutionStarted { graph_id: Uuid, execution_id: Uuid },
    GraphExecutionCompleted { graph_id: Uuid, execution_id: Uuid },
    
    TaskCompleted { task_id: Uuid, result: Option<serde_json::Value> },
    TaskFailed { task_id: Uuid, error: String },
    
    AssetAdded { asset_id: Uuid },
    AssetRemoved { asset_id: Uuid },
    AssetModified { asset_id: Uuid },
    
    UiEvent(UiEvent),
    SettingsChanged(String, serde_json::Value),
    
    Error(String),
    Warning(String),
    Info(String),
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    WindowResized(u32, u32),
    WindowMoved(i32, i32),
    WindowFocused,
    WindowUnfocused,
    WindowMinimized,
    WindowRestored,
    
    PanelResized(String, u32, u32),
    PanelMoved(String, i32, i32),
    PanelClosed(String),
    PanelOpened(String),
    
    KeyPressed(String, bool, bool, bool),
    KeyReleased(String, bool, bool, bool),
    MouseMoved(f32, f32),
    MousePressed(u32, f32, f32),
    MouseReleased(u32, f32, f32),
    MouseScrolled(f32, f32),
    
    DragStart(String, f32, f32),
    DragMove(f32, f32),
    DragEnd(f32, f32),
    Drop(String, f32, f32),
    
    MenuItemClicked(String),
    ButtonClicked(String),
    TextChanged(String, String),
    SelectionChanged(String, String),
    
    ThemeChanged(String),
    LanguageChanged(String),
    FontSizeChanged(f32),
}

pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
    subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::Sender<AppEvent>>>>>,
    event_history: Arc<RwLock<Vec<AppEvent>>>,
    max_history_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    pub async fn publish(&self, event: AppEvent) -> Result<()> {
        
        {
            let mut subscribers = self.subscribers.write();
            for (subscriber_id, senders) in subscribers.iter_mut() {
                senders.retain(|sender| {
                    if let Err(_) = sender.try_send(event.clone()) {
                        false
                    } else {
                        true
                    }
                });
            }
        }
        
        {
            let mut history = self.event_history.write();
            history.push(event);
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }
        
        Ok(())
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub async fn subscribe_to_type(&self, subscriber_id: String) -> mpsc::Receiver<AppEvent> {
        let (sender, receiver) = mpsc::channel(100);
        
        let mut subscribers = self.subscribers.write();
        subscribers.entry(subscriber_id).or_insert_with(Vec::new).push(sender);
        
        receiver
    }

    pub async fn subscribe_quit(&self) -> broadcast::Receiver<AppEvent> {
        let receiver = self.sender.subscribe();
        receiver
    }

    pub fn get_event_history(&self) -> Vec<AppEvent> {
        let history = self.event_history.read();
        history.clone()
    }

    pub fn get_events_by_type(&self, event_type: &str) -> Vec<AppEvent> {
        let history = self.event_history.read();
        history.iter()
            .filter(|event| matches!(event, AppEvent::UiEvent(_)) && event_type == "ui" ||
                              matches!(event, AppEvent::Error(_)) && event_type == "error" ||
                              matches!(event, AppEvent::Warning(_)) && event_type == "warning" ||
                              matches!(event, AppEvent::Info(_)) && event_type == "info")
            .cloned()
            .collect()
    }

    pub fn clear_history(&self) {
        let mut history = self.event_history.write();
        history.clear();
    }

    pub fn get_pending_events(&self) -> Result<Vec<AppEvent>> {
        let history = self.event_history.read();
        Ok(history.clone())
    }

    pub fn get_subscriber_count(&self) -> usize {
        let subscribers = self.subscribers.read();
        subscribers.values().map(|senders| senders.len()).sum()
    }

    pub fn unsubscribe(&self, subscriber_id: &str) {
        let mut subscribers = self.subscribers.write();
        subscribers.remove(subscriber_id);
    }

    pub async fn wait_for_event<F>(&self, predicate: F) -> AppEvent 
    where 
        F: Fn(&AppEvent) -> bool,
    {
        let mut receiver = self.subscribe().await;
        
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if predicate(&event) {
                        return event;
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
        
        AppEvent::Error("Event stream ended".to_string())
    }

    pub async fn wait_for_quit(&self) -> AppEvent {
        self.wait_for_event(|event| matches!(event, AppEvent::Quit)).await
    }

    pub async fn wait_for_project_change(&self) -> AppEvent {
        self.wait_for_event(|event| {
            matches!(event, 
                AppEvent::ProjectCreated { .. } |
                AppEvent::ProjectOpened { .. } |
                AppEvent::ProjectSaved { .. } |
                AppEvent::ProjectClosed { .. })
        }).await
    }

    pub async fn wait_for_task_completion(&self, task_id: Uuid) -> AppEvent {
        self.wait_for_event(|event| {
            matches!(event, 
                AppEvent::TaskCompleted { task_id: id, .. } |
                AppEvent::TaskFailed { task_id: id, .. } if *id == task_id)
        }).await
    }

    pub fn get_statistics(&self) -> EventStatistics {
        let history = self.event_history.read();
        let mut stats = EventStatistics::default();
        
        for event in history.iter() {
            stats.total_events += 1;
            
            match event {
                AppEvent::Quit => stats.quit_events += 1,
                AppEvent::NewProject => stats.project_events += 1,
                AppEvent::OpenProject(_) => stats.project_events += 1,
                AppEvent::SaveProject => stats.project_events += 1,
                AppEvent::ImportFile(_) => stats.file_events += 1,
                AppEvent::ExportFile(_, _) => stats.file_events += 1,
                AppEvent::Error(_) => stats.error_events += 1,
                AppEvent::Warning(_) => stats.warning_events += 1,
                AppEvent::Info(_) => stats.info_events += 1,
                AppEvent::UiEvent(_) => stats.ui_events += 1,
                AppEvent::TaskCompleted { .. } => stats.task_events += 1,
                AppEvent::TaskFailed { .. } => stats.task_events += 1,
                _ => {}
            }
        }
        
        stats
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventStatistics {
    pub total_events: usize,
    pub quit_events: usize,
    pub project_events: usize,
    pub file_events: usize,
    pub task_events: usize,
    pub ui_events: usize,
    pub error_events: usize,
    pub warning_events: usize,
    pub info_events: usize,
}

pub struct EventFilter {
    pub event_types: Vec<String>,
    pub source_filters: Vec<String>,
    pub time_range: Option<(std::time::SystemTime, std::time::SystemTime)>,
    pub custom_filter: Option<Box<dyn Fn(&AppEvent) -> bool + Send + Sync>>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            event_types: Vec::new(),
            source_filters: Vec::new(),
            time_range: None,
            custom_filter: None,
        }
    }

    pub fn with_event_type(mut self, event_type: String) -> Self {
        self.event_types.push(event_type);
        self
    }

    pub fn with_source_filter(mut self, source: String) -> Self {
        self.source_filters.push(source);
        self
    }

    pub fn with_time_range(mut self, start: std::time::SystemTime, end: std::time::SystemTime) -> Self {
        self.time_range = Some((start, end));
        self
    }

    pub fn with_custom_filter<F>(mut self, filter: F) -> Self 
    where 
        F: Fn(&AppEvent) -> bool + Send + Sync + 'static,
    {
        self.custom_filter = Some(Box::new(filter));
        self
    }

    pub fn matches(&self, event: &AppEvent) -> bool {
        if !self.event_types.is_empty() {
            let event_type = match event {
                AppEvent::Quit => "quit",
                AppEvent::NewProject => "project",
                AppEvent::OpenProject(_) => "project",
                AppEvent::SaveProject => "project",
                AppEvent::ImportFile(_) => "file",
                AppEvent::ExportFile(_, _) => "file",
                AppEvent::Error(_) => "error",
                AppEvent::Warning(_) => "warning",
                AppEvent::Info(_) => "info",
                AppEvent::UiEvent(_) => "ui",
                AppEvent::TaskCompleted { .. } => "task",
                AppEvent::TaskFailed { .. } => "task",
                _ => "other",
            };
            
            if !self.event_types.contains(&event_type.to_string()) {
                return false;
            }
        }

        if let Some((start, end)) = self.time_range {
            let event_time = std::time::SystemTime::now();
            if event_time < start || event_time > end {
                return false;
            }
        }

        if let Some(ref custom_filter) = self.custom_filter {
            if !custom_filter(event) {
                return false;
            }
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}
