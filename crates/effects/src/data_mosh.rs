use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct DataMoshEffect {
    pub id: String,
    pub name: String,
    pub mosh_type: Arc<RwLock<MoshType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<MoshEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<MoshEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoshType {
    Digital,
    Analog,
    Compression,
    Decoding,
    Network,
    Temporal,
    Spatial,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum MoshEvent {
    MoshStarted,
    MoshProgress(f32),
    MoshCompleted(MoshResult),
    Error(String),
    FrameProcessed(usize),
    SegmentProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct MoshResult {
    pub success: bool,
    pub mosh_type: MoshType,
    pub output_frames: Vec<Vec<u8>>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MoshConfig {
    pub mosh_type: MoshType,
    pub intensity: f32,
    pub frequency: f32,
    pub seed: Option<u32>,
    pub frame_count: u32,
    pub segment_size: u32,
    pub preserve_metadata: bool,
    pub output_format: super::databend::OutputFormat,
    pub mosh_parameters: MoshParameters,
}

#[derive(Debug, Clone)]
pub struct MoshParameters {
    pub glitch_probability: f32,
    pub corruption_probability: f32,
    pub frame_drop_probability: f32,
    pub pixel_shift_probability: f32,
    pub color_shift_probability: f32,
    pub temporal_shift_probability: f32,
    pub spatial_distortion_probability: f32,
    pub compression_artifact_probability: f32,
}

impl DataMoshEffect {
    pub fn new(id: String, name: String, mosh_type: MoshType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            mosh_type: Arc::new(RwLock::new(mosh_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_frames: &[Vec<u8>], config: MoshConfig) -> Result<MoshResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(MoshEvent::MoshStarted);
        let start_time = std::time::Instant::now();

        let result = match config.mosh_type {
            MoshType::Digital => self.apply_digital_mosh(input_frames, &config).await,
            MoshType::Analog => self.apply_analog_mosh(input_frames, &config).await,
            MoshType::Compression => self.apply_compression_mosh(input_frames, &config).await,
            MoshType::Decoding => self.apply_decoding_mosh(input_frames, &config).await,
            MoshType::Network => self.apply_network_mosh(input_frames, &config).await,
            MoshType::Temporal => self.apply_temporal_mosh(input_frames, &config).await,
            MoshType::Spatial => self.apply_spatial_mosh(input_frames, &config).await,
            MoshType::Custom(_) => self.apply_custom_mosh(input_frames, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_frames) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(MoshEvent::MoshCompleted(MoshResult {
                    success: true,
                    mosh_type: config.mosh_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(MoshResult {
                    success: true,
                    mosh_type: config.mosh_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Data mosh effect failed: {}", e);
                let _ = self.event_sender.send(MoshEvent::Error(error_msg.clone()));

                Ok(MoshResult {
                    success: false,
                    mosh_type: config.mosh_type.clone(),
                    output_frames: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_digital_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_digital_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
Report progress
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(output_frames)
    }

    fn apply_digital_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.glitch_probability {
            let corruption_count = (frame_size as f32 * config.intensity / 100.0) as usize;
            
            for _ in 0..corruption_count {
                let pos = rand::random::<usize>() % frame_size;
                let corruption_type = rand::random::<u8>() % 4;
                
                match corruption_type {
                    0 => {
                        output_frame[pos] = rand::random::<u8>();
                    },
                    1 => {
                        output_frame[pos] ^= 0xFF;
                    },
                    2 => {
                        output_frame[pos] = 0;
                    },
                    3 => {
                        output_frame[pos] = output_frame[pos].wrapping_add(rand::random::<i8>());
                    },
                    _ => {}
                }
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.frame_drop_probability {
            for byte in output_frame.iter_mut() {
                *byte = rand::random::<u8>();
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.pixel_shift_probability {
            let shift_amount = (config.intensity * 10.0) as usize;
            
            for i in 0..frame_size {
                let new_pos = (i + shift_amount) % frame_size;
                if new_pos < frame_size {
                    output_frame[i] = output_frame[new_pos];
                }
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.color_shift_probability {
            let shift_amount = (config.intensity * 5.0) as u8;
            
            for i in (0..frame_size).step_by(4) {
                if i + 3 < frame_size {
                    let temp = output_frame[i];
                    output_frame[i] = output_frame[i + 1];
                    output_frame[i + 1] = output_frame[i + 2];
                    output_frame[i + 2] = (temp as u8).wrapping_add(shift_amount);
                }
            }
        }

        output_frame
    }

    async fn apply_analog_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_analog_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok(output_frames)
    }

    fn apply_analog_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.glitch_probability {
            let shift_amount = (config.intensity * 8.0) as u8;
            
            for i in (0..frame_size).step_by(3) {
                if i + 2 < frame_size {
                    let temp = output_frame[i];
                    output_frame[i] = output_frame[i + 1];
                    output_frame[i + 1] = output_frame[i + 2];
                    output_frame[i + 2] = (temp as u8).wrapping_add(shift_amount);
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

        if rand::random::<f32>() < config.mosh_parameters.spatial_distortion_probability {
            let distortion_amount = (config.intensity * 0.3) as u8;
            
            for i in (0..frame_size).step_by(3) {
                if i + 2 < frame_size {
                    let r = output_frame[i];
                    let g = output_frame[i + 1];
                    let b = output_frame[i + 2];
                    
                    output_frame[i] = (r as f32 * (1.0 - distortion_amount as f32 / 255.0)).round() as u8;
                    output_frame[i + 1] = (g as f32 * (1.0 - distortion_amount as f32 / 255.0)).round() as u8;
                    output_frame[i + 2] = (b as f32 * (1.0 - distortion_amount as f32 / 255.0)).round() as u8;
                }
            }
        }

        output_frame
    }

    async fn apply_compression_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_compression_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        }

        Ok(output_frames)
    }

    fn apply_compression_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.compression_artifact_probability {
            let block_size = ((config.intensity * 15.0) as usize).max(4);
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

        if rand::random::<f32>() < config.intensity {
            let quantization_levels = ((config.intensity * 8.0) as u8).max(2);
            let step_size = 256 / quantization_levels;
            
            for byte in output_frame.iter_mut() {
                *byte = (*byte as u32 / step_size as u32) * step_size as u8;
            }
        }

        output_frame
    }

    async fn apply_decoding_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_decoding_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(22)).await;
        }

        Ok(output_frames)
    }

    fn apply_decoding_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.corruption_probability {
            let error_count = (frame_size as f32 * config.intensity / 200.0) as usize;
            
            for _ in 0..error_count {
                let pos = rand::random::<usize>() % frame_size;
                let error_type = rand::random::<u8>() % 3;
                
                match error_type {
                    0 => {
                        output_frame[pos] = if pos > 0 { output_frame[pos - 1] } else { 0 };
                    },
                    1 => {
                        output_frame[pos] = rand::random::<u8>();
                    },
                    2 => {
                        output_frame[pos] = 0;
                    },
                    _ => {}
                }
            }
        }

        if rand::random::<f32>() < config.intensity {
            let section_count = (frame_size as f32 * config.intensity / 500.0) as usize;
            
            for _ in 0..section_count {
                let section_start = rand::random::<usize>() % frame_size;
                let section_size = rand::random::<usize>() % 64 + 1;
                
                for i in 0..section_size {
                    let pos = (section_start + i) % frame_size;
                    output_frame[pos] = 0;
                }
            }
        }

        output_frame
    }

    async fn apply_network_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_network_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(18)).await;
        }

        Ok(output_frames)
    }

    fn apply_network_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.frame_drop_probability {
            for byte in output_frame.iter_mut() {
                *byte = 0;
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.corruption_probability {
            let corruption_count = (frame_size as f32 * config.intensity / 300.0) as usize;
            
            for _ in 0..corruption_count {
                let pos = rand::random::<usize>() % frame_size;
                let corruption_size = rand::random::<usize>() % 32 + 1;
                
                for i in 0..corruption_size {
                    let corrupt_pos = (pos + i) % frame_size;
                    output_frame[corrupt_pos] = rand::random::<u8>();
                }
            }
        }

        if rand::random::<f32>() < config.intensity {
            let reduction_factor = (config.intensity * 0.5) as usize;
            
            for i in (0..frame_size).step_by(4) {
                if i + 3 < frame_size {
                    let r = (output_frame[i] + output_frame[i + 1]) / 2;
                    let g = (output_frame[i + 2] + output_frame[i + 3]) / 2;
                    
                    output_frame[i] = r;
                    output_frame[i + 1] = r;
                    output_frame[i + 2] = g;
                    output_frame[i + 3] = g;
                }
            }
        }

        output_frame
    }

    async fn apply_temporal_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_temporal_frame_mosh(frame, frame_index, input_frames, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        Ok(output_frames)
    }

    fn apply_temporal_frame_mosh(&self, frame: &[u8], frame_index: usize, all_frames: &[Vec<u8>], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.temporal_shift_probability {
            if frame_index > 0 {
                let previous_frame = &all_frames[frame_index - 1];
                let shift_amount = (config.intensity * 0.3) as usize;
                
                for i in 0..frame_size.min(previous_frame.len()) {
                    if i + shift_amount < previous_frame.len() {
                        output_frame[i] = previous_frame[i + shift_amount];
                    }
                }
            }
        }

        if rand::random::<f32>() < config.intensity {
            if frame_index > 0 && rand::random::<f32>() < 0.5 {
                let previous_frame = &all_frames[frame_index - 1];
                if previous_frame.len() == frame_size {
                    output_frame = previous_frame.to_vec();
                }
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.glitch_probability {
            let blend_factor = config.intensity * 0.2;
            
            if frame_index > 0 && frame_index < all_frames.len() - 1 {
                let previous_frame = &all_frames[frame_index - 1];
                let next_frame = &all_frames[frame_index + 1];
                
                for i in 0..frame_size.min(previous_frame.len()).min(next_frame.len()) {
                    let prev_val = previous_frame[i] as f32;
                    let next_val = next_frame[i] as f32;
                    let current_val = output_frame[i] as f32;
                    
                    output_frame[i] = (current_val * (1.0 - blend_factor) + 
                                      (prev_val + next_val) * 0.5 * blend_factor).round() as u8;
                }
            }
        }

        output_frame
    }

    async fn apply_spatial_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_spatial_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(28)).await;
        }

        Ok(output_frames)
    }

    fn apply_spatial_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.mosh_parameters.spatial_distortion_probability {
            let artifact_count = (frame_size as f32 * config.intensity / 400.0) as usize;
            
            for _ in 0..artifact_count {
                let artifact_type = rand::random::<u8>() % 3;
                
                match artifact_type {
                    0 => {
                        let line_width = (config.intensity * 5.0) as usize;
                        let line_spacing = (config.intensity * 20.0) as usize;
                        
                        for y in (0..frame_size).step_by(line_spacing * 4) {
                            for x in 0..line_width {
                                if y + x < frame_size {
                                    output_frame[y + x] = 0;
                                }
                            }
                        }
                    },
                    1 => {
                        let block_size = (config.intensity * 8.0) as usize;
                        
                        for y in (0..frame_size).step_by(block_size * 4) {
                            for x in (0..frame_size).step_by(block_size) {
                                if (x / block_size + y / block_size) % 2 == 0 {
                                    if y + x < frame_size {
                                        output_frame[y + x] = 255;
                                    }
                                }
                            }
                        }
                    },
                    2 => {
                        let center_x = frame_size / 8;
                        let center_y = frame_size / 8;
                        let radius = (config.intensity * 10.0) as usize;
                        
                        for y in 0..frame_size {
                            for x in 0..frame_size {
                                let dx = (x / 4) as isize - center_x as isize;
                                let dy = (y / 4) as isize - center_y as isize;
                                let distance = (dx * dx + dy * dy) as f32;
                                
                                if distance < (radius * radius) as f32 {
                                    if y + x < frame_size {
                                        output_frame[y + x] = 128;
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        if rand::random::<f32>() < config.mosh_parameters.pixel_shift_probability {
            let displacement_count = (frame_size as f32 * config.intensity / 150.0) as usize;
            
            for _ in 0..displacement_count {
                let pos = rand::random::<usize>() % frame_size;
                let displacement = (rand::random::<i8>() as isize).clamp(-20, 20);
                let new_pos = (pos as isize + displacement).clamp(0, frame_size as isize - 1) as usize;
                
                if new_pos < frame_size {
                    output_frame[new_pos] = output_frame[pos];
                }
            }
        }

        output_frame
    }

    async fn apply_custom_mosh(&self, input_frames: &[Vec<u8>], config: &MoshConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let moshed_frame = self.apply_custom_frame_mosh(frame, config);
            output_frames.push(moshed_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(MoshEvent::MoshProgress(progress));
            let _ = self.event_sender.send(MoshEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(24)).await;
        }

        Ok(output_frames)
    }

    fn apply_custom_frame_mosh(&self, frame: &[u8], config: &MoshConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        
        for byte in output_frame.iter_mut() {
            if rand::random::<f32>() < config.intensity {
                *byte = (*byte as f32 * (1.0 + (rand::random::<f32>() - 0.5) * config.intensity)).round() as u8;
            }
        }

        output_frame
    }

    fn generate_metadata(&self, config: &MoshConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("mosh_type".to_string(), format!("{:?}", config.mosh_type));
        metadata.insert("intensity".to_string(), format!("{:.2}", config.intensity));
        metadata.insert("frequency".to_string(), format!("{:.2}", config.frequency));
        metadata.insert("frame_count".to_string(), config.frame_count.to_string());
        metadata.insert("segment_size".to_string(), config.segment_size.to_string());
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

    pub fn set_mosh_type(&self, mosh_type: MoshType) {
        let mut current_type = self.mosh_type.write();
        *current_type = mosh_type;
    }

    pub fn get_mosh_type(&self) -> MoshType {
        self.mosh_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<MoshEvent> {
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

    pub fn get_supported_mosh_types(&self) -> Vec<MoshType> {
        vec![
            MoshType::Digital,
            MoshType::Analog,
            MoshType::Compression,
            MoshType::Decoding,
            MoshType::Network,
            MoshType::Temporal,
            MoshType::Spatial,
        ]
    }

    pub fn can_apply_mosh_type(&self, mosh_type: &MoshType) -> bool {
        self.get_supported_mosh_types().contains(mosh_type)
    }

    pub fn clone_effect(&self) -> DataMoshEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_mosh_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, frame_count: usize, config: &MoshConfig) -> std::time::Duration {
        let base_time_ms = match config.mosh_type {
            MoshType::Digital => 20.0,
            MoshType::Analog => 25.0,
            MoshType::Compression => 30.0,
            MoshType::Decoding => 22.0,
            MoshType::Network => 18.0,
            MoshType::Temporal => 15.0,
            MoshType::Spatial => 28.0,
            MoshType::Custom(_) => 24.0,
        };

        let time_per_frame = base_time_ms / 1000.0;
        let total_time = frame_count as f64 * time_per_frame;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> MoshConfig {
        match preset_name {
            "subtle" => MoshConfig {
                mosh_type: self.get_mosh_type(),
                intensity: 0.2,
                frequency: 0.1,
                seed: None,
                frame_count: 30,
                segment_size: 10,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                mosh_parameters: MoshParameters::default(),
            },
            "moderate" => MoshConfig {
                mosh_type: self.get_mosh_type(),
                intensity: 0.5,
                frequency: 0.3,
                seed: None,
                frame_count: 30,
                segment_size: 10,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                mosh_parameters: MoshParameters::default(),
            },
            "intense" => MoshConfig {
                mosh_type: self.get_mosh_type(),
                intensity: 0.8,
                frequency: 0.6,
                seed: None,
                frame_count: 30,
                segment_size: 10,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                mosh_parameters: MoshParameters::default(),
            },
            "extreme" => MoshConfig {
                mosh_type: self.get_mosh_type(),
                intensity: 1.0,
                frequency: 0.9,
                seed: None,
                frame_count: 30,
                segment_size: 10,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                mosh_parameters: MoshParameters::default(),
            },
            _ => MoshConfig::default(),
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

impl Default for DataMoshEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Data Mosh Effect".to_string(),
            MoshType::Digital,
        )
    }
}

impl Default for MoshType {
    fn default() -> Self {
        MoshType::Digital
    }
}

impl Default for MoshEvent {
    fn default() -> Self {
        MoshEvent::MoshStarted
    }
}

impl Default for MoshResult {
    fn default() -> Self {
        Self {
            success: false,
            mosh_type: MoshType::default(),
            output_frames: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for MoshConfig {
    fn default() -> Self {
        Self {
            mosh_type: MoshType::default(),
            intensity: 0.5,
            frequency: 0.3,
            seed: None,
            frame_count: 30,
            segment_size: 10,
            preserve_metadata: true,
            output_format: super::databend::OutputFormat::Png,
            mosh_parameters: MoshParameters::default(),
        }
    }
}

impl Default for MoshParameters {
    fn default() -> Self {
        Self {
            glitch_probability: 0.3,
            corruption_probability: 0.2,
            frame_drop_probability: 0.1,
            pixel_shift_probability: 0.2,
            color_shift_probability: 0.2,
            temporal_shift_probability: 0.15,
            spatial_distortion_probability: 0.2,
            compression_artifact_probability: 0.25,
        }
    }
}
