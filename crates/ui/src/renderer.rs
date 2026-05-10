use crate::theme::Theme;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct UiRenderer {
    device: Arc<RwLock<Option<wgpu::Device>>>,
    queue: Arc<RwLock<Option<wgpu::Queue>>>,
    surface: Arc<RwLock<Option<wgpu::Surface<'static>>>>,
    config: Arc<RwLock<Option<wgpu::SurfaceConfiguration>>>,
    pipeline: Arc<RwLock<Option<wgpu::RenderPipeline>>>,
    theme: Arc<RwLock<Theme>>,
    is_initialized: Arc<RwLock<bool>>,
}

impl UiRenderer {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            device: Arc::new(RwLock::new(None)),
            queue: Arc::new(RwLock::new(None)),
            surface: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(None)),
            pipeline: Arc::new(RwLock::new(None)),
            theme: Arc::new(RwLock::new(Theme::dark())),
            is_initialized: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn initialize(&self, window: &winit::window::Window) -> Result<()> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = unsafe { instance.create_surface(window) }?;
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| tiffiny_core::CoreError::Memory("Failed to get GPU adapter".to_string()))?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("UI Renderer Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ).await?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        {
            let mut device_guard = self.device.write();
            *device_guard = Some(device);
        }

        {
            let mut queue_guard = self.queue.write();
            *queue_guard = Some(queue);
        }

        {
            let mut surface_guard = self.surface.write();
            *surface_guard = Some(surface);
        }

        {
            let mut config_guard = self.config.write();
            *config_guard = Some(config);
        }

        self.create_render_pipeline().await?;

        {
            let mut initialized = self.is_initialized.write();
            *initialized = true;
        }

        tracing::info!("UI Renderer initialized successfully");
        Ok(())
    }

    pub async fn apply_theme(&self, theme: &Theme) -> Result<()> {
        {
            let mut theme_guard = self.theme.write();
            *theme_guard = theme.clone();
        }

        self.recreate_render_pipeline().await?;
        Ok(())
    }

    pub async fn resize(&self, new_width: u32, new_height: u32) -> Result<()> {
        let surface = {
            let surface_guard = self.surface.read();
            surface_guard.clone()
        };

        let device = {
            let device_guard = self.device.read();
            device_guard.clone()
        };

        let config = {
            let config_guard = self.config.read();
            config_guard.clone()
        };

        if let (Some(surface), Some(device), Some(mut config)) = (surface, device, config) {
            config.width = new_width;
            config.height = new_height;
            surface.configure(&device, &config);

            let mut config_guard = self.config.write();
            *config_guard = Some(config);
        }

        Ok(())
    }

    pub async fn render(&self) -> Result<()> {
        if !*self.is_initialized.read() {
            return Ok(());
        }

        let surface = {
            let surface_guard = self.surface.read();
            surface_guard.clone()
        };

        let device = {
            let device_guard = self.device.read();
            device_guard.clone()
        };

        let queue = {
            let queue_guard = self.queue.read();
            queue_guard.clone()
        };

        let pipeline = {
            let pipeline_guard = self.pipeline.read();
            pipeline_guard.clone()
        };

        if let (Some(surface), Some(device), Some(queue), Some(pipeline)) = (surface, device, queue, pipeline) {
            let output = surface.get_current_texture()?;
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Renderer Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("UI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(&pipeline);
                render_pass.draw(0..3, 0..1);
            }

            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }

    async fn create_render_pipeline(&self) -> Result<()> {
        let device = {
            let device_guard = self.device.read();
            device_guard.clone()
        };

        let config = {
            let config_guard = self.config.read();
            config_guard.clone()
        };

        let theme = {
            let theme_guard = self.theme.read();
            theme_guard.clone()
        };

        if let (Some(device), Some(config)) = (device, config) {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("UI Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("UI Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("UI Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            {
                let mut pipeline_guard = self.pipeline.write();
                *pipeline_guard = Some(pipeline);
            }
        }

        Ok(())
    }

    async fn recreate_render_pipeline(&self) -> Result<()> {
        self.create_render_pipeline().await
    }

    pub async fn cleanup(&self) -> Result<()> {
        {
            let mut device = self.device.write();
            *device = None;
        }

        {
            let mut queue = self.queue.write();
            *queue = None;
        }

        {
            let mut surface = self.surface.write();
            *surface = None;
        }

        {
            let mut config = self.config.write();
            *config = None;
        }

        {
            let mut pipeline = self.pipeline.write();
            *pipeline = None;
        }

        {
            let mut initialized = self.is_initialized.write();
            *initialized = false;
        }

        tracing::info!("UI Renderer cleanup complete");
        Ok(())
    }

    pub fn get_device(&self) -> Option<wgpu::Device> {
        let device_guard = self.device.read();
        device_guard.clone()
    }

    pub fn get_queue(&self) -> Option<wgpu::Queue> {
        let queue_guard = self.queue.read();
        queue_guard.clone()
    }

    pub fn get_theme(&self) -> Theme {
        let theme_guard = self.theme.read();
        theme_guard.clone()
    }

    pub fn is_initialized(&self) -> bool {
        *self.is_initialized.read()
    }
}
