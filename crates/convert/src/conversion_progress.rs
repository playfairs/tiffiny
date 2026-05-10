use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub id: String,
    pub name: String,
    pub total_steps: Arc<RwLock<usize>>,
    pub current_step: Arc<RwLock<usize>>,
    pub progress_percentage: Arc<RwLock<f32>>,
    pub status: Arc<RwLock<ProgressStatus>>,
    pub event_sender: mpsc::UnboundedSender<ProgressEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProgressEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    ProgressStarted,
    StepStarted(usize, String),
    StepProgress(usize, f32),
    StepCompleted(usize, StepResult),
    StepFailed(usize, String),
    ProgressUpdated(f32),
    ProgressCompleted(ProgressResult),
    ProgressFailed(String),
    ProgressPaused,
    ProgressResumed,
    ProgressCancelled,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: usize,
    pub step_name: String,
    pub success: bool,
    pub duration: std::time::Duration,
    pub output_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProgressResult {
    pub success: bool,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub total_duration: std::time::Duration,
    pub step_results: Vec<StepResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProgressStep {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub is_optional: bool,
    pub sub_steps: Vec<ProgressStep>,
}

#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub enable_progress_reporting: bool,
    pub update_interval: std::time::Duration,
    pub enable_step_reporting: bool,
    pub enable_eta_calculation: bool,
    pub enable_rate_calculation: bool,
    pub progress_callback: Option<ProgressCallback>,
}

#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub current_step: usize,
    pub progress_percentage: f32,
    pub elapsed_time: std::time::Duration,
    pub estimated_time_remaining: Option<std::time::Duration>,
    pub current_rate: f64,
    pub average_rate: f64,
}

pub type ProgressCallback = Arc<dyn Fn(f32, &str) + Send + Sync>;

impl ConversionProgress {
    pub fn new(id: String, name: String, total_steps: usize) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            total_steps: Arc::new(RwLock::new(total_steps))),
            current_step: Arc::new(RwLock::new(0))),
            progress_percentage: Arc::new(RwLock::new(0.0))),
            status: Arc::new(RwLock::new(ProgressStatus::NotStarted))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_steps(id: String, name: String, steps: Vec<ProgressStep>) -> Self {
        let total_steps = Self::calculate_total_steps(&steps);
        let mut progress = Self::new(id, name, total_steps);
        
Store steps internally (simplified - in real implementation would store them)
        progress
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        *status = ProgressStatus::InProgress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressStarted);
        Ok(())
    }

    pub fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        *status = ProgressStatus::Paused;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressPaused);
        Ok(())
    }

    pub fn resume(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        *status = ProgressStatus::InProgress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressResumed);
        Ok(())
    }

    pub fn cancel(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        *status = ProgressStatus::Cancelled;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressCancelled);
        Ok(())
    }

    pub fn complete(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        let mut current_step = self.current_step.write();
        let mut progress_percentage = self.progress_percentage.write();
        
        *status = ProgressStatus::Completed;
        *current_step = *self.total_steps.read();
        *progress_percentage = 100.0;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressCompleted(ProgressResult {
            success: true,
            total_steps: *self.total_steps.read(),
            completed_steps: *self.total_steps.read(),
            failed_steps: 0,
            total_duration: std::time::Duration::from_secs(0),
            step_results: Vec::new(),
            error_message: None,
        }));
        
        Ok(())
    }

    pub fn fail(&self, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut status = self.status.write();
        *status = ProgressStatus::Failed(error.to_string());
        
        let _ = self.event_sender.send(ProgressEvent::ProgressFailed(error.to_string()));
        Ok(())
    }

    pub fn start_step(&self, step_id: usize, step_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_step = self.current_step.write();
        let mut progress_percentage = self.progress_percentage.write();
        
        *current_step = step_id;
        
        let total_steps = *self.total_steps.read();
        let progress = if total_steps > 0 {
            (step_id as f32 / total_steps as f32) * 100.0
        } else {
            0.0
        };
        
        *progress_percentage = progress;
        
        let _ = self.event_sender.send(ProgressEvent::StepStarted(step_id, step_name.to_string()));
        let _ = self.event_sender.send(ProgressEvent::ProgressUpdated(progress));
        
        Ok(())
    }

    pub fn update_step_progress(&self, step_id: usize, progress: f32) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ProgressEvent::StepProgress(step_id, progress));
        
        let total_steps = *self.total_steps.read();
        let current_step = *self.current_step.read();
        
        let overall_progress = if total_steps > 0 {
            ((current_step as f32 + (progress / 100.0)) / total_steps as f32) * 100.0
        } else {
            0.0
        };
        
        let mut progress_percentage = self.progress_percentage.write();
        *progress_percentage = overall_progress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressUpdated(overall_progress));
        
        Ok(())
    }

    pub fn complete_step(&self, step_id: usize, step_name: &str, output_data: Option<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let step_result = StepResult {
            step_id,
            step_name: step_name.to_string(),
            success: true,
            duration: std::time::Duration::from_millis(0),
            output_data,
            error_message: None,
        };
        
        let _ = self.event_sender.send(ProgressEvent::StepCompleted(step_id, step_result));
        
        let mut current_step = self.current_step.write();
        let total_steps = *self.total_steps.read();
        
        if *current_step < total_steps - 1 {
            *current_step += 1;
        }
        
        Ok(())
    }

    pub fn fail_step(&self, step_id: usize, step_name: &str, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ProgressEvent::StepFailed(step_id, error.to_string()));
        
        let mut current_step = self.current_step.write();
        let total_steps = *self.total_steps.read();
        
        if *current_step < total_steps - 1 {
            *current_step += 1;
        }
        
        Ok(())
    }

    pub fn update_progress(&self, progress: f32) -> Result<(), Box<dyn std::error::Error>> {
        let progress = progress.clamp(0.0, 100.0);
        
        let mut progress_percentage = self.progress_percentage.write();
        *progress_percentage = progress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressUpdated(progress));
        
        Ok(())
    }

    pub fn increment_progress(&self, increment: f32) -> Result<(), Box<dyn std::error::Error>> {
        let mut progress_percentage = self.progress_percentage.write();
        let new_progress = (*progress_percentage + increment).clamp(0.0, 100.0);
        *progress_percentage = new_progress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressUpdated(new_progress));
        
        Ok(())
    }

    pub fn set_progress(&self, step: usize, total: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut total_steps = self.total_steps.write();
        let mut current_step = self.current_step.write();
        let mut progress_percentage = self.progress_percentage.write();
        
        *total_steps = total;
        
        if step <= total {
            *current_step = step;
        }
        
        let progress = if total > 0 {
            (step as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        
        *progress_percentage = progress;
        
        let _ = self.event_sender.send(ProgressEvent::ProgressUpdated(progress));
        
        Ok(())
    }

    pub fn get_progress(&self) -> f32 {
        *self.progress_percentage.read()
    }

    pub fn get_current_step(&self) -> usize {
        *self.current_step.read()
    }

    pub fn get_total_steps(&self) -> usize {
        *self.total_steps.read()
    }

    pub fn get_status(&self) -> ProgressStatus {
        self.status.read().clone()
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.get_status(), ProgressStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.get_status(), ProgressStatus::Failed(_))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.get_status(), ProgressStatus::Cancelled)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.get_status(), ProgressStatus::Paused)
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self.get_status(), ProgressStatus::InProgress)
    }

    pub fn get_error_message(&self) -> Option<String> {
        match self.get_status() {
            ProgressStatus::Failed(msg) => Some(msg),
            _ => None,
        }
    }

    pub fn get_stats(&self) -> ProgressStats {
        let total_steps = *self.total_steps.read();
        let current_step = *self.current_step.read();
        let progress_percentage = *self.progress_percentage.read();
        
        ProgressStats {
            total_steps,
            completed_steps: if self.is_completed() { total_steps } else { current_step },
            failed_steps: 0,
            current_step,
            progress_percentage,
            elapsed_time: std::time::Duration::from_secs(0),
            estimated_time_remaining: self.calculate_eta(progress_percentage),
            current_rate: 0.0,
            average_rate: 0.0,
        }
    }

    fn calculate_eta(&self, progress_percentage: f32) -> Option<std::time::Duration> {
        if progress_percentage <= 0.0 {
            return None;
        }
        
        let elapsed_time = std::time::Duration::from_secs(60);
        let remaining_percentage = 100.0 - progress_percentage;
        let eta_seconds = (elapsed_time.as_secs_f64() * remaining_percentage / 100.0) as u64;
        
        Some(std::time::Duration::from_secs(eta_seconds))
    }

    fn calculate_total_steps(steps: &[ProgressStep]) -> usize {
        steps.iter().fold(0, |total, step| {
            total + 1 + Self::calculate_total_steps(&step.sub_steps)
        })
    }

    pub async fn get_events(&mut self) -> Vec<ProgressEvent> {
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

    pub fn reset(&self) {
        let mut status = self.status.write();
        let mut current_step = self.current_step.write();
        let mut progress_percentage = self.progress_percentage.write();
        
        *status = ProgressStatus::NotStarted;
        *current_step = 0;
        *progress_percentage = 0.0;
    }

    pub fn clone_progress(&self) -> ConversionProgress {
        let total_steps = *self.total_steps.read();
        
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", self.name),
            total_steps,
        )
    }

    pub fn set_progress_callback(&self, callback: ProgressCallback) {
    }

    pub fn remove_progress_callback(&self) {
    }

    pub fn enable_eta_calculation(&self, enable: bool) {
    }

    pub fn enable_rate_calculation(&self, enable: bool) {
    }

    pub fn set_update_interval(&self, interval: std::time::Duration) {
    }

    pub fn get_step_progress(&self, step_id: usize) -> f32 {
        0.0
    }

    pub fn get_step_status(&self, step_id: usize) -> Option<ProgressStatus> {
        None
    }

    pub fn add_step(&self, step: ProgressStep) -> Result<(), Box<dyn std::error::Error>> {
        let mut total_steps = self.total_steps.write();
        *total_steps += 1 + Self::calculate_total_steps(&step.sub_steps);
        Ok(())
    }

    pub fn remove_step(&self, step_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn get_step_count(&self) -> usize {
        *self.total_steps.read()
    }

    pub fn get_completed_step_count(&self) -> usize {
        if self.is_completed() {
            *self.total_steps.read()
        } else {
            *self.current_step.read()
        }
    }

    pub fn get_remaining_step_count(&self) -> usize {
        let total_steps = *self.total_steps.read();
        let current_step = *self.current_step.read();
        
        if current_step < total_steps {
            total_steps - current_step
        } else {
            0
        }
    }

    pub fn get_progress_ratio(&self) -> f64 {
        let total_steps = *self.total_steps.read();
        let current_step = *self.current_step.read();
        
        if total_steps == 0 {
            0.0
        } else {
            current_step as f64 / total_steps as f64
        }
    }

    pub fn get_percentage_string(&self) -> String {
        format!("{:.1}%", self.get_progress())
    }

    pub fn get_time_remaining_string(&self) -> Option<String> {
        self.calculate_eta(self.get_progress()).map(|eta| {
            let total_seconds = eta.as_secs();
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            
            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            } else if minutes > 0 {
                format!("{:02}:{:02}", minutes, seconds)
            } else {
                format!("{:02}s", seconds)
            }
        })
    }

    pub fn get_rate_string(&self) -> Option<String> {
        let stats = self.get_stats();
        if stats.current_rate > 0.0 {
            Some(format!("{:.2} steps/sec", stats.current_rate))
        } else {
            None
        }
    }

    pub fn create_sub_progress(&self, parent_step_id: usize, sub_steps: usize) -> ConversionProgress {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Sub-Progress", self.name),
            sub_steps,
        )
    }

    pub fn merge_progress(&self, other: &ConversionProgress) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn export_progress(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let stats = self.get_stats();
        
        Ok(serde_json::json!({
            "id": self.id,
            "name": self.name,
            "total_steps": stats.total_steps,
            "current_step": stats.current_step,
            "progress_percentage": stats.progress_percentage,
            "status": format!("{:?}", self.get_status()),
            "elapsed_time": format!("{:?}", stats.elapsed_time),
            "estimated_time_remaining": stats.estimated_time_remaining.map(|d| format!("{:?}", d)),
            "current_rate": stats.current_rate,
            "average_rate": stats.average_rate,
        }))
    }

    pub fn import_progress(&self, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Progress".to_string(),
            100,
        )
    }
}

impl Default for ProgressStatus {
    fn default() -> Self {
        ProgressStatus::NotStarted
    }
}

impl Default for ProgressEvent {
    fn default() -> Self {
        ProgressEvent::ProgressStarted
    }
}

impl Default for StepResult {
    fn default() -> Self {
        Self {
            step_id: 0,
            step_name: String::new(),
            success: false,
            duration: std::time::Duration::from_millis(0),
            output_data: None,
            error_message: None,
        }
    }
}

impl Default for ProgressResult {
    fn default() -> Self {
        Self {
            success: false,
            total_steps: 0,
            completed_steps: 0,
            failed_steps: 0,
            total_duration: std::time::Duration::from_millis(0),
            step_results: Vec::new(),
            error_message: None,
        }
    }
}

impl Default for ProgressStep {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            weight: 1.0,
            is_optional: false,
            sub_steps: Vec::new(),
        }
    }
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            enable_progress_reporting: true,
            update_interval: std::time::Duration::from_millis(100),
            enable_step_reporting: true,
            enable_eta_calculation: true,
            enable_rate_calculation: true,
            progress_callback: None,
        }
    }
}

impl Default for ProgressStats {
    fn default() -> Self {
        Self {
            total_steps: 0,
            completed_steps: 0,
            failed_steps: 0,
            current_step: 0,
            progress_percentage: 0.0,
            elapsed_time: std::time::Duration::from_secs(0),
            estimated_time_remaining: None,
            current_rate: 0.0,
            average_rate: 0.0,
        }
    }
}
