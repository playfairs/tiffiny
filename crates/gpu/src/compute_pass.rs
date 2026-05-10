use std::sync::Arc;
use parking_lot::RwLock;
use wgpu::{ShaderStages, BufferUsages};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ComputePassManager {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>>,
    pub passes: Arc<RwLock<std::collections::HashMap<String, ComputePass>>>>,
    pub event_sender: mpsc::UnboundedSender<ComputePassEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ComputePassEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ComputePassEvent {
    PassCreated(String),
    PassDestroyed(String),
    PassStarted(String),
    PassCompleted(String),
    PassFailed(String, String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ComputePass {
    pub id: String,
    pub name: String,
    pub pipeline: Arc<RwLock<Option<wgpu::ComputePipeline>>>>,
    pub bind_groups: Arc<RwLock<Vec<wgpu::BindGroup>>>>,
    pub workgroup_count: Arc<RwLock<(u32, u32, u32)>>>,
    pub state: Arc<RwLock<ComputePassState>>,
    pub timestamp: Arc<RwLock<std::time::Instant>>,
    pub metadata: Arc<RwLock<std::collections::HashMap<String, String>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputePassState {
    Initial,
    Configured,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ComputePassConfig {
    pub pipeline_id: String,
    pub bind_group_ids: Vec<String>,
    pub workgroup_count: (u32, u32, u32),
    pub push_constants: Option<Vec<u8>>,
    pub label: Option<String>,
    pub timestamp: bool,
}

#[derive(Debug, Clone)]
pub struct ComputePassBuilder {
    pub id: String,
    pub name: String,
    pub config: ComputePassConfig,
    pub pipeline_ref: Arc<RwLock<Option<wgpu::ComputePipeline>>>>,
    pub bind_group_refs: Arc<RwLock<Vec<wgpu::BindGroup>>>>,
}

#[derive(Debug, Clone)]
pub struct ComputePassResult {
    pub success: bool,
    pub pass_id: String,
    pub execution_time: std::time::Duration,
    pub work_items_processed: u32,
    pub error_message: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ComputePassCache {
    pub id: String,
    pub name: String,
    pub cache: Arc<RwLock<std::collections::HashMap<String, CachedComputePass>>>>,
}

#[derive(Debug, Clone)]
pub struct CachedComputePass {
    pub pass: ComputePass,
    pub last_used: std::time::Instant,
    pub access_count: u64,
    pub creation_time: std::time::Instant,
}

impl ComputePassManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            passes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub fn set_device(&self, device: wgpu::Device) {
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
    }

    pub async fn create_pass(&self, config: ComputePassConfig) -> Result<ComputePass, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let pass = ComputePass {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.label.unwrap_or_else(|| format!("Compute Pass {}", uuid::Uuid::new_v4())),
            pipeline: Arc::new(RwLock::new(None))),
            bind_groups: Arc::new(RwLock::new(Vec::new()))),
            workgroup_count: Arc::new(RwLock::new(config.workgroup_count)),
            state: Arc::new(RwLock::new(ComputePassState::Initial))),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

Add to manager
        {
            let mut passes = self.passes.write();
            passes.insert(pass.id.clone(), pass.clone());
        }

        let _ = self.event_sender.send(ComputePassEvent::PassCreated(pass.id.clone()));
        Ok(pass)
    }

    pub async fn configure_pass(&self, pass: &ComputePass, pipeline: wgpu::ComputePipeline, bind_groups: Vec<wgpu::BindGroup>) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut pipeline_ref = pass.pipeline.write();
            *pipeline_ref = Some(pipeline);
        }

        {
            let mut bind_group_refs = pass.bind_groups.write();
            *bind_group_refs = bind_groups;
        }

        {
            let mut state = pass.state.write();
            *state = ComputePassState::Configured;
        }

        Ok(())
    }

    pub async fn execute_pass(&self, pass: &ComputePass) -> Result<ComputePassResult, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;
        
        let start_time = std::time::Instant::now();
        
        {
            let mut state = pass.state.write();
            *state = ComputePassState::Running;
        }

        let _ = self.event_sender.send(ComputePassEvent::PassStarted(pass.id.clone()));

        let pipeline = {
            let pipeline_ref = pass.pipeline.read();
            pipeline_ref.as_ref().ok_or("Pipeline not configured")?.clone()
        };

        let bind_groups = pass.bind_groups.read().clone();
        let workgroup_count = *pass.workgroup_count.read();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("Compute Pass Encoder {}", pass.name)),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!("Compute Pass {}", pass.name)),
            });

            compute_pass.set_pipeline(&pipeline);
            
            for (i, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(i as u32, bind_group);
            }

            compute_pass.dispatch_workgroups(workgroup_count.0, workgroup_count.1, workgroup_count.2);
        }

        let command_buffer = encoder.finish();
        let _ = queue.submit(Some(command_buffer));
        device.poll(wgpu::Maintain::Wait);

        let execution_time = start_time.elapsed();
        let work_items_processed = workgroup_count.0 * workgroup_count.1 * workgroup_count.2;

        {
            let mut state = pass.state.write();
            *state = ComputePassState::Completed;
        }

        let result = ComputePassResult {
            success: true,
            pass_id: pass.id.clone(),
            execution_time,
            work_items_processed,
            error_message: None,
            metadata: self.generate_pass_metadata(&pass, &result),
        };

        let _ = self.event_sender.send(ComputePassEvent::PassCompleted(pass.id.clone()));
        Ok(result)
    }

    pub async fn execute_pass_with_callback<F>(&self, pass: &ComputePass, callback: F) -> Result<ComputePassResult, Box<dyn std::error::Error>>
    where
        F: Fn(ComputePassResult) -> (),
        F: Send + Sync,
    {
        let result = self.execute_pass(pass).await?;
        callback(result);
        Ok(result)
    }

    pub async fn execute_pass_async(&self, pass: &ComputePass) -> Result<std::pin::Pin<Box<dyn std::future::Future<Output = Result<ComputePassResult, Box<dyn std::error::Error>>>>, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;
        
        let pass_clone = pass.clone();
        
        Box::pin(async move {
            let start_time = std::time::Instant::now();
            
            {
                let mut state = pass_clone.state.write();
                *state = ComputePassState::Running;
            }

            let pipeline = {
                let pipeline_ref = pass_clone.pipeline.read();
                pipeline_ref.as_ref().ok_or("Pipeline not configured")?.clone()
            };

            let bind_groups = pass_clone.bind_groups.read().clone();
            let workgroup_count = *pass_clone.workgroup_count.read();

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Async Compute Pass Encoder {}", pass_clone.name)),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(&format!("Async Compute Pass {}", pass_clone.name)),
                });

                compute_pass.set_pipeline(&pipeline);
                
                for (i, bind_group) in bind_groups.iter().enumerate() {
                    compute_pass.set_bind_group(i as u32, bind_group);
                }

                compute_pass.dispatch_workgroups(workgroup_count.0, workgroup_count.1, workgroup_count.2);
            }

            let command_buffer = encoder.finish();
            let _ = queue.submit(Some(command_buffer));
            device.poll(wgpu::Maintain::Wait);

            let execution_time = start_time.elapsed();
            let work_items_processed = workgroup_count.0 * workgroup_count.1 * workgroup_count.2;

            {
                let mut state = pass_clone.state.write();
                *state = ComputePassState::Completed;
            }

            Ok(ComputePassResult {
                success: true,
                pass_id: pass_clone.id.clone(),
                execution_time,
                work_items_processed,
                error_message: None,
                metadata: std::collections::HashMap::new(),
            })
        })
    }

    pub fn destroy_pass(&self, pass_id: &str) -> bool {
        let mut passes = self.passes.write();
        
        if passes.remove(pass_id).is_some() {
            let _ = self.event_sender.send(ComputePassEvent::PassDestroyed(pass_id.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get_pass(&self, pass_id: &str) -> Option<ComputePass> {
        let passes = self.passes.read();
        passes.get(pass_id).cloned()
    }

    pub fn list_passes(&self) -> Vec<ComputePass> {
        let passes = self.passes.read();
        passes.values().cloned().collect()
    }

    pub fn get_pass_count(&self) -> usize {
        let passes = self.passes.read();
        passes.len()
    }

    pub fn find_passes_by_state(&self, state: ComputePassState) -> Vec<ComputePass> {
        let passes = self.passes.read();
        passes.values()
            .filter(|pass| *pass.state.read() == state)
            .cloned()
            .collect()
    }

    pub async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    pub async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        device.poll(wgpu::Maintain::Wait);
        Err("Queue access not implemented".into())
    }

    pub async fn get_events(&mut self) -> Vec<ComputePassEvent> {
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

    pub fn create_pass_builder(&self, config: ComputePassConfig) -> ComputePassBuilder {
        ComputePassBuilder::new(
            uuid::Uuid::new_v4().to_string(),
            format!("Pass Builder {}", config.pipeline_id),
            config,
        )
    }

    pub fn create_pass_cache(&self) -> ComputePassCache {
        ComputePassCache::new(
            uuid::Uuid::new_v4().to_string(),
            "Compute Pass Cache".to_string(),
        )
    }

    pub fn get_pass_info(&self, pass: &ComputePass) -> ComputePassInfo {
        let state = pass.state.read().clone();
        let workgroup_count = *pass.workgroup_count.read();
        let timestamp = *pass.timestamp.read();
        let metadata = pass.metadata.read().clone();

        ComputePassInfo {
            id: pass.id.clone(),
            name: pass.name.clone(),
            state,
            workgroup_count,
            pipeline_configured: pass.pipeline.read().is_some(),
            bind_groups_count: pass.bind_groups.read().len(),
            timestamp,
            metadata,
        }
    }

    pub fn benchmark_pass(&self, pass: &ComputePass, iterations: u32) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let mut execution_times = Vec::new();
        
        for _ in 0..iterations {
            let result = self.execute_pass(pass).await?;
            execution_times.push(result.execution_time);
        }

        let total_time = start_time.elapsed();
        let avg_time = execution_times.iter().sum::<std::time::Duration>() / iterations as u32;
        let min_time = execution_times.iter().min().unwrap_or(&std::time::Duration::from_millis(0));
        let max_time = execution_times.iter().max().unwrap_or(&std::time::Duration::from_millis(0));

        Ok(BenchmarkResult {
            pass_name: pass.name.clone(),
            iterations,
            total_time,
            average_time: avg_time,
            min_time: *min_time,
            max_time: *max_time,
            throughput: self.calculate_throughput(&pass, &avg_time),
        })
    }

    fn calculate_throughput(&self, pass: &ComputePass, time: &std::time::Duration) -> f64 {
        let workgroup_count = *pass.workgroup_count.read();
        let work_items = workgroup_count.0 * workgroup_count.1 * workgroup_count.2;
        let time_seconds = time.as_secs_f64();
        
        if time_seconds > 0.0 {
            work_items as f64 / time_seconds
        } else {
            0.0
        }
    }

    fn generate_pass_metadata(&self, pass: &ComputePass, result: &ComputePassResult) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("pass_id".to_string(), pass.id.clone());
        metadata.insert("pass_name".to_string(), pass.name.clone());
        metadata.insert("execution_time_ms".to_string(), format!("{:.2}", result.execution_time.as_millis() as f64));
        metadata.insert("work_items_processed".to_string(), result.work_items_processed.to_string());
        metadata.insert("success".to_string(), result.success.to_string());
        
        if let Some(ref error) = result.error_message {
            metadata.insert("error".to_string(), error.clone());
        }

        metadata
    }

    pub fn clone_pass(&self, pass: &ComputePass) -> ComputePass {
        ComputePass {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", pass.name),
            pipeline: Arc::new(RwLock::new(pass.pipeline.read().clone()))),
            bind_groups: Arc::new(RwLock::new(pass.bind_groups.read().clone()))),
            workgroup_count: Arc::new(RwLock::new(*pass.workgroup_count.read()))),
            state: Arc::new(RwLock::new(ComputePassState::Initial))),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
            metadata: Arc::new(RwLock::new(pass.metadata.read().clone()))),
        }
    }

    pub fn reset_pass(&self, pass: &ComputePass) {
        {
            let mut state = pass.state.write();
            *state = ComputePassState::Initial;
        }

        {
            let mut timestamp = pass.timestamp.write();
            *timestamp = std::time::Instant::now();
        }

        {
            let mut metadata = pass.metadata.write();
            metadata.clear();
        }
    }

    pub fn cleanup_completed_passes(&self) -> usize {
        let mut passes = self.passes.write();
        let initial_count = passes.len();
        
        passes.retain(|_, pass| {
            let state = *pass.state.read();
            state != ComputePassState::Completed
        });

        let cleaned_count = initial_count - passes.len();
        let _ = self.event_sender.send(ComputePassEvent::Error(format!("Cleaned up {} completed passes", cleaned_count)));
        
        cleaned_count
    }

    pub fn reset(&self) {
        let mut passes = self.passes.write();
        passes.clear();
    }
}

impl ComputePassBuilder {
    pub fn new(id: String, name: String, config: ComputePassConfig) -> Self {
        Self {
            id,
            name,
            config,
            pipeline_ref: Arc::new(RwLock::new(None))),
            bind_group_refs: Arc::new(RwLock::new(Vec::new()))),
        }
    }

    pub fn set_pipeline(&self, pipeline: wgpu::ComputePipeline) {
        let mut pipeline_ref = self.pipeline_ref.write();
        *pipeline_ref = Some(pipeline);
    }

    pub fn add_bind_group(&self, bind_group: wgpu::BindGroup) {
        let mut bind_group_refs = self.bind_group_refs.write();
        bind_group_refs.push(bind_group);
    }

    pub fn set_workgroup_count(&self, count: (u32, u32, u32)) {
    }

    pub fn build(self) -> Result<ComputePass, Box<dyn std::error::Error>> {
        let pass = ComputePass {
            id: self.id,
            name: self.name,
            pipeline: self.pipeline_ref,
            bind_groups: self.bind_group_refs,
            workgroup_count: Arc::new(RwLock::new(self.config.workgroup_count))),
            state: Arc::new(RwLock::new(ComputePassState::Initial))),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        Ok(pass)
    }
}

impl ComputePassCache {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn cache_pass(&self, pass: ComputePass) {
        let mut cache = self.cache.write();
        cache.insert(pass.id.clone(), CachedComputePass {
            pass,
            last_used: std::time::Instant::now(),
            access_count: 1,
            creation_time: std::time::Instant::now(),
        });
    }

    pub fn get_cached_pass(&self, pass_id: &str) -> Option<ComputePass> {
        let mut cache = self.cache.write();
        if let Some(cached) = cache.get_mut(pass_id) {
            cached.last_used = std::time::Instant::now();
            cached.access_count += 1;
            Some(cached.pass.clone())
        } else {
            None
        }
    }

    pub fn remove_cached_pass(&self, pass_id: &str) -> Option<ComputePass> {
        let mut cache = self.cache.write();
        cache.remove(pass_id).map(|cached| cached.pass)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_passes = cache.len();
        let total_accesses: u64 = cache.values().map(|cached| cached.access_count).sum();
        let oldest_access = cache.values().map(|cached| cached.last_used).min();
        let newest_access = cache.values().map(|cached| cached.last_used).max();

        CacheStats {
            total_passes,
            total_accesses,
            oldest_access,
            newest_access,
        }
    }

    pub fn cleanup_old_passes(&self, max_age: std::time::Duration) -> usize {
        let mut cache = self.cache.write();
        let now = std::time::Instant::now();
        let initial_count = cache.len();
        
        cache.retain(|_, cached| now.duration_since(cached.last_used) <= max_age);
        
        initial_count - cache.len()
    }
}

#[derive(Debug, Clone)]
pub struct ComputePassInfo {
    pub id: String,
    pub name: String,
    pub state: ComputePassState,
    pub workgroup_count: (u32, u32, u32),
    pub pipeline_configured: bool,
    pub bind_groups_count: usize,
    pub timestamp: std::time::Instant,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub pass_name: String,
    pub iterations: u32,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
    pub min_time: std::time::Duration,
    pub max_time: std::time::Duration,
    pub throughput: f64,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_passes: usize,
    pub total_accesses: u64,
    pub oldest_access: Option<std::time::Instant>,
    pub newest_access: Option<std::time::Instant>,
}

impl Default for ComputePassManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Compute Pass Manager".to_string(),
        )
    }
}

impl Default for ComputePass {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Compute Pass".to_string(),
            pipeline: Arc::new(RwLock::new(None))),
            bind_groups: Arc::new(RwLock::new(Vec::new()))),
            workgroup_count: Arc::new(RwLock::new((1, 1, 1)))),
            state: Arc::new(RwLock::new(ComputePassState::Initial))),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for ComputePassState {
    fn default() -> Self {
        ComputePassState::Initial
    }
}

impl Default for ComputePassConfig {
    fn default() -> Self {
        Self {
            pipeline_id: "default".to_string(),
            bind_group_ids: Vec::new(),
            workgroup_count: (1, 1, 1),
            push_constants: None,
            label: None,
            timestamp: false,
        }
    }
}

impl Default for ComputePassBuilder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Compute Pass Builder".to_string(),
            ComputePassConfig::default(),
        )
    }
}

impl Default for ComputePassResult {
    fn default() -> Self {
        Self {
            success: false,
            pass_id: "default".to_string(),
            execution_time: std::time::Duration::from_millis(0),
            work_items_processed: 0,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for ComputePassCache {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Compute Pass Cache".to_string(),
        )
    }
}

impl Default for CachedComputePass {
    fn default() -> Self {
        Self {
            pass: ComputePass::default(),
            last_used: std::time::Instant::now(),
            access_count: 0,
            creation_time: std::time::Instant::now(),
        }
    }
}

impl Default for ComputePassInfo {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Compute Pass".to_string(),
            state: ComputePassState::default(),
            workgroup_count: (1, 1, 1),
            pipeline_configured: false,
            bind_groups_count: 0,
            timestamp: std::time::Instant::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            pass_name: "Default Pass".to_string(),
            iterations: 1,
            total_time: std::time::Duration::from_millis(0),
            average_time: std::time::Duration::from_millis(0),
            min_time: std::time::Duration::from_millis(0),
            max_time: std::time::Duration::from_millis(0),
            throughput: 0.0,
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_passes: 0,
            total_accesses: 0,
            oldest_access: None,
            newest_access: None,
        }
    }
}
