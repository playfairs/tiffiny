use eframe::egui;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TimelinePanel {
  pub zoom_level: f32,
  pub scroll_position: f32,
  pub playhead_position: f64,
  pub is_playing: bool,
  pub tracks: Vec<Track>,
  pub selection: Option<Selection>,
  pub snap_to_grid: bool,
  pub grid_size: f64,
}

#[derive(Debug, Clone)]
pub struct Track {
  pub id: String,
  pub name: String,
  pub track_type: TrackType,
  pub height: f32,
  pub muted: bool,
  pub solo: bool,
  pub clips: Vec<Clip>,
  pub effects: Vec<TrackEffect>,
}

#[derive(Debug, Clone)]
pub enum TrackType {
  Audio,
  Video,
  Subtitle,
  Effect,
  Control,
}

#[derive(Debug, Clone)]
pub struct Clip {
  pub id: String,
  pub name: String,
  pub start_time: f64,
  pub duration: f64,
  pub track_id: String,
  pub clip_type: ClipType,
  pub color: egui::Color32,
  pub content: ClipContent,
}

#[derive(Debug, Clone)]
pub enum ClipType {
  Audio,
  Video,
  Image,
  Effect,
  Text,
}

#[derive(Debug, Clone)]
pub enum ClipContent {
  AudioFile {
    path: String,
    sample_rate: u32,
  },
  VideoFile {
    path: String,
    frame_rate: f64,
  },
  ImageFile {
    path: String,
  },
  Effect {
    effect_id: String,
    parameters: std::collections::HashMap<String, serde_json::Value>,
  },
  Text {
    content: String,
    font: String,
  },
}

#[derive(Debug, Clone)]
pub struct TrackEffect {
  pub id: String,
  pub name: String,
  pub effect_type: String,
  pub enabled: bool,
  pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Selection {
  pub start_time: f64,
  pub end_time: f64,
  pub track_ids: Vec<String>,
  pub clip_ids: Vec<String>,
}

impl TimelinePanel {
  pub fn new() -> Self {
    Self {
      zoom_level: 1.0,
      scroll_position: 0.0,
      playhead_position: 0.0,
      is_playing: false,
      tracks: Vec::new(),
      selection: None,
      snap_to_grid: true,
      grid_size: 1.0,
    }
  }

  pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  pub fn add_track(&mut self, track: Track) {
    self.tracks.push(track);
  }

  pub fn remove_track(&mut self, track_id: &str) -> Option<Track> {
    let index = self.tracks.iter().position(|t| t.id == track_id);
    if let Some(index) = index {
      Some(self.tracks.remove(index))
    } else {
      None
    }
  }

  pub fn get_track(&self, track_id: &str) -> Option<&Track> {
    self.tracks.iter().find(|t| t.id == track_id)
  }

  pub fn get_mut_track(&mut self, track_id: &str) -> Option<&mut Track> {
    self.tracks.iter_mut().find(|t| t.id == track_id)
  }

  pub fn add_clip(&mut self, track_id: &str, clip: Clip) -> Result<(), String> {
    if let Some(track) = self.get_mut_track(track_id) {
      track.clips.push(clip);
      Ok(())
    } else {
      Err(format!("Track {} not found", track_id))
    }
  }

  pub fn remove_clip(&mut self, track_id: &str, clip_id: &str) -> Option<Clip> {
    if let Some(track) = self.get_mut_track(track_id) {
      let index = track.clips.iter().position(|c| c.id == clip_id);
      if let Some(index) = index {
        Some(track.clips.remove(index))
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn set_playhead_position(&mut self, position: f64) {
    self.playhead_position = position;
  }

  pub fn set_zoom_level(&mut self, zoom: f32) {
    self.zoom_level = zoom.clamp(0.1, 10.0);
  }

  pub fn set_scroll_position(&mut self, position: f32) {
    self.scroll_position = position;
  }

  pub fn set_selection(&mut self, selection: Option<Selection>) {
    self.selection = selection;
  }

  pub fn get_time_at_pixel(&self, pixel_x: f32, timeline_width: f32) -> f64 {
    let pixels_per_second = timeline_width / 10.0 * self.zoom_level;
    let time = (pixel_x + self.scroll_position) / pixels_per_second;
    if self.snap_to_grid {
      (time / self.grid_size).round() * self.grid_size
    } else {
      time
    }
  }

  pub fn get_pixel_at_time(&self, time: f64, timeline_width: f32) -> f32 {
    let pixels_per_second = timeline_width / 10.0 * self.zoom_level;
    (time * pixels_per_second) - self.scroll_position
  }

  pub fn get_clips_in_range(&self, start_time: f64, end_time: f64) -> Vec<&Clip> {
    let mut clips = Vec::new();
    for track in &self.tracks {
      for clip in &track.clips {
        if clip.start_time < end_time && (clip.start_time + clip.duration) > start_time {
          clips.push(clip);
        }
      }
    }
    clips
  }

  pub fn get_total_duration(&self) -> f64 {
    self
      .tracks
      .iter()
      .flat_map(|track| track.clips.iter())
      .map(|clip| clip.start_time + clip.duration)
      .fold(0.0, f64::max)
  }

  pub fn clear_all(&mut self) {
    self.tracks.clear();
    self.selection = None;
    self.playhead_position = 0.0;
  }

  pub fn create_default_tracks(&mut self) {
    self.add_track(Track {
      id: "audio_1".to_string(),
      name: "Audio 1".to_string(),
      track_type: TrackType::Audio,
      height: 60.0,
      muted: false,
      solo: false,
      clips: Vec::new(),
      effects: Vec::new(),
    });

    self.add_track(Track {
      id: "audio_2".to_string(),
      name: "Audio 2".to_string(),
      track_type: TrackType::Audio,
      height: 60.0,
      muted: false,
      solo: false,
      clips: Vec::new(),
      effects: Vec::new(),
    });

    self.add_track(Track {
      id: "video_1".to_string(),
      name: "Video 1".to_string(),
      track_type: TrackType::Video,
      height: 80.0,
      muted: false,
      solo: false,
      clips: Vec::new(),
      effects: Vec::new(),
    });
  }

  pub fn render(&mut self, ui: &mut egui::Ui) {
    let available_rect = ui.available_rect_before_wrap();
    let timeline_height = available_rect.height();
    let timeline_width = available_rect.width();

    ui.horizontal(|ui| {
      if ui.button("▶").clicked() {
        self.is_playing = !self.is_playing;
      }

      if ui.button("⏹").clicked() {
        self.is_playing = false;
        self.playhead_position = 0.0;
      }

      ui.add_space(10.0);

      ui.label(format!("Zoom: {:.1}x", self.zoom_level));
      ui.add(egui::Slider::new(&mut self.zoom_level, 0.1..=10.0).show_value(false));

      ui.add_space(10.0);

      ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");
      if self.snap_to_grid {
        ui.add(egui::Slider::new(&mut self.grid_size, 0.1..=10.0).text("Grid"));
      }
    });

    ui.separator();

    let header_height = 30.0;
    let track_header_width = 150.0;
    let timeline_area_height = timeline_height - header_height;

    egui::ScrollArea::both()
      .auto_shrink([false, false])
      .show(ui, |ui| {
        ui.horizontal(|ui| {
          ui.vertical(|ui| {
            ui.set_width(track_header_width);
            ui.set_height(header_height);
            ui.heading("Time");
          });

          ui.vertical(|ui| {
            ui.set_width(timeline_width - track_header_width);
            ui.set_height(header_height);
            self.render_time_ruler(ui, timeline_width - track_header_width);
          });
        });

        ui.separator();

        let mut current_y = 0.0;
        for track in &mut self.tracks {
          ui.horizontal(|ui| {
            ui.vertical(|ui| {
              ui.set_width(track_header_width);
              ui.set_height(track.height);
              self.render_track_header(ui, track);
            });

            ui.vertical(|ui| {
              ui.set_width(timeline_width - track_header_width);
              ui.set_height(track.height);
              self.render_track_content(ui, track, timeline_width - track_header_width);
            });
          });
          current_y += track.height;
        }
      });

    self.render_playhead(ui, timeline_width, header_height);
  }

  fn render_time_ruler(&self, ui: &mut egui::Ui, width: f32) {
    let pixels_per_second = width / 10.0 * self.zoom_level;
    let start_time = self.scroll_position / pixels_per_second;
    let end_time = (self.scroll_position + width) / pixels_per_second;

    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 40));

    let mut current_time = (start_time / 1.0).floor() * 1.0;
    while current_time <= end_time {
      let x = self.get_pixel_at_time(current_time, width);

      painter.line_segment(
        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
      );

      painter.text(
        egui::pos2(x + 2.0, rect.min.y + 2.0),
        egui::Align2::LEFT_TOP,
        format!("{:.1}s", current_time),
        egui::FontId::default(),
        egui::Color32::from_rgb(200, 200, 200),
      );

      current_time += 1.0;
    }
  }

  fn render_track_header(&self, ui: &mut egui::Ui, track: &Track) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let bg_color = if track.muted {
      egui::Color32::from_rgb(60, 40, 40)
    } else if track.solo {
      egui::Color32::from_rgb(40, 60, 40)
    } else {
      egui::Color32::from_rgb(50, 50, 50)
    };

    painter.rect_filled(rect, 4.0, bg_color);
    painter.rect_stroke(
      rect,
      4.0,
      egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
    );

    ui.horizontal(|ui| {
      ui.checkbox(&mut track.muted, "M");
      ui.checkbox(&mut track.solo, "S");
      ui.label(&track.name);
    });
  }

  fn render_track_content(&self, ui: &mut egui::Ui, track: &Track, width: f32) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 30));

    for clip in &track.clips {
      let clip_x = self.get_pixel_at_time(clip.start_time, width);
      let clip_width = (clip.duration * width / 10.0 * self.zoom_level as f64) as f32;

      if clip_x + clip_width < 0.0 || clip_x > width {
        continue;
      }

      let clip_rect = egui::Rect::from_min_size(
        egui::pos2(clip_x, rect.min.y + 2.0),
        egui::vec2(clip_width, rect.height() - 4.0),
      );

      painter.rect_filled(clip_rect, 2.0, clip.color);
      painter.rect_stroke(
        clip_rect,
        2.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
      );

      painter.text(
        egui::pos2(clip_x + 4.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        &clip.name,
        egui::FontId::default(),
        egui::Color32::WHITE,
      );
    }
  }

  fn render_playhead(&self, ui: &mut egui::Ui, width: f32, header_height: f32) {
    let playhead_x = self.get_pixel_at_time(self.playhead_position, width);

    if playhead_x >= 0.0 && playhead_x <= width {
      let painter = ui.painter();
      let rect = ui.available_rect_before_wrap();

      let playhead_top = rect.min.y + header_height;
      let playhead_bottom = rect.max.y;

      painter.line_segment(
        [
          egui::pos2(playhead_x, playhead_top),
          egui::pos2(playhead_x, playhead_bottom),
        ],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 69, 0)),
      );

      let triangle_points = vec![
        egui::pos2(playhead_x - 5.0, playhead_top),
        egui::pos2(playhead_x + 5.0, playhead_top),
        egui::pos2(playhead_x, playhead_top + 8.0),
      ];

      painter.add(egui::Shape::convex_polygon(
        triangle_points,
        egui::Color32::from_rgb(255, 69, 0),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 50, 0)),
      ));
    }
  }
}

impl Default for TimelinePanel {
  fn default() -> Self {
    Self::new()
  }
}
