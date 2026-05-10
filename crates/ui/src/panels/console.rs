use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ConsolePanel {
    pub messages: Vec<ConsoleMessage>,
    pub max_messages: usize,
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub show_levels: bool,
    pub level_filter: LogLevel,
    pub search_query: String,
    pub wrap_text: bool,
    pub font_size: f32,
}

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub id: String,
    pub timestamp: std::time::SystemTime,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 1000,
            auto_scroll: true,
            show_timestamps: true,
            show_levels: true,
            level_filter: LogLevel::Debug,
            search_query: String::new(),
            wrap_text: true,
            font_size: 12.0,
        }
    }

    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn log_message(&mut self, level: LogLevel, source: String, message: String, details: Option<String>) {
        let console_message = ConsoleMessage {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now(),
            level,
            source,
            message,
            details,
        };

        self.messages.push(console_message);

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn debug(&mut self, source: String, message: String) {
        self.log_message(LogLevel::Debug, source, message, None);
    }

    pub fn info(&mut self, source: String, message: String) {
        self.log_message(LogLevel::Info, source, message, None);
    }

    pub fn warning(&mut self, source: String, message: String) {
        self.log_message(LogLevel::Warning, source, message, None);
    }

    pub fn error(&mut self, source: String, message: String, details: Option<String>) {
        self.log_message(LogLevel::Error, source, message, details);
    }

    pub fn critical(&mut self, source: String, message: String, details: Option<String>) {
        self.log_message(LogLevel::Critical, source, message, details);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
        if self.messages.len() > max {
            self.messages.truncate(max);
        }
    }

    pub fn set_level_filter(&mut self, level: LogLevel) {
        self.level_filter = level;
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.clamp(8.0, 24.0);
    }

    pub fn export_messages(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        for message in &self.messages {
            let timestamp = message.timestamp.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            content.push_str(&format!(
                "[{}] [{}] [{}] {}\n",
                timestamp,
                format!("{:?}", message.level),
                message.source,
                message.message
            ));
            
            if let Some(details) = &message.details {
                content.push_str(&format!("  Details: {}\n", details));
            }
        }

        std::fs::write(file_path, content)?;
        Ok(())
    }

    pub fn get_filtered_messages(&self) -> Vec<&ConsoleMessage> {
        self.messages
            .iter()
            .filter(|msg| msg.level >= self.level_filter)
            .filter(|msg| {
                self.search_query.is_empty() || 
                msg.message.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                msg.source.to_lowercase().contains(&self.search_query.to_lowercase())
            })
            .collect()
    }

    pub fn get_message_count_by_level(&self) -> std::collections::HashMap<LogLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for message in &self.messages {
            *counts.entry(message.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear();
            }

            if ui.button("Export").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Log Files", &["log", "txt"])
                    .save_file() {
                    
                    if let Err(e) = self.export_messages(&path.to_string_lossy()) {
                        self.error("Console".to_string(), format!("Failed to export log: {}", e), None);
                    }
                }
            }

            ui.add_space(10.0);

            ui.checkbox(&mut self.auto_scroll, "Auto Scroll");
            ui.checkbox(&mut self.show_timestamps, "Timestamps");
            ui.checkbox(&mut self.show_levels, "Levels");
            ui.checkbox(&mut self.wrap_text, "Wrap");

            ui.add_space(10.0);

            egui::ComboBox::from_label("Level")
                .selected_text(format!("{:?}", self.level_filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.level_filter, LogLevel::Debug, "Debug");
                    ui.selectable_value(&mut self.level_filter, LogLevel::Info, "Info");
                    ui.selectable_value(&mut self.level_filter, LogLevel::Warning, "Warning");
                    ui.selectable_value(&mut self.level_filter, LogLevel::Error, "Error");
                    ui.selectable_value(&mut self.level_filter, LogLevel::Critical, "Critical");
                });

            ui.add_space(10.0);

            ui.add(egui::Slider::new(&mut self.font_size, 8.0..=24.0).text("Font"));

            ui.add_space(10.0);

            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("Search..."));
        });

        ui.separator();

        let filtered_messages = self.get_filtered_messages();
        let level_counts = self.get_message_count_by_level();

        ui.horizontal(|ui| {
            ui.label(format!("Total: {}", filtered_messages.len()));
            
            for (level, count) in level_counts {
                let color = self.get_level_color(&level);
                ui.colored_label(color, format!("{}: {}", format!("{:?}", level), count));
            }
        });

        ui.separator();

        let text_style = egui::TextStyle::Monospace;
        let font_id = egui::FontId::new(self.font_size, egui::FontFamily::Monospace);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                for message in &filtered_messages {
                    self.render_message(ui, message, &font_id, text_style);
                }
            });
    }

    fn render_message(&self, ui: &mut egui::Ui, message: &ConsoleMessage, font_id: &egui::FontId, text_style: egui::TextStyle) {
        let level_color = self.get_level_color(&message.level);
        
        ui.horizontal(|ui| {
            if self.show_timestamps {
                let timestamp = message.timestamp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                ui.colored_label(egui::Color32::from_rgb(128, 128, 128), 
                    format!("[{}]", timestamp));
            }

            if self.show_levels {
                ui.colored_label(level_color, format!("[{:?}]", message.level));
            }

            ui.colored_label(egui::Color32::from_rgb(160, 160, 160), 
                format!("[{}]", message.source));

            if self.wrap_text {
                ui.label(egui::RichText::new(&message.message).font(font_id.clone()));
            } else {
                ui.label(egui::RichText::new(&message.message).font(font_id.clone()).monospace());
            }
        });

        if let Some(details) = &message.details {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.colored_label(egui::Color32::from_rgb(100, 100, 100), 
                    format!("Details: {}", details));
            });
        }
    }

    fn get_level_color(&self, level: &LogLevel) -> egui::Color32 {
        match level {
            LogLevel::Debug => egui::Color32::from_rgb(128, 128, 128),
            LogLevel::Info => egui::Color32::from_rgb(100, 150, 255),
            LogLevel::Warning => egui::Color32::from_rgb(255, 193, 7),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
            LogLevel::Critical => egui::Color32::from_rgb(255, 50, 50),
        }
    }

    pub fn add_sample_messages(&mut self) {
        self.info("System", "Tiffiny Studio console initialized");
        self.info("Audio", "Audio engine loaded successfully");
        self.warning("GPU", "GPU acceleration not available, falling back to CPU");
        self.error("File", "Failed to load project file: permission denied", Some("The file '/path/to/project.tiffiny' could not be accessed due to insufficient permissions."));
        self.critical("Memory", "Out of memory - cannot allocate additional buffers", Some("System has only 128MB free RAM available, but 256MB is required for the current operation."));
        self.debug("Render", "Frame rendered in 16.7ms (60 FPS)");
        self.info("Export", "Export completed successfully: output.mp4");
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        let mut panel = Self::new();
        panel.add_sample_messages();
        panel
    }
}
