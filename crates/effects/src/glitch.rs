use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct GlitchEffect {
    pub id: String,
    pub name: String,
    pub glitch_type: Arc<RwLock<GlitchType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<GlitchEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<GlitchEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlitchType {
    Digital,
    Analog,
    Compression,
    DataCorruption,
    ScanLines,
    ColorBleed,
    Pixelation,
    ChromaticAberration,
    FrameDrop,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum GlitchEvent {
    GlitchStarted,
    GlitchProgress(f32),
    GlitchCompleted(GlitchResult),
    Error(String),
    FrameProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct GlitchResult {
    pub success: bool,
    pub glitch_type: GlitchType,
    pub output_frames: Vec<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GlitchConfig {
    pub glitch_type: GlitchType,
    pub intensity: f32,
    pub frequency: f32,
    pub seed: Option<u32>,
    pub frame_count: u32,
    pub preserve_metadata: bool,
    pub output_format: super::databend::OutputFormat,
}

#[derive(Debug, Clone)]
pub struct GlitchParameters {
    pub color_shift: f32,
    pub pixel_shuffle: f32,
    pub line_noise: f32,
    pub bit_corruption: f32,
    pub compression_artifacts: f32,
    pub temporal_glitch: f32,
    pub spatial_glitch: f32,
}

impl GlitchEffect {
    pub fn new(id: String, name: String, glitch_type: GlitchType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            glitch_type: Arc::new(RwLock::new(glitch_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_frames: &[Vec<u8>], config: GlitchConfig) -> Result<GlitchResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(GlitchEvent::GlitchStarted);
        let start_time = std::time::Instant::now();

        let result = match config.glitch_type {
            GlitchType::Digital => self.apply_digital_glitch(input_frames, &config).await,
            GlitchType::Analog => self.apply_analog_glitch(input_frames, &config).await,
            GlitchType::Compression => self.apply_compression_glitch(input_frames, &config).await,
            GlitchType::DataCorruption => self.apply_data_corruption_glitch(input_frames, &config).await,
            GlitchType::ScanLines => self.apply_scanlines_glitch(input_frames, &config).await,
            GlitchType::ColorBleed => self.apply_color_bleed_glitch(input_frames, &config).await,
            GlitchType::Pixelation => self.apply_pixelation_glitch(input_frames, &config).await,
            GlitchType::ChromaticAberration => self.apply_chromatic_aberration_glitch(input_frames, &config).await,
            GlitchType::FrameDrop => self.apply_frame_drop_glitch(input_frames, &config).await,
            GlitchType::Custom(_) => self.apply_custom_glitch(input_frames, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_frames) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(GlitchEvent::GlitchCompleted(GlitchResult {
                    success: true,
                    glitch_type: config.glitch_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(GlitchResult {
                    success: true,
                    glitch_type: config.glitch_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Glitch effect failed: {}", e);
                let _ = self.event_sender.send(GlitchEvent::Error(error_msg.clone()));

                Ok(GlitchResult {
                    success: false,
                    glitch_type: config.glitch_type.clone(),
                    output_frames: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_digital_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_digital_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
Report progress
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        Ok(output_frames)
    }

    fn apply_digital_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let displacement_count = (frame_size as f32 * config.intensity / 100.0) as usize;
            
            for _ in 0..displacement_count {
                let pos = rand::random::<usize>() % frame_size;
                let displacement = (rand::random::<i8>() as isize).clamp(-10, 10);
                let new_pos = (pos as isize + displacement).clamp(0, frame_size as isize - 1) as usize;
                
                if new_pos < frame_size {
                    output_frame[new_pos] = output_frame[pos];
                }
            }
        }

        if rand::random::<f32>() < config.intensity {
            let corruption_count = (frame_size as f32 * config.intensity / 200.0) as usize;
            
            for _ in 0..corruption_count {
                let pos = rand::random::<usize>() % frame_size;
                let bit_pos = rand::random::<u8>() % 8;
                output_frame[pos] ^= 1 << bit_pos;
            }
        }

        output_frame
    }

    async fn apply_analog_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_analog_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(output_frames)
    }

    fn apply_analog_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let shift_amount = (config.intensity * 10.0) as u8;
            
            for i in (0..frame_size).step_by(3) {
                if i + 2 < frame_size {
                    let temp = output_frame[i];
                    output_frame[i] = output_frame[i + 2];
                    output_frame[i + 2] = temp;
                }
            }
        }

        if rand::random::<f32>() < config.intensity {
            let noise_level = (config.intensity * 50.0) as u8;
            
            for byte in output_frame.iter_mut() {
                let noise = if rand::random::<f32>() < 0.5 {
                    -noise_level as i8
                } else {
                    noise_level as i8
                };
                
                *byte = byte.wrapping_add(noise as u8);
            }
        }

        output_frame
    }

    async fn apply_compression_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_compression_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok(output_frames)
    }

    fn apply_compression_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let block_size = ((config.intensity * 20.0) as usize).max(4);
            let block_count = frame_size / (block_size * block_size);
            
            for _ in 0..block_count {
                let block_start = rand::random::<usize>() % (frame_size - block_size);
                let block_end = (block_start + block_size).min(frame_size);
                
                if block_end <= frame_size {
                    let sum: u32 = output_frame[block_start..block_end].iter().map(|&b| b as u32).sum();
                    let avg = (sum / block_size as u32) as u8;
                    
                    for i in block_start..block_end {
                        output_frame[i] = avg;
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_data_corruption_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_data_corruption_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(18)).await;
        }

        Ok(output_frames)
    }

    fn apply_data_corruption_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let corruption_count = (frame_size as f32 * config.intensity / 50.0) as usize;
            
            for _ in 0..corruption_count {
                let pos = rand::random::<usize>() % frame_size;
                let corruption_type = rand::random::<u8>() % 4;
                
                match corruption_type {
                    0 => {
                        output_frame[pos] = rand::random::<u8>();
                    },
                    1 => {
                        output_frame[pos] = 0;
                    },
                    2 => {
                        output_frame[pos] = 0xFF;
                    },
                    3 => {
                        output_frame[pos] ^= 0x55;
                    },
                    _ => {}
                }
            }
        }

        output_frame
    }

    async fn apply_scanlines_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_scanlines_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        }

        Ok(output_frames)
    }

    fn apply_scanlines_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let line_height = ((config.intensity * 10.0) as usize).max(1);
            let line_spacing = ((config.intensity * 20.0) as usize).max(2);
            
            for y in (0..frame_size).step_by(line_spacing) {
                if y + line_height < frame_size {
                    for i in 0..line_height {
                        if y + i < frame_size {
                            output_frame[y + i] = (output_frame[y + i] as f32 * 0.7) as u8;
                        }
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_color_bleed_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_color_bleed_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(22)).await;
        }

        Ok(output_frames)
    }

    fn apply_color_bleed_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let bleed_amount = (config.intensity * 0.3) as u8;
            
            for i in (3..frame_size).step_by(3) {
                if i < frame_size {
                    if i + 3 < frame_size {
                        output_frame[i + 1] = output_frame[i + 1].saturating_add(bleed_amount);
                        output_frame[i + 2] = output_frame[i + 2].saturating_add(bleed_amount);
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_pixelation_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_pixelation_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        }

        Ok(output_frames)
    }

    fn apply_pixelation_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let pixel_size = ((config.intensity * 8.0) as usize).max(2);
            
            for i in (0..frame_size).step_by(pixel_size) {
                if i < frame_size {
                    let pixel_value = output_frame[i];
                    
                    for j in 0..pixel_size {
                        if i + j < frame_size {
                            output_frame[i + j] = pixel_value;
                        }
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_chromatic_aberration_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let glitched_frame = self.apply_chromatic_aberration_frame_glitch(frame, config);
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(19)).await;
        }

        Ok(output_frames)
    }

    fn apply_chromatic_aberration_frame_glitch(&self, frame: &[u8], config: &GlitchConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.frequency {
            let shift_amount = (config.intensity * 5.0) as usize;
            
            for i in (0..frame_size).step_by(3) {
                if i + 2 < frame_size {
                    let temp_r = output_frame[i];
                    let temp_g = output_frame[i + 1];
                    let temp_b = output_frame[i + 2];
                    
                    if shift_amount < frame_size {
                        output_frame[i] = output_frame[(i + shift_amount) % frame_size];
                    }
                    
                    if shift_amount * 2 < frame_size {
                        output_frame[i + 2] = output_frame[(i + shift_amount * 2) % frame_size];
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_frame_drop_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            if rand::random::<f32>() > config.frequency {
                output_frames.push(frame.to_vec());
            } else {
                if frame_index > 0 {
                    output_frames.push(input_frames[frame_index - 1].clone());
                } else {
                    output_frames.push(vec![0; frame.len()]);
                }
            }
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output_frames)
    }

    async fn apply_custom_glitch(&self, input_frames: &[Vec<u8>], config: &GlitchConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let mut glitched_frame = frame.to_vec();
            
            for byte in glitched_frame.iter_mut() {
                if rand::random::<f32>() < config.intensity {
                    *byte = (*byte as f32 * (1.0 + (rand::random::<f32>() - 0.5) * config.intensity)).round() as u8;
                }
            }
            
            output_frames.push(glitched_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(GlitchEvent::GlitchProgress(progress));
            let _ = self.event_sender.send(GlitchEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(21)).await;
        }

        Ok(output_frames)
    }

    fn generate_metadata(&self, config: &GlitchConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("glitch_type".to_string(), format!("{:?}", config.glitch_type));
        metadata.insert("intensity".to_string(), format!("{:.2}", config.intensity));
        metadata.insert("frequency".to_string(), format!("{:.2}", config.frequency));
        metadata.insert("frame_count".to_string(), config.frame_count.to_string());
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

    pub fn set_glitch_type(&self, glitch_type: GlitchType) {
        let mut current_type = self.glitch_type.write();
        *current_type = glitch_type;
    }

    pub fn get_glitch_type(&self) -> GlitchType {
        self.glitch_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<GlitchEvent> {
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

    pub fn get_supported_glitches(&self) -> Vec<GlitchType> {
        vec![
            GlitchType::Digital,
            GlitchType::Analog,
            GlitchType::Compression,
            GlitchType::DataCorruption,
            GlitchType::ScanLines,
            GlitchType::ColorBleed,
            GlitchType::Pixelation,
            GlitchType::ChromaticAberration,
            GlitchType::FrameDrop,
        ]
    }

    pub fn can_apply_glitch(&self, glitch_type: &GlitchType) -> bool {
        self.get_supported_glitches().contains(glitch_type)
    }

    pub fn clone_effect(&self) -> GlitchEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_glitch_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, frame_count: usize, config: &GlitchConfig) -> std::time::Duration {
        let base_time_ms = match config.glitch_type {
            GlitchType::Digital => 15.0,
            GlitchType::Analog => 20.0,
            GlitchType::Compression => 25.0,
            GlitchType::DataCorruption => 18.0,
            GlitchType::ScanLines => 12.0,
            GlitchType::ColorBleed => 22.0,
            GlitchType::Pixelation => 16.0,
            GlitchType::ChromaticAberration => 19.0,
            GlitchType::FrameDrop => 10.0,
            GlitchType::Custom(_) => 21.0,
        };

        let total_time_ms = frame_count as f64 * base_time_ms;
        std::time::Duration::from_millis(total_time_ms as u64)
    }

    pub fn create_preset(&self, preset_name: &str) -> GlitchConfig {
        match preset_name {
            "subtle" => GlitchConfig {
                glitch_type: self.get_glitch_type(),
                intensity: 0.2,
                frequency: 0.1,
                seed: None,
                frame_count: 30,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "moderate" => GlitchConfig {
                glitch_type: self.get_glitch_type(),
                intensity: 0.5,
                frequency: 0.3,
                seed: None,
                frame_count: 30,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "intense" => GlitchConfig {
                glitch_type: self.get_glitch_type(),
                intensity: 0.8,
                frequency: 0.6,
                seed: None,
                frame_count: 30,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            "extreme" => GlitchConfig {
                glitch_type: self.get_glitch_type(),
                intensity: 1.0,
                frequency: 0.9,
                seed: None,
                frame_count: 30,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
            },
            _ => GlitchConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "subtle".to_string(),
            "moderate".to_string(),
            "intense".to_string(),
            "extreme".to_string(),
        ]
    }
}

impl Default for GlitchEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Glitch Effect".to_string(),
            GlitchType::Digital,
        )
    }
}

impl Default for GlitchType {
    fn default() -> Self {
        GlitchType::Digital
    }
}

impl Default for GlitchEvent {
    fn default() -> Self {
        GlitchEvent::GlitchStarted
    }
}

impl Default for GlitchResult {
    fn default() -> Self {
        Self {
            success: false,
            glitch_type: GlitchType::default(),
            output_frames: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for GlitchConfig {
    fn default() -> Self {
        Self {
            glitch_type: GlitchType::default(),
            intensity: 0.5,
            frequency: 0.3,
            seed: None,
            frame_count: 30,
            preserve_metadata: true,
            output_format: super::databend::OutputFormat::Png,
        }
    }
}

impl Default for GlitchParameters {
    fn default() -> Self {
        Self {
            color_shift: 0.5,
            pixel_shuffle: 0.3,
            line_noise: 0.4,
            bit_corruption: 0.2,
            compression_artifacts: 0.6,
            temporal_glitch: 0.5,
            spatial_glitch: 0.4,
        }
    }
}
