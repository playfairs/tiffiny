use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BitCrushEffect {
    pub id: String,
    pub name: String,
    pub crush_type: Arc<RwLock<CrushType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<BitCrushEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<BitCrushEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrushType {
    BitReduction,
    SampleRate,
    BitDepth,
    Dithering,
    Posterization,
    Quantization,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum BitCrushEvent {
    CrushStarted,
    CrushProgress(f32),
    CrushCompleted(BitCrushResult),
    Error(String),
    FrameProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct BitCrushResult {
    pub success: bool,
    pub crush_type: CrushType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BitCrushConfig {
    pub crush_type: CrushType,
    pub bit_depth: u8,
    pub sample_rate: u32,
    pub dithering_type: DitheringType,
    pub quantization_levels: u8,
    pub posterization_levels: u8,
    pub color_mode: ColorMode,
    pub preserve_highlights: bool,
    pub preserve_shadows: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DitheringType {
    None,
    FloydSteinberg,
    Ordered,
    Random,
    Pattern,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorMode {
    RGB,
    Grayscale,
    Sepia,
    Custom(String),
}

impl BitCrushEffect {
    pub fn new(id: String, name: String, crush_type: CrushType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            crush_type: Arc::new(RwLock::new(crush_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_data: &[u8], config: BitCrushConfig) -> Result<BitCrushResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(BitCrushEvent::CrushStarted);
        let start_time = std::time::Instant::now();

        let result = match config.crush_type {
            CrushType::BitReduction => self.apply_bit_reduction(input_data, &config).await,
            CrushType::SampleRate => self.apply_sample_rate_reduction(input_data, &config).await,
            CrushType::BitDepth => self.apply_bit_depth_reduction(input_data, &config).await,
            CrushType::Dithering => self.apply_dithering(input_data, &config).await,
            CrushType::Posterization => self.apply_posterization(input_data, &config).await,
            CrushType::Quantization => self.apply_quantization(input_data, &config).await,
            CrushType::Custom(_) => self.apply_custom_crush(input_data, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let _ = self.event_sender.send(BitCrushEvent::CrushCompleted(BitCrushResult {
                    success: true,
                    crush_type: config.crush_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                }));

                Ok(BitCrushResult {
                    success: true,
                    crush_type: config.crush_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Bit crush effect failed: {}", e);
                let _ = self.event_sender.send(BitCrushEvent::Error(error_msg.clone()));

                Ok(BitCrushResult {
                    success: false,
                    crush_type: config.crush_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_bit_reduction(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
Apply bit reduction
        let bit_mask = (1 << config.bit_depth) - 1;
        
        for i in 0..config.bit_depth {
            let progress = (i as f32 / config.bit_depth as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for byte in output_data.iter_mut() {
                *byte &= bit_mask;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output_data)
    }

    async fn apply_sample_rate_reduction(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        let reduction_factor = (44100 / config.sample_rate) as usize;
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for j in (0..data_len).step_by(reduction_factor) {
                if j < output_data.len() {
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output_data)
    }

    async fn apply_bit_depth_reduction(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        let shift_amount = 8 - config.bit_depth;
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for byte in output_data.iter_mut() {
                *byte = *byte >> shift_amount;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(8)).await;
        }

        Ok(output_data)
    }

    async fn apply_dithering(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        match config.dithering_type {
            DitheringType::FloydSteinberg => {
                self.apply_floyd_steinberg_dithering(&mut output_data, config).await?;
            },
            DitheringType::Ordered => {
                self.apply_ordered_dithering(&mut output_data, config).await?;
            },
            DitheringType::Random => {
                self.apply_random_dithering(&mut output_data, config).await?;
            },
            DitheringType::Pattern => {
                self.apply_pattern_dithering(&mut output_data, config).await?;
            },
            DitheringType::Custom(_) => {
                self.apply_custom_dithering(&mut output_data, config).await?;
            },
            DitheringType::None => {
            },
        }

        Ok(output_data)
    }

    async fn apply_floyd_steinberg_dithering(&self, data: &mut [u8], config: &BitCrushConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        
        for y in 0..height {
            for x in 0..width {
                let pixel_start = (y * width + x) * 4;
                if pixel_start + 3 < data.len() {
                    let old_value = data[pixel_start] as f32;
                    
                    let new_value = (old_value / 255.0 * (config.quantization_levels - 1) as f32).round() as u8;
                    
                    let error = old_value - new_value as f32;
                    
                    data[pixel_start] = new_value;
                    
                    if x + 1 < width && pixel_start + 4 < data.len() {
                        data[pixel_start + 4] = (data[pixel_start + 4] as f32 + error * 0.4375).clamp(0.0, 255.0) as u8;
                    }
                    if y + 1 < height {
                        let next_row_start = ((y + 1) * width + x) * 4;
                        if next_row_start + 3 < data.len() {
                            data[next_row_start + 3] = (data[next_row_start + 3] as f32 + error * 0.1875).clamp(0.0, 255.0) as u8;
                        }
                        if x > 0 && next_row_start - 1 < data.len() {
                            data[next_row_start - 1] = (data[next_row_start - 1] as f32 + error * 0.3125).clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        Ok(())
    }

    async fn apply_ordered_dithering(&self, data: &mut [u8], config: &BitCrushConfig) -> Result<(), Box<dyn std::error::Error>> {
        let data_len = data.len();
        
        for i in 0..data_len {
            let progress = (i as f32 / data_len as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            let x = i % 8;
            let y = i / 8;
            let threshold = ((x + y) % 8) as f32 / 8.0 * 255.0;
            
            if data[i] as f32 > threshold {
                data[i] = 255;
            } else {
                data[i] = 0;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(())
    }

    async fn apply_random_dithering(&self, data: &mut [u8], config: &BitCrushConfig) -> Result<(), Box<dyn std::error::Error>> {
        let data_len = data.len();
        
        for i in 0..data_len {
            let progress = (i as f32 / data_len as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            let noise = (rand::random::<f32>() - 0.5) * 50.0;
            let new_value = (data[i] as f32 + noise).clamp(0.0, 255.0) as u8;
            data[i] = new_value;
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(())
    }

    async fn apply_pattern_dithering(&self, data: &mut [u8], config: &BitCrushConfig) -> Result<(), Box<dyn std::error::Error>> {
        let data_len = data.len();
        let pattern = [0, 128, 32, 160, 48, 176, 80, 208];
        
        for i in 0..data_len {
            let progress = (i as f32 / data_len as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            let pattern_value = pattern[i % 8];
            if data[i] as f32 > pattern_value as f32 {
                data[i] = 255;
            } else {
                data[i] = 0;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(())
    }

    async fn apply_custom_dithering(&self, data: &mut [u8], config: &BitCrushConfig) -> Result<(), Box<dyn std::error::Error>> {
        let data_len = data.len();
        
        for i in 0..data_len {
            let progress = (i as f32 / data_len as f32) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            let threshold = ((i as f32 * 7.0) % 255.0) as u8;
            data[i] = if data[i] > threshold { 255 } else { 0 };
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(())
    }

    async fn apply_posterization(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        let step_size = 256 / config.posterization_levels;
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for byte in output_data.iter_mut() {
                *byte = (*byte / step_size) * step_size;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output_data)
    }

    async fn apply_quantization(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        let step_size = 256 / config.quantization_levels;
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for byte in output_data.iter_mut() {
                *byte = (*byte / step_size) * step_size + step_size / 2;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(8)).await;
        }

        Ok(output_data)
    }

    async fn apply_custom_crush(&self, input_data: &[u8], config: &BitCrushConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(BitCrushEvent::CrushProgress(progress));
            
            for byte in output_data.iter_mut() {
                *byte = (*byte as f32 * 0.7).round() as u8;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        }

        Ok(output_data)
    }

    fn generate_metadata(&self, config: &BitCrushConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("crush_type".to_string(), format!("{:?}", config.crush_type));
        metadata.insert("bit_depth".to_string(), config.bit_depth.to_string());
        metadata.insert("sample_rate".to_string(), config.sample_rate.to_string());
        metadata.insert("dithering_type".to_string(), format!("{:?}", config.dithering_type));
        metadata.insert("quantization_levels".to_string(), config.quantization_levels.to_string());
        metadata.insert("posterization_levels".to_string(), config.posterization_levels.to_string());
        metadata.insert("color_mode".to_string(), format!("{:?}", config.color_mode));
        metadata.insert("preserve_highlights".to_string(), config.preserve_highlights.to_string());
        metadata.insert("preserve_shadows".to_string(), config.preserve_shadows.to_string());
        
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

    pub fn set_crush_type(&self, crush_type: CrushType) {
        let mut current_type = self.crush_type.write();
        *current_type = crush_type;
    }

    pub fn get_crush_type(&self) -> CrushType {
        self.crush_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<BitCrushEvent> {
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

    pub fn get_supported_crush_types(&self) -> Vec<CrushType> {
        vec![
            CrushType::BitReduction,
            CrushType::SampleRate,
            CrushType::BitDepth,
            CrushType::Dithering,
            CrushType::Posterization,
            CrushType::Quantization,
        ]
    }

    pub fn can_apply_crush_type(&self, crush_type: &CrushType) -> bool {
        self.get_supported_crush_types().contains(crush_type)
    }

    pub fn clone_effect(&self) -> BitCrushEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_crush_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, input_size: usize, config: &BitCrushConfig) -> std::time::Duration {
        let base_time_ms = match config.crush_type {
            CrushType::BitReduction => 50.0,
            CrushType::SampleRate => 100.0,
            CrushType::BitDepth => 80.0,
            CrushType::Dithering => 120.0,
            CrushType::Posterization => 60.0,
            CrushType::Quantization => 70.0,
            CrushType::Custom(_) => 90.0,
        };

        let time_per_byte = base_time_ms / 1000.0;
        let total_time = input_size as f64 * time_per_byte;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> BitCrushConfig {
        match preset_name {
            "8bit" => BitCrushConfig {
                crush_type: CrushType::BitDepth,
                bit_depth: 8,
                sample_rate: 44100,
                dithering_type: DitheringType::None,
                quantization_levels: 256,
                posterization_levels: 256,
                color_mode: ColorMode::RGB,
                preserve_highlights: true,
                preserve_shadows: true,
            },
            "4bit" => BitCrushConfig {
                crush_type: CrushType::BitDepth,
                bit_depth: 4,
                sample_rate: 22050,
                dithering_type: DitheringType::FloydSteinberg,
                quantization_levels: 16,
                posterization_levels: 16,
                color_mode: ColorMode::Grayscale,
                preserve_highlights: false,
                preserve_shadows: false,
            },
            "1bit" => BitCrushConfig {
                crush_type: CrushType::BitDepth,
                bit_depth: 1,
                sample_rate: 11025,
                dithering_type: DitheringType::FloydSteinberg,
                quantization_levels: 2,
                posterization_levels: 2,
                color_mode: ColorMode::Grayscale,
                preserve_highlights: false,
                preserve_shadows: false,
            },
            "retro" => BitCrushConfig {
                crush_type: CrushType::BitReduction,
                bit_depth: 6,
                sample_rate: 16000,
                dithering_type: DitheringType::Ordered,
                quantization_levels: 64,
                posterization_levels: 32,
                color_mode: ColorMode::Sepia,
                preserve_highlights: true,
                preserve_shadows: false,
            },
            _ => BitCrushConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "8bit".to_string(),
            "4bit".to_string(),
            "1bit".to_string(),
            "retro".to_string(),
        ]
    }
}

impl Default for BitCrushEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Bit Crush Effect".to_string(),
            CrushType::BitDepth,
        )
    }
}

impl Default for CrushType {
    fn default() -> Self {
        CrushType::BitDepth
    }
}

impl Default for BitCrushEvent {
    fn default() -> Self {
        BitCrushEvent::CrushStarted
    }
}

impl Default for BitCrushResult {
    fn default() -> Self {
        Self {
            success: false,
            crush_type: CrushType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for BitCrushConfig {
    fn default() -> Self {
        Self {
            crush_type: CrushType::default(),
            bit_depth: 8,
            sample_rate: 44100,
            dithering_type: DitheringType::None,
            quantization_levels: 256,
            posterization_levels: 256,
            color_mode: ColorMode::RGB,
            preserve_highlights: true,
            preserve_shadows: true,
        }
    }
}

impl Default for DitheringType {
    fn default() -> Self {
        DitheringType::None
    }
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::RGB
    }
}
