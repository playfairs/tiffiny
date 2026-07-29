use crate::prelude::*;
use parking_lot::RwLock;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub job_type: JobType,
  pub tasks: Vec<JobTask>,
  pub dependencies: Vec<Uuid>,
  pub priority: JobPriority,
  pub created_at: std::time::SystemTime,
  pub scheduled_at: Option<std::time::SystemTime>,
  pub started_at: Option<std::time::SystemTime>,
  pub completed_at: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
  BatchConversion,
  DataBending,
  EffectProcessing,
  MediaAnalysis,
  Export,
  Import,
  Cleanup,
  Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTask {
  pub id: Uuid,
  pub name: String,
  pub task_type: String,
  pub parameters: HashMap<String, serde_json::Value>,
  pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
  pub job_id: Uuid,
  pub status: JobStatus,
  pub progress: f32,
  pub current_task: Option<Uuid>,
  pub completed_tasks: Vec<Uuid>,
  pub failed_tasks: Vec<Uuid>,
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
  Pending,
  Scheduled,
  Running,
  Paused,
  Completed,
  Failed,
  Cancelled,
}

pub struct JobManager {
  jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
  executions: Arc<RwLock<HashMap<Uuid, JobExecution>>>,
}

impl JobManager {
  pub fn new() -> Self {
    Self {
      jobs: Arc::new(RwLock::new(HashMap::new())),
      executions: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn create_job(&self, job: Job) -> Result<()> {
    let mut jobs = self.jobs.write();
    jobs.insert(job.id, job.clone());

    let execution = JobExecution {
      job_id: job.id,
      status: JobStatus::Pending,
      progress: 0.0,
      current_task: None,
      completed_tasks: Vec::new(),
      failed_tasks: Vec::new(),
      error: None,
    };

    let mut executions = self.executions.write();
    executions.insert(job.id, execution);

    Ok(())
  }

  pub fn schedule_job(&self, job_id: Uuid, scheduled_at: std::time::SystemTime) -> Result<()> {
    {
      let mut jobs = self.jobs.write();
      if let Some(job) = jobs.get_mut(&job_id) {
        job.scheduled_at = Some(scheduled_at);
      } else {
        return Err(CoreError::Task(format!("Job {} not found", job_id)));
      }
    }

    {
      let mut executions = self.executions.write();
      if let Some(execution) = executions.get_mut(&job_id) {
        execution.status = JobStatus::Scheduled;
      }
    }

    Ok(())
  }

  pub fn start_job(&self, job_id: Uuid) -> Result<()> {
    let now = std::time::SystemTime::now();

    {
      let mut jobs = self.jobs.write();
      if let Some(job) = jobs.get_mut(&job_id) {
        job.started_at = Some(now);
      } else {
        return Err(CoreError::Task(format!("Job {} not found", job_id)));
      }
    }

    {
      let mut executions = self.executions.write();
      if let Some(execution) = executions.get_mut(&job_id) {
        execution.status = JobStatus::Running;
      }
    }

    Ok(())
  }

  pub fn complete_job(&self, job_id: Uuid) -> Result<()> {
    let now = std::time::SystemTime::now();

    {
      let mut jobs = self.jobs.write();
      if let Some(job) = jobs.get_mut(&job_id) {
        job.completed_at = Some(now);
      } else {
        return Err(CoreError::Task(format!("Job {} not found", job_id)));
      }
    }

    {
      let mut executions = self.executions.write();
      if let Some(execution) = executions.get_mut(&job_id) {
        execution.status = JobStatus::Completed;
        execution.progress = 1.0;
      }
    }

    Ok(())
  }

  pub fn fail_job(&self, job_id: Uuid, error: String) -> Result<()> {
    let now = std::time::SystemTime::now();

    {
      let mut jobs = self.jobs.write();
      if let Some(job) = jobs.get_mut(&job_id) {
        job.completed_at = Some(now);
      } else {
        return Err(CoreError::Task(format!("Job {} not found", job_id)));
      }
    }

    {
      let mut executions = self.executions.write();
      if let Some(execution) = executions.get_mut(&job_id) {
        execution.status = JobStatus::Failed;
        execution.error = Some(error);
      }
    }

    Ok(())
  }

  pub fn update_job_progress(
    &self,
    job_id: Uuid,
    progress: f32,
    current_task: Option<Uuid>,
  ) -> Result<()> {
    let mut executions = self.executions.write();
    if let Some(execution) = executions.get_mut(&job_id) {
      execution.progress = progress.clamp(0.0, 1.0);
      execution.current_task = current_task;
      Ok(())
    } else {
      Err(CoreError::Task(format!(
        "Job execution {} not found",
        job_id
      )))
    }
  }

  pub fn complete_task(&self, job_id: Uuid, task_id: Uuid) -> Result<()> {
    let mut executions = self.executions.write();
    if let Some(execution) = executions.get_mut(&job_id) {
      execution.completed_tasks.push(task_id);

      let job = {
        let jobs = self.jobs.read();
        jobs.get(&job_id).cloned()
      };

      if let Some(job) = job {
        let total_tasks = job.tasks.len();
        let completed_tasks = execution.completed_tasks.len();
        let new_progress = completed_tasks as f32 / total_tasks as f32;
        execution.progress = new_progress;
      }

      Ok(())
    } else {
      Err(CoreError::Task(format!(
        "Job execution {} not found",
        job_id
      )))
    }
  }

  pub fn fail_task(&self, job_id: Uuid, task_id: Uuid) -> Result<()> {
    let mut executions = self.executions.write();
    if let Some(execution) = executions.get_mut(&job_id) {
      execution.failed_tasks.push(task_id);
      Ok(())
    } else {
      Err(CoreError::Task(format!(
        "Job execution {} not found",
        job_id
      )))
    }
  }

  pub fn get_job(&self, job_id: Uuid) -> Option<Job> {
    let jobs = self.jobs.read();
    jobs.get(&job_id).cloned()
  }

  pub fn get_job_execution(&self, job_id: Uuid) -> Option<JobExecution> {
    let executions = self.executions.read();
    executions.get(&job_id).cloned()
  }

  pub fn list_jobs(&self) -> Vec<Job> {
    let jobs = self.jobs.read();
    jobs.values().cloned().collect()
  }

  pub fn list_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
    let executions = self.executions.read();
    let jobs = self.jobs.read();

    executions
      .values()
      .filter(|e| e.status == status)
      .filter_map(|e| jobs.get(&e.job_id).cloned())
      .collect()
  }

  pub fn get_jobs_by_priority(&self, priority: JobPriority) -> Vec<Job> {
    let jobs = self.jobs.read();
    jobs
      .values()
      .filter(|j| j.priority == priority)
      .cloned()
      .collect()
  }

  pub fn cancel_job(&self, job_id: Uuid) -> Result<()> {
    let now = std::time::SystemTime::now();

    {
      let mut jobs = self.jobs.write();
      if let Some(job) = jobs.get_mut(&job_id) {
        job.completed_at = Some(now);
      } else {
        return Err(CoreError::Task(format!("Job {} not found", job_id)));
      }
    }

    {
      let mut executions = self.executions.write();
      if let Some(execution) = executions.get_mut(&job_id) {
        execution.status = JobStatus::Cancelled;
      }
    }

    Ok(())
  }

  pub fn cleanup_completed_jobs(&self, older_than: std::time::Duration) -> Result<usize> {
    let cutoff = std::time::SystemTime::now() - older_than;
    let mut removed_count = 0;

    {
      let mut jobs = self.jobs.write();
      let initial_count = jobs.len();

      jobs.retain(|_, job| {
        if let Some(completed_at) = job.completed_at {
          completed_at > cutoff
        } else {
          true
        }
      });

      removed_count += initial_count - jobs.len();
    }

    {
      let mut executions = self.executions.write();
      let initial_count = executions.len();

      executions.retain(|_, execution| {
        if let Some(job) = self.jobs.read().get(&execution.job_id) {
          if let Some(completed_at) = job.completed_at {
            completed_at > cutoff
          } else {
            true
          }
        } else {
          false
        }
      });

      removed_count += initial_count - executions.len();
    }

    Ok(removed_count)
  }
}
