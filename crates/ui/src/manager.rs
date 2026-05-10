use crate::{UiRenderer, Theme, Layout, PanelManager, WidgetManager};
use crate::app::AppState;
use crate::events::EventBus;
use tiffiny_utils::platform::Platform;
use std::sync::Arc;
use parking_lot::RwLock;
use eframe::{egui, NativeOptions};

pub struct UiManager {
    renderer: Arc<UiRenderer>,
    theme: Arc<RwLock<Theme>>,
    layout: Arc<RwLock<Layout>>,
    panel_manager: Arc<PanelManager>,
    widget_manager: Arc<WidgetManager>,
    state: Arc<RwLock<AppState>>,
    event_bus: Arc<EventBus>,
    platform: Arc<Platform>,
    is_initialized: Arc<RwLock<bool>>,
}

impl UiManager {
    pub async fn new(
        state: Arc<RwLock<AppState>>,
        event_bus: Arc<EventBus>,
        platform: Arc<Platform>,
    ) -> Result<Self> {
        let renderer = Arc::new(UiRenderer::new().await?);
        let theme = Arc::new(RwLock::new(Theme::dark()));
        let layout = Arc::new(RwLock::new(Layout::default()));
        let panel_manager = Arc::new(PanelManager::new());
        let widget_manager = Arc::new(WidgetManager::new());

        Ok(Self {
            renderer,
            theme,
            layout,
            panel_manager,
            widget_manager,
            state,
            event_bus,
            platform,
            is_initialized: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        if *self.is_initialized.read() {
            return Ok(());
        }

        self.setup_event_handlers().await?;
        self.load_default_layout().await?;
        self.apply_theme().await?;

        {
            let mut initialized = self.is_initialized.write();
            *initialized = true;
        }

        tracing::info!("UI Manager initialized successfully");
        Ok(())
    }

    pub async fn update(&self) -> Result<()> {
        if !*self.is_initialized.read() {
            return Ok(());
        }

        self.process_ui_events().await?;
        self.update_panels().await?;
        self.update_widgets().await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.save_layout().await?;
        self.cleanup_resources().await?;

        tracing::info!("UI Manager shutdown complete");
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let native_options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1920.0, 1080.0])
                .with_min_inner_size([800.0, 600.0])
                .with_title("Tiffiny Studio"),
            ..Default::default()
        };

        let app = TiffinyEguiApp::new(
            self.renderer.clone(),
            self.theme.clone(),
            self.layout.clone(),
            self.panel_manager.clone(),
            self.widget_manager.clone(),
            self.state.clone(),
            self.event_bus.clone(),
        );

        eframe::run_native(
            "Tiffiny Studio",
            native_options,
            Box::new(|cc| {
                cc.egui_ctx.set_visuals(egui::Visuals::dark());
                Box::new(app)
            }),
        )?;

        Ok(())
    }

    async fn setup_event_handlers(&self) -> Result<()> {
        let event_bus = self.event_bus.clone();
        let theme = self.theme.clone();
        let layout = self.layout.clone();
        let panel_manager = self.panel_manager.clone();

        let mut ui_event_rx = event_bus.subscribe_to_type("ui".to_string()).await;

        tokio::spawn(async move {
            while let Some(event) = ui_event_rx.recv().await {
                if let crate::app::AppEvent::UiEvent(ui_event) = event {
                    match ui_event {
                        crate::app::UiEvent::ThemeChanged(theme_name) => {
                            let new_theme = match theme_name.as_str() {
                                "light" => Theme::light(),
                                "dark" => Theme::dark(),
                                "amoled" => Theme::amoled(),
                                _ => Theme::dark(),
                            };
                            *theme.write() = new_theme;
                        },
                        crate::app::UiEvent::WindowResized(width, height) => {
                            let mut layout_guard = layout.write();
                            layout_guard.set_window_size(width, height);
                        },
                        crate::app::UiEvent::PanelOpened(panel_id) => {
                            panel_manager.open_panel(&panel_id).await;
                        },
                        crate::app::UiEvent::PanelClosed(panel_id) => {
                            panel_manager.close_panel(&panel_id).await;
                        },
                        _ => {}
                    }
                }
            }
        });

        Ok(())
    }

    async fn load_default_layout(&self) -> Result<()> {
        let mut layout = self.layout.write();
        layout.load_default_preset();
        
        let window_size = self.platform.get_window_size().await?;
        layout.set_window_size(window_size.0, window_size.1);

        Ok(())
    }

    async fn apply_theme(&self) -> Result<()> {
        let theme = self.theme.read();
        self.renderer.apply_theme(&*theme).await?;
        Ok(())
    }

    async fn process_ui_events(&self) -> Result<()> {
        let events = self.event_bus.get_events_by_type("ui");
        
        for event in events {
            if let crate::app::AppEvent::UiEvent(ui_event) = event {
                self.handle_ui_event(ui_event).await?;
            }
        }

        Ok(())
    }

    async fn handle_ui_event(&self, ui_event: crate::app::UiEvent) -> Result<()> {
        match ui_event {
            crate::app::UiEvent::KeyPressed(key, ctrl, shift, alt) => {
                self.handle_key_press(key, ctrl, shift, alt).await?;
            },
            crate::app::UiEvent::MousePressed(button, x, y) => {
                self.handle_mouse_press(button, x, y).await?;
            },
            crate::app::UiEvent::MouseMoved(x, y) => {
                self.handle_mouse_move(x, y).await?;
            },
            crate::app::UiEvent::MouseScrolled(dx, dy) => {
                self.handle_mouse_scroll(dx, dy).await?;
            },
            _ => {}
        }

        Ok(())
    }

    async fn handle_key_press(&self, key: String, ctrl: bool, shift: bool, alt: bool) -> Result<()> {
        let shortcut = format!("{}{}{}{}",
            if ctrl { "Ctrl+" } else { "" },
            if shift { "Shift+" } else { "" },
            if alt { "Alt+" } else { "" },
            key
        );

        let event = crate::app::AppEvent::UiEvent(crate::app::UiEvent::KeyPressed(key, ctrl, shift, alt));
        self.event_bus.publish(event).await?;

        Ok(())
    }

    async fn handle_mouse_press(&self, button: u32, x: f32, y: f32) -> Result<()> {
        let event = crate::app::AppEvent::UiEvent(crate::app::UiEvent::MousePressed(button, x, y));
        self.event_bus.publish(event).await?;
        Ok(())
    }

    async fn handle_mouse_move(&self, x: f32, y: f32) -> Result<()> {
        let event = crate::app::AppEvent::UiEvent(crate::app::UiEvent::MouseMoved(x, y));
        self.event_bus.publish(event).await?;
        Ok(())
    }

    async fn handle_mouse_scroll(&self, dx: f32, dy: f32) -> Result<()> {
        let event = crate::app::AppEvent::UiEvent(crate::app::UiEvent::MouseScrolled(dx, dy));
        self.event_bus.publish(event).await?;
        Ok(())
    }

    async fn update_panels(&self) -> Result<()> {
        self.panel_manager.update().await?;
        Ok(())
    }

    async fn update_widgets(&self) -> Result<()> {
        self.widget_manager.update().await?;
        Ok(())
    }

    async fn save_layout(&self) -> Result<()> {
        let layout = self.layout.read();
        let layout_data = serde_json::to_value(&*layout)?;
        
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tiffiny");
        
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(
            config_dir.join("layout.json"),
            serde_json::to_string_pretty(&layout_data)?
        )?;

        Ok(())
    }

    async fn cleanup_resources(&self) -> Result<()> {
        self.renderer.cleanup().await?;
        self.panel_manager.cleanup().await?;
        self.widget_manager.cleanup().await?;
        Ok(())
    }
}

struct TiffinyEguiApp {
    renderer: Arc<UiRenderer>,
    theme: Arc<RwLock<Theme>>,
    layout: Arc<RwLock<Layout>>,
    panel_manager: Arc<PanelManager>,
    widget_manager: Arc<WidgetManager>,
    state: Arc<RwLock<AppState>>,
    event_bus: Arc<EventBus>,
}

impl TiffinyEguiApp {
    fn new(
        renderer: Arc<UiRenderer>,
        theme: Arc<RwLock<Theme>>,
        layout: Arc<RwLock<Layout>>,
        panel_manager: Arc<PanelManager>,
        widget_manager: Arc<WidgetManager>,
        state: Arc<RwLock<AppState>>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            renderer,
            theme,
            layout,
            panel_manager,
            widget_manager,
            state,
            event_bus,
        }
    }
}

impl eframe::App for TiffinyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme = self.theme.read();
        ctx.set_visuals(theme.to_egui_visuals());

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.render_menu_bar(ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_area(ui);
        });

        self.render_dockable_panels(ctx);
    }
}

impl TiffinyEguiApp {
    fn render_menu_bar(&self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if ui.button("New Project").clicked() {
                    let event = crate::app::AppEvent::NewProject;
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
                
                if ui.button("Open Project").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Tiffiny Project", &["tiffiny"])
                        .pick_file() {
                        
                        let event = crate::app::AppEvent::OpenProject(path.to_string_lossy().to_string());
                        let _ = self.event_bus.publish(event);
                        ui.close_menu();
                    }
                }
                
                if ui.button("Save Project").clicked() {
                    let event = crate::app::AppEvent::SaveProject;
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Import File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .pick_file() {
                        
                        let event = crate::app::AppEvent::ImportFile(path.to_string_lossy().to_string());
                        let _ = self.event_bus.publish(event);
                        ui.close_menu();
                    }
                }
                
                ui.separator();
                
                if ui.button("Quit").clicked() {
                    let event = crate::app::AppEvent::Quit;
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
            });

            egui::menu::menu_button(ui, "Edit", |ui| {
                if ui.button("Undo").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Redo").clicked() {
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Cut").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Copy").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Paste").clicked() {
                    ui.close_menu();
                }
            });

            egui::menu::menu_button(ui, "View", |ui| {
                if ui.button("Timeline").clicked() {
                    let event = crate::app::AppEvent::UiEvent(
                        crate::app::UiEvent::PanelOpened("timeline".to_string())
                    );
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
                
                if ui.button("Waveform").clicked() {
                    let event = crate::app::AppEvent::UiEvent(
                        crate::app::UiEvent::PanelOpened("waveform".to_string())
                    );
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
                
                if ui.button("Spectrogram").clicked() {
                    let event = crate::app::AppEvent::UiEvent(
                        crate::app::UiEvent::PanelOpened("spectrogram".to_string())
                    );
                    let _ = self.event_bus.publish(event);
                    ui.close_menu();
                }
            });

            egui::menu::menu_button(ui, "Effects", |ui| {
                if ui.button("DataBend").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("Glitch").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("VHS").clicked() {
                    ui.close_menu();
                }
                
                if ui.button("CRT").clicked() {
                    ui.close_menu();
                }
            });

            egui::menu::menu_button(ui, "Help", |ui| {
                if ui.button("About").clicked() {
                    ui.close_menu();
                }
            });
        });
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Ready");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("FPS: {:.1}", ui.ctx().frame_rate()));
                
                let state = self.state.read();
                if let Some(project_id) = state.get_current_project_id() {
                    ui.label(format!("Project: {}", project_id));
                }
            });
        });
    }

    fn render_main_area(&self, ui: &mut egui::Ui) {
        let layout = self.layout.read();
        let main_area = layout.get_main_area_rect();
        
        ui.allocate_ui_at_rect(main_area, |ui| {
            self.render_viewport(ui);
        });
    }

    fn render_viewport(&self, ui: &mut egui::Ui) {
        ui.heading("Viewport");
        ui.separator();
        
        ui.label("Main rendering area for media preview and processing visualization");
        
        ui.add_space(10.0);
        
        if ui.button("Load Media").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Media Files", &["mp4", "wav", "png", "jpg", "tiff"])
                .pick_file() {
                
                let event = crate::app::AppEvent::ImportFile(path.to_string_lossy().to_string());
                let _ = self.event_bus.publish(event);
            }
        }
    }

    fn render_dockable_panels(&self, ctx: &egui::Context) {
        let panels = self.panel_manager.get_visible_panels();
        
        for panel in panels {
            match panel.panel_type.as_str() {
                "timeline" => self.render_timeline_panel(ctx),
                "waveform" => self.render_waveform_panel(ctx),
                "spectrogram" => self.render_spectrogram_panel(ctx),
                "project_explorer" => self.render_project_explorer_panel(ctx),
                "inspector" => self.render_inspector_panel(ctx),
                "console" => self.render_console_panel(ctx),
                _ => {}
            }
        }
    }

    fn render_timeline_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Timeline")
            .default_size([800.0, 200.0])
            .show(ctx, |ui| {
                ui.heading("Timeline");
                ui.separator();
                ui.label("Timeline view for audio/video editing");
            });
    }

    fn render_waveform_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Waveform")
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading("Waveform");
                ui.separator();
                ui.label("Audio waveform visualization");
            });
    }

    fn render_spectrogram_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Spectrogram")
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading("Spectrogram");
                ui.separator();
                ui.label("Audio spectrogram visualization");
            });
    }

    fn render_project_explorer_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Project Explorer")
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("Project Explorer");
                ui.separator();
                
                let state = self.state.read();
                if let Some(project_id) = state.get_current_project_id() {
                    ui.label(format!("Current Project: {}", project_id));
                } else {
                    ui.label("No project loaded");
                }
            });
    }

    fn render_inspector_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Inspector")
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                ui.label("Properties and settings panel");
            });
    }

    fn render_console_panel(&self, ctx: &egui::Context) {
        egui::Window::new("Console")
            .default_size([600.0, 200.0])
            .show(ctx, |ui| {
                ui.heading("Console");
                ui.separator();
                ui.label("Output and log messages");
            });
    }
}
