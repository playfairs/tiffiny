use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ComputeEngine {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>,
    pub queue: Arc<RwLock<Option<wgpu::Queue>>>,
    pub event_sender: mpsc::UnboundedSender<ComputeEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ComputeEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ComputeEvent {
    ComputeStarted,
    ComputeProgress(f32),
    ComputeCompleted(ComputeResult),
    Error(String),
    ShaderCompiled,
    ShaderFailed(String),
}

#[derive(Debug, Clone)]
pub struct ComputeResult {
    pub success: bool,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ComputeConfig {
    pub shader_source: String,
    pub entry_point: String,
    pub workgroup_size: u32,
    pub workgroup_count: u32,
    pub buffer_size: u64,
    pub use_gpu: bool,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Discrete,
    Integrated,
    Virtual,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ComputeShader {
    pub id: String,
    pub name: String,
    pub source: String,
    pub entry_point: String,
    pub module: Arc<RwLock<Option<wgpu::ShaderModule>>>,
    pub bind_group_layout: Arc<RwLock<Option<wgpu::BindGroupLayout>>>,
    pub pipeline: Arc<RwLock<Option<wgpu::ComputePipeline>>>,
}

#[derive(Debug, Clone)]
pub struct ComputeBuffer {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub usage: wgpu::BufferUsages,
    pub buffer: Arc<RwLock<Option<wgpu::Buffer>>>,
    pub mapped_data: Arc<RwLock<Option<Vec<u8>>>>,
}

#[derive(Debug, Clone)]
pub struct ComputeWorkgroup {
    pub size: u32,
    pub count: u32,
    pub total_work_items: u32,
}

impl ComputeEngine {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            queue: Arc::new(RwLock::new(None))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn initialize(&self, device: wgpu::Device, queue: wgpu::Queue) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ComputeEvent::ComputeStarted);
        
Store device and queue
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
        
        let mut queue_ref = self.queue.write();
        *queue_ref = Some(queue);

        Ok(())
    }

    pub async fn compile_shader(&self, source: &str, entry_point: &str) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ComputeEvent::ShaderCompiled);
        
        let device = self.get_device().await?;
        
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        Ok(ComputeShader {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Compute Shader".to_string(),
            source: source.to_string(),
            entry_point: entry_point.to_string(),
            module: Arc::new(RwLock::new(Some(shader_module))),
            bind_group_layout: Arc::new(RwLock::new(Some(bind_group_layout))),
            pipeline: Arc::new(RwLock::new(None))),
        })
    }

    pub async fn create_compute_pipeline(&self, shader: &ComputeShader) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ComputeEvent::ShaderCompiled);
        
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;
        
        let module = {
            let module_ref = shader.module.read();
            module_ref.as_ref().ok_or("Shader module not initialized")?.clone()
        };
        
        let bind_group_layout = {
            let layout_ref = shader.bind_group_layout.read();
            layout_ref.as_ref().ok_or("Bind group layout not initialized")?.clone()
        };

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&bind_group_layout),
            module: &module,
            entry_point: &shader.entry_point,
        });

        let mut pipeline_ref = shader.pipeline.write();
        *pipeline_ref = Some(pipeline);

        Ok(())
    }

    pub async fn create_buffer(&self, size: u64, usage: wgpu::BufferUsages) -> Result<ComputeBuffer, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Buffer"),
            size,
            usage,
            mapped_at_creation: false,
        });

        Ok(ComputeBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Compute Buffer".to_string(),
            size,
            usage,
            buffer: Arc::new(RwLock::new(Some(buffer))),
            mapped_data: Arc::new(RwLock::new(None))),
        })
    }

    pub async fn create_mapped_buffer(&self, size: u64, usage: wgpu::BufferUsages) -> Result<ComputeBuffer, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mapped Compute Buffer"),
            size,
            usage: usage | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });

        let mapped_data = vec![0u8; size as usize];
        
        Ok(ComputeBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Mapped Compute Buffer".to_string(),
            size,
            usage,
            buffer: Arc::new(RwLock::new(Some(buffer))),
            mapped_data: Arc::new(RwLock::new(Some(mapped_data))),
        })
    }

    pub async fn execute_compute(&self, config: ComputeConfig) -> Result<ComputeResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ComputeEvent::ComputeStarted);
        let start_time = std::time::Instant::now();

        if !config.use_gpu {
            return self.execute_cpu_fallback(&config).await;
        }

        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let shader = self.compile_shader(&config.shader_source, &config.entry_point).await?;
        self.create_compute_pipeline(&shader).await?;

        let input_buffer = self.create_buffer(config.buffer_size, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST).await?;
        let output_buffer = self.create_buffer(config.buffer_size, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC).await?;
        let uniform_buffer = self.create_buffer(256, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST).await?;

        let bind_group_layout = {
            let layout_ref = shader.bind_group_layout.read();
            layout_ref.as_ref().ok_or("Bind group layout not initialized")?.clone()
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.buffer.read().ok_or("Input buffer not initialized")?.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.buffer.read().ok_or("Uniform buffer not initialized")?.as_entire_binding(),
                },
            ],
        });

        let pipeline = {
            let pipeline_ref = shader.pipeline.read();
            pipeline_ref.as_ref().ok_or("Pipeline not initialized")?.clone()
        };

        let workgroup_count = config.workgroup_count;
        let workgroup_size = config.workgroup_size;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let input_buffer_ref = input_buffer.buffer.read();
            let uniform_buffer_ref = uniform_buffer.buffer.read();
            
            if let (Some(input_buf), Some(uniform_buf)) = (*input_buffer_ref, *uniform_buffer_ref) {
                encoder.copy_buffer_to_buffer(&input_buf, 0, &uniform_buf, 0, 256);
            }
        }

        {
            let uniform_buffer_ref = uniform_buffer.buffer.read();
            let output_buffer_ref = output_buffer.buffer.read();
            
            if let (Some(uniform_buf), Some(output_buf)) = (*uniform_buffer_ref, *output_buffer_ref) {
                encoder.copy_buffer_to_buffer(&uniform_buf, 0, &output_buf, 0, config.buffer_size);
            }
        }

        {
            let output_buffer_ref = output_buffer.buffer.read();
            
            if let Some(output_buf) = *output_buffer_ref {
                encoder.copy_buffer_to_buffer(&output_buf, 0, &output_buf, 0, config.buffer_size);
            }
        }

        let command_buffer = encoder.finish();

        let _ = queue.submit(Some(command_buffer));

        device.poll(wgpu::Maintain::Wait);

        let processing_time = start_time.elapsed();

        let output_data = self.read_buffer_data(&output_buffer).await?;

        Ok(ComputeResult {
            success: true,
            output_data,
            metadata: self.generate_metadata(&config),
            processing_time,
            error_message: None,
        })
    }

    async fn execute_cpu_fallback(&self, config: &ComputeConfig) -> Result<ComputeResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ComputeEvent::ComputeStarted);
        let start_time = std::time::Instant::now();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let processing_time = start_time.elapsed();
        let output_data = vec![0u8; config.buffer_size as usize];

        Ok(ComputeResult {
            success: true,
            output_data,
            metadata: self.generate_metadata(&config),
            processing_time,
            error_message: Some("CPU fallback used".to_string()),
        })
    }

    async fn read_buffer_data(&self, buffer: &ComputeBuffer) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let buffer_ref = buffer.buffer.read();
        let buffer = buffer_ref.as_ref().ok_or("Buffer not initialized")?;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(buffer, 0, &staging_buffer, 0, buffer.size());

        let command_buffer = encoder.finish();

        let queue = self.get_queue().await?;
        let _ = queue.submit(Some(command_buffer));

        device.poll(wgpu::Maintain::Wait);

        let data_slice = staging_buffer.slice(..);
        let data = data_slice.get_mapped_range().map_err(|e| format!("Failed to map buffer: {:?}", e))?;

        Ok(data.to_vec())
    }

    async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let queue_ref = self.queue.read();
        queue_ref.as_ref().ok_or("Queue not initialized")?.clone()
    }

    fn generate_metadata(&self, config: &ComputeConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("shader_source".to_string(), config.shader_source.clone());
        metadata.insert("entry_point".to_string(), config.entry_point.clone());
        metadata.insert("workgroup_size".to_string(), config.workgroup_size.to_string());
        metadata.insert("workgroup_count".to_string(), config.workgroup_count.to_string());
        metadata.insert("buffer_size".to_string(), config.buffer_size.to_string());
        metadata.insert("use_gpu".to_string(), config.use_gpu.to_string());
        metadata.insert("device_type".to_string(), format!("{:?}", config.device_type));
        
        metadata
    }

    pub async fn get_events(&mut self) -> Vec<ComputeEvent> {
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

    pub fn set_device(&self, device: wgpu::Device) {
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
    }

    pub fn set_queue(&self, queue: wgpu::Queue) {
        let mut queue_ref = self.queue.write();
        *queue_ref = Some(queue);
    }

    pub fn get_compute_workgroup(&self, total_items: u32, workgroup_size: u32) -> ComputeWorkgroup {
        let workgroup_count = (total_items + workgroup_size - 1) / workgroup_size;
        
        ComputeWorkgroup {
            size: workgroup_size,
            count: workgroup_count,
            total_work_items: total_items,
        }
    }

    pub fn clone_engine(&self) -> ComputeEngine {
        let mut new_engine = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
        );

        if let Some(device) = self.device.read().clone() {
            new_engine.set_device(device);
        }

        if let Some(queue) = self.queue.read().clone() {
            new_engine.set_queue(queue);
        }

        new_engine
    }

    pub fn reset(&self) {
        let mut device_ref = self.device.write();
        *device_ref = None;
        
        let mut queue_ref = self.queue.write();
        *queue_ref = None;
    }

    pub fn estimate_compute_time(&self, work_items: u32, workgroup_size: u32, config: &ComputeConfig) -> std::time::Duration {
        let base_time_ms = if config.use_gpu {
            10.0
        } else {
            100.0
        };

        let workgroups = (work_items + workgroup_size - 1) / workgroup_size;
        let total_time_ms = workgroups as f64 * base_time_ms;
        
        std::time::Duration::from_millis(total_time_ms as u64)
    }

    pub fn estimate_memory_usage(&self, buffer_size: u64, config: &ComputeConfig) -> u64 {
        let input_buffer_size = buffer_size;
        
        let output_buffer_size = buffer_size;
        
        let uniform_buffer_size = 256;
        
        let shader_size = config.shader_source.len() as u64;
        
        input_buffer_size + output_buffer_size + uniform_buffer_size + shader_size
    }

    pub fn get_supported_extensions(&self) -> Vec<String> {
        vec![
            "compute-shader".to_string(),
            "storage-buffers".to_string(),
            "uniform-buffers".to_string(),
        ]
    }

    pub fn can_use_extension(&self, extension: &str) -> bool {
        self.get_supported_extensions().contains(&extension.to_string())
    }

    pub fn get_device_info(&self) -> Option<DeviceInfo> {
        if let Some(device) = self.device.read().clone() {
            Some(DeviceInfo {
                name: "GPU Device".to_string(),
                vendor: "Unknown".to_string(),
                driver_version: "Unknown".to_string(),
                device_type: DeviceType::Discrete,
                memory: 0,
                features: vec![],
                limits: DeviceLimits::default(),
            })
        } else {
            None
        }
    }

    pub fn get_queue_info(&self) -> Option<QueueInfo> {
        if let Some(queue) = self.queue.read().clone() {
            Some(QueueInfo {
                queue_type: "Compute".to_string(),
                max_workgroup_size: 1024,
                max_workgroups_per_dispatch: 65535,
                max_total_workgroups: 65535,
            })
        } else {
            None
        }
    }

    pub fn create_compute_pass(&self) -> Result<ComputePass, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        Ok(ComputePass {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Compute Pass".to_string(),
            device: Arc::new(RwLock::new(Some(device))),
            queue: Arc::new(RwLock::new(Some(queue))),
            encoder: Arc::new(RwLock::new(None))),
            commands: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn create_shader_from_file(&self, file_path: &str) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        let source = std::fs::read_to_string(file_path)?;
        self.compile_shader(&source, "main").await
    }

    pub fn create_shader_from_template(&self, template: &str, params: &std::collections::HashMap<String, String>) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        let mut source = template.to_string();
        
        for (key, value) in params {
            source = source.replace(&format!("{{{}}}", key), value);
        }

        self.compile_shader(&source, "main").await
    }

    pub fn optimize_shader(&self, shader: &ComputeShader) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        let optimized_source = self.optimize_shader_source(&shader.source);
        
        let device = self.get_device().await?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Optimized Compute Shader"),
            source: &optimized_source,
        });

        Ok(ComputeShader {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} (Optimized)", shader.name),
            source: optimized_source,
            entry_point: shader.entry_point.clone(),
            module: Arc::new(RwLock::new(Some(module))),
            bind_group_layout: shader.bind_group_layout.clone(),
            pipeline: Arc::new(RwLock::new(None))),
        })
    }

    fn optimize_shader_source(&self, source: &str) -> String {
        let mut optimized = source.to_string();
        
        optimized = optimized.lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        
        optimized = optimized.lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n");
        
        optimized
    }

    pub fn validate_shader(&self, source: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !source.contains("@compute") {
            return Err("Shader must contain @compute attribute".into());
        }
        
        if !source.contains("main") {
            return Err("Shader must contain main function".into());
        }
        
        Ok(())
    }

    pub fn benchmark_shader(&self, shader: &ComputeShader, config: &ComputeConfig) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let iterations = 10;
        let mut times = Vec::new();
        
        for _ in 0..iterations {
            let result = self.execute_compute(config).await?;
            times.push(result.processing_time);
        }
        
        let total_time = start_time.elapsed();
        let avg_time = times.iter().sum::<std::time::Duration>() / iterations as u32;
        let min_time = times.iter().min().unwrap_or(&std::time::Duration::from_millis(0));
        let max_time = times.iter().max().unwrap_or(&std::time::Duration::from_millis(0));

        Ok(BenchmarkResult {
            shader_name: shader.name.clone(),
            iterations,
            total_time,
            average_time: avg_time,
            min_time: *min_time,
            max_time: *max_time,
            throughput: self.calculate_throughput(&config, &avg_time),
        })
    }

    fn calculate_throughput(&self, config: &ComputeConfig, time: &std::time::Duration) -> f64 {
        let work_items = config.workgroup_size * config.workgroup_count;
        let time_seconds = time.as_secs_f64();
        
        if time_seconds > 0.0 {
            work_items as f64 / time_seconds
        } else {
            0.0
        }
    }

    pub fn create_compute_shader_from_wgsl(&self, wgsl_source: &str) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        self.compile_shader(wgsl_source, "main").await
    }

    pub fn create_compute_shader_from_spirv(&self, spirv_data: &[u8]) -> Result<ComputeShader, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let shader_module = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: Some("SPIRV Compute Shader"),
                source: wgpu::util::make_spirv_raw(spirv_data),
            })
        };

        Ok(ComputeShader {
            id: uuid::Uuid::new_v4().to_string(),
            name: "SPIRV Compute Shader".to_string(),
            source: "SPIRV Binary".to_string(),
            entry_point: "main".to_string(),
            module: Arc::new(RwLock::new(Some(shader_module))),
            bind_group_layout: Arc::new(RwLock::new(None))),
            pipeline: Arc::new(RwLock::new(None))),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor: String,
    pub driver_version: String,
    pub device_type: DeviceType,
    pub memory: u64,
    pub features: Vec<String>,
    pub limits: DeviceLimits,
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub queue_type: String,
    pub max_workgroup_size: u32,
    pub max_workgroups_per_dispatch: u32,
    pub max_total_workgroups: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceLimits {
    pub max_compute_workgroup_size: u32,
    pub max_compute_workgroups_per_dispatch: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_count_x: u32,
    pub max_compute_workgroup_count_y: u32,
    pub max_compute_workgroup_count_z: u32,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub shader_name: String,
    pub iterations: u32,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
    pub min_time: std::time::Duration,
    pub max_time: std::time::Duration,
    pub throughput: f64,
}

#[derive(Debug, Clone)]
pub struct ComputePass {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>,
    pub queue: Arc<RwLock<Option<wgpu::Queue>>>,
    pub encoder: Arc<RwLock<Option<wgpu::CommandEncoder>>>,
    pub commands: Arc<RwLock<Vec<wgpu::CommandBuffer>>>,
}

impl Default for ComputeEngine {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Compute Engine".to_string(),
        )
    }
}

impl Default for ComputeConfig {
    fn default() -> Self {
        Self {
            shader_source: String::new(),
            entry_point: "main".to_string(),
            workgroup_size: 64,
            workgroup_count: 1,
            buffer_size: 1024,
            use_gpu: true,
            device_type: DeviceType::Discrete,
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Discrete
    }
}

impl Default for ComputeShader {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Compute Shader".to_string(),
            source: String::new(),
            entry_point: "main".to_string(),
            module: Arc::new(RwLock::new(None))),
            bind_group_layout: Arc::new(RwLock::new(None))),
            pipeline: Arc::new(RwLock::new(None))),
        }
    }
}

impl Default for ComputeBuffer {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Compute Buffer".to_string(),
            size: 1024,
            usage: wgpu::BufferUsages::STORAGE,
            buffer: Arc::new(RwLock::new(None))),
            mapped_data: Arc::new(RwLock::new(None))),
        }
    }
}

impl Default for ComputeResult {
    fn default() -> Self {
        Self {
            success: false,
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for ComputeWorkgroup {
    fn default() -> Self {
        Self {
            size: 64,
            count: 1,
            total_work_items: 64,
        }
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            name: "Default GPU".to_string(),
            vendor: "Unknown".to_string(),
            driver_version: "Unknown".to_string(),
            device_type: DeviceType::default(),
            memory: 0,
            features: Vec::new(),
            limits: DeviceLimits::default(),
        }
    }
}

impl Default for QueueInfo {
    fn default() -> Self {
        Self {
            queue_type: "Compute".to_string(),
            max_workgroup_size: 1024,
            max_workgroups_per_dispatch: 65535,
            max_total_workgroups: 65535,
        }
    }
}

impl Default for DeviceLimits {
    fn default() -> Self {
        Self {
            max_compute_workgroup_size: 1024,
            max_compute_workgroups_per_dispatch: 65535,
            max_compute_invocations_per_workgroup: 1024,
            max_compute_workgroup_count_x: 65535,
            max_compute_workgroup_count_y: 65535,
            max_compute_workgroup_count_z: 65535,
        }
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            shader_name: "Default Shader".to_string(),
            iterations: 1,
            total_time: std::time::Duration::from_millis(0),
            average_time: std::time::Duration::from_millis(0),
            min_time: std::time::Duration::from_millis(0),
            max_time: std::time::Duration::from_millis(0),
            throughput: 0.0,
        }
    }
}

impl Default for ComputePass {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Compute Pass".to_string(),
            device: Arc::new(RwLock::new(None))),
            queue: Arc::new(RwLock::new(None))),
            encoder: Arc::new(RwLock::new(None))),
            commands: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
