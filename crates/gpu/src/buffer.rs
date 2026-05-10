use std::sync::Arc;
use parking_lot::RwLock;
use wgpu::BufferUsages;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BufferManager {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>,
    pub buffers: Arc<RwLock<std::collections::HashMap<String, GpuBuffer>>>>,
    pub event_sender: mpsc::UnboundedSender<BufferEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<BufferEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum BufferEvent {
    BufferCreated(String),
    BufferDestroyed(String),
    BufferMapped(String),
    BufferUnmapped(String),
    BufferWritten(String, u64),
    BufferRead(String, u64),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub usage: BufferUsages,
    pub buffer: Arc<RwLock<Option<wgpu::Buffer>>>>,
    pub mapped_data: Arc<RwLock<Option<Vec<u8>>>>,
    pub staging_buffer: Arc<RwLock<Option<wgpu::Buffer>>>>,
    pub memory_usage: Arc<RwLock<BufferMemoryUsage>>>,
}

#[derive(Debug, Clone)]
pub struct BufferMemoryUsage {
    pub allocated_size: u64,
    pub used_size: u64,
    pub peak_usage: u64,
    pub fragmentation: f32,
    pub allocation_count: u32,
}

#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub size: u64,
    pub usage: BufferUsages,
    pub mapped_at_creation: bool,
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
pub struct BufferAllocation {
    pub buffer: GpuBuffer,
    pub offset: u64,
    pub size: u64,
    pub timestamp: std::time::Instant,
}

impl BufferManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            buffers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn set_device(&self, device: wgpu::Device) {
        let mut device_ref = self.device.write();
        *device_ref = Some(device);
    }

    pub async fn create_buffer(&self, config: BufferConfig) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: config.label.as_deref(),
            size: config.size,
            usage: config.usage,
            mapped_at_creation: config.mapped_at_creation,
        });

        let mapped_data = if config.mapped_at_creation {
            let data = vec![0u8; config.size as usize];
            Some(data)
        } else {
            None
        };

        let gpu_buffer = GpuBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.label.unwrap_or_else(|| format!("Buffer {}", uuid::Uuid::new_v4())),
            size: config.size,
            usage: config.usage,
            buffer: Arc::new(RwLock::new(Some(buffer))),
            mapped_data: Arc::new(RwLock::new(mapped_data))),
            staging_buffer: Arc::new(RwLock::new(None))),
            memory_usage: Arc::new(RwLock::new(BufferMemoryUsage::default())),
        };

Add to buffer manager
        {
            let mut buffers = self.buffers.write();
            buffers.insert(gpu_buffer.id.clone(), gpu_buffer.clone());
        }

        let _ = self.event_sender.send(BufferEvent::BufferCreated(gpu_buffer.id.clone()));
        
        Ok(gpu_buffer)
    }

    pub async fn create_staging_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: true,
            label: Some("Staging Buffer".to_string()),
            memory_type: MemoryType::HostVisible,
        };

        self.create_buffer(config).await
    }

    pub async fn create_vertex_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Vertex Buffer".to_string()),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_buffer(config).await
    }

    pub async fn create_index_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Index Buffer".to_string()),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_buffer(config).await
    }

    pub async fn create_uniform_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Uniform Buffer".to_string()),
            memory_type: MemoryType::HostVisible,
        };

        self.create_buffer(config).await
    }

    pub async fn create_storage_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Storage Buffer".to_string()),
            memory_type: MemoryType::DeviceLocal,
        };

        self.create_buffer(config).await
    }

    pub async fn create_readback_buffer(&self, size: u64) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
        let config = BufferConfig {
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: true,
            label: Some("Readback Buffer".to_string()),
            memory_type: MemoryType::HostVisible,
        };

        self.create_buffer(config).await
    }

    pub fn destroy_buffer(&self, buffer_id: &str) -> bool {
        let mut buffers = self.buffers.write();
        
        if let Some(buffer) = buffers.remove(buffer_id) {
            let _ = self.event_sender.send(BufferEvent::BufferDestroyed(buffer_id.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get_buffer(&self, buffer_id: &str) -> Option<GpuBuffer> {
        let buffers = self.buffers.read();
        buffers.get(buffer_id).cloned()
    }

    pub fn list_buffers(&self) -> Vec<GpuBuffer> {
        let buffers = self.buffers.read();
        buffers.values().cloned().collect()
    }

    pub fn get_buffer_count(&self) -> usize {
        let buffers = self.buffers.read();
        buffers.len()
    }

    pub async fn write_buffer(&self, buffer: &GpuBuffer, data: &[u8], offset: u64) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        let staging_buffer = if buffer.mapped_data.read().is_none() {
            Some(self.create_staging_buffer(data.len() as u64).await?)
        } else {
            None
        };

        if let Some(staging) = staging_buffer {
            {
                let staging_ref = staging.buffer.read();
                let staging_buf = staging_ref.as_ref().ok_or("Staging buffer not initialized")?;
                let staging_slice = staging_buf.slice(..);
                
                let mut staging_data = staging.mapped_data.write();
                if let Some(data_slice) = staging_data.as_mut() {
                    data_slice.copy_from_slice(data);
                }
            }
        }

        {
            let buffer_ref = buffer.buffer.read();
            let target_buf = buffer_ref.as_ref().ok_or("Target buffer not initialized")?;
            
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Write Encoder"),
            });

            if let Some(staging) = staging_buffer {
                let staging_ref = staging.buffer.read();
                let staging_buf = staging_ref.as_ref().ok_or("Staging buffer not initialized")?;
                
                encoder.copy_buffer_to_buffer(staging_buf, 0, &target_buf, offset, data.len() as u64);
            } else {
                let mut mapped_data = buffer.mapped_data.write();
                if let Some(data_slice) = mapped_data.as_mut() {
                    data_slice[offset as usize..offset as usize + data.len()].min(data_slice.len() - offset as usize)].copy_from_slice(data);
                }
            }

            let command_buffer = encoder.finish();
            let _ = queue.submit(Some(command_buffer));
        }

        let _ = self.event_sender.send(BufferEvent::BufferWritten(buffer.id.clone(), data.len() as u64));
        Ok(())
    }

    pub async fn read_buffer(&self, buffer: &GpuBuffer, data: &mut [u8], offset: u64) -> Result<u64, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        let queue = self.get_queue().await?;

        if let Some(mapped_data) = buffer.mapped_data.read().as_ref() {
            let data_len = mapped_data.len().min(data.len());
            let copy_len = (offset as usize + data_len).min(mapped_data.len() - offset as usize);
            
            data[0..data_len].copy_from_slice(&mapped_data[offset as usize..copy_len]);
            
            let _ = self.event_sender.send(BufferEvent::BufferRead(buffer.id.clone(), data_len as u64));
            Ok(data_len as u64)
        } else {
            let staging_buffer = self.create_readback_buffer(data.len() as u64).await?;
            
            {
                let buffer_ref = buffer.buffer.read();
                let target_buf = buffer_ref.as_ref().ok_or("Target buffer not initialized")?;
                let staging_ref = staging_buffer.buffer.read();
                let staging_buf = staging_ref.as_ref().ok_or("Staging buffer not initialized")?;
                
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Buffer Read Encoder"),
                });

                encoder.copy_buffer_to_buffer(target_buf, offset, &staging_buf, 0, data.len() as u64);
                
                let command_buffer = encoder.finish();
                let _ = queue.submit(Some(command_buffer));
            }

            device.poll(wgpu::Maintain::Wait);
            
            {
                let staging_ref = staging_buffer.buffer.read();
                let staging_buf = staging_ref.as_ref().ok_or("Staging buffer not initialized")?;
                let staging_slice = staging_buf.slice(..);
                
                let mut staging_data = staging_buffer.mapped_data.write();
                if let Some(data_slice) = staging_data.as_mut() {
                    data[0..data.len()].copy_from_slice(data_slice);
                }
            }

            let data_len = data.len() as u64;
            let _ = self.event_sender.send(BufferEvent::BufferRead(buffer.id.clone(), data_len));
            Ok(data_len)
        }
    }

    pub async fn map_buffer(&self, buffer: &GpuBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let buffer_ref = buffer.buffer.read();
            let buf = buffer_ref.as_ref().ok_or("Buffer not initialized")?;
            
            let mapped_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Mapped {}", buffer.name)),
                size: buffer.size,
                usage: BufferUsages::MAP_READ | BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });

            let queue = self.get_queue().await?;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Map Encoder"),
            });

            encoder.copy_buffer_to_buffer(buf, 0, &mapped_buffer, 0, buffer.size);

            let command_buffer = encoder.finish();
            let _ = queue.submit(Some(command_buffer));
            device.poll(wgpu::Maintain::Wait);

            let mut buffer_ref = buffer.buffer.write();
            *buffer_ref = Some(mapped_buffer);

            let mut mapped_data = buffer.mapped_data.write();
            *mapped_data = Some(vec![0u8; buffer.size as usize]);

            let _ = self.event_sender.send(BufferEvent::BufferMapped(buffer.id.clone()));
        }

        Ok(())
    }

    pub async fn unmap_buffer(&self, buffer: &GpuBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let buffer_ref = buffer.buffer.read();
            let buf = buffer_ref.as_ref().ok_or("Buffer not initialized")?;
            
            let queue = self.get_queue().await?;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Unmap Encoder"),
            });

            encoder.copy_buffer_to_buffer(&buf, 0, &buf, 0, buffer.size);

            let command_buffer = encoder.finish();
            let _ = queue.submit(Some(command_buffer));
            device.poll(wgpu::Maintain::Wait);

            let mut mapped_data = buffer.mapped_data.write();
            *mapped_data = None;

            let mut buffer_ref = buffer.buffer.write();
            *buffer_ref = None;

            let _ = self.event_sender.send(BufferEvent::BufferUnmapped(buffer.id.clone()));
        }

        Ok(())
    }

    pub async fn resize_buffer(&self, buffer: &GpuBuffer, new_size: u64) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let buffer_ref = buffer.buffer.read();
            let buf = buffer_ref.as_ref().ok_or("Buffer not initialized")?;
            
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Resized {}", buffer.name)),
                size: new_size,
                usage: buf.usage(),
                mapped_at_creation: false,
            });

            let queue = self.get_queue().await?;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Resize Encoder"),
            });

            let copy_size = buffer.size.min(new_size);
            encoder.copy_buffer_to_buffer(&buf, 0, &new_buffer, 0, copy_size);

            let command_buffer = encoder.finish();
            let _ = queue.submit(Some(command_buffer));
            device.poll(wgpu::Maintain::Wait);

            let mut buffer_ref = buffer.buffer.write();
            *buffer_ref = Some(new_buffer);

            {
                let mut buffer_clone = buffer.clone();
                buffer_clone.size = new_size;
                let mut buffers = self.buffers.write();
                buffers.insert(buffer.id.clone(), buffer_clone);
            }

            let _ = self.event_sender.send(BufferEvent::BufferWritten(buffer.id.clone(), copy_size));
        }

        Ok(())
    }

    pub async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    pub async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let queue = device.poll(wgpu::Maintain::Wait);
        queue
    }

    pub fn get_memory_usage(&self) -> BufferMemoryUsage {
        let buffers = self.buffers.read();
        let total_size: u64 = buffers.values().map(|b| b.size).sum();
        let allocated_size = total_size;
        
        BufferMemoryUsage {
            allocated_size,
            used_size: allocated_size,
            peak_usage: allocated_size,
            fragmentation: 0.0,
            allocation_count: buffers.len() as u32,
        }
    }

    pub fn get_buffer_info(&self, buffer: &GpuBuffer) -> Option<BufferInfo> {
        Some(BufferInfo {
            id: buffer.id.clone(),
            name: buffer.name.clone(),
            size: buffer.size,
            usage: buffer.usage,
            memory_type: MemoryType::DeviceLocal,
            is_mapped: buffer.mapped_data.read().is_some(),
            is_staging: buffer.staging_buffer.read().is_some(),
            memory_usage: *buffer.memory_usage.read(),
        })
    }

    pub fn cleanup_unused_buffers(&self) -> usize {
        let mut buffers = self.buffers.write();
        let initial_count = buffers.len();
        
        buffers.retain(|_, buffer| {
            true
        });

        let cleaned_count = initial_count - buffers.len();
        let _ = self.event_sender.send(BufferEvent::Error(format!("Cleaned up {} unused buffers", cleaned_count)));
        
        cleaned_count
    }

    pub async fn get_events(&mut self) -> Vec<BufferEvent> {
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

    pub fn clone_buffer(&self, buffer: &GpuBuffer) -> GpuBuffer {
        GpuBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Clone", buffer.name),
            size: buffer.size,
            usage: buffer.usage,
            buffer: Arc::new(RwLock::new(None))),
            mapped_data: Arc::new(RwLock::new(None))),
            staging_buffer: Arc::new(RwLock::new(None))),
            memory_usage: Arc::new(RwLock::new(BufferMemoryUsage::default())),
        }
    }

    pub fn create_buffer_pool(&self, buffer_size: u64, pool_size: usize) -> BufferPool {
        BufferPool::new(
            uuid::Uuid::new_v4().to_string(),
            format!("Buffer Pool {}", buffer_size),
            self.clone(),
            buffer_size,
            pool_size,
        )
    }

    pub fn create_ring_buffer(&self, capacity: u64) -> Result<RingBuffer, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ring Buffer"),
            size: capacity,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(RingBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Ring Buffer".to_string(),
            capacity,
            buffer: Arc::new(RwLock::new(Some(buffer)))),
            write_position: Arc::new(RwLock::new(0))),
            read_position: Arc::new(RwLock::new(0))),
            size: Arc::new(RwLock::new(0))),
            memory_usage: Arc::new(RwLock::new(BufferMemoryUsage::default())),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub usage: BufferUsages,
    pub memory_type: MemoryType,
    pub is_mapped: bool,
    pub is_staging: bool,
    pub memory_usage: BufferMemoryUsage,
}

#[derive(Debug, Clone)]
pub struct BufferPool {
    pub id: String,
    pub name: String,
    pub manager: BufferManager,
    pub buffer_size: u64,
    pub pool_size: usize,
    pub available_buffers: Arc<RwLock<Vec<GpuBuffer>>>,
    pub used_buffers: Arc<RwLock<std::collections::HashSet<String>>>>,
}

#[derive(Debug, Clone)]
pub struct RingBuffer {
    pub id: String,
    pub name: String,
    pub capacity: u64,
    pub buffer: Arc<RwLock<Option<wgpu::Buffer>>>>,
    pub write_position: Arc<RwLock<u64>>,
    pub read_position: Arc<RwLock<u64>>,
    pub size: Arc<RwLock<u64>>,
    pub memory_usage: Arc<RwLock<BufferMemoryUsage>>,
}

impl BufferPool {
    pub fn new(id: String, name: String, manager: BufferManager, buffer_size: u64, pool_size: usize) -> Self {
        Self {
            id,
            name,
            manager,
            buffer_size,
            pool_size,
            available_buffers: Arc::new(RwLock::new(Vec::new())),
            used_buffers: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    pub async fn acquire_buffer(&self) -> Option<GpuBuffer> {
        let mut available = self.available_buffers.write();
        let mut used = self.used_buffers.write();
        
        for buffer in available.iter() {
            if !used.contains(&buffer.id) {
                used.insert(buffer.id.clone());
                return Some(buffer.clone());
            }
        }
        
        None
    }

    pub fn release_buffer(&self, buffer: &GpuBuffer) {
        let mut used = self.used_buffers.write();
        used.remove(&buffer.id);
    }

    pub fn get_available_count(&self) -> usize {
        let available = self.available_buffers.read();
        available.len()
    }

    pub fn get_used_count(&self) -> usize {
        let used = self.used_buffers.read();
        used.len()
    }

    pub fn expand_pool(&self, additional_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut available = self.available_buffers.write();
        
        for _ in 0..additional_size {
            let buffer_size = self.buffer_size;
            let config = BufferConfig {
                size: buffer_size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
                mapped_at_creation: false,
                label: Some(format!("Pool Buffer {}", available.len())),
                memory_type: MemoryType::DeviceLocal,
            };

            if let Ok(new_buffer) = self.manager.create_buffer(config).await {
                available.push(new_buffer);
            }
        }

        Ok(())
    }
}

impl RingBuffer {
    pub async fn write(&self, data: &[u8]) -> Result<u64, Box<dyn std::error::Error>> {
        let device = self.manager.get_device().await?;
        let queue = self.manager.get_queue().await?;
        
        {
            let buffer_ref = self.buffer.read();
            let buf = buffer_ref.as_ref().ok_or("Ring buffer not initialized")?;
            
            let write_pos = {
                let mut pos = self.write_position.write();
                *pos
            };
            
            let available_space = self.capacity - write_pos;
            let data_len = data.len().min(available_space as usize) as u64;
            
            if data_len > 0 {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Ring Buffer Write Encoder"),
                });

                encoder.copy_buffer_to_buffer(&buf, write_pos, &buf, 0, data_len);
                
                let command_buffer = encoder.finish();
                let _ = queue.submit(Some(command_buffer));
                device.poll(wgpu::Maintain::Wait);

                let mut write_pos = self.write_position.write();
                *write_pos = (write_pos + data_len) % self.capacity;
                
                let mut size = self.size.write();
                *size = (*size + data_len).min(self.capacity);
            }

            Ok(data_len)
        }
    }

    pub async fn read(&self, data: &mut [u8], len: u64) -> Result<u64, Box<dyn std::error::Error>> {
        let device = self.manager.get_device().await?;
        let queue = self.manager.get_queue().await?;
        
        {
            let buffer_ref = self.buffer.read();
            let buf = buffer_ref.as_ref().ok_or("Ring buffer not initialized")?;
            
            let read_pos = {
                let mut pos = self.read_position.write();
                *pos
            };
            
            let available_data = self.size.read() - read_pos;
            let read_len = len.min(available_data);
            
            if read_len > 0 {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Ring Buffer Read Encoder"),
                });

                encoder.copy_buffer_to_buffer(&buf, read_pos, &buf, 0, read_len);
                
                let command_buffer = encoder.finish();
                let _ = queue.submit(Some(command_buffer));
                device.poll(wgpu::Maintain::Wait);

                let mut read_pos = self.read_position.write();
                *read_pos = (read_pos + read_len) % self.capacity;
                
                let mut size = self.size.write();
                *size = (*size - read_len).max(0);
            }

            Ok(read_len)
        }
    }

    pub fn get_write_position(&self) -> u64 {
        *self.write_position.read()
    }

    pub fn get_read_position(&self) -> u64 {
        *self.read_position.read()
    }

    pub fn get_available_space(&self) -> u64 {
        self.capacity - *self.size.read()
    }

    pub fn get_used_space(&self) -> u64 {
        *self.size.read()
    }

    pub fn reset(&self) {
        let mut write_pos = self.write_position.write();
        *write_pos = 0;
        
        let mut read_pos = self.read_position.write();
        *read_pos = 0;
        
        let mut size = self.size.write();
        *size = 0;
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Buffer Manager".to_string(),
        )
    }
}

impl Default for GpuBuffer {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Buffer".to_string(),
            size: 1024,
            usage: BufferUsages::STORAGE,
            buffer: Arc::new(RwLock::new(None))),
            mapped_data: Arc::new(RwLock::new(None))),
            staging_buffer: Arc::new(RwLock::new(None))),
            memory_usage: Arc::new(RwLock::new(BufferMemoryUsage::default())),
        }
    }
}

impl Default for BufferMemoryUsage {
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

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            size: 1024,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
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

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Buffer Pool".to_string(),
            BufferManager::default(),
            1024,
            10,
        )
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Ring Buffer".to_string(),
            capacity: 1024,
            buffer: Arc::new(RwLock::new(None))),
            write_position: Arc::new(RwLock::new(0))),
            read_position: Arc::new(RwLock::new(0))),
            size: Arc::new(RwLock::new(0))),
            memory_usage: Arc::new(RwLock::new(BufferMemoryUsage::default())),
        }
    }
}

impl Default for BufferInfo {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Buffer".to_string(),
            size: 1024,
            usage: BufferUsages::STORAGE,
            memory_type: MemoryType::default(),
            is_mapped: false,
            is_staging: false,
            memory_usage: BufferMemoryUsage::default(),
        }
    }
}
