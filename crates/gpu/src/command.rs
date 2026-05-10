use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CommandManager {
    pub id: String,
    pub name: String,
    pub device: Arc<RwLock<Option<wgpu::Device>>>>,
    pub queue: Arc<RwLock<Option<wgpu::Queue>>>>,
    pub command_buffers: Arc<RwLock<std::collections::HashMap<String, CommandBuffer>>>>,
    pub event_sender: mpsc::UnboundedSender<CommandEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<CommandEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum CommandEvent {
    CommandBufferCreated(String),
    CommandBufferDestroyed(String),
    CommandBufferSubmitted(String),
    CommandBufferCompleted(String),
    CommandBufferFailed(String, String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct CommandBuffer {
    pub id: String,
    pub name: String,
    pub buffer: Arc<RwLock<Option<wgpu::CommandBuffer>>>>,
    pub encoder: Arc<RwLock<Option<wgpu::CommandEncoder>>>>,
    pub state: Arc<RwLock<CommandBufferState>>,
    pub commands: Arc<RwLock<Vec<Command>>>>,
    pub timestamp: Arc<RwLock<std::time::Instant>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandBufferState {
    Initial,
    Recording,
    Finished,
    Submitted,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub enum Command {
    CopyBuffer {
        source: String,
        destination: String,
        size: u64,
        source_offset: u64,
        destination_offset: u64,
    },
    CopyTexture {
        source: String,
        destination: String,
        size: (u32, u32),
        source_offset: (u32, u32),
        destination_offset: (u32, u32),
    },
    FillBuffer {
        buffer: String,
        size: u64,
        offset: u64,
        data: Vec<u8>,
    },
    FillTexture {
        texture: String,
        size: (u32, u32),
        offset: (u32, u32),
        data: Vec<u8>,
    },
    ComputePass {
        pipeline: String,
        workgroup_count: (u32, u32, u32),
        bind_groups: Vec<String>,
    },
    RenderPass {
        pipeline: String,
        color_attachments: Vec<String>,
        depth_stencil: Option<String>,
        vertex_buffers: Vec<String>,
        index_buffer: Option<String>,
        draw_count: u32,
        instance_count: u32,
    },
    Barrier {
        barriers: Vec<MemoryBarrier>,
    },
    Custom {
        name: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct MemoryBarrier {
    pub buffer: Option<String>,
    pub texture: Option<String>,
    pub usage_before: wgpu::BufferUsages,
    pub usage_after: wgpu::BufferUsages,
}

#[derive(Debug, Clone)]
pub struct CommandEncoder {
    pub id: String,
    pub name: String,
    pub encoder: Arc<RwLock<Option<wgpu::CommandEncoder>>>>,
    pub command_buffers: Arc<RwLock<Vec<CommandBuffer>>>>,
    pub current_buffer: Arc<RwLock<Option<CommandBuffer>>>>,
}

#[derive(Debug, Clone)]
pub struct CommandQueue {
    pub id: String,
    pub name: String,
    pub queue: Arc<RwLock<Option<wgpu::Queue>>>>,
    pub pending_buffers: Arc<RwLock<Vec<CommandBuffer>>>>,
    pub submitted_buffers: Arc<RwLock<Vec<CommandBuffer>>>>,
    pub completed_buffers: Arc<RwLock<Vec<CommandBuffer>>>>,
}

impl CommandManager {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            device: Arc::new(RwLock::new(None))),
            queue: Arc::new(RwLock::new(None))),
            command_buffers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
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

    pub async fn create_command_buffer(&self, name: String) -> Result<CommandBuffer, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&name),
        });

        let command_buffer = CommandBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            buffer: Arc::new(RwLock::new(None))),
            encoder: Arc::new(RwLock::new(Some(encoder))),
            state: Arc::new(RwLock::new(CommandBufferState::Initial))),
            commands: Arc::new(RwLock::new(Vec::new())),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
        };

Add to manager
        {
            let mut buffers = self.command_buffers.write();
            buffers.insert(command_buffer.id.clone(), command_buffer.clone());
        }

        let _ = self.event_sender.send(CommandEvent::CommandBufferCreated(command_buffer.id.clone()));
        Ok(command_buffer)
    }

    pub async fn create_command_encoder(&self, name: String) -> Result<CommandEncoder, Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&name),
        });

        let command_encoder = CommandEncoder {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            encoder: Arc::new(RwLock::new(Some(encoder))),
            command_buffers: Arc::new(RwLock::new(Vec::new())),
            current_buffer: Arc::new(RwLock::new(None))),
        };

        Ok(command_encoder)
    }

    pub async fn create_command_queue(&self, name: String) -> Result<CommandQueue, Box<dyn std::error::Error>> {
        let queue = self.get_queue().await?;
        
        let command_queue = CommandQueue {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            queue: Arc::new(RwLock::new(Some(queue))),
            pending_buffers: Arc::new(RwLock::new(Vec::new())),
            submitted_buffers: Arc::new(RwLock::new(Vec::new())),
            completed_buffers: Arc::new(RwLock::new(Vec::new())),
        };

        Ok(command_queue)
    }

    pub fn destroy_command_buffer(&self, buffer_id: &str) -> bool {
        let mut buffers = self.command_buffers.write();
        
        if buffers.remove(buffer_id).is_some() {
            let _ = self.event_sender.send(CommandEvent::CommandBufferDestroyed(buffer_id.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get_command_buffer(&self, buffer_id: &str) -> Option<CommandBuffer> {
        let buffers = self.command_buffers.read();
        buffers.get(buffer_id).cloned()
    }

    pub fn list_command_buffers(&self) -> Vec<CommandBuffer> {
        let buffers = self.command_buffers.read();
        buffers.values().cloned().collect()
    }

    pub fn get_command_buffer_count(&self) -> usize {
        let buffers = self.command_buffers.read();
        buffers.len()
    }

    pub async fn begin_recording(&self, command_buffer: &CommandBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let mut state = command_buffer.state.write();
            *state = CommandBufferState::Recording;
        }

        {
            let mut encoder_ref = command_buffer.encoder.write();
            *encoder_ref = Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&command_buffer.name),
            }));
        }

        {
            let mut timestamp = command_buffer.timestamp.write();
            *timestamp = std::time::Instant::now();
        }

        Ok(())
    }

    pub async fn end_recording(&self, command_buffer: &CommandBuffer) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut state = command_buffer.state.write();
            *state = CommandBufferState::Finished;
        }

        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let buffer = encoder.finish();
                
                {
                    let mut buffer_ref = command_buffer.buffer.write();
                    *buffer_ref = Some(buffer);
                }
            }
        }

        Ok(())
    }

    pub async fn submit_command_buffer(&self, command_buffer: &CommandBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let queue = self.get_queue().await?;
        
        {
            let mut state = command_buffer.state.write();
            *state = CommandBufferState::Submitted;
        }

        {
            let buffer_ref = command_buffer.buffer.read();
            if let Some(buffer) = buffer_ref.as_ref() {
                let _ = queue.submit(Some(buffer));
            }
        }

        let _ = self.event_sender.send(CommandEvent::CommandBufferSubmitted(command_buffer.id.clone()));
        Ok(())
    }

    pub async fn submit_multiple_buffers(&self, buffers: &[CommandBuffer]) -> Result<(), Box<dyn std::error::Error>> {
        let queue = self.get_queue().await?;
        
        let command_buffers: Vec<wgpu::CommandBuffer> = buffers
            .iter()
            .filter_map(|buffer| {
                {
                    let mut state = buffer.state.write();
                    *state = CommandBufferState::Submitted;
                }

                buffer.buffer.read().clone()
            })
            .collect();

        if !command_buffers.is_empty() {
            let _ = queue.submit(Some(&command_buffers));
        }

        for buffer in buffers {
            let _ = self.event_sender.send(CommandEvent::CommandBufferSubmitted(buffer.id.clone()));
        }

        Ok(())
    }

    pub async fn execute_command(&self, command: Command) -> Result<(), Box<dyn std::error::Error>> {
        let command_buffer = self.create_command_buffer("Command Execution".to_string()).await?;
        
        self.begin_recording(&command_buffer).await?;
        
        match command {
            Command::CopyBuffer { source, destination, size, source_offset, destination_offset } => {
                self.execute_copy_buffer(&command_buffer, &source, &destination, size, source_offset, destination_offset).await?;
            },
            Command::CopyTexture { source, destination, size, source_offset, destination_offset } => {
                self.execute_copy_texture(&command_buffer, &source, &destination, size, source_offset, destination_offset).await?;
            },
            Command::FillBuffer { buffer: _, size, offset, data } => {
                self.execute_fill_buffer(&command_buffer, size, offset, &data).await?;
            },
            Command::FillTexture { texture: _, size, offset, data } => {
                self.execute_fill_texture(&command_buffer, size, offset, &data).await?;
            },
            Command::ComputePass { pipeline, workgroup_count, bind_groups } => {
                self.execute_compute_pass(&command_buffer, &pipeline, workgroup_count, &bind_groups).await?;
            },
            Command::RenderPass { pipeline, color_attachments, depth_stencil, vertex_buffers, index_buffer, draw_count, instance_count } => {
                self.execute_render_pass(&command_buffer, &pipeline, &color_attachments, depth_stencil.as_deref(), &vertex_buffers, index_buffer.as_deref(), draw_count, instance_count).await?;
            },
            Command::Barrier { barriers } => {
                self.execute_barrier(&command_buffer, &barriers).await?;
            },
            Command::Custom { name: _, data: _ } => {
            },
        }

        self.end_recording(&command_buffer).await?;
        self.submit_command_buffer(&command_buffer).await?;

        Ok(())
    }

    async fn execute_copy_buffer(&self, command_buffer: &CommandBuffer, source: &str, destination: &str, size: u64, source_offset: u64, destination_offset: u64) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let _ = encoder.copy_buffer_to_buffer(
                    &create_dummy_buffer(),
                    source_offset,
                    &create_dummy_buffer(),
                    destination_offset,
                    size,
                );
            }
        }

        Ok(())
    }

    async fn execute_copy_texture(&self, command_buffer: &CommandBuffer, source: &str, destination: &str, size: (u32, u32), source_offset: (u32, u32), destination_offset: (u32, u32)) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let _ = encoder.copy_texture_to_texture(
                    &create_dummy_texture(),
                    wgpu::ImageCopyTexture {
                        src_base: wgpu::Origin3d { x: source_offset.0, y: source_offset.1, z: 0 },
                        mip_level: 0,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &create_dummy_texture(),
                    wgpu::ImageCopyTexture {
                        src_base: wgpu::Origin3d { x: destination_offset.0, y: destination_offset.1, z: 0 },
                        mip_level: 0,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        Ok(())
    }

    async fn execute_fill_buffer(&self, command_buffer: &CommandBuffer, size: u64, offset: u64, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fill Data Buffer"),
                    size: data.len() as u64,
                    usage: wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                });

                let _ = encoder.copy_buffer_to_buffer(&temp_buffer, 0, &create_dummy_buffer(), offset, data.len() as u64);
            }
        }

        Ok(())
    }

    async fn execute_fill_texture(&self, command_buffer: &CommandBuffer, size: (u32, u32), offset: (u32, u32), data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fill Texture Data Buffer"),
                    size: data.len() as u64,
                    usage: wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                });

                let _ = encoder.copy_buffer_to_texture(
                    &temp_buffer,
                    0,
                    &create_dummy_texture(),
                    wgpu::ImageCopyBuffer {
                        buffer_layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(size.0 * 4),
                            rows_per_image: Some(size.1),
                        },
                        offset: wgpu::Origin3d { x: offset.0, y: offset.1, z: 0 },
                        copy_size: wgpu::Extent3d {
                            width: size.0,
                            height: size.1,
                            depth_or_array_layers: 1,
                        },
                    },
                );
            }
        }

        Ok(())
    }

    async fn execute_compute_pass(&self, command_buffer: &CommandBuffer, pipeline: &str, workgroup_count: (u32, u32, u32), bind_groups: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                });

                compute_pass.set_pipeline(&create_dummy_compute_pipeline());
                
                for (i, bind_group) in bind_groups.iter().enumerate() {
                    let _ = (i, bind_group);
                }

                compute_pass.dispatch_workgroups(workgroup_count.0, workgroup_count.1, workgroup_count.2);
            }
        }

        Ok(())
    }

    async fn execute_render_pass(&self, command_buffer: &CommandBuffer, pipeline: &str, color_attachments: &[String], depth_stencil: Option<&str>, vertex_buffers: &[String], index_buffer: Option<&str>, draw_count: u32, instance_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let color_attachment_descriptors: Vec<wgpu::RenderPassColorAttachment> = color_attachments
                    .iter()
                    .map(|_| wgpu::RenderPassColorAttachment {
                        view: &create_dummy_texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })
                    .collect();

                let depth_stencil_attachment = depth_stencil.map(|_| wgpu::RenderPassDepthStencilAttachment {
                    view: &create_dummy_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &color_attachment_descriptors,
                    depth_stencil_attachment,
                });

                render_pass.set_pipeline(&create_dummy_render_pipeline());

                for (i, vertex_buffer) in vertex_buffers.iter().enumerate() {
                    let _ = (i, vertex_buffer);
                }

                if let Some(_) = index_buffer {
                }

                render_pass.draw(0..draw_count, 0..instance_count);
            }
        }

        Ok(())
    }

    async fn execute_barrier(&self, command_buffer: &CommandBuffer, barriers: &[MemoryBarrier]) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.get_device().await?;
        
        {
            let encoder_ref = command_buffer.encoder.read();
            if let Some(encoder) = encoder_ref.as_ref() {
                let wgpu_barriers: Vec<wgpu::MemoryBarrier> = barriers
                    .iter()
                    .map(|barrier| wgpu::MemoryBarrier {
                        uses: wgpu::ResourceUses::empty(),
                    })
                    .collect();

                encoder.insert_memory_barrier(&wgpu_barriers);
            }
        }

        Ok(())
    }

    async fn get_device(&self) -> Result<wgpu::Device, Box<dyn std::error::Error>> {
        let device_ref = self.device.read();
        device_ref.as_ref().ok_or("Device not initialized")?.clone()
    }

    async fn get_queue(&self) -> Result<wgpu::Queue, Box<dyn std::error::Error>> {
        let queue_ref = self.queue.read();
        queue_ref.as_ref().ok_or("Queue not initialized")?.clone()
    }

    pub async fn get_events(&mut self) -> Vec<CommandEvent> {
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

    pub fn get_command_buffer_info(&self, buffer: &CommandBuffer) -> CommandBufferInfo {
        let state = buffer.state.read().clone();
        let command_count = buffer.commands.read().len();
        let timestamp = *buffer.timestamp.read();

        CommandBufferInfo {
            id: buffer.id.clone(),
            name: buffer.name.clone(),
            state,
            command_count,
            timestamp,
            is_recording: state == CommandBufferState::Recording,
            is_finished: state == CommandBufferState::Finished,
            is_submitted: state == CommandBufferState::Submitted,
            is_completed: state == CommandBufferState::Completed,
            is_failed: state == CommandBufferState::Failed,
        }
    }

    pub fn get_command_stats(&self) -> CommandStats {
        let buffers = self.command_buffers.read();
        let total_buffers = buffers.len();
        let recording_count = buffers.values().filter(|b| *b.state.read() == CommandBufferState::Recording).count();
        let finished_count = buffers.values().filter(|b| *b.state.read() == CommandBufferState::Finished).count();
        let submitted_count = buffers.values().filter(|b| *b.state.read() == CommandBufferState::Submitted).count();
        let completed_count = buffers.values().filter(|b| *b.state.read() == CommandBufferState::Completed).count();
        let failed_count = buffers.values().filter(|b| *b.state.read() == CommandBufferState::Failed).count();

        CommandStats {
            total_buffers,
            recording_count,
            finished_count,
            submitted_count,
            completed_count,
            failed_count,
        }
    }

    pub fn cleanup_completed_buffers(&self) -> usize {
        let mut buffers = self.command_buffers.write();
        let initial_count = buffers.len();
        
        buffers.retain(|_, buffer| {
            let state = *buffer.state.read();
            state != CommandBufferState::Completed && state != CommandBufferState::Failed
        });

        let cleaned_count = initial_count - buffers.len();
        let _ = self.event_sender.send(CommandEvent::Error(format!("Cleaned up {} completed buffers", cleaned_count)));
        
        cleaned_count
    }

    pub fn reset(&self) {
        let mut buffers = self.command_buffers.write();
        buffers.clear();
    }
}

fn create_dummy_buffer() -> wgpu::Buffer {
    unimplemented!("Dummy buffer creation not implemented")
}

fn create_dummy_texture() -> wgpu::Texture {
    unimplemented!("Dummy texture creation not implemented")
}

fn create_dummy_texture_view() -> wgpu::TextureView {
    unimplemented!("Dummy texture view creation not implemented")
}

fn create_dummy_compute_pipeline() -> wgpu::ComputePipeline {
    unimplemented!("Dummy compute pipeline creation not implemented")
}

fn create_dummy_render_pipeline() -> wgpu::RenderPipeline {
    unimplemented!("Dummy render pipeline creation not implemented")
}

#[derive(Debug, Clone)]
pub struct CommandBufferInfo {
    pub id: String,
    pub name: String,
    pub state: CommandBufferState,
    pub command_count: usize,
    pub timestamp: std::time::Instant,
    pub is_recording: bool,
    pub is_finished: bool,
    pub is_submitted: bool,
    pub is_completed: bool,
    pub is_failed: bool,
}

#[derive(Debug, Clone)]
pub struct CommandStats {
    pub total_buffers: usize,
    pub recording_count: usize,
    pub finished_count: usize,
    pub submitted_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Command Manager".to_string(),
        )
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Command Buffer".to_string(),
            buffer: Arc::new(RwLock::new(None))),
            encoder: Arc::new(RwLock::new(None))),
            state: Arc::new(RwLock::new(CommandBufferState::Initial))),
            commands: Arc::new(RwLock::new(Vec::new())),
            timestamp: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }
}

impl Default for CommandBufferState {
    fn default() -> Self {
        CommandBufferState::Initial
    }
}

impl Default for CommandEncoder {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Command Encoder".to_string(),
            encoder: Arc::new(RwLock::new(None))),
            command_buffers: Arc::new(RwLock::new(Vec::new())),
            current_buffer: Arc::new(RwLock::new(None))),
        }
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Command Queue".to_string(),
            queue: Arc::new(RwLock::new(None))),
            pending_buffers: Arc::new(RwLock::new(Vec::new())),
            submitted_buffers: Arc::new(RwLock::new(Vec::new())),
            completed_buffers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for CommandBufferInfo {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Command Buffer".to_string(),
            state: CommandBufferState::default(),
            command_count: 0,
            timestamp: std::time::Instant::now(),
            is_recording: false,
            is_finished: false,
            is_submitted: false,
            is_completed: false,
            is_failed: false,
        }
    }
}

impl Default for CommandStats {
    fn default() -> Self {
        Self {
            total_buffers: 0,
            recording_count: 0,
            finished_count: 0,
            submitted_count: 0,
            completed_count: 0,
            failed_count: 0,
        }
    }
}
