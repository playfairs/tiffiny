use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EffectsPanel {
  pub available_effects: Vec<Effect>,
  pub applied_effects: Vec<AppliedEffect>,
  pub selected_effect: Option<String>,
  pub search_query: String,
  pub category_filter: Option<EffectCategory>,
  pub show_presets: bool,
  pub preview_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct Effect {
  pub id: String,
  pub name: String,
  pub description: String,
  pub category: EffectCategory,
  pub parameters: Vec<EffectParameter>,
  pub presets: Vec<EffectPreset>,
  pub icon: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectCategory {
  Audio,
  Image,
  Video,
  DataBend,
  Glitch,
  Analog,
  Color,
  Filter,
  Transform,
}

#[derive(Debug, Clone)]
pub struct EffectParameter {
  pub id: String,
  pub name: String,
  pub parameter_type: ParameterType,
  pub default_value: serde_json::Value,
  pub current_value: serde_json::Value,
  pub min_value: Option<f32>,
  pub max_value: Option<f32>,
  pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
  Float,
  Integer,
  Boolean,
  Color,
  Choice(Vec<String>),
  Text,
  FilePath,
}

#[derive(Debug, Clone)]
pub struct EffectPreset {
  pub id: String,
  pub name: String,
  pub description: String,
  pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct AppliedEffect {
  pub effect_id: String,
  pub instance_id: String,
  pub name: String,
  pub enabled: bool,
  pub parameters: std::collections::HashMap<String, serde_json::Value>,
  pub order: usize,
}

impl EffectsPanel {
  pub fn new() -> Self {
    let mut panel = Self {
      available_effects: Vec::new(),
      applied_effects: Vec::new(),
      selected_effect: None,
      search_query: String::new(),
      category_filter: None,
      show_presets: true,
      preview_enabled: true,
    };

    panel.load_default_effects();
    panel
  }

  pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  fn load_default_effects(&mut self) {
    self.available_effects = vec![
      Effect {
        id: "databend_xor".to_string(),
        name: "XOR DataBend".to_string(),
        description: "Applies XOR operation to raw data for databending effects".to_string(),
        category: EffectCategory::DataBend,
        parameters: vec![
          EffectParameter {
            id: "xor_value".to_string(),
            name: "XOR Value".to_string(),
            parameter_type: ParameterType::Integer,
            default_value: serde_json::Value::Number(serde_json::Number::from(128)),
            current_value: serde_json::Value::Number(serde_json::Number::from(128)),
            min_value: Some(0.0),
            max_value: Some(255.0),
            description: "XOR value to apply (0-255)".to_string(),
          },
          EffectParameter {
            id: "apply_to_header".to_string(),
            name: "Apply to Header".to_string(),
            parameter_type: ParameterType::Boolean,
            default_value: serde_json::Value::Bool(false),
            current_value: serde_json::Value::Bool(false),
            min_value: None,
            max_value: None,
            description: "Apply XOR to file header as well".to_string(),
          },
        ],
        presets: vec![
          EffectPreset {
            id: "subtle".to_string(),
            name: "Subtle".to_string(),
            description: "Light databending effect".to_string(),
            parameters: {
              let mut params = std::collections::HashMap::new();
              params.insert(
                "xor_value".to_string(),
                serde_json::Value::Number(serde_json::Number::from(64)),
              );
              params.insert(
                "apply_to_header".to_string(),
                serde_json::Value::Bool(false),
              );
              params
            },
          },
          EffectPreset {
            id: "heavy".to_string(),
            name: "Heavy".to_string(),
            description: "Intense databending effect".to_string(),
            parameters: {
              let mut params = std::collections::HashMap::new();
              params.insert(
                "xor_value".to_string(),
                serde_json::Value::Number(serde_json::Number::from(200)),
              );
              params.insert("apply_to_header".to_string(), serde_json::Value::Bool(true));
              params
            },
          },
        ],
        icon: "🔀".to_string(),
      },
      Effect {
        id: "glitch_datamosh".to_string(),
        name: "Datamosh Glitch".to_string(),
        description: "Creates datamoshing artifacts by manipulating video compression".to_string(),
        category: EffectCategory::Glitch,
        parameters: vec![
          EffectParameter {
            id: "intensity".to_string(),
            name: "Intensity".to_string(),
            parameter_type: ParameterType::Float,
            default_value: serde_json::Value::Number(serde_json::Number::from_f64(0.5)),
            current_value: serde_json::Value::Number(serde_json::Number::from_f64(0.5)),
            min_value: Some(0.0),
            max_value: Some(1.0),
            description: "Glitch intensity (0.0-1.0)".to_string(),
          },
          EffectParameter {
            id: "frame_skip".to_string(),
            name: "Frame Skip".to_string(),
            parameter_type: ParameterType::Integer,
            default_value: serde_json::Value::Number(serde_json::Number::from(5)),
            current_value: serde_json::Value::Number(serde_json::Number::from(5)),
            min_value: Some(1.0),
            max_value: Some(30.0),
            description: "Number of frames to skip between glitches".to_string(),
          },
        ],
        presets: vec![EffectPreset {
          id: "subtle_glitch".to_string(),
          name: "Subtle".to_string(),
          description: "Light glitching effect".to_string(),
          parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
              "intensity".to_string(),
              serde_json::Value::Number(serde_json::Number::from_f64(0.2)),
            );
            params.insert(
              "frame_skip".to_string(),
              serde_json::Value::Number(serde_json::Number::from(10)),
            );
            params
          },
        }],
        icon: "📺".to_string(),
      },
      Effect {
        id: "vhs_effect".to_string(),
        name: "VHS Effect".to_string(),
        description: "Simulates VHS tape degradation and artifacts".to_string(),
        category: EffectCategory::Analog,
        parameters: vec![
          EffectParameter {
            id: "noise_level".to_string(),
            name: "Noise Level".to_string(),
            parameter_type: ParameterType::Float,
            default_value: serde_json::Value::Number(serde_json::Number::from_f64(0.3)),
            current_value: serde_json::Value::Number(serde_json::Number::from_f64(0.3)),
            min_value: Some(0.0),
            max_value: Some(1.0),
            description: "Amount of noise to add".to_string(),
          },
          EffectParameter {
            id: "chromatic_aberration".to_string(),
            name: "Chromatic Aberration".to_string(),
            parameter_type: ParameterType::Float,
            default_value: serde_json::Value::Number(serde_json::Number::from_f64(0.1)),
            current_value: serde_json::Value::Number(serde_json::Number::from_f64(0.1)),
            min_value: Some(0.0),
            max_value: Some(1.0),
            description: "Chromatic aberration intensity".to_string(),
          },
        ],
        presets: vec![EffectPreset {
          id: "vhs_clean".to_string(),
          name: "Clean".to_string(),
          description: "Clean VHS look with minimal artifacts".to_string(),
          parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
              "noise_level".to_string(),
              serde_json::Value::Number(serde_json::Number::from_f64(0.1)),
            );
            params.insert(
              "chromatic_aberration".to_string(),
              serde_json::Value::Number(serde_json::Number::from_f64(0.05)),
            );
            params
          },
        }],
        icon: "📼".to_string(),
      },
      Effect {
        id: "crt_effect".to_string(),
        name: "CRT Effect".to_string(),
        description: "Simulates old CRT monitor display characteristics".to_string(),
        category: EffectCategory::Analog,
        parameters: vec![
          EffectParameter {
            id: "scanlines".to_string(),
            name: "Scanlines".to_string(),
            parameter_type: ParameterType::Boolean,
            default_value: serde_json::Value::Bool(true),
            current_value: serde_json::Value::Bool(true),
            min_value: None,
            max_value: None,
            description: "Enable scanline effect".to_string(),
          },
          EffectParameter {
            id: "curvature".to_string(),
            name: "Screen Curvature".to_string(),
            parameter_type: ParameterType::Float,
            default_value: serde_json::Value::Number(serde_json::Number::from_f64(0.2)),
            current_value: serde_json::Value::Number(serde_json::Number::from_f64(0.2)),
            min_value: Some(0.0),
            max_value: Some(1.0),
            description: "Screen curvature amount".to_string(),
          },
        ],
        presets: vec![EffectPreset {
          id: "crt_arcade".to_string(),
          name: "Arcade".to_string(),
          description: "Classic arcade CRT look".to_string(),
          parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert("scanlines".to_string(), serde_json::Value::Bool(true));
            params.insert(
              "curvature".to_string(),
              serde_json::Value::Number(serde_json::Number::from_f64(0.4)),
            );
            params
          },
        }],
        icon: "🖥".to_string(),
      },
      Effect {
        id: "audio_to_image".to_string(),
        name: "Audio to Image".to_string(),
        description: "Converts audio waveform data into visual image representation".to_string(),
        category: EffectCategory::Audio,
        parameters: vec![
          EffectParameter {
            id: "width".to_string(),
            name: "Width".to_string(),
            parameter_type: ParameterType::Integer,
            default_value: serde_json::Value::Number(serde_json::Number::from(1920)),
            current_value: serde_json::Value::Number(serde_json::Number::from(1920)),
            min_value: Some(100.0),
            max_value: Some(8192.0),
            description: "Output image width".to_string(),
          },
          EffectParameter {
            id: "height".to_string(),
            name: "Height".to_string(),
            parameter_type: ParameterType::Integer,
            default_value: serde_json::Value::Number(serde_json::Number::from(1080)),
            current_value: serde_json::Value::Number(serde_json::Number::from(1080)),
            min_value: Some(100.0),
            max_value: Some(8192.0),
            description: "Output image height".to_string(),
          },
          EffectParameter {
            id: "color_mode".to_string(),
            name: "Color Mode".to_string(),
            parameter_type: ParameterType::Choice(vec![
              "Grayscale".to_string(),
              "RGB".to_string(),
              "HSV".to_string(),
            ]),
            default_value: serde_json::Value::String("RGB".to_string()),
            current_value: serde_json::Value::String("RGB".to_string()),
            min_value: None,
            max_value: None,
            description: "Color representation mode".to_string(),
          },
        ],
        presets: vec![EffectPreset {
          id: "waveform_classic".to_string(),
          name: "Classic Waveform".to_string(),
          description: "Traditional oscilloscope-style waveform".to_string(),
          parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
              "width".to_string(),
              serde_json::Value::Number(serde_json::Number::from(1920)),
            );
            params.insert(
              "height".to_string(),
              serde_json::Value::Number(serde_json::Number::from(1080)),
            );
            params.insert(
              "color_mode".to_string(),
              serde_json::Value::String("Grayscale".to_string()),
            );
            params
          },
        }],
        icon: "🎵".to_string(),
      },
    ];
  }

  pub fn apply_effect(&mut self, effect_id: &str) -> Result<String, String> {
    let effect = self
      .available_effects
      .iter()
      .find(|e| e.id == effect_id)
      .ok_or_else(|| "Effect not found".to_string())?;

    let instance_id = uuid::Uuid::new_v4().to_string();
    let applied_effect = AppliedEffect {
      effect_id: effect.id.clone(),
      instance_id: instance_id.clone(),
      name: effect.name.clone(),
      enabled: true,
      parameters: effect
        .parameters
        .iter()
        .map(|p| (p.id.clone(), p.default_value.clone()))
        .collect(),
      order: self.applied_effects.len(),
    };

    self.applied_effects.push(applied_effect);
    Ok(instance_id)
  }

  pub fn remove_effect(&mut self, instance_id: &str) -> bool {
    let index = self
      .applied_effects
      .iter()
      .position(|e| e.instance_id == instance_id);
    if let Some(index) = index {
      self.applied_effects.remove(index);
      true
    } else {
      false
    }
  }

  pub fn toggle_effect(&mut self, instance_id: &str) -> bool {
    if let Some(effect) = self
      .applied_effects
      .iter_mut()
      .find(|e| e.instance_id == instance_id)
    {
      effect.enabled = !effect.enabled;
      true
    } else {
      false
    }
  }

  pub fn reorder_effects(&mut self, instance_id: &str, new_order: usize) -> bool {
    let index = self
      .applied_effects
      .iter()
      .position(|e| e.instance_id == instance_id);
    if let Some(index) = index {
      if let Some(effect) = self.applied_effects.get(index) {
        let effect = effect.clone();
        self.applied_effects.remove(index);
        self
          .applied_effects
          .insert(new_order.min(self.applied_effects.len()), effect);
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  pub fn update_effect_parameter(
    &mut self,
    instance_id: &str,
    parameter_id: &str,
    value: serde_json::Value,
  ) -> bool {
    if let Some(effect) = self
      .applied_effects
      .iter_mut()
      .find(|e| e.instance_id == instance_id)
    {
      effect.parameters.insert(parameter_id.to_string(), value);
      true
    } else {
      false
    }
  }

  pub fn get_effect(&self, effect_id: &str) -> Option<&Effect> {
    self.available_effects.iter().find(|e| e.id == effect_id)
  }

  pub fn get_applied_effect(&self, instance_id: &str) -> Option<&AppliedEffect> {
    self
      .applied_effects
      .iter()
      .find(|e| e.instance_id == instance_id)
  }

  pub fn set_search_query(&mut self, query: String) {
    self.search_query = query;
  }

  pub fn set_category_filter(&mut self, category: Option<EffectCategory>) {
    self.category_filter = category;
  }

  pub fn toggle_presets(&mut self) {
    self.show_presets = !self.show_presets;
  }

  pub fn toggle_preview(&mut self) {
    self.preview_enabled = !self.preview_enabled;
  }

  pub fn clear_all_effects(&mut self) {
    self.applied_effects.clear();
  }

  fn get_filtered_effects(&self) -> Vec<&Effect> {
    self
      .available_effects
      .iter()
      .filter(|effect| {
        if !self.search_query.is_empty()
          && !effect
            .name
            .to_lowercase()
            .contains(&self.search_query.to_lowercase())
          && !effect
            .description
            .to_lowercase()
            .contains(&self.search_query.to_lowercase())
        {
          return false;
        }

        if let Some(filter_category) = &self.category_filter {
          &effect.category != filter_category
        } else {
          false
        }
      })
      .collect()
  }

  pub fn render(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search effects..."));

      ui.add_space(10.0);

      egui::ComboBox::from_label("Category")
        .selected_text(format!(
          "{:?}",
          self.category_filter.unwrap_or(EffectCategory::Audio)
        ))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.category_filter, None, "All");
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Audio),
            "Audio",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Image),
            "Image",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Video),
            "Video",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::DataBend),
            "DataBend",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Glitch),
            "Glitch",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Analog),
            "Analog",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Color),
            "Color",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Filter),
            "Filter",
          );
          ui.selectable_value(
            &mut self.category_filter,
            Some(EffectCategory::Transform),
            "Transform",
          );
        });

      ui.add_space(10.0);

      ui.checkbox(&mut self.show_presets, "Presets");
      ui.checkbox(&mut self.preview_enabled, "Preview");

      if ui.button("Clear All").clicked() {
        self.clear_all_effects();
      }
    });

    ui.separator();

    egui::Splitter::horizontal(&mut 200.0).show(
      ui,
      |ui| {
        self.render_available_effects(ui);
      },
      |ui| {
        self.render_applied_effects(ui);
      },
    );
  }

  fn render_available_effects(&mut self, ui: &mut egui::Ui) {
    ui.heading("Available Effects");
    ui.separator();

    egui::ScrollArea::vertical()
      .auto_shrink([false, false])
      .show(ui, |ui| {
        let filtered_effects = self.get_filtered_effects();

        for effect in filtered_effects {
          self.render_effect_item(ui, effect);
        }
      });
  }

  fn render_effect_item(&mut self, ui: &mut egui::Ui, effect: &Effect) {
    let is_selected = self
      .selected_effect
      .as_ref()
      .map_or(false, |id| id == &effect.id);

    let (rect, _) =
      ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::click());

    let painter = ui.painter();
    let bg_color = if is_selected {
      egui::Color32::from_rgb(60, 90, 120)
    } else if ui.rect_contains_pointer(rect) {
      egui::Color32::from_rgb(50, 50, 50)
    } else {
      egui::Color32::from_rgb(40, 40, 40)
    };

    painter.rect_filled(rect, 4.0, bg_color);
    painter.rect_stroke(
      rect,
      4.0,
      egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
    );

    ui.horizontal(|ui| {
      ui.label(&effect.icon);
      ui.vertical(|ui| {
        ui.label(&effect.name);
        ui.label(
          egui::RichText::new(&effect.description)
            .size(10.0)
            .color(egui::Color32::from_rgb(160, 160, 160)),
        );
      });
    });

    if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
      self.selected_effect = Some(effect.id.clone());
    }

    if ui.button("Apply").clicked() {
      if let Ok(instance_id) = self.apply_effect(&effect.id) {
        tracing::info!(
          "Applied effect: {} with instance ID: {}",
          effect.name,
          instance_id
        );
      }
    }

    if self.show_presets && !effect.presets.is_empty() {
      ui.separator();
      ui.label("Presets:");
      for preset in &effect.presets {
        if ui.button(&preset.name).clicked() {
          tracing::info!(
            "Applied preset: {} for effect: {}",
            preset.name,
            effect.name
          );
        }
      }
    }
  }

  fn render_applied_effects(&mut self, ui: &mut egui::Ui) {
    ui.heading("Applied Effects");
    ui.separator();

    egui::ScrollArea::vertical()
      .auto_shrink([false, false])
      .show(ui, |ui| {
        for (index, effect) in self.applied_effects.iter_mut().enumerate() {
          self.render_applied_effect_item(ui, effect, index);
        }
      });
  }

  fn render_applied_effect_item(
    &mut self,
    ui: &mut egui::Ui,
    effect: &mut AppliedEffect,
    index: usize,
  ) {
    egui::CollapsingHeader::new(&effect.name)
      .default_open(true)
      .show(ui, |ui| {
        ui.horizontal(|ui| {
          ui.checkbox(&mut effect.enabled, "Enabled");

          if ui.button("Remove").clicked() {
            self.remove_effect(&effect.instance_id);
          }

          if ui.button("↑").clicked() && index > 0 {
            self.reorder_effects(&effect.instance_id, index - 1);
          }

          if ui.button("↓").clicked() && index < self.applied_effects.len() - 1 {
            self.reorder_effects(&effect.instance_id, index + 1);
          }
        });

        if let Some(original_effect) = self.get_effect(&effect.effect_id) {
          self.render_effect_parameters(ui, original_effect, effect);
        }
      });
  }

  fn render_effect_parameters(
    &mut self,
    ui: &mut egui::Ui,
    original_effect: &Effect,
    applied_effect: &mut AppliedEffect,
  ) {
    for parameter in &original_effect.parameters {
      let current_value = applied_effect
        .parameters
        .get(&parameter.id)
        .cloned()
        .unwrap_or(parameter.default_value.clone());

      ui.horizontal(|ui| {
        ui.label(&parameter.name);
        ui.add_space(10.0);

        let mut new_value = current_value.clone();
        let mut changed = false;

        match &parameter.parameter_type {
          ParameterType::Float => {
            if let Some(min) = parameter.min_value {
              if let Some(max) = parameter.max_value {
                if let Some(value) = new_value.as_f64() {
                  let mut float_val = value as f32;
                  if ui
                    .add(egui::Slider::new(&mut float_val, min..=max))
                    .changed()
                  {
                    new_value =
                      serde_json::Value::Number(serde_json::Number::from_f64(float_val as f64));
                    changed = true;
                  }
                }
              }
            }
          }
          ParameterType::Integer => {
            if let Some(min) = parameter.min_value {
              if let Some(max) = parameter.max_value {
                if let Some(value) = new_value.as_i64() {
                  let mut int_val = value as i32;
                  if ui
                    .add(egui::Slider::new(&mut int_val, min as i32..=max as i32))
                    .changed()
                  {
                    new_value = serde_json::Value::Number(serde_json::Number::from(int_val));
                    changed = true;
                  }
                }
              }
            }
          }
          ParameterType::Boolean => {
            if let Some(value) = new_value.as_bool() {
              let mut bool_val = *value;
              if ui.checkbox(&mut bool_val, "").changed() {
                new_value = serde_json::Value::Bool(bool_val);
                changed = true;
              }
            }
          }
          ParameterType::Choice(choices) => {
            if let Some(current_choice) = new_value.as_str() {
              let mut selected_choice = current_choice.to_string();
              if egui::ComboBox::from_label("")
                .selected_text(&selected_choice)
                .show_ui(ui, |ui| {
                  for choice in choices {
                    ui.selectable_value(&mut selected_choice, choice.clone(), choice);
                  }
                })
                .changed()
              {
                new_value = serde_json::Value::String(selected_choice);
                changed = true;
              }
            }
          }
          _ => {
            ui.label(format!("{:?}", new_value));
          }
        }

        if changed {
          applied_effect
            .parameters
            .insert(parameter.id.clone(), new_value);
          if self.preview_enabled {
            tracing::info!("Preview updated for effect: {}", applied_effect.name);
          }
        }
      });

      if !parameter.description.is_empty() {
        ui.label("ℹ");
        if ui.is_item_hovered() {
          egui::show_tooltip_at_pointer(ui.ctx(), || egui::Label::new(&parameter.description));
        }
      }
    }
  }
}

impl Default for EffectsPanel {
  fn default() -> Self {
    Self::new()
  }
}
