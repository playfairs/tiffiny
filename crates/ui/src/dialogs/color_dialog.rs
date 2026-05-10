use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct ColorDialog {
    pub id: String,
    pub title: String,
    pub initial_color: egui::Color32,
    pub selected_color: egui::Color32,
    pub show_alpha: bool,
    pub show_hex: bool,
    pub show_rgb: bool,
    pub show_hsv: bool,
    pub preset_colors: Vec<egui::Color32>,
    pub visible: bool,
    pub on_select: Option<Arc<dyn Fn(egui::Color32) + Send + Sync>>,
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ColorDialog {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: "Select Color".to_string(),
            initial_color: egui::Color32::WHITE,
            selected_color: egui::Color32::WHITE,
            show_alpha: true,
            show_hex: true,
            show_rgb: true,
            show_hsv: true,
            preset_colors: vec![
                egui::Color32::WHITE,
                egui::Color32::BLACK,
                egui::Color32::from_rgb(255, 0, 0),Red
                egui::Color32::from_rgb(0, 255, 0),
                egui::Color32::from_rgb(0, 0, 255),
                egui::Color32::from_rgb(255, 255, 0),
                egui::Color32::from_rgb(255, 0, 255),
                egui::Color32::from_rgb(0, 255, 255),
                egui::Color32::from_rgb(128, 128, 128),
                egui::Color32::from_rgb(255, 165, 0),
                egui::Color32::from_rgb(128, 0, 128),
                egui::Color32::from_rgb(0, 128, 128),
            ],
            visible: false,
            on_select: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn initial_color(mut self, color: egui::Color32) -> Self {
        self.initial_color = color;
        self.selected_color = color;
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

    pub fn preset_colors(mut self, colors: Vec<egui::Color32>) -> Self {
        self.preset_colors = colors;
        self
    }

    pub fn add_preset_color(mut self, color: egui::Color32) -> Self {
        self.preset_colors.push(color);
        self
    }

    pub fn on_select(mut self, callback: impl Fn(egui::Color32) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(callback));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.selected_color = self.initial_color;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_selected_color(&self) -> egui::Color32 {
        self.selected_color
    }

    pub fn set_selected_color(&mut self, color: egui::Color32) {
        self.selected_color = color;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.visible {
            return false;
        }

        let mut closed = false;

        let screen_rect = ui.ctx().screen_rect();
        let dialog_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::vec2(400.0, 300.0)
        );

        egui::Area::new(dialog_rect)
            .interactable(true)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                let frame = egui::Frame::dark_canvas(ui.style())
                    .stroke(egui::Stroke::new(2.0, ui.visuals().window_fill))
                    .rounding(8.0);

                frame.show(ui, |ui| {
                    self.render_title_bar(ui, &mut closed);

                    ui.separator();

                    self.render_color_content(ui);
                });
            });

        if closed {
            self.hide();
            if let Some(callback) = &self.on_cancel {
                callback();
            }
        }

        closed
    }

    fn render_title_bar(&self, ui: &mut egui::Ui, closed: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    *closed = true;
                }
            });
        });
    }

    fn render_color_content(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.render_color_preview(ui);

                ui.add_space(10.0);

                if self.show_rgb {
                    self.render_rgb_sliders(ui);
                }

                if self.show_hsv {
                    ui.add_space(10.0);
                    self.render_hsv_sliders(ui);
                }

                if self.show_alpha {
                    ui.add_space(10.0);
                    self.render_alpha_slider(ui);
                }

                if self.show_hex {
                    ui.add_space(10.0);
                    self.render_hex_input(ui);
                }
            });

            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.label("Preset Colors:");
                self.render_preset_colors(ui);
            });
        });

        ui.separator();

        self.render_buttons(ui);
    }

    fn render_color_preview(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Preview:");
            
            let preview_size = egui::vec2(60.0, 40.0);
            let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
            
            let painter = ui.painter();
            painter.rect_filled(rect, 4.0, self.selected_color);
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 128, 128)));
        });
    }

    fn render_rgb_sliders(&mut self, ui: &mut egui::Ui) {
        ui.label("RGB:");
        
        let mut r = self.selected_color.r() as f32;
        let mut g = self.selected_color.g() as f32;
        let mut b = self.selected_color.b() as f32;

        ui.horizontal(|ui| {
            ui.label("R:");
            if ui.add(egui::Slider::new(&mut r, 0.0..=255.0)).changed() {
                self.update_color_from_rgb(r as u8, g as u8, b as u8);
            }
        });

        ui.horizontal(|ui| {
            ui.label("G:");
            if ui.add(egui::Slider::new(&mut g, 0.0..=255.0)).changed() {
                self.update_color_from_rgb(r as u8, g as u8, b as u8);
            }
        });

        ui.horizontal(|ui| {
            ui.label("B:");
            if ui.add(egui::Slider::new(&mut b, 0.0..=255.0)).changed() {
                self.update_color_from_rgb(r as u8, g as u8, b as u8);
            }
        });
    }

    fn render_hsv_sliders(&mut self, ui: &mut egui::Ui) {
        ui.label("HSV:");
        
        let hsv = self.rgb_to_hsv(self.selected_color);
        let mut h = hsv.0;
        let mut s = hsv.1;
        let mut v = hsv.2;

        ui.horizontal(|ui| {
            ui.label("H:");
            if ui.add(egui::Slider::new(&mut h, 0.0..=360.0)).changed() {
                self.update_color_from_hsv(h, s, v);
            }
        });

        ui.horizontal(|ui| {
            ui.label("S:");
            if ui.add(egui::Slider::new(&mut s, 0.0..=1.0)).changed() {
                self.update_color_from_hsv(h, s, v);
            }
        });

        ui.horizontal(|ui| {
            ui.label("V:");
            if ui.add(egui::Slider::new(&mut v, 0.0..=1.0)).changed() {
                self.update_color_from_hsv(h, s, v);
            }
        });
    }

    fn render_alpha_slider(&mut self, ui: &mut egui::Ui) {
        ui.label("Alpha:");
        
        let mut a = self.selected_color.a() as f32;
        if ui.add(egui::Slider::new(&mut a, 0.0..=255.0)).changed() {
            let mut color = self.selected_color;
            color = egui::Color32::from_rgba_unmultiplied(
                color.r(), color.g(), color.b(), a as u8
            );
            self.selected_color = color;
        }
    }

    fn render_hex_input(&mut self, ui: &mut egui::Ui) {
        ui.label("Hex:");
        
        let mut hex = self.color_to_hex(self.selected_color);
        if ui.add(egui::TextEdit::singleline(&mut hex)
            .desired_width(100.0)
            .hint_text("#RRGGBBAA")).changed() {
            if let Ok(color) = self.hex_to_color(&hex) {
                self.selected_color = color;
            }
        }
    }

    fn render_preset_colors(&mut self, ui: &mut egui::Ui) {
        let colors_per_row = 8;
        let color_size = 24.0;
        let spacing = 4.0;

        for (row_index, color_row) in self.preset_colors.chunks(colors_per_row).enumerate() {
            ui.horizontal(|ui| {
                for (col_index, &color) in color_row.iter().enumerate() {
                    let is_selected = *color == self.selected_color;
                    
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(color_size, color_size),
                        egui::Sense::click()
                    );

                    let painter = ui.painter();
                    painter.rect_filled(rect, 2.0, *color);
                    
                    if is_selected {
                        painter.rect_stroke(rect, 2.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                    } else {
                        painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 128, 128)));
                    }

                    if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
                        self.selected_color = *color;
                    }

                    if col_index < colors_per_row - 1 {
                        ui.add_space(spacing);
                    }
                }
            });

            if row_index < (self.preset_colors.len() / colors_per_row) - 1 {
                ui.add_space(spacing);
            }
        }
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("OK").clicked() {
                    if let Some(callback) = &self.on_select {
                        callback(self.selected_color);
                    }
                }
            });

            ui.add_space(10.0);

                if ui.button("Cancel").clicked() {
                    if let Some(callback) = &self.on_cancel {
                        callback();
                    }
                }
            });
        });
    }

    fn update_color_from_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.selected_color = egui::Color32::from_rgba_unmultiplied(r, g, b, self.selected_color.a());
    }

    fn update_color_from_hsv(&mut self, h: f32, s: f32, v: f32) {
        self.selected_color = self.hsv_to_rgb(h, s, v);
    }

    fn rgb_to_hsv(&self, color: egui::Color32) -> (f32, f32, f32) {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let mut h = 0.0;
        let mut s = 0.0;
        let mut v = max;

        if delta != 0.0 {
            if max == r {
                h = (g - b) / delta * 60.0;
                if h < 0.0 {
                    h += 360.0;
                }
            } else if max == g {
                h = (b - r) / delta * 60.0 + 120.0;
                if h < 0.0 {
                    h += 360.0;
                }
            } else if max == b {
                h = (r - g) / delta * 60.0 + 240.0;
                if h < 0.0 {
                    h += 360.0;
                }
            }

            s = if max != 0.0 {
                delta / max
            } else {
                0.0
            };
        }

        (h, s, v)
    }

    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> egui::Color32 {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0 % 2.0 - 1.0).abs()) * 2.0);
        let m = v - c;

        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;

        match (h / 60.0) as usize % 6 {
            0 => {
                r = v;
                g = x + m;
                b = m;
            },
            1 => {
                r = x + m;
                g = v;
                b = m;
            },
            2 => {
                r = m;
                g = v;
                b = x + m;
            },
            3 => {
                r = m;
                g = x + m;
                b = v;
            },
            4 => {
                r = x + m;
                g = m;
                b = v;
            },
            5 => {
                r = v;
                g = m;
                b = x + m;
            },
            _ => unreachable!(),
        }

        egui::Color32::from_rgba_unmultiplied(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            self.selected_color.a()
        )
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

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
            Ok(egui::Color32::from_rgb(r, g, b))
        } else {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| ())?;
            Ok(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
        }
    }
}

impl Default for ColorDialog {
    fn default() -> Self {
        Self::new("default_color_dialog".to_string())
    }
}
