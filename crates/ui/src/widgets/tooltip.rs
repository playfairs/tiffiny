use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct Tooltip {
    pub id: String,
    pub text: String,
    pub title: Option<String>,
    pub position: TooltipPosition,
    pub delay: f32,
    pub duration: Option<f32>,
    pub max_width: Option<f32>,
    pub visible: bool,
    pub on_show: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_hide: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TooltipPosition {
    Cursor,
    Above,
    Below,
    Left,
    Right,
    Center,
}

impl Tooltip {
    pub fn new(id: String) -> Self {
        Self {
            id,
            text: String::new(),
            title: None,
            position: TooltipPosition::Cursor,
            delay: 0.5,
            duration: None,
            max_width: Some(300.0),
            visible: false,
            on_show: None,
            on_hide: None,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn on_show(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_show = Some(Arc::new(callback));
        self
    }

    pub fn on_hide(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_hide = Some(Arc::new(callback));
        self
    }

    pub fn show_at(&mut self, ui: &mut egui::Ui, pos: egui::Pos2) {
        if !self.visible {
            return;
        }

        let screen_rect = ui.ctx().screen_rect();
        let tooltip_rect = self.calculate_tooltip_rect(ui, pos);

Ensure tooltip stays on screen
        let final_rect = egui::Rect {
            min: egui::pos2(
                tooltip_rect.min.x.max(screen_rect.min.x),
                tooltip_rect.min.y.max(screen_rect.min.y)
            ),
            max: egui::pos2(
                tooltip_rect.max.x.min(screen_rect.max.x),
                tooltip_rect.max.y.min(screen_rect.max.y)
            ),
        };

        egui::Area::new(final_rect)
            .movable(false)
            .interactable(false)
            .anchor(egui::Align2::LEFT_TOP)
            .show(ui.ctx(), |ui| {
                self.render_tooltip_content(ui);
            });

        if let Some(callback) = &self.on_show {
            callback();
        }

        self.visible = true;

        if let Some(duration) = self.duration {
            ui.ctx().request_repaint_after(std::time::Duration::from_secs_f32(duration));
        }
    }

    fn calculate_tooltip_rect(&self, ui: &mut egui::Ui, pos: egui::Pos2) -> egui::Rect {
        let text_style = egui::TextStyle::Body;
        let font_id = egui::FontId::default();

        let title_size = if let Some(title) = &self.title {
            ui.painter().layout_text(
                egui::RichText::new(title)
                    .font(font_id.clone())
                    .style(text_style),
                ui.available_width()
            ).size
        } else {
            egui::vec2(0.0, 0.0)
        };

        let text_size = ui.painter().layout_text(
            egui::RichText::new(&self.text)
                .font(font_id)
                .style(text_style),
            self.max_width.unwrap_or(f32::INFINITY)
        ).size;

        let padding = egui::vec2(8.0, 6.0);
        let total_size = egui::vec2(
            title_size.x.max(text_size.x) + padding.x * 2.0,
            title_size.y + text_size.y + padding.y * 2.0
        );

        let tooltip_pos = match self.position {
            TooltipPosition::Cursor => pos,
            TooltipPosition::Above => egui::pos2(pos.x, pos.y - total_size.y - 5.0),
            TooltipPosition::Below => egui::pos2(pos.x, pos.y + 25.0),
            TooltipPosition::Left => egui::pos2(pos.x - total_size.x - 5.0, pos.y),
            TooltipPosition::Right => egui::pos2(pos.x + 25.0, pos.y),
            TooltipPosition::Center => egui::pos2(
                pos.x - total_size.x / 2.0,
                pos.y - total_size.y / 2.0
            ),
        };

        egui::Rect::from_min_size(tooltip_pos, total_size)
    }

    fn render_tooltip_content(&self, ui: &mut egui::Ui) {
        let bg_color = ui.visuals().extreme_bg_color;
        let text_color = ui.visuals().text_color();
        let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let border_width = ui.visuals().widgets.noninteractive.bg_stroke.width;

        egui::Frame::dark_canvas(ui.style())
            .stroke(egui::Stroke::new(border_width, border_color))
            .rounding(4.0)
            .fill(bg_color)
            .show(ui, |ui| {
                if let Some(title) = &self.title {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 150, 255),
                            title
                        );
                    });
                    ui.separator();
                }

                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    ui.label(&self.text);
                });
            });
    }

    pub fn hide(&mut self) {
        if self.visible {
            self.visible = false;
            
            if let Some(callback) = &self.on_hide {
                callback();
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.visible = true;
            
            if let Some(callback) = &self.on_show {
                callback();
            }
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new("default_tooltip".to_string())
    }
}

pub struct TooltipManager {
    pub tooltips: std::collections::HashMap<String, Tooltip>,
    pub active_tooltip: Option<String>,
    pub default_delay: f32,
    pub default_duration: f32,
}

impl TooltipManager {
    pub fn new() -> Self {
        Self {
            tooltips: std::collections::HashMap::new(),
            active_tooltip: None,
            default_delay: 0.5,
            default_duration: 3.0,
        }
    }

    pub fn create_tooltip(&mut self, id: String) -> &mut Tooltip {
        self.tooltips.entry(id.clone()).or_insert_with(|| {
            Tooltip::new(id.clone())
                .delay(self.default_delay)
                .duration(self.default_duration)
        })
    }

    pub fn show_tooltip(&mut self, id: &str, ui: &mut egui::Ui, pos: egui::Pos2, text: impl Into<String>) {
        if let Some(tooltip) = self.tooltips.get_mut(id) {
            tooltip.text(text.into());
            tooltip.show_at(ui, pos);
            self.active_tooltip = Some(id.to_string());
        }
    }

    pub fn show_tooltip_with_title(&mut self, id: &str, ui: &mut egui::Ui, pos: egui::Pos2, title: impl Into<String>, text: impl Into<String>) {
        if let Some(tooltip) = self.tooltips.get_mut(id) {
            tooltip.title(title);
            tooltip.text(text.into());
            tooltip.show_at(ui, pos);
            self.active_tooltip = Some(id.to_string());
        }
    }

    pub fn hide_tooltip(&mut self, id: &str) {
        if let Some(tooltip) = self.tooltips.get_mut(id) {
            tooltip.hide();
        }
        
        if self.active_tooltip.as_ref().map_or(false, |active| active == id) {
            self.active_tooltip = None;
        }
    }

    pub fn hide_all(&mut self) {
        for tooltip in self.tooltips.values_mut() {
            tooltip.hide();
        }
        self.active_tooltip = None;
    }

    pub fn get_tooltip(&self, id: &str) -> Option<&Tooltip> {
        self.tooltips.get(id)
    }

    pub fn get_tooltip_mut(&mut self, id: &str) -> Option<&mut Tooltip> {
        self.tooltips.get_mut(id)
    }

    pub fn remove_tooltip(&mut self, id: &str) -> Option<Tooltip> {
        if let Some(tooltip) = self.tooltips.remove(id) {
            if self.active_tooltip.as_ref().map_or(false, |active| active == id) {
                self.active_tooltip = None;
            }
            Some(tooltip)
        } else {
            None
        }
    }

    pub fn get_active_tooltip(&self) -> Option<&str> {
        self.active_tooltip.as_deref()
    }

    pub fn set_default_delay(&mut self, delay: f32) {
        self.default_delay = delay;
        
        for tooltip in self.tooltips.values_mut() {
            tooltip.delay(delay);
        }
    }

    pub fn set_default_duration(&mut self, duration: f32) {
        self.default_duration = duration;
        
        for tooltip in self.tooltips.values_mut() {
            tooltip.duration(duration);
        }
    }
}

impl Default for TooltipManager {
    fn default() -> Self {
        Self::new()
    }
}
