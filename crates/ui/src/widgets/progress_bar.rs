use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub id: String,
    pub label: String,
    pub progress: f32,
    pub min: f32,
    pub max: f32,
    pub show_percentage: bool,
    pub show_text: bool,
    pub color: Option<egui::Color32>,
    pub height: f32,
    pub animated: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
}

impl ProgressBar {
    pub fn new(id: String) -> Self {
        Self {
            id,
            label: String::new(),
            progress: 0.0,
            min: 0.0,
            max: 1.0,
            show_percentage: true,
            show_text: false,
            color: None,
            height: 20.0,
            animated: true,
            enabled: true,
            visible: true,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(self.min, self.max);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.progress = self.progress.clamp(min, max);
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn show_text(mut self, show: bool) -> Self {
        self.show_text = show;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
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

        if !self.label.is_empty() {
            ui.label(&self.label);
        }

        let bar_color = self.color.unwrap_or(ui.visuals().selection.bg_fill);
        let text_color = ui.visuals().text_color();

Calculate normalized progress
        let normalized_progress = if self.max > self.min {
            (self.progress - self.min) / (self.max - self.min)
        } else {
            0.0
        }.clamp(0.0, 1.0);

        let progress_bar = egui::ProgressBar::new(normalized_progress)
            .desired_width(f32::INFINITY)
            .desired_height(self.height)
            .fill(bar_color)
            .show_percentage(self.show_percentage);

        let response = ui.add(progress_bar);

        if self.show_text {
            ui.horizontal(|ui| {
                ui.label(format!("{:.1}%", normalized_progress * 100.0));
                
                if self.min != 0.0 || self.max != 1.0 {
                    ui.label(format!("({:.1}/{:.1})", self.progress, self.max));
                }
            });
        }

        if self.animated && ui.ctx().frame_time() > 0.0 {
            let time = ui.ctx().frame_time().elapsed_secs_f64();
            let animated_progress = (time.sin() * 0.5 + 0.5) as f32;
            
            if (animated_progress - self.progress).abs() > 0.01 {
                self.progress = animated_progress * (self.max - self.min) + self.min;
                changed = true;
                
                if let Some(callback) = &self.on_change {
                    callback(self.progress);
                }
            }
        }

        changed
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn set_progress(&mut self, progress: f32) {
        let old_progress = self.progress;
        self.progress = progress.clamp(self.min, self.max);
        
        if (self.progress - old_progress).abs() > 0.001 {
            if let Some(callback) = &self.on_change {
                callback(self.progress);
            }
        }
    }

    pub fn get_normalized_progress(&self) -> f32 {
        if self.max > self.min {
            (self.progress - self.min) / (self.max - self.min)
        } else {
            0.0
        }.clamp(0.0, 1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.max
    }

    pub fn reset(&mut self) {
        self.progress = self.min;
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new("default_progress_bar".to_string())
    }
}

pub struct IndeterminateProgressBar {
    pub id: String,
    pub label: String,
    pub color: Option<egui::Color32>,
    pub height: f32,
    pub speed: f32,
    pub enabled: bool,
    pub visible: bool,
    animation_time: f64,
}

impl IndeterminateProgressBar {
    pub fn new(id: String) -> Self {
        Self {
            id,
            label: String::new(),
            color: None,
            height: 20.0,
            speed: 1.0,
            enabled: true,
            visible: true,
            animation_time: 0.0,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
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

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if !self.visible {
            return;
        }

        if !self.label.is_empty() {
            ui.label(&self.label);
        }

        self.animation_time += ui.ctx().frame_time().elapsed_secs_f64() * self.speed as f64;

        let progress = (self.animation_time.sin() * 0.5 + 0.5) as f32;
        
        let bar_color = self.color.unwrap_or(ui.visuals().selection.bg_fill);
        
        let progress_bar = egui::ProgressBar::new(progress)
            .desired_width(f32::INFINITY)
            .desired_height(self.height)
            .fill(bar_color)
            .show_percentage(false);

        ui.add(progress_bar);
    }

    pub fn reset_animation(&mut self) {
        self.animation_time = 0.0;
    }
}

impl Default for IndeterminateProgressBar {
    fn default() -> Self {
        Self::new("default_indeterminate_progress_bar".to_string())
    }
}

pub struct CircularProgress {
    pub id: String,
    pub label: String,
    pub progress: f32,
    pub radius: f32,
    pub stroke_width: f32,
    pub color: Option<egui::Color32>,
    pub background_color: Option<egui::Color32>,
    pub show_percentage: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
}

impl CircularProgress {
    pub fn new(id: String) -> Self {
        Self {
            id,
            label: String::new(),
            progress: 0.0,
            radius: 30.0,
            stroke_width: 4.0,
            color: None,
            background_color: None,
            show_percentage: true,
            enabled: true,
            visible: true,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn background_color(mut self, color: egui::Color32) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
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

        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(self.radius * 2.0, self.radius * 2.0),
                egui::Sense::hover()
            );

            let painter = ui.painter();
            let center = rect.center();

            let bg_color = self.background_color.unwrap_or(ui.visuals().extreme_bg_color);
            painter.circle_filled(center, self.radius, bg_color);

            let progress_color = self.color.unwrap_or(ui.visuals().selection.bg_fill);
            let start_angle = -std::f32::consts::FRAC_PI_2;
            let end_angle = start_angle + (self.progress * 2.0 * std::f32::consts::PI);

            painter.arc_stroke(
                center,
                self.radius - self.stroke_width / 2.0,
                start_angle,
                end_angle,
                egui::Stroke::new(self.stroke_width, progress_color)
            );

            if self.show_percentage {
                let text = format!("{:.0}%", self.progress * 100.0);
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    &text,
                    egui::FontId::default(),
                    ui.visuals().text_color()
                );
            }
        });

        if !self.label.is_empty() {
            ui.label(&self.label);
        }

        changed
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn set_progress(&mut self, progress: f32) {
        let old_progress = self.progress;
        self.progress = progress.clamp(0.0, 1.0);
        
        if (self.progress - old_progress).abs() > 0.001 {
            if let Some(callback) = &self.on_change {
                callback(self.progress);
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
    }
}

impl Default for CircularProgress {
    fn default() -> Self {
        Self::new("default_circular_progress".to_string())
    }
}
