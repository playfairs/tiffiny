use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ProgressDialog {
    pub id: String,
    pub title: String,
    pub message: String,
    pub progress: f32,
    pub min: f32,
    pub max: f32,
    pub show_percentage: bool,
    pub show_cancel: bool,
    pub cancel_text: String,
    pub visible: bool,
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_complete: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ProgressDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "Progress".to_string(),
            message: String::new(),
            progress: 0.0,
            min: 0.0,
            max: 1.0,
            show_percentage: true,
            show_cancel: true,
            cancel_text: "Cancel".to_string(),
            visible: false,
            on_cancel: None,
            on_complete: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(self.min, self.max);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.progress = self.progress.clamp(min, max);
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn show_cancel(mut self, show: bool) -> Self {
        self.show_cancel = show;
        self
    }

    pub fn cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(callback));
        self
    }

    pub fn on_complete(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_complete = Some(Arc::new(callback));
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

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(self.min, self.max);
    }

    pub fn increment_progress(&mut self, amount: f32) {
        self.progress = (self.progress + amount).clamp(self.min, self.max);
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn get_normalized_progress(&self) -> f32 {
        if self.max > self.min {
            (self.progress - self.min) / (self.max - self.min)
        } else {
            0.0
        }.clamp(0.0, 1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.max
    }

    pub fn reset(&mut self) {
        self.progress = self.min;
    }

    pub fn complete(&mut self) {
        self.progress = self.max;
        
        if let Some(callback) = &self.on_complete {
            callback();
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut cancelled = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(400.0, 200.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
                    .rounding(8.0);

                frame.show(ui, |ui| {
Title bar
                    self.render_title_bar(ui, &mut cancelled);

                    ui.separator();

                    self.render_content(ui);
                });
            });

        if cancelled {
            self.hide();
            if let Some(callback) = &self.on_cancel {
                callback();
            }
        }

        cancelled
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, cancelled: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.show_cancel {
                    if ui.button(&self.cancel_text).clicked() {
                        *cancelled = true;
                    }
                }
            });
        });
    }

    fn render_content(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            if !self.message.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.message);
                });
                ui.add_space(20.0);
            }

            let progress_bar = egui::ProgressBar::new(self.get_normalized_progress())
                .desired_width(f32::INFINITY)
                .show_percentage(self.show_percentage)
                .text(if self.show_percentage {
                    Some(format!("{:.1}%", self.get_normalized_progress() * 100.0))
                } else {
                    None
                });

            ui.add(progress_bar);

            if self.show_percentage {
                ui.horizontal(|ui| {
                    ui.label(format!("{:.1} / {:.1}", self.progress, self.max));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}%", self.get_normalized_progress() * 100.0));
                    });
                });
            }

            ui.add_space(20.0);

            let status_color = if self.is_complete() {
                egui::Color32::from_rgb(46, 160, 67)
            } else {
                egui::Color32::from_rgb(255, 193, 7)
            };

            ui.colored_label(
                status_color,
                if self.is_complete() {
                    "Complete"
                } else {
                    "In Progress"
                }
            );
        });
    }
}

impl Default for ProgressDialog {
    fn default() -> Self {
        Self::new("default_progress_dialog".to_string())
    }
}
