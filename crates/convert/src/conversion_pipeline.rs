use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ConversionPipeline {
    pub id: String,
    pub name: String,
    pub stages: Arc<RwLock<Vec<ConversionStage>>>,
    pub current_stage: Arc<RwLock<usize>>,
    pub status: Arc<RwLock<PipelineStatus>>,
    pub event_sender: mpsc::UnboundedSender<PipelineEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PipelineEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    StageStarted(usize),
    StageCompleted(usize, StageResult),
    StageFailed(usize, String),
    PipelineStarted,
    PipelineCompleted(PipelineResult),
    PipelineFailed(String),
    PipelineCancelled,
    Progress(f32),
}

#[derive(Debug, Clone)]
pub struct ConversionStage {
    pub id: String,
    pub name: String,
    pub stage_type: StageType,
    pub processor: Arc<dyn StageProcessor + Send + Sync>,
    pub config: Arc<RwLock<super::conversion_config::ConversionConfig>>,
    pub status: Arc<RwLock<StageStatus>>,
    pub progress: Arc<RwLock<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageType {
    Input,
    Preprocessing,
    Processing,
    Postprocessing,
    Output,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageStatus {
    Idle,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_id: String,
    pub success: bool,
    pub output_data: Option<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub success: bool,
    pub stage_results: Vec<StageResult>,
    pub total_processing_time: std::time::Duration,
    pub output_data: Option<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub pipeline_id: String,
    pub pipeline_name: String,
    pub stages: Vec<StageConfig>,
    pub parallel_execution: bool,
    pub max_concurrent_stages: Option<usize>,
    pub error_handling: ErrorHandling,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandling {
    StopOnError,
    SkipOnError,
    RetryOnError,
    ContinueOnError,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay: std::time::Duration,
    pub backoff_multiplier: f32,
    pub retry_on_timeout: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_stages: usize,
    pub completed_stages: usize,
    pub failed_stages: usize,
    pub skipped_stages: usize,
    pub total_processing_time: std::time::Duration,
    pub average_stage_time: std::time::Duration,
    pub throughput: f64,
}

#[async_trait::async_trait]
pub trait StageProcessor: Send + Sync {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>>;
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn get_name(&self) -> &str;
    fn get_stage_type(&self) -> StageType;
}

impl ConversionPipeline {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stages: Arc::new(RwLock::new(Vec::new())),
            current_stage: Arc::new(RwLock::new(0))),
            status: Arc::new(RwLock::new(PipelineStatus::Idle))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_config(config: PipelineConfig) -> Self {
        let mut pipeline = Self::new(config.pipeline_id.clone(), config.pipeline_name.clone());
        
Add stages from config
        for stage_config in config.stages {
            let stage = ConversionStage {
                id: stage_config.id.clone(),
                name: stage_config.name.clone(),
                stage_type: stage_config.stage_type.clone(),
                processor: Arc::new(create_stage_processor(&stage_config)),
                config: Arc::new(RwLock::new(stage_config.config.clone())),
                status: Arc::new(RwLock::new(StageStatus::Idle)),
                progress: Arc::new(RwLock::new(0.0)),
            };
            
            let mut stages = pipeline.stages.write();
            stages.push(stage);
        }
        
        pipeline
    }

    pub async fn execute(&self, input_data: &[u8]) -> Result<PipelineResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(PipelineEvent::PipelineStarted);
        let start_time = std::time::Instant::now();

        let mut status = self.status.write();
        *status = PipelineStatus::Running;

        let stages = self.stages.read();
        let mut stage_results = Vec::new();
        let mut current_data = input_data.to_vec();

        for (stage_index, stage) in stages.iter().enumerate() {
            let mut current_stage = self.current_stage.write();
            *current_stage = stage_index;

            let _ = self.event_sender.send(PipelineEvent::StageStarted(stage_index));

            let mut stage_status = stage.status.write();
            *stage_status = StageStatus::Running;

            let stage_config = stage.config.read();
            let result = stage.processor.process(&current_data, &stage_config).await;

            match result {
                Ok(stage_result) => {
                    let _ = self.event_sender.send(PipelineEvent::StageCompleted(stage_index, stage_result.clone()));
                    
                    *stage_status = StageStatus::Completed;
                    
                    let mut progress = stage.progress.write();
                    *progress = 100.0;
                    
                    let pipeline_progress = ((stage_index + 1) as f32 / stages.len() as f32) * 100.0;
                    let _ = self.event_sender.send(PipelineEvent::Progress(pipeline_progress));
                    
                    stage_results.push(stage_result);
                    
                    if let Some(output_data) = stage_result.output_data {
                        current_data = output_data;
                    }
                },
                Err(e) => {
                    let error_msg = format!("Stage {} failed: {}", stage.name, e);
                    let _ = self.event_sender.send(PipelineEvent::StageFailed(stage_index, error_msg.clone()));
                    
                    *stage_status = StageStatus::Failed(error_msg);
                    
                    let pipeline_config = self.get_pipeline_config();
                    match pipeline_config.error_handling {
                        ErrorHandling::StopOnError => {
                            let mut status = self.status.write();
                            *status = PipelineStatus::Failed(error_msg);
                            
                            let _ = self.event_sender.send(PipelineEvent::PipelineFailed(error_msg.clone()));
                            return Ok(PipelineResult {
                                pipeline_id: self.id.clone(),
                                success: false,
                                stage_results,
                                total_processing_time: start_time.elapsed(),
                                output_data: None,
                                metadata: std::collections::HashMap::new(),
                                error_message: Some(error_msg),
                            });
                        },
                        ErrorHandling::SkipOnError => {
                            *stage_status = StageStatus::Skipped;
                            
                            let skipped_result = StageResult {
                                stage_id: stage.id.clone(),
                                success: false,
                                output_data: Some(current_data.clone()),
                                metadata: std::collections::HashMap::new(),
                                processing_time: std::time::Duration::from_millis(0),
                                error_message: Some(error_msg),
                            };
                            
                            stage_results.push(skipped_result);
                            continue;
                        },
                        ErrorHandling::RetryOnError => {
                            let retry_count = 0;
                            let max_retries = pipeline_config.retry_config.max_retries;
                            
                            while retry_count < max_retries {
                                let retry_delay = pipeline_config.retry_config.retry_delay;
                                tokio::time::sleep(retry_delay).await;
                                
                                match stage.processor.process(&current_data, &stage_config).await {
                                    Ok(retry_result) => {
                                        let _ = self.event_sender.send(PipelineEvent::StageCompleted(stage_index, retry_result.clone()));
                                        
                                        *stage_status = StageStatus::Completed;
                                        let mut progress = stage.progress.write();
                                        *progress = 100.0;
                                        
                                        stage_results.push(retry_result);
                                        
                                        if let Some(output_data) = retry_result.output_data {
                                            current_data = output_data;
                                        }
                                        break;
                                    },
                                    Err(_) => {
                                        retry_count += 1;
                                    }
                                }
                            }
                            
                            if retry_count >= max_retries {
                                let final_error = format!("Stage {} failed after {} retries", stage.name, max_retries);
                                let _ = self.event_sender.send(PipelineEvent::StageFailed(stage_index, final_error.clone()));
                                
                                *stage_status = StageStatus::Failed(final_error);
                                
                                let failed_result = StageResult {
                                    stage_id: stage.id.clone(),
                                    success: false,
                                    output_data: Some(current_data.clone()),
                                    metadata: std::collections::HashMap::new(),
                                    processing_time: std::time::Duration::from_millis(0),
                                    error_message: Some(final_error),
                                };
                                
                                stage_results.push(failed_result);
                                continue;
                            }
                        },
                        ErrorHandling::ContinueOnError => {
                            *stage_status = StageStatus::Completed;
                            
                            let error_result = StageResult {
                                stage_id: stage.id.clone(),
                                success: false,
                                output_data: Some(current_data.clone()),
                                metadata: std::collections::HashMap::new(),
                                processing_time: std::time::Duration::from_millis(0),
                                error_message: Some(error_msg),
                            };
                            
                            stage_results.push(error_result);
                            continue;
                        },
                    }
                },
            }
        }

        let mut status = self.status.write();
        *status = PipelineStatus::Completed;

        let total_processing_time = start_time.elapsed();
        let _ = self.event_sender.send(PipelineEvent::PipelineCompleted(PipelineResult {
            pipeline_id: self.id.clone(),
            success: true,
            stage_results,
            total_processing_time,
            output_data: Some(current_data),
            metadata: std::collections::HashMap::new(),
            error_message: None,
        }));

        Ok(PipelineResult {
            pipeline_id: self.id.clone(),
            success: true,
            stage_results,
            total_processing_time,
            output_data: Some(current_data),
            metadata: std::collections::HashMap::new(),
            error_message: None,
        })
    }

    pub async fn execute_parallel(&self, input_data: &[u8]) -> Result<PipelineResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(PipelineEvent::PipelineStarted);
        let start_time = std::time::Instant::now();

        let mut status = self.status.write();
        *status = PipelineStatus::Running;

        let stages = self.stages.read();
        let mut stage_results = Vec::new();

        let mut tasks = Vec::new();
        let mut stage_data = input_data.to_vec();

        for (stage_index, stage) in stages.iter().enumerate() {
            let stage_clone = stage.clone();
            let data_clone = stage_data.clone();
            let config_clone = stage.config.read().clone();

            let task = tokio::spawn(async move {
                let _ = stage_clone.event_sender.send(PipelineEvent::StageStarted(stage_index));
                
                let mut stage_status = stage_clone.status.write();
                *stage_status = StageStatus::Running;

                let result = stage_clone.processor.process(&data_clone, &config_clone).await;

                match result {
                    Ok(stage_result) => {
                        let _ = stage_clone.event_sender.send(PipelineEvent::StageCompleted(stage_index, stage_result.clone()));
                        
                        *stage_status = StageStatus::Completed;
                        let mut progress = stage_clone.progress.write();
                        *progress = 100.0;
                        
                        Ok(stage_result)
                    },
                    Err(e) => {
                        let error_msg = format!("Stage {} failed: {}", stage_clone.name, e);
                        let _ = stage_clone.event_sender.send(PipelineEvent::StageFailed(stage_index, error_msg.clone()));
                        
                        *stage_status = StageStatus::Failed(error_msg);
                        
                        Err(e)
                    },
                }
            });

            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    let error_msg = format!("Parallel stage execution failed: {}", e);
                    let _ = self.event_sender.send(PipelineEvent::PipelineFailed(error_msg));
                    
                    let mut status = self.status.write();
                    *status = PipelineStatus::Failed(error_msg);
                    
                    return Ok(PipelineResult {
                        pipeline_id: self.id.clone(),
                        success: false,
                        stage_results,
                        total_processing_time: start_time.elapsed(),
                        output_data: None,
                        metadata: std::collections::HashMap::new(),
                        error_message: Some(error_msg),
                    });
                },
            }
        }

        let final_data = results.into_iter().find_map(|r| {
            if r.success {
                r.output_data
            } else {
                None
            }
        }).unwrap_or_else(|| input_data.to_vec());

        let mut status = self.status.write();
        *status = PipelineStatus::Completed;

        let total_processing_time = start_time.elapsed();
        let _ = self.event_sender.send(PipelineEvent::PipelineCompleted(PipelineResult {
            pipeline_id: self.id.clone(),
            success: true,
            stage_results,
            total_processing_time,
            output_data: Some(final_data),
            metadata: std::collections::HashMap::new(),
            error_message: None,
        }));

        Ok(PipelineResult {
            pipeline_id: self.id.clone(),
            success: true,
            stage_results,
            total_processing_time,
            output_data: Some(final_data),
            metadata: std::collections::HashMap::new(),
            error_message: None,
        })
    }

    pub fn add_stage(&self, stage: ConversionStage) {
        let mut stages = self.stages.write();
        stages.push(stage);
    }

    pub fn remove_stage(&self, stage_id: &str) -> bool {
        let mut stages = self.stages.write();
        if let Some(index) = stages.iter().position(|s| s.id == stage_id) {
            stages.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_stage(&self, stage_id: &str) -> Option<ConversionStage> {
        let stages = self.stages.read();
        stages.iter().find(|s| s.id == stage_id).cloned()
    }

    pub fn get_stages(&self) -> Vec<ConversionStage> {
        self.stages.read().clone()
    }

    pub fn get_current_stage(&self) -> usize {
        *self.current_stage.read()
    }

    pub fn get_status(&self) -> PipelineStatus {
        self.status.read().clone()
    }

    pub fn pause(&self) {
        let mut status = self.status.write();
        *status = PipelineStatus::Paused;
    }

    pub fn resume(&self) {
        let mut status = self.status.write();
        *status = PipelineStatus::Running;
    }

    pub fn cancel(&self) {
        let mut status = self.status.write();
        *status = PipelineStatus::Cancelled;
        
        let _ = self.event_sender.send(PipelineEvent::PipelineCancelled);
    }

    pub fn reset(&self) {
        let mut status = self.status.write();
        *status = PipelineStatus::Idle;
        
        let mut current_stage = self.current_stage.write();
        *current_stage = 0;
        
        let stages = self.stages.read();
        for stage in stages.iter() {
            let mut stage_status = stage.status.write();
            *stage_status = StageStatus::Idle;
            
            let mut progress = stage.progress.write();
            *progress = 0.0;
        }
    }

    pub fn get_stats(&self) -> PipelineStats {
        let stages = self.stages.read();
        let completed_stages = stages.iter().filter(|s| {
            matches!(*s.status.read(), StageStatus::Completed)
        }).count();
        
        let failed_stages = stages.iter().filter(|s| {
            matches!(*s.status.read(), StageStatus::Failed(_))
        }).count();
        
        let skipped_stages = stages.iter().filter(|s| {
            matches!(*s.status.read(), StageStatus::Skipped)
        }).count();

        PipelineStats {
            total_stages: stages.len(),
            completed_stages,
            failed_stages,
            skipped_stages,
            total_processing_time: std::time::Duration::from_secs(0),
            average_stage_time: std::time::Duration::from_secs(0),
            throughput: 0.0,
        }
    }

    pub fn get_pipeline_config(&self) -> PipelineConfig {
        let stages = self.stages.read();
        let stage_configs: Vec<StageConfig> = stages.iter().map(|s| StageConfig {
            id: s.id.clone(),
            name: s.name.clone(),
            stage_type: s.stage_type.clone(),
            config: s.config.read().clone(),
        }).collect();

        PipelineConfig {
            pipeline_id: self.id.clone(),
            pipeline_name: self.name.clone(),
            stages: stage_configs,
            parallel_execution: false,
            max_concurrent_stages: None,
            error_handling: ErrorHandling::StopOnError,
            retry_config: RetryConfig::default(),
        }
    }

    pub async fn get_events(&mut self) -> Vec<PipelineEvent> {
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

    pub fn clone_pipeline(&self) -> ConversionPipeline {
        let stages = self.stages.read();
        let mut new_pipeline = Self::new(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", self.name),
        );

        for stage in stages.iter() {
            let cloned_stage = ConversionStage {
                id: stage.id.clone(),
                name: stage.name.clone(),
                stage_type: stage.stage_type.clone(),
                processor: stage.processor.clone(),
                config: Arc::new(RwLock::new(stage.config.read().clone())),
                status: Arc::new(RwLock::new(StageStatus::Idle)),
                progress: Arc::new(RwLock::new(0.0)),
            };
            
            new_pipeline.add_stage(cloned_stage);
        }

        new_pipeline
    }
}

fn create_stage_processor(config: &StageConfig) -> Box<dyn StageProcessor + Send + Sync> {
    match config.stage_type {
        StageType::Input => Box::new(InputStage::new()),
        StageType::Preprocessing => Box::new(PreprocessingStage::new()),
        StageType::Processing => Box::new(ProcessingStage::new()),
        StageType::Postprocessing => Box::new(PostprocessingStage::new()),
        StageType::Output => Box::new(OutputStage::new()),
        StageType::Custom(_) => Box::new(CustomStage::new()),
    }
}

struct InputStage {
    name: String,
}

impl InputStage {
    fn new() -> Self {
        Self {
            name: "Input Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for InputStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "input".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(10),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Input
    }
}

struct PreprocessingStage {
    name: String,
}

impl PreprocessingStage {
    fn new() -> Self {
        Self {
            name: "Preprocessing Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for PreprocessingStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "preprocessing".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(50),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Preprocessing
    }
}

struct ProcessingStage {
    name: String,
}

impl ProcessingStage {
    fn new() -> Self {
        Self {
            name: "Processing Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for ProcessingStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "processing".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(100),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Processing
    }
}

struct PostprocessingStage {
    name: String,
}

impl PostprocessingStage {
    fn new() -> Self {
        Self {
            name: "Postprocessing Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for PostprocessingStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "postprocessing".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(50),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Postprocessing
    }
}

struct OutputStage {
    name: String,
}

impl OutputStage {
    fn new() -> Self {
        Self {
            name: "Output Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for OutputStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "output".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(10),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Output
    }
}

struct CustomStage {
    name: String,
}

impl CustomStage {
    fn new() -> Self {
        Self {
            name: "Custom Stage".to_string(),
        }
    }

#[async_trait::async_trait]
impl StageProcessor for CustomStage {
    async fn process(&self, input_data: &[u8], config: &super::conversion_config::ConversionConfig) -> Result<StageResult, Box<dyn std::error::Error>> {
        Ok(StageResult {
            stage_id: "custom".to_string(),
            success: true,
            output_data: Some(input_data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(75),
            error_message: None,
        })
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_stage_type(&self) -> StageType {
        StageType::Custom("custom".to_string())
    }
}

impl Default for ConversionPipeline {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Pipeline".to_string(),
        )
    }
}

impl Default for PipelineStatus {
    fn default() -> Self {
        PipelineStatus::Idle
    }
}

impl Default for PipelineEvent {
    fn default() -> Self {
        PipelineEvent::PipelineStarted
    }
}

impl Default for StageStatus {
    fn default() -> Self {
        StageStatus::Idle
    }
}

impl Default for StageResult {
    fn default() -> Self {
        Self {
            stage_id: String::new(),
            success: false,
            output_data: None,
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for PipelineResult {
    fn default() -> Self {
        Self {
            pipeline_id: String::new(),
            success: false,
            stage_results: Vec::new(),
            total_processing_time: std::time::Duration::from_millis(0),
            output_data: None,
            metadata: std::collections::HashMap::new(),
            error_message: None,
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            pipeline_id: String::new(),
            pipeline_name: "Default Pipeline".to_string(),
            stages: Vec::new(),
            parallel_execution: false,
            max_concurrent_stages: None,
            error_handling: ErrorHandling::StopOnError,
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for ErrorHandling {
    fn default() -> Self {
        ErrorHandling::StopOnError
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            retry_on_timeout: false,
        }
    }
}

impl Default for PipelineStats {
    fn default() -> Self {
        Self {
            total_stages: 0,
            completed_stages: 0,
            failed_stages: 0,
            skipped_stages: 0,
            total_processing_time: std::time::Duration::from_secs(0),
            average_stage_time: std::time::Duration::from_secs(0),
            throughput: 0.0,
        }
    }
}
