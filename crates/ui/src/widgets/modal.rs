use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct Modal {
    pub id: String,
    pub title: String,
    pub content: ModalContent,
    pub size: ModalSize,
    pub position: ModalPosition,
    pub resizable: bool,
    pub modal: bool,
    pub visible: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_submit: Option<Arc<dyn Fn(serde_json::Value) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum ModalContent {
    Text(TextModalContent),
    Input(InputModalContent),
    Confirm(ConfirmModalContent),
    Custom(CustomModalContent),
}

#[derive(Debug, Clone)]
pub struct TextModalContent {
    pub text: String,
    pub scrollable: bool,
}

#[derive(Debug, Clone)]
pub struct InputModalContent {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub multiline: bool,
    pub password: bool,
    pub validator: Option<Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ConfirmModalContent {
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub confirm_style: ButtonStyle,
    pub cancel_style: ButtonStyle,
}

#[derive(Debug, Clone)]
pub struct CustomModalContent {
    pub render_func: Option<Arc<dyn Fn(&mut egui::Ui) + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    Custom(f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalPosition {
    Center,
    Top,
    Bottom,
    Custom(egui::Pos2),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
}

impl Modal {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: String::new(),
            content: ModalContent::Text(TextModalContent {
                text: String::new(),
                scrollable: false,
            }),
            size: ModalSize::Medium,
            position: ModalPosition::Center,
            resizable: false,
            modal: true,
            visible: false,
            on_close: None,
            on_submit: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn content(mut self, content: ModalContent) -> Self {
        self.content = content;
        self
    }

    pub fn size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    pub fn position(mut self, position: ModalPosition) -> Self {
        self.position = position;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn on_close(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(callback));
        self
    }

    pub fn on_submit(mut self, callback: impl Fn(serde_json::Value) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(callback));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

Modal backdrop
        let screen_rect = ui.ctx().screen_rect();
        egui::Area::new(screen_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER)
            .show(ui, |ui| {
                let painter = ui.painter();
                painter.rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128)
                );

                let modal_rect = self.calculate_modal_rect(ui, screen_rect);
                
                egui::Area::new(modal_rect)
                    .interactable(true)
                    .movable(self.resizable)
                    .order(egui::Order::Foreground)
                    .show(ui, |ui| {
                        self.render_modal_window(ui, &mut closed);
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

    fn calculate_modal_rect(&self, ui: &mut egui::Ui, screen_rect: egui::Rect) -> egui::Rect {
        let (width, height) = match self.size {
            ModalSize::Small => (400.0, 200.0),
            ModalSize::Medium => (600.0, 400.0),
            ModalSize::Large => (800.0, 600.0),
            ModalSize::Custom(width, height) => (width, height),
        };

        let center = match self.position {
            ModalPosition::Center => screen_rect.center(),
            ModalPosition::Top => egui::pos2(screen_rect.center().x, screen_rect.min.y + 50.0),
            ModalPosition::Bottom => egui::pos2(screen_rect.center().x, screen_rect.max.y - 50.0),
            ModalPosition::Custom(pos) => pos,
        };

        egui::Rect::from_center_size(center, egui::vec2(width, height))
    }

    fn render_modal_window(&mut self, ui: &mut egui::Ui, closed: &mut bool) {
        let frame = egui::Frame::dark_canvas(ui.style())
            .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
            .rounding(8.0);

        frame.show(ui, |ui| {
            self.render_title_bar(ui, closed);

            ui.separator();

            self.render_content(ui, closed);
        });
    }

    fn render_title_bar(&mut self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, closed: &mut bool) {
        match &mut self.content {
            ModalContent::Text(text_content) => {
                self.render_text_modal(ui, text_content);
            },
            ModalContent::Input(input_content) => {
                self.render_input_modal(ui, input_content, closed);
            },
            ModalContent::Confirm(confirm_content) => {
                self.render_confirm_modal(ui, confirm_content, closed);
            },
            ModalContent::Custom(custom_content) => {
                if let Some(render_func) = &custom_content.render_func {
                    render_func(ui);
                }
            },
        }
    }

    fn render_text_modal(&self, ui: &mut egui::Ui, content: &TextModalContent) {
        if content.scrollable {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(&content.text);
                });
        } else {
            ui.label(&content.text);
        }
    }

    fn render_input_modal(&mut self, ui: &mut egui::Ui, content: &mut InputModalContent, closed: &mut bool) {
        ui.label(&content.label);
        ui.add_space(8.0);

        let mut text_edit = if content.multiline {
            egui::TextEdit::multiline(&mut content.value)
                .desired_width(f32::INFINITY)
                .hint_text(&content.placeholder)
        } else {
            egui::TextEdit::singleline(&mut content.value)
                .desired_width(f32::INFINITY)
                .hint_text(&content.placeholder)
        };

        if content.password {
            text_edit = text_edit.password(true);
        }

        let response = ui.add(text_edit);

        if response.changed() {
            if let Some(validator) = &content.validator {
                if let Err(error) = validator(&content.value) {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &error);
                }
            }
        }

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(validator) = &content.validator {
                if validator(&content.value).is_ok() {
                    if let Some(callback) = &self.on_submit {
                        callback(serde_json::Value::String(content.value.clone()));
                    }
                    *closed = true;
                }
            } else {
                if let Some(callback) = &self.on_submit {
                    callback(serde_json::Value::String(content.value.clone()));
                }
                *closed = true;
            }
        }
    }

    fn render_confirm_modal(&mut self, ui: &mut egui::Ui, content: &ConfirmModalContent, closed: &mut bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(&content.message);
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button(&content.cancel_text)
                    .fill(self.get_button_color(&content.cancel_style, ui))
                    .stroke(self.get_button_stroke(&content.cancel_style, ui))
                    .clicked() {
                        *closed = true;
                    } {
                        if let Some(callback) = &self.on_close {
                            callback();
                        }
                    }
                };

                ui.add_space(20.0);

                if ui.button(&content.confirm_text)
                    .fill(self.get_button_color(&content.confirm_style, ui))
                    .stroke(self.get_button_stroke(&content.confirm_style, ui))
                    .clicked() {
                        if let Some(callback) = &self.on_submit {
                            callback(serde_json::Value::Bool(true));
                        }
                        *closed = true;
                    }
                };
            });
        });
    }

    fn get_button_color(&self, style: &ButtonStyle, ui: &egui::Ui) -> egui::Color32 {
        match style {
            ButtonStyle::Primary => ui.visuals().widgets.active.bg_fill,
            ButtonStyle::Secondary => egui::Color32::from_rgb(88, 88, 88),
            ButtonStyle::Danger => egui::Color32::from_rgb(220, 53, 69),
        }
    }

    fn get_button_stroke(&self, style: &ButtonStyle, ui: &egui::Ui) -> egui::Stroke {
        match style {
            ButtonStyle::Primary => ui.visuals().widgets.active.bg_stroke,
            ButtonStyle::Secondary => ui.visuals().widgets.active.bg_stroke,
            ButtonStyle::Danger => ui.visuals().widgets.active.bg_stroke,
        }
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new("default_modal".to_string())
    }
}

pub struct ModalManager {
    pub modals: std::collections::HashMap<String, Modal>,
    pub active_modal: Option<String>,
    pub modal_stack: Vec<String>,
}

impl ModalManager {
    pub fn new() -> Self {
        Self {
            modals: std::collections::HashMap::new(),
            active_modal: None,
            modal_stack: Vec::new(),
        }
    }

    pub fn add_modal(&mut self, modal: Modal) {
        self.modals.insert(modal.id.clone(), modal);
    }

    pub fn remove_modal(&mut self, modal_id: &str) -> Option<Modal> {
        self.modal_stack.retain(|id| id != modal_id);
        if self.active_modal.as_ref().map_or(false, |active| active == modal_id) {
            if let Some(new_active) = self.modal_stack.last() {
                self.active_modal = Some(new_active.clone());
            } else {
                self.active_modal = None;
            }
        }
        self.modals.remove(modal_id)
    }

    pub fn show_modal(&mut self, modal_id: &str) -> bool {
        if let Some(modal) = self.modals.get_mut(modal_id) {
            modal.show();
            self.modal_stack.push(modal_id.to_string());
            self.active_modal = Some(modal_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn hide_modal(&mut self, modal_id: &str) -> bool {
        if let Some(modal) = self.modals.get_mut(modal_id) {
            modal.hide();
            self.modal_stack.retain(|id| id != modal_id);
            if self.active_modal.as_ref().map_or(false, |active| active == modal_id) {
                if let Some(new_active) = self.modal_stack.last() {
                    self.active_modal = Some(new_active.clone());
                } else {
                    self.active_modal = None;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn get_modal(&self, modal_id: &str) -> Option<&Modal> {
        self.modals.get(modal_id)
    }

    pub fn get_active_modal(&self) -> Option<&str> {
        self.active_modal.as_deref()
    }

    pub fn get_modal_mut(&mut self, modal_id: &str) -> Option<&mut Modal> {
        self.modals.get_mut(modal_id)
    }

    pub fn is_modal_active(&self, modal_id: &str) -> bool {
        self.active_modal.as_ref().map_or(false, |active| active == modal_id)
    }

    pub fn hide_all(&mut self) {
        for modal in self.modals.values_mut() {
            modal.hide();
        }
        self.modal_stack.clear();
        self.active_modal = None;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let mut closed_modals = Vec::new();

        for modal_id in self.modal_stack.iter().rev() {
            if let Some(modal) = self.modals.get_mut(modal_id) {
                if modal.render(ui) {
                    closed_modals.push(modal_id.clone());
                }
            }
        }

        for closed_modal_id in closed_modals {
            self.hide_modal(&closed_modal_id);
        }
    }
}

impl Default for ModalManager {
    fn default() -> Self {
        Self::new()
    }
}
