use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ColorManipulationEffect {
    pub id: String,
    pub name: String,
    pub manipulation_type: Arc<RwLock<ManipulationType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<ColorManipulationEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ColorManipulationEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManipulationType {
    HueShift,
    Saturation,
    Brightness,
    Contrast,
    Gamma,
    ColorBalance,
    ColorMatrix,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ColorManipulationEvent {
    ManipulationStarted,
    ManipulationProgress(f32),
    ManipulationCompleted(ColorManipulationResult),
    Error(String),
    FrameProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct ColorManipulationResult {
    pub success: bool,
    pub manipulation_type: ManipulationType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColorManipulationConfig {
    pub manipulation_type: ManipulationType,
    pub hue_shift: f32,
    pub saturation_adjustment: f32,
    pub brightness_adjustment: f32,
    pub contrast_adjustment: f32,
    pub gamma_correction: f32,
    pub color_balance: ColorBalance,
    pub color_matrix: ColorMatrix,
    pub preserve_metadata: bool,
    pub output_format: super::databend::OutputFormat,
}

#[derive(Debug, Clone)]
pub struct ColorBalance {
    pub red_adjustment: f32,
    pub green_adjustment: f32,
    pub blue_adjustment: f32,
    pub cyan_adjustment: f32,
    pub magenta_adjustment: f32,
    pub yellow_adjustment: f32,
}

#[derive(Debug, Clone)]
pub struct ColorMatrix {
    pub matrix: [[f32; 3]; 3],3x3 matrix for RGB
    pub offset: [f32; 3],
}

impl ColorManipulationEffect {
    pub fn new(id: String, name: String, manipulation_type: ManipulationType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            manipulation_type: Arc::new(RwLock::new(manipulation_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_data: &[u8], config: ColorManipulationConfig) -> Result<ColorManipulationResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ColorManipulationEvent::ManipulationStarted);
        let start_time = std::time::Instant::now();

        let result = match config.manipulation_type {
            ManipulationType::HueShift => self.apply_hue_shift(input_data, &config).await,
            ManipulationType::Saturation => self.apply_saturation(input_data, &config).await,
            ManipulationType::Brightness => self.apply_brightness(input_data, &config).await,
            ManipulationType::Contrast => self.apply_contrast(input_data, &config).await,
            ManipulationType::Gamma => self.apply_gamma(input_data, &config).await,
            ManipulationType::ColorBalance => self.apply_color_balance(input_data, &config).await,
            ManipulationType::ColorMatrix => self.apply_color_matrix(input_data, &config).await,
            ManipulationType::Custom(_) => self.apply_custom_manipulation(input_data, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(ColorManipulationEvent::ManipulationCompleted(ColorManipulationResult {
                    success: true,
                    manipulation_type: config.manipulation_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(ColorManipulationResult {
                    success: true,
                    manipulation_type: config.manipulation_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Color manipulation effect failed: {}", e);
                let _ = self.event_sender.send(ColorManipulationEvent::Error(error_msg.clone()));

                Ok(ColorManipulationResult {
                    success: false,
                    manipulation_type: config.manipulation_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_hue_shift(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                let rgb = self.rgb_to_hsv(&output_data[i], &output_data[i + 1], &output_data[i + 2]);
                let mut hsv = rgb;
                hsv.0 = (hsv.0 + config.hue_shift) % 360.0;
                
                let new_rgb = self.hsv_to_rgb(&hsv.0, &hsv.1, &hsv.2);
                output_data[i] = new_rgb.0;
                output_data[i + 1] = new_rgb.1;
                output_data[i + 2] = new_rgb.2;
            }
        }

        Ok(output_data)
    }

    async fn apply_saturation(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                let rgb = self.rgb_to_hsv(&output_data[i], &output_data[i + 1], &output_data[i + 2]);
                let mut hsv = rgb;
                hsv.1 = (hsv.1 * config.saturation_adjustment).clamp(0.0, 1.0);
                
                let new_rgb = self.hsv_to_rgb(&hsv.0, &hsv.1, &hsv.2);
                output_data[i] = new_rgb.0;
                output_data[i + 1] = new_rgb.1;
                output_data[i + 2] = new_rgb.2;
            }
        }

        Ok(output_data)
    }

    async fn apply_brightness(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                output_data[i] = (output_data[i] as f32 * config.brightness_adjustment).clamp(0.0, 255.0) as u8;
                output_data[i + 1] = (output_data[i + 1] as f32 * config.brightness_adjustment).clamp(0.0, 255.0) as u8;
                output_data[i + 2] = (output_data[i + 2] as f32 * config.brightness_adjustment).clamp(0.0, 255.0) as u8;
            }
        }

        Ok(output_data)
    }

    async fn apply_contrast(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                output_data[i] = ((output_data[i] as f32 - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0) as u8;
                output_data[i + 1] = ((output_data[i + 1] as f32 - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0) as u8;
                output_data[i + 2] = ((output_data[i + 2] as f32 - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0) as u8;
            }
        }

        Ok(output_data)
    }

    async fn apply_gamma(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                output_data[i] = ((output_data[i] as f32 / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0) as u8;
                output_data[i + 1] = ((output_data[i + 1] as f32 / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0) as u8;
                output_data[i + 2] = ((output_data[i + 2] as f32 / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0) as u8;
            }
        }

        Ok(output_data)
    }

    async fn apply_color_balance(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                let r = output_data[i] as f32;
                let g = output_data[i + 1] as f32;
                let b = output_data[i + 2] as f32;
                
                let new_r = (r + config.color_balance.red_adjustment * 255.0 / 100.0).clamp(0.0, 255.0);
                let new_g = (g + config.color_balance.green_adjustment * 255.0 / 100.0).clamp(0.0, 255.0);
                let new_b = (b + config.color_balance.blue_adjustment * 255.0 / 100.0).clamp(0.0, 255.0);
                
                output_data[i] = new_r as u8;
                output_data[i + 1] = new_g as u8;
                output_data[i + 2] = new_b as u8;
            }
        }

        Ok(output_data)
    }

    async fn apply_color_matrix(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                let r = output_data[i] as f32;
                let g = output_data[i + 1] as f32;
                let b = output_data[i + 2] as f32;
                
                let new_r = (config.color_matrix.matrix[0][0] * r + 
                               config.color_matrix.matrix[0][1] * g + 
                               config.color_matrix.matrix[0][2] * b + 
                               config.color_matrix.offset[0]).clamp(0.0, 255.0);
                
                let new_g = (config.color_matrix.matrix[1][0] * r + 
                               config.color_matrix.matrix[1][1] * g + 
                               config.color_matrix.matrix[1][2] * b + 
                               config.color_matrix.offset[1]).clamp(0.0, 255.0);
                
                let new_b = (config.color_matrix.matrix[2][0] * r + 
                               config.color_matrix.matrix[2][1] * g + 
                               config.color_matrix.matrix[2][2] * b + 
                               config.color_matrix.offset[2]).clamp(0.0, 255.0);
                
                output_data[i] = new_r as u8;
                output_data[i + 1] = new_g as u8;
                output_data[i + 2] = new_b as u8;
            }
        }

        Ok(output_data)
    }

    async fn apply_custom_manipulation(&self, input_data: &[u8], config: &ColorManipulationConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in (0..data_len).step_by(4) {
            if i + 3 < data_len {
                let r = output_data[i] as f32;
                let g = output_data[i + 1] as f32;
                let b = output_data[i + 2] as f32;
                
                let mut new_r = r;
                let mut new_g = g;
                let mut new_b = b;
                
                new_r = (new_r * config.brightness_adjustment).clamp(0.0, 255.0);
                new_g = (new_g * config.brightness_adjustment).clamp(0.0, 255.0);
                new_b = (new_b * config.brightness_adjustment).clamp(0.0, 255.0);
                
                new_r = ((new_r - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0);
                new_g = ((new_g - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0);
                new_b = ((new_b - 128.0) * config.contrast_adjustment + 128.0).clamp(0.0, 255.0);
                
                new_r = ((new_r / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0);
                new_g = ((new_g / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0);
                new_b = ((new_b / 255.0).powf(1.0 / config.gamma_correction) * 255.0).clamp(0.0, 255.0);
                
                output_data[i] = new_r as u8;
                output_data[i + 1] = new_g as u8;
                output_data[i + 2] = new_b as u8;
            }
        }

        Ok(output_data)
    }

    fn rgb_to_hsv(&self, r: &u8, g: &u8, b: &u8) -> (f32, f32, f32) {
        let r_f = *r as f32 / 255.0;
        let g_f = *g as f32 / 255.0;
        let b_f = *b as f32 / 255.0;
        
        let max = r_f.max(g_f).max(b_f);
        let min = r_f.min(g_f).min(b_f);
        let delta = max - min;
        
        let hue = if delta == 0.0 {
            0.0
        } else if max == r_f {
            ((g_f - b_f) / delta).rem_euclid(6.0) * 60.0
        } else if max == g_f {
            ((b_f - r_f) / delta + 2.0) * 60.0
        } else {
            ((r_f - g_f) / delta + 4.0) * 60.0
        };
        
        let saturation = if max == 0.0 {
            0.0
        } else {
            delta / max
        };
        
        let value = max;
        
        (hue, saturation, value)
    }

    fn hsv_to_rgb(&self, h: &f32, s: &f32, v: &f32) -> (u8, u8, u8) {
        let h = *h / 60.0;
        let s = *s;
        let v = *v;
        
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        let r_f = ((r + m) * 255.0).clamp(0.0, 255.0) as u8;
        let g_f = ((g + m) * 255.0).clamp(0.0, 255.0) as u8;
        let b_f = ((b + m) * 255.0).clamp(0.0, 255.0) as u8;
        
        (r_f, g_f, b_f)
    }

    fn generate_metadata(&self, config: &ColorManipulationConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("manipulation_type".to_string(), format!("{:?}", config.manipulation_type));
        metadata.insert("hue_shift".to_string(), format!("{:.2}", config.hue_shift));
        metadata.insert("saturation_adjustment".to_string(), format!("{:.2}", config.saturation_adjustment));
        metadata.insert("brightness_adjustment".to_string(), format!("{:.2}", config.brightness_adjustment));
        metadata.insert("contrast_adjustment".to_string(), format!("{:.2}", config.contrast_adjustment));
        metadata.insert("gamma_correction".to_string(), format!("{:.2}", config.gamma_correction));
        metadata.insert("preserve_metadata".to_string(), config.preserve_metadata.to_string());
        metadata.insert("output_format".to_string(), format!("{:?}", config.output_format));
        
        metadata.insert("red_adjustment".to_string(), format!("{:.2}", config.color_balance.red_adjustment));
        metadata.insert("green_adjustment".to_string(), format!("{:.2}", config.color_balance.green_adjustment));
        metadata.insert("blue_adjustment".to_string(), format!("{:.2}", config.color_balance.blue_adjustment));
        metadata.insert("cyan_adjustment".to_string(), format!("{:.2}", config.color_balance.cyan_adjustment));
        metadata.insert("magenta_adjustment".to_string(), format!("{:.2}", config.color_balance.magenta_adjustment));
        metadata.insert("yellow_adjustment".to_string(), format!("{:.2}", config.color_balance.yellow_adjustment));
        
        for i in 0..3 {
            for j in 0..3 {
                metadata.insert(format!("matrix_{}_{}", i, j), format!("{:.3}", config.color_matrix.matrix[i][j]));
            }
            metadata.insert(format!("offset_{}", i), format!("{:.3}", config.color_matrix.offset[i]));
        }
        
        metadata
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub fn set_manipulation_type(&self, manipulation_type: ManipulationType) {
        let mut current_type = self.manipulation_type.write();
        *current_type = manipulation_type;
    }

    pub fn get_manipulation_type(&self) -> ManipulationType {
        self.manipulation_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<ColorManipulationEvent> {
        let mut receiver = self.event_receiver.write();
        if let Some(ref mut rx) = *receiver {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
            events
        } else {
            Vec::new()
        }
    }

    pub fn get_supported_manipulation_types(&self) -> Vec<ManipulationType> {
        vec![
            ManipulationType::HueShift,
            ManipulationType::Saturation,
            ManipulationType::Brightness,
            ManipulationType::Contrast,
            ManipulationType::Gamma,
            ManipulationType::ColorBalance,
            ManipulationType::ColorMatrix,
        ]
    }

    pub fn can_apply_manipulation_type(&self, manipulation_type: &ManipulationType) -> bool {
        self.get_supported_manipulation_types().contains(manipulation_type)
    }

    pub fn clone_effect(&self) -> ColorManipulationEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_manipulation_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, input_size: usize, config: &ColorManipulationConfig) -> std::time::Duration {
        let base_time_ms = match config.manipulation_type {
            ManipulationType::HueShift => 5.0,
            ManipulationType::Saturation => 3.0,
            ManipulationType::Brightness => 2.0,
            ManipulationType::Contrast => 3.0,
            ManipulationType::Gamma => 4.0,
            ManipulationType::ColorBalance => 6.0,
            ManipulationType::ColorMatrix => 8.0,
            ManipulationType::Custom(_) => 7.0,
        };

        let time_per_pixel = base_time_ms / 1000.0;
        let total_time = input_size as f64 * time_per_pixel;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> ColorManipulationConfig {
        match preset_name {
            "vintage" => ColorManipulationConfig {
                manipulation_type: self.get_manipulation_type(),
                hue_shift: 15.0,
                saturation_adjustment: 0.8,
                brightness_adjustment: 1.1,
                contrast_adjustment: 0.9,
                gamma_correction: 1.2,
                color_balance: ColorBalance::default(),
                color_matrix: ColorMatrix::default(),
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "vivid" => ColorManipulationConfig {
                manipulation_type: self.get_manipulation_type(),
                hue_shift: 0.0,
                saturation_adjustment: 1.3,
                brightness_adjustment: 1.0,
                contrast_adjustment: 1.1,
                gamma_correction: 0.9,
                color_balance: ColorBalance::default(),
                color_matrix: ColorMatrix::default(),
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "dramatic" => ColorManipulationConfig {
                manipulation_type: self.get_manipulation_type(),
                hue_shift: -10.0,
                saturation_adjustment: 1.2,
                brightness_adjustment: 0.8,
                contrast_adjustment: 1.4,
                gamma_correction: 0.8,
                color_balance: ColorBalance::default(),
                color_matrix: ColorMatrix::default(),
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "black_and_white" => ColorManipulationConfig {
                manipulation_type: ManipulationType::Saturation,
                hue_shift: 0.0,
                saturation_adjustment: 0.0,
                brightness_adjustment: 1.0,
                contrast_adjustment: 1.0,
                gamma_correction: 1.0,
                color_balance: ColorBalance::default(),
                color_matrix: ColorMatrix::default(),
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            _ => ColorManipulationConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "vintage".to_string(),
            "vivid".to_string(),
            "dramatic".to_string(),
            "black_and_white".to_string(),
        ]
    }
}

impl Default for ColorManipulationEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Color Manipulation Effect".to_string(),
            ManipulationType::Brightness,
        )
    }
}

impl Default for ManipulationType {
    fn default() -> Self {
        ManipulationType::Brightness
    }
}

impl Default for ColorManipulationEvent {
    fn default() -> Self {
        ColorManipulationEvent::ManipulationStarted
    }
}

impl Default for ColorManipulationResult {
    fn default() -> Self {
        Self {
            success: false,
            manipulation_type: ManipulationType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for ColorManipulationConfig {
    fn default() -> Self {
        Self {
            manipulation_type: ManipulationType::default(),
            hue_shift: 0.0,
            saturation_adjustment: 1.0,
            brightness_adjustment: 1.0,
            contrast_adjustment: 1.0,
            gamma_correction: 1.0,
            color_balance: ColorBalance::default(),
            color_matrix: ColorMatrix::default(),
            preserve_metadata: true,
            output_format: super::databend::OutputFormat::Png,
        }
    }
}

impl Default for ColorBalance {
    fn default() -> Self {
        Self {
            red_adjustment: 0.0,
            green_adjustment: 0.0,
            blue_adjustment: 0.0,
            cyan_adjustment: 0.0,
            magenta_adjustment: 0.0,
            yellow_adjustment: 0.0,
        }
    }
}

impl Default for ColorMatrix {
    fn default() -> Self {
        Self {
            matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            offset: [0.0, 0.0, 0.0],
        }
    }
}
