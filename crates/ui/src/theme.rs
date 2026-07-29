use eframe::egui;
use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
  pub name: String,
  pub colors: ThemeColors,
  pub fonts: ThemeFonts,
  pub spacing: ThemeSpacing,
  pub sizing: ThemeSizing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
  pub background: egui::Color32,
  pub foreground: egui::Color32,
  pub primary: egui::Color32,
  pub secondary: egui::Color32,
  pub accent: egui::Color32,
  pub success: egui::Color32,
  pub warning: egui::Color32,
  pub error: egui::Color32,
  pub info: egui::Color32,
  pub border: egui::Color32,
  pub shadow: egui::Color32,
  pub text_primary: egui::Color32,
  pub text_secondary: egui::Color32,
  pub text_disabled: egui::Color32,
  pub panel_background: egui::Color32,
  pub panel_header: egui::Color32,
  pub button_primary: egui::Color32,
  pub button_secondary: egui::Color32,
  pub button_hover: egui::Color32,
  pub button_active: egui::Color32,
  pub input_background: egui::Color32,
  pub input_border: egui::Color32,
  pub input_focused: egui::Color32,
  pub timeline_background: egui::Color32,
  pub timeline_track: egui::Color32,
  pub timeline_playhead: egui::Color32,
  pub waveform_background: egui::Color32,
  pub waveform_foreground: egui::Color32,
  pub spectrogram_background: egui::Color32,
  pub spectrogram_heatmap_low: egui::Color32,
  pub spectrogram_heatmap_high: egui::Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
  pub primary: FontConfig,
  pub monospace: FontConfig,
  pub icon: FontConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
  pub family: String,
  pub size: f32,
  pub weight: String,
  pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
  pub xs: f32,
  pub sm: f32,
  pub md: f32,
  pub lg: f32,
  pub xl: f32,
  pub xxl: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSizing {
  pub button_height: f32,
  pub button_min_width: f32,
  pub input_height: f32,
  pub panel_header_height: f32,
  pub timeline_height: f32,
  pub timeline_track_height: f32,
  pub sidebar_width: f32,
  pub status_bar_height: f32,
  pub menu_bar_height: f32,
  pub border_radius: f32,
  pub border_width: f32,
}

impl Theme {
  pub fn dark() -> Self {
    Self {
      name: "Dark".to_string(),
      colors: ThemeColors {
        background: egui::Color32::from_rgb(24, 24, 24),
        foreground: egui::Color32::from_rgb(240, 240, 240),
        primary: egui::Color32::from_rgb(66, 135, 245),
        secondary: egui::Color32::from_rgb(88, 88, 88),
        accent: egui::Color32::from_rgb(255, 119, 0),
        success: egui::Color32::from_rgb(46, 160, 67),
        warning: egui::Color32::from_rgb(255, 193, 7),
        error: egui::Color32::from_rgb(220, 53, 69),
        info: egui::Color32::from_rgb(23, 162, 184),
        border: egui::Color32::from_rgb(64, 64, 64),
        shadow: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128),
        text_primary: egui::Color32::from_rgb(240, 240, 240),
        text_secondary: egui::Color32::from_rgb(160, 160, 160),
        text_disabled: egui::Color32::from_rgb(88, 88, 88),
        panel_background: egui::Color32::from_rgb(32, 32, 32),
        panel_header: egui::Color32::from_rgb(48, 48, 48),
        button_primary: egui::Color32::from_rgb(66, 135, 245),
        button_secondary: egui::Color32::from_rgb(88, 88, 88),
        button_hover: egui::Color32::from_rgb(86, 153, 255),
        button_active: egui::Color32::from_rgb(46, 115, 225),
        input_background: egui::Color32::from_rgb(40, 40, 40),
        input_border: egui::Color32::from_rgb(64, 64, 64),
        input_focused: egui::Color32::from_rgb(66, 135, 245),
        timeline_background: egui::Color32::from_rgb(20, 20, 20),
        timeline_track: egui::Color32::from_rgb(48, 48, 48),
        timeline_playhead: egui::Color32::from_rgb(255, 69, 0),
        waveform_background: egui::Color32::from_rgb(16, 16, 16),
        waveform_foreground: egui::Color32::from_rgb(0, 255, 127),
        spectrogram_background: egui::Color32::from_rgb(16, 16, 16),
        spectrogram_heatmap_low: egui::Color32::from_rgb(0, 0, 255),
        spectrogram_heatmap_high: egui::Color32::from_rgb(255, 0, 0),
      },
      fonts: ThemeFonts {
        primary: FontConfig {
          family: "Inter".to_string(),
          size: 14.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        monospace: FontConfig {
          family: "JetBrains Mono".to_string(),
          size: 13.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        icon: FontConfig {
          family: "Font Awesome 6 Free".to_string(),
          size: 16.0,
          weight: "900".to_string(),
          style: "normal".to_string(),
        },
      },
      spacing: ThemeSpacing {
        xs: 4.0,
        sm: 8.0,
        md: 16.0,
        lg: 24.0,
        xl: 32.0,
        xxl: 48.0,
      },
      sizing: ThemeSizing {
        button_height: 32.0,
        button_min_width: 80.0,
        input_height: 32.0,
        panel_header_height: 36.0,
        timeline_height: 200.0,
        timeline_track_height: 40.0,
        sidebar_width: 280.0,
        status_bar_height: 24.0,
        menu_bar_height: 28.0,
        border_radius: 4.0,
        border_width: 1.0,
      },
    }
  }

  pub fn light() -> Self {
    Self {
      name: "Light".to_string(),
      colors: ThemeColors {
        background: egui::Color32::from_rgb(255, 255, 255),
        foreground: egui::Color32::from_rgb(33, 33, 33),
        primary: egui::Color32::from_rgb(25, 118, 210),
        secondary: egui::Color32::from_rgb(158, 158, 158),
        accent: egui::Color32::from_rgb(255, 152, 0),
        success: egui::Color32::from_rgb(56, 142, 60),
        warning: egui::Color32::from_rgb(251, 192, 45),
        error: egui::Color32::from_rgb(229, 57, 53),
        info: egui::Color32::from_rgb(2, 136, 209),
        border: egui::Color32::from_rgb(189, 189, 189),
        shadow: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 64),
        text_primary: egui::Color32::from_rgb(33, 33, 33),
        text_secondary: egui::Color32::from_rgb(117, 117, 117),
        text_disabled: egui::Color32::from_rgb(189, 189, 189),
        panel_background: egui::Color32::from_rgb(250, 250, 250),
        panel_header: egui::Color32::from_rgb(245, 245, 245),
        button_primary: egui::Color32::from_rgb(25, 118, 210),
        button_secondary: egui::Color32::from_rgb(158, 158, 158),
        button_hover: egui::Color32::from_rgb(66, 165, 245),
        button_active: egui::Color32::from_rgb(21, 101, 192),
        input_background: egui::Color32::from_rgb(255, 255, 255),
        input_border: egui::Color32::from_rgb(189, 189, 189),
        input_focused: egui::Color32::from_rgb(25, 118, 210),
        timeline_background: egui::Color32::from_rgb(248, 248, 248),
        timeline_track: egui::Color32::from_rgb(245, 245, 245),
        timeline_playhead: egui::Color32::from_rgb(255, 87, 34),
        waveform_background: egui::Color32::from_rgb(250, 250, 250),
        waveform_foreground: egui::Color32::from_rgb(0, 150, 136),
        spectrogram_background: egui::Color32::from_rgb(248, 248, 248),
        spectrogram_heatmap_low: egui::Color32::from_rgb(33, 150, 243),
        spectrogram_heatmap_high: egui::Color32::from_rgb(244, 67, 54),
      },
      fonts: ThemeFonts {
        primary: FontConfig {
          family: "Inter".to_string(),
          size: 14.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        monospace: FontConfig {
          family: "JetBrains Mono".to_string(),
          size: 13.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        icon: FontConfig {
          family: "Font Awesome 6 Free".to_string(),
          size: 16.0,
          weight: "900".to_string(),
          style: "normal".to_string(),
        },
      },
      spacing: ThemeSpacing {
        xs: 4.0,
        sm: 8.0,
        md: 16.0,
        lg: 24.0,
        xl: 32.0,
        xxl: 48.0,
      },
      sizing: ThemeSizing {
        button_height: 32.0,
        button_min_width: 80.0,
        input_height: 32.0,
        panel_header_height: 36.0,
        timeline_height: 200.0,
        timeline_track_height: 40.0,
        sidebar_width: 280.0,
        status_bar_height: 24.0,
        menu_bar_height: 28.0,
        border_radius: 4.0,
        border_width: 1.0,
      },
    }
  }

  pub fn amoled() -> Self {
    Self {
      name: "AMOLED".to_string(),
      colors: ThemeColors {
        background: egui::Color32::from_rgb(0, 0, 0),
        foreground: egui::Color32::from_rgb(255, 255, 255),
        primary: egui::Color32::from_rgb(0, 122, 255),
        secondary: egui::Color32::from_rgb(142, 142, 147),
        accent: egui::Color32::from_rgb(255, 149, 0),
        success: egui::Color32::from_rgb(52, 199, 89),
        warning: egui::Color32::from_rgb(255, 204, 0),
        error: egui::Color32::from_rgb(255, 59, 48),
        info: egui::Color32::from_rgb(10, 132, 255),
        border: egui::Color32::from_rgb(58, 58, 60),
        shadow: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
        text_primary: egui::Color32::from_rgb(255, 255, 255),
        text_secondary: egui::Color32::from_rgb(179, 179, 179),
        text_disabled: egui::Color32::from_rgb(99, 99, 102),
        panel_background: egui::Color32::from_rgb(28, 28, 30),
        panel_header: egui::Color32::from_rgb(44, 44, 46),
        button_primary: egui::Color32::from_rgb(0, 122, 255),
        button_secondary: egui::Color32::from_rgb(142, 142, 147),
        button_hover: egui::Color32::from_rgb(64, 156, 255),
        button_active: egui::Color32::from_rgb(0, 99, 220),
        input_background: egui::Color32::from_rgb(28, 28, 30),
        input_border: egui::Color32::from_rgb(58, 58, 60),
        input_focused: egui::Color32::from_rgb(0, 122, 255),
        timeline_background: egui::Color32::from_rgb(20, 20, 20),
        timeline_track: egui::Color32::from_rgb(44, 44, 46),
        timeline_playhead: egui::Color32::from_rgb(255, 45, 85),
        waveform_background: egui::Color32::from_rgb(10, 10, 10),
        waveform_foreground: egui::Color32::from_rgb(50, 255, 126),
        spectrogram_background: egui::Color32::from_rgb(10, 10, 10),
        spectrogram_heatmap_low: egui::Color32::from_rgb(0, 100, 255),
        spectrogram_heatmap_high: egui::Color32::from_rgb(255, 50, 50),
      },
      fonts: ThemeFonts {
        primary: FontConfig {
          family: "SF Pro Display".to_string(),
          size: 14.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        monospace: FontConfig {
          family: "SF Mono".to_string(),
          size: 13.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
        icon: FontConfig {
          family: "SF Symbols".to_string(),
          size: 16.0,
          weight: "400".to_string(),
          style: "normal".to_string(),
        },
      },
      spacing: ThemeSpacing {
        xs: 4.0,
        sm: 8.0,
        md: 16.0,
        lg: 24.0,
        xl: 32.0,
        xxl: 48.0,
      },
      sizing: ThemeSizing {
        button_height: 32.0,
        button_min_width: 80.0,
        input_height: 32.0,
        panel_header_height: 36.0,
        timeline_height: 200.0,
        timeline_track_height: 40.0,
        sidebar_width: 280.0,
        status_bar_height: 24.0,
        menu_bar_height: 28.0,
        border_radius: 6.0,
        border_width: 0.5,
      },
    }
  }

  pub fn to_egui_visuals(&self) -> egui::Visuals {
    egui::Visuals {
      window_fill: self.colors.background,
      panel_fill: self.colors.panel_background,
      dark_mode: self.name != "Light",
      override_text_color: Some(self.colors.text_primary),
      error_fg_color: self.colors.error,
      warn_fg_color: self.colors.warning,
      info_fg_color: self.colors.info,
      debug_fg_color: self.colors.info,
      hyperlink_color: self.colors.primary,
      faint_bg_color: self.colors.background.linear_multiply(0.8),
      extreme_bg_color: self.colors.background.linear_multiply(0.6),
      code_bg_color: self.colors.input_background,
      window_shadow: egui::Shadow {
        offset: egui::vec2(0.0, 2.0),
        blur: 8.0,
        spread: 0.0,
        color: self.colors.shadow,
      },
      popup_shadow: egui::Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 12.0,
        spread: 0.0,
        color: self.colors.shadow,
      },
      resize_corner_size: 12.0,
      text_cursor: egui::Stroke::new(2.0, self.colors.primary),
      text_cursor_preview: false,
      clip_rect_margin: 3.0,
      button_frame: true,
      collapsing_header_frame: true,
      indent_has_left_vline: true,
      striped: true,
      slider_trailing_fill: true,
      handle_shape: egui::MarkerShape::Circle,
      menu_rounding: egui::Rounding::same(self.sizing.border_radius),
      window_rounding: egui::Rounding::same(self.sizing.border_radius),
      window_shadow: egui::Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: self.colors.shadow,
      },
      popup_rounding: egui::Rounding::same(self.sizing.border_radius),
      selection: egui::Stroke::new(1.0, self.colors.primary),
      nav_highlight: egui::Stroke::new(1.0, self.colors.primary),
      window_fill: self.colors.background,
      panel_fill: self.colors.panel_background,
      ..Default::default()
    }
  }

  pub fn get_spacing(&self, size: &str) -> f32 {
    match size {
      "xs" => self.spacing.xs,
      "sm" => self.spacing.sm,
      "md" => self.spacing.md,
      "lg" => self.spacing.lg,
      "xl" => self.spacing.xl,
      "xxl" => self.spacing.xxl,
      _ => self.spacing.md,
    }
  }

  pub fn get_font(&self, font_type: &str) -> &FontConfig {
    match font_type {
      "monospace" => &self.fonts.monospace,
      "icon" => &self.fonts.icon,
      _ => &self.fonts.primary,
    }
  }

  pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(self)?;
    std::fs::write(path, json)?;
    Ok(())
  }

  pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(path)?;
    let theme: Self = serde_json::from_str(&json)?;
    Ok(theme)
  }
}

impl Default for Theme {
  fn default() -> Self {
    Self::dark()
  }
}
