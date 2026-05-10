use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ColorPicker {
    pub id: String,
    pub label: String,
    pub color: egui::Color32,
    pub show_alpha: bool,
    pub show_hex: bool,
    pub show_rgb: bool,
    pub show_hsv: bool,
    pub enabled: bool,
    pub visible: bool,
    pub on_change: Option<Arc<dyn Fn(egui::Color32) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ColorSpace {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub alpha: f32,
}

impl ColorPicker {
    pub fn new(id: String) -> Self {
        Self {
            id,
            label: String::new(),
            color: egui::Color32::WHITE,
            show_alpha: true,
            show_hex: true,
            show_rgb: true,
            show_hsv: true,
            enabled: true,
            visible: true,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
        self
    }

    pub fn show_hex(mut self, show: bool) -> Self {
        self.show_hex = show;
        self
    }

    pub fn show_rgb(mut self, show: bool) -> Self {
        self.show_rgb = show;
        self
    }

    pub fn show_hsv(mut self, show: bool) -> Self {
        self.show_hsv = show;
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

    pub fn on_change(mut self, callback: impl Fn(egui::Color32) + Send + Sync + 'static) -> Self {
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

        ui.horizontal(|ui| {
Color preview box
            let preview_size = egui::vec2(60.0, 30.0);
            let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::click());
            
            let painter = ui.painter();
            painter.rect_filled(rect, 4.0, self.color);
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 128, 128)));

            if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
            }

            ui.add_space(10.0);

            ui.vertical(|ui| {
                if self.show_alpha {
                    let mut alpha = self.color.to_array()[3] as f32 / 255.0;
                    if ui.add(egui::Slider::new(&mut alpha, 0.0..=1.0).text("Alpha")).changed() {
                        let mut color_array = self.color.to_array();
                        color_array[3] = (alpha * 255.0) as u8;
                        self.color = egui::Color32::from_rgba_unmultiplied(
                            color_array[0], color_array[1], color_array[2], color_array[3]
                        );
                        changed = true;
                    }
                }

                if self.show_rgb {
                    ui.separator();
                    ui.label("RGB");
                    ui.horizontal(|ui| {
                        let mut r = self.color.r() as f32;
                        let mut g = self.color.g() as f32;
                        let mut b = self.color.b() as f32;

                        if ui.add(egui::Slider::new(&mut r, 0.0..=255.0).text("R")).changed() {
                            self.color = egui::Color32::from_rgb(r as u8, self.color.g(), self.color.b());
                            changed = true;
                        }

                        if ui.add(egui::Slider::new(&mut g, 0.0..=255.0).text("G")).changed() {
                            self.color = egui::Color32::from_rgb(self.color.r(), g as u8, self.color.b());
                            changed = true;
                        }

                        if ui.add(egui::Slider::new(&mut b, 0.0..=255.0).text("B")).changed() {
                            self.color = egui::Color32::from_rgb(self.color.r(), self.color.g(), b as u8);
                            changed = true;
                        }
                    });
                }

                if self.show_hsv {
                    ui.separator();
                    ui.label("HSV");
                    let mut hsv = self.rgb_to_hsv(self.color);
                    
                    ui.horizontal(|ui| {
                        if ui.add(egui::Slider::new(&mut hsv.hue, 0.0..=360.0).text("H")).changed() {
                            self.color = self.hsv_to_rgb(hsv);
                            changed = true;
                        }

                        if ui.add(egui::Slider::new(&mut hsv.saturation, 0.0..=1.0).text("S")).changed() {
                            self.color = self.hsv_to_rgb(hsv);
                            changed = true;
                        }

                        if ui.add(egui::Slider::new(&mut hsv.value, 0.0..=1.0).text("V")).changed() {
                            self.color = self.hsv_to_rgb(hsv);
                            changed = true;
                        }
                    });
                }

                if self.show_hex {
                    ui.separator();
                    ui.label("Hex");
                    let mut hex = self.color_to_hex(self.color);
                    if ui.add(egui::TextEdit::singleline(&mut hex).desired_width(100.0)).changed() {
                        if let Ok(color) = self.hex_to_color(&hex) {
                            self.color = color;
                            changed = true;
                        }
                    }
                }
            });
        });

        if changed {
            if let Some(callback) = &self.on_change {
                callback(self.color);
            }
        }

        changed
    }

    fn rgb_to_hsv(&self, color: egui::Color32) -> ColorSpace {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };

        let saturation = if max == 0.0 {
            0.0
        } else {
            delta / max
        };

        let value = max;

        ColorSpace {
            hue,
            saturation,
            value,
            alpha: color.a() as f32 / 255.0,
        }
    }

    fn hsv_to_rgb(&self, hsv: ColorSpace) -> egui::Color32 {
        let c = hsv.value * hsv.saturation;
        let x = c * (1.0 - ((hsv.hue / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = hsv.value - c;

        let r = if hsv.hue < 60.0 {
            c
        } else if hsv.hue < 120.0 {
            x
        } else if hsv.hue < 180.0 {
            0.0
        } else if hsv.hue < 240.0 {
            x
        } else if hsv.hue < 300.0 {
            c
        } else {
            x
        };

        let g = if hsv.hue < 60.0 {
            x
        } else if hsv.hue < 120.0 {
            c
        } else if hsv.hue < 180.0 {
            c
        } else if hsv.hue < 240.0 {
            x
        } else if hsv.hue < 300.0 {
            0.0
        } else {
            x
        };

        let b = if hsv.hue < 60.0 {
            0.0
        } else if hsv.hue < 120.0 {
            x
        } else if hsv.hue < 180.0 {
            c
        } else if hsv.hue < 240.0 {
            c
        } else if hsv.hue < 300.0 {
            x
        } else {
            c
        };

        let r = ((r + m) * 255.0) as u8;
        let g = ((g + m) * 255.0) as u8;
        let b = ((b + m) * 255.0) as u8;
        let a = (hsv.alpha * 255.0) as u8;

        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn color_to_hex(&self, color: egui::Color32) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", 
                color.r(), color.g(), color.b(), color.a())
    }

    fn hex_to_color(&self, hex: &str) -> Result<egui::Color32, ()> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(());
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| ())?
        } else {
            255
        };

        Ok(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
    }

    pub fn get_color(&self) -> egui::Color32 {
        self.color
    }

    pub fn set_color(&mut self, color: egui::Color32) {
        self.color = color;
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new("default_color_picker".to_string())
    }
}

pub struct ColorPalette {
    pub id: String,
    pub colors: Vec<egui::Color32>,
    pub selected_color: Option<egui::Color32>,
    pub columns: usize,
    pub enabled: bool,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(egui::Color32) + Send + Sync>>,
}

impl ColorPalette {
    pub fn new(id: String) -> Self {
        Self {
            id,
            colors: Vec::new(),
            selected_color: None,
            columns: 8,
            enabled: true,
            visible: true,
            on_select: None,
        }
    }

    pub fn colors(mut self, colors: Vec<egui::Color32>) -> Self {
        self.colors = colors;
        self
    }

    pub fn add_color(mut self, color: egui::Color32) -> Self {
        self.colors.push(color);
        self
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
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

    pub fn on_select(mut self, callback: impl Fn(egui::Color32) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut selected = false;

        ui.horizontal_wrapped(|ui| {
            for (index, color) in self.colors.iter().enumerate() {
                let color_size = egui::vec2(24.0, 24.0);
                let (rect, _) = ui.allocate_exact_size(color_size, egui::Sense::click());
                
                let painter = ui.painter();
                painter.rect_filled(rect, 2.0, *color);
                
                if self.selected_color == Some(*color) {
                    painter.rect_stroke(rect, 2.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                } else {
                    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 128, 128)));
                }

                if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
                    self.selected_color = Some(*color);
                    selected = true;
                    
                    if let Some(callback) = &self.on_select {
                        callback(*color);
                    }
                }

                if (index + 1) % self.columns == 0 {
                    ui.end_row();
                }
            }
        });

        selected
    }

    pub fn get_selected_color(&self) -> Option<egui::Color32> {
        self.selected_color
    }

    pub fn set_selected_color(&mut self, color: Option<egui::Color32>) {
        self.selected_color = color;
    }

    pub fn clear_selection(&mut self) {
        self.selected_color = None;
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new("default_palette".to_string())
    }
}
