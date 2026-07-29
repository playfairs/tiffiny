use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WaveformPanel {
  pub audio_data: Option<Vec<f32>>,
  pub sample_rate: u32,
  pub zoom_level: f32,
  pub scroll_position: f32,
  pub show_ruler: bool,
  pub show_grid: bool,
  pub color: egui::Color32,
  pub background_color: egui::Color32,
  pub ruler_color: egui::Color32,
  pub grid_color: egui::Color32,
}

impl WaveformPanel {
  pub fn new() -> Self {
    Self {
      audio_data: None,
      sample_rate: 44100,
      zoom_level: 1.0,
      scroll_position: 0.0,
      show_ruler: true,
      show_grid: true,
      color: egui::Color32::from_rgb(0, 255, 127),
      background_color: egui::Color32::from_rgb(16, 16, 16),
      ruler_color: egui::Color32::from_rgb(100, 100, 100),
      grid_color: egui::Color32::from_rgb(40, 40, 40),
    }
  }

  pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub fn set_audio_data(&mut self, data: Vec<f32>, sample_rate: u32) {
    self.audio_data = Some(data);
    self.sample_rate = sample_rate;
  }

  pub fn clear_audio_data(&mut self) {
    self.audio_data = None;
  }

  pub fn set_zoom_level(&mut self, zoom: f32) {
    self.zoom_level = zoom.clamp(0.1, 100.0);
  }

  pub fn set_scroll_position(&mut self, position: f32) {
    self.scroll_position = position.max(0.0);
  }

  pub fn get_duration_seconds(&self) -> f64 {
    if let Some(audio_data) = &self.audio_data {
      audio_data.len() as f64 / self.sample_rate as f64
    } else {
      0.0
    }
  }

  pub fn get_sample_at_time(&self, time_seconds: f64) -> Option<f32> {
    if let Some(audio_data) = &self.audio_data {
      let sample_index = (time_seconds * self.sample_rate as f64) as usize;
      if sample_index < audio_data.len() {
        Some(audio_data[sample_index])
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn get_rms_in_range(&self, start_time: f64, end_time: f64) -> Option<f32> {
    if let Some(audio_data) = &self.audio_data {
      let start_sample = (start_time * self.sample_rate as f64) as usize;
      let end_sample = (end_time * self.sample_rate as f64) as usize;

      if start_sample >= audio_data.len() || end_sample >= audio_data.len() {
        return None;
      }

      let range = &audio_data[start_sample..end_sample.min(audio_data.len())];
      if range.is_empty() {
        return Some(0.0);
      }

      let sum_squares: f32 = range.iter().map(|&sample| sample * sample).sum();
      (sum_squares / range.len() as f32).sqrt().into()
    } else {
      None
    }
  }

  pub fn get_peak_in_range(&self, start_time: f64, end_time: f64) -> Option<f32> {
    if let Some(audio_data) = &self.audio_data {
      let start_sample = (start_time * self.sample_rate as f64) as usize;
      let end_sample = (end_time * self.sample_rate as f64) as usize;

      if start_sample >= audio_data.len() || end_sample >= audio_data.len() {
        return None;
      }

      let range = &audio_data[start_sample..end_sample.min(audio_data.len())];
      if range.is_empty() {
        return Some(0.0);
      }

      range
        .iter()
        .map(|&sample| sample.abs())
        .fold(0.0f32, f32::max)
        .into()
    } else {
      None
    }
  }

  pub fn render(&mut self, ui: &mut egui::Ui) {
    let available_rect = ui.available_rect_before_wrap();
    let width = available_rect.width();
    let height = available_rect.height();

    ui.horizontal(|ui| {
      if ui.button("Load Audio").clicked() {
        if let Some(path) = rfd::FileDialog::new()
          .add_filter("Audio Files", &["wav", "mp3", "flac"])
          .pick_file()
        {
          self.load_audio_file(path.to_string_lossy().to_string());
        }
      }

      if ui.button("Clear").clicked() {
        self.clear_audio_data();
      }

      ui.add_space(10.0);

      ui.label(format!("Zoom: {:.1}x", self.zoom_level));
      ui.add(egui::Slider::new(&mut self.zoom_level, 0.1..=100.0).show_value(false));

      ui.checkbox(&mut self.show_ruler, "Ruler");
      ui.checkbox(&mut self.show_grid, "Grid");
    });

    ui.separator();

    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();

    painter.rect_filled(rect, 0.0, self.background_color);
    painter.rect_stroke(
      rect,
      0.0,
      egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)),
    );

    if self.show_ruler {
      self.render_ruler(ui, width, height);
    }

    if self.show_grid {
      self.render_grid(ui, width, height);
    }

    if let Some(audio_data) = &self.audio_data {
      self.render_waveform(ui, audio_data, width, height);
    } else {
      self.render_empty_state(ui, width, height);
    }
  }

  fn load_audio_file(&mut self, file_path: String) {
    tracing::info!("Loading audio file: {}", file_path);

    let sample_data = vec![0.5f32; 44100 * 10];
    self.set_audio_data(sample_data, 44100);
  }

  fn render_ruler(&self, ui: &mut egui::Ui, width: f32, height: f32) {
    let painter = ui.painter();
    let ruler_height = 30.0;
    let ruler_rect =
      egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(width, ruler_height));

    painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(32, 32, 32));

    let duration = self.get_duration_seconds();
    let pixels_per_second = width / duration as f32 * self.zoom_level;
    let start_time = self.scroll_position / pixels_per_second;
    let end_time = (self.scroll_position + width) / pixels_per_second;

    let mut current_time = (start_time / 1.0).floor() * 1.0;
    while current_time <= end_time {
      let x = self.time_to_pixel_x(current_time, width);

      painter.line_segment(
        [
          egui::pos2(x, ruler_rect.min.y),
          egui::pos2(x, ruler_rect.max.y),
        ],
        egui::Stroke::new(1.0, self.ruler_color),
      );

      painter.text(
        egui::pos2(x + 2.0, ruler_rect.min.y + 2.0),
        egui::Align2::LEFT_TOP,
        format!("{:.1}s", current_time),
        egui::FontId::default(),
        egui::Color32::from_rgb(200, 200, 200),
      );

      current_time += 1.0;
    }
  }

  fn render_grid(&self, ui: &mut egui::Ui, width: f32, height: f32) {
    let painter = ui.painter();
    let grid_spacing = 50.0;
    let ruler_height = if self.show_ruler { 30.0 } else { 0.0 };

    for x in (0..width as i32).step_by(grid_spacing as usize) {
      let x = x as f32;
      painter.line_segment(
        [egui::pos2(x, ruler_height), egui::pos2(x, height)],
        egui::Stroke::new(1.0, self.grid_color),
      );
    }

    for y in (ruler_height as i32..height as i32).step_by(grid_spacing as usize) {
      let y = y as f32;
      painter.line_segment(
        [egui::pos2(0.0, y), egui::pos2(width, y)],
        egui::Stroke::new(1.0, self.grid_color),
      );
    }

    let center_y = height / 2.0;
    painter.line_segment(
      [egui::pos2(0.0, center_y), egui::pos2(width, center_y)],
      egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 80)),
    );
  }

  fn render_waveform(&self, ui: &mut egui::Ui, audio_data: &[f32], width: f32, height: f32) {
    let painter = ui.painter();
    let ruler_height = if self.show_ruler { 30.0 } else { 0.0 };
    let waveform_rect = egui::Rect::from_min_size(
      egui::pos2(0.0, ruler_height),
      egui::vec2(width, height - ruler_height),
    );

    let duration = self.get_duration_seconds();
    let pixels_per_second = width / duration as f32 * self.zoom_level;
    let start_time = self.scroll_position / pixels_per_second;
    let end_time = (self.scroll_position + width) / pixels_per_second;

    let samples_per_pixel = (self.sample_rate as f64 / pixels_per_second as f64) as usize;
    let center_y = waveform_rect.center().y;

    for pixel_x in 0..width as usize {
      let time = start_time + (pixel_x as f64 / pixels_per_second as f64);
      if time > duration {
        break;
      }

      let sample_start = (time * self.sample_rate as f64) as usize;
      let sample_end = (sample_start + samples_per_pixel).min(audio_data.len());

      if sample_start >= audio_data.len() {
        break;
      }

      let range = &audio_data[sample_start..sample_end];
      if range.is_empty() {
        continue;
      }

      let max_sample = range.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
      let amplitude = max_sample * (waveform_rect.height() / 2.0 - 4.0);

      let x = pixel_x as f32;
      let y_top = center_y - amplitude;
      let y_bottom = center_y + amplitude;

      painter.line_segment(
        [egui::pos2(x, y_top), egui::pos2(x, y_bottom)],
        egui::Stroke::new(1.0, self.color),
      );
    }
  }

  fn render_empty_state(&self, ui: &mut egui::Ui, width: f32, height: f32) {
    let painter = ui.painter();
    let center = egui::pos2(width / 2.0, height / 2.0);

    painter.text(
      center,
      egui::Align2::CENTER_CENTER,
      "No audio data loaded\nClick 'Load Audio' to open a file",
      egui::FontId::default(),
      egui::Color32::from_rgb(128, 128, 128),
    );
  }

  fn time_to_pixel_x(&self, time: f64, width: f32) -> f32 {
    let duration = self.get_duration_seconds();
    let pixels_per_second = width / duration as f32 * self.zoom_level;
    (time * pixels_per_second as f64) as f32 - self.scroll_position
  }

  fn pixel_x_to_time(&self, pixel_x: f32, width: f32) -> f64 {
    let duration = self.get_duration_seconds();
    let pixels_per_second = width / duration as f32 * self.zoom_level;
    ((pixel_x + self.scroll_position) / pixels_per_second) as f64
  }
}

impl Default for WaveformPanel {
  fn default() -> Self {
    Self::new()
  }
}
