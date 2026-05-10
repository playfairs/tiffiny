use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct DatabendEffect {
    pub id: String,
    pub name: String,
    pub effect_type: Arc<RwLock<DatabendType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<DatabendEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<DatabendEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabendType {
    PixelSort,
    BitCrush,
    DataMosh,
    Glitch,
    Corruption,
    BitShift,
    ByteManipulation,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum DatabendEvent {
    EffectStarted,
    EffectProgress(f32),
    EffectCompleted(DatabendResult),
    Error(String),
    FrameProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct DatabendResult {
    pub success: bool,
    pub effect_type: DatabendType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DatabendConfig {
    pub effect_type: DatabendType,
    pub intensity: f32,
    pub seed: Option<u32>,
    pub iterations: u32,
    pub preserve_metadata: bool,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    Raw,
    Custom(String),
}

impl DatabendEffect {
    pub fn new(id: String, name: String, effect_type: DatabendType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            effect_type: Arc::new(RwLock::new(effect_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn apply(&self, input_data: &[u8], config: DatabendConfig) -> Result<DatabendResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(DatabendEvent::EffectStarted);
        let start_time = std::time::Instant::now();

        let result = match config.effect_type {
            DatabendType::PixelSort => self.apply_pixel_sort(input_data, &config).await,
            DatabendType::BitCrush => self.apply_bit_crush(input_data, &config).await,
            DatabendType::DataMosh => self.apply_data_mosh(input_data, &config).await,
            DatabendType::Glitch => self.apply_glitch(input_data, &config).await,
            DatabendType::Corruption => self.apply_corruption(input_data, &config).await,
            DatabendType::BitShift => self.apply_bit_shift(input_data, &config).await,
            DatabendType::ByteManipulation => self.apply_byte_manipulation(input_data, &config).await,
            DatabendType::Custom(_) => self.apply_custom(input_data, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let _ = self.event_sender.send(DatabendEvent::EffectCompleted(DatabendResult {
                    success: true,
                    effect_type: config.effect_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                }));

                Ok(DatabendResult {
                    success: true,
                    effect_type: config.effect_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Databend effect failed: {}", e);
                let _ = self.event_sender.send(DatabendEvent::Error(error_msg.clone()));

                Ok(DatabendResult {
                    success: false,
                    effect_type: config.effect_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_pixel_sort(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
Simulate pixel sort effect
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            output_data.sort_by(|a, b| {
                let a_sum = a.iter().take(3).sum();
                let b_sum = b.iter().take(3).sum();
                a_sum.cmp(&b_sum)
            });

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output_data)
    }

    async fn apply_bit_crush(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            for byte in output_data.iter_mut() {
                if rand::random::<f32>() < config.intensity {
                    let bit_to_flip = rand::random::<u8>() % 8;
                    *byte ^= 1 << bit_to_flip;
                }
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        Ok(output_data)
    }

    async fn apply_data_mosh(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            let chunk_size = (data_len / config.iterations as usize).max(1);
            let start_pos = (i * chunk_size) % data_len;
            let end_pos = (start_pos + chunk_size).min(data_len);
            
            if start_pos < data_len && end_pos <= data_len {
                let chunk = output_data[start_pos..end_pos].to_vec();
                let modified_chunk = self.mosh_chunk(&chunk, config.intensity);
                
                for (j, byte) in modified_chunk.iter().enumerate() {
                    if start_pos + j < output_data.len() {
                        output_data[start_pos + j] = *byte;
                    }
                }
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(output_data)
    }

    fn mosh_chunk(&self, chunk: &[u8], intensity: f32) -> Vec<u8> {
        let mut output = chunk.to_vec();
        
        for byte in output.iter_mut() {
            if rand::random::<f32>() < intensity {
                *byte = self.apply_byte_mosh(*byte);
            }
        }

        output
    }

    fn apply_byte_mosh(&self, byte: u8) -> u8 {
        let mut result = byte;
        
        if rand::random::<f32>() < 0.5 {
            result ^= rand::random::<u8>();
        }
        
        if rand::random::<f32>() < 0.3 {
            result = result.rotate_left(1);
        }
        
        if rand::random::<f32>() < 0.4 {
            let delta = rand::random::<i8>();
            result = result.wrapping_add(delta);
        }
        
        result
    }

    async fn apply_glitch(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            let glitch_count = (data_len as f32 * config.intensity / 100.0) as usize;
            
            for _ in 0..glitch_count {
                let position = rand::random::<usize>() % data_len;
                if position < output_data.len() {
                    output_data[position] = rand::random::<u8>();
                }
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output_data)
    }

    async fn apply_corruption(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            let corruption_count = (data_len as f32 * config.intensity / 100.0) as usize;
            
            for _ in 0..corruption_count {
                let position = rand::random::<usize>() % data_len;
                if position < output_data.len() {
                    let corruption_type = rand::random::<u8>() % 4;
                    
                    match corruption_type {
                        0 => {
                            output_data[position] = rand::random::<u8>();
                        },
                        1 => {
                            output_data[position] ^= 0xFF;
                        },
                        2 => {
                            output_data[position] = 0;
                        },
                        3 => {
                            output_data[position] = output_data[position].wrapping_add(rand::random::<i8>());
                        },
                        _ => {}
                    }
                }
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(8)).await;
        }

        Ok(output_data)
    }

    async fn apply_bit_shift(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            let shift_amount = (config.intensity * 8.0) as u8;
            
            for byte in output_data.iter_mut() {
                *byte = byte.rotate_left(shift_amount);
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        }

        Ok(output_data)
    }

    async fn apply_byte_manipulation(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..config.iterations {
            for j in 0..data_len {
                let byte_index = (i * data_len + j) % data_len;
                
                if byte_index < output_data.len() {
                    let manipulation_type = rand::random::<u8>() % 6;
                    
                    match manipulation_type {
                        0 => {
                            output_data[byte_index] = !output_data[byte_index];
                        },
                        1 => {
                            output_data[byte_index] = output_data[byte_index].wrapping_add(1);
                        },
                        2 => {
                            output_data[byte_index] = output_data[byte_index].wrapping_sub(1);
                        },
                        3 => {
                            output_data[byte_index] = output_data[byte_index].wrapping_mul(2);
                        },
                        4 => {
                            output_data[byte_index] ^= 0xAA;
                        },
                        5 => {
                            let byte = output_data[byte_index];
                            let high_nibble = byte & 0xF0;
                            let low_nibble = byte & 0x0F;
                            output_data[byte_index] = (low_nibble << 4) | (high_nibble >> 4);
                        },
                        _ => {}
                    }
                }
            }

            let progress = (i as f32 / config.iterations as f32) * 100.0;
            let _ = self.event_sender.send(DatabendEvent::EffectProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(18)).await;
        }

        Ok(output_data)
    }

    async fn apply_custom(&self, input_data: &[u8], config: &DatabendConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        for byte in output_data.iter_mut() {
            *byte = (*byte as f32 * config.intensity).round() as u8;
        }

        Ok(output_data)
    }

    fn generate_metadata(&self, config: &DatabendConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("effect_type".to_string(), format!("{:?}", config.effect_type));
        metadata.insert("intensity".to_string(), format!("{:.2}", config.intensity));
        metadata.insert("iterations".to_string(), config.iterations.to_string());
        metadata.insert("seed".to_string(), config.seed.map(|s| s.to_string()).unwrap_or("random".to_string()));
        metadata.insert("preserve_metadata".to_string(), config.preserve_metadata.to_string());
        metadata.insert("output_format".to_string(), format!("{:?}", config.output_format));
        
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

    pub fn set_effect_type(&self, effect_type: DatabendType) {
        let mut current_type = self.effect_type.write();
        *current_type = effect_type;
    }

    pub fn get_effect_type(&self) -> DatabendType {
        self.effect_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<DatabendEvent> {
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

    pub fn get_supported_effects(&self) -> Vec<DatabendType> {
        vec![
            DatabendType::PixelSort,
            DatabendType::BitCrush,
            DatabendType::DataMosh,
            DatabendType::Glitch,
            DatabendType::Corruption,
            DatabendType::BitShift,
            DatabendType::ByteManipulation,
        ]
    }

    pub fn can_apply_effect(&self, effect_type: &DatabendType) -> bool {
        self.get_supported_effects().contains(effect_type)
    }

    pub fn clone_effect(&self) -> DatabendEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_effect_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, input_size: usize, config: &DatabendConfig) -> std::time::Duration {
        let base_time_ms = match config.effect_type {
            DatabendType::PixelSort => 50.0,
            DatabendType::BitCrush => 100.0,
            DatabendType::DataMosh => 200.0,
            DatabendType::Glitch => 30.0,
            DatabendType::Corruption => 40.0,
            DatabendType::BitShift => 60.0,
            DatabendType::ByteManipulation => 80.0,
            DatabendType::Custom(_) => 150.0,
        };

        let time_per_byte = base_time_ms / 1000.0;
        let total_time = (input_size as f64 * time_per_byte) * config.iterations as f64;
        
        std::time::Duration::from_millis(total_time as u64)
    }

    pub fn estimate_output_size(&self, input_size: usize, config: &DatabendConfig) -> usize {
        input_size
    }

    pub fn validate_config(&self, config: &DatabendConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.iterations == 0 {
            return Err("Iterations must be greater than 0".into());
        }

        if config.intensity < 0.0 || config.intensity > 1.0 {
            return Err("Intensity must be between 0.0 and 1.0".into());
        }

        Ok(())
    }

    pub fn create_preset(&self, preset_name: &str) -> DatabendConfig {
        match preset_name {
            "light" => DatabendConfig {
                effect_type: self.get_effect_type(),
                intensity: 0.3,
                seed: None,
                iterations: 10,
                preserve_metadata: true,
                output_format: OutputFormat::Png,
            },
            "medium" => DatabendConfig {
                effect_type: self.get_effect_type(),
                intensity: 0.5,
                seed: None,
                iterations: 25,
                preserve_metadata: true,
                output_format: OutputFormat::Png,
            },
            "heavy" => DatabendConfig {
                effect_type: self.get_effect_type(),
                intensity: 0.8,
                seed: None,
                iterations: 50,
                preserve_metadata: true,
                output_format: OutputFormat::Png,
            },
            "extreme" => DatabendConfig {
                effect_type: self.get_effect_type(),
                intensity: 1.0,
                seed: None,
                iterations: 100,
                preserve_metadata: true,
                output_format: OutputFormat::Png,
            },
            _ => DatabendConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "light".to_string(),
            "medium".to_string(),
            "heavy".to_string(),
            "extreme".to_string(),
        ]
    }
}

impl Default for DatabendEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Databend Effect".to_string(),
            DatabendType::PixelSort,
        )
    }
}

impl Default for DatabendType {
    fn default() -> Self {
        DatabendType::PixelSort
    }
}

impl Default for DatabendEvent {
    fn default() -> Self {
        DatabendEvent::EffectStarted
    }
}

impl Default for DatabendResult {
    fn default() -> Self {
        Self {
            success: false,
            effect_type: DatabendType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for DatabendConfig {
    fn default() -> Self {
        Self {
            effect_type: DatabendType::default(),
            intensity: 0.5,
            seed: None,
            iterations: 25,
            preserve_metadata: true,
            output_format: OutputFormat::Png,
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Png
    }
}
