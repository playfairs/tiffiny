use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Threshold {
    pub id: String,
    pub name: String,
    pub threshold_type: ThresholdType,
    pub threshold_value: Arc<RwLock<f32>>,
    pub lower_value: Arc<RwLock<f32>>,
    pub upper_value: Arc<RwLock<f32>>,
    pub invert: Arc<RwLock<bool>>,
    pub event_sender: mpsc::UnboundedSender<ThresholdEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ThresholdEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdType {
    Binary,
    Adaptive,
    Otsu,
    Niblack,
    Sauvola,
    Bernsen,
    MultiLevel,
    Color,
    Hysteresis,
    Local,
    Double,
}

#[derive(Debug, Clone)]
pub enum ThresholdEvent {
    ThresholdChanged(f32),
    LowerChanged(f32),
    UpperChanged(f32),
    InvertChanged(bool),
    TypeChanged(ThresholdType),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ThresholdResult {
    pub binary_image: crate::image_buffer::ImageBuffer,
    pub threshold_value: f32,
    pub method: String,
    pub parameters: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct AdaptiveThresholdParams {
    pub method: AdaptiveMethod,
    pub block_size: u32,
    pub c_constant: f32,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdaptiveMethod {
    Mean,
    Gaussian,
    WeightedMean,
}

#[derive(Debug, Clone)]
pub struct OtsuParams {
    pub bin_count: u8,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct NiblackParams {
    pub window_size: u32,
    pub k: f32,
    pub r: f32,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct SauvolaParams {
    pub window_size: u32,
    pub k: f32,
    pub r: f32,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct MultiLevelParams {
    pub levels: Vec<f32>,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct ColorThresholdParams {
    pub color_space: crate::color_space::ColorSpace,
    pub threshold: f32,
    pub channel: String,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct HysteresisParams {
    pub low_threshold: f32,
    pub high_threshold: f32,
    pub invert: bool,
}

impl Threshold {
    pub fn new(id: String, name: String, threshold_type: ThresholdType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            threshold_type,
            threshold_value: Arc::new(RwLock::new(128.0)),
            lower_value: Arc::new(RwLock::new(0.0)),
            upper_value: Arc::new(RwLock::new(255.0)),
            invert: Arc::new(RwLock::new(false)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn apply(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        match self.threshold_type {
            ThresholdType::Binary => self.apply_binary_threshold(input),
            ThresholdType::Adaptive => self.apply_adaptive_threshold(input),
            ThresholdType::Otsu => self.apply_otsu_threshold(input),
            ThresholdType::Niblack => self.apply_niblack_threshold(input),
            ThresholdType::Sauvola => self.apply_sauvola_threshold(input),
            ThresholdType::Bernsen => self.apply_bernsen_threshold(input),
            ThresholdType::MultiLevel => self.apply_multilevel_threshold(input),
            ThresholdType::Color => self.apply_color_threshold(input),
            ThresholdType::Hysteresis => self.apply_hysteresis_threshold(input),
            ThresholdType::Local => self.apply_local_threshold(input),
            ThresholdType::Double => self.apply_double_threshold(input),
        }
    }

    fn apply_binary_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let threshold = *self.threshold_value.read();
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > threshold) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: threshold,
            method: "Binary".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_adaptive_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let threshold = *self.threshold_value.read();
        let invert = *self.invert.read();
        
Default parameters
        let block_size = 11;
        let c_constant = 2.0;
        let method = AdaptiveMethod::Mean;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        let mut grayscale = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let gray_pixel = crate::image_buffer::Pixel::gray(gray);
                    grayscale.set_pixel(x, y, gray_pixel);
                }
            }
        }
        
        for y in (block_size/2)..(input.height - block_size/2) {
            for x in (block_size/2)..(input.width - block_size/2) {
                let local_mean = self.calculate_local_mean(&grayscale, x, y, block_size);
                let threshold = local_mean - c_constant;
                
                for dy in -(block_size/2)..=(block_size/2) {
                    for dx in -(block_size/2)..=(block_size/2) {
                        let src_x = x + dx;
                        let src_y = y + dy;
                        
                        if let Some(pixel) = grayscale.get_pixel(src_x, src_y) {
                            let gray = pixel.luma();
                            let binary_value = if (gray > threshold) ^ invert { 255.0 } else { 0.0 };
                            
                            let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                            output.set_pixel(src_x, src_y, binary_pixel);
                        }
                    }
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: threshold,
            method: "Adaptive".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("block_size".to_string(), block_size as f32),
                ("c_constant".to_string(), c_constant),
                ("method".to_string(), format!("{:?}", method)),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn calculate_local_mean(&self, image: &crate::image_buffer::ImageBuffer, center_x: u32, center_y: u32, block_size: u32) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;
        
        let half_block = block_size / 2;
        
        for dy in -(half_block as i32)..=(half_block as i32) {
            for dx in -(half_block as i32)..=(half_block as i32) {
                let src_x = center_x as i32 + dx;
                let src_y = center_y as i32 + dy;
                
                if let Some(pixel) = image.get_pixel(src_x as u32, src_y as u32) {
                    sum += pixel.luma();
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }

    fn apply_otsu_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let mut histogram = [0u32; 256];
        let mut total_pixels = 0u32;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma() as u8;
                    histogram[gray as usize] += 1;
                    total_pixels += 1;
                }
            }
        }
        
        let (threshold, _) = self.calculate_otsu_threshold_internal(&histogram, total_pixels);
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > threshold as f32) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: threshold as f32,
            method: "Otsu".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold as f32),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn calculate_otsu_threshold_internal(&self, histogram: &[u32; 256], total_pixels: u32) -> (u8, f64) {
        let mut sum = 0u64;
        let mut sum_squared = 0u64;
        
        for (i, &count) in histogram.iter().enumerate() {
            sum += i as u64 * count as u64;
            sum_squared += (i as u64 * i as u64) * count as u64;
        }
        
        let total = total_pixels as f64;
        let mean = sum as f64 / total;
        let variance = (sum_squared as f64 / total) - (mean * mean);
        
        let mut best_threshold = 0u8;
        let mut best_variance = f64::INFINITY;
        
        for threshold in 1..256 {
            let mut w0 = 0.0;
            let mut w1 = 0.0;
            let mut mu0 = 0.0;
            let mut mu1 = 0.0;
            
            for i in 0..threshold {
                let count = histogram[i] as f64;
                w0 += count;
                mu0 += i as f64 * count;
            }
            
            for i in threshold..256 {
                let count = histogram[i] as f64;
                w1 += count;
                mu1 += i as f64 * count;
            }
            
            if w0 > 0.0 && w1 > 0.0 {
                mu0 /= w0;
                mu1 /= w1;
                
                let variance0 = 0.0;
                let variance1 = 0.0;
                
                for i in 0..threshold {
                    let count = histogram[i] as f64;
                    variance0 += (i as f64 - mu0) * (i as f64 - mu0) * (count / w0);
                    variance1 += (i as f64 - mu1) * (i as f64 - mu1) * (count / w1);
                }
                
                let current_variance = w0 * variance0 + w1 * variance1;
                
                if current_variance < best_variance {
                    best_variance = current_variance;
                    best_threshold = threshold as u8;
                }
            }
        }
        
        (best_threshold, best_variance)
    }

    fn apply_niblack_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let window_size = 51;
        let k = 0.2;
        let r = 128.0;
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        let half_window = window_size / 2;
        
        for y in half_window..(input.height - half_window) {
            for x in half_window..(input.width - half_window) {
                let local_mean = self.calculate_local_mean(input, x, y, window_size);
                let local_variance = self.calculate_local_variance(input, x, y, window_size);
                
                let threshold = local_mean + k * ((local_variance / r) - 1.0).sqrt();
                
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > threshold) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: 0.0,
            method: "Niblack".to_string(),
            parameters: std::collections::HashMap::from([
                ("window_size".to_string(), window_size as f32),
                ("k".to_string(), k),
                ("r".to_string(), r),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn calculate_local_variance(&self, image: &crate::image_buffer::ImageBuffer, center_x: u32, center_y: u32, window_size: u32) -> f32 {
        let mut sum = 0.0;
        let mut sum_squared = 0.0;
        let mut count = 0.0;
        
        let half_window = window_size / 2;
        
        for dy in -(half_window as i32)..=(half_window as i32) {
            for dx in -(half_window as i32)..=(half_window as i32) {
                let src_x = center_x as i32 + dx;
                let src_y = center_y as i32 + dy;
                
                if let Some(pixel) = image.get_pixel(src_x as u32, src_y as u32) {
                    let gray = pixel.luma();
                    sum += gray;
                    sum_squared += gray * gray;
                    count += 1.0;
                }
            }
        }
        
        if count > 0.0 {
            (sum_squared / count) - (sum / count) * (sum / count)
        } else {
            0.0
        }
    }

    fn apply_sauvola_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let window_size = 51;
        let k = 0.34;
        let r = 128.0;
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        let half_window = window_size / 2;
        
        for y in half_window..(input.height - half_window) {
            for x in half_window..(input.width - half_window) {
                let local_mean = self.calculate_local_mean(input, x, y, window_size);
                let local_variance = self.calculate_local_variance(input, x, y, window_size);
                
                let threshold = local_mean * (1.0 + k * ((local_variance / r) - 1.0));
                
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > threshold) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: 0.0,
            method: "Sauvola".to_string(),
            parameters: std::collections::HashMap::from([
                ("window_size".to_string(), window_size as f32),
                ("k".to_string(), k),
                ("r".to_string(), r),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_bernsen_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 1..(input.height - 1) {
            for x in 1..(input.width - 1) {
                let p1 = input.get_pixel(x - 1, y - 1).map_or(0.0, |p| p.luma());
                let p2 = input.get_pixel(x, y - 1).map_or(0.0, |p| p.luma());
                let p3 = input.get_pixel(x + 1, y - 1).map_or(0.0, |p| p.luma());
                let p4 = input.get_pixel(x + 1, y + 1).map_or(0.0, |p| p.luma());
                let p5 = input.get_pixel(x - 1, y).map_or(0.0, |p| p.luma());
                let p6 = input.get_pixel(x, y + 1).map_or(0.0, |p| p.luma());
                let p7 = input.get_pixel(x + 1, y + 1).map_or(0.0, |p| p.luma());
                let p8 = input.get_pixel(x, y).map_or(0.0, |p| p.luma());
                
                let a = p1 + p3 + p5 + p7 + 2.0 * (p2 + p4 + p6 + p8);
                let b = p2 + p4 + p6 + p8 + 2.0 * (p1 + p3 + p5 + p7);
                
                let binary_value = if a > b { 255.0 } else { 0.0 };
                let final_value = if binary_value ^ invert { 255.0 } else { 0.0 };
                
                let binary_pixel = crate::image_buffer::Pixel::gray(final_value);
                output.set_pixel(x, y, binary_pixel);
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: 0.0,
            method: "Bernsen".to_string(),
            parameters: std::collections::HashMap::from([
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_multilevel_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let levels = vec![64.0, 128.0, 192.0];
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    
                    let mut binary_value = 0.0;
                    for &level in &levels {
                        if gray > level {
                            binary_value = 255.0;
                            break;
                        }
                    }
                    
                    let final_value = if binary_value ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(final_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: 0.0,
            method: "MultiLevel".to_string(),
            parameters: std::collections::HashMap::from([
                ("levels".to_string(), levels.len() as f32),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_color_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let threshold = *self.threshold_value.read();
        let invert = *self.invert.read();
        
        let color_space = crate::color_space::ColorSpace::RGB;
        let channel = "red";
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let channel_value = match channel.as_str() {
                        "red" => pixel.r,
                        "green" => pixel.g,
                        "blue" => pixel.b,
                        _ => pixel.luma(),
                    };
                    
                    let binary_value = if (channel_value > threshold) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: threshold,
            method: "Color".to_string(),
            parameters: std::collections::HashMap::from([
                ("threshold".to_string(), threshold),
                ("color_space".to_string(), format!("{:?}", color_space)),
                ("channel".to_string(), channel),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_hysteresis_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let low_threshold = *self.lower_value.read();
        let high_threshold = *self.upper_value.read();
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        let mut intermediate = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let intermediate_value = if gray > high_threshold { 255.0 } else { 0.0 };
                    
                    let intermediate_pixel = crate::image_buffer::Pixel::gray(intermediate_value);
                    intermediate.set_pixel(x, y, intermediate_pixel);
                }
            }
        }
        
        for y in 1..(input.height - 1) {
            for x in 1..(input.width - 1) {
                let mut is_white = false;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if let Some(pixel) = intermediate.get_pixel(x + dx, y + dy) {
                            if pixel.luma() > 0.0 {
                                is_white = true;
                                break;
                            }
                        }
                    }
                }
                
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > low_threshold && is_white) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: (low_threshold + high_threshold) / 2.0,
            method: "Hysteresis".to_string(),
            parameters: std::collections::HashMap::from([
                ("low_threshold".to_string(), low_threshold),
                ("high_threshold".to_string(), high_threshold),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn apply_local_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let window_size = 15;
        let contrast_limit = 0.0;
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in (window_size/2)..(input.height - window_size/2) {
            for x in (window_size/2)..(input.width - window_size/2) {
                let local_threshold = self.calculate_local_threshold(input, x, y, window_size, contrast_limit);
                
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = if (gray > local_threshold) ^ invert { 255.0 } else { 0.0 };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: 0.0,
            method: "Local".to_string(),
            parameters: std::collections::HashMap::from([
                ("window_size".to_string(), window_size as f32),
                ("contrast_limit".to_string(), contrast_limit),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    fn calculate_local_threshold(&self, image: &crate::image_buffer::ImageBuffer, center_x: u32, center_y: u32, window_size: u32, contrast_limit: f32) -> f32 {
        let half_window = window_size / 2;
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        
        for dy in -(half_window as i32)..=(half_window as i32) {
            for dx in -(half_window as i32)..=(half_window as i32) {
                let src_x = center_x as i32 + dx;
                let src_y = center_y as i32 + dy;
                
                if let Some(pixel) = image.get_pixel(src_x as u32, src_y as u32) {
                    let gray = pixel.luma();
                    min_val = min_val.min(gray);
                    max_val = max_val.max(gray);
                }
            }
        }
        
        let contrast = max_val - min_val;
        let threshold = if contrast > contrast_limit {
            min_val + contrast_limit
        } else {
            (min_val + max_val) / 2.0
        };
        
        threshold
    }

    fn apply_double_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<ThresholdResult, Box<dyn std::error::Error>> {
        let lower_threshold = *self.lower_value.read();
        let upper_threshold = *self.upper_value.read();
        let invert = *self.invert.read();
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary_value = match (gray > lower_threshold, gray > upper_threshold) {
                        (true, _) => if invert { 0.0 } else { 255.0 },
                        (_, true) => if invert { 255.0 } else { 0.0 },
                        (false, false) => if invert { 255.0 } else { 0.0 },
                        (false, true) => if invert { 0.0 } else { 255.0 },
                    };
                    
                    let binary_pixel = crate::image_buffer::Pixel::gray(binary_value);
                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
        
        Ok(ThresholdResult {
            binary_image: output,
            threshold_value: (lower_threshold + upper_threshold) / 2.0,
            method: "Double".to_string(),
            parameters: std::collections::HashMap::from([
                ("lower_threshold".to_string(), lower_threshold),
                ("upper_threshold".to_string(), upper_threshold),
                ("invert".to_string(), if invert { 1.0 } else { 0.0 }),
            ]),
        })
    }

    pub fn set_threshold_value(&self, value: f32) {
        let mut threshold = self.threshold_value.write();
        *threshold = value;
        
        let _ = self.event_sender.send(ThresholdEvent::ThresholdChanged(value));
    }

    pub fn set_lower_value(&self, value: f32) {
        let mut lower = self.lower_value.write();
        *lower = value;
        
        let _ = self.event_sender.send(ThresholdEvent::LowerChanged(value));
    }

    pub fn set_upper_value(&self, value: f32) {
        let mut upper = self.upper_value.write();
        *upper = value;
        
        let _ = self.event_sender.send(ThresholdEvent::UpperChanged(value));
    }

    pub fn set_invert(&self, invert: bool) {
        let mut invert_state = self.invert.write();
        *invert_state = invert;
        
        let _ = self.event_sender.send(ThresholdEvent::InvertChanged(invert));
    }

    pub fn get_threshold_value(&self) -> f32 {
        *self.threshold_value.read()
    }

    pub fn get_lower_value(&self) -> f32 {
        *self.lower_value.read()
    }

    pub fn get_upper_value(&self) -> f32 {
        *self.upper_value.read()
    }

    pub fn is_inverted(&self) -> bool {
        *self.invert.read()
    }

    pub async fn get_events(&mut self) -> Vec<ThresholdEvent> {
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

    pub fn clone_threshold(&self) -> Threshold {
        let mut new_threshold = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.threshold_type.clone(),
        );
        
        let threshold_value = self.threshold_value.read();
        let lower_value = self.lower_value.read();
        let upper_value = self.upper_value.read();
        let invert = self.invert.read();
        
        *new_threshold.threshold_value = *threshold_value;
        *new_threshold.lower_value = *lower_value;
        *new_threshold.upper_value = *upper_value;
        *new_threshold.invert = *invert;
        
        new_threshold
    }
}

impl Default for Threshold {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Threshold".to_string(),
            ThresholdType::Binary,
        )
    }
}

impl Default for ThresholdType {
    fn default() -> Self {
        ThresholdType::Binary
    }
}

impl Default for ThresholdResult {
    fn default() -> Self {
        let image = crate::image_buffer::ImageBuffer::default();
        Self {
            binary_image: image,
            threshold_value: 128.0,
            method: "Binary".to_string(),
            parameters: std::collections::HashMap::new(),
        }
    }
}

impl Default for AdaptiveThresholdParams {
    fn default() -> Self {
        Self {
            method: AdaptiveMethod::Mean,
            block_size: 11,
            c_constant: 2.0,
            invert: false,
        }
    }
}

impl Default for OtsuParams {
    fn default() -> Self {
        Self {
            bin_count: 256,
            invert: false,
        }
    }
}

impl Default for NiblackParams {
    fn default() -> Self {
        Self {
            window_size: 51,
            k: 0.2,
            r: 128.0,
            invert: false,
        }
    }
}

impl Default for SauvolaParams {
    fn default() -> Self {
        Self {
            window_size: 51,
            k: 0.34,
            r: 128.0,
            invert: false,
        }
    }
}

impl Default for MultiLevelParams {
    fn default() -> Self {
        Self {
            levels: vec![64.0, 128.0, 192.0],
            invert: false,
        }
    }
}

impl Default for ColorThresholdParams {
    fn default() -> Self {
        Self {
            color_space: crate::color_space::ColorSpace::RGB,
            threshold: 128.0,
            channel: "red".to_string(),
            invert: false,
        }
    }
}

impl Default for HysteresisParams {
    fn default() -> Self {
        Self {
            low_threshold: 0.0,
            high_threshold: 255.0,
            invert: false,
        }
    }
}
