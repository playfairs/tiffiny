use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub id: String,
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub confirm_style: ConfirmStyle,
    pub cancel_style: ConfirmStyle,
    pub show_icon: bool,
    pub icon_type: IconType,
    pub visible: bool,
    pub on_confirm: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmStyle {
    Primary,
    Secondary,
    Danger,
    Warning,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IconType {
    Warning,
    Error,
    Info,
    Question,
    Success,
}

impl ConfirmDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "Confirm Action".to_string(),
            message: String::new(),
            confirm_text: "OK".to_string(),
            cancel_text: "Cancel".to_string(),
            confirm_style: ConfirmStyle::Primary,
            cancel_style: ConfirmStyle::Secondary,
            show_icon: false,
            icon_type: IconType::Question,
            visible: false,
            on_confirm: None,
            on_cancel: None,
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

    pub fn confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    pub fn cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    pub fn confirm_style(mut self, style: ConfirmStyle) -> Self {
        self.confirm_style = style;
        self
    }

    pub fn cancel_style(mut self, style: ConfirmStyle) -> Self {
        self.cancel_style = style;
        self
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn icon_type(mut self, icon_type: IconType) -> Self {
        self.icon_type = icon_type;
        self
    }

    pub fn on_confirm(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = Some(Arc::new(callback));
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(callback));
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

    pub fn show_confirmation(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_style = ConfirmStyle::Warning;
        self.show_icon = true;
        self.icon_type = IconType::Question;
        self.show();
    }

    pub fn show_warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_style = ConfirmStyle::Warning;
        self.confirm_text = "OK".to_string();
        self.cancel_text = "Cancel".to_string();
        self.show_icon = true;
        self.icon_type = IconType::Warning;
        self.show();
    }

    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_style = ConfirmStyle::Danger;
        self.confirm_text = "OK".to_string();
        self.cancel_text = "Cancel".to_string();
        self.show_icon = true;
        self.icon_type = IconType::Error;
        self.show();
    }

    pub fn show_info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.confirm_style = ConfirmStyle::Primary;
        self.confirm_text = "OK".to_string();
        self.cancel_text = "Cancel".to_string();
        self.show_icon = true;
        self.icon_type = IconType::Info;
        self.show();
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut result = false;

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
                    .stroke(self.get_border_color())
                    .rounding(8.0);

                frame.show(ui, |ui| {
Title bar
                    self.render_title_bar(ui, &mut result);

                    ui.separator();

                    self.render_content(ui, &mut result);
                });
            });

        if result {
            self.hide();
            if let Some(callback) = &self.on_confirm {
                callback();
            }
        }

        result
    }

    fn get_border_color(&self) -> egui::Stroke {
        match self.confirm_style {
            ConfirmStyle::Primary => egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            ConfirmStyle::Secondary => egui::Stroke::new(2.0, egui::Color32::from_rgb(88, 88, 88)),
            ConfirmStyle::Danger => egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 53, 69)),
            ConfirmStyle::Warning => egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 193, 7)),
        }
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, result: &mut bool) {
        ui.horizontal(|ui| {
            if self.show_icon {
                let icon_text = match self.icon_type {
                    IconType::Warning => "⚠",
                    IconType::Error => "❌",
                    IconType::Info => "ℹ",
                    IconType::Question => "❓",
                    IconType::Success => "✓",
                };

                ui.colored_label(
                    self.get_icon_color(),
                    icon_text
                );
                ui.add_space(10.0);
            }

            ui.colored_label(
                ui.visuals().text_color(),
                &self.title
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *result = true;
                }
            });
        });
    }

    fn get_icon_color(&self) -> egui::Color32 {
        match self.icon_type {
            IconType::Warning => egui::Color32::from_rgb(255, 193, 7),
            IconType::Error => egui::Color32::from_rgb(220, 53, 69),
            IconType::Info => egui::Color32::from_rgb(100, 150, 255),
            IconType::Question => egui::Color32::from_rgb(100, 150, 255),
            IconType::Success => egui::Color32::from_rgb(46, 160, 67),
        }
    }

    fn render_content(&self, ui: &mut egui::Ui, result: &mut bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(&self.message);
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let cancel_color = match self.cancel_style {
                    ConfirmStyle::Primary => ui.visuals().widgets.active.bg_fill,
                    ConfirmStyle::Secondary => egui::Color32::from_rgb(88, 88, 88),
                    ConfirmStyle::Danger => egui::Color32::from_rgb(220, 53, 69),
                    ConfirmStyle::Warning => egui::Color32::from_rgb(255, 193, 7),
                };

                if ui.add(
                    egui::Button::new(&self.cancel_text)
                        .fill(cancel_color)
                        .stroke(self.get_cancel_border_color())
                ).clicked() {
                    *result = false;
                }

                ui.add_space(20.0);

                let confirm_color = match self.confirm_style {
                    ConfirmStyle::Primary => ui.visuals().widgets.active.bg_fill,
                    ConfirmStyle::Secondary => egui::Color32::from_rgb(88, 88, 88),
                    ConfirmStyle::Danger => egui::Color32::from_rgb(220, 53, 69),
                    ConfirmStyle::Warning => egui::Color32::from_rgb(255, 193, 7),
                };

                if ui.add(
                    egui::Button::new(&self.confirm_text)
                        .fill(confirm_color)
                        .stroke(self.get_confirm_border_color())
                ).clicked() {
                    *result = true;
                }
            });
        });

        ui.add_space(20.0);
    }

    fn get_confirm_border_color(&self) -> egui::Stroke {
        match self.confirm_style {
            ConfirmStyle::Primary => egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            ConfirmStyle::Secondary => egui::Stroke::new(2.0, egui::Color32::from_rgb(88, 88, 88)),
            ConfirmStyle::Danger => egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 53, 69)),
            ConfirmStyle::Warning => egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 193, 7)),
        }
    }

    fn get_cancel_border_color(&self) -> egui::Stroke {
        match self.cancel_style {
            ConfirmStyle::Primary => egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            ConfirmStyle::Secondary => egui::Stroke::new(2.0, egui::Color32::from_rgb(88, 88, 88)),
            ConfirmStyle::Danger => egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 53, 69)),
            ConfirmStyle::Warning => egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 193, 7)),
        }
    }
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        Self::new("default_confirm_dialog".to_string())
    }
}
