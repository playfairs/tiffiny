use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct PreferencesDialog {
    pub id: String,
    pub title: String,
    pub categories: Vec<PreferenceCategory>,
    pub active_category: Option<String>,
    pub visible: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_save: Option<Arc<dyn Fn(std::collections::HashMap<String, serde_json::Value>) + Send + Sync>>,
    pub on_reset: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct PreferenceCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub preferences: Vec<PreferenceItem>,
}

#[derive(Debug, Clone)]
pub struct PreferenceItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub preference_type: PreferenceType,
    pub value: serde_json::Value,
    pub default_value: serde_json::Value,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub options: Option<Vec<String>>,
    pub requires_restart: bool,
}

#[derive(Debug, Clone)]
pub enum PreferenceType {
    Boolean,
    Integer,
    Float,
    String,
    Choice,
    Color,
    Path,
    Shortcut,
}

impl PreferencesDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "Preferences".to_string(),
            categories: vec![
                PreferenceCategory {
                    id: "general".to_string(),
                    name: "General".to_string(),
                    icon: "⚙".to_string(),
                    preferences: vec![
                        PreferenceItem {
                            id: "theme".to_string(),
                            name: "Theme".to_string(),
                            description: "Application theme".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("dark".to_string()),
                            default_value: serde_json::Value::String("dark".to_string()),
                            options: Some(vec!["light".to_string(), "dark".to_string(), "amoled".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "auto_save".to_string(),
                            name: "Auto-save".to_string(),
                            description: "Automatically save projects".to_string(),
                            preference_type: PreferenceType::Boolean,
                            value: serde_json::Value::Bool(true),
                            default_value: serde_json::Value::Bool(true),
                            options: None,
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "backup_interval".to_string(),
                            name: "Backup Interval".to_string(),
                            description: "Auto-backup interval in minutes".to_string(),
                            preference_type: PreferenceType::Integer,
                            value: serde_json::Value::Number(serde_json::Number::from(30)),
                            default_value: serde_json::Value::Number(serde_json::Number::from(30)),
                            options: None,
                            min_value: Some(serde_json::Value::Number(serde_json::Number::from(1))),
                            max_value: Some(serde_json::Value::Number(serde_json::Number::from(1440))),
                            requires_restart: false,
                        },
                    ],
                },
                PreferenceCategory {
                    id: "audio".to_string(),
                    name: "Audio".to_string(),
                    icon: "🎵".to_string(),
                    preferences: vec![
                        PreferenceItem {
                            id: "sample_rate".to_string(),
                            name: "Sample Rate".to_string(),
                            description: "Default audio sample rate".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("44100".to_string()),
                            default_value: serde_json::Value::String("44100".to_string()),
                            options: Some(vec!["22050".to_string(), "44100".to_string(), "48000".to_string(), "96000".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: true,
                        },
                        PreferenceItem {
                            id: "buffer_size".to_string(),
                            name: "Buffer Size".to_string(),
                            description: "Audio buffer size in samples".to_string(),
                            preference_type: PreferenceType::Integer,
                            value: serde_json::Value::Number(serde_json::Number::from(512)),
                            default_value: serde_json::Value::Number(serde_json::Number::from(512)),
                            options: None,
                            min_value: Some(serde_json::Value::Number(serde_json::Number::from(64))),
                            max_value: Some(serde_json::Value::Number(serde_json::Number::from(8192))),
                            requires_restart: true,
                        },
                        PreferenceItem {
                            id: "latency_compensation".to_string(),
                            name: "Latency Compensation".to_string(),
                            description: "Compensate for audio latency".to_string(),
                            preference_type: PreferenceType::Boolean,
                            value: serde_json::Value::Bool(true),
                            default_value: serde_json::Value::Bool(true),
                            options: None,
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                    ],
                },
                PreferenceCategory {
                    id: "video".to_string(),
                    name: "Video".to_string(),
                    icon: "🎬".to_string(),
                    preferences: vec![
                        PreferenceItem {
                            id: "default_resolution".to_string(),
                            name: "Default Resolution".to_string(),
                            description: "Default video resolution".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("1920x1080".to_string()),
                            default_value: serde_json::Value::String("1920x1080".to_string()),
                            options: Some(vec!["1280x720".to_string(), "1920x1080".to_string(), "3840x2160".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "frame_rate".to_string(),
                            name: "Frame Rate".to_string(),
                            description: "Default video frame rate".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("30".to_string()),
                            default_value: serde_json::Value::String("30".to_string()),
                            options: Some(vec!["24".to_string(), "30".to_string(), "60".to_string(), "120".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "hardware_acceleration".to_string(),
                            name: "Hardware Acceleration".to_string(),
                            description: "Use GPU acceleration when available".to_string(),
                            preference_type: PreferenceType::Boolean,
                            value: serde_json::Value::Bool(true),
                            default_value: serde_json::Value::Bool(true),
                            options: None,
                            min_value: None,
                            max_value: None,
                            requires_restart: true,
                        },
                    ],
                },
                PreferenceCategory {
                    id: "export".to_string(),
                    name: "Export".to_string(),
                    icon: "📤".to_string(),
                    preferences: vec![
                        PreferenceItem {
                            id: "default_format".to_string(),
                            name: "Default Format".to_string(),
                            description: "Default export format".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("mp4".to_string()),
                            default_value: serde_json::Value::String("mp4".to_string()),
                            options: Some(vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "avi".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "default_quality".to_string(),
                            name: "Default Quality".to_string(),
                            description: "Default export quality preset".to_string(),
                            preference_type: PreferenceType::Choice,
                            value: serde_json::Value::String("high".to_string()),
                            default_value: serde_json::Value::String("high".to_string()),
                            options: Some(vec!["low".to_string(), "medium".to_string(), "high".to_string(), "ultra".to_string(), "lossless".to_string()]),
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "export_path".to_string(),
                            name: "Export Path".to_string(),
                            description: "Default export directory".to_string(),
                            preference_type: PreferenceType::Path,
                            value: serde_json::Value::String("/exports".to_string()),
                            default_value: serde_json::Value::String("/exports".to_string()),
                            options: None,
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                    ],
                },
                PreferenceCategory {
                    id: "advanced".to_string(),
                    name: "Advanced".to_string(),
                    icon: "⚡".to_string(),
                    preferences: vec![
                        PreferenceItem {
                            id: "max_memory_usage".to_string(),
                            name: "Max Memory Usage".to_string(),
                            description: "Maximum memory usage in GB".to_string(),
                            preference_type: PreferenceType::Integer,
                            value: serde_json::Value::Number(serde_json::Number::from(8)),
                            default_value: serde_json::Value::Number(serde_json::Number::from(8)),
                            options: None,
                            min_value: Some(serde_json::Value::Number(serde_json::Number::from(1))),
                            max_value: Some(serde_json::Value::Number(serde_json::Number::from(32))),
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "cache_size".to_string(),
                            name: "Cache Size".to_string(),
                            description: "Cache size in MB".to_string(),
                            preference_type: PreferenceType::Integer,
                            value: serde_json::Value::Number(serde_json::Number::from(1024)),
                            default_value: serde_json::Value::Number(serde_json::Number::from(1024)),
                            options: None,
                            min_value: Some(serde_json::Value::Number(serde_json::Number::from(64))),
                            max_value: Some(serde_json::Value::Number(serde_json::Number::from(8192))),
                            requires_restart: false,
                        },
                        PreferenceItem {
                            id: "debug_mode".to_string(),
                            name: "Debug Mode".to_string(),
                            description: "Enable debug logging and features".to_string(),
                            preference_type: PreferenceType::Boolean,
                            value: serde_json::Value::Bool(false),
                            default_value: serde_json::Value::Bool(false),
                            options: None,
                            min_value: None,
                            max_value: None,
                            requires_restart: false,
                        },
                    ],
                },
            ],
            active_category: Some("general".to_string()),
            visible: false,
            on_close: None,
            on_save: None,
            on_reset: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn add_category(mut self, category: PreferenceCategory) -> Self {
        self.categories.push(category);
        self
    }

    pub fn active_category(mut self, category: Option<String>) -> Self {
        self.active_category = category;
        self
    }

    pub fn on_close(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(callback));
        self
    }

    pub fn on_save(mut self, callback: impl Fn(std::collections::HashMap<String, serde_json::Value>) + Send + Sync + 'static) -> Self {
        self.on_save = Some(Arc::new(callback));
        self
    }

    pub fn on_reset(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_reset = Some(Arc::new(callback));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_preference(&self, category_id: &str, preference_id: &str) -> Option<&PreferenceItem> {
        if let Some(category) = self.categories.iter().find(|cat| cat.id == category_id) {
            category.preferences.iter().find(|pref| pref.id == preference_id)
        } else {
            None
        }
    }

    pub fn get_preference_value(&self, category_id: &str, preference_id: &str) -> Option<&serde_json::Value> {
        if let Some(preference) = self.get_preference(category_id, preference_id) {
            Some(&preference.value)
        } else {
            None
        }
    }

    pub fn set_preference_value(&mut self, category_id: &str, preference_id: &str, value: serde_json::Value) -> bool {
        if let Some(category) = self.categories.iter_mut().find(|cat| cat.id == category_id) {
            if let Some(preference) = category.preferences.iter_mut().find(|pref| pref.id == preference_id) {
                preference.value = value;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn reset_to_defaults(&mut self) {
        for category in &mut self.categories {
            for preference in &mut category.preferences {
                preference.value = preference.default_value.clone();
            }
        }
    }

    pub fn export_preferences(&self) -> Result<String, String> {
        let preferences_map: std::collections::HashMap<String, serde_json::Value> = self.categories
            .iter()
            .flat_map(|category| {
                category.preferences.iter().map(|pref| {
                    (pref.id.clone(), pref.value.clone())
                })
            })
            .collect();

        serde_json::to_string(&preferences_map)
            .map_err(|e| format!("Failed to export preferences: {}", e))
    }

    pub fn import_preferences(&mut self, data: &str) -> Result<(), String> {
        let preferences_map: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(data)
            .map_err(|e| format!("Failed to import preferences: {}", e))?;

        for (category_id, category) in &mut self.categories {
            for preference in &mut category.preferences {
                if let Some(value) = preferences_map.get(&preference.id) {
                    preference.value = value.clone();
                }
            }
        }

        Ok(())
    }

    pub fn get_changed_preferences(&self) -> Vec<(&PreferenceItem)> {
        self.categories
            .iter()
            .flat_map(|category| {
                category.preferences.iter().filter(|pref| pref.value != pref.default_value)
            })
            .collect()
    }

    pub fn get_preferences_requiring_restart(&self) -> Vec<(&PreferenceItem)> {
        self.categories
            .iter()
            .flat_map(|category| {
                category.preferences.iter().filter(|pref| pref.requires_restart)
            })
            .collect()
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(800.0, 600.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
                    .rounding(8.0);

                frame.show(ui, |ui| {
                    self.render_title_bar(ui, &mut closed);
                    ui.separator();

                    ui.horizontal(|ui| {
Category list
                        self.render_category_list(ui);

                        ui.separator();

                        self.render_preferences_panel(ui);
                    });
                });
            });

        if closed {
            self.hide();
            if let Some(callback) = &self.on_close {
                callback();
            }
        }

        closed
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Save").clicked() {
                    self.save_preferences();
                    if let Some(callback) = &self.on_save {
                        callback(self.collect_current_preferences());
                    }
                }

                ui.add_space(10.0);

                if ui.button("Reset").clicked() {
                    self.reset_to_defaults();
                    if let Some(callback) = &self.on_reset {
                        callback();
                    }
                }

                ui.add_space(10.0);

                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_category_list(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Categories");
            ui.separator();

            for category in &self.categories {
                let is_active = self.active_category.as_ref()
                    .map_or(false, |active| active == &category.id);

                let button_color = if is_active {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };

                let response = ui.add(
                    egui::Button::new(format!("{} {}", category.icon, category.name))
                        .fill(button_color)
                );

                if response.clicked() {
                    tracing::info!("Selected category: {}", category.name);
                }
            }
        });
    }

    fn render_preferences_panel(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if let Some(active_category_id) = &self.active_category {
                if let Some(category) = self.categories.iter().find(|cat| cat.id == active_category_id) {
                    ui.heading(&category.name);
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for preference in &category.preferences {
                                self.render_preference_item(ui, preference);
                            }
                        });
                }
            }
        });
    }

    fn render_preference_item(&self, ui: &mut egui::Ui, preference: &PreferenceItem) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            ui.vertical(|ui| {
                ui.label(&preference.name);
                ui.colored_label(
                    ui.visuals().text_color().multiply(0.7),
                    &preference.description
                );
            });

            ui.add_space(20.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_preference_control(ui, preference);
            });
        });

        ui.separator();
    }

    fn render_preference_control(&self, ui: &mut egui::Ui, preference: &PreferenceItem) {
        match &preference.preference_type {
            PreferenceType::Boolean => {
                if let Some(serde_json::Value::Bool(mut value)) = preference.value {
                    if ui.checkbox(&mut value, "").changed() {
                    }
                }
            },
            PreferenceType::Integer => {
                if let Some(serde_json::Value::Number(ref number)) = preference.value {
                    let mut int_value = number.as_i64().unwrap_or(0);
                    let min_value = preference.min_value.as_ref()
                        .and_then(|v| v.as_i64())
                        .unwrap_or(i64::MIN);
                    let max_value = preference.max_value.as_ref()
                        .and_then(|v| v.as_i64())
                        .unwrap_or(i64::MAX);

                    if ui.add(egui::DragValue::new(&mut int_value)
                        .clamp_range(min_value..=max_value)
                    ).changed() {
                    }
                }
            },
            PreferenceType::Float => {
                if let Some(serde_json::Value::Number(ref number)) = preference.value {
                    let mut float_value = number.as_f64().unwrap_or(0.0) as f32;
                    let min_value = preference.min_value.as_ref()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f32::NEG_INFINITY);
                    let max_value = preference.max_value.as_ref()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f32::INFINITY);

                    if ui.add(egui::DragValue::new(&mut float_value)
                        .clamp_range(min_value..=max_value)
                    ).changed() {
                    }
                }
            },
            PreferenceType::String => {
                if let Some(serde_json::Value::String(ref mut value)) = preference.value {
                    if ui.add(egui::TextEdit::singleline(value)).changed() {
                    }
                }
            },
            PreferenceType::Choice => {
                if let Some(serde_json::Value::String(ref mut current_value)) = preference.value {
                    if let Some(options) = &preference.options {
                        egui::ComboBox::from_label("")
                            .selected_text(current_value)
                            .show_ui(ui, |ui| {
                                for option in options {
                                    ui.selectable_value(current_value, option, option);
                                }
                            });
                    }
                }
            },
            PreferenceType::Color => {
                if let Some(serde_json::Value::String(ref hex_value)) = preference.value {
                    ui.label(format!("Color: {}", hex_value));
                }
            },
            PreferenceType::Path => {
                if let Some(serde_json::Value::String(ref mut path_value)) = preference.value {
                    ui.horizontal(|ui| {
                        if ui.add(egui::TextEdit::singleline(&mut path_value)).changed() {
                        }

                        if ui.button("Browse...").clicked() {
                        }
                    });
                }
            },
            PreferenceType::Shortcut => {
                ui.label("Shortcut capture not implemented");
            },
        }
    }

    fn collect_current_preferences(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.categories
            .iter()
            .flat_map(|category| {
                category.preferences.iter().map(|pref| {
                    (pref.id.clone(), pref.value.clone())
                })
            })
            .collect()
    }

    fn save_preferences(&self) {
        if let Ok(preferences_json) = self.export_preferences() {
            if let Some(home_dir) = dirs::home_dir() {
                let config_path = home_dir.join(".tiffiny").join("preferences.json");
                if let Err(e) = std::fs::write(&config_path, preferences_json) {
                    tracing::error!("Failed to save preferences: {}", e);
                } else {
                    tracing::info!("Preferences saved to: {}", config_path.display());
                }
            }
        }
    }
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self::new("default_preferences_dialog".to_string())
    }
}

impl Default for PreferenceCategory {
    fn default() -> Self {
        Self {
            id: "default_category".to_string(),
            name: "Default Category".to_string(),
            icon: "📁".to_string(),
            preferences: Vec::new(),
        }
    }
}

impl Default for PreferenceItem {
    fn default() -> Self {
        Self {
            id: "default_preference".to_string(),
            name: "Default Preference".to_string(),
            description: "Default preference description".to_string(),
            preference_type: PreferenceType::Boolean,
            value: serde_json::Value::Bool(false),
            default_value: serde_json::Value::Bool(false),
            options: None,
            min_value: None,
            max_value: None,
            requires_restart: false,
        }
    }
}
