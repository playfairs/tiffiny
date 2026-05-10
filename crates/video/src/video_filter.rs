use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoFilter {
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
    Blur,
    Sharpen,
    Denoise,
    Brightness,
    Contrast,
    Saturation,
    Hue,
    Gamma,
    ColorBalance,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum FilterEvent {
    ParameterChanged(String, f32),
    EnabledChanged(bool),
    FilterApplied,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FilterResult {
    pub success: bool,
    pub output_frame: Option<crate::video_buffer::VideoFrame>,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

impl VideoFilter {
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

    pub async fn apply(&self, input: &crate::video_buffer::VideoFrame) -> Result<FilterResult, Box<dyn std::error::Error>> {
        if !*self.enabled.read() {
            return Ok(FilterResult {
                success: true,
                output_frame: Some(input.clone()),
                processing_time: std::time::Duration::from_millis(0),
                error_message: None,
            });
        }

        let _ = self.event_sender.send(FilterEvent::FilterApplied);
        let start_time = std::time::Instant::now();

        let result = match self.filter_type {
            FilterType::Blur => self.apply_blur(input),
            FilterType::Sharpen => self.apply_sharpen(input),
            FilterType::Denoise => self.apply_denoise(input),
            FilterType::Brightness => self.apply_brightness(input),
            FilterType::Contrast => self.apply_contrast(input),
            FilterType::Saturation => self.apply_saturation(input),
            FilterType::Hue => self.apply_hue(input),
            FilterType::Gamma => self.apply_gamma(input),
            FilterType::ColorBalance => self.apply_color_balance(input),
            FilterType::Custom(_) => self.apply_custom(input),
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(output_frame) => Ok(FilterResult {
                success: true,
                output_frame: Some(output_frame),
                processing_time,
                error_message: None,
            }),
            Err(e) => Ok(FilterResult {
                success: false,
                output_frame: None,
                processing_time,
                error_message: Some(format!("Filter failed: {}", e)),
            }),
        }
    }

    fn apply_blur(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let radius = parameters.get("radius").copied().unwrap_or(1.0) as u32;
        let sigma = parameters.get("sigma").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();
        let kernel_size = radius * 2 + 1;
        let kernel = self.create_gaussian_kernel(kernel_size, sigma);

        for y in radius..(frame.height - radius) {
            for x in radius..(frame.width - radius) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                let mut total_weight = 0.0;

                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let src_x = x + kx - radius;
                        let src_y = y + ky - radius;

                        if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                            let weight = kernel[ky * kernel_size + kx];
                            sum_r += pixel.r * weight;
                            sum_g += pixel.g * weight;
                            sum_b += pixel.b * weight;
                            sum_a += pixel.a * weight;
                            total_weight += weight;
                        }
                    }
                }

                if total_weight > 0.0 {
                    let blurred_pixel = crate::video_buffer::Pixel {
                        r: sum_r / total_weight,
                        g: sum_g / total_weight,
                        b: sum_b / total_weight,
                        a: sum_a / total_weight,
                    };
                    output_frame.set_pixel(x, y, blurred_pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_sharpen(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amount = parameters.get("amount").copied().unwrap_or(1.0);
        let sigma = parameters.get("sigma").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();
        let kernel_size = 3;
        let kernel = vec![
            0.0, -amount, 0.0,
            -amount, 1.0 + 4.0 * amount, -amount,
            0.0, -amount, 0.0,
        ];

        for y in 1..(frame.height - 1) {
            for x in 1..(frame.width - 1) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;

                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let src_x = x + kx - 1;
                        let src_y = y + ky - 1;

                        if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                            let weight = kernel[ky * kernel_size + kx];
                            sum_r += pixel.r * weight;
                            sum_g += pixel.g * weight;
                            sum_b += pixel.b * weight;
                            sum_a += pixel.a * weight;
                        }
                    }
                }

                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    pixel.r = (pixel.r + sum_r).clamp(0.0, 255.0);
                    pixel.g = (pixel.g + sum_g).clamp(0.0, 255.0);
                    pixel.b = (pixel.b + sum_b).clamp(0.0, 255.0);
                    pixel.a = (pixel.a + sum_a).clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_denoise(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let strength = parameters.get("strength").copied().unwrap_or(0.5);
        let window_size = parameters.get("window_size").copied().unwrap_or(3.0) as u32;

        let mut output_frame = frame.clone();
        let half_window = window_size / 2;

        for y in half_window..(frame.height - half_window) {
            for x in half_window..(frame.width - half_window) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                let mut count = 0.0;

                for dy in -(half_window as i32)..=(half_window as i32) {
                    for dx in -(half_window as i32)..=(half_window as i32) {
                        let src_x = x as i32 + dx;
                        let src_y = y as i32 + dy;

                        if let Some(pixel) = frame.get_pixel(src_x as u32, src_y as u32) {
                            sum_r += pixel.r;
                            sum_g += pixel.g;
                            sum_b += pixel.b;
                            sum_a += pixel.a;
                            count += 1.0;
                        }
                    }
                }

                if count > 0.0 {
                    let avg_pixel = crate::video_buffer::Pixel {
                        r: sum_r / count,
                        g: sum_g / count,
                        b: sum_b / count,
                        a: sum_a / count,
                    };

                    if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                        pixel.r = pixel.r * (1.0 - strength) + avg_pixel.r * strength;
                        pixel.g = pixel.g * (1.0 - strength) + avg_pixel.g * strength;
                        pixel.b = pixel.b * (1.0 - strength) + avg_pixel.b * strength;
                        pixel.a = pixel.a * (1.0 - strength) + avg_pixel.a * strength;
                        output_frame.set_pixel(x, y, pixel);
                    }
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_brightness(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let brightness = parameters.get("brightness").copied().unwrap_or(0.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    pixel.r = (pixel.r + brightness).clamp(0.0, 255.0);
                    pixel.g = (pixel.g + brightness).clamp(0.0, 255.0);
                    pixel.b = (pixel.b + brightness).clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_contrast(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let contrast = parameters.get("contrast").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    pixel.r = ((pixel.r - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    pixel.g = ((pixel.g - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    pixel.b = ((pixel.b - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_saturation(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let saturation = parameters.get("saturation").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    let gray = pixel.luma();
                    pixel.r = (gray + saturation * (pixel.r - gray)).clamp(0.0, 255.0);
                    pixel.g = (gray + saturation * (pixel.g - gray)).clamp(0.0, 255.0);
                    pixel.b = (gray + saturation * (pixel.b - gray)).clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_hue(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let hue_shift = parameters.get("hue_shift").copied().unwrap_or(0.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    let (h, s, v) = self.rgb_to_hsv(pixel.r, pixel.g, pixel.b);
                    let new_h = (h + hue_shift) % 360.0;
                    let (r, g, b) = self.hsv_to_rgb(new_h, s, v);
                    
                    pixel.r = r.clamp(0.0, 255.0);
                    pixel.g = g.clamp(0.0, 255.0);
                    pixel.b = b.clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_gamma(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let gamma = parameters.get("gamma").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    pixel.r = (pixel.r / 255.0).powf(1.0 / gamma) * 255.0;
                    pixel.g = (pixel.g / 255.0).powf(1.0 / gamma) * 255.0;
                    pixel.b = (pixel.b / 255.0).powf(1.0 / gamma) * 255.0;
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_color_balance(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let red_gain = parameters.get("red_gain").copied().unwrap_or(1.0);
        let green_gain = parameters.get("green_gain").copied().unwrap_or(1.0);
        let blue_gain = parameters.get("blue_gain").copied().unwrap_or(1.0);

        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = output_frame.get_pixel(x, y) {
                    pixel.r = (pixel.r * red_gain).clamp(0.0, 255.0);
                    pixel.g = (pixel.g * green_gain).clamp(0.0, 255.0);
                    pixel.b = (pixel.b * blue_gain).clamp(0.0, 255.0);
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output_frame)
    }

    fn apply_custom(&self, frame: &crate::video_buffer::VideoFrame) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
Custom filter implementation
        Ok(frame.clone())
    }

    fn create_gaussian_kernel(&self, size: u32, sigma: f32) -> Vec<f32> {
        let mut kernel = Vec::with_capacity((size * size) as usize);
        let center = size as f32 / 2.0;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let value = (- (dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                kernel.push(value);
            }
        }

        let sum: f32 = kernel.iter().sum();
        kernel.iter_mut().for_each(|v| *v /= sum);

        kernel
    }

    fn rgb_to_hsv(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let r_norm = r / 255.0;
        let g_norm = g / 255.0;
        let b_norm = b / 255.0;

        let max = r_norm.max(g_norm).max(b_norm);
        let min = r_norm.min(g_norm).min(b_norm);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r_norm {
            60.0 * ((g_norm - b_norm) / delta)
        } else if max == g_norm {
            60.0 * ((b_norm - r_norm) / delta) + 120.0
        } else {
            60.0 * ((r_norm - g_norm) / delta) + 240.0
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h, s, v)
    }

    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h_prime < 1.0 {
            (c, x, m)
        } else if h_prime < 2.0 {
            (x, c, m)
        } else if h_prime < 3.0 {
            (m, c, x)
        } else if h_prime < 4.0 {
            (m, x, c)
        } else if h_prime < 5.0 {
            (x, m, c)
        } else {
            (c, m, x)
        };

        (r * 255.0, g * 255.0, b * 255.0)
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

    pub fn clone_filter(&self) -> VideoFilter {
        let mut new_filter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.filter_type.clone(),
        );

        let parameters = self.parameters.read();
        *new_filter.parameters = parameters.clone();

        new_filter
    }
}

impl Default for VideoFilter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Filter".to_string(),
            FilterType::Blur,
        )
    }
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::Blur
    }
}

impl Default for FilterResult {
    fn default() -> Self {
        Self {
            success: false,
            output_frame: None,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}
