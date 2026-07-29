use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;
use tiffiny_core::prelude::*;

#[derive(Debug, Clone)]
pub struct InspectorPanel {
  pub selected_object: Option<InspectableObject>,
  pub properties: Vec<Property>,
  pub show_advanced: bool,
  pub search_query: String,
  pub category_filter: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InspectableObject {
  pub id: String,
  pub name: String,
  pub object_type: ObjectType,
  pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
  Asset,
  Clip,
  Effect,
  Track,
  Project,
  Pipeline,
  Graph,
  Node,
}

#[derive(Debug, Clone)]
pub struct Property {
  pub id: String,
  pub name: String,
  pub property_type: PropertyType,
  pub value: PropertyValue,
  pub category: String,
  pub description: String,
  pub read_only: bool,
  pub advanced: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
  String,
  Integer,
  Float,
  Boolean,
  Color,
  Vector2,
  Vector3,
  Vector4,
  FilePath,
  DirectoryPath,
  Choice(Vec<String>),
  Range(f32, f32),
  Text,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
  String(String),
  Integer(i64),
  Float(f32),
  Boolean(bool),
  Color(egui::Color32),
  Vector2(glam::Vec2),
  Vector3(glam::Vec3),
  Vector4(glam::Vec4),
  FilePath(String),
  DirectoryPath(String),
  Choice(String),
  Range(f32),
  Text(String),
}

impl InspectorPanel {
  pub fn new() -> Self {
    Self {
      selected_object: None,
      properties: Vec::new(),
      show_advanced: false,
      search_query: String::new(),
      category_filter: None,
    }
  }

  pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub fn set_selected_object(&mut self, object: Option<InspectableObject>) {
    self.selected_object = object;
    self.update_properties();
  }

  pub fn clear_selection(&mut self) {
    self.selected_object = None;
    self.properties.clear();
  }

  fn update_properties(&mut self) {
    self.properties.clear();

    if let Some(object) = &self.selected_object {
      match object.object_type {
        ObjectType::Asset => {
          self.add_asset_properties(object);
        }
        ObjectType::Clip => {
          self.add_clip_properties(object);
        }
        ObjectType::Effect => {
          self.add_effect_properties(object);
        }
        ObjectType::Track => {
          self.add_track_properties(object);
        }
        ObjectType::Project => {
          self.add_project_properties(object);
        }
        ObjectType::Pipeline => {
          self.add_pipeline_properties(object);
        }
        ObjectType::Graph => {
          self.add_graph_properties(object);
        }
        ObjectType::Node => {
          self.add_node_properties(object);
        }
      }
    }
  }

  fn add_asset_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Asset name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "type".to_string(),
      name: "Type".to_string(),
      property_type: PropertyType::Choice(vec![
        "Audio".to_string(),
        "Image".to_string(),
        "Video".to_string(),
        "Raw".to_string(),
      ]),
      value: PropertyValue::Choice("Audio".to_string()),
      category: "General".to_string(),
      description: "Asset type".to_string(),
      read_only: true,
      advanced: false,
    });

    self.properties.push(Property {
      id: "file_path".to_string(),
      name: "File Path".to_string(),
      property_type: PropertyType::FilePath,
      value: PropertyValue::FilePath("/path/to/asset.wav".to_string()),
      category: "File".to_string(),
      description: "Path to the asset file".to_string(),
      read_only: true,
      advanced: false,
    });

    self.properties.push(Property {
      id: "duration".to_string(),
      name: "Duration".to_string(),
      property_type: PropertyType::Float,
      value: PropertyValue::Float(180.0),
      category: "Audio".to_string(),
      description: "Duration in seconds".to_string(),
      read_only: true,
      advanced: false,
    });

    self.properties.push(Property {
      id: "sample_rate".to_string(),
      name: "Sample Rate".to_string(),
      property_type: PropertyType::Integer,
      value: PropertyValue::Integer(44100),
      category: "Audio".to_string(),
      description: "Audio sample rate in Hz".to_string(),
      read_only: true,
      advanced: false,
    });

    self.properties.push(Property {
      id: "channels".to_string(),
      name: "Channels".to_string(),
      property_type: PropertyType::Integer,
      value: PropertyValue::Integer(2),
      category: "Audio".to_string(),
      description: "Number of audio channels".to_string(),
      read_only: true,
      advanced: false,
    });

    self.properties.push(Property {
      id: "bit_depth".to_string(),
      name: "Bit Depth".to_string(),
      property_type: PropertyType::Integer,
      value: PropertyValue::Integer(16),
      category: "Audio".to_string(),
      description: "Audio bit depth".to_string(),
      read_only: true,
      advanced: true,
    });
  }

  fn add_clip_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Clip name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "start_time".to_string(),
      name: "Start Time".to_string(),
      property_type: PropertyType::Float,
      value: PropertyValue::Float(0.0),
      category: "Timing".to_string(),
      description: "Clip start time in seconds".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "duration".to_string(),
      name: "Duration".to_string(),
      property_type: PropertyType::Float,
      value: PropertyValue::Float(10.0),
      category: "Timing".to_string(),
      description: "Clip duration in seconds".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "volume".to_string(),
      name: "Volume".to_string(),
      property_type: PropertyType::Range(0.0, 2.0),
      value: PropertyValue::Range(1.0),
      category: "Audio".to_string(),
      description: "Clip volume multiplier".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "pan".to_string(),
      name: "Pan".to_string(),
      property_type: PropertyType::Range(-1.0, 1.0),
      value: PropertyValue::Range(0.0),
      category: "Audio".to_string(),
      description: "Stereo pan position".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "color".to_string(),
      name: "Color".to_string(),
      property_type: PropertyType::Color,
      value: PropertyValue::Color(egui::Color32::from_rgb(100, 150, 255)),
      category: "Visual".to_string(),
      description: "Clip color for timeline display".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  fn add_effect_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Effect name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "enabled".to_string(),
      name: "Enabled".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(true),
      category: "General".to_string(),
      description: "Whether the effect is enabled".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "intensity".to_string(),
      name: "Intensity".to_string(),
      property_type: PropertyType::Range(0.0, 1.0),
      value: PropertyValue::Range(0.5),
      category: "Parameters".to_string(),
      description: "Effect intensity".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "mix".to_string(),
      name: "Mix".to_string(),
      property_type: PropertyType::Range(0.0, 1.0),
      value: PropertyValue::Range(1.0),
      category: "Parameters".to_string(),
      description: "Dry/wet mix".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  fn add_track_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Track name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "muted".to_string(),
      name: "Muted".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(false),
      category: "Audio".to_string(),
      description: "Whether the track is muted".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "solo".to_string(),
      name: "Solo".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(false),
      category: "Audio".to_string(),
      description: "Whether the track is soloed".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "volume".to_string(),
      name: "Volume".to_string(),
      property_type: PropertyType::Range(0.0, 2.0),
      value: PropertyValue::Range(1.0),
      category: "Audio".to_string(),
      description: "Track volume multiplier".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "pan".to_string(),
      name: "Pan".to_string(),
      property_type: PropertyType::Range(-1.0, 1.0),
      value: PropertyValue::Range(0.0),
      category: "Audio".to_string(),
      description: "Track pan position".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  fn add_project_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Project name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "sample_rate".to_string(),
      name: "Sample Rate".to_string(),
      property_type: PropertyType::Integer,
      value: PropertyValue::Integer(44100),
      category: "Audio".to_string(),
      description: "Project sample rate".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "bit_depth".to_string(),
      name: "Bit Depth".to_string(),
      property_type: PropertyType::Integer,
      value: PropertyValue::Integer(16),
      category: "Audio".to_string(),
      description: "Project bit depth".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "tempo".to_string(),
      name: "Tempo".to_string(),
      property_type: PropertyType::Float,
      value: PropertyValue::Float(120.0),
      category: "Timing".to_string(),
      description: "Project tempo in BPM".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "time_signature".to_string(),
      name: "Time Signature".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String("4/4".to_string()),
      category: "Timing".to_string(),
      description: "Project time signature".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  fn add_pipeline_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Pipeline name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "enabled".to_string(),
      name: "Enabled".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(true),
      category: "General".to_string(),
      description: "Whether the pipeline is enabled".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "parallel".to_string(),
      name: "Parallel".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(false),
      category: "Execution".to_string(),
      description: "Whether to run stages in parallel".to_string(),
      read_only: false,
      advanced: true,
    });
  }

  fn add_graph_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Graph name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "auto_execute".to_string(),
      name: "Auto Execute".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(false),
      category: "Execution".to_string(),
      description: "Whether to auto-execute on changes".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  fn add_node_properties(&mut self, object: &InspectableObject) {
    self.properties.push(Property {
      id: "name".to_string(),
      name: "Name".to_string(),
      property_type: PropertyType::String,
      value: PropertyValue::String(object.name.clone()),
      category: "General".to_string(),
      description: "Node name".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "position".to_string(),
      name: "Position".to_string(),
      property_type: PropertyType::Vector2,
      value: PropertyValue::Vector2(glam::Vec2::new(100.0, 100.0)),
      category: "Layout".to_string(),
      description: "Node position in graph".to_string(),
      read_only: false,
      advanced: false,
    });

    self.properties.push(Property {
      id: "enabled".to_string(),
      name: "Enabled".to_string(),
      property_type: PropertyType::Boolean,
      value: PropertyValue::Boolean(true),
      category: "General".to_string(),
      description: "Whether the node is enabled".to_string(),
      read_only: false,
      advanced: false,
    });
  }

  pub fn render(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search properties..."));

      ui.checkbox(&mut self.show_advanced, "Advanced");

      if ui.button("Reset").clicked() {
        self.reset_properties();
      }
    });

    ui.separator();

    if let Some(object) = &self.selected_object {
      ui.heading(&object.name);
      ui.label(format!("Type: {:?}", object.object_type));
      ui.separator();

      let mut categories: std::collections::HashSet<String> =
        self.properties.iter().map(|p| p.category.clone()).collect();

      let mut categories: Vec<String> = categories.into_iter().collect();
      categories.sort();

      for category in categories {
        if let Some(filter) = &self.category_filter {
          if category != *filter {
            continue;
          }
        }

        egui::CollapsingHeader::new(&category)
          .default_open(true)
          .show(ui, |ui| {
            self.render_properties_in_category(ui, &category);
          });
      }
    } else {
      ui.centered_and_justified(|ui| {
        ui.label("No object selected\nSelect an object to view its properties");
      });
    }
  }

  fn render_properties_in_category(&mut self, ui: &mut egui::Ui, category: &str) {
    for property in &mut self
      .properties
      .iter_mut()
      .filter(|p| p.category == category && (!p.advanced || self.show_advanced))
    {
      if !self.search_query.is_empty()
        && !property
          .name
          .to_lowercase()
          .contains(&self.search_query.to_lowercase())
      {
        continue;
      }

      self.render_property(ui, property);
    }
  }

  fn render_property(&mut self, ui: &mut egui::Ui, property: &mut Property) {
    ui.horizontal(|ui| {
      ui.label(&property.name);
      ui.add_space(10.0);

      match &mut property.value {
        PropertyValue::String(value) => {
          if property.read_only {
            ui.label(value);
          } else {
            ui.add(egui::TextEdit::singleline(value));
          }
        }
        PropertyValue::Integer(value) => {
          if property.read_only {
            ui.label(format!("{}", value));
          } else {
            ui.add(egui::DragValue::new(value));
          }
        }
        PropertyValue::Float(value) => {
          if property.read_only {
            ui.label(format!("{:.2}", value));
          } else {
            ui.add(egui::DragValue::new(value).speed(0.1));
          }
        }
        PropertyValue::Boolean(value) => {
          if property.read_only {
            ui.label(if *value { "True" } else { "False" });
          } else {
            ui.checkbox(value, "");
          }
        }
        PropertyValue::Color(value) => {
          if property.read_only {
            ui.colored_label(*value, format!("{:?}", value));
          } else {
            let mut color = [value.r(), value.g(), value.b(), value.a()];
            if egui::color_picker::color_edit_button_rgba_unmultiplied(ui, &mut color) {
              *value =
                egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            }
          }
        }
        PropertyValue::Vector2(value) => {
          if property.read_only {
            ui.label(format!("({:.1}, {:.1})", value.x, value.y));
          } else {
            ui.horizontal(|ui| {
              ui.add(egui::DragValue::new(&mut value.x).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.y).speed(0.1));
            });
          }
        }
        PropertyValue::Vector3(value) => {
          if property.read_only {
            ui.label(format!("({:.1}, {:.1}, {:.1})", value.x, value.y, value.z));
          } else {
            ui.horizontal(|ui| {
              ui.add(egui::DragValue::new(&mut value.x).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.y).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.z).speed(0.1));
            });
          }
        }
        PropertyValue::Vector4(value) => {
          if property.read_only {
            ui.label(format!(
              "({:.1}, {:.1}, {:.1}, {:.1})",
              value.x, value.y, value.z, value.w
            ));
          } else {
            ui.horizontal(|ui| {
              ui.add(egui::DragValue::new(&mut value.x).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.y).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.z).speed(0.1));
              ui.add(egui::DragValue::new(&mut value.w).speed(0.1));
            });
          }
        }
        PropertyValue::FilePath(value) => {
          if property.read_only {
            ui.label(value);
          } else {
            ui.horizontal(|ui| {
              ui.add(egui::TextEdit::singleline(value));
              if ui.button("...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                  *value = path.to_string_lossy().to_string();
                }
              }
            });
          }
        }
        PropertyValue::DirectoryPath(value) => {
          if property.read_only {
            ui.label(value);
          } else {
            ui.horizontal(|ui| {
              ui.add(egui::TextEdit::singleline(value));
              if ui.button("...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                  *value = path.to_string_lossy().to_string();
                }
              }
            });
          }
        }
        PropertyValue::Choice(value) => {
          if property.read_only {
            ui.label(value);
          } else {
            if let PropertyType::Choice(choices) = &property.property_type {
              egui::ComboBox::from_label("")
                .selected_text(value)
                .show_ui(ui, |ui| {
                  for choice in choices {
                    ui.selectable_value(value, choice.clone(), choice);
                  }
                });
            }
          }
        }
        PropertyValue::Range(value) => {
          if property.read_only {
            ui.label(format!("{:.2}", value));
          } else {
            if let PropertyType::Range(min, max) = &property.property_type {
              ui.add(egui::Slider::new(value, *min..=*max));
            }
          }
        }
        PropertyValue::Text(value) => {
          if property.read_only {
            ui.label(value);
          } else {
            ui.add(
              egui::TextEdit::multiline(value)
                .desired_width(f32::INFINITY)
                .desired_rows(3),
            );
          }
        }
      }

      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if !property.description.is_empty() {
          ui.label("ℹ");
          if ui.is_item_hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), || egui::Label::new(&property.description));
          }
        }
      });
    });
  }

  fn reset_properties(&mut self) {
    if let Some(object) = &self.selected_object {
      self.update_properties();
    }
  }
}

impl Default for InspectorPanel {
  fn default() -> Self {
    Self::new()
  }
}
