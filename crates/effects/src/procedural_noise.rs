use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ProceduralNoiseEffect {
    pub id: String,
    pub name: String,
    pub noise_type: Arc<RwLock<NoiseType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<NoiseEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<NoiseEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Worley,
    Fractal,
    Turbulence,
    Celluar,
    Value,
    Gradient,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum NoiseEvent {
    NoiseStarted,
    NoiseProgress(f32),
    NoiseCompleted(NoiseResult),
    Error(String),
    FrameProcessed(usize),
    LayerProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct NoiseResult {
    pub success: bool,
    pub noise_type: NoiseType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NoiseConfig {
    pub noise_type: NoiseType,
    pub width: u32,
    pub height: u32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub scale: f32,
    pub seed: Option<u32>,
    pub output_format: super::databend::OutputFormat,
    pub color_mode: ColorMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorMode {
    Grayscale,
    RGB,
    RGBA,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct NoiseLayer {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub offset: f32,
    pub enabled: bool,
}

impl ProceduralNoiseEffect {
    pub fn new(id: String, name: String, noise_type: NoiseType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            noise_type: Arc::new(RwLock::new(noise_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, config: NoiseConfig) -> Result<NoiseResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(NoiseEvent::NoiseStarted);
        let start_time = std::time::Instant::now();

        let result = match config.noise_type {
            NoiseType::Perlin => self.apply_perlin_noise(&config).await,
            NoiseType::Simplex => self.apply_simplex_noise(&config).await,
            NoiseType::Worley => self.apply_worley_noise(&config).await,
            NoiseType::Fractal => self.apply_fractal_noise(&config).await,
            NoiseType::Turbulence => self.apply_turbulence_noise(&config).await,
            NoiseType::Celluar => self.apply_celluar_noise(&config).await,
            NoiseType::Value => self.apply_value_noise(&config).await,
            NoiseType::Gradient => self.apply_gradient_noise(&config).await,
            NoiseType::Custom(_) => self.apply_custom_noise(&config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let metadata = self.generate_metadata(&config);
                let _ = self.event_sender.send(NoiseEvent::NoiseCompleted(NoiseResult {
                    success: true,
                    noise_type: config.noise_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                }));

                Ok(NoiseResult {
                    success: true,
                    noise_type: config.noise_type.clone(),
                    output_data,
                    metadata,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Procedural noise effect failed: {}", e);
                let _ = self.event_sender.send(NoiseEvent::Error(error_msg.clone()));

                Ok(NoiseResult {
                    success: false,
                    noise_type: config.noise_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_perlin_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;RGBA
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.perlin_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(output_data)
    }

    fn perlin_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for _ in 0..octaves {
            total += self.fade(self.interpolated_noise(x * frequency, y * frequency, seed)) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        total / max_value
    }

    fn interpolated_noise(&self, x: f32, y: f32, seed: u32) -> f32 {
        let int_x = x.floor() as i32;
        let int_y = y.floor() as i32;
        let frac_x = x - int_x as f32;
        let frac_y = y - int_y as f32;
        
        let a = self.pseudo_random(int_x, int_y, seed);
        let b = self.pseudo_random(int_x + 1, int_y, seed);
        let c = self.pseudo_random(int_x, int_y + 1, seed);
        let d = self.pseudo_random(int_x + 1, int_y + 1, seed);
        
        let i1 = self.interpolate(a, b, frac_x);
        let i2 = self.interpolate(c, d, frac_x);
        
        self.interpolate(i1, i2, frac_y)
    }

    fn pseudo_random(&self, x: i32, y: i32, seed: u32) -> f32 {
        let n = x + y * 57 + seed as i32 * 131;
        ((n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff) as f32 / 1073741824.0
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn interpolate(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    async fn apply_simplex_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.simplex_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        Ok(output_data)
    }

    fn simplex_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for _ in 0..octaves {
            total += self.simplex_noise_single(x * frequency, y * frequency, seed) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        total / max_value
    }

    fn simplex_noise_single(&self, x: f32, y: f32, seed: u32) -> f32 {
        let s = (x + y) * 0.3660254037844386;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;
        let t = (i + j) as f32 * 0.211324865405187;
        let x0 = x + t - i as f32;
        let y0 = y + t - j as f32;
        
        let hash = self.hash2d(i, j, seed);
        let grad_x = (hash & 0xFF) as f32 / 127.0 - 1.0;
        let grad_y = ((hash >> 8) & 0xFF) as f32 / 127.0 - 1.0;
        
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 < 0.0 {
            0.0
        } else {
            t0 * t0 * t0 * t0 * (grad_x * x0 + grad_y * y0)
        }
    }

    fn hash2d(&self, x: i32, y: i32, seed: u32) -> u32 {
        let mut h = seed;
        h ^= x as u32;
        h *= 374761393;
        h ^= y as u32;
        h *= 668265263;
        h
    }

    async fn apply_worley_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.worley_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
        }

        Ok(output_data)
    }

    fn worley_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for _ in 0..octaves {
            total += self.worley_noise_single(x * frequency, y * frequency, seed) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        total / max_value
    }

    fn worley_noise_single(&self, x: f32, y: f32, seed: u32) -> f32 {
        let cell_x = x.floor() as i32;
        let cell_y = y.floor() as i32;
        
        let mut min_distance = f32::INFINITY;
        
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_x = cell_x + dx;
                let neighbor_y = cell_y + dy;
                
                let hash = self.hash2d(neighbor_x, neighbor_y, seed);
                let point_x = (hash & 0xFF) as f32 / 255.0;
                let point_y = ((hash >> 8) & 0xFF) as f32 / 255.0;
                
                let distance = ((x - (neighbor_x as f32 + point_x)).powi(2) + 
                              (y - (neighbor_y as f32 + point_y)).powi(2)).sqrt();
                
                if distance < min_distance {
                    min_distance = distance;
                }
            }
        }
        
        min_distance
    }

    async fn apply_fractal_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.fractal_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(4)).await;
        }

        Ok(output_data)
    }

    fn fractal_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for octave in 0..octaves {
            let octave_seed = seed + octave;
            total += self.perlin_noise_2d(x * frequency, y * frequency, 1, persistence, lacunarity, octave_seed) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        total / max_value
    }

    async fn apply_turbulence_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.turbulence_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(output_data)
    }

    fn turbulence_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let perlin_x = self.perlin_noise_2d(x, y, octaves, persistence, lacunarity, seed);
        let perlin_y = self.perlin_noise_2d(x + 100.0, y + 100.0, octaves, persistence, lacunarity, seed);
        
        (perlin_x * perlin_y).abs()
    }

    async fn apply_celluar_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.celluar_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(6)).await;
        }

        Ok(output_data)
    }

    fn celluar_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let cell_x = x.floor() as i32;
        let cell_y = y.floor() as i32;
        
        let hash = self.hash2d(cell_x, cell_y, seed);
        let cell_value = (hash & 0xFF) as f32 / 255.0;
        
        cell_value
    }

    async fn apply_value_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.value_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
        }

        Ok(output_data)
    }

    fn value_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let cell_x = x.floor() as i32;
        let cell_y = y.floor() as i32;
        
        let hash = self.hash2d(cell_x, cell_y, seed);
        let value = ((hash >> 16) & 0xFF) as f32 / 255.0;
        
        value
    }

    async fn apply_gradient_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.gradient_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(4)).await;
        }

        Ok(output_data)
    }

    fn gradient_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let cell_x = x.floor() as i32;
        let cell_y = y.floor() as i32;
        
        let hash = self.hash2d(cell_x, cell_y, seed);
        let grad_x = ((hash >> 24) & 0xFF) as f32 / 127.0 - 1.0;
        let grad_y = ((hash >> 16) & 0xFF) as f32 / 127.0 - 1.0;
        
        let dx = x - cell_x as f32 - 0.5;
        let dy = y - cell_y as f32 - 0.5;
        
        grad_x * dx + grad_y * dy
    }

    async fn apply_custom_noise(&self, config: &NoiseConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = Vec::new();
        let width = config.width as usize;
        let height = config.height as usize;
        let data_size = width * height * 4;
        
        output_data.resize(data_size, 0);
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.custom_noise_2d(
                    x as f32 / config.scale,
                    y as f32 / config.scale,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                    config.seed.unwrap_or(0),
                );
                
                let color = self.noise_to_color(noise_value, &config.color_mode);
                let pixel_index = (y * width + x) * 4;
                
                if pixel_index + 3 < output_data.len() {
                    output_data[pixel_index] = color.0;
                    output_data[pixel_index + 1] = color.1;
                    output_data[pixel_index + 2] = color.2;
                    output_data[pixel_index + 3] = color.3;
                }
            }
            
            let progress = ((y + 1) as f32 / height as f32) * 100.0;
            let _ = self.event_sender.send(NoiseEvent::NoiseProgress(progress));
            let _ = self.event_sender.send(NoiseEvent::FrameProcessed(y));
            
            tokio::time::sleep(std::time::Duration::from_millis(7)).await;
        }

        Ok(output_data)
    }

    fn custom_noise_2d(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32, seed: u32) -> f32 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for octave in 0..octaves {
            let octave_seed = seed + octave;
            let noise = self.perlin_noise_2d(x * frequency, y * frequency, 1, persistence, lacunarity, octave_seed);
            total += noise * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        total / max_value
    }

    fn noise_to_color(&self, noise_value: f32, color_mode: &ColorMode) -> (u8, u8, u8, u8) {
        let normalized = ((noise_value + 1.0) * 0.5).clamp(0.0, 1.0);
        
        match color_mode {
            ColorMode::Grayscale => {
                let value = (normalized * 255.0) as u8;
                (value, value, value, 255)
            },
            ColorMode::RGB => {
                let value = (normalized * 255.0) as u8;
                (value, value, value, 255)
            },
            ColorMode::RGBA => {
                let value = (normalized * 255.0) as u8;
                (value, value, value, value)
            },
            ColorMode::Custom(_) => {
                let value = (normalized * 255.0) as u8;
                (value, value, value, 255)
            },
        }
    }

    fn generate_metadata(&self, config: &NoiseConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("noise_type".to_string(), format!("{:?}", config.noise_type));
        metadata.insert("width".to_string(), config.width.to_string());
        metadata.insert("height".to_string(), config.height.to_string());
        metadata.insert("octaves".to_string(), config.octaves.to_string());
        metadata.insert("persistence".to_string(), format!("{:.2}", config.persistence));
        metadata.insert("lacunarity".to_string(), format!("{:.2}", config.lacunarity));
        metadata.insert("scale".to_string(), format!("{:.2}", config.scale));
        metadata.insert("seed".to_string(), config.seed.map(|s| s.to_string()).unwrap_or("random".to_string()));
        metadata.insert("output_format".to_string(), format!("{:?}", config.output_format));
        metadata.insert("color_mode".to_string(), format!("{:?}", config.color_mode));
        
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

    pub fn set_noise_type(&self, noise_type: NoiseType) {
        let mut current_type = self.noise_type.write();
        *current_type = noise_type;
    }

    pub fn get_noise_type(&self) -> NoiseType {
        self.noise_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<NoiseEvent> {
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

    pub fn get_supported_noise_types(&self) -> Vec<NoiseType> {
        vec![
            NoiseType::Perlin,
            NoiseType::Simplex,
            NoiseType::Worley,
            NoiseType::Fractal,
            NoiseType::Turbulence,
            NoiseType::Celluar,
            NoiseType::Value,
            NoiseType::Gradient,
        ]
    }

    pub fn can_apply_noise_type(&self, noise_type: &NoiseType) -> bool {
        self.get_supported_noise_types().contains(noise_type)
    }

    pub fn clone_effect(&self) -> ProceduralNoiseEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_noise_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, width: u32, height: u32, config: &NoiseConfig) -> std::time::Duration {
        let pixel_count = width * height;
        let base_time_ms = match config.noise_type {
            NoiseType::Perlin => 2.0,
            NoiseType::Simplex => 3.0,
            NoiseType::Worley => 4.0,
            NoiseType::Fractal => 5.0,
            NoiseType::Turbulence => 6.0,
            NoiseType::Celluar => 3.0,
            NoiseType::Value => 2.0,
            NoiseType::Gradient => 3.0,
            NoiseType::Custom(_) => 4.0,
        };

        let time_per_pixel = base_time_ms / 1000.0;
        let total_time = pixel_count as f64 * time_per_pixel;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> NoiseConfig {
        match preset_name {
            "clouds" => NoiseConfig {
                noise_type: self.get_noise_type(),
                width: 512,
                height: 512,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
                scale: 50.0,
                seed: None,
                output_format: super::databend::OutputFormat::Png,
                color_mode: ColorMode::RGBA,
            },
            "marble" => NoiseConfig {
                noise_type: self.get_noise_type(),
                width: 512,
                height: 512,
                octaves: 6,
                persistence: 0.3,
                lacunarity: 2.0,
                scale: 30.0,
                seed: None,
                output_format: super::databend::OutputFormat::Png,
                color_mode: ColorMode::RGB,
            },
            "wood" => NoiseConfig {
                noise_type: self.get_noise_type(),
                width: 512,
                height: 512,
                octaves: 3,
                persistence: 0.7,
                lacunarity: 1.5,
                scale: 20.0,
                seed: None,
                output_format: super::databend::OutputFormat::Png,
                color_mode: ColorMode::RGB,
            },
            "terrain" => NoiseConfig {
                noise_type: self.get_noise_type(),
                width: 1024,
                height: 1024,
                octaves: 8,
                persistence: 0.6,
                lacunarity: 2.5,
                scale: 100.0,
                seed: None,
                output_format: super::databend::OutputFormat::Png,
                color_mode: ColorMode::Grayscale,
            },
            _ => NoiseConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "clouds".to_string(),
            "marble".to_string(),
            "wood".to_string(),
            "terrain".to_string(),
        ]
    }
}

impl Default for ProceduralNoiseEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Procedural Noise Effect".to_string(),
            NoiseType::Perlin,
        )
    }
}

impl Default for NoiseType {
    fn default() -> Self {
        NoiseType::Perlin
    }
}

impl Default for NoiseEvent {
    fn default() -> Self {
        NoiseEvent::NoiseStarted
    }
}

impl Default for NoiseResult {
    fn default() -> Self {
        Self {
            success: false,
            noise_type: NoiseType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            noise_type: NoiseType::default(),
            width: 512,
            height: 512,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
            scale: 50.0,
            seed: None,
            output_format: super::databend::OutputFormat::Png,
            color_mode: ColorMode::RGB,
        }
    }
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::RGB
    }
}

impl Default for NoiseLayer {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            phase: 0.0,
            offset: 0.0,
            enabled: true,
        }
    }
}
