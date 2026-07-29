use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Button {
  pub id: String,
  pub text: String,
  pub icon: Option<String>,
  pub style: ButtonStyle,
  pub size: ButtonSize,
  pub enabled: bool,
  pub visible: bool,
  pub tooltip: Option<String>,
  pub shortcut: Option<String>,
  pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonStyle {
  Primary,
  Secondary,
  Success,
  Warning,
  Error,
  Ghost,
  Link,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonSize {
  Small,
  Medium,
  Large,
  Custom(f32, f32),
}

impl Button {
  pub fn new(id: String) -> Self {
    Self {
      id,
      text: String::new(),
      icon: None,
      style: ButtonStyle::Primary,
      size: ButtonSize::Medium,
      enabled: true,
      visible: true,
      tooltip: None,
      shortcut: None,
      on_click: None,
    }
  }

  pub fn text(mut self, text: impl Into<String>) -> Self {
    self.text = text.into();
    self
  }

  pub fn icon(mut self, icon: impl Into<String>) -> Self {
    self.icon = Some(icon.into());
    self
  }

  pub fn style(mut self, style: ButtonStyle) -> Self {
    self.style = style;
    self
  }

  pub fn size(mut self, size: ButtonSize) -> Self {
    self.size = size;
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

  pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
    self.tooltip = Some(tooltip.into());
    self
  }

  pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
    self.shortcut = Some(shortcut.into());
    self
  }

  pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
    self.on_click = Some(Arc::new(callback));
    self
  }

  pub fn render(&self, ui: &mut egui::Ui) -> bool {
    if !self.visible {
      return false;
    }

    let button_color = self.get_button_color(ui);
    let text_color = self.get_text_color(ui);
    let (min_size, padding) = self.get_size_and_padding();

    let mut response = ui.add_sized(
      egui::Button::new(
        egui::RichText::new(self.get_display_text())
          .color(text_color)
          .size(self.get_font_size()),
      )
      .fill(button_color)
      .stroke(self.get_stroke(ui)),
      min_size,
    );

    if !self.enabled {
      response = response.on_hover_cursor(egui::CursorIcon::NotAllowed);
    }

    if let Some(tooltip) = &self.tooltip {
      response = response.on_hover_text(tooltip);
    }

    if let Some(shortcut) = &self.shortcut {
      response = response.on_hover_text(format!("Shortcut: {}", shortcut));
    }

    let clicked = response.clicked() && self.enabled;

    if clicked {
      if let Some(callback) = &self.on_click {
        callback();
      }
    }

    clicked
  }

  fn get_display_text(&self) -> String {
    match (&self.icon, &self.text) {
      (Some(icon), Some(text)) => format!("{} {}", icon, text),
      (Some(icon), None) => icon.clone(),
      (None, Some(text)) => text.clone(),
      (None, None) => String::new(),
    }
  }

  fn get_button_color(&self, ui: &egui::Ui) -> egui::Color32 {
    if !self.enabled {
      return ui.visuals().widgets.inactive.bg_fill;
    }

    match self.style {
      ButtonStyle::Primary => ui.visuals().widgets.active.bg_fill,
      ButtonStyle::Secondary => egui::Color32::from_rgb(88, 88, 88),
      ButtonStyle::Success => egui::Color32::from_rgb(46, 160, 67),
      ButtonStyle::Warning => egui::Color32::from_rgb(255, 193, 7),
      ButtonStyle::Error => egui::Color32::from_rgb(220, 53, 69),
      ButtonStyle::Ghost => egui::Color32::TRANSPARENT,
      ButtonStyle::Link => egui::Color32::TRANSPARENT,
    }
  }

  fn get_text_color(&self, ui: &egui::Ui) -> egui::Color32 {
    if !self.enabled {
      return ui.visuals().widgets.inactive.text_color();
    }

    match self.style {
      ButtonStyle::Primary => ui.visuals().widgets.active.text_color(),
      ButtonStyle::Secondary => egui::Color32::WHITE,
      ButtonStyle::Success => egui::Color32::WHITE,
      ButtonStyle::Warning => egui::Color32::BLACK,
      ButtonStyle::Error => egui::Color32::WHITE,
      ButtonStyle::Ghost => ui.visuals().widgets.active.text_color(),
      ButtonStyle::Link => egui::Color32::from_rgb(66, 135, 245),
    }
  }

  fn get_stroke(&self, ui: &egui::Ui) -> egui::Stroke {
    match self.style {
      ButtonStyle::Ghost => egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill),
      ButtonStyle::Link => egui::Stroke::new(1.0, egui::Color32::TRANSPARENT),
      _ => ui.visuals().widgets.active.bg_stroke,
    }
  }

  fn get_size_and_padding(&self) -> egui::Vec2 {
    match self.size {
      ButtonSize::Small => egui::vec2(60.0, 24.0),
      ButtonSize::Medium => egui::vec2(80.0, 32.0),
      ButtonSize::Large => egui::vec2(120.0, 40.0),
      ButtonSize::Custom(width, height) => egui::vec2(width, height),
    }
  }

  fn get_font_size(&self) -> f32 {
    match self.size {
      ButtonSize::Small => 12.0,
      ButtonSize::Medium => 14.0,
      ButtonSize::Large => 16.0,
      ButtonSize::Custom(..) => 14.0,
    }
  }
}

impl Default for Button {
  fn default() -> Self {
    Self::new("default_button".to_string())
  }
}

pub struct ButtonGroup {
  pub id: String,
  pub buttons: Vec<Button>,
  pub orientation: ButtonOrientation,
  pub spacing: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonOrientation {
  Horizontal,
  Vertical,
}

impl ButtonGroup {
  pub fn new(id: String) -> Self {
    Self {
      id,
      buttons: Vec::new(),
      orientation: ButtonOrientation::Horizontal,
      spacing: 8.0,
    }
  }

  pub fn add_button(mut self, button: Button) -> Self {
    self.buttons.push(button);
    self
  }

  pub fn orientation(mut self, orientation: ButtonOrientation) -> Self {
    self.orientation = orientation;
    self
  }

  pub fn spacing(mut self, spacing: f32) -> Self {
    self.spacing = spacing;
    self
  }

  pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<String> {
    let mut clicked_buttons = Vec::new();

    match self.orientation {
      ButtonOrientation::Horizontal => {
        ui.horizontal(|ui| {
          for button in &mut self.buttons {
            ui.add_space(self.spacing);
            if button.render(ui) {
              clicked_buttons.push(button.id.clone());
            }
          }
        });
      }
      ButtonOrientation::Vertical => {
        for button in &mut self.buttons {
          ui.add_space(self.spacing);
          if button.render(ui) {
            clicked_buttons.push(button.id.clone());
          }
        }
      }
    }

    clicked_buttons
  }
}

impl Default for ButtonGroup {
  fn default() -> Self {
    Self::new("default_group".to_string())
  }
}
