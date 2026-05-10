use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub stages: Vec<PipelineStage>,
    pub status: PipelineStatus,
    pub settings: PipelineSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: Uuid,
    pub name: String,
    pub stage_type: StageType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageType {
    Input,
    Processing,
    Output,
    Filter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSettings {
    pub max_concurrent_stages: usize,
    pub timeout_seconds: u64,
    pub retry_count: usize,
    pub error_handling: ErrorHandlingStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandlingStrategy {
    StopOnError,
    ContinueOnError,
    RetryOnError,
}

impl Pipeline {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            stages: Vec::new(),
            status: PipelineStatus::Pending,
            settings: PipelineSettings::default(),
        }
    }

    pub fn add_stage(&mut self, stage: PipelineStage) -> Result<()> {
        self.stages.push(stage);
        Ok(())
    }

    pub fn remove_stage(&mut self, stage_id: &Uuid) -> Result<()> {
        self.stages.retain(|s| s.id != *stage_id);
        Ok(())
    }

    pub fn get_stage(&self, stage_id: &Uuid) -> Option<&PipelineStage> {
        self.stages.iter().find(|s| s.id == *stage_id)
    }

    pub fn start(&mut self) -> Result<()> {
        self.status = PipelineStatus::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.status = PipelineStatus::Completed;
        Ok(())
    }

    pub fn get_status(&self) -> &PipelineStatus {
        &self.status
    }
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            max_concurrent_stages: 4,
            timeout_seconds: 300,
            retry_count: 3,
            error_handling: ErrorHandlingStrategy::StopOnError,
        }
    }
}
