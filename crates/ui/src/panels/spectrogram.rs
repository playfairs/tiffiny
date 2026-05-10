use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct SpectrogramPanel {
    pub audio_data: Option<Vec<f32>>,
    pub sample_rate: u32,
    pub fft_size: usize,
    pub window_size: usize,
    pub hop_size: usize,
    pub zoom_level: f32,
    pub scroll_position: f32,
    pub color_map: ColorMap,
    pub show_ruler: bool,
    pub show_grid: bool,
    pub min_freq: f32,
    pub max_freq: f32,
    pub min_db: f32,
    pub max_db: f32,
}

#[derive(Debug, Clone)]
pub enum ColorMap {
    Heat,
    Viridis,
    Plasma,
    Grayscale,
    BlueRed,
    Custom(Vec<egui::Color32>),
}

impl SpectrogramPanel {
    pub fn new() -> Self {
        Self {
            audio_data: None,
            sample_rate: 44100,
            fft_size: 2048,
            window_size: 2048,
            hop_size: 512,
            zoom_level: 1.0,
            scroll_position: 0.0,
            color_map: ColorMap::Heat,
            show_ruler: true,
            show_grid: true,
            min_freq: 20.0,
            max_freq: 20000.0,
            min_db: -80.0,
            max_db: 0.0,
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

    pub fn set_frequency_range(&mut self, min_freq: f32, max_freq: f32) {
        self.min_freq = min_freq.max(0.0);
        self.max_freq = max_freq.max(min_freq);
    }

    pub fn set_db_range(&mut self, min_db: f32, max_db: f32) {
        self.min_db = min_db;
        self.max_db = max_db.max(min_db);
    }

    pub fn compute_spectrogram(&self) -> Option<Vec<Vec<f32>>> {
        if let Some(audio_data) = &self.audio_data {
            Some(self.compute_fft_spectrogram(audio_data))
        } else {
            None
        }
    }

    fn compute_fft_spectrogram(&self, audio_data: &[f32]) -> Vec<Vec<f32>> {
        let num_frames = (audio_data.len() - self.window_size) / self.hop_size + 1;
        let mut spectrogram = Vec::with_capacity(num_frames);

        for frame_idx in 0..num_frames {
            let start = frame_idx * self.hop_size;
            let end = start + self.window_size;
            
            if end > audio_data.len() {
                break;
            }

            let frame = &audio_data[start..end];
            let windowed_frame = self.apply_window(frame);
            let fft_result = self.compute_fft(&windowed_frame);
            let magnitudes = self.compute_magnitudes(&fft_result);
            let log_magnitudes = self.apply_log_scale(&magnitudes);
            
            spectrogram.push(log_magnitudes);
        }

        spectrogram
    }

    fn apply_window(&self, frame: &[f32]) -> Vec<f32> {
        let mut windowed = Vec::with_capacity(frame.len());
        for (i, &sample) in frame.iter().enumerate() {
            let window_value = (std::f32::consts::PI * i as f32 / (frame.len() - 1) as f32).sin();
            windowed.push(sample * window_value);
        }
        windowed
    }

    fn compute_fft(&self, samples: &[f32]) -> Vec<rustfft::num_complex::Complex32> {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_size);
        
        let mut buffer: Vec<rustfft::num_complex::Complex32> = samples
            .iter()
            .map(|&x| rustfft::num_complex::Complex32::new(x, 0.0))
            .collect();
        
        buffer.resize(self.fft_size, rustfft::num_complex::Complex32::new(0.0, 0.0));
        fft.process(&mut buffer);
        
        buffer
    }

    fn compute_magnitudes(&self, fft_result: &[rustfft::num_complex::Complex32]) -> Vec<f32> {
        fft_result
            .iter()
            .take(self.fft_size / 2 + 1)
            .map(|c| (c.norm_sqr() as f32).sqrt())
            .collect()
    }

    fn apply_log_scale(&self, magnitudes: &[f32]) -> Vec<f32> {
        magnitudes
            .iter()
            .map(|&mag| {
                if mag > 0.0 {
                    20.0 * mag.log10()
                } else {
                    self.min_db
                }
            })
            .collect()
    }

    fn frequency_to_bin(&self, freq: f32) -> usize {
        ((freq * self.fft_size as f32) / self.sample_rate as f32) as usize
    }

    fn bin_to_frequency(&self, bin: usize) -> f32 {
        (bin as f32 * self.sample_rate as f32) / self.fft_size as f32
    }

    fn db_to_color(&self, db: f32) -> egui::Color32 {
        let normalized = ((db - self.min_db) / (self.max_db - self.min_db)).clamp(0.0, 1.0);
        
        match &self.color_map {
            ColorMap::Heat => {
                let r = (normalized * 255.0) as u8;
                let g = ((normalized * 255.0) * 0.7) as u8;
                let b = ((normalized * 255.0) * 0.4) as u8;
                egui::Color32::from_rgb(r, g, b)
            },
            ColorMap::Viridis => {
                let r = (normalized * 255.0 * 0.4) as u8;
                let g = (normalized * 255.0 * 0.8) as u8;
                let b = (normalized * 255.0) as u8;
                egui::Color32::from_rgb(r, g, b)
            },
            ColorMap::Plasma => {
                let r = (normalized * 255.0) as u8;
                let g = ((1.0 - normalized) * 255.0 * 0.8) as u8;
                let b = ((1.0 - normalized) * 255.0) as u8;
                egui::Color32::from_rgb(r, g, b)
            },
            ColorMap::Grayscale => {
                let gray = (normalized * 255.0) as u8;
                egui::Color32::from_gray(gray)
            },
            ColorMap::BlueRed => {
                let r = (normalized * 255.0) as u8;
                let b = ((1.0 - normalized) * 255.0) as u8;
                egui::Color32::from_rgb(r, 0, b)
            },
            ColorMap::Custom(colors) => {
                let index = (normalized * (colors.len() - 1) as f32) as usize;
                colors[index.min(colors.len() - 1)]
            },
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
                    .pick_file() {
                    
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

            ui.add_space(10.0);

            egui::ComboBox::from_label("Color Map")
                .selected_text(format!("{:?}", self.color_map))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.color_map, ColorMap::Heat, "Heat");
                    ui.selectable_value(&mut self.color_map, ColorMap::Viridis, "Viridis");
                    ui.selectable_value(&mut self.color_map, ColorMap::Plasma, "Plasma");
                    ui.selectable_value(&mut self.color_map, ColorMap::Grayscale, "Grayscale");
                    ui.selectable_value(&mut self.color_map, ColorMap::BlueRed, "Blue-Red");
                });
        });

        ui.separator();

        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(16, 16, 16));
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)));

        if self.show_ruler {
            self.render_frequency_ruler(ui, width, height);
            self.render_time_ruler(ui, width, height);
        }

        if self.show_grid {
            self.render_grid(ui, width, height);
        }

        if let Some(spectrogram) = self.compute_spectrogram() {
            self.render_spectrogram(ui, &spectrogram, width, height);
        } else {
            self.render_empty_state(ui, width, height);
        }
    }

    fn load_audio_file(&mut self, file_path: String) {
        tracing::info!("Loading audio file for spectrogram: {}", file_path);
        
        let sample_data = vec![0.5f32; 44100 * 10];
        self.set_audio_data(sample_data, 44100);
    }

    fn render_frequency_ruler(&self, ui: &mut egui::Ui, width: f32, height: f32) {
        let painter = ui.painter();
        let ruler_width = 60.0;
        let ruler_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(ruler_width, height)
        );

        painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(32, 32, 32));

        let freq_range = self.max_freq - self.min_freq;
        let pixels_per_hz = (height - 30.0) / freq_range;

        for freq in (0..=20000).step_by(2000) {
            if freq < self.min_freq || freq > self.max_freq {
                continue;
            }

            let y = height - 30.0 - ((freq - self.min_freq) * pixels_per_hz);
            
            painter.line_segment(
                [egui::pos2(ruler_rect.min.x, y), egui::pos2(ruler_rect.max.x, y)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100))
            );

            painter.text(
                egui::pos2(ruler_rect.min.x + 2.0, y - 6.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{}Hz", freq),
                egui::FontId::default(),
                egui::Color32::from_rgb(200, 200, 200)
            );
        }
    }

    fn render_time_ruler(&self, ui: &mut egui::Ui, width: f32, height: f32) {
        let painter = ui.painter();
        let ruler_height = 30.0;
        let ruler_rect = egui::Rect::from_min_size(
            egui::pos2(60.0, height - ruler_height),
            egui::vec2(width - 60.0, ruler_height)
        );

        painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(32, 32, 32));

        if let Some(audio_data) = &self.audio_data {
            let duration = audio_data.len() as f64 / self.sample_rate as f64;
            let pixels_per_second = (width - 60.0) / duration as f32 * self.zoom_level;
            let start_time = self.scroll_position / pixels_per_second;
            let end_time = (self.scroll_position + width - 60.0) / pixels_per_second;

            let mut current_time = (start_time / 1.0).floor() * 1.0;
            while current_time <= end_time {
                let x = 60.0 + ((current_time - start_time) * pixels_per_second as f64) as f32;
                
                painter.line_segment(
                    [egui::pos2(x, ruler_rect.min.y), egui::pos2(x, ruler_rect.max.y)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100))
                );

                painter.text(
                    egui::pos2(x + 2.0, ruler_rect.min.y + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.1}s", current_time),
                    egui::FontId::default(),
                    egui::Color32::from_rgb(200, 200, 200)
                );

                current_time += 1.0;
            }
        }
    }

    fn render_grid(&self, ui: &mut egui::Ui, width: f32, height: f32) {
        let painter = ui.painter();
        let grid_spacing = 50.0;
        let freq_ruler_width = 60.0;
        let time_ruler_height = 30.0;

        for x in (freq_ruler_width as i32..width as i32).step_by(grid_spacing as usize) {
            let x = x as f32;
            painter.line_segment(
                [egui::pos2(x, 0.0), egui::pos2(x, height - time_ruler_height)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40))
            );
        }

        for y in (0..(height - time_ruler_height) as i32).step_by(grid_spacing as usize) {
            let y = y as f32;
            painter.line_segment(
                [egui::pos2(freq_ruler_width, y), egui::pos2(width, y)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40))
            );
        }
    }

    fn render_spectrogram(&self, ui: &mut egui::Ui, spectrogram: &[Vec<f32>], width: f32, height: f32) {
        let painter = ui.painter();
        let freq_ruler_width = 60.0;
        let time_ruler_height = 30.0;
        let spectrogram_width = width - freq_ruler_width;
        let spectrogram_height = height - time_ruler_height;

        let spectrogram_rect = egui::Rect::from_min_size(
            egui::pos2(freq_ruler_width, 0.0),
            egui::vec2(spectrogram_width, spectrogram_height)
        );

        if spectrogram.is_empty() {
            return;
        }

        let num_frames = spectrogram.len();
        let num_bins = spectrogram[0].len();
        let pixels_per_frame = spectrogram_width / num_frames as f32 * self.zoom_level;
        let pixels_per_bin = spectrogram_height / num_bins as f32;

        for (frame_idx, frame) in spectrogram.iter().enumerate() {
            let x = freq_ruler_width + (frame_idx as f32 * pixels_per_frame) - self.scroll_position;
            
            if x < freq_ruler_width || x > width {
                continue;
            }

            for (bin_idx, &db) in frame.iter().enumerate() {
                let freq_bin = self.frequency_to_bin(self.min_freq + bin_idx as f32 * (self.max_freq - self.min_freq) / num_bins as f32);
                if freq_bin >= num_bins {
                    continue;
                }

                let y = spectrogram_height - ((bin_idx as f32 + 1.0) * pixels_per_bin);
                
                if y < 0.0 || y > spectrogram_height {
                    continue;
                }

                let color = self.db_to_color(db);
                let pixel_rect = egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::vec2(pixels_per_frame.max(1.0), pixels_per_bin.max(1.0))
                );

                painter.rect_filled(pixel_rect, 0.0, color);
            }
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
            egui::Color32::from_rgb(128, 128, 128)
        );
    }
}

impl Default for SpectrogramPanel {
    fn default() -> Self {
        Self::new()
    }
}
