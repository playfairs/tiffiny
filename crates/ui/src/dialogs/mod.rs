pub mod file_dialog;
pub mod color_dialog;
pub mod preferences_dialog;
pub mod about_dialog;
pub mod error_dialog;
pub mod progress_dialog;
pub mod confirm_dialog;

use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct DialogManager {
    pub dialogs: Arc<RwLock<Vec<Dialog>>>,
    pub active_dialog: Arc<RwLock<Option<String>>>,
    pub modal_stack: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: String,
    pub title: String,
    pub content: DialogContent,
    pub size: DialogSize,
    pub position: DialogPosition,
    pub modal: bool,
    pub visible: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_submit: Option<Arc<dyn Fn(serde_json::Value) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum DialogContent {
    FileDialog(file_dialog::FileDialogContent),
    ColorDialog(color_dialog::ColorDialogContent),
    PreferencesDialog(preferences_dialog::PreferencesDialogContent),
    AboutDialog(about_dialog::AboutDialogContent),
    ErrorDialog(error_dialog::ErrorDialogContent),
    ProgressDialog(progress_dialog::ProgressDialogContent),
    ConfirmDialog(confirm_dialog::ConfirmDialogContent),
    Custom(CustomDialogContent),
}

#[derive(Debug, Clone)]
pub struct CustomDialogContent {
    pub render_func: Option<Arc<dyn Fn(&mut eframe::egui::Ui) + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogSize {
    Small,
    Medium,
    Large,
    Custom(f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogPosition {
    Center,
    Top,
    Bottom,
    Custom(eframe::egui::Pos2),
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            dialogs: Arc::new(RwLock::new(Vec::new())),
            active_dialog: Arc::new(RwLock::new(None)),
            modal_stack: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_dialog(&mut self, dialog: Dialog) {
        let mut dialogs = self.dialogs.write();
        dialogs.push(dialog);
    }

    pub fn remove_dialog(&mut self, dialog_id: &str) -> Option<Dialog> {
        let mut dialogs = self.dialogs.write();
        let index = dialogs.iter().position(|d| d.id == dialog_id);
        if let Some(index) = index {
            let dialog = dialogs.remove(index);
            
Remove from modal stack
            let mut modal_stack = self.modal_stack.write();
            modal_stack.retain(|id| id != dialog_id);
            
            let mut active_dialog = self.active_dialog.write();
            if active_dialog.as_ref().map_or(false, |active| active == dialog_id) {
                if let Some(new_active) = modal_stack.last() {
                    *active_dialog = Some(new_active.clone());
                } else {
                    *active_dialog = None;
                }
            }
            
            Some(dialog)
        } else {
            None
        }
    }

    pub fn show_dialog(&mut self, dialog_id: &str) -> bool {
        let mut dialogs = self.dialogs.write();
        
        if let Some(dialog) = dialogs.iter_mut().find(|d| d.id == dialog_id) {
            dialog.visible = true;
            
            let mut modal_stack = self.modal_stack.write();
            if dialog.modal && !modal_stack.contains(&dialog_id.to_string()) {
                modal_stack.push(dialog_id.to_string());
            }
            
            let mut active_dialog = self.active_dialog.write();
            *active_dialog = Some(dialog_id.to_string());
            
            true
        } else {
            false
        }
    }

    pub fn hide_dialog(&mut self, dialog_id: &str) -> bool {
        let mut dialogs = self.dialogs.write();
        
        if let Some(dialog) = dialogs.iter_mut().find(|d| d.id == dialog_id) {
            dialog.visible = false;
            
            let mut modal_stack = self.modal_stack.write();
            modal_stack.retain(|id| id != dialog_id);
            
            let mut active_dialog = self.active_dialog.write();
            if active_dialog.as_ref().map_or(false, |active| active == dialog_id) {
                if let Some(new_active) = modal_stack.last() {
                    *active_dialog = Some(new_active.clone());
                } else {
                    *active_dialog = None;
                }
            }
            
            true
        } else {
            false
        }
    }

    pub fn get_dialog(&self, dialog_id: &str) -> Option<&Dialog> {
        let dialogs = self.dialogs.read();
        dialogs.iter().find(|d| d.id == dialog_id)
    }

    pub fn get_dialog_mut(&mut self, dialog_id: &str) -> Option<&mut Dialog> {
        let mut dialogs = self.dialogs.write();
        dialogs.iter_mut().find(|d| d.id == dialog_id)
    }

    pub fn get_active_dialog(&self) -> Option<&str> {
        let active_dialog = self.active_dialog.read();
        active_dialog.as_deref()
    }

    pub fn is_dialog_active(&self, dialog_id: &str) -> bool {
        let active_dialog = self.active_dialog.read();
        active_dialog.as_ref().map_or(false, |active| active == dialog_id)
    }

    pub fn hide_all(&mut self) {
        let mut dialogs = self.dialogs.write();
        for dialog in &mut dialogs {
            dialog.visible = false;
        }
        
        let mut modal_stack = self.modal_stack.write();
        modal_stack.clear();
        
        let mut active_dialog = self.active_dialog.write();
        *active_dialog = None;
    }

    pub fn get_modal_stack(&self) -> Vec<String> {
        let modal_stack = self.modal_stack.read();
        modal_stack.clone()
    }

    pub fn render(&mut self, ui: &mut eframe::egui::Ui) {
        let mut dialogs = self.dialogs.write();
        let mut closed_dialogs = Vec::new();

        for dialog in dialogs.iter_mut().rev() {
            if dialog.visible {
                if self.render_dialog(ui, dialog) {
                    closed_dialogs.push(dialog.id.clone());
                }
            }
        }

        for dialog_id in closed_dialogs {
            self.hide_dialog(&dialog_id);
        }
    }

    fn render_dialog(&self, ui: &mut eframe::egui::Ui, dialog: &mut Dialog) -> bool {
        let mut closed = false;

        if dialog.modal {
            let screen_rect = ui.ctx().screen_rect();
            eframe::egui::Area::new(screen_rect)
                .interactable(true)
                .order(eframe::egui::Order::Foreground)
                .anchor(eframe::egui::Align2::CENTER_CENTER)
                .show(ui, |ui| {
                    let painter = ui.painter();
                    painter.rect_filled(
                        screen_rect,
                        0.0,
                        eframe::egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128)
                    );

                    let dialog_rect = self.calculate_dialog_rect(ui, screen_rect, dialog);
                    
                    eframe::egui::Area::new(dialog_rect)
                        .interactable(true)
                        .movable(false)
                        .order(eframe::egui::Order::Foreground)
                        .show(ui, |ui| {
                            self.render_dialog_window(ui, dialog, &mut closed);
                        });
                });
        } else {
            let screen_rect = ui.ctx().screen_rect();
            let dialog_rect = self.calculate_dialog_rect(ui, screen_rect, dialog);
            
            eframe::egui::Area::new(dialog_rect)
                .interactable(true)
                .movable(false)
                .order(eframe::egui::Order::Foreground)
                .show(ui, |ui| {
                    self.render_dialog_window(ui, dialog, &mut closed);
                });
        }

        if closed {
            if let Some(callback) = &dialog.on_close {
                callback();
            }
        }

        closed
    }

    fn calculate_dialog_rect(&self, ui: &mut eframe::egui::Ui, screen_rect: eframe::egui::Rect, dialog: &Dialog) -> eframe::egui::Rect {
        let (width, height) = match dialog.size {
            DialogSize::Small => (400.0, 200.0),
            DialogSize::Medium => (600.0, 400.0),
            DialogSize::Large => (800.0, 600.0),
            DialogSize::Custom(width, height) => (width, height),
        };

        let center = match dialog.position {
            DialogPosition::Center => screen_rect.center(),
            DialogPosition::Top => eframe::egui::pos2(screen_rect.center().x, screen_rect.min.y + 50.0),
            DialogPosition::Bottom => eframe::egui::pos2(screen_rect.center().x, screen_rect.max.y - 50.0),
            DialogPosition::Custom(pos) => pos,
        };

        eframe::egui::Rect::from_center_size(center, eframe::egui::vec2(width, height))
    }

    fn render_dialog_window(&self, ui: &mut eframe::egui::Ui, dialog: &mut Dialog, closed: &mut bool) {
        let frame = eframe::egui::Frame::dark_canvas(ui.style())
            .stroke(eframe::egui::Stroke::new(2.0, ui.visuals().window_fill))
            .rounding(8.0);

        frame.show(ui, |ui| {
            self.render_title_bar(ui, dialog, closed);

            ui.separator();

            self.render_dialog_content(ui, dialog, closed);
        });
    }

    fn render_title_bar(&self, ui: &mut eframe::egui::Ui, dialog: &mut Dialog, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&dialog.title);
            
            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_dialog_content(&self, ui: &mut eframe::egui::Ui, dialog: &mut Dialog, closed: &mut bool) {
        match &mut dialog.content {
            DialogContent::FileDialog(file_dialog_content) => {
                file_dialog::render_file_dialog(ui, file_dialog_content, closed);
            },
            DialogContent::ColorDialog(color_dialog_content) => {
                color_dialog::render_color_dialog(ui, color_dialog_content, closed);
            },
            DialogContent::PreferencesDialog(preferences_dialog_content) => {
                preferences_dialog::render_preferences_dialog(ui, preferences_dialog_content, closed);
            },
            DialogContent::AboutDialog(about_dialog_content) => {
                about_dialog::render_about_dialog(ui, about_dialog_content, closed);
            },
            DialogContent::ErrorDialog(error_dialog_content) => {
                error_dialog::render_error_dialog(ui, error_dialog_content, closed);
            },
            DialogContent::ProgressDialog(progress_dialog_content) => {
                progress_dialog::render_progress_dialog(ui, progress_dialog_content, closed);
            },
            DialogContent::ConfirmDialog(confirm_dialog_content) => {
                confirm_dialog::render_confirm_dialog(ui, confirm_dialog_content, closed);
            },
            DialogContent::Custom(custom_content) => {
                if let Some(render_func) = &custom_content.render_func {
                    render_func(ui);
                }
            },
        }
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            id: "default_dialog".to_string(),
            title: "Default Dialog".to_string(),
            content: DialogContent::Custom(CustomDialogContent {
                render_func: None,
            }),
            size: DialogSize::Medium,
            position: DialogPosition::Center,
            modal: true,
            visible: false,
            on_close: None,
            on_submit: None,
        }
    }
}

impl Default for CustomDialogContent {
    fn default() -> Self {
        Self {
            render_func: None,
        }
    }
}
