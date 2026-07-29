use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Slider {
  pub id: String,
  pub label: String,
  pub value: f32,
  pub min: f32,
  pub max: f32,
  pub step: Option<f32>,
  pub orientation: SliderOrientation,
  pub show_value: bool,
  pub show_ticks: bool,
  pub enabled: bool,
  pub visible: bool,
  pub on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SliderOrientation {
  Horizontal,
  Vertical,
}

impl Slider {
  pub fn new(id: String) -> Self {
    Self {
      id,
      label: String::new(),
      value: 0.0,
      min: 0.0,
      max: 1.0,
      step: None,
      orientation: SliderOrientation::Horizontal,
      show_value: true,
      show_ticks: false,
      enabled: true,
      visible: true,
      on_change: None,
    }
  }

  pub fn label(mut self, label: impl Into<String>) -> Self {
    self.label = label.into();
    self
  }

  pub fn range(mut self, min: f32, max: f32) -> Self {
    self.min = min;
    self.max = max;
    self.value = self.value.clamp(min, max);
    self
  }

  pub fn value(mut self, value: f32) -> Self {
    self.value = value.clamp(self.min, self.max);
    self
  }

  pub fn step(mut self, step: f32) -> Self {
    self.step = Some(step);
    self
  }

  pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
    self.orientation = orientation;
    self
  }

  pub fn show_value(mut self, show: bool) -> Self {
    self.show_value = show;
    self
  }

  pub fn show_ticks(mut self, show: bool) -> Self {
    self.show_ticks = show;
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

  pub fn on_change(mut self, callback: impl Fn(f32) + Send + Sync + 'static) -> Self {
    self.on_change = Some(Arc::new(callback));
    self
  }

  pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
    if !self.visible {
      return false;
    }

    let mut changed = false;

    match self.orientation {
      SliderOrientation::Horizontal => {
        ui.horizontal(|ui| {
          if !self.label.is_empty() {
            ui.label(&self.label);
            ui.add_space(10.0);
          }

          let slider_response = if let Some(step) = self.step {
            let mut value = self.value;
            let response = ui.add(
              egui::Slider::new(&mut value, self.min..=self.max)
                .step_by(step)
                .show_value(self.show_value),
            );
            changed = response.changed();
            response
          } else {
            let mut value = self.value;
            let response = ui
              .add(egui::Slider::new(&mut value, self.min..=self.max).show_value(self.show_value));
            changed = response.changed();
            response
          };

          if changed {
            self.value = self.value.clamp(self.min, self.max);
            if let Some(callback) = &self.on_change {
              callback(self.value);
            }
          }

          if self.show_ticks {
            self.render_ticks(ui);
          }
        });
      }
      SliderOrientation::Vertical => {
        ui.vertical(|ui| {
          if !self.label.is_empty() {
            ui.label(&self.label);
            ui.add_space(5.0);
          }

          let slider_response = if let Some(step) = self.step {
            let mut value = self.value;
            let response = ui.add(
              egui::Slider::new(&mut value, self.min..=self.max)
                .step_by(step)
                .show_value(self.show_value)
                .vertical(),
            );
            changed = response.changed();
            response
          } else {
            let mut value = self.value;
            let response = ui.add(
              egui::Slider::new(&mut value, self.min..=self.max)
                .show_value(self.show_value)
                .vertical(),
            );
            changed = response.changed();
            response
          };

          if changed {
            self.value = self.value.clamp(self.min, self.max);
            if let Some(callback) = &self.on_change {
              callback(self.value);
            }
          }

          if self.show_ticks {
            self.render_ticks(ui);
          }
        });
      }
    }

    changed
  }

  fn render_ticks(&self, ui: &mut egui::Ui) {
    let tick_count = 5;
    let range = self.max - self.min;
    let tick_spacing = range / (tick_count - 1) as f32;

    ui.horizontal(|ui| {
      for i in 0..tick_count {
        let tick_value = self.min + (i as f32 * tick_spacing);
        ui.label(format!("{:.1}", tick_value));
        if i < tick_count - 1 {
          ui.add_space(10.0);
        }
      }
    });
  }

  pub fn get_value(&self) -> f32 {
    self.value
  }

  pub fn set_value(&mut self, value: f32) {
    self.value = value.clamp(self.min, self.max);
  }
}

impl Default for Slider {
  fn default() -> Self {
    Self::new("default_slider".to_string())
  }
}

pub struct RangeSlider {
  pub id: String,
  pub label: String,
  pub start: f32,
  pub end: f32,
  pub min: f32,
  pub max: f32,
  pub step: Option<f32>,
  pub show_values: bool,
  pub enabled: bool,
  pub visible: bool,
  pub on_change: Option<Arc<dyn Fn(f32, f32) + Send + Sync>>,
}

impl RangeSlider {
  pub fn new(id: String) -> Self {
    Self {
      id,
      label: String::new(),
      start: 0.0,
      end: 1.0,
      min: 0.0,
      max: 1.0,
      step: None,
      show_values: true,
      enabled: true,
      visible: true,
      on_change: None,
    }
  }

  pub fn label(mut self, label: impl Into<String>) -> Self {
    self.label = label.into();
    self
  }

  pub fn range(mut self, min: f32, max: f32) -> Self {
    self.min = min;
    self.max = max;
    self.start = self.start.clamp(min, max);
    self.end = self.end.clamp(min, max);
    self
  }

  pub fn values(mut self, start: f32, end: f32) -> Self {
    self.start = start.clamp(self.min, self.max);
    self.end = end.clamp(self.min, self.max);
    self
  }

  pub fn step(mut self, step: f32) -> Self {
    self.step = Some(step);
    self
  }

  pub fn show_values(mut self, show: bool) -> Self {
    self.show_values = show;
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

  pub fn on_change(mut self, callback: impl Fn(f32, f32) + Send + Sync + 'static) -> Self {
    self.on_change = Some(Arc::new(callback));
    self
  }

  pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
    if !self.visible {
      return false;
    }

    let mut changed = false;

    ui.horizontal(|ui| {
      if !self.label.is_empty() {
        ui.label(&self.label);
        ui.add_space(10.0);
      }

      let mut start = self.start;
      let mut end = self.end;

      let response = if let Some(step) = self.step {
        ui.add(
          egui::Slider::new(&mut start, self.min..=self.end)
            .step_by(step)
            .show_value(self.show_values)
            .text("Start"),
        );
      } else {
        ui.add(
          egui::Slider::new(&mut start, self.min..=self.end)
            .show_value(self.show_values)
            .text("Start"),
        );
      };

      if response.changed() {
        changed = true;
        self.start = start.clamp(self.min, self.max);
      }

      ui.add_space(10.0);

      let response = if let Some(step) = self.step {
        ui.add(
          egui::Slider::new(&mut end, self.start..=self.max)
            .step_by(step)
            .show_value(self.show_values)
            .text("End"),
        );
      } else {
        ui.add(
          egui::Slider::new(&mut end, self.start..=self.max)
            .show_value(self.show_values)
            .text("End"),
        );
      };

      if response.changed() {
        changed = true;
        self.end = end.clamp(self.start, self.max);
      }

      if changed {
        if let Some(callback) = &self.on_change {
          callback(self.start, self.end);
        }
      }
    });

    changed
  }

  pub fn get_values(&self) -> (f32, f32) {
    (self.start, self.end)
  }

  pub fn set_values(&mut self, start: f32, end: f32) {
    self.start = start.clamp(self.min, self.max);
    self.end = end.clamp(self.start, self.max);
  }
}

impl Default for RangeSlider {
  fn default() -> Self {
    Self::new("default_range_slider".to_string())
  }
}
