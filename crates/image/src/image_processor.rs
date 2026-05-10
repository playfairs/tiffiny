use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageProcessor {
    pub id: String,
    pub name: String,
    pub processor_type: ProcessorType,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub enabled: Arc<RwLock<bool>>,
    pub event_sender: mpsc::UnboundedSender<ProcessorEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProcessorEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorType {
    Resize,
    Crop,
    Rotate,
    Flip,
    Grayscale,
    Brightness,
    Contrast,
    Saturation,
    Hue,
    Blur,
    Sharpen,
    EdgeDetection,
    Threshold,
    Morphology,
    ColorSpace,
    Histogram,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    ParameterChanged(String, f32),
    EnabledChanged(bool),
    ProcessingStarted,
    ProcessingCompleted,
    Error(String),
}

impl ImageProcessor {
    pub fn new(id: String, name: String, processor_type: ProcessorType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            processor_type,
            parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn process(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        if !*self.enabled.read() {
            return Ok(input.clone());
        }

        let _ = self.event_sender.send(ProcessorEvent::ProcessingStarted);

        let result = match self.processor_type {
            ProcessorType::Resize => self.process_resize(input),
            ProcessorType::Crop => self.process_crop(input),
            ProcessorType::Rotate => self.process_rotate(input),
            ProcessorType::Flip => self.process_flip(input),
            ProcessorType::Grayscale => self.process_grayscale(input),
            ProcessorType::Brightness => self.process_brightness(input),
            ProcessorType::Contrast => self.process_contrast(input),
            ProcessorType::Saturation => self.process_saturation(input),
            ProcessorType::Hue => self.process_hue(input),
            ProcessorType::Blur => self.process_blur(input),
            ProcessorType::Sharpen => self.process_sharpen(input),
            ProcessorType::EdgeDetection => self.process_edge_detection(input),
            ProcessorType::Threshold => self.process_threshold(input),
            ProcessorType::Morphology => self.process_morphology(input),
            ProcessorType::ColorSpace => self.process_color_space(input),
            ProcessorType::Histogram => self.process_histogram(input),
            ProcessorType::Custom(_) => self.process_custom(input),
        };

        match result {
            Ok(output) => {
                let _ = self.event_sender.send(ProcessorEvent::ProcessingCompleted);
                Ok(output)
            },
            Err(e) => {
                let error_msg = format!("Processing failed: {}", e);
                let _ = self.event_sender.send(ProcessorEvent::Error(error_msg));
                Err(e)
            },
        }
    }

    fn process_resize(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;
        let algorithm = parameters.get("algorithm").copied().unwrap_or(0.0);0=nearest, 1=bilinear, 2=bicubic

        let mut output = crate::image_buffer::ImageBuffer::new(width, height, input.pixel_format);

        match algorithm as i32 {
            0 => {
                for y in 0..height {
                    for x in 0..width {
                        let src_x = (x as f32 * input.width as f32 / width as f32) as u32;
                        let src_y = (y as f32 * input.height as f32 / height as f32) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x.min(input.width - 1), src_y.min(input.height - 1)) {
                            output.set_pixel(x, y, pixel);
                        }
                    }
                }
            },
            1 => {
                let src_image = input.to_image_buffer();
                let resized_image = fast_image_resize::resize(
                    &src_image,
                    width,
                    height,
                    fast_image_resize::FilterType::Lanczos3,
                );
                
                for y in 0..height.min(resized_image.height()) {
                    for x in 0..width.min(resized_image.width()) {
                        if let Some(pixel) = resized_image.get_pixel(x, y) {
                            let converted_pixel = crate::image_buffer::Pixel {
                                r: pixel[0] as f32,
                                g: pixel[1] as f32,
                                b: pixel[2] as f32,
                                a: pixel.get(3).copied().unwrap_or(255) as f32,
                            };
                            output.set_pixel(x, y, converted_pixel);
                        }
                    }
                }
            },
            2 => {
                let src_image = input.to_image_buffer();
                let resized_image = fast_image_resize::resize(
                    &src_image,
                    width,
                    height,
                    fast_image_resize::FilterType::Lanczos8,
                );
                
                for y in 0..height.min(resized_image.height()) {
                    for x in 0..width.min(resized_image.width()) {
                        if let Some(pixel) = resized_image.get_pixel(x, y) {
                            let converted_pixel = crate::image_buffer::Pixel {
                                r: pixel[0] as f32,
                                g: pixel[1] as f32,
                                b: pixel[2] as f32,
                                a: pixel.get(3).copied().unwrap_or(255) as f32,
                            };
                            output.set_pixel(x, y, converted_pixel);
                        }
                    }
                }
            },
            _ => {
                return Err("Invalid resize algorithm".into());
            },
        }

        Ok(output)
    }

    fn process_crop(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let x = parameters.get("x").copied().unwrap_or(0.0) as u32;
        let y = parameters.get("y").copied().unwrap_or(0.0) as u32;
        let width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;

        let mut output = crate::image_buffer::ImageBuffer::new(width, height, input.pixel_format);

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                
                if src_x < input.width && src_y < input.height {
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(dx, dy, pixel);
                    }
                }
            }
        }

        Ok(output)
    }

    fn process_rotate(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let angle = parameters.get("angle").copied().unwrap_or(0.0);
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let background_color = parameters.get("background_r").copied().unwrap_or(0.0);

        let angle_rad = angle.to_radians();
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                let src_x = (dx * cos_angle - dy * sin_angle + center_x).round() as i32;
                let src_y = (dx * sin_angle + dy * cos_angle + center_y).round() as i32;
                
                if src_x >= 0 && src_x < input.width as i32 && 
                   src_y >= 0 && src_y < input.height as i32 {
                    if let Some(pixel) = input.get_pixel(src_x as u32, src_y as u32) {
                        output.set_pixel(x, y, pixel);
                    }
                } else {
                    let background_pixel = crate::image_buffer::Pixel {
                        r: background_color,
                        g: background_color,
                        b: background_color,
                        a: 255.0,
                    };
                    output.set_pixel(x, y, background_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_flip(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let flip_horizontal = parameters.get("horizontal").copied().unwrap_or(0.0) > 0.5;
        let flip_vertical = parameters.get("vertical").copied().unwrap_or(0.0) > 0.5;

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                let src_x = if flip_horizontal { input.width - 1 - x } else { x };
                let src_y = if flip_vertical { input.height - 1 - y } else { y };
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_grayscale(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let method = parameters.get("method").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray_value = match method as i32 {
                        0 => (pixel.r + pixel.g + pixel.b) / 3.0,
                        1 => 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b,
                        2 => (pixel.r.max(pixel.g.max(pixel.b)) + pixel.r.min(pixel.g.min(pixel.b))) / 2.0,
                        _ => (pixel.r + pixel.g + pixel.b) / 3.0,
                    };

                    let gray_pixel = crate::image_buffer::Pixel {
                        r: gray_value,
                        g: gray_value,
                        b: gray_value,
                        a: pixel.a,
                    };
                    
                    output.set_pixel(x, y, gray_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_brightness(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let brightness = parameters.get("brightness").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let adjusted_pixel = crate::image_buffer::Pixel {
                        r: (pixel.r + brightness).clamp(0.0, 255.0),
                        g: (pixel.g + brightness).clamp(0.0, 255.0),
                        b: (pixel.b + brightness).clamp(0.0, 255.0),
                        a: pixel.a,
                    };
                    
                    output.set_pixel(x, y, adjusted_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_contrast(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let contrast = parameters.get("contrast").copied().unwrap_or(1.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let adjusted_pixel = crate::image_buffer::Pixel {
                        r: ((pixel.r - 128.0) * contrast + 128.0).clamp(0.0, 255.0),
                        g: ((pixel.g - 128.0) * contrast + 128.0).clamp(0.0, 255.0),
                        b: ((pixel.b - 128.0) * contrast + 128.0).clamp(0.0, 255.0),
                        a: pixel.a,
                    };
                    
                    output.set_pixel(x, y, adjusted_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_saturation(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let saturation = parameters.get("saturation").copied().unwrap_or(1.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = 0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b;
                    
                    let adjusted_pixel = crate::image_buffer::Pixel {
                        r: (gray + saturation * (pixel.r - gray)).clamp(0.0, 255.0),
                        g: (gray + saturation * (pixel.g - gray)).clamp(0.0, 255.0),
                        b: (gray + saturation * (pixel.b - gray)).clamp(0.0, 255.0),
                        a: pixel.a,
                    };
                    
                    output.set_pixel(x, y, adjusted_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_hue(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let hue_shift = parameters.get("hue").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let (h, s, v) = self.rgb_to_hsv(pixel.r, pixel.g, pixel.b);
                    
                    let new_h = (h + hue_shift) % 360.0;
                    
                    let (r, g, b) = self.hsv_to_rgb(new_h, s, v);
                    
                    let adjusted_pixel = crate::image_buffer::Pixel {
                        r: r.clamp(0.0, 255.0),
                        g: g.clamp(0.0, 255.0),
                        b: b.clamp(0.0, 255.0),
                        a: pixel.a,
                    };
                    
                    output.set_pixel(x, y, adjusted_pixel);
                }
            }
        }

        Ok(output)
    }

    fn process_blur(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let radius = parameters.get("radius").copied().unwrap_or(1.0) as u32;
        let sigma = parameters.get("sigma").copied().unwrap_or(1.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 0..input.height {
            for x in 0..input.width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut count = 0;

                for dy in -(radius as i32)..=(radius as i32) {
                    for dx in -(radius as i32)..=(radius as i32) {
                        let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            r_sum += pixel.r;
                            g_sum += pixel.g;
                            b_sum += pixel.b;
                            count += 1;
                        }
                    }
                }

                let blurred_pixel = crate::image_buffer::Pixel {
                    r: r_sum / count as f32,
                    g: g_sum / count as f32,
                    b: b_sum / count as f32,
                    a: input.get_pixel(x, y).map_or(255.0, |p| p.a),
                };
                
                output.set_pixel(x, y, blurred_pixel);
            }
        }

        Ok(output)
    }

    fn process_sharpen(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amount = parameters.get("amount").copied().unwrap_or(1.0);
        let threshold = parameters.get("threshold").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        for y in 1..input.height - 1 {
            for x in 1..input.width - 1 {
                let center = input.get_pixel(x, y).unwrap();
                let top = input.get_pixel(x, y - 1).unwrap();
                let bottom = input.get_pixel(x, y + 1).unwrap();
                let left = input.get_pixel(x - 1, y).unwrap();
                let right = input.get_pixel(x + 1, y).unwrap();

                let blurred = crate::image_buffer::Pixel {
                    r: (top.r + bottom.r + left.r + right.r) / 4.0,
                    g: (top.g + bottom.g + left.g + right.g) / 4.0,
                    b: (top.b + bottom.b + left.b + right.b) / 4.0,
                    a: center.a,
                };

                let diff = crate::image_buffer::Pixel {
                    r: center.r - blurred.r,
                    g: center.g - blurred.g,
                    b: center.b - blurred.b,
                    a: 0.0,
                };

                let magnitude = (diff.r * diff.r + diff.g * diff.g + diff.b * diff.b).sqrt();
                
                let sharpened_pixel = if magnitude > threshold {
                    crate::image_buffer::Pixel {
                        r: (center.r + diff.r * amount).clamp(0.0, 255.0),
                        g: (center.g + diff.g * amount).clamp(0.0, 255.0),
                        b: (center.b + diff.b * amount).clamp(0.0, 255.0),
                        a: center.a,
                    }
                } else {
                    center
                };

                output.set_pixel(x, y, sharpened_pixel);
            }
        }

        Ok(output)
    }

    fn process_edge_detection(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let method = parameters.get("method").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale);

        match method as i32 {
            0 => self.sobel_edge_detection(input, &mut output),
            1 => self.prewitt_edge_detection(input, &mut output),
            2 => self.roberts_edge_detection(input, &mut output),
            _ => return Err("Invalid edge detection method".into()),
        }

        Ok(output)
    }

    fn process_threshold(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(128.0);
        let method = parameters.get("method").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, crate::image_buffer::PixelFormat::Grayscale);

        match method as i32 {
            0 => self.binary_threshold(input, &mut output, threshold),
            1 => self.adaptive_threshold(input, &mut output),
            _ => return Err("Invalid threshold method".into()),
        }

        Ok(output)
    }

    fn process_morphology(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let operation = parameters.get("operation").copied().unwrap_or(0.0);
        let kernel_size = parameters.get("kernel_size").copied().unwrap_or(3.0) as u32;

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        match operation as i32 {
            0 => self.erosion(input, &mut output, kernel_size),
            1 => self.dilation(input, &mut output, kernel_size),
            2 => self.opening(input, &mut output, kernel_size),
            3 => self.closing(input, &mut output, kernel_size),
            _ => return Err("Invalid morphology operation".into()),
        }

        Ok(output)
    }

    fn process_color_space(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let target_space = parameters.get("space").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);

        match target_space as i32 {
            0 => self.rgb_to_hsv(input, &mut output),
            1 => self.rgb_to_lab(input, &mut output),
            2 => self.rgb_to_xyz(input, &mut output),
            _ => return Err("Invalid color space".into()),
        }

        Ok(output)
    }

    fn process_histogram(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let operation = parameters.get("operation").copied().unwrap_or(0.0);

        let mut output = input.clone();

        match operation as i32 {
            0 => self.histogram_equalization(&mut output),
            1 => self.histogram_stretch(&mut output),
            _ => return Err("Invalid histogram operation".into()),
        }

        Ok(output)
    }

    fn process_custom(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        Ok(input.clone())
    }

    fn rgb_to_hsv(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta)
        } else if max == g {
            60.0 * ((b - r) / delta) + 120.0
        } else {
            60.0 * ((r - g) / delta) + 240.0
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h, s, v)
    }

    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0 % 2.0 - 1.0).abs() - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match (h / 60.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        (r + m, g + m, b + m)
    }

    fn sobel_edge_detection(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer) {
        for y in 1..input.height - 1 {
            for x in 1..input.width - 1 {
                let pixel = input.get_pixel(x, y).unwrap();
                let gray = (pixel.r + pixel.g + pixel.b) / 3.0;

                let gx = self.get_pixel_value(input, x - 1, y - 1) * -1.0 +
                         self.get_pixel_value(input, x, y - 1) * -2.0 +
                         self.get_pixel_value(input, x + 1, y - 1) * -1.0 +
                         self.get_pixel_value(input, x - 1, y + 1) * 1.0 +
                         self.get_pixel_value(input, x + 1, y + 1) * 1.0;

                let gy = self.get_pixel_value(input, x - 1, y - 1) * -1.0 +
                         self.get_pixel_value(input, x, y - 1) * -2.0 +
                         self.get_pixel_value(input, x + 1, y - 1) * -1.0 +
                         self.get_pixel_value(input, x - 1, y + 1) * 1.0 +
                         self.get_pixel_value(input, x + 1, y + 1) * 1.0;

                let magnitude = (gx * gx + gy * gy).sqrt();
                let edge_value = magnitude.clamp(0.0, 255.0);

                let edge_pixel = crate::image_buffer::Pixel {
                    r: edge_value,
                    g: edge_value,
                    b: edge_value,
                    a: 255.0,
                };

                output.set_pixel(x, y, edge_pixel);
            }
        }
    }

    fn get_pixel_value(&self, input: &crate::image_buffer::ImageBuffer, x: u32, y: u32) -> f32 {
        if let Some(pixel) = input.get_pixel(x, y) {
            (pixel.r + pixel.g + pixel.b) / 3.0
        } else {
            0.0
        }
    }

    fn binary_threshold(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, threshold: f32) {
        for y in 0..input.height {
            for x in 0..input.width {
                if let Some(pixel) = input.get_pixel(x, y) {
                    let gray = (pixel.r + pixel.g + pixel.b) / 3.0;
                    let binary_value = if gray > threshold { 255.0 } else { 0.0 };

                    let binary_pixel = crate::image_buffer::Pixel {
                        r: binary_value,
                        g: binary_value,
                        b: binary_value,
                        a: pixel.a,
                    };

                    output.set_pixel(x, y, binary_pixel);
                }
            }
        }
    }

    fn erosion(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, kernel_size: u32) {
        let half_kernel = kernel_size / 2;

        for y in 0..input.height {
            for x in 0..input.width {
                let mut min_value = 255.0;

                for dy in -(half_kernel as i32)..=(half_kernel as i32) {
                    for dx in -(half_kernel as i32)..=(half_kernel as i32) {
                        let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;

                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            let gray = (pixel.r + pixel.g + pixel.b) / 3.0;
                            min_value = min_value.min(gray);
                        }
                    }
                }

                let eroded_pixel = crate::image_buffer::Pixel {
                    r: min_value,
                    g: min_value,
                    b: min_value,
                    a: input.get_pixel(x, y).map_or(255.0, |p| p.a),
                };

                output.set_pixel(x, y, eroded_pixel);
            }
        }
    }

    fn dilation(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, kernel_size: u32) {
        let half_kernel = kernel_size / 2;

        for y in 0..input.height {
            for x in 0..input.width {
                let mut max_value = 0.0;

                for dy in -(half_kernel as i32)..=(half_kernel as i32) {
                    for dx in -(half_kernel as i32)..=(half_kernel as i32) {
                        let src_x = (x as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let src_y = (y as i32 + dy).clamp(0, input.height as i32 - 1) as u32;

                        if let Some(pixel) = input.get_pixel(src_x, src_y) {
                            let gray = (pixel.r + pixel.g + pixel.b) / 3.0;
                            max_value = max_value.max(gray);
                        }
                    }
                }

                let dilated_pixel = crate::image_buffer::Pixel {
                    r: max_value,
                    g: max_value,
                    b: max_value,
                    a: input.get_pixel(x, y).map_or(255.0, |p| p.a),
                };

                output.set_pixel(x, y, dilated_pixel);
            }
        }
    }

    fn opening(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, kernel_size: u32) {
        let mut temp = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);
        self.erosion(input, &mut temp, kernel_size);
        self.dilation(&temp, output, kernel_size);
    }

    fn closing(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, kernel_size: u32) {
        let mut temp = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format);
        self.dilation(input, &mut temp, kernel_size);
        self.erosion(&temp, output, kernel_size);
    }

    fn histogram_equalization(&self, image: &mut crate::image_buffer::ImageBuffer) {
        let mut histogram = [0u32; 256];
        let total_pixels = (image.width * image.height) as f32;

        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(pixel) = image.get_pixel(x, y) {
                    let gray = (pixel.r + pixel.g + pixel.b) / 3.0 as u8;
                    histogram[gray as usize] += 1;
                }
            }
        }

        let mut cdf = [0u32; 256];
        let mut sum = 0u32;
        for i in 0..256 {
            sum += histogram[i];
            cdf[i] = sum;
        }

        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(mut pixel) = image.get_pixel(x, y) {
                    let gray = (pixel.r + pixel.g + pixel.b) / 3.0 as u8;
                    let equalized_value = (cdf[gray as usize] as f32 * 255.0 / total_pixels) as u8;

                    pixel.r = equalized_value as f32;
                    pixel.g = equalized_value as f32;
                    pixel.b = equalized_value as f32;

                    image.set_pixel(x, y, pixel);
                }
            }
        }
    }

    fn histogram_stretch(&self, image: &mut crate::image_buffer::ImageBuffer) {
        let mut min_value = 255.0;
        let mut max_value = 0.0;

        for y in 0..image.height {
            for x in 0..image.width {
                if let Some(pixel) = image.get_pixel(x, y) {
                    let gray = (pixel.r + pixel.g + pixel.b) / 3.0;
                    min_value = min_value.min(gray);
                    max_value = max_value.max(gray);
                }
            }
        }

        let range = max_value - min_value;
        if range > 0.0 {
            for y in 0..image.height {
                for x in 0..image.width {
                    if let Some(mut pixel) = image.get_pixel(x, y) {
                        let gray = (pixel.r + pixel.g + pixel.b) / 3.0;
                        let stretched = ((gray - min_value) * 255.0 / range).clamp(0.0, 255.0) as u8;

                        pixel.r = stretched as f32;
                        pixel.g = stretched as f32;
                        pixel.b = stretched as f32;

                        image.set_pixel(x, y, pixel);
                    }
                }
            }
        }
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(ProcessorEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled_state = self.enabled.write();
        *enabled_state = enabled;
        
        let _ = self.event_sender.send(ProcessorEvent::EnabledChanged(enabled));
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    pub async fn get_events(&mut self) -> Vec<ProcessorEvent> {
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

    pub fn clone_processor(&self) -> ImageProcessor {
        let mut new_processor = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.processor_type.clone(),
        );
        
        let parameters = self.parameters.read();
        *new_processor.parameters.write() = parameters.clone();
        
        new_processor
    }
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Processor".to_string(),
            ProcessorType::Grayscale,
        )
    }
}

impl Default for ProcessorType {
    fn default() -> Self {
        ProcessorType::Grayscale
    }
}
