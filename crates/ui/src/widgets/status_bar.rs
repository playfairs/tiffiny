use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct StatusBar {
    pub id: String,
    pub sections: Vec<StatusSection>,
    pub visible: bool,
    pub enabled: bool,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct StatusSection {
    pub id: String,
    pub label: String,
    pub value: String,
    pub tooltip: Option<String>,
    pub alignment: StatusAlignment,
    pub width: Option<f32>,
    pub color: Option<egui::Color32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusAlignment {
    Left,
    Center,
    Right,
}

impl StatusBar {
    pub fn new(id: String) -> Self {
        Self {
            id,
            sections: Vec::new(),
            visible: true,
            enabled: true,
            height: 24.0,
        }
    }

    pub fn add_section(mut self, section: StatusSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn add_sections(mut self, sections: Vec<StatusSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    pub fn clear_sections(&mut self) -> Self {
        self.sections.clear();
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if !self.visible {
            return;
        }

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), self.height),
            egui::Sense::hover()
        );

        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color));

        let mut left_x = rect.min.x + 8.0;
        let center_y = rect.center().y;

        for section in &self.sections {
            let section_width = section.width.unwrap_or(200.0);
            let text_color = section.color.unwrap_or(ui.visuals().text_color());

            match section.alignment {
                StatusAlignment::Left => {
                    let text_pos = egui::pos2(left_x, center_y);
                    painter.text(
                        text_pos,
                        egui::Align2::LEFT_CENTER,
                        &section.label,
                        egui::FontId::default(),
                        text_color
                    );

                    let value_pos = egui::pos2(left_x + 100.0, center_y);
                    painter.text(
                        value_pos,
                        egui::Align2::LEFT_CENTER,
                        &section.value,
                        egui::FontId::default(),
                        text_color
                    );

                    left_x += section_width + 16.0;
                },
                StatusAlignment::Center => {
                    let text_pos = egui::pos2(left_x + section_width / 2.0, center_y);
                    painter.text(
                        text_pos,
                        egui::Align2::CENTER_CENTER,
                        &section.label,
                        egui::FontId::default(),
                        text_color
                    );

                    left_x += section_width + 16.0;
                },
                StatusAlignment::Right => {
                    let text_pos = egui::pos2(left_x + section_width - 8.0, center_y);
                    painter.text(
                        text_pos,
                        egui::Align2::RIGHT_CENTER,
                        &section.label,
                        egui::FontId::default(),
                        text_color
                    );

                    left_x += section_width + 16.0;
                },
            }

Handle tooltip
            if let Some(tooltip) = &section.tooltip {
                let tooltip_rect = egui::Rect::from_min_size(
                    egui::pos2(left_x - 8.0, rect.min.y),
                    egui::vec2(section_width + 16.0, self.height)
                );

                if ui.rect_contains_pointer(tooltip_rect) {
                    egui::show_tooltip_at_pointer(ui.ctx(), || {
                        egui::Label::new(tooltip)
                    });
                }
            }
        }
    }

    pub fn get_section(&self, section_id: &str) -> Option<&StatusSection> {
        self.sections.iter().find(|section| section.id == section_id)
    }

    pub fn get_section_mut(&mut self, section_id: &str) -> Option<&mut StatusSection> {
        self.sections.iter_mut().find(|section| section.id == section_id)
    }

    pub fn update_section_value(&mut self, section_id: &str, value: String) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.value = value;
            true
        } else {
            false
        }
    }

    pub fn update_section_label(&mut self, section_id: &str, label: String) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.label = label;
            true
        } else {
            false
        }
    }

    pub fn remove_section(&mut self, section_id: &str) -> Option<StatusSection> {
        let index = self.sections.iter().position(|section| section.id == section_id);
        if let Some(index) = index {
            Some(self.sections.remove(index))
        } else {
            None
        }
    }

    pub fn set_section_color(&mut self, section_id: &str, color: egui::Color32) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.color = Some(color);
            true
        } else {
            false
        }
    }

    pub fn set_section_tooltip(&mut self, section_id: &str, tooltip: Option<String>) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.tooltip = tooltip;
            true
        } else {
            false
        }
    }

    pub fn set_section_width(&mut self, section_id: &str, width: f32) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.width = Some(width);
            true
        } else {
            false
        }
    }

    pub fn get_section_count(&self) -> usize {
        self.sections.len()
    }

    pub fn get_visible_sections(&self) -> Vec<&StatusSection> {
        self.sections.iter().collect()
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new("default_status_bar".to_string())
    }
}

impl StatusSection {
    pub fn new(id: String, label: String, value: String) -> Self {
        Self {
            id,
            label,
            value,
            tooltip: None,
            alignment: StatusAlignment::Left,
            width: None,
            color: None,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn alignment(mut self, alignment: StatusAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn left(id: String, label: String, value: String) -> Self {
        Self::new(id, label, value).alignment(StatusAlignment::Left)
    }

    pub fn center(id: String, label: String, value: String) -> Self {
        Self::new(id, label, value).alignment(StatusAlignment::Center)
    }

    pub fn right(id: String, label: String, value: String) -> Self {
        Self::new(id, label, value).alignment(StatusAlignment::Right)
    }
}

impl Default for StatusSection {
    fn default() -> Self {
        Self::new("default_section".to_string(), "Status".to_string(), "Ready".to_string())
    }
}

pub struct StatusBarBuilder {
    status_bar: StatusBar,
}

impl StatusBarBuilder {
    pub fn new() -> Self {
        Self {
            status_bar: StatusBar::new("builder_status_bar".to_string()),
        }
    }

    pub fn add_section(mut self, section: StatusSection) -> Self {
        self.status_bar.add_section(section);
        self
    }

    pub fn add_left_section(mut self, id: String, label: String, value: String) -> Self {
        self.status_bar.add_section(StatusSection::left(id, label, value));
        self
    }

    pub fn add_center_section(mut self, id: String, label: String, value: String) -> Self {
        self.status_bar.add_section(StatusSection::center(id, label, value));
        self
    }

    pub fn add_right_section(mut self, id: String, label: String, value: String) -> Self {
        self.status_bar.add_section(StatusSection::right(id, label, value));
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.status_bar.height(height);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.status_bar.visible(visible);
        self
    }

    pub fn build(self) -> StatusBar {
        self.status_bar
    }
}

impl Default for StatusBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}
