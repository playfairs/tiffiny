use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageFilter {
    pub id: String,
    pub name: String,
    pub filter_type: FilterType,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub enabled: Arc<RwLock<bool>>,
    pub event_sender: mpsc::UnboundedSender<FilterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<FilterEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    GaussianBlur,
    BoxBlur,
    MotionBlur,
    MedianFilter,
    BilateralFilter,
    Sharpen,
    UnsharpMask,
    EdgeDetect,
    Emboss,
    Sobel,
    Prewitt,
    Laplacian,
    Canny,
    Noise,
    Pixelate,
    OilPaint,
    Watercolor,
    Cartoon,
    Vintage,
    Sepia,
    BlackWhite,
    Invert,
    Threshold,
    Dither,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum FilterEvent {
    ParameterChanged(String, f32),
    EnabledChanged(bool),
    FilterApplied,
    Error(String),
}

impl ImageFilter {
    pub fn new(id: String, name: String, filter_type: FilterType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            filter_type,
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn apply(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        if !*self.enabled.read() {
            return Ok(input.clone());
        }

        let _ = self.event_sender.send(FilterEvent::FilterApplied);

        match self.filter_type {
            FilterType::GaussianBlur => self.apply_gaussian_blur(input),
            FilterType::BoxBlur => self.apply_box_blur(input),
            FilterType::MotionBlur => self.apply_motion_blur(input),
            FilterType::MedianFilter => self.apply_median_filter(input),
            FilterType::BilateralFilter => self.apply_bilateral_filter(input),
            FilterType::Sharpen => self.apply_sharpen(input),
            FilterType::UnsharpMask => self.apply_unsharp_mask(input),
            FilterType::EdgeDetect => self.apply_edge_detect(input),
            FilterType::Emboss => self.apply_emboss(input),
            FilterType::Sobel => self.apply_sobel(input),
            FilterType::Prewitt => self.apply_prewitt(input),
            FilterType::Laplacian => self.apply_laplacian(input),
            FilterType::Canny => self.apply_canny(input),
            FilterType::Noise => self.apply_noise(input),
            FilterType::Pixelate => self.apply_pixelate(input),
            FilterType::OilPaint => self.apply_oil_paint(input),
            FilterType::Watercolor => self.apply_watercolor(input),
            FilterType::Cartoon => self.apply_cartoon(input),
            FilterType::Vintage => self.apply_vintage(input),
            FilterType::Sepia => self.apply_sepia(input),
            FilterType::BlackWhite => self.apply_black_white(input),
            FilterType::Invert => self.apply_invert(input),
            FilterType::Threshold => self.apply_threshold(input),
            FilterType::Dither => self.apply_dither(input),
            FilterType::Custom(_) => self.apply_custom(input),
        }
    }

    fn apply_gaussian_blur(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let sigma = parameters.get("sigma").copied().unwrap_or(1.0);
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(5.0) as u32;
        
        let kernel = self.gaussian_kernel(kernel_size, sigma);
        self.apply_convolution(input, &kernel)
    }

    fn apply_box_blur(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let radius = parameters.get("radius").copied().unwrap_or(2.0) as u32;
        
        let kernel = self.box_kernel(radius * 2 + 1);
        self.apply_convolution(input, &kernel)
    }

    fn apply_motion_blur(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let distance = parameters.get("distance").copied().unwrap_or(5.0) as u32;
        let angle = parameters.get("angle").copied().unwrap_or(0.0);
        
        let kernel = self.motion_kernel(distance, angle);
        self.apply_convolution(input, &kernel)
    }

    fn apply_median_filter(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;
        
        let mut output = input.clone();
        let half_kernel = kernel_size / 2;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut neighbors = Vec::new();
                
                for dy in -(half_kernel as i32)..=(half_kernel as i32) {
                    for dx in -(half_kernel as i32)..=(half_kernel as i32) {
                        let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            neighbors.push(pixel.luma());
                        }
                    }
                }
                
                neighbors.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let median = neighbors[neighbors.len() / 2];
                
                if let Some(mut pixel) = input.get_pixel(x, y) {
                    pixel.r = median;
                    pixel.g = median;
                    pixel.b = median;
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_bilateral_filter(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let sigma_spatial = parameters.get("sigma_spatial").copied().unwrap_or(2.0);
        let sigma_color = parameters.get("sigma_color").copied().unwrap_or(0.1);
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(5.0) as u32;
        
        let mut output = input.clone();
        let half_kernel = kernel_size / 2;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(center_pixel) = input.get_pixel(x, y) {
                    let mut sum_weight = 0.0;
                    let mut sum_color = crate::image_buffer::Pixel::new(0.0, 0.0, 0.0, 0.0);
                    
                    for dy in -(half_kernel as i32)..=(half_kernel as i32) {
                        for dx in -(half_kernel as i32)..=(half_kernel as i32) {
                            let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                            let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;
                            
                            if let Some(neighbor_pixel) = input.get_pixel(src_x, src_y) {
                                let spatial_dist = ((dx * dx + dy * dy) as f32).sqrt();
                                let spatial_weight = (-spatial_dist * spatial_dist / (2.0 * sigma_spatial * sigma_spatial)).exp();
                                
                                let color_diff = (center_pixel.luma() - neighbor_pixel.luma()).abs();
                                let color_weight = (-color_diff * color_diff / (2.0 * sigma_color * sigma_color)).exp();
                                
                                let weight = spatial_weight * color_weight;
                                sum_weight += weight;
                                sum_color.r += neighbor_pixel.r * weight;
                                sum_color.g += neighbor_pixel.g * weight;
                                sum_color.b += neighbor_pixel.b * weight;
                                sum_color.a += neighbor_pixel.a * weight;
                            }
                        }
                    }
                    
                    let filtered_pixel = crate::image_buffer::Pixel::new(
                        sum_color.r / sum_weight,
                        sum_color.g / sum_weight,
                        sum_color.b / sum_weight,
                        sum_color.a / sum_weight,
                    );
                    
                    output.set_pixel(x, y, filtered_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_sharpen(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amount = parameters.get("amount").copied().unwrap_or(1.0);
        
        let kernel = vec![
            0.0, -amount, 0.0,
            -amount, 1.0 + 4.0 * amount, -amount,
            0.0, -amount, 0.0,
        ];
        
        self.apply_convolution(input, &kernel)
    }

    fn apply_unsharp_mask(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amount = parameters.get("amount").copied().unwrap_or(1.5);
        let radius = parameters.get("radius").copied().unwrap_or(2.0) as u32;
        let threshold = parameters.get("threshold").copied().unwrap_or(0.0);
        
First apply Gaussian blur
        let blurred = self.apply_gaussian_blur(input)?;
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(original), Some(blurred_pixel)) = (input.get_pixel(x, y), blurred.get_pixel(x, y)) {
                    let diff = crate::image_buffer::Pixel::new(
                        original.r - blurred_pixel.r,
                        original.g - blurred_pixel.g,
                        original.b - blurred_pixel.b,
                        0.0,
                    );
                    
                    let magnitude = (diff.r * diff.r + diff.g * diff.g + diff.b * diff.b).sqrt();
                    
                    let sharpened_pixel = if magnitude > threshold {
                        crate::image_buffer::Pixel::new(
                            (original.r + diff.r * amount).clamp(0.0, 255.0),
                            (original.g + diff.g * amount).clamp(0.0, 255.0),
                            (original.b + diff.b * amount).clamp(0.0, 255.0),
                            original.a,
                        )
                    } else {
                        original
                    };
                    
                    output.set_pixel(x, y, sharpened_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_edge_detect(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);
        
        let kernel = vec![
            -1.0, -1.0, -1.0,
            -1.0, 8.0, -1.0,
            -1.0, -1.0, -1.0,
        ];
        
        let mut output = self.apply_convolution(input, &kernel)?;
        
        for y in 0..output.height {
            for x in 0..output.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let edge = if gray > threshold * 255.0 { 255.0 } else { 0.0 };
                    pixel.r = edge;
                    pixel.g = edge;
                    pixel.b = edge;
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_emboss(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = vec![
            -2.0, -1.0, 0.0,
            -1.0, 1.0, 1.0,
            0.0, 1.0, 2.0,
        ];
        
        let mut output = self.apply_convolution(input, &kernel)?;
        
        for y in 0..output.height {
            for x in 0..output.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    pixel.r = (pixel.r + 128.0).clamp(0.0, 255.0);
                    pixel.g = (pixel.g + 128.0).clamp(0.0, 255.0);
                    pixel.b = (pixel.b + 128.0).clamp(0.0, 255.0);
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_sobel(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let gx_kernel = vec![
            -1.0, 0.0, 1.0,
            -2.0, 0.0, 2.0,
            -1.0, 0.0, 1.0,
        ];
        
        let gy_kernel = vec![
            -1.0, -2.0, -1.0,
            0.0, 0.0, 0.0,
            1.0, 2.0, 1.0,
        ];
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let magnitude = (gx_pixel.luma() * gx_pixel.luma() + gy_pixel.luma() * gy_pixel.luma()).sqrt();
                    let edge = magnitude.clamp(0.0, 255.0);
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(edge);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_prewitt(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let gx_kernel = vec![
            -1.0, 0.0, 1.0,
            -1.0, 0.0, 1.0,
            -1.0, 0.0, 1.0,
        ];
        
        let gy_kernel = vec![
            -1.0, -1.0, -1.0,
            0.0, 0.0, 0.0,
            1.0, 1.0, 1.0,
        ];
        
        let gx = self.apply_convolution(input, &gx_kernel)?;
        let gy = self.apply_convolution(input, &gy_kernel)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(gx_pixel), Some(gy_pixel)) = (gx.get_pixel(x, y), gy.get_pixel(x, y)) {
                    let magnitude = (gx_pixel.luma() * gx_pixel.luma() + gy_pixel.luma() * gy_pixel.luma()).sqrt();
                    let edge = magnitude.clamp(0.0, 255.0);
                    
                    let edge_pixel = crate::image_buffer::Pixel::gray(edge);
                    output.set_pixel(x, y, edge_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_laplacian(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel = vec![
            0.0, 1.0, 0.0,
            1.0, -4.0, 1.0,
            0.0, 1.0, 0.0,
        ];
        
        self.apply_convolution(input, &kernel)
    }

    fn apply_canny(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let low_threshold = parameters.get("low_threshold").copied().unwrap_or(0.05);
        let high_threshold = parameters.get("high_threshold").copied().unwrap_or(0.15);
        
        let sobel = self.apply_sobel(input)?;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale8);
        
        for y in 1..input.height - 1 {
            for x in 1..input.width - 1 {
                if let Some(center_pixel) = sobel.get_pixel(x, y) {
                    let center_value = center_pixel.luma();
                    
                    let is_max = self.is_local_maximum(&sobel, x, y, center_value);
                    
                    if is_max {
                        let edge = if center_value > high_threshold * 255.0 {
                            255.0
                        } else if center_value > low_threshold * 255.0 {
                            128.0
                        } else {
                            0.0
                        };
                        
                        let edge_pixel = crate::image_buffer::Pixel::gray(edge);
                        output.set_pixel(x, y, edge_pixel);
                    } else {
                        let no_edge_pixel = crate::image_buffer::Pixel::gray(0.0);
                        output.set_pixel(x, y, no_edge_pixel);
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn apply_noise(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amount = parameters.get("amount").copied().unwrap_or(0.1);
        let noise_type = parameters.get("type").copied().unwrap_or(0.0);
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    if rand::random::<f32>() < amount {
                        match noise_type as i32 {
                            0 => {
                                pixel.r = (pixel.r + rand::random::<f32>() * 40.0 - 20.0).clamp(0.0, 255.0);
                                pixel.g = (pixel.g + rand::random::<f32>() * 40.0 - 20.0).clamp(0.0, 255.0);
                                pixel.b = (pixel.b + rand::random::<f32>() * 40.0 - 20.0).clamp(0.0, 255.0);
                            },
                            1 => {
                                pixel.r = (pixel.r + rand::random::<f32>() * 60.0 - 30.0).clamp(0.0, 255.0);
                                pixel.g = (pixel.g + rand::random::<f32>() * 60.0 - 30.0).clamp(0.0, 255.0);
                                pixel.b = (pixel.b + rand::random::<f32>() * 60.0 - 30.0).clamp(0.0, 255.0);
                            },
                            2 => {
                                if rand::random::<f32>() < 0.5 {
                                    pixel.r = 0.0;
                                    pixel.g = 0.0;
                                    pixel.b = 0.0;
                                } else {
                                    pixel.r = 255.0;
                                    pixel.g = 255.0;
                                    pixel.b = 255.0;
                                }
                            },
                            _ => {}
                        }
                        
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn apply_pixelate(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let pixel_size = parameters.get("pixel_size").copied().unwrap_or(10.0) as u32;
        
        let mut output = input.clone();
        
        for y in (0..input.height).step_by(pixel_size as usize) {
            for x in (0..input.width).step_by(pixel_size as usize) {
                if let Some(pixel) = input.get_pixel(x, y) {
                    for dy in 0..pixel_size {
                        for dx in 0..pixel_size {
                            let dst_x = x + dx;
                            let dst_y = y + dy;
                            
                            if dst_x < input.width && dst_y < input.height {
                                output.set_pixel(dst_x, dst_y, pixel);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn apply_oil_paint(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let radius = parameters.get("radius").copied().unwrap_or(5.0) as u32;
        let intensity = parameters.get("intensity").copied().unwrap_or(20.0);
        
        let mut output = input.clone();
        let half_radius = radius / 2;
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut intensity_counts = std::collections::HashMap::new();
                
                for dy in -(half_radius as i32)..=(half_radius as i32) {
                    for dx in -(half_radius as i32)..=(half_radius as i32) {
                        let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            let intensity = (pixel.luma() / intensity).round() as i32;
                            *intensity_counts.entry(intensity).or_insert(0) += 1;
                        }
                    }
                }
                
                let most_common_intensity = intensity_counts
                    .iter()
                    .max_by_key(|(_, &count)| *count)
                    .map(|(&intensity, _)| intensity)
                    .unwrap_or(128);
                
                let oil_pixel = crate::image_buffer::Pixel::gray(most_common_intensity as f32);
                output.set_pixel(x, y, oil_pixel);
            }
        }
        
        Ok(output)
    }

    fn apply_watercolor(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let blur_radius = parameters.get("blur_radius").copied().unwrap_or(3.0) as u32;
        
        let blurred = self.apply_gaussian_blur(input)?;
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(original), Some(blurred_pixel)) = (input.get_pixel(x, y), blurred.get_pixel(x, y)) {
                    let watercolor_pixel = crate::image_buffer::Pixel::new(
                        original.r * 0.7 + blurred_pixel.r * 0.3,
                        original.g * 0.7 + blurred_pixel.g * 0.3,
                        original.b * 0.7 + blurred_pixel.b * 0.3,
                        original.a,
                    );
                    
                    output.set_pixel(x, y, watercolor_pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_cartoon(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let levels = parameters.get("levels").copied().unwrap_or(6.0) as u32;
        let edge_threshold = parameters.get("edge_threshold").copied().unwrap_or(0.1);
        
        let quantized = self.quantize_colors(input, levels);
        
        let edges = self.apply_edge_detect(input)?;
        
        let mut output = quantized.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let (Some(quant_pixel), Some(edge_pixel)) = (quantized.get_pixel(x, y), edges.get_pixel(x, y)) {
                    let edge_strength = edge_pixel.luma() / 255.0;
                    
                    if edge_strength > edge_threshold {
                        let cartoon_pixel = crate::image_buffer::Pixel::new(
                            quant_pixel.r * (1.0 - edge_strength),
                            quant_pixel.g * (1.0 - edge_strength),
                            quant_pixel.b * (1.0 - edge_strength),
                            quant_pixel.a,
                        );
                        
                        output.set_pixel(x, y, cartoon_pixel);
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn apply_vintage(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let sepia_strength = parameters.get("sepia_strength").copied().unwrap_or(0.8);
        let vignette_strength = parameters.get("vignette_strength").copied().unwrap_or(0.5);
        let noise_amount = parameters.get("noise_amount").copied().unwrap_or(0.05);
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let tr = 0.393 * pixel.r + 0.769 * pixel.g + 0.189 * pixel.b;
                    let tg = 0.349 * pixel.r + 0.686 * pixel.g + 0.168 * pixel.b;
                    let tb = 0.272 * pixel.r + 0.534 * pixel.g + 0.131 * pixel.b;
                    
                    pixel.r = tr * sepia_strength + pixel.r * (1.0 - sepia_strength);
                    pixel.g = tg * sepia_strength + pixel.g * (1.0 - sepia_strength);
                    pixel.b = tb * sepia_strength + pixel.b * (1.0 - sepia_strength);
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        let center_x = input.width as f32 / 2.0;
        let center_y = input.height as f32 / 2.0;
        let max_dist = (center_x * center_x + center_y * center_y).sqrt();
        
        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                let vignette_factor = 1.0 - (dist / max_dist) * vignette_strength;
                
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    pixel.r *= vignette_factor;
                    pixel.g *= vignette_factor;
                    pixel.b *= vignette_factor;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        for y in 0..output.height {
            for x in 0..output.width {
                if rand::random::<f32>() < noise_amount {
                    if let Some(mut pixel) = output.get_pixel(x, y) {
                        pixel.r = (pixel.r + rand::random::<f32>() * 20.0 - 10.0).clamp(0.0, 255.0);
                        pixel.g = (pixel.g + rand::random::<f32>() * 20.0 - 10.0).clamp(0.0, 255.0);
                        pixel.b = (pixel.b + rand::random::<f32>() * 20.0 - 10.0).clamp(0.0, 255.0);
                        
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        Ok(output)
    }

    fn apply_sepia(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let strength = parameters.get("strength").copied().unwrap_or(1.0);
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let tr = 0.393 * pixel.r + 0.769 * pixel.g + 0.189 * pixel.b;
                    let tg = 0.349 * pixel.r + 0.686 * pixel.g + 0.168 * pixel.b;
                    let tb = 0.272 * pixel.r + 0.534 * pixel.g + 0.131 * pixel.b;
                    
                    pixel.r = tr * strength + pixel.r * (1.0 - strength);
                    pixel.g = tg * strength + pixel.g * (1.0 - strength);
                    pixel.b = tb * strength + pixel.b * (1.0 - strength);
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_black_white(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let method = parameters.get("method").copied().unwrap_or(0.0);
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let gray_value = match method as i32 {
                        0 => (pixel.r + pixel.g + pixel.b) / 3.0,
                        1 => 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b,
                        2 => (pixel.r.max(pixel.g.max(pixel.b)) + pixel.r.min(pixel.g.min(pixel.b))) / 2.0,
                        _ => (pixel.r + pixel.g + pixel.b) / 3.0,
                    };
                    
                    pixel.r = gray_value;
                    pixel.g = gray_value;
                    pixel.b = gray_value;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_invert(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    pixel.r = 255.0 - pixel.r;
                    pixel.g = 255.0 - pixel.g;
                    pixel.b = 255.0 - pixel.b;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(128.0);
        
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let binary = if gray > threshold { 255.0 } else { 0.0 };
                    
                    pixel.r = binary;
                    pixel.g = binary;
                    pixel.b = binary;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_dither(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let dither_type = parameters.get("type").copied().unwrap_or(0.0);
        
        match dither_type as i32 {
            0 => self.apply_floyd_steinberg_dither(input),
            1 => self.apply_ordered_dither(input),
            2 => self.apply_random_dither(input),
            _ => self.apply_floyd_steinberg_dither(input),
        }
    }

    fn apply_custom(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        Ok(input.clone())
    }

    fn gaussian_kernel(&self, size: u32, sigma: f32) -> Vec<f32> {
        let mut kernel = Vec::with_capacity((size * size) as usize);
        let center = size as f32 / 2.0;
        let sum = 0.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let value = (- (dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                kernel.push(value);
            }
        }
        
        let kernel_sum: f32 = kernel.iter().sum();
        kernel.iter_mut().for_each(|v| *v /= kernel_sum);
        
        kernel
    }

    fn box_kernel(&self, size: u32) -> Vec<f32> {
        let kernel_size = size * size;
        let value = 1.0 / kernel_size as f32;
        vec![value; kernel_size as usize]
    }

    fn motion_kernel(&self, distance: u32, angle: f32) -> Vec<f32> {
        let mut kernel = vec![0.0; (distance * distance) as usize];
        let center = distance as f32 / 2.0;
        let angle_rad = angle.to_radians();
        
        for i in 0..distance {
            let x = (center + (i as f32 - center) * angle_rad.cos()).round() as u32;
            let y = (center + (i as f32 - center) * angle_rad.sin()).round() as u32;
            
            if x < distance && y < distance {
                kernel[(y * distance + x) as usize] = 1.0 / distance as f32;
            }
        }
        
        kernel
    }

    fn apply_convolution(&self, input: &crate::image_buffer::ImageBuffer, kernel: &[f32]) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let kernel_size = (kernel.len() as f32).sqrt() as u32;
        let half_kernel = kernel_size / 2;
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        for y in 0..input.height {
            for x in 0..input.width {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let src_x = (x as i32 + kx as i32 - half_kernel as i32).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + ky as i32 - half_kernel as i32).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            let weight = kernel[(ky * kernel_size + kx) as usize];
                            sum_r += pixel.r * weight;
                            sum_g += pixel.g * weight;
                            sum_b += pixel.b * weight;
                            sum_a += pixel.a * weight;
                        }
                    }
                }
                
                let convolved_pixel = crate::image_buffer::Pixel::new(
                    sum_r.clamp(0.0, 255.0),
                    sum_g.clamp(0.0, 255.0),
                    sum_b.clamp(0.0, 255.0),
                    sum_a.clamp(0.0, 255.0),
                );
                
                output.set_pixel(x, y, convolved_pixel);
            }
        }
        
        Ok(output)
    }

    fn is_local_maximum(&self, image: &crate::image_buffer::ImageBuffer, x: u32, y: u32, value: f32) -> bool {
        let neighbors = [
            (-1, -1), (0, -1), (1, -1),
            (-1, 0),           (1, 0),
            (-1, 1), (0, 1), (1, 1),
        ];
        
        for (dx, dy) in &neighbors {
            let nx = (x as i32 + dx).clamp(0, image.width as i32 - 1) as u32;
            let ny = (y as i32 + dy).clamp(0, image.height as i32 - 1) as u32;
            
            if let Some(neighbor_pixel) = image.get_pixel(nx, ny) {
                if neighbor_pixel.luma() >= value {
                    return false;
                }
            }
        }
        
        true
    }

    fn quantize_colors(&self, input: &crate::image_buffer::ImageBuffer, levels: u32) -> crate::image_buffer::ImageBuffer {
        let mut output = input.clone();
        let step = 255.0 / levels as f32;
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let quantized_r = (pixel.r / step).round() * step;
                    let quantized_g = (pixel.g / step).round() * step;
                    let quantized_b = (pixel.b / step).round() * step;
                    
                    pixel.r = quantized_r.clamp(0.0, 255.0);
                    pixel.g = quantized_g.clamp(0.0, 255.0);
                    pixel.b = quantized_b.clamp(0.0, 255.0);
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        output
    }

    fn apply_floyd_steinberg_dither(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for y in 0..input.height {
            let mut error_r = 0.0;
            let mut error_g = 0.0;
            let mut error_b = 0.0;
            
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let old_r = pixel.r + error_r;
                    let old_g = pixel.g + error_g;
                    let old_b = pixel.b + error_b;
                    
                    let new_r = (old_r / 255.0).round() * 255.0;
                    let new_g = (old_g / 255.0).round() * 255.0;
                    let new_b = (old_b / 255.0).round() * 255.0;
                    
                    pixel.r = new_r.clamp(0.0, 255.0);
                    pixel.g = new_g.clamp(0.0, 255.0);
                    pixel.b = new_b.clamp(0.0, 255.0);
                    
                    output.set_pixel(x, y, pixel);
                    
                    let quant_error_r = old_r - new_r;
                    let quant_error_g = old_g - new_g;
                    let quant_error_b = old_b - new_b;
                    
                    if x + 1 < input.width {
                        if let Some(mut next_pixel) = output.get_pixel(x + 1, y) {
                            next_pixel.r = (next_pixel.r + quant_error_r * 7.0 / 16.0).clamp(0.0, 255.0);
                            next_pixel.g = (next_pixel.g + quant_error_g * 7.0 / 16.0).clamp(0.0, 255.0);
                            next_pixel.b = (next_pixel.b + quant_error_b * 7.0 / 16.0).clamp(0.0, 255.0);
                            output.set_pixel(x + 1, y, next_pixel);
                        }
                    }
                    
                    if y + 1 < input.height {
                        if x > 0 {
                            if let Some(mut below_pixel) = output.get_pixel(x - 1, y + 1) {
                                below_pixel.r = (below_pixel.r + quant_error_r * 3.0 / 16.0).clamp(0.0, 255.0);
                                below_pixel.g = (below_pixel.g + quant_error_g * 3.0 / 16.0).clamp(0.0, 255.0);
                                below_pixel.b = (below_pixel.b + quant_error_b * 3.0 / 16.0).clamp(0.0, 255.0);
                                output.set_pixel(x - 1, y + 1, below_pixel);
                            }
                        }
                        
                        if let Some(mut below_pixel) = output.get_pixel(x, y + 1) {
                            below_pixel.r = (below_pixel.r + quant_error_r * 5.0 / 16.0).clamp(0.0, 255.0);
                            below_pixel.g = (below_pixel.g + quant_error_g * 5.0 / 16.0).clamp(0.0, 255.0);
                            below_pixel.b = (below_pixel.b + quant_error_b * 5.0 / 16.0).clamp(0.0, 255.0);
                            output.set_pixel(x, y + 1, below_pixel);
                        }
                        
                        if x + 1 < input.width {
                            if let Some(mut below_pixel) = output.get_pixel(x + 1, y + 1) {
                                below_pixel.r = (below_pixel.r + quant_error_r * 1.0 / 16.0).clamp(0.0, 255.0);
                                below_pixel.g = (below_pixel.g + quant_error_g * 1.0 / 16.0).clamp(0.0, 255.0);
                                below_pixel.b = (below_pixel.b + quant_error_b * 1.0 / 16.0).clamp(0.0, 255.0);
                                output.set_pixel(x + 1, y + 1, below_pixel);
                            }
                        }
                    }
                    
                    error_r = quant_error_r / 16.0;
                    error_g = quant_error_g / 16.0;
                    error_b = quant_error_b / 16.0;
                }
            }
        }
        
        Ok(output)
    }

    fn apply_ordered_dither(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        let bayer_matrix = [
            [0, 8, 2, 10],
            [12, 4, 14, 6],
            [3, 11, 1, 9],
            [15, 7, 13, 5],
        ];
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let matrix_x = x % 4;
                    let matrix_y = y % 4;
                    let threshold = bayer_matrix[matrix_y as usize][matrix_x as usize] as f32 / 16.0 * 255.0;
                    
                    let gray = pixel.luma();
                    let dithered = if gray > threshold { 255.0 } else { 0.0 };
                    
                    pixel.r = dithered;
                    pixel.g = dithered;
                    pixel.b = dithered;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    fn apply_random_dither(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = input.clone();
        
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(mut pixel) = output.get_pixel(x, y) {
                    let gray = pixel.luma();
                    let threshold = rand::random::<f32>() * 255.0;
                    let dithered = if gray > threshold { 255.0 } else { 0.0 };
                    
                    pixel.r = dithered;
                    pixel.g = dithered;
                    pixel.b = dithered;
                    
                    output.set_pixel(x, y, pixel);
                }
            }
        }
        
        Ok(output)
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(FilterEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled_state = self.enabled.write();
        *enabled_state = enabled;
        
        let _ = self.event_sender.send(FilterEvent::EnabledChanged(enabled));
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    pub async fn get_events(&mut self) -> Vec<FilterEvent> {
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

    pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
        self.parameters.read().clone()
    }

    pub fn clone_filter(&self) -> ImageFilter {
        let mut new_filter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.filter_type.clone(),
        );
        
        let parameters = self.parameters.read();
        *new_filter.parameters.write() = parameters.clone();
        
        new_filter
    }
}

impl Default for ImageFilter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Filter".to_string(),
            FilterType::GaussianBlur,
        )
    }
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::GaussianBlur
    }
}
