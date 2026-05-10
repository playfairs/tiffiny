use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct TextInput {
    pub id: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub multiline: bool,
    pub password: bool,
    pub max_length: Option<usize>,
    pub validator: Option<Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
    pub enabled: bool,
    pub visible: bool,
    pub on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_focus: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_blur: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl TextInput {
    pub fn new(id: String) -> Self {
        Self {
            id,
            label: String::new(),
            value: String::new(),
            placeholder: String::new(),
            multiline: false,
            password: false,
            max_length: None,
            validator: None,
            enabled: true,
            visible: true,
            on_change: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn validator(mut self, validator: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static) -> Self {
        self.validator = Some(Arc::new(validator));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(callback));
        self
    }

    pub fn on_submit(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(callback));
        self
    }

    pub fn on_focus(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_focus = Some(Arc::new(callback));
        self
    }

    pub fn on_blur(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;
        let mut submitted = false;

        if !self.label.is_empty() {
            ui.label(&self.label);
        }

        let display_value = if self.password {
            "•".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        };

        let response = if self.multiline {
            let mut text_edit = egui::TextEdit::multiline(&mut self.value)
                .desired_width(f32::INFINITY)
                .desired_rows(if self.value.is_empty() { 3 } else { 0 })
                .hint_text(&self.placeholder);

            if !self.enabled {
                text_edit = text_edit.interactive(false);
            }

            ui.add(text_edit)
        } else {
            let mut text_edit = egui::TextEdit::singleline(&mut self.value)
                .desired_width(f32::INFINITY)
                .hint_text(&self.placeholder);

            if !self.enabled {
                text_edit = text_edit.interactive(false);
            }

            ui.add(text_edit)
        };

        changed = response.changed();
        submitted = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if changed {
            if let Some(max_length) = self.max_length {
                if self.value.len() > max_length {
                    self.value.truncate(max_length);
                }
            }

            if let Some(validator) = &self.validator {
                if let Err(error) = validator(&self.value) {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &error);
                    return changed;
                }
            }

            if let Some(callback) = &self.on_change {
                callback(self.value.clone());
            }
        }

        if submitted {
            if let Some(callback) = &self.on_submit {
                callback(self.value.clone());
            }
        }

        if response.gained_focus() {
            if let Some(callback) = &self.on_focus {
                callback();
            }
        }

        if response.lost_focus() {
            if let Some(callback) = &self.on_blur {
                callback();
            }
        }

        changed || submitted
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }

    pub fn is_valid(&self) -> bool {
        if let Some(validator) = &self.validator {
            validator(&self.value).is_ok()
        } else {
            true
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new("default_text_input".to_string())
    }
}

pub struct SearchInput {
    pub id: String,
    pub query: String,
    pub placeholder: String,
    pub search_delay: f32,
    pub enabled: bool,
    pub visible: bool,
    pub on_search: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_clear: Option<Arc<dyn Fn() + Send + Sync>>,
    search_timer: Option<f32>,
}

impl SearchInput {
    pub fn new(id: String) -> Self {
        Self {
            id,
            query: String::new(),
            placeholder: "Search...".to_string(),
            search_delay: 0.5,
            enabled: true,
            visible: true,
            on_search: None,
            on_clear: None,
            search_timer: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn search_delay(mut self, delay: f32) -> Self {
        self.search_delay = delay;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn on_search(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_search = Some(Arc::new(callback));
        self
    }

    pub fn on_clear(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_clear = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        ui.horizontal(|ui| {
            let mut text_edit = egui::TextEdit::singleline(&mut self.query)
                .desired_width(f32::INFINITY)
                .hint_text(&self.placeholder);

            if !self.enabled {
                text_edit = text_edit.interactive(false);
            }

            let response = ui.add(text_edit);
            changed = response.changed();

            if !self.query.is_empty() {
                if ui.button("✕").clicked() {
                    self.query.clear();
                    if let Some(callback) = &self.on_clear {
                        callback();
                    }
                }
            }
        });

        if changed {
            self.search_timer = Some(self.search_delay);
        }

        if let Some(ref mut timer) = self.search_timer {
            *timer -= ui.ctx().frame_time();
            if *timer <= 0.0 {
                self.search_timer = None;
                if let Some(callback) = &self.on_search {
                    callback(self.query.clone());
                }
            }
        }

        changed
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.search_timer = None;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.search_timer = None;
        if let Some(callback) = &self.on_clear {
            callback();
        }
    }
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new("default_search_input".to_string())
    }
}
