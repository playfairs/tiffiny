use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ErrorDiffusionEffect {
    pub id: String,
    pub name: String,
    pub diffusion_type: Arc<RwLock<DiffusionType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<ErrorDiffusionEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ErrorDiffusionEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffusionType {
    Glitch,
    Corruption,
    DataLoss,
    BitFlip,
    Compression,
    Network,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ErrorDiffusionEvent {
    DiffusionStarted,
    DiffusionProgress(f32),
    DiffusionCompleted(ErrorDiffusionResult),
    Error(String),
    FrameProcessed(usize),
    IterationCompleted(usize),
}

#[derive(Debug, Clone)]
pub struct ErrorDiffusionResult {
    pub success: bool,
    pub diffusion_type: DiffusionType,
    pub output_frames: Vec<Vec<u8>>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ErrorDiffusionConfig {
    pub diffusion_type: DiffusionType,
    pub intensity: f32,
    pub iterations: u32,
    pub seed: Option<u32>,
    pub preserve_metadata: bool,
    pub output_format: super::databend::OutputFormat,
    pub diffusion_parameters: DiffusionParameters,
}

#[derive(Debug, Clone)]
pub struct DiffusionParameters {
    pub error_probability: f32,
    pub corruption_rate: f32,
    pub data_loss_rate: f32,
    pub bit_flip_probability: f32,
    pub compression_artifacts: f32,
    pub network_latency: f32,
    pub packet_loss_rate: f32,
    pub temporal_drift: f32,
    pub spatial_distortion: f32,
    pub noise_level: f32,
}

impl ErrorDiffusionEffect {
    pub fn new(id: String, name: String, diffusion_type: DiffusionType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            diffusion_type: Arc::new(RwLock::new(diffusion_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_frames: &[Vec<u8>], config: ErrorDiffusionConfig) -> Result<ErrorDiffusionResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionStarted);
        let start_time = std::time::Instant::now();

        let result = match config.diffusion_type {
            DiffusionType::Glitch => self.apply_glitch_diffusion(input_frames, &config).await,
            DiffusionType::Corruption => self.apply_corruption_diffusion(input_frames, &config).await,
            DiffusionType::DataLoss => self.apply_data_loss_diffusion(input_frames, &config).await,
            DiffusionType::BitFlip => self.apply_bit_flip_diffusion(input_frames, &config).await,
            DiffusionType::Compression => self.apply_compression_diffusion(input_frames, &config).await,
            DiffusionType::Network => self.apply_network_diffusion(input_frames, &config).await,
            DiffusionType::Custom(_) => self.apply_custom_diffusion(input_frames, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_frames) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionCompleted(ErrorDiffusionResult {
                    success: true,
                    diffusion_type: config.diffusion_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(ErrorDiffusionResult {
                    success: true,
                    diffusion_type: config.diffusion_type.clone(),
                    output_frames,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Error diffusion effect failed: {}", e);
                let _ = self.event_sender.send(ErrorDiffusionEvent::Error(error_msg.clone()));

                Ok(ErrorDiffusionResult {
                    success: false,
                    diffusion_type: config.diffusion_type.clone(),
                    output_frames: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_glitch_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_glitch_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
Report progress
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok(output_frames)
    }

    fn apply_glitch_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        if rand::random::<f32>() < config.diffusion_parameters.error_probability {
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

        if rand::random::<f32>() < config.diffusion_parameters.temporal_drift {
            let shift_amount = (config.intensity * 10.0) as usize;
            
            for i in 0..frame_size {
                let new_pos = (i + shift_amount) % frame_size;
                if new_pos < frame_size {
                    output_frame[i] = output_frame[new_pos];
                }
            }
        }

        if rand::random::<f32>() < config.diffusion_parameters.spatial_distortion {
            let artifact_count = (frame_size as f32 * config.intensity / 200.0) as usize;
            
            for _ in 0..artifact_count {
                let pos = rand::random::<usize>() % frame_size;
                let artifact_type = rand::random::<u8>() % 3;
                
                match artifact_type {
                    0 => {
                        let line_width = (config.intensity * 5.0) as usize;
                        let line_spacing = (config.intensity * 15.0) as usize;
                        
                        for y in (0..frame_size).step_by(line_spacing) {
                            for x in 0..line_width {
                                if y + x < frame_size {
                                    output_frame[y + x] = 128;
                                }
                            }
                        }
                    },
                    1 => {
                        let block_size = (config.intensity * 8.0) as usize;
                        
                        for y in (0..frame_size).step_by(block_size) {
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
                        let center_x = frame_size / 2;
                        let center_y = frame_size / 2;
                        let radius = (config.intensity * 20.0) as usize;
                        
                        for y in 0..frame_size {
                            for x in 0..frame_size {
                                let dx = (x as isize - center_x as isize);
                                let dy = (y as isize - center_y as isize);
                                let distance = (dx * dx + dy * dy) as f32;
                                
                                if distance < (radius * radius) as f32 && y + x < frame_size {
                                    output_frame[y + x] = 128;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        output_frame
    }

    async fn apply_corruption_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_corruption_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        }

        Ok(output_frames)
    }

    fn apply_corruption_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        let corruption_rate = config.diffusion_parameters.corruption_rate;
        let corruption_count = (frame_size as f32 * corruption_rate) as usize;
        
        for _ in 0..corruption_count {
            let pos = rand::random::<usize>() % frame_size;
            let corruption_type = rand::random::<u8>() % 5;
            
            match corruption_type {
                0 => {
                    output_frame[pos] = rand::random::<u8>();
                },
                1 => {
                    output_frame[pos] = 255;
                },
                2 => {
                    output_frame[pos] = 0;
                },
                3 => {
                    output_frame[pos] = !output_frame[pos];
                },
                4 => {
                    let noise = (rand::random::<f32>() - 0.5) * 50.0;
                    output_frame[pos] = (output_frame[pos] as f32 + noise).clamp(0.0, 255.0)) as u8;
                },
                _ => {}
            }
        }

        if rand::random::<f32>() < config.diffusion_parameters.data_loss_rate {
            let loss_count = (frame_size as f32 * config.diffusion_parameters.data_loss_rate) as usize;
            
            for _ in 0..loss_count {
                let pos = rand::random::<usize>() % frame_size;
                output_frame[pos] = 0;
            }
        }

        output_frame
    }

    async fn apply_data_loss_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        let mut previous_frame: Option<Vec<u8>> = None;
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_data_loss_frame_diffusion(frame, previous_frame.as_ref(), config);
            output_frames.push(diffused_frame);
            
            previous_frame = Some(frame.to_vec());
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(output_frames)
    }

    fn apply_data_loss_frame_diffusion(&self, frame: &[u8], previous_frame: Option<&Vec<u8>>, config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        let data_loss_rate = config.diffusion_parameters.data_loss_rate;
        let loss_count = (frame_size as f32 * data_loss_rate) as usize;
        
        for _ in 0..loss_count {
            let pos = rand::random::<usize>() % frame_size;
            output_frame[pos] = 0;
        }
        
        if let Some(prev_frame) = previous_frame {
            let blend_factor = 1.0 - data_loss_rate;
            
            for i in 0..frame_size {
                output_frame[i] = (output_frame[i] as f32 * blend_factor + 
                                   prev_frame[i] as f32 * data_loss_rate).clamp(0.0, 255.0)) as u8;
            }
        }

        output_frame
    }

    async fn apply_bit_flip_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_bit_flip_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        }

        Ok(output_frames)
    }

    fn apply_bit_flip_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        let bit_flip_probability = config.diffusion_parameters.bit_flip_probability;
        let flip_count = (frame_size as f32 * bit_flip_probability) as usize;
        
        for _ in 0..flip_count {
            let pos = rand::random::<usize>() % frame_size;
            let bit_to_flip = rand::random::<u8>() % 8;
            
            if bit_to_flip < 8 {
                output_frame[pos] ^= 1 << bit_to_flip;
            }
        }

        if rand::random::<f32>() < config.intensity {
            let region_count = (frame_size as f32 * config.intensity / 500.0) as usize;
            
            for _ in 0..region_count {
                let region_start = rand::random::<usize>() % frame_size;
                let region_size = (frame_size / region_count).min(16);
                let region_end = (region_start + region_size).min(frame_size);
                
                for i in region_start..region_end {
                    if i < frame_size {
                        output_frame[i] = !output_frame[i];
                    }
                }
            }
        }

        output_frame
    }

    async fn apply_compression_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_compression_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(35)).await;
        }

        Ok(output_frames)
    }

    fn apply_compression_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        let artifact_intensity = config.diffusion_parameters.compression_artifacts;
        
        if rand::random::<f32>() < artifact_intensity {
            let block_size = ((artifact_intensity * 20.0) as usize).max(4);
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
        
        if rand::random::<f32>() < artifact_intensity {
            let quantization_levels = ((artifact_intensity * 8.0) as u8).max(2);
            let step_size = 256 / quantization_levels;
            
            for byte in output_frame.iter_mut() {
                *byte = (*byte as u32 / step_size as u32) * step_size as u8;
            }
        }
        
        if rand::random::<f32>() < artifact_intensity {
            for byte in output_frame.iter_mut() {
                let ringing = (rand::random::<f32>() - 0.5) * artifact_intensity * 10.0;
                *byte = (*byte as f32 + ringing).clamp(0.0, 255.0)).round() as u8;
            }
        }

        output_frame
    }

    async fn apply_network_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_network_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }

        Ok(output_frames)
    }

    fn apply_network_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        let packet_loss_rate = config.diffusion_parameters.packet_loss_rate;
        let network_latency = config.diffusion_parameters.network_latency;
        
        if rand::random::<f32>() < packet_loss_rate {
            let loss_count = (frame_size as f32 * packet_loss_rate) as usize;
            
            for _ in 0..loss_count {
                let packet_start = rand::random::<usize>() % (frame_size / 64);
                let packet_end = (packet_start + 64).min(frame_size);
                
                for i in packet_start..packet_end {
                    if i < frame_size {
                        for j in 0..64 {
                            if i + j < frame_size {
                                output_frame[i + j] = 0;
                            }
                        }
                    }
                }
            }
        }
        
        if rand::random::<f32>() < network_latency {
            tokio::time::sleep(std::time::Duration::from_millis((network_latency * 100.0) as u64));
        }
        
        if rand::random::<f32>() < config.diffusion_parameters.corruption_rate {
            let corruption_count = (frame_size as f32 * config.diffusion_parameters.corruption_rate) as usize;
            
            for _ in 0..corruption_count {
                let pos = rand::random::<usize>() % frame_size;
                let corruption_type = rand::random::<u8>() % 3;
                
                match corruption_type {
                    0 => {
                        output_frame[pos] ^= 0x55;
                    },
                    1 => {
                        output_frame[pos] = rand::random::<u8>();
                    },
                    2 => {
                        if pos + 1 < frame_size {
                            output_frame[pos + 1] = output_frame[pos];
                        }
                    },
                    _ => {}
                }
            }
        }

        output_frame
    }

    async fn apply_custom_diffusion(&self, input_frames: &[Vec<u8>], config: &ErrorDiffusionConfig) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut output_frames = Vec::new();
        
        for (frame_index, frame) in input_frames.iter().enumerate() {
            let diffused_frame = self.apply_custom_frame_diffusion(frame, config);
            output_frames.push(diffused_frame);
            
            let progress = ((frame_index + 1) as f32 / input_frames.len() as f32) * 100.0;
            let _ = self.event_sender.send(ErrorDiffusionEvent::DiffusionProgress(progress));
            let _ = self.event_sender.send(ErrorDiffusionEvent::FrameProcessed(frame_index));
            
            if frame_index % config.iterations == 0 {
                let iteration_progress = ((frame_index / config.iterations) as f32) * 100.0;
                let _ = self.event_sender.send(ErrorDiffusionEvent::IterationCompleted(frame_index));
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(28)).await;
        }

        Ok(output_frames)
    }

    fn apply_custom_frame_diffusion(&self, frame: &[u8], config: &ErrorDiffusionConfig) -> Vec<u8> {
        let mut output_frame = frame.to_vec();
        let frame_size = output_frame.len();
        
        for byte in output_frame.iter_mut() {
            if rand::random::<f32>() < config.intensity {
                let transform_type = rand::random::<u8>() % 4;
                
                match transform_type {
                    0 => {
                        let noise = (rand::random::<f32>() - 0.5) * config.diffusion_parameters.noise_level;
                        *byte = (*byte as f32 + noise).clamp(0.0, 255.0)).round() as u8;
                    },
                    1 => {
                        let freq = (rand::random::<f32>() * config.diffusion_parameters.noise_level * 0.1);
                        let phase = (frame_size as f32 * freq) * std::f32::consts::PI * 2.0);
                        *byte = (*byte as f32 * (phase.cos() * 0.5 + 0.5)).round() as u8;
                    },
                    2 => {
                        let amp = (rand::random::<f32>() * config.diffusion_parameters.noise_level * 2.0);
                        *byte = (*byte as f32 * amp).clamp(0.0, 2.0)).round() as u8;
                    },
                    3 => {
                        let phase_shift = (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
                        *byte = (*byte as f32 * phase_shift.cos()).round() as u8;
                    },
                    _ => {}
                }
            }
        }

        output_frame
    }

    fn generate_metadata(&self, config: &ErrorDiffusionConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("diffusion_type".to_string(), format!("{:?}", config.diffusion_type));
        metadata.insert("intensity".to_string(), format!("{:.2}", config.intensity));
        metadata.insert("iterations".to_string(), config.iterations.to_string());
        metadata.insert("seed".to_string(), config.seed.map(|s| s.to_string()).unwrap_or("random".to_string()));
        metadata.insert("preserve_metadata".to_string(), config.preserve_metadata.to_string());
        metadata.insert("output_format".to_string(), format!("{:?}", config.output_format));
        
        metadata.insert("error_probability".to_string(), format!("{:.2}", config.diffusion_parameters.error_probability));
        metadata.insert("corruption_rate".to_string(), format!("{:.2}", config.diffusion_parameters.corruption_rate));
        metadata.insert("data_loss_rate".to_string(), format!("{:.2}", config.diffusion_parameters.data_loss_rate));
        metadata.insert("bit_flip_probability".to_string(), format!("{:.2}", config.diffusion_parameters.bit_flip_probability));
        metadata.insert("compression_artifacts".to_string(), format!("{:.2}", config.diffusion_parameters.compression_artifacts));
        metadata.insert("network_latency".to_string(), format!("{:.2}", config.diffusion_parameters.network_latency));
        metadata.insert("packet_loss_rate".to_string(), format!("{:.2}", config.diffusion_parameters.packet_loss_rate));
        metadata.insert("temporal_drift".to_string(), format!("{:.2}", config.diffusion_parameters.temporal_drift));
        metadata.insert("spatial_distortion".to_string(), format!("{:.2}", config.diffusion_parameters.spatial_distortion));
        metadata.insert("noise_level".to_string(), format!("{:.2}", config.diffusion_parameters.noise_level));
        
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

    pub fn set_diffusion_type(&self, diffusion_type: DiffusionType) {
        let mut current_type = self.diffusion_type.write();
        *current_type = diffusion_type;
    }

    pub fn get_diffusion_type(&self) -> DiffusionType {
        self.diffusion_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<ErrorDiffusionEvent> {
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

    pub fn get_supported_diffusion_types(&self) -> Vec<DiffusionType> {
        vec![
            DiffusionType::Glitch,
            DiffusionType::Corruption,
            DiffusionType::DataLoss,
            DiffusionType::BitFlip,
            DiffusionType::Compression,
            DiffusionType::Network,
        ]
    }

    pub fn can_apply_diffusion_type(&self, diffusion_type: &DiffusionType) -> bool {
        self.get_supported_diffusion_types().contains(diffusion_type)
    }

    pub fn clone_effect(&self) -> ErrorDiffusionEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_diffusion_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, frame_count: usize, config: &ErrorDiffusionConfig) -> std::time::Duration {
        let base_time_ms = match config.diffusion_type {
            DiffusionType::Glitch => 25.0,
            DiffusionType::Corruption => 30.0,
            DiffusionType::DataLoss => 20.0,
            DiffusionType::BitFlip => 15.0,
            DiffusionType::Compression => 35.0,
            DiffusionType::Network => 40.0,
            DiffusionType::Custom(_) => 32.0,
        };

        let time_per_frame = base_time_ms / 1000.0;
        let total_time = frame_count as f64 * time_per_frame;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> ErrorDiffusionConfig {
        match preset_name {
            "subtle" => ErrorDiffusionConfig {
                diffusion_type: self.get_diffusion_type(),
                intensity: 0.2,
                iterations: 10,
                seed: None,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                diffusion_parameters: DiffusionParameters::default(),
            },
            "moderate" => ErrorDiffusionConfig {
                diffusion_type: self.get_diffusion_type(),
                intensity: 0.5,
                iterations: 25,
                seed: None,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                diffusion_parameters: DiffusionParameters::default(),
            },
            "intense" => ErrorDiffusionConfig {
                diffusion_type: self.get_diffusion_type(),
                intensity: 0.8,
                iterations: 50,
                seed: None,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                diffusion_parameters: DiffusionParameters::default(),
            },
            "extreme" => ErrorDiffusionConfig {
                diffusion_type: self.get_diffusion_type(),
                intensity: 1.0,
                iterations: 100,
                seed: None,
                preserve_metadata: true,
                output_format: super::databend::OutputFormat::Png,
                diffusion_parameters: DiffusionParameters::default(),
            },
            _ => ErrorDiffusionConfig::default(),
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

impl Default for ErrorDiffusionEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Error Diffusion Effect".to_string(),
            DiffusionType::Glitch,
        )
    }
}

impl Default for DiffusionType {
    fn default() -> Self {
        DiffusionType::Glitch
    }
}

impl Default for ErrorDiffusionEvent {
    fn default() -> Self {
        ErrorDiffusionEvent::DiffusionStarted
    }
}

impl Default for ErrorDiffusionResult {
    fn default() -> Self {
        Self {
            success: false,
            diffusion_type: DiffusionType::default(),
            output_frames: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for ErrorDiffusionConfig {
    fn default() -> Self {
        Self {
            diffusion_type: DiffusionType::default(),
            intensity: 0.5,
            iterations: 25,
            seed: None,
            preserve_metadata: true,
            output_format: super::databend::OutputFormat::Png,
            diffusion_parameters: DiffusionParameters::default(),
        }
    }
}

impl Default for DiffusionParameters {
    fn default() -> Self {
        Self {
            error_probability: 0.1,
            corruption_rate: 0.05,
            data_loss_rate: 0.02,
            bit_flip_probability: 0.05,
            compression_artifacts: 0.1,
            network_latency: 0.1,
            packet_loss_rate: 0.01,
            temporal_drift: 0.05,
            spatial_distortion: 0.1,
            noise_level: 0.1,
        }
    }
}
