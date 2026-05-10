use std::sync::Arc;
use parking_lot::RwLock;
use wgpu::{Backends, InstanceDescriptor, PowerPreference, RequestAdapterOptions, RequestDeviceOptions, Features, Limits};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct GpuContext {
    pub id: String,
    pub name: String,
    pub instance: Arc<RwLock<Option<wgpu::Instance>>>>,
    pub adapter: Arc<RwLock<Option<wgpu::Adapter>>>>,
    pub device: Arc<RwLock<Option<wgpu::Device>>>>,
    pub queue: Arc<RwLock<Option<wgpu::Queue>>>>,
    pub surface: Arc<RwLock<Option<wgpu::Surface>>>>,
    pub event_sender: mpsc::UnboundedSender<GpuContextEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<GpuContextEvent>>>>,
    pub config: Arc<RwLock<GpuContextConfig>>,
    pub capabilities: Arc<RwLock<GpuCapabilities>>,
}

#[derive(Debug, Clone)]
pub enum GpuContextEvent {
    InstanceCreated,
    InstanceDestroyed,
    AdapterFound(String),
    AdapterSelected(String),
    DeviceCreated(String),
    DeviceLost(String),
    SurfaceCreated,
    SurfaceLost,
    Error(String),
    Warning(String),
}

#[derive(Debug, Clone)]
pub struct GpuContextConfig {
    pub backends: Vec<Backends>,
    pub power_preference: PowerPreference,
    pub required_features: Vec<Features>,
    pub required_limits: Limits,
    pub compatible_surface: bool,
    pub trace_path: Option<String>,
    pub validation_mode: ValidationMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationMode {
    Disabled,
    Error,
    Warning,
    Strict,
}

#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    pub supported_features: Vec<Features>,
    pub current_features: Vec<Features>,
    pub supported_limits: Limits,
    pub current_limits: Limits,
    pub adapter_info: Option<AdapterInfo>,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor: String,
    pub device_type: AdapterType,
    pub backend: Backends,
    pub driver_info: String,
    pub driver_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdapterType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: AdapterType,
    pub features: Vec<Features>,
    pub limits: Limits,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub memory_types: Vec<MemoryType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    DeviceLocal,
    HostVisible,
    HostCoherent,
    HostCached,
    Custom(String),
}

impl GpuContext {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            instance: Arc::new(RwLock::new(None))),
            adapter: Arc::new(RwLock::new(None))),
            device: Arc::new(RwLock::new(None))),
            queue: Arc::new(RwLock::new(None))),
            surface: Arc::new(RwLock::new(None))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
            config: Arc::new(RwLock::new(GpuContextConfig::default()))),
            capabilities: Arc::new(RwLock::new(GpuCapabilities::default()))),
        }
    }

    pub async fn initialize(&self, config: GpuContextConfig) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(GpuContextEvent::InstanceCreated);
        
Store configuration
        {
            let mut config_ref = self.config.write();
            *config_ref = config.clone();
        }

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: config.backends.clone(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        {
            let mut instance_ref = self.instance.write();
            *instance_ref = Some(instance);
        }

        let adapter = self.find_adapter(&instance, &config).await?;
        
        {
            let mut adapter_ref = self.adapter.write();
            *adapter_ref = Some(adapter.clone());
        }

        let _ = self.event_sender.send(GpuContextEvent::AdapterSelected(
            self.get_adapter_name(&adapter).unwrap_or_else(|| "Unknown".to_string())
        ));

        let device = self.request_device(&adapter, &config).await?;
        
        {
            let mut device_ref = self.device.write();
            *device_ref = Some(device.clone());
        }

        let _ = self.event_sender.send(GpuContextEvent::DeviceCreated(
            self.get_device_name(&device).unwrap_or_else(|| "Unknown".to_string())
        ));

        let queue = device.queue();
        
        {
            let mut queue_ref = self.queue.write();
            *queue_ref = Some(queue);
        }

        self.update_capabilities(&adapter, &device).await?;

        Ok(())
    }

    async fn find_adapter(&self, instance: &wgpu::Instance, config: &GpuContextConfig) -> Result<wgpu::Adapter, Box<dyn std::error::Error>> {
        let mut adapters = instance.enumerate_adapters(Backends::all());
        
        adapters.sort_by(|a, b| {
            let a_score = self.score_adapter(a, config);
            let b_score = self.score_adapter(b, config);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        for adapter in adapters {
            if self.is_adapter_compatible(&adapter, config).await {
                let _ = self.event_sender.send(GpuContextEvent::AdapterFound(
                    self.get_adapter_name(&adapter).unwrap_or_else(|| "Unknown".to_string())
                ));
                return Ok(adapter);
            }
        }

        Err("No compatible adapter found".into())
    }

    async fn request_device(&self, adapter: &wgpu::Adapter, config: &GpuContextConfig) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let mut features = Features::empty();
        
        let adapter_features = adapter.features();
        for feature in &config.required_features {
            if adapter_features.contains(*feature) {
                features |= *feature;
            }
        }

        let device = adapter.request_device(&RequestDeviceOptions {
            label: Some(&format!("{} Device", self.name)),
            features,
            limits: config.required_limits.clone(),
        }).await?;

        Ok(device)
    }

    async fn is_adapter_compatible(&self, adapter: &wgpu::Adapter, config: &GpuContextConfig) -> bool {
        let adapter_features = adapter.features();
        for feature in &config.required_features {
            if !adapter_features.contains(*feature) {
                return false;
            }
        }

        let adapter_limits = adapter.limits();
        if !self.check_limits_compatibility(&adapter_limits, &config.required_limits) {
            return false;
        }

        true
    }

    fn check_limits_compatibility(&self, adapter_limits: &Limits, required_limits: &Limits) -> bool {
        if adapter_limits.max_texture_dimension_2d < required_limits.max_texture_dimension_2d {
            return false;
        }
        
        if adapter_limits.max_texture_dimension_3d < required_limits.max_texture_dimension_3d {
            return false;
        }
        
        if adapter_limits.max_texture_array_layers < required_limits.max_texture_array_layers {
            return false;
        }
        
        if adapter_limits.max_bind_groups < required_limits.max_bind_groups {
            return false;
        }
        
        if adapter_limits.max_bindings_per_bind_group < required_limits.max_bindings_per_bind_group {
            return false;
        }
        
        if adapter_limits.max_dynamic_uniform_buffers_per_pipeline_layout < required_limits.max_dynamic_uniform_buffers_per_pipeline_layout {
            return false;
        }
        
        if adapter_limits.max_dynamic_storage_buffers_per_pipeline_layout < required_limits.max_dynamic_storage_buffers_per_pipeline_layout {
            return false;
        }
        
        if adapter_limits.max_sampled_textures_per_shader_stage < required_limits.max_sampled_textures_per_shader_stage {
            return false;
        }
        
        if adapter_limits.max_samplers_per_shader_stage < required_limits.max_samplers_per_shader_stage {
            return false;
        }
        
        if adapter_limits.max_storage_buffers_per_shader_stage < required_limits.max_storage_buffers_per_shader_stage {
            return false;
        }
        
        if adapter_limits.max_storage_textures_per_shader_stage < required_limits.max_storage_textures_per_shader_stage {
            return false;
        }
        
        if adapter_limits.max_uniform_buffers_per_shader_stage < required_limits.max_uniform_buffers_per_shader_stage {
            return false;
        }
        
        if adapter_limits.max_uniform_buffer_binding_size < required_limits.max_uniform_buffer_binding_size {
            return false;
        }
        
        if adapter_limits.max_push_constant_size < required_limits.max_push_constant_size {
            return false;
        }
        
        if adapter_limits.min_uniform_buffer_offset_alignment < required_limits.min_uniform_buffer_offset_alignment {
            return false;
        }
        
        if adapter_limits.min_storage_buffer_offset_alignment < required_limits.min_storage_buffer_offset_alignment {
            return false;
        }
        
        if adapter_limits.max_vertex_buffers < required_limits.max_vertex_buffers {
            return false;
        }
        
        if adapter_limits.max_vertex_buffer_array_stride < required_limits.max_vertex_buffer_array_stride {
            return false;
        }
        
        if adapter_limits.max_inter_stage_shader_components < required_limits.max_inter_stage_shader_components {
            return false;
        }
        
        if adapter_limits.max_compute_workgroup_storage_size < required_limits.max_compute_workgroup_storage_size {
            return false;
        }
        
        if adapter_limits.max_compute_invocations_per_workgroup < required_limits.max_compute_invocations_per_workgroup {
            return false;
        }
        
        if adapter_limits.max_compute_workgroup_size_x < required_limits.max_compute_workgroup_size_x {
            return false;
        }
        
        if adapter_limits.max_compute_workgroup_size_y < required_limits.max_compute_workgroup_size_y {
            return false;
        }
        
        if adapter_limits.max_compute_workgroup_size_z < required_limits.max_compute_workgroup_size_z {
            return false;
        }
        
        if adapter_limits.max_compute_workgroups_per_dimension < required_limits.max_compute_workgroups_per_dimension {
            return false;
        }
        
        if adapter_limits.max_compute_workgroups_per_dispatch < required_limits.max_compute_workgroups_per_dispatch {
            return false;
        }
        
        true
    }

    fn score_adapter(&self, adapter: &wgpu::Adapter, config: &GpuContextConfig) -> f32 {
        let mut score = 0.0;
        
        match self.get_adapter_type(adapter) {
            AdapterType::DiscreteGpu => score += 100.0,
            AdapterType::IntegratedGpu => score += 50.0,
            AdapterType::VirtualGpu => score += 20.0,
            AdapterType::Cpu => score += 10.0,
            AdapterType::Other => score += 5.0,
        }
        
        match config.power_preference {
            PowerPreference::HighPerformance => {
                if self.get_adapter_type(adapter) == AdapterType::DiscreteGpu {
                    score += 50.0;
                }
            },
            PowerPreference::LowPower => {
                if self.get_adapter_type(adapter) == AdapterType::IntegratedGpu {
                    score += 50.0;
                }
            },
            PowerPreference::Default => {
            },
        }
        
        let adapter_features = adapter.features();
        for feature in &config.required_features {
            if adapter_features.contains(*feature) {
                score += 10.0;
            }
        }
        
        score
    }

    async fn update_capabilities(&self, adapter: &wgpu::Adapter, device: &wgpu::Device) -> Result<(), Box<dyn std::error::Error>> {
        let mut capabilities = GpuCapabilities {
            supported_features: adapter.features().to_vec(),
            current_features: device.features().to_vec(),
            supported_limits: adapter.limits(),
            current_limits: device.limits(),
            adapter_info: Some(AdapterInfo {
                name: self.get_adapter_name(adapter).unwrap_or_else(|| "Unknown".to_string()),
                vendor: self.get_adapter_vendor(adapter).unwrap_or_else(|| "Unknown".to_string()),
                device_type: self.get_adapter_type(adapter),
                backend: self.get_adapter_backend(adapter),
                driver_info: self.get_adapter_driver_info(adapter).unwrap_or_else(|| "Unknown".to_string()),
                driver_version: self.get_adapter_driver_version(adapter).unwrap_or_else(|| "Unknown".to_string()),
            }),
            device_info: Some(DeviceInfo {
                name: self.get_device_name(device).unwrap_or_else(|| "Unknown".to_string()),
                device_type: self.get_adapter_type(adapter),
                features: device.features().to_vec(),
                limits: device.limits(),
                memory_info: MemoryInfo {
                    total_memory: 0,
                    available_memory: 0,
                    memory_types: vec![MemoryType::DeviceLocal, MemoryType::HostVisible],
                },
            }),
        };

        {
            let mut capabilities_ref = self.capabilities.write();
            *capabilities_ref = capabilities;
        }

        Ok(())
    }

    fn get_adapter_name(&self, adapter: &wgpu::Adapter) -> Option<String> {
        adapter.get_info().map(|info| info.name)
    }

    fn get_adapter_vendor(&self, adapter: &wgpu::Adapter) -> Option<String> {
        adapter.get_info().map(|info| info.vendor)
    }

    fn get_adapter_type(&self, adapter: &wgpu::Adapter) -> AdapterType {
        adapter.get_info()
            .map(|info| match info.device_type {
                wgpu::DeviceType::Other => AdapterType::Other,
                wgpu::DeviceType::IntegratedGpu => AdapterType::IntegratedGpu,
                wgpu::DeviceType::DiscreteGpu => AdapterType::DiscreteGpu,
                wgpu::DeviceType::VirtualGpu => AdapterType::VirtualGpu,
                wgpu::DeviceType::Cpu => AdapterType::Cpu,
            })
            .unwrap_or(AdapterType::Other)
    }

    fn get_adapter_backend(&self, adapter: &wgpu::Adapter) -> Backends {
        adapter.get_info()
            .map(|info| info.backend)
            .unwrap_or(Backends::VULKAN)
    }

    fn get_adapter_driver_info(&self, adapter: &wgpu::Adapter) -> Option<String> {
        adapter.get_info().map(|info| info.driver)
    }

    fn get_adapter_driver_version(&self, adapter: &wgpu::Adapter) -> Option<String> {
        adapter.get_info().map(|info| info.driver_info)
    }

    fn get_device_name(&self, device: &wgpu::Device) -> Option<String> {
        Some(format!("GPU Device {}", uuid::Uuid::new_v4()))
    }

    pub async fn create_surface(&self, window: &winit::window::Window) -> Result<(), Box<dyn std::error::Error>> {
        let instance = self.get_instance().await?;
        let surface = unsafe { instance.create_surface(&window) };

        {
            let mut surface_ref = self.surface.write();
            *surface_ref = Some(surface);
        }

        let _ = self.event_sender.send(GpuContextEvent::SurfaceCreated);
        Ok(())
    }

    pub fn destroy_surface(&self) {
        {
            let mut surface_ref = self.surface.write();
            *surface_ref = None;
        }

        let _ = self.event_sender.send(GpuContextEvent::SurfaceLost);
    }

    pub async fn get_instance(&self) -> Result<wgpu::Instance, Box<dyn std::error::Error>> {
        let instance_ref = self.instance.read();
        instance_ref.as_ref().ok_or("Instance not initialized")?.clone()
    }

    pub async fn get_adapter(&self) -> Result<wgpu::Adapter, Box<dyn std::error::Error>> {
        let adapter_ref = self.adapter.read();
        adapter_ref.as_ref().ok_or("Adapter not initialized")?.clone()
    }

    pub async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    pub async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let queue_ref = self.queue.read();
        queue_ref.as_ref().ok_or("Queue not initialized")?.clone()
    }

    pub async fn get_surface(&self) -> Result<Option<wgpu::Surface>, Box<dyn std::error::Error>> {
        let surface_ref = self.surface.read();
        Ok(surface_ref.clone())
    }

    pub fn get_config(&self) -> GpuContextConfig {
        self.config.read().clone()
    }

    pub fn get_capabilities(&self) -> GpuCapabilities {
        self.capabilities.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<GpuContextEvent> {
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

    pub fn is_initialized(&self) -> bool {
        let instance_ref = self.instance.read();
        let adapter_ref = self.adapter.read();
        let device_ref = self.device.read();
        let queue_ref = self.queue.read();
        
        instance_ref.is_some() && 
        adapter_ref.is_some() && 
        device_ref.is_some() && 
        queue_ref.is_some()
    }

    pub fn is_surface_created(&self) -> bool {
        let surface_ref = self.surface.read();
        surface_ref.is_some()
    }

    pub fn get_adapter_info(&self) -> Option<AdapterInfo> {
        self.capabilities.read().adapter_info.clone()
    }

    pub fn get_device_info(&self) -> Option<DeviceInfo> {
        self.capabilities.read().device_info.clone()
    }

    pub fn get_memory_info(&self) -> Option<MemoryInfo> {
        self.capabilities.read().device_info.as_ref().map(|info| info.memory_info.clone())
    }

    pub fn supports_feature(&self, feature: Features) -> bool {
        let capabilities = self.capabilities.read();
        capabilities.current_features.contains(&feature)
    }

    pub fn supports_all_features(&self, features: &[Features]) -> bool {
        let capabilities = self.capabilities.read();
        features.iter().all(|f| capabilities.current_features.contains(f))
    }

    pub fn get_feature_support(&self) -> Vec<(Features, bool)> {
        let capabilities = self.capabilities.read();
        let mut feature_support = Vec::new();
        
        for feature in &capabilities.supported_features {
            feature_support.push((*feature, capabilities.current_features.contains(feature)));
        }
        
        feature_support
    }

    pub fn check_limit_support(&self, required_limit: &str, required_value: u32) -> bool {
        let capabilities = self.capabilities.read();
        
        match required_limit {
            "max_texture_dimension_2d" => capabilities.current_limits.max_texture_dimension_2d >= required_value,
            "max_texture_dimension_3d" => capabilities.current_limits.max_texture_dimension_3d >= required_value,
            "max_texture_array_layers" => capabilities.current_limits.max_texture_array_layers >= required_value,
            "max_bind_groups" => capabilities.current_limits.max_bind_groups >= required_value,
            "max_bindings_per_bind_group" => capabilities.current_limits.max_bindings_per_bind_group >= required_value,
            "max_dynamic_uniform_buffers_per_pipeline_layout" => capabilities.current_limits.max_dynamic_uniform_buffers_per_pipeline_layout >= required_value,
            "max_dynamic_storage_buffers_per_pipeline_layout" => capabilities.current_limits.max_dynamic_storage_buffers_per_pipeline_layout >= required_value,
            "max_sampled_textures_per_shader_stage" => capabilities.current_limits.max_sampled_textures_per_shader_stage >= required_value,
            "max_samplers_per_shader_stage" => capabilities.current_limits.max_samplers_per_shader_stage >= required_value,
            "max_storage_buffers_per_shader_stage" => capabilities.current_limits.max_storage_buffers_per_shader_stage >= required_value,
            "max_storage_textures_per_shader_stage" => capabilities.current_limits.max_storage_textures_per_shader_stage >= required_value,
            "max_uniform_buffers_per_shader_stage" => capabilities.current_limits.max_uniform_buffers_per_shader_stage >= required_value,
            "max_uniform_buffer_binding_size" => capabilities.current_limits.max_uniform_buffer_binding_size >= required_value as u32,
            "max_storage_buffer_offset_alignment" => capabilities.current_limits.max_storage_buffer_offset_alignment >= required_value as u32,
            "min_uniform_buffer_offset_alignment" => capabilities.current_limits.min_uniform_buffer_offset_alignment >= required_value as u32,
            "max_vertex_buffers" => capabilities.current_limits.max_vertex_buffers >= required_value,
            "max_vertex_buffer_array_stride" => capabilities.current_limits.max_vertex_buffer_array_stride >= required_value,
            "max_inter_stage_shader_components" => capabilities.current_limits.max_inter_stage_shader_components >= required_value,
            "max_compute_workgroup_storage_size" => capabilities.current_limits.max_compute_workgroup_storage_size >= required_value,
            "max_compute_invocations_per_workgroup" => capabilities.current_limits.max_compute_invocations_per_workgroup >= required_value,
            "max_compute_workgroup_size_x" => capabilities.current_limits.max_compute_workgroup_size_x >= required_value,
            "max_compute_workgroup_size_y" => capabilities.current_limits.max_compute_workgroup_size_y >= required_value,
            "max_compute_workgroup_size_z" => capabilities.current_limits.max_compute_workgroup_size_z >= required_value,
            "max_compute_workgroups_per_dimension" => capabilities.current_limits.max_compute_workgroups_per_dimension >= required_value,
            "max_compute_workgroups_per_dispatch" => capabilities.current_limits.max_compute_workgroups_per_dispatch >= required_value,
            "max_push_constant_size" => capabilities.current_limits.max_push_constant_size >= required_value,
            _ => false,
        }
    }

    pub fn get_limit_value(&self, limit_name: &str) -> Option<u32> {
        let capabilities = self.capabilities.read();
        
        match limit_name {
            "max_texture_dimension_2d" => Some(capabilities.current_limits.max_texture_dimension_2d),
            "max_texture_dimension_3d" => Some(capabilities.current_limits.max_texture_dimension_3d),
            "max_texture_array_layers" => Some(capabilities.current_limits.max_texture_array_layers),
            "max_bind_groups" => Some(capabilities.current_limits.max_bind_groups),
            "max_bindings_per_bind_group" => Some(capabilities.current_limits.max_bindings_per_bind_group),
            "max_dynamic_uniform_buffers_per_pipeline_layout" => Some(capabilities.current_limits.max_dynamic_uniform_buffers_per_pipeline_layout),
            "max_dynamic_storage_buffers_per_pipeline_layout" => Some(capabilities.current_limits.max_dynamic_storage_buffers_per_pipeline_layout),
            "max_sampled_textures_per_shader_stage" => Some(capabilities.current_limits.max_sampled_textures_per_shader_stage),
            "max_samplers_per_shader_stage" => Some(capabilities.current_limits.max_samplers_per_shader_stage),
            "max_storage_buffers_per_shader_stage" => Some(capabilities.current_limits.max_storage_buffers_per_shader_stage),
            "max_storage_textures_per_shader_stage" => Some(capabilities.current_limits.max_storage_textures_per_shader_stage),
            "max_uniform_buffers_per_shader_stage" => Some(capabilities.current_limits.max_uniform_buffers_per_shader_stage),
            "max_uniform_buffer_binding_size" => Some(capabilities.current_limits.max_uniform_buffer_binding_size),
            "max_storage_buffer_offset_alignment" => Some(capabilities.current_limits.max_storage_buffer_offset_alignment as u32),
            "min_uniform_buffer_offset_alignment" => Some(capabilities.current_limits.min_uniform_buffer_offset_alignment as u32),
            "max_vertex_buffers" => Some(capabilities.current_limits.max_vertex_buffers),
            "max_vertex_buffer_array_stride" => Some(capabilities.current_limits.max_vertex_buffer_array_stride),
            "max_inter_stage_shader_components" => Some(capabilities.current_limits.max_inter_stage_shader_components),
            "max_compute_workgroup_storage_size" => Some(capabilities.current_limits.max_compute_workgroup_storage_size),
            "max_compute_invocations_per_workgroup" => Some(capabilities.current_limits.max_compute_invocations_per_workgroup),
            "max_compute_workgroup_size_x" => Some(capabilities.current_limits.max_compute_workgroup_size_x),
            "max_compute_workgroup_size_y" => Some(capabilities.current_limits.max_compute_workgroup_size_y),
            "max_compute_workgroup_size_z" => Some(capabilities.current_limits.max_compute_workgroup_size_z),
            "max_compute_workgroups_per_dimension" => Some(capabilities.current_limits.max_compute_workgroups_per_dimension),
            "max_compute_workgroups_per_dispatch" => Some(capabilities.current_limits.max_compute_workgroups_per_dispatch),
            "max_push_constant_size" => Some(capabilities.current_limits.max_push_constant_size),
            _ => None,
        }
    }

    pub fn reset(&self) {
        {
            let mut instance_ref = self.instance.write();
            *instance_ref = None;
        }
        
        {
            let mut adapter_ref = self.adapter.write();
            *adapter_ref = None;
        }
        
        {
            let mut device_ref = self.device.write();
            *device_ref = None;
        }
        
        {
            let mut queue_ref = self.queue.write();
            *queue_ref = None;
        }
        
        {
            let mut surface_ref = self.surface.write();
            *surface_ref = None;
        }
        
        {
            let mut capabilities_ref = self.capabilities.write();
            *capabilities_ref = GpuCapabilities::default();
        }
    }

    pub fn clone_context(&self) -> GpuContext {
        GpuContext {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", self.name),
            instance: Arc::new(RwLock::new(self.instance.read().clone()))),
            adapter: Arc::new(RwLock::new(self.adapter.read().clone()))),
            device: Arc::new(RwLock::new(self.device.read().clone()))),
            queue: Arc::new(RwLock::new(self.queue.read().clone()))),
            surface: Arc::new(RwLock::new(self.surface.read().clone()))),
            event_sender: self.event_sender.clone(),
            event_receiver: Arc::new(RwLock::new(None))),
            config: Arc::new(RwLock::new(self.config.read().clone()))),
            capabilities: Arc::new(RwLock::new(self.capabilities.read().clone()))),
        }
    }

    pub fn validate_configuration(&self, config: &GpuContextConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.required_features.is_empty() {
            return Err("At least one required feature must be specified".into());
        }
        
        if config.required_limits.max_texture_dimension_2d == 0 {
            return Err("Invalid max_texture_dimension_2d limit".into());
        }
        
        if config.required_limits.max_texture_dimension_3d == 0 {
            return Err("Invalid max_texture_dimension_3d limit".into());
        }
        
        if config.required_limits.max_bind_groups == 0 {
            return Err("Invalid max_bind_groups limit".into());
        }
        
        Ok(())
    }

    pub fn create_optimal_config(&self, features: Vec<Features>, min_texture_size: u32) -> GpuContextConfig {
        GpuContextConfig {
            backends: Backends::all(),
            power_preference: PowerPreference::HighPerformance,
            required_features: features,
            required_limits: Limits {
                max_texture_dimension_2d: min_texture_size.max(256),
                max_texture_dimension_3d: min_texture_size.max(256),
                max_texture_array_layers: 256,
                max_bind_groups: 8,
                max_bindings_per_bind_group: 1000,
                max_dynamic_uniform_buffers_per_pipeline_layout: 8,
                max_dynamic_storage_buffers_per_pipeline_layout: 4,
                max_sampled_textures_per_shader_stage: 16,
                max_samplers_per_shader_stage: 16,
                max_storage_buffers_per_shader_stage: 8,
                max_storage_textures_per_shader_stage: 8,
                max_uniform_buffers_per_shader_stage: 16,
                max_uniform_buffer_binding_size: 16384,
                max_storage_buffer_offset_alignment: 4,
                min_uniform_buffer_offset_alignment: 256,
                max_vertex_buffers: 16,
                max_vertex_buffer_array_stride: 2048,
                max_inter_stage_shader_components: 60,
                max_compute_workgroup_storage_size: 16384,
                max_compute_invocations_per_workgroup: 256,
                max_compute_workgroup_size_x: 256,
                max_compute_workgroup_size_y: 256,
                max_compute_workgroup_size_z: 64,
                max_compute_workgroups_per_dimension: 65535,
                max_compute_workgroups_per_dispatch: 65535,
                max_push_constant_size: 128,
            },
            compatible_surface: true,
            trace_path: None,
            validation_mode: ValidationMode::Warning,
        }
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "GPU Context".to_string(),
        )
    }
}

impl Default for GpuContextConfig {
    fn default() -> Self {
        Self {
            backends: Backends::all(),
            power_preference: PowerPreference::Default,
            required_features: vec![Features::PUSH_CONSTANTS],
            required_limits: Limits::default(),
            compatible_surface: true,
            trace_path: None,
            validation_mode: ValidationMode::Warning,
        }
    }
}

impl Default for GpuCapabilities {
    fn default() -> Self {
        Self {
            supported_features: Vec::new(),
            current_features: Vec::new(),
            supported_limits: Limits::default(),
            current_limits: Limits::default(),
            adapter_info: None,
            device_info: None,
        }
    }
}

impl Default for AdapterInfo {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            vendor: "Unknown".to_string(),
            device_type: AdapterType::Other,
            backend: Backends::VULKAN,
            driver_info: "Unknown".to_string(),
            driver_version: "Unknown".to_string(),
        }
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            name: "Unknown Device".to_string(),
            device_type: AdapterType::Other,
            features: Vec::new(),
            limits: Limits::default(),
            memory_info: MemoryInfo::default(),
        }
    }
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            total_memory: 0,
            available_memory: 0,
            memory_types: vec![MemoryType::DeviceLocal],
        }
    }
}

impl Default for ValidationMode {
    fn default() -> Self {
        ValidationMode::Warning
    }
}
