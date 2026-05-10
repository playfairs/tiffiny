use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub requirements: TaskRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    AudioDecode,
    AudioEncode,
    ImageDecode,
    ImageEncode,
    VideoDecode,
    VideoEncode,
    RawReinterpret,
    DataBend,
    Glitch,
    FFT,
    Waveform,
    Spectrogram,
    ColorSpace,
    Filter,
    Transform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub memory_mb: u64,
    pub cpu_cores: u8,
    pub gpu_required: bool,
    pub temp_storage_mb: u64,
    pub estimated_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub progress: f32,
    pub started_at: Option<std::time::SystemTime>,
    pub completed_at: Option<std::time::SystemTime>,
    pub result: Option<TaskResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output_data: Option<serde_json::Value>,
    pub output_files: Vec<String>,
    pub metrics: TaskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub duration_ms: u64,
    pub memory_used_mb: u64,
    pub cpu_time_ms: u64,
    pub gpu_time_ms: Option<u64>,
}

pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<Uuid, TaskDefinition>>>,
    executions: Arc<RwLock<HashMap<Uuid, TaskExecution>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_task(&self, task: TaskDefinition) -> Result<()> {
        let mut tasks = self.tasks.write();
        tasks.insert(task.id, task);
        Ok(())
    }

    pub fn create_execution(&self, task_id: Uuid) -> Result<Uuid> {
        let execution_id = Uuid::new_v4();
        let execution = TaskExecution {
            id: execution_id,
            task_id,
            status: TaskStatus::Pending,
            progress: 0.0,
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        };

        let mut executions = self.executions.write();
        executions.insert(execution_id, execution);
        Ok(execution_id)
    }

    pub fn start_execution(&self, execution_id: Uuid) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = TaskStatus::Running;
            execution.started_at = Some(std::time::SystemTime::now());
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }

    pub fn update_progress(&self, execution_id: Uuid, progress: f32) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.progress = progress.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }

    pub fn complete_execution(&self, execution_id: Uuid, result: TaskResult) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = TaskStatus::Completed;
            execution.completed_at = Some(std::time::SystemTime::now());
            execution.result = Some(result);
            execution.progress = 1.0;
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }

    pub fn fail_execution(&self, execution_id: Uuid, error: String) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = TaskStatus::Failed;
            execution.completed_at = Some(std::time::SystemTime::now());
            execution.error = Some(error);
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }

    pub fn cancel_execution(&self, execution_id: Uuid) -> Result<()> {
        let mut executions = self.executions.write();
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = TaskStatus::Cancelled;
            execution.completed_at = Some(std::time::SystemTime::now());
            Ok(())
        } else {
            Err(CoreError::Task(format!("Execution {} not found", execution_id)))
        }
    }

    pub fn get_execution(&self, execution_id: Uuid) -> Option<TaskExecution> {
        let executions = self.executions.read();
        executions.get(&execution_id).cloned()
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<TaskDefinition> {
        let tasks = self.tasks.read();
        tasks.get(&task_id).cloned()
    }

    pub fn list_executions(&self) -> Vec<TaskExecution> {
        let executions = self.executions.read();
        executions.values().cloned().collect()
    }

    pub fn list_tasks(&self) -> Vec<TaskDefinition> {
        let tasks = self.tasks.read();
        tasks.values().cloned().collect()
    }

    pub fn get_executions_by_status(&self, status: TaskStatus) -> Vec<TaskExecution> {
        let executions = self.executions.read();
        executions
            .values()
            .filter(|e| e.status == status)
            .cloned()
            .collect()
    }

    pub fn cleanup_completed_executions(&self, older_than: std::time::Duration) -> Result<usize> {
        let mut executions = self.executions.write();
        let cutoff = std::time::SystemTime::now() - older_than;
        let initial_count = executions.len();

        executions.retain(|_, execution| {
            if let Some(completed_at) = execution.completed_at {
                completed_at > cutoff
            } else {
                true
            }
        });

        Ok(initial_count - executions.len())
    }
}
