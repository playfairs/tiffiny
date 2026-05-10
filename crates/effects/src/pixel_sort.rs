use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct PixelSortEffect {
    pub id: String,
    pub name: String,
    pub sort_type: Arc<RwLock<PixelSortType>>,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub event_sender: mpsc::UnboundedSender<PixelSortEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PixelSortEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PixelSortType {
    Brightness,
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
    Alpha,
    Luminance,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum PixelSortEvent {
    SortStarted,
    SortProgress(f32),
    SortCompleted(PixelSortResult),
    Error(String),
    RowProcessed(usize),
    ColumnProcessed(usize),
}

#[derive(Debug, Clone)]
pub struct PixelSortResult {
    pub success: bool,
    pub sort_type: PixelSortType,
    pub output_data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PixelSortConfig {
    pub sort_type: PixelSortType,
    pub direction: SortDirection,
    pub threshold: f32,
    pub threshold_mode: ThresholdMode,
    pub black_threshold: f32,
    pub white_threshold: f32,
    pub sort_mode: SortMode,
    pub interval: usize,
    pub reverse: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Horizontal,
    Vertical,
    Diagonal,
    Radial,
    Spiral,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdMode {
    None,
    Brightness,
    Hue,
    Saturation,
    Luminance,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Normal,
    Random,
    Checkerboard,
    Diagonal,
    Circular,
    Custom(String),
}

impl PixelSortEffect {
    pub fn new(id: String, name: String, sort_type: PixelSortType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            sort_type: Arc::new(RwLock::new(sort_type))),
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub async fn apply(&self, input_data: &[u8], config: PixelSortConfig) -> Result<PixelSortResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(PixelSortEvent::SortStarted);
        let start_time = std::time::Instant::now();

        let result = match config.sort_type {
            PixelSortType::Brightness => self.apply_brightness_sort(input_data, &config).await,
            PixelSortType::Hue => self.apply_hue_sort(input_data, &config).await,
            PixelSortType::Saturation => self.apply_saturation_sort(input_data, &config).await,
            PixelSortType::Red => self.apply_red_sort(input_data, &config).await,
            PixelSortType::Green => self.apply_green_sort(input_data, &config).await,
            PixelSortType::Blue => self.apply_blue_sort(input_data, &config).await,
            PixelSortType::Alpha => self.apply_alpha_sort(input_data, &config).await,
            PixelSortType::Luminance => self.apply_luminance_sort(input_data, &config).await,
            PixelSortType::Custom(_) => self.apply_custom_sort(input_data, &config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_data) => {
                let _ = self.event_sender.send(PixelSortEvent::SortCompleted(PixelSortResult {
                    success: true,
                    sort_type: config.sort_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                }));

                Ok(PixelSortResult {
                    success: true,
                    sort_type: config.sort_type.clone(),
                    output_data,
                    metadata: self.generate_metadata(&config),
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Pixel sort effect failed: {}", e);
                let _ = self.event_sender.send(PixelSortEvent::Error(error_msg.clone()));

                Ok(PixelSortResult {
                    success: false,
                    sort_type: config.sort_type.clone(),
                    output_data: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn apply_brightness_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
Simulate brightness-based pixel sorting
        let mut output_data = input_data.to_vec();
        let data_len = output_data.len();
        
        match config.direction {
            SortDirection::Horizontal => {
                self.sort_horizontal_by_brightness(&mut output_data, config).await?;
            },
            SortDirection::Vertical => {
                self.sort_vertical_by_brightness(&mut output_data, config).await?;
            },
            SortDirection::Diagonal => {
                self.sort_diagonal_by_brightness(&mut output_data, config).await?;
            },
            SortDirection::Radial => {
                self.sort_radial_by_brightness(&mut output_data, config).await?;
            },
            SortDirection::Spiral => {
                self.sort_spiral_by_brightness(&mut output_data, config).await?;
            },
            SortDirection::Custom(_) => {
                self.sort_horizontal_by_brightness(&mut output_data, config).await?;
            },
        }

        Ok(output_data)
    }

    async fn sort_horizontal_by_brightness(&self, data: &mut [u8], config: &PixelSortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        
        for y in 0..height {
            let row_start = y * width * 4;
            let row_end = row_start + width * 4;
            
            if row_end <= data.len() {
                let row = &mut data[row_start..row_end];
                self.sort_row_by_brightness(row, config);
                
                let progress = ((y + 1) as f32 / height as f32) * 100.0;
                let _ = self.event_sender.send(PixelSortEvent::SortProgress(progress));
                let _ = self.event_sender.send(PixelSortEvent::RowProcessed(y));
                
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        }

        Ok(())
    }

    async fn sort_vertical_by_brightness(&self, data: &mut [u8], config: &PixelSortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        
        for x in 0..width {
            let mut column_pixels = Vec::new();
            
            for y in 0..height {
                let pixel_start = (y * width + x) * 4;
                if pixel_start + 3 < data.len() {
                    column_pixels.extend_from_slice(&data[pixel_start..pixel_start + 4]);
                }
            }
            
            self.sort_pixels_by_brightness(&mut column_pixels, config);
            
            for y in 0..height {
                let pixel_start = (y * width + x) * 4;
                if pixel_start + 3 < data.len() && y * 4 < column_pixels.len() + 3 {
                    data[pixel_start..pixel_start + 4].copy_from_slice(&column_pixels[y * 4..y * 4 + 4]);
                }
            }
            
            let progress = ((x + 1) as f32 / width as f32) * 100.0;
            let _ = self.event_sender.send(PixelSortEvent::SortProgress(progress));
            let _ = self.event_sender.send(PixelSortEvent::ColumnProcessed(x));
            
            tokio::time::sleep(std::time::Duration::from_millis(6)).await;
        }

        Ok(())
    }

    async fn sort_diagonal_by_brightness(&self, data: &mut [u8], config: &PixelSortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        let diagonal_count = width + height - 1;
        
        for d in 0..diagonal_count {
            let mut diagonal_pixels = Vec::new();
            
            for i in 0..width.min(height) {
                let x = d.saturating_sub(i);
                let y = i;
                
                if x < width && y < height {
                    let pixel_start = (y * width + x) * 4;
                    if pixel_start + 3 < data.len() {
                        diagonal_pixels.extend_from_slice(&data[pixel_start..pixel_start + 4]);
                    }
                }
            }
            
            self.sort_pixels_by_brightness(&mut diagonal_pixels, config);
            
            for i in 0..width.min(height) {
                let x = d.saturating_sub(i);
                let y = i;
                
                if x < width && y < height {
                    let pixel_start = (y * width + x) * 4;
                    if pixel_start + 3 < data.len() && i * 4 < diagonal_pixels.len() + 3 {
                        data[pixel_start..pixel_start + 4].copy_from_slice(&diagonal_pixels[i * 4..i * 4 + 4]);
                    }
                }
            }
            
            let progress = ((d + 1) as f32 / diagonal_count as f32) * 100.0;
            let _ = self.event_sender.send(PixelSortEvent::SortProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(8)).await;
        }

        Ok(())
    }

    async fn sort_radial_by_brightness(&self, data: &mut [u8], config: &PixelSortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        let center_x = width / 2;
        let center_y = height / 2;
        let max_radius = center_x.min(center_y);
        
        for r in 0..max_radius {
            let mut ring_pixels = Vec::new();
            
            for angle in 0..360 {
                let x = center_x + (r as f32 * (angle as f32 * std::f32::consts::PI / 180.0)).cos() as i32;
                let y = center_y + (r as f32 * (angle as f32 * std::f32::consts::PI / 180.0)).sin() as i32;
                
                if x >= 0 && x < width && y >= 0 && y < height {
                    let pixel_start = (y * width + x) * 4;
                    if pixel_start + 3 < data.len() {
                        ring_pixels.extend_from_slice(&data[pixel_start..pixel_start + 4]);
                    }
                }
            }
            
            self.sort_pixels_by_brightness(&mut ring_pixels, config);
            
            for angle in 0..360 {
                let x = center_x + (r as f32 * (angle as f32 * std::f32::consts::PI / 180.0)).cos() as i32;
                let y = center_y + (r as f32 * (angle as f32 * std::f32::consts::PI / 180.0)).sin() as i32;
                
                if x >= 0 && x < width && y >= 0 && y < height {
                    let pixel_start = (y * width + x) * 4;
                    if pixel_start + 3 < data.len() && angle < ring_pixels.len() / 4 {
                        data[pixel_start..pixel_start + 4].copy_from_slice(&ring_pixels[angle * 4..angle * 4 + 4]);
                    }
                }
            }
            
            let progress = ((r + 1) as f32 / max_radius as f32) * 100.0;
            let _ = self.event_sender.send(PixelSortEvent::SortProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn sort_spiral_by_brightness(&self, data: &mut [u8], config: &PixelSortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let width = 100;
        let height = data.len() / (width * 4);
        let center_x = width / 2;
        let center_y = height / 2;
        let max_spiral = width * height;
        
        let mut x = center_x;
        let mut y = center_y;
        let mut dx = 0;
        let mut dy = = -1;
        let mut segment_length = 1;
        let mut segment_passed = 0;
        
        for i in 0..max_spiral {
            if x >= 0 && x < width && y >= 0 && y < height {
                let pixel_start = (y * width + x) * 4;
                if pixel_start + 3 < data.len() {
                    let mut segment_pixels = Vec::new();
                    
                    for j in 0..segment_length {
                        let segment_x = x + dx * j;
                        let segment_y = y + dy * j;
                        
                        if segment_x >= 0 && segment_x < width && segment_y >= 0 && segment_y < height {
                            let segment_pixel_start = (segment_y * width + segment_x) * 4;
                            if segment_pixel_start + 3 < data.len() {
                                segment_pixels.extend_from_slice(&data[segment_pixel_start..segment_pixel_start + 4]);
                            }
                        }
                    }
                    
                    self.sort_pixels_by_brightness(&mut segment_pixels, config);
                    
                    for j in 0..segment_length {
                        let segment_x = x + dx * j;
                        let segment_y = y + dy * j;
                        
                        if segment_x >= 0 && segment_x < width && segment_y >= 0 && segment_y < height {
                            let segment_pixel_start = (segment_y * width + segment_x) * 4;
                            if segment_pixel_start + 3 < data.len() && j < segment_pixels.len() / 4 {
                                data[segment_pixel_start..segment_pixel_start + 4].copy_from_slice(&segment_pixels[j * 4..j * 4 + 4]);
                            }
                        }
                    }
                }
            }
            
            x += dx * segment_length;
            y += dy * segment_length;
            segment_passed += 1;
            
            if segment_passed == 2 {
                segment_passed = 0;
                let temp = dx;
                dx = -dy;
                dy = temp;
                segment_length += 1;
            }
            
            let progress = ((i + 1) as f32 / max_spiral as f32) * 100.0;
            let _ = self.event_sender.send(PixelSortEvent::SortProgress(progress));
            
            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
        }

        Ok(())
    }

    fn sort_row_by_brightness(&self, row: &mut [u8], config: &PixelSortConfig) {
        let pixel_count = row.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < row.len() {
                pixels.push([
                    row[pixel_start],
                    row[pixel_start + 1],
                    row[pixel_start + 2],
                    row[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let brightness_a = self.calculate_brightness(a);
            let brightness_b = self.calculate_brightness(b);
            brightness_a.partial_cmp(&brightness_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        if config.threshold_mode != ThresholdMode::None {
            pixels.retain(|pixel| {
                let brightness = self.calculate_brightness(pixel);
                match config.threshold_mode {
                    ThresholdMode::Brightness => brightness >= config.threshold,
                    _ => true,
                }
            });
        }
        
        pixels.retain(|pixel| {
            let brightness = self.calculate_brightness(pixel);
            brightness >= config.black_threshold && brightness <= config.white_threshold
        });
        
        if config.reverse {
            pixels.reverse();
        }
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < row.len() {
                row[pixel_start] = pixel[0];
                row[pixel_start + 1] = pixel[1];
                row[pixel_start + 2] = pixel[2];
                row[pixel_start + 3] = pixel[3];
            }
        }
    }

    fn sort_pixels_by_brightness(&self, pixels: &mut [u8], config: &PixelSortConfig) {
        let pixel_count = pixels.len() / 4;
        let mut pixel_array = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < pixels.len() {
                pixel_array.push([
                    pixels[pixel_start],
                    pixels[pixel_start + 1],
                    pixels[pixel_start + 2],
                    pixels[pixel_start + 3],
                ]);
            }
        }
        
        pixel_array.sort_by(|a, b| {
            let brightness_a = self.calculate_brightness(a);
            let brightness_b = self.calculate_brightness(b);
            brightness_a.partial_cmp(&brightness_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        if config.reverse {
            pixel_array.reverse();
        }
        
        for (i, pixel) in pixel_array.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < pixels.len() {
                pixels[pixel_start] = pixel[0];
                pixels[pixel_start + 1] = pixel[1];
                pixels[pixel_start + 2] = pixel[2];
                pixels[pixel_start + 3] = pixel[3];
            }
        }
    }

    fn calculate_brightness(&self, pixel: &[u8]) -> f32 {
        if pixel.len() >= 3 {
            (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0
        } else {
            0.0
        }
    }

    async fn apply_hue_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        let pixel_count = output_data.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                pixels.push([
                    output_data[pixel_start],
                    output_data[pixel_start + 1],
                    output_data[pixel_start + 2],
                    output_data[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let hue_a = self.calculate_hue(a);
            let hue_b = self.calculate_hue(b);
            hue_a.partial_cmp(&hue_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                output_data[pixel_start] = pixel[0];
                output_data[pixel_start + 1] = pixel[1];
                output_data[pixel_start + 2] = pixel[2];
                output_data[pixel_start + 3] = pixel[3];
            }
        }
        
        Ok(output_data)
    }

    fn calculate_hue(&self, pixel: &[u8]) -> f32 {
        if pixel.len() >= 3 {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let delta = max - min;
            
            if delta == 0.0 {
                0.0
            } else if max == r {
                ((g - b) / delta + 6.0).rem_euclid(6.0) / 6.0
            } else if max == g {
                ((b - r) / delta + 2.0) / 6.0
            } else {
                ((r - g) / delta + 4.0) / 6.0
            }
        } else {
            0.0
        }
    }

    async fn apply_saturation_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        let pixel_count = output_data.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                pixels.push([
                    output_data[pixel_start],
                    output_data[pixel_start + 1],
                    output_data[pixel_start + 2],
                    output_data[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let saturation_a = self.calculate_saturation(a);
            let saturation_b = self.calculate_saturation(b);
            saturation_a.partial_cmp(&saturation_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                output_data[pixel_start] = pixel[0];
                output_data[pixel_start + 1] = pixel[1];
                output_data[pixel_start + 2] = pixel[2];
                output_data[pixel_start + 3] = pixel[3];
            }
        }
        
        Ok(output_data)
    }

    fn calculate_saturation(&self, pixel: &[u8]) -> f32 {
        if pixel.len() >= 3 {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let delta = max - min;
            
            if max == 0.0 {
                0.0
            } else {
                delta / max
            }
        } else {
            0.0
        }
    }

    async fn apply_red_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.apply_channel_sort(input_data, config, 0).await
    }

    async fn apply_green_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.apply_channel_sort(input_data, config, 1).await
    }

    async fn apply_blue_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.apply_channel_sort(input_data, config, 2).await
    }

    async fn apply_alpha_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.apply_channel_sort(input_data, config, 3).await
    }

    async fn apply_channel_sort(&self, input_data: &[u8], config: &PixelSortConfig, channel: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        let pixel_count = output_data.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                pixels.push([
                    output_data[pixel_start],
                    output_data[pixel_start + 1],
                    output_data[pixel_start + 2],
                    output_data[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let channel_a = if channel < a.len() { a[channel] } else { 0 };
            let channel_b = if channel < b.len() { b[channel] } else { 0 };
            channel_a.partial_cmp(&channel_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                output_data[pixel_start] = pixel[0];
                output_data[pixel_start + 1] = pixel[1];
                output_data[pixel_start + 2] = pixel[2];
                output_data[pixel_start + 3] = pixel[3];
            }
        }
        
        Ok(output_data)
    }

    async fn apply_luminance_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        let pixel_count = output_data.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                pixels.push([
                    output_data[pixel_start],
                    output_data[pixel_start + 1],
                    output_data[pixel_start + 2],
                    output_data[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let luminance_a = self.calculate_luminance(a);
            let luminance_b = self.calculate_luminance(b);
            luminance_a.partial_cmp(&luminance_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                output_data[pixel_start] = pixel[0];
                output_data[pixel_start + 1] = pixel[1];
                output_data[pixel_start + 2] = pixel[2];
                output_data[pixel_start + 3] = pixel[3];
            }
        }
        
        Ok(output_data)
    }

    fn calculate_luminance(&self, pixel: &[u8]) -> f32 {
        if pixel.len() >= 3 {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            
            0.299 * r + 0.587 * g + 0.114 * b
        } else {
            0.0
        }
    }

    async fn apply_custom_sort(&self, input_data: &[u8], config: &PixelSortConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut output_data = input_data.to_vec();
        
        let pixel_count = output_data.len() / 4;
        let mut pixels = Vec::new();
        
        for i in 0..pixel_count {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                pixels.push([
                    output_data[pixel_start],
                    output_data[pixel_start + 1],
                    output_data[pixel_start + 2],
                    output_data[pixel_start + 3],
                ]);
            }
        }
        
        pixels.sort_by(|a, b| {
            let value_a = self.custom_pixel_value(a);
            let value_b = self.custom_pixel_value(b);
            value_a.partial_cmp(&value_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (i, pixel) in pixels.iter().enumerate() {
            let pixel_start = i * 4;
            if pixel_start + 3 < output_data.len() {
                output_data[pixel_start] = pixel[0];
                output_data[pixel_start + 1] = pixel[1];
                output_data[pixel_start + 2] = pixel[2];
                output_data[pixel_start + 3] = pixel[3];
            }
        }
        
        Ok(output_data)
    }

    fn custom_pixel_value(&self, pixel: &[u8]) -> f32 {
        if pixel.len() >= 3 {
            (pixel[0] as f32 * 0.3 + pixel[1] as f32 * 0.5 + pixel[2] as f32 * 0.2)
        } else {
            0.0
        }
    }

    fn generate_metadata(&self, config: &PixelSortConfig) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert("sort_type".to_string(), format!("{:?}", config.sort_type));
        metadata.insert("direction".to_string(), format!("{:?}", config.direction));
        metadata.insert("threshold".to_string(), format!("{:.2}", config.threshold));
        metadata.insert("threshold_mode".to_string(), format!("{:?}", config.threshold_mode));
        metadata.insert("black_threshold".to_string(), format!("{:.2}", config.black_threshold));
        metadata.insert("white_threshold".to_string(), format!("{:.2}", config.white_threshold));
        metadata.insert("sort_mode".to_string(), format!("{:?}", config.sort_mode));
        metadata.insert("interval".to_string(), config.interval.to_string());
        metadata.insert("reverse".to_string(), config.reverse.to_string());
        
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

    pub fn set_sort_type(&self, sort_type: PixelSortType) {
        let mut current_type = self.sort_type.write();
        *current_type = sort_type;
    }

    pub fn get_sort_type(&self) -> PixelSortType {
        self.sort_type.read().clone()
    }

    pub async fn get_events(&mut self) -> Vec<PixelSortEvent> {
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

    pub fn get_supported_sort_types(&self) -> Vec<PixelSortType> {
        vec![
            PixelSortType::Brightness,
            PixelSortType::Hue,
            PixelSortType::Saturation,
            PixelSortType::Red,
            PixelSortType::Green,
            PixelSortType::Blue,
            PixelSortType::Alpha,
            PixelSortType::Luminance,
        ]
    }

    pub fn can_apply_sort_type(&self, sort_type: &PixelSortType) -> bool {
        self.get_supported_sort_types().contains(sort_type)
    }

    pub fn clone_effect(&self) -> PixelSortEffect {
        let mut new_effect = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_sort_type(),
        );

        let parameters = self.parameters.read();
        *new_effect.parameters = parameters.clone();

        new_effect
    }

    pub fn reset(&self) {
        let mut parameters = self.parameters.write();
        parameters.clear();
    }

    pub fn estimate_processing_time(&self, pixel_count: usize, config: &PixelSortConfig) -> std::time::Duration {
        let base_time_ms = match config.sort_type {
            PixelSortType::Brightness => 2.0,
            PixelSortType::Hue => 3.0,
            PixelSortType::Saturation => 2.5,
            PixelSortType::Red => 1.5,
            PixelSortType::Green => 1.5,
            PixelSortType::Blue => 1.5,
            PixelSortType::Alpha => 1.5,
            PixelSortType::Luminance => 2.0,
            PixelSortType::Custom(_) => 4.0,
        };

        let time_per_pixel = base_time_ms / 1000.0;
        let total_time = pixel_count as f64 * time_per_pixel;
        
        std::time::Duration::from_secs_f64(total_time)
    }

    pub fn create_preset(&self, preset_name: &str) -> PixelSortConfig {
        match preset_name {
            "subtle" => PixelSortConfig {
                sort_type: self.get_sort_type(),
                direction: SortDirection::Horizontal,
                threshold: 0.3,
                threshold_mode: ThresholdMode::Brightness,
                black_threshold: 0.1,
                white_threshold: 0.9,
                sort_mode: SortMode::Normal,
                interval: 1,
                reverse: false,
            },
            "moderate" => PixelSortConfig {
                sort_type: self.get_sort_type(),
                direction: SortDirection::Diagonal,
                threshold: 0.5,
                threshold_mode: ThresholdMode::Luminance,
                black_threshold: 0.0,
                white_threshold: 1.0,
                sort_mode: SortMode::Checkerboard,
                interval: 2,
                reverse: false,
            },
            "intense" => PixelSortConfig {
                sort_type: self.get_sort_type(),
                direction: SortDirection::Spiral,
                threshold: 0.7,
                threshold_mode: ThresholdMode::Hue,
                black_threshold: 0.0,
                white_threshold: 1.0,
                sort_mode: SortMode::Random,
                interval: 3,
                reverse: true,
            },
            _ => PixelSortConfig::default(),
        }
    }

    pub fn get_presets(&self) -> Vec<String> {
        vec![
            "subtle".to_string(),
            "moderate".to_string(),
            "intense".to_string(),
        ]
    }
}

impl Default for PixelSortEffect {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Pixel Sort Effect".to_string(),
            PixelSortType::Brightness,
        )
    }
}

impl Default for PixelSortType {
    fn default() -> Self {
        PixelSortType::Brightness
    }
}

impl Default for PixelSortEvent {
    fn default() -> Self {
        PixelSortEvent::SortStarted
    }
}

impl Default for PixelSortResult {
    fn default() -> Self {
        Self {
            success: false,
            sort_type: PixelSortType::default(),
            output_data: Vec::new(),
            metadata: std::collections::HashMap::new(),
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for PixelSortConfig {
    fn default() -> Self {
        Self {
            sort_type: PixelSortType::default(),
            direction: SortDirection::Horizontal,
            threshold: 0.5,
            threshold_mode: ThresholdMode::None,
            black_threshold: 0.0,
            white_threshold: 1.0,
            sort_mode: SortMode::Normal,
            interval: 1,
            reverse: false,
        }
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Horizontal
    }
}

impl Default for ThresholdMode {
    fn default() -> Self {
        ThresholdMode::None
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::Normal
    }
}
