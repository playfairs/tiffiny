use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ErrorDialog {
    pub id: String,
    pub title: String,
    pub error_type: ErrorType,
    pub message: String,
    pub details: Option<String>,
    pub show_details: bool,
    pub can_retry: bool,
    pub retry_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    pub visible: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Error,
    Warning,
    Critical,
    Info,
}

impl ErrorDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "Error".to_string(),
            error_type: ErrorType::Error,
            message: String::new(),
            details: None,
            show_details: false,
            can_retry: false,
            retry_callback: None,
            visible: false,
            on_close: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn error_type(mut self, error_type: ErrorType) -> Self {
        self.error_type = error_type;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn details(mut self, details: Option<String>) -> Self {
        self.details = details;
        self
    }

    pub fn can_retry(mut self, can_retry: bool) -> Self {
        self.can_retry = can_retry;
        self
    }

    pub fn retry_callback(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.retry_callback = Some(Arc::new(callback));
        self
    }

    pub fn on_close(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(callback));
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

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.error_type = ErrorType::Error;
        self.message = message.into();
        self.can_retry = false;
        self.show();
    }

    pub fn show_warning(&mut self, message: impl Into<String>) {
        self.error_type = ErrorType::Warning;
        self.message = message.into();
        self.can_retry = true;
        self.show();
    }

    pub fn show_critical(&mut self, message: impl Into<String>) {
        self.error_type = ErrorType::Critical;
        self.message = message.into();
        self.can_retry = false;
        self.show();
    }

    pub fn show_info(&mut self, message: impl Into<String>) {
        self.error_type = ErrorType::Info;
        self.message = message.into();
        self.can_retry = false;
        self.show();
    }

    pub fn show_with_details(&mut self, message: impl Into<String>, details: impl Into<String>) {
        self.message = message.into();
        self.details = Some(details.into());
        self.show_details = true;
        self.show();
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(500.0, 400.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(self.get_border_color())
                    .rounding(8.0);

                frame.show(ui, |ui| {
Title bar
                    self.render_title_bar(ui, &mut closed);

                    ui.separator();

                    self.render_content(ui);
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

    fn get_border_color(&self) -> egui::Stroke {
        match self.error_type {
            ErrorType::Error => egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 53, 69)),
            ErrorType::Warning => egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 193, 7)),
            ErrorType::Critical => egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
            ErrorType::Info => egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
        }
    }

    fn get_icon(&self) -> &'static str {
        match self.error_type {
            ErrorType::Error => "❌",
            ErrorType::Warning => "⚠",
            ErrorType::Critical => "🔴",
            ErrorType::Info => "ℹ",
        }
    }

    fn get_title_color(&self) -> egui::Color32 {
        match self.error_type {
            ErrorType::Error => egui::Color32::from_rgb(220, 53, 69),
            ErrorType::Warning => egui::Color32::from_rgb(255, 193, 7),
            ErrorType::Critical => egui::Color32::from_rgb(255, 0, 0),
            ErrorType::Info => egui::Color32::from_rgb(100, 150, 255),
        }
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(self.get_icon());
            ui.add_space(10.0);
            
            ui.colored_label(
                self.get_title_color(),
                &self.title
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(&self.message);
            });

            ui.add_space(20.0);

            if let Some(details) = &self.details {
                ui.horizontal(|ui| {
                    ui.label("Details:");
                    ui.add_space(10.0);
                    
                    if ui.button(if self.show_details { "Hide ▲" } else { "Show ▼" }).clicked() {
                        self.show_details = !self.show_details;
                    }
                });
            });

            if self.show_details {
                ui.add_space(10.0);
                
                let details_frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)))
                    .rounding(4.0);

                details_frame.show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(details);
                    });
                });
            }
        });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if self.can_retry {
                if ui.button("Retry").clicked() {
                    if let Some(callback) = &self.retry_callback {
                        callback();
                    }
                }
            }

            ui.add_space(10.0);

            if ui.button("OK").clicked() {
            }
        });
    }
}

impl Default for ErrorDialog {
    fn default() -> Self {
        Self::new("default_error_dialog".to_string())
    }
}
