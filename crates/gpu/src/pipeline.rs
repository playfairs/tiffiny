use std::sync::Arc;
use parking_lot::RwLock;
use wgpu::{BufferUsages, ShaderStages};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct PipelineManager {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>>,
    pub pipelines: Arc<RwLock<std::collections::HashMap<String, GpuPipeline>>>>,
    pub event_sender: mpsc::UnboundedSender<PipelineEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PipelineEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    PipelineCreated(String),
    PipelineDestroyed(String),
    PipelineBound(String),
    PipelineExecuted(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct GpuPipeline {
    pub id: String,
    pub name: String,
    pub pipeline_type: PipelineType,
    pub compute_pipeline: Arc<RwLock<Option<wgpu::ComputePipeline>>>>,
    pub render_pipeline: Arc<RwLock<Option<wgpu::RenderPipeline>>>>,
    pub bind_group_layouts: Arc<RwLock<Vec<wgpu::BindGroupLayout>>>>,
    pub bind_groups: Arc<RwLock<Vec<wgpu::BindGroup>>>>,
    pub vertex_buffers: Arc<RwLock<Vec<wgpu::VertexBufferLayout>>>>,
    pub color_targets: Arc<RwLock<Vec<wgpu::ColorTargetState>>>>,
    pub depth_stencil: Arc<RwLock<Option<wgpu::DepthStencilState>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineType {
    Compute,
    Render,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub pipeline_type: PipelineType,
    pub shader_stages: Vec<ShaderStage>,
    pub bind_group_descriptors: Vec<wgpu::BindGroupLayoutEntry>,
    pub vertex_buffer_descriptors: Vec<wgpu::VertexBufferLayout>,
    pub color_target_descriptors: Vec<wgpu::ColorTargetState>,
    pub depth_stencil_descriptor: Option<wgpu::DepthStencilState>,
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
    pub multiview: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ShaderStage {
    pub stage: wgpu::ShaderStages,
    pub module: Arc<RwLock<Option<wgpu::ShaderModule>>>>,
    pub entry_point: String,
}

#[derive(Debug, Clone)]
pub struct PipelineCache {
    pub id: String,
    pub name: String,
    pub cache: Arc<RwLock<std::collections::HashMap<String, CachedPipeline>>>>,
}

#[derive(Debug, Clone)]
pub struct CachedPipeline {
    pub pipeline: GpuPipeline,
    pub last_used: std::time::Instant,
    pub access_count: u64,
    pub creation_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PipelineBuilder {
    pub id: String,
    pub name: String,
    pub config: PipelineConfig,
    pub shader_modules: Arc<RwLock<Vec<ShaderStage>>>>,
    pub bind_group_layouts: Arc<RwLock<Vec<wgpu::BindGroupLayout>>>>,
    pub vertex_buffer_layouts: Arc<RwLock<Vec<wgpu::VertexBufferLayout>>>>,
}

impl PipelineManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            pipelines: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn set_device(&self, device: wgpu::Device) {
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
    }

    pub async fn create_pipeline(&self, config: PipelineConfig) -> Result<GpuPipeline, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let mut pipeline = GpuPipeline {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.pipeline_type.to_string(),
            pipeline_type: config.pipeline_type.clone(),
            compute_pipeline: Arc::new(RwLock::new(None))),
            render_pipeline: Arc::new(RwLock::new(None))),
            bind_group_layouts: Arc::new(RwLock::new(Vec::new())),
            bind_groups: Arc::new(RwLock::new(Vec::new())),
            vertex_buffers: Arc::new(RwLock::new(Vec::new())),
            color_targets: Arc::new(RwLock::new(config.color_target_descriptors))),
            depth_stencil: Arc::new(RwLock::new(config.depth_stencil_descriptor))),
        };

        match config.pipeline_type {
            PipelineType::Compute => {
                self.create_compute_pipeline(&mut pipeline, &config, &device).await?;
            },
            PipelineType::Render => {
                self.create_render_pipeline(&mut pipeline, &config, &device).await?;
            },
            PipelineType::Custom(_) => {
                return Err("Custom pipeline type not implemented".into());
            },
        }

Add to manager
        {
            let mut pipelines = self.pipelines.write();
            pipelines.insert(pipeline.id.clone(), pipeline.clone());
        }

        let _ = self.event_sender.send(PipelineEvent::PipelineCreated(pipeline.id.clone()));
        Ok(pipeline)
    }

    async fn create_compute_pipeline(&self, pipeline: &mut GpuPipeline, config: &PipelineConfig, device: &wgpu::Device) -> Result<(), Box<dyn std::error::Error>> {
        let compute_stage = config.shader_stages.iter()
            .find(|stage| stage.stage.contains(ShaderStages::COMPUTE))
            .ok_or("No compute shader stage found")?;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &config.bind_group_descriptors,
        });

        {
            let mut layouts = pipeline.bind_group_layouts.write();
            layouts.push(bind_group_layout);
        }

        let shader_module = {
            let module_ref = compute_stage.module.read();
            module_ref.as_ref().ok_or("Shader module not initialized")?.clone()
        };

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&pipeline.name),
            layout: Some(&pipeline.bind_group_layouts.read()[0]),
            module: &shader_module,
            entry_point: &compute_stage.entry_point,
        });

        {
            let mut compute_ref = pipeline.compute_pipeline.write();
            *compute_ref = Some(compute_pipeline);
        }

        Ok(())
    }

    async fn create_render_pipeline(&self, pipeline: &mut GpuPipeline, config: &PipelineConfig, device: &wgpu::Device) -> Result<(), Box<dyn std::error::Error>> {
        let vertex_stage = config.shader_stages.iter()
            .find(|stage| stage.stage.contains(ShaderStages::VERTEX))
            .ok_or("No vertex shader stage found")?;

        let fragment_stage = config.shader_stages.iter()
            .find(|stage| stage.stage.contains(ShaderStages::FRAGMENT));

        let bind_group_layouts: Vec<wgpu::BindGroupLayout> = config.bind_group_descriptors
            .chunks(4)
            .enumerate()
            .map(|(i, entries)| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("Render Bind Group Layout {}", i)),
                    entries,
                })
            })
            .collect();

        {
            let mut layouts = pipeline.bind_group_layouts.write();
            *layouts = bind_group_layouts;
        }

        let vertex_module = {
            let module_ref = vertex_stage.module.read();
            module_ref.as_ref().ok_or("Vertex shader module not initialized")?.clone()
        };

        let fragment_module = if let Some(fragment_stage) = fragment_stage {
            let module_ref = fragment_stage.module.read();
            Some(module_ref.as_ref().ok_or("Fragment shader module not initialized")?.clone())
        } else {
            None
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&pipeline.name),
            layout: Some(&pipeline.bind_group_layouts.read()[0]),
            vertex: wgpu::VertexState {
                module: &vertex_module,
                entry_point: &vertex_stage.entry_point,
                buffers: &config.vertex_buffer_descriptors,
            },
            fragment: fragment_stage.as_ref().map(|stage| wgpu::FragmentState {
                module: &{
                    let module_ref = stage.module.read();
                    module_ref.as_ref().ok_or("Fragment shader module not initialized")?.clone()
                },
                entry_point: &stage.entry_point,
                targets: &config.color_target_descriptors,
            }),
            primitive: config.primitive,
            depth_stencil: config.depth_stencil_descriptor,
            multisample: config.multisample,
            multiview: config.multiview,
        });

        {
            let mut render_ref = pipeline.render_pipeline.write();
            *render_ref = Some(render_pipeline);
        }

        Ok(())
    }

    pub fn destroy_pipeline(&self, pipeline_id: &str) -> bool {
        let mut pipelines = self.pipelines.write();
        
        if pipelines.remove(pipeline_id).is_some() {
            let _ = self.event_sender.send(PipelineEvent::PipelineDestroyed(pipeline_id.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get_pipeline(&self, pipeline_id: &str) -> Option<GpuPipeline> {
        let pipelines = self.pipelines.read();
        pipelines.get(pipeline_id).cloned()
    }

    pub fn list_pipelines(&self) -> Vec<GpuPipeline> {
        let pipelines = self.pipelines.read();
        pipelines.values().cloned().collect()
    }

    pub fn get_pipeline_count(&self) -> usize {
        let pipelines = self.pipelines.read();
        pipelines.len()
    }

    pub fn find_pipelines_by_type(&self, pipeline_type: &PipelineType) -> Vec<GpuPipeline> {
        let pipelines = self.pipelines.read();
        pipelines.values()
            .filter(|pipeline| &pipeline.pipeline_type == pipeline_type)
            .cloned()
            .collect()
    }

    pub async fn bind_pipeline(&self, pipeline: &GpuPipeline, bind_groups: Vec<wgpu::BindGroup>) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut pipeline_bind_groups = pipeline.bind_groups.write();
            *pipeline_bind_groups = bind_groups;
        }

        let _ = self.event_sender.send(PipelineEvent::PipelineBound(pipeline.id.clone()));
        Ok(())
    }

    pub async fn execute_compute_pipeline(&self, pipeline: &GpuPipeline, workgroup_count: u32, workgroup_size: u32) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let compute_pipeline = {
            let pipeline_ref = pipeline.compute_pipeline.read();
            pipeline_ref.as_ref().ok_or("Compute pipeline not initialized")?.clone()
        };

        let bind_groups = pipeline.bind_groups.read();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Pipeline Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&compute_pipeline);
            
            for (i, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(i as u32, bind_group);
            }

            compute_pass.dispatch_workgroups(workgroup_count, workgroup_size, 1);
        }

        let command_buffer = encoder.finish();
        let _ = queue.submit(Some(command_buffer));

        let _ = self.event_sender.send(PipelineEvent::PipelineExecuted(pipeline.id.clone()));
        Ok(())
    }

    pub async fn execute_render_pipeline(&self, pipeline: &GpuPipeline, vertex_buffer: &wgpu::Buffer, index_buffer: Option<&wgpu::Buffer>, color_attachments: Vec<wgpu::RenderPassColorAttachment>) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let render_pipeline = {
            let pipeline_ref = pipeline.render_pipeline.read();
            pipeline_ref.as_ref().ok_or("Render pipeline not initialized")?.clone()
        };

        let bind_groups = pipeline.bind_groups.read();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Pipeline Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&render_pipeline);
            
            for (i, bind_group) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group);
            }

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            
            if let Some(idx_buffer) = index_buffer {
                render_pass.set_index_buffer(idx_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..65536, 0, 0..1);
            } else {
                render_pass.draw(0..65536, 0..1);
            }
        }

        let command_buffer = encoder.finish();
        let _ = queue.submit(Some(command_buffer));

        let _ = self.event_sender.send(PipelineEvent::PipelineExecuted(pipeline.id.clone()));
        Ok(())
    }

    async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        device.poll(wgpu::Maintain::Wait);
        Err("Queue access not implemented".into())
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

    pub fn create_pipeline_builder(&self, pipeline_type: PipelineType) -> PipelineBuilder {
        PipelineBuilder::new(
            uuid::Uuid::new_v4().to_string(),
            format!("Pipeline Builder {}", pipeline_type.to_string()),
            pipeline_type,
        )
    }

    pub fn create_pipeline_cache(&self) -> PipelineCache {
        PipelineCache::new(
            uuid::Uuid::new_v4().to_string(),
            "Pipeline Cache".to_string(),
        )
    }

    pub fn get_pipeline_info(&self, pipeline: &GpuPipeline) -> PipelineInfo {
        PipelineInfo {
            id: pipeline.id.clone(),
            name: pipeline.name.clone(),
            pipeline_type: pipeline.pipeline_type.clone(),
            bind_group_count: pipeline.bind_group_layouts.read().len(),
            vertex_buffer_count: pipeline.vertex_buffers.read().len(),
            color_target_count: pipeline.color_targets.read().len(),
            has_depth_stencil: pipeline.depth_stencil.read().is_some(),
            is_compute: pipeline.compute_pipeline.read().is_some(),
            is_render: pipeline.render_pipeline.read().is_some(),
        }
    }

    pub fn benchmark_pipeline(&self, pipeline: &GpuPipeline, iterations: u32) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let mut execution_times = Vec::new();
        
        for _ in 0..iterations {
            let iteration_start = std::time::Instant::now();
            
            match pipeline.pipeline_type {
                PipelineType::Compute => {
                    if let Err(e) = self.execute_compute_pipeline(pipeline, 64, 1).await {
                        return Err(format!("Compute pipeline execution failed: {}", e).into());
                    }
                },
                PipelineType::Render => {
                },
                PipelineType::Custom(_) => {
                    return Err("Custom pipeline benchmarking not implemented".into());
                },
            }
            
            execution_times.push(iteration_start.elapsed());
        }

        let total_time = start_time.elapsed();
        let avg_time = execution_times.iter().sum::<std::time::Duration>() / iterations as u32;
        let min_time = execution_times.iter().min().unwrap_or(&std::time::Duration::from_millis(0));
        let max_time = execution_times.iter().max().unwrap_or(&std::time::Duration::from_millis(0));

        Ok(BenchmarkResult {
            pipeline_name: pipeline.name.clone(),
            iterations,
            total_time,
            average_time: avg_time,
            min_time: *min_time,
            max_time: *max_time,
            throughput: self.calculate_throughput(&avg_time),
        })
    }

    fn calculate_throughput(&self, time: &std::time::Duration) -> f64 {
        let time_seconds = time.as_secs_f64();
        if time_seconds > 0.0 {
            1000.0 / time_seconds
        } else {
            0.0
        }
    }

    pub fn optimize_pipeline(&self, pipeline: &mut GpuPipeline) -> Result<(), Box<dyn std::error::Error>> {
        
        {
            let mut layouts = pipeline.bind_group_layouts.write();
            for layout in layouts.iter_mut() {
            }
        }

        {
            let mut vertex_buffers = pipeline.vertex_buffers.write();
            for buffer in vertex_buffers.iter_mut() {
            }
        }

        Ok(())
    }

    pub fn validate_pipeline(&self, pipeline: &GpuPipeline) -> Result<(), Box<dyn std::error::Error>> {
        
        match pipeline.pipeline_type {
            PipelineType::Compute => {
                if pipeline.compute_pipeline.read().is_none() {
                    return Err("Compute pipeline missing compute pipeline".into());
                }
            },
            PipelineType::Render => {
                if pipeline.render_pipeline.read().is_none() {
                    return Err("Render pipeline missing render pipeline".into());
                }
                
                if pipeline.vertex_buffers.read().is_empty() {
                    return Err("Render pipeline missing vertex buffers".into());
                }
                
                if pipeline.color_targets.read().is_empty() {
                    return Err("Render pipeline missing color targets".into());
                }
            },
            PipelineType::Custom(_) => {
            },
        }

        Ok(())
    }

    pub fn clone_pipeline(&self, pipeline: &GpuPipeline) -> GpuPipeline {
        GpuPipeline {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", pipeline.name),
            pipeline_type: pipeline.pipeline_type.clone(),
            compute_pipeline: Arc::new(RwLock::new(pipeline.compute_pipeline.read().clone()))),
            render_pipeline: Arc::new(RwLock::new(pipeline.render_pipeline.read().clone()))),
            bind_group_layouts: Arc::new(RwLock::new(pipeline.bind_group_layouts.read().clone()))),
            bind_groups: Arc::new(RwLock::new(Vec::new()))),
            vertex_buffers: Arc::new(RwLock::new(pipeline.vertex_buffers.read().clone()))),
            color_targets: Arc::new(RwLock::new(pipeline.color_targets.read().clone()))),
            depth_stencil: Arc::new(RwLock::new(pipeline.depth_stencil.read().clone()))),
        }
    }

    pub fn reset(&self) {
        let mut pipelines = self.pipelines.write();
        pipelines.clear();
    }
}

#[derive(Debug, Clone)]
pub struct PipelineInfo {
    pub id: String,
    pub name: String,
    pub pipeline_type: PipelineType,
    pub bind_group_count: usize,
    pub vertex_buffer_count: usize,
    pub color_target_count: usize,
    pub has_depth_stencil: bool,
    pub is_compute: bool,
    pub is_render: bool,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub pipeline_name: String,
    pub iterations: u32,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
    pub min_time: std::time::Duration,
    pub max_time: std::time::Duration,
    pub throughput: f64,
}

impl PipelineBuilder {
    pub fn new(id: String, name: String, pipeline_type: PipelineType) -> Self {
        Self {
            id,
            name,
            config: PipelineConfig::default(),
            shader_modules: Arc::new(RwLock::new(Vec::new())),
            bind_group_layouts: Arc::new(RwLock::new(Vec::new())),
            vertex_buffer_layouts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_shader_stage(&self, stage: ShaderStage) {
        let mut modules = self.shader_modules.write();
        modules.push(stage);
    }

    pub fn add_bind_group_layout(&self, layout: wgpu::BindGroupLayout) {
        let mut layouts = self.bind_group_layouts.write();
        layouts.push(layout);
    }

    pub fn add_vertex_buffer_layout(&self, layout: wgpu::VertexBufferLayout) {
        let mut layouts = self.vertex_buffer_layouts.write();
        layouts.push(layout);
    }

    pub fn set_primitive_state(&mut self, primitive: wgpu::PrimitiveState) {
        self.config.primitive = primitive;
    }

    pub fn set_multisample_state(&mut self, multisample: wgpu::MultisampleState) {
        self.config.multisample = multisample;
    }

    pub fn add_color_target(&mut self, target: wgpu::ColorTargetState) {
        self.config.color_target_descriptors.push(target);
    }

    pub fn set_depth_stencil(&mut self, depth_stencil: wgpu::DepthStencilState) {
        self.config.depth_stencil_descriptor = Some(depth_stencil);
    }

    pub fn build(self) -> Result<PipelineConfig, Box<dyn std::error::Error>> {
        if self.shader_modules.read().is_empty() {
            return Err("No shader stages specified".into());
        }

        Ok(self.config)
    }
}

impl PipelineCache {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn cache_pipeline(&self, pipeline: GpuPipeline) {
        let mut cache = self.cache.write();
        cache.insert(pipeline.id.clone(), CachedPipeline {
            pipeline,
            last_used: std::time::Instant::now(),
            access_count: 1,
            creation_time: std::time::Instant::now(),
        });
    }

    pub fn get_cached_pipeline(&self, pipeline_id: &str) -> Option<GpuPipeline> {
        let mut cache = self.cache.write();
        if let Some(cached) = cache.get_mut(pipeline_id) {
            cached.last_used = std::time::Instant::now();
            cached.access_count += 1;
            Some(cached.pipeline.clone())
        } else {
            None
        }
    }

    pub fn remove_cached_pipeline(&self, pipeline_id: &str) -> Option<GpuPipeline> {
        let mut cache = self.cache.write();
        cache.remove(pipeline_id).map(|cached| cached.pipeline)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_pipelines = cache.len();
        let total_accesses: u64 = cache.values().map(|cached| cached.access_count).sum();
        let oldest_access = cache.values().map(|cached| cached.last_used).min();
        let newest_access = cache.values().map(|cached| cached.last_used).max();

        CacheStats {
            total_pipelines,
            total_accesses,
            oldest_access,
            newest_access,
        }
    }

    pub fn cleanup_old_pipelines(&self, max_age: std::time::Duration) -> usize {
        let mut cache = self.cache.write();
        let now = std::time::Instant::now();
        let initial_count = cache.len();
        
        cache.retain(|_, cached| now.duration_since(cached.last_used) <= max_age);
        
        initial_count - cache.len()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_pipelines: usize,
    pub total_accesses: u64,
    pub oldest_access: Option<std::time::Instant>,
    pub newest_access: Option<std::time::Instant>,
}

impl Default for PipelineManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Pipeline Manager".to_string(),
        )
    }
}

impl Default for GpuPipeline {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Pipeline".to_string(),
            pipeline_type: PipelineType::Compute,
            compute_pipeline: Arc::new(RwLock::new(None))),
            render_pipeline: Arc::new(RwLock::new(None))),
            bind_group_layouts: Arc::new(RwLock::new(Vec::new())),
            bind_groups: Arc::new(RwLock::new(Vec::new())),
            vertex_buffers: Arc::new(RwLock::new(Vec::new())),
            color_targets: Arc::new(RwLock::new(Vec::new())),
            depth_stencil: Arc::new(RwLock::new(None))),
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            pipeline_type: PipelineType::Compute,
            shader_stages: Vec::new(),
            bind_group_descriptors: Vec::new(),
            vertex_buffer_descriptors: Vec::new(),
            color_target_descriptors: Vec::new(),
            depth_stencil_descriptor: None,
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        }
    }
}

impl Default for ShaderStage {
    fn default() -> Self {
        Self {
            stage: ShaderStages::COMPUTE,
            module: Arc::new(RwLock::new(None))),
            entry_point: "main".to_string(),
        }
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Pipeline Cache".to_string(),
        )
    }
}

impl Default for CachedPipeline {
    fn default() -> Self {
        Self {
            pipeline: GpuPipeline::default(),
            last_used: std::time::Instant::now(),
            access_count: 0,
            creation_time: std::time::Instant::now(),
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Pipeline Builder".to_string(),
            PipelineType::Compute,
        )
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_pipelines: 0,
            total_accesses: 0,
            oldest_access: None,
            newest_access: None,
        }
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            pipeline_name: "Default Pipeline".to_string(),
            iterations: 1,
            total_time: std::time::Duration::from_millis(0),
            average_time: std::time::Duration::from_millis(0),
            min_time: std::time::Duration::from_millis(0),
            max_time: std::time::Duration::from_millis(0),
            throughput: 0.0,
        }
    }
}

impl Default for PipelineInfo {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Pipeline".to_string(),
            pipeline_type: PipelineType::Compute,
            bind_group_count: 0,
            vertex_buffer_count: 0,
            color_target_count: 0,
            has_depth_stencil: false,
            is_compute: false,
            is_render: false,
        }
    }
}

impl Default for PipelineType {
    fn default() -> Self {
        PipelineType::Compute
    }
}

impl std::fmt::Display for PipelineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineType::Compute => write!(f, "Compute"),
            PipelineType::Render => write!(f, "Render"),
            PipelineType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}
