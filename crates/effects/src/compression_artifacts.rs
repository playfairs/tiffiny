use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CompressionArtifactsEffect {
    pub id: String,
    pub name: String,
    pub artifact_type: Arc<RwLock<ArtifactType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<CompressionArtifactsEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<CompressionArtifactsEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactType {
    Jpeg,
    Mpeg,
    H264,
    H265,
    Vp9,
    Av1,
    WebP,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum CompressionArtifactsEvent {
    ArtifactsStarted,
    ArtifactsProgress(f32),
    ArtifactsCompleted(CompressionArtifactsResult),
    Error(String),
    FrameProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct CompressionArtifactsResult {
    pub success: bool,
    pub artifact_type: ArtifactType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompressionArtifactsConfig {
    pub artifact_type: ArtifactType,
    pub quality: u8,
    pub compression_level: u8,
    pub block_size: u32,
    pub quantization: bool,
    pub chroma_subsampling: ChromaSubsampling,
    pub artifacts_intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChromaSubsampling {
    None,
    YUV420,
    YUV422,
    YUV444,
    Custom(String),
}

impl CompressionArtifactsEffect {
    pub fn new(id: String, name: String, artifact_type: ArtifactType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            artifact_type: Arc::new(RwLock::new(artifact_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_data: &[u8], config: CompressionArtifactsConfig) -> Result<CompressionArtifactsResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsStarted);
        let start_time = std::time::Instant::now();

        let result = match config.artifact_type {
            ArtifactType::Jpeg => self.apply_jpeg_artifacts(input_data, &config).await,
            ArtifactType::Mpeg => self.apply_mpeg_artifacts(input_data, &config).await,
            ArtifactType::H264 => self.apply_h264_artifacts(input_data, &config).await,
            ArtifactType::H265 => self.apply_h265_artifacts(input_data, &config).await,
            ArtifactType::Vp9 => self.apply_vp9_artifacts(input_data, &config).await,
            ArtifactType::Av1 => self.apply_av1_artifacts(input_data, &config).await,
            ArtifactType::WebP => self.apply_webp_artifacts(input_data, &config).await,
            ArtifactType::Custom(_) => self.apply_custom_artifacts(input_data, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsCompleted(CompressionArtifactsResult {
                    success: true,
                    artifact_type: config.artifact_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(CompressionArtifactsResult {
                    success: true,
                    artifact_type: config.artifact_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Compression artifacts effect failed: {}", e);
                let _ = self.event_sender.send(CompressionArtifactsEvent::Error(error_msg.clone()));

                Ok(CompressionArtifactsResult {
                    success: false,
                    artifact_type: config.artifact_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_jpeg_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
Simulate JPEG compression artifacts
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        let block_size = config.block_size as usize;
        let artifact_intensity = config.artifacts_intensity;
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            for block_start in (0..data_len).step_by(block_size) {
                let block_end = (block_start + block_size).min(data_len);
                
                if block_end <= data_len && rand::random::<f32>() < artifact_intensity {
                    let block = &mut output_data[block_start..block_end];
                    
                    let avg = block.iter().sum::<u8>() as f32 / block.len() as f32;
                    
                    for byte in block.iter_mut() {
                        *byte = (*byte as f32 * 0.7 + avg * 0.3).round() as u8;
                    }
                }
            }
            
            if rand::random::<f32>() < artifact_intensity * 0.5 {
                for byte in output_data.iter_mut() {
                    if rand::random::<f32>() < 0.1 {
                        *byte = (*byte as f32 * 0.9 + rand::random::<f32>() * 0.1).round() as u8;
                    }
                }
            }
            
            if rand::random::<f32>() < artifact_intensity * 0.3 {
                for block_start in (0..data_len).step_by(block_size * 2) {
                    let block_end = (block_start + block_size * 2).min(data_len);
                    
                    if block_end <= data_len {
                        for byte in output_data[block_start..block_end].iter_mut().step_by(8) {
                            *byte = (*byte / 8) * 8;
                        }
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(output_data)
    }

    async fn apply_mpeg_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                for frame_start in (0..data_len).step_by(data_len / 30) {
                    let frame_end = (frame_start + data_len / 30).min(data_len);
                    
                    if frame_end <= data_len {
                        for byte in output_data[frame_start..frame_end].iter_mut() {
                            if rand::random::<f32>() < 0.05 {
                                *byte = (*byte as f32 * 0.8).round() as u8;
                            }
                        }
                    }
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.4 {
                for byte in output_data.iter_mut() {
                    if rand::random::<f32>() < 0.02 {
                        *byte = (*byte as f32 + (rand::random::<f32>() - 0.5) * 10.0).clamp(-128.0, 128.0)).round() as u8;
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        Ok(output_data)
    }

    async fn apply_h264_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                let macroblock_size = 16;
                
                for macroblock_start in (0..data_len).step_by(macroblock_size) {
                    let macroblock_end = (macroblock_start + macroblock_size).min(data_len);
                    
                    if macroblock_end <= data_len {
                        let macroblock = &mut output_data[macroblock_start..macroblock_end];
                        
                        for y in 0..4 {
                            for x in 0..4 {
                                let pixel_start = (y * 4 + x) * 4;
                                if pixel_start + 3 < macroblock.len() {
                                    let avg = (macroblock[pixel_start] + macroblock[pixel_start + 1] + 
                                              macroblock[pixel_start + 2] + macroblock[pixel_start + 3]) as f32 / 4.0;
                                    
                                    for pixel in pixel_start..pixel_start + 4 {
                                        if pixel < macroblock.len() {
                                            macroblock[pixel] = avg.round() as u8;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.6 {
                for byte in output_data.iter_mut() {
                    if rand::random::<f32>() < 0.1 {
                        *byte = (*byte as f32 * 1.1 + rand::random::<f32>() * 0.2).clamp(0.0, 255.0)).round() as u8;
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(18)).await;
        }

        Ok(output_data)
    }

    async fn apply_h265_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                let ctu_size = 32;
                
                for ctu_start in (0..data_len).step_by(ctu_size) {
                    let ctu_end = (ctu_start + ctu_size).min(data_len);
                    
                    if ctu_end <= data_len {
                        let ctu = &mut output_data[ctu_start..ctu_end];
                        
                        for y in 0..8 {
                            for x in 0..4 {
                                let pixel_start = (y * 4 + x) * 4;
                                if pixel_start + 3 < ctu.len() {
                                    let avg = (ctu[pixel_start] + ctu[pixel_start + 1] + 
                                              ctu[pixel_start + 2] + ctu[pixel_start + 3]) as f32 / 4.0;
                                    
                                    for pixel in pixel_start..pixel_start + 4 {
                                        if pixel < ctu.len() {
                                            ctu[pixel] = (avg * 0.6 + ctu[pixel] as f32 * 0.4).round() as u8;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.5 {
                for byte in output_data.iter_mut().step_by(4) {
                    if rand::random::<f32>() < 0.1 {
                        byte[0] = (byte[0] as f32 * 0.8).round() as u8;
                        byte[1] = (byte[1] as f32 * 0.8).round() as u8;
                        byte[2] = (byte[2] as f32 * 0.8).round() as u8;
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(output_data)
    }

    async fn apply_vp9_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                for byte in output_data.iter_mut().step_by(2) {
                    if rand::random::<f32>() < 0.05 {
                        byte[1] = (byte[1] as f32 * 0.9).round() as u8;
                    }
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.3 {
                for byte in output_data.iter_mut() {
                    if rand::random::<f32>() < 0.02 {
                        *byte = (*byte as f32 + (rand::random::<f32>() - 0.5) * 20.0).clamp(-128.0, 128.0)).round() as u8;
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        }

        Ok(output_data)
    }

    async fn apply_av1_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                for byte in output_data.iter_mut() {
                    let grain = (rand::random::<f32>() - 0.5) * config.artifacts_intensity * 10.0;
                    *byte = (*byte as f32 + grain).clamp(0.0, 255.0)).round() as u8;
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.7 {
                let superblock_size = 64;
                
                for superblock_start in (0..data_len).step_by(superblock_size) {
                    let superblock_end = (superblock_start + superblock_size).min(data_len);
                    
                    if superblock_end <= data_len {
                        let superblock = &mut output_data[superblock_start..superblock_end];
                        
                        for i in 0..superblock.len() {
                            superblock[i] = (superblock[i] as f32 * 0.7).round() as u8;
                        }
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(22)).await;
        }

        Ok(output_data)
    }

    async fn apply_webp_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            if rand::random::<f32>() < config.artifacts_intensity {
                for byte in output_data.iter_mut().step_by(3) {
                    if rand::random::<f32>() < 0.1 {
                        byte[1] = (byte[1] as f32 * 0.8 + byte[0] as f32 * 0.2).round() as u8;
                        byte[2] = (byte[2] as f32 * 0.8 + byte[1] as f32 * 0.2).round() as u8;
                    }
                }
            }
            
            if rand::random::<f32>() < config.artifacts_intensity * 0.4 {
                for byte in output_data.iter_mut() {
                    if rand::random::<f32>() < 0.05 {
                        let pattern = (byte as f32 * 0.5).round() as u8;
                        *byte = if rand::random::<f32>() < 0.5 { pattern } else { 255 - pattern };
                    }
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(14)).await;
        }

        Ok(output_data)
    }

    async fn apply_custom_artifacts(&self, input_data: &[u8], config: &CompressionArtifactsConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        for i in 0..100 {
            let progress = (i as f32 / 100.0) * 100.0;
            let _ = self.event_sender.send(CompressionArtifactsEvent::ArtifactsProgress(progress));
            
            for byte in output_data.iter_mut() {
                if rand::random::<f32>() < config.artifacts_intensity {
                    *byte = (*byte as f32 * (1.0 + (rand::random::<f32>() - 0.5) * config.artifacts_intensity)).round() as u8;
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(17)).await;
        }

        Ok(output_data)
    }

    fn generate_metadata(&self, config: &CompressionArtifactsConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("artifact_type".to_string(), format!("{:?}", config.artifact_type));
        metadata.insert("quality".to_string(), config.quality.to_string());
        metadata.insert("compression_level".to_string(), config.compression_level.to_string());
        metadata.insert("block_size".to_string(), config.block_size.to_string());
        metadata.insert("quantization".to_string(), config.quantization.to_string());
        metadata.insert("chroma_subsampling".to_string(), format!("{:?}", config.chroma_subsampling));
        metadata.insert("artifacts_intensity".to_string(), format!("{:.2}", config.artifacts_intensity));
        
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

    pub fn set_artifact_type(&self, artifact_type: ArtifactType) {
        let mut current_type = self.artifact_type.write();
        *current_type = artifact_type;
    }

    pub fn get_artifact_type(&self) -> ArtifactType {
        self.artifact_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<CompressionArtifactsEvent> {
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

    pub fn get_supported_artifact_types(&self) -> Vec<ArtifactType> {
        vec![
            ArtifactType::Jpeg,
            ArtifactType::Mpeg,
            ArtifactType::H264,
            ArtifactType::H265,
            ArtifactType::Vp9,
            ArtifactType::Av1,
            ArtifactType::WebP,
        ]
    }

    pub fn can_apply_artifact_type(&self, artifact_type: &ArtifactType) -> bool {
        self.get_supported_artifact_types().contains(artifact_type)
    }

    pub fn clone_effect(&self) -> CompressionArtifactsEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_artifact_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, input_size: usize, config: &CompressionArtifactsConfig) -> std::time::Duration {
        let base_time_ms = match config.artifact_type {
            ArtifactType::Jpeg => 25.0,
            ArtifactType::Mpeg => 35.0,
            ArtifactType::H264 => 45.0,
            ArtifactType::H265 => 55.0,
            ArtifactType::Vp9 => 40.0,
            ArtifactType::Av1 => 50.0,
            ArtifactType::WebP => 30.0,
            ArtifactType::Custom(_) => 40.0,
        };

        let time_per_byte = base_time_ms / 1000.0;
        let total_time = input_size as f64 * time_per_byte;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> CompressionArtifactsConfig {
        match preset_name {
            "light" => CompressionArtifactsConfig {
                artifact_type: self.get_artifact_type(),
                quality: 70,
                compression_level: 3,
                block_size: 16,
                quantization: false,
                chroma_subsampling: ChromaSubsampling::YUV420,
                artifacts_intensity: 0.2,
            },
            "medium" => CompressionArtifactsConfig {
                artifact_type: self.get_artifact_type(),
                quality: 50,
                compression_level: 5,
                block_size: 16,
                quantization: true,
                chroma_subsampling: ChromaSubsampling::YUV420,
                artifacts_intensity: 0.5,
            },
            "heavy" => CompressionArtifactsConfig {
                artifact_type: self.get_artifact_type(),
                quality: 30,
                compression_level: 7,
                block_size: 8,
                quantization: true,
                chroma_subsampling: ChromaSubsampling::YUV422,
                artifacts_intensity: 0.8,
            },
            "extreme" => CompressionArtifactsConfig {
                artifact_type: self.get_artifact_type(),
                quality: 10,
                compression_level: 10,
                block_size: 4,
                quantization: true,
                chroma_subsampling: ChromaSubsampling::YUV444,
                artifacts_intensity: 1.0,
            },
            _ => CompressionArtifactsConfig::default(),
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

impl Default for CompressionArtifactsEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Compression Artifacts Effect".to_string(),
            ArtifactType::Jpeg,
        )
    }
}

impl Default for ArtifactType {
    fn default() -> Self {
        ArtifactType::Jpeg
    }
}

impl Default for CompressionArtifactsEvent {
    fn default() -> Self {
        CompressionArtifactsEvent::ArtifactsStarted
    }
}

impl Default for CompressionArtifactsResult {
    fn default() -> Self {
        Self {
            success: false,
            artifact_type: ArtifactType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for CompressionArtifactsConfig {
    fn default() -> Self {
        Self {
            artifact_type: ArtifactType::default(),
            quality: 50,
            compression_level: 5,
            block_size: 16,
            quantization: false,
            chroma_subsampling: ChromaSubsampling::YUV420,
            artifacts_intensity: 0.5,
        }
    }
}

impl Default for ChromaSubsampling {
    fn default() -> Self {
        ChromaSubsampling::YUV420
    }
}
