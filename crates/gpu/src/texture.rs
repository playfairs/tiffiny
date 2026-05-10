use std::sync::Arc;
use parking_lot::RwLock;
use wgpu::{TextureUsages, TextureFormat, TextureDimension};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct TextureManager {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>>,
    pub textures: Arc<RwLock<std::collections::HashMap<String, GpuTexture>>>>,
    pub event_sender: mpsc::UnboundedSender<TextureEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TextureEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum TextureEvent {
    TextureCreated(String),
    TextureDestroyed(String),
    TextureUploaded(String, u64),
    TextureDownloaded(String, u64),
    TextureBound(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct GpuTexture {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub texture: Arc<RwLock<Option<wgpu::Texture>>>>,
    pub view: Arc<RwLock<Option<wgpu::TextureView>>>>,
    pub sampler: Arc<RwLock<Option<wgpu::Sampler>>>>,
    pub memory_usage: Arc<RwLock<TextureMemoryUsage>>>,
}

#[derive(Debug, Clone)]
pub struct TextureMemoryUsage {
    pub allocated_size: u64,
    pub used_size: u64,
    pub peak_usage: u64,
    pub fragmentation: f32,
    pub allocation_count: u32,
}

#[derive(Debug, Clone)]
pub struct TextureConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub dimension: TextureDimension,
    pub mip_levels: u32,
    pub sample_count: u32,
    pub label: Option<String>,
    pub memory_type: MemoryType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    DeviceLocal,
    HostVisible,
    HostCoherent,
    HostMapped,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TextureUpload {
    pub texture: GpuTexture,
    pub data: Vec<u8>,
    pub offset: (u32, u32, u32),
    pub size: (u32, u32, u32),
    pub mip_level: u32,
}

#[derive(Debug, Clone)]
pub struct TextureDownload {
    pub texture: GpuTexture,
    pub data: Arc<RwLock<Option<Vec<u8>>>>>,
    pub offset: (u32, u32, u32),
    pub size: (u32, u32, u32),
    pub mip_level: u32,
}

#[derive(Debug, Clone)]
pub struct TextureBinding {
    pub texture: GpuTexture,
    pub binding: u32,
    pub view_type: TextureViewType,
    pub sampler_type: SamplerType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureViewType {
    Full,
    MipLevel(u32),
    ArrayLayer(u32),
    CubemapFace(u32),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SamplerType {
    Point,
    Linear,
    Anisotropic,
    Custom(String),
}

impl TextureManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            textures: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub fn set_device(&self, device: wgpu::Device) {
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
    }

    pub async fn create_texture(&self, config: TextureConfig) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: config.label.as_deref(),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: config.depth,
            },
            mip_level_count: config.mip_levels,
            sample_count: config.sample_count,
            dimension: config.dimension,
            format: config.format,
            usage: config.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} View", config.label.as_deref().unwrap_or("Texture"))),
            format: Some(config.format),
            dimension: Some(config.dimension),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", config.label.as_deref().unwrap_or("Texture"))),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 16,
            border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
        });

        let gpu_texture = GpuTexture {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.label.unwrap_or_else(|| format!("Texture {}", uuid::Uuid::new_v4())),
            width: config.width,
            height: config.height,
            depth: config.depth,
            format: config.format,
            usage: config.usage,
            texture: Arc::new(RwLock::new(Some(texture))),
            view: Arc::new(RwLock::new(Some(view))),
            sampler: Arc::new(RwLock::new(Some(sampler))),
            memory_usage: Arc::new(RwLock::new(TextureMemoryUsage::default())),
        };

Add to texture manager
        {
            let mut textures = self.textures.write();
            textures.insert(gpu_texture.id.clone(), gpu_texture.clone());
        }

        let _ = self.event_sender.send(TextureEvent::TextureCreated(gpu_texture.id.clone()));
        Ok(gpu_texture)
    }

    pub async fn create_2d_texture(&self, width: u32, height: u32, format: TextureFormat, usage: TextureUsages) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let config = TextureConfig {
            width,
            height,
            depth: 1,
            format,
            usage,
            dimension: TextureDimension::D2,
            mip_levels: 1,
            sample_count: 1,
            label: Some(format!("2D Texture {}x{}", width, height)),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_texture(config).await
    }

    pub async fn create_3d_texture(&self, width: u32, height: u32, depth: u32, format: TextureFormat, usage: TextureUsages) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let config = TextureConfig {
            width,
            height,
            depth,
            format,
            usage,
            dimension: TextureDimension::D3,
            mip_levels: 1,
            sample_count: 1,
            label: Some(format!("3D Texture {}x{}x{}", width, height, depth)),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_texture(config).await
    }

    pub async fn create_cube_texture(&self, size: u32, format: TextureFormat, usage: TextureUsages) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let config = TextureConfig {
            width: size,
            height: size,
            depth: 6,
            format,
            usage,
            dimension: TextureDimension::D2,
            mip_levels: 1,
            sample_count: 1,
            label: Some(format!("Cube Texture {}x{}", size, size)),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_texture(config).await
    }

    pub async fn create_array_texture(&self, width: u32, height: u32, array_size: u32, format: TextureFormat, usage: TextureUsages) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let config = TextureConfig {
            width,
            height,
            depth: array_size,
            format,
            usage,
            dimension: TextureDimension::D2,
            mip_levels: 1,
            sample_count: 1,
            label: Some(format!("Array Texture {}x{}x{}", width, height, array_size)),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_texture(config).await
    }

    pub async fn create_render_target(&self, width: u32, height: u32, format: TextureFormat) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        self.create_2d_texture(width, height, format, usage).await
    }

    pub async fn create_depth_stencil_target(&self, width: u32, height: u32) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        let format = TextureFormat::Depth24PlusStencil8;
        let usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        self.create_2d_texture(width, height, format, usage).await
    }

    pub fn destroy_texture(&self, texture_id: &str) -> bool {
        let mut textures = self.textures.write();
        
        if let Some(texture) = textures.remove(texture_id) {
            let _ = self.event_sender.send(TextureEvent::TextureDestroyed(texture_id.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get_texture(&self, texture_id: &str) -> Option<GpuTexture> {
        let textures = self.textures.read();
        textures.get(texture_id).cloned()
    }

    pub fn list_textures(&self) -> Vec<GpuTexture> {
        let textures = self.textures.read();
        textures.values().cloned().collect()
    }

    pub fn get_texture_count(&self) -> usize {
        let textures = self.textures.read();
        textures.len()
    }

    pub fn find_textures_by_format(&self, format: TextureFormat) -> Vec<GpuTexture> {
        let textures = self.textures.read();
        textures.values()
            .filter(|texture| texture.format == format)
            .cloned()
            .collect()
    }

    pub fn find_textures_by_usage(&self, usage: TextureUsages) -> Vec<GpuTexture> {
        let textures = self.textures.read();
        textures.values()
            .filter(|texture| texture.usage.contains(usage))
            .cloned()
            .collect()
    }

    pub async fn upload_texture_data(&self, texture: &GpuTexture, data: &[u8], offset: (u32, u32, u32)) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Upload Staging Buffer"),
            size: data.len() as u64,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });

        {
            let slice = staging_buffer.slice(..);
            let mut mapped = slice.get_mapped_range_mut().map_err(|e| format!("Failed to map buffer: {:?}", e))?;
            mapped.copy_from_slice(data);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Upload Encoder"),
        });

        {
            let texture_ref = texture.texture.read();
            let tex = texture_ref.as_ref().ok_or("Texture not initialized")?;

            let buffer_size = data.len() as u64;
            let bytes_per_row = (texture.width * self.get_bytes_per_pixel(texture.format)) as u64;
            let rows_per_copy = (buffer_size + bytes_per_row - 1) / bytes_per_row;

            encoder.copy_buffer_to_texture(
                &staging_buffer,
                wgpu::ImageCopyBuffer {
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(rows_per_copy),
                    },
                    offset: wgpu::Origin3d { x: offset.0, y: offset.1, z: offset.2 },
                    copy_extent: wgpu::Extent3d {
                        width: texture.width,
                        height: texture.height,
                        depth_or_array_layers: 1,
                    },
                },
                &tex,
                wgpu::ImageCopyTexture {
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
            );
        }

        let command_buffer = encoder.finish();
        let _ = queue.submit(Some(command_buffer));
        device.poll(wgpu::Maintain::Wait);

        let _ = self.event_sender.send(TextureEvent::TextureUploaded(texture.id.clone(), data.len() as u64));
        Ok(())
    }

    async fn download_texture_data(&self, texture: &GpuTexture, offset: (u32, u32, u32), size: (u32, u32, u32)) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let bytes_per_pixel = self.get_bytes_per_pixel(texture.format);
        let buffer_size = (size.0 * size.1 * bytes_per_pixel) as u64;

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Download Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Download Encoder"),
        });

        {
            let texture_ref = texture.texture.read();
            let tex = texture_ref.as_ref().ok_or("Texture not initialized")?;

            encoder.copy_texture_to_buffer(
                &tex,
                wgpu::ImageCopyTexture {
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: offset.0, y: offset.1, z: offset.2 },
                    aspect: wgpu::TextureAspect::All,
                },
                &readback_buffer,
                wgpu::ImageCopyBuffer {
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((size.0 * bytes_per_pixel) as u64),
                        rows_per_image: Some(size.1 as u64),
                    },
                    offset: wgpu::Origin3d::ZERO,
                    copy_extent: wgpu::Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                },
            );
        }

        let command_buffer = encoder.finish();
        let _ = queue.submit(Some(command_buffer));
        device.poll(wgpu::Maintain::Wait);

        let slice = readback_buffer.slice(..);
        let data = slice.get_mapped_range().map_err(|e| format!("Failed to map buffer: {:?}", e))?;
        
        let _ = self.event_sender.send(TextureEvent::TextureDownloaded(texture.id.clone(), data.len() as u64));
        Ok(data.to_vec())
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

    fn get_bytes_per_pixel(&self, format: TextureFormat) -> u32 {
        match format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Depth24PlusStencil8 => 4,
            TextureFormat::Depth32Float => 4,
            _ => 4,
        }
    }

    fn generate_mipmaps(&self, texture: &GpuTexture) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let texture_ref = texture.texture.read();
            let tex = texture_ref.as_ref().ok_or("Texture not initialized")?;

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mipmap Generation Encoder"),
            });

            let command_buffer = encoder.finish();
            
            let queue = self.get_queue().await?;
            let _ = queue.submit(Some(command_buffer));
            device.poll(wgpu::Maintain::Wait);
        }

        Ok(())
    }

    pub fn get_memory_usage(&self) -> TextureMemoryUsage {
        let textures = self.textures.read();
        let total_size: u64 = textures.values()
            .map(|t| (t.width * t.height * t.depth * self.get_bytes_per_pixel(t.format)) as u64)
            .sum();

        TextureMemoryUsage {
            allocated_size: total_size,
            used_size: total_size,
            peak_usage: total_size,
            fragmentation: 0.0,
            allocation_count: textures.len() as u32,
        }
    }

    pub fn get_texture_info(&self, texture: &GpuTexture) -> TextureInfo {
        TextureInfo {
            id: texture.id.clone(),
            name: texture.name.clone(),
            width: texture.width,
            height: texture.height,
            depth: texture.depth,
            format: texture.format,
            usage: texture.usage,
            memory_type: MemoryType::DeviceLocal,
            has_view: texture.view.read().is_some(),
            has_sampler: texture.sampler.read().is_some(),
            memory_usage: *texture.memory_usage.read(),
        }
    }

    pub async fn get_events(&mut self) -> Vec<TextureEvent> {
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

    pub fn clone_texture(&self, texture: &GpuTexture) -> GpuTexture {
        GpuTexture {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", texture.name),
            width: texture.width,
            height: texture.height,
            depth: texture.depth,
            format: texture.format,
            usage: texture.usage,
            texture: Arc::new(RwLock::new(None))),
            view: Arc::new(RwLock::new(None))),
            sampler: Arc::new(RwLock::new(None))),
            memory_usage: Arc::new(RwLock::new(TextureMemoryUsage::default())),
        }
    }

    pub fn create_texture_binding(&self, texture: &GpuTexture, binding: u32, view_type: TextureViewType, sampler_type: SamplerType) -> TextureBinding {
        TextureBinding {
            texture: texture.clone(),
            binding,
            view_type,
            sampler_type,
        }
    }

    pub fn create_texture_array(&self, textures: &[GpuTexture]) -> Result<TextureArray, Box<dyn std::error::Error>> {
        if textures.is_empty() {
            return Err("Cannot create texture array from empty list".into());
        }

        let first_texture = &textures[0];
        for texture in textures {
            if texture.width != first_texture.width || 
               texture.height != first_texture.height || 
               texture.format != first_texture.format {
                return Err("All textures in array must have same dimensions and format".into());
            }
        }

        Ok(TextureArray {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("Texture Array {}", textures.len()),
            textures: textures.to_vec(),
            width: first_texture.width,
            height: first_texture.height,
            format: first_texture.format,
            depth: textures.len() as u32,
        })
    }

    pub fn cleanup_unused_textures(&self) -> usize {
        let mut textures = self.textures.write();
        let initial_count = textures.len();
        
        textures.retain(|_, texture| {
            true
        });

        let cleaned_count = initial_count - textures.len();
        let _ = self.event_sender.send(TextureEvent::Error(format!("Cleaned up {} unused textures", cleaned_count)));
        
        cleaned_count
    }

    pub fn reset(&self) {
        let mut textures = self.textures.write();
        textures.clear();
    }
}

#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub memory_type: MemoryType,
    pub has_view: bool,
    pub has_sampler: bool,
    pub memory_usage: TextureMemoryUsage,
}

#[derive(Debug, Clone)]
pub struct TextureArray {
    pub id: String,
    pub name: String,
    pub textures: Vec<GpuTexture>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub depth: u32,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Texture Manager".to_string(),
        )
    }
}

impl Default for GpuTexture {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Texture".to_string(),
            width: 256,
            height: 256,
            depth: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            texture: Arc::new(RwLock::new(None))),
            view: Arc::new(RwLock::new(None))),
            sampler: Arc::new(RwLock::new(None))),
            memory_usage: Arc::new(RwLock::new(TextureMemoryUsage::default())),
        }
    }
}

impl Default for TextureMemoryUsage {
    fn default() -> Self {
        Self {
            allocated_size: 0,
            used_size: 0,
            peak_usage: 0,
            fragmentation: 0.0,
            allocation_count: 0,
        }
    }
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            depth: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            dimension: TextureDimension::D2,
            mip_levels: 1,
            sample_count: 1,
            label: None,
            memory_type: MemoryType::DeviceLocal,
        }
    }
}

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::DeviceLocal
    }
}

impl Default for TextureUpload {
    fn default() -> Self {
        Self {
            texture: GpuTexture::default(),
            data: Vec::new(),
            offset: (0, 0, 0),
            size: (0, 0, 0),
            mip_level: 0,
        }
    }
}

impl Default for TextureDownload {
    fn default() -> Self {
        Self {
            texture: GpuTexture::default(),
            data: Arc::new(RwLock::new(None))),
            offset: (0, 0, 0),
            size: (0, 0, 0),
            mip_level: 0,
        }
    }
}

impl Default for TextureBinding {
    fn default() -> Self {
        Self {
            texture: GpuTexture::default(),
            binding: 0,
            view_type: TextureViewType::Full,
            sampler_type: SamplerType::Linear,
        }
    }
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Texture".to_string(),
            width: 256,
            height: 256,
            depth: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            memory_type: MemoryType::default(),
            has_view: false,
            has_sampler: false,
            memory_usage: TextureMemoryUsage::default(),
        }
    }
}

impl Default for TextureArray {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Texture Array".to_string(),
            textures: Vec::new(),
            width: 256,
            height: 256,
            format: TextureFormat::Rgba8Unorm,
            depth: 1,
        }
    }
}
