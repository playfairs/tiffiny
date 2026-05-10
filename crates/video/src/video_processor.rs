use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoProcessor {
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
    Filter,
    ColorCorrection,
    Stabilization,
    Tracking,
    ObjectDetection,
    SceneDetection,
    MotionDetection,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    ParameterChanged(String, f32),
    EnabledChanged(bool),
    ProcessingStarted,
    ProcessingProgress(f32),
    ProcessingCompleted,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub success: bool,
    pub output_video: Option<Arc<crate::video_buffer::VideoBuffer>>,
    pub processing_time: std::time::Duration,
    pub frames_processed: usize,
    pub error_message: Option<String>,
}

impl VideoProcessor {
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

    pub async fn process(&self, input: &crate::video_buffer::VideoBuffer) -> Result<ProcessingResult, Box<dyn std::error::Error>> {
        if !*self.enabled.read() {
            return Ok(ProcessingResult {
                success: true,
                output_video: Some(Arc::new(input.clone())),
                processing_time: std::time::Duration::from_millis(0),
                frames_processed: input.frame_count,
                error_message: None,
            });
        }

        let _ = self.event_sender.send(ProcessorEvent::ProcessingStarted);
        let start_time = std::time::Instant::now();

        let result = match self.processor_type {
            ProcessorType::Resize => self.process_resize(input),
            ProcessorType::Crop => self.process_crop(input),
            ProcessorType::Rotate => self.process_rotate(input),
            ProcessorType::Flip => self.process_flip(input),
            ProcessorType::Filter => self.process_filter(input),
            ProcessorType::ColorCorrection => self.process_color_correction(input),
            ProcessorType::Stabilization => self.process_stabilization(input),
            ProcessorType::Tracking => self.process_tracking(input),
            ProcessorType::ObjectDetection => self.process_object_detection(input),
            ProcessorType::SceneDetection => self.process_scene_detection(input),
            ProcessorType::MotionDetection => self.process_motion_detection(input),
            ProcessorType::Custom(_) => self.process_custom(input),
        };

        let processing_time = start_time.elapsed();
        let frames_processed = input.frame_count;

        match result {
            Ok(output_video) => {
                let _ = self.event_sender.send(ProcessorEvent::ProcessingCompleted);
                Ok(ProcessingResult {
                    success: true,
                    output_video: Some(Arc::new(output_video)),
                    processing_time,
                    frames_processed,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Processing failed: {}", e);
                let _ = self.event_sender.send(ProcessorEvent::Error(error_msg.clone()));
                Ok(ProcessingResult {
                    success: false,
                    output_video: None,
                    processing_time,
                    frames_processed,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    fn process_resize(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let target_width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let target_height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;
        let algorithm = parameters.get("algorithm").copied().unwrap_or(1.0);0=nearest, 1=bilinear, 2=bicubic

        let mut output = crate::video_buffer::VideoBuffer::new(
            target_width,
            target_height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let resized_frame = self.resize_frame(input_frame, target_width, target_height, algorithm as u32);
                output.set_frame(frame_index, resized_frame);
            }
        }

        Ok(output)
    }

    fn process_crop(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let x = parameters.get("x").copied().unwrap_or(0.0) as u32;
        let y = parameters.get("y").copied().unwrap_or(0.0) as u32;
        let width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;

        let mut output = crate::video_buffer::VideoBuffer::new(
            width,
            height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let cropped_frame = self.crop_frame(input_frame, x, y, width, height);
                output.set_frame(frame_index, cropped_frame);
            }
        }

        Ok(output)
    }

    fn process_rotate(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let angle = parameters.get("angle").copied().unwrap_or(0.0);
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let background_color = parameters.get("background_r").copied().unwrap_or(0.0);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let rotated_frame = self.rotate_frame(input_frame, angle, center_x, center_y, background_color);
                output.set_frame(frame_index, rotated_frame);
            }
        }

        Ok(output)
    }

    fn process_flip(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let flip_horizontal = parameters.get("horizontal").copied().unwrap_or(0.0) > 0.5;
        let flip_vertical = parameters.get("vertical").copied().unwrap_or(0.0) > 0.5;

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let flipped_frame = self.flip_frame(input_frame, flip_horizontal, flip_vertical);
                output.set_frame(frame_index, flipped_frame);
            }
        }

        Ok(output)
    }

    fn process_filter(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let filter_type = parameters.get("filter_type").copied().unwrap_or(0.0);
        let strength = parameters.get("strength").copied().unwrap_or(1.0);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let filtered_frame = self.apply_frame_filter(input_frame, filter_type as u32, strength);
                output.set_frame(frame_index, filtered_frame);
            }
        }

        Ok(output)
    }

    fn process_color_correction(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let brightness = parameters.get("brightness").copied().unwrap_or(0.0);
        let contrast = parameters.get("contrast").copied().unwrap_or(1.0);
        let saturation = parameters.get("saturation").copied().unwrap_or(1.0);
        let gamma = parameters.get("gamma").copied().unwrap_or(1.0);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let corrected_frame = self.apply_color_correction(input_frame, brightness, contrast, saturation, gamma);
                output.set_frame(frame_index, corrected_frame);
            }
        }

        Ok(output)
    }

    fn process_stabilization(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let smoothing = parameters.get("smoothing").copied().unwrap_or(0.5);
        let max_translation = parameters.get("max_translation").copied().unwrap_or(50.0);
        let max_rotation = parameters.get("max_rotation").copied().unwrap_or(5.0);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let stabilized_frame = self.apply_stabilization(input_frame, frame_index, smoothing, max_translation, max_rotation);
                output.set_frame(frame_index, stabilized_frame);
            }
        }

        Ok(output)
    }

    fn process_tracking(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let tracker_type = parameters.get("tracker_type").copied().unwrap_or(0.0);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let tracked_frame = self.apply_tracking(input_frame, frame_index, tracker_type as u32);
                output.set_frame(frame_index, tracked_frame);
            }
        }

        Ok(output)
    }

    fn process_object_detection(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let detection_type = parameters.get("detection_type").copied().unwrap_or(0.0);
        let confidence_threshold = parameters.get("confidence").copied().unwrap_or(0.5);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let detected_frame = self.apply_object_detection(input_frame, detection_type as u32, confidence_threshold);
                output.set_frame(frame_index, detected_frame);
            }
        }

        Ok(output)
    }

    fn process_scene_detection(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let threshold = parameters.get("threshold").copied().unwrap_or(0.3);
        let min_scene_length = parameters.get("min_scene_length").copied().unwrap_or(15.0) as u32;

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let scene_detected_frame = self.apply_scene_detection(input_frame, frame_index, threshold, min_scene_length);
                output.set_frame(frame_index, scene_detected_frame);
            }
        }

        Ok(output)
    }

    fn process_motion_detection(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let sensitivity = parameters.get("sensitivity").copied().unwrap_or(0.5);
        let threshold = parameters.get("threshold").copied().unwrap_or(0.1);

        let mut output = crate::video_buffer::VideoBuffer::new(
            input.width,
            input.height,
            input.frame_rate,
            input.pixel_format,
            input.frame_count,
        );

        for frame_index in 0..input.frame_count {
            if let Some(input_frame) = input.get_frame(frame_index) {
                let motion_detected_frame = self.apply_motion_detection(input_frame, frame_index, sensitivity, threshold);
                output.set_frame(frame_index, motion_detected_frame);
            }
        }

        Ok(output)
    }

    fn process_custom(&self, input: &crate::video_buffer::VideoBuffer) -> Result<crate::video_buffer::VideoBuffer, Box<dyn std::error::Error>> {
        Ok(input.clone())
    }

    fn resize_frame(&self, frame: &crate::video_buffer::VideoFrame, width: u32, height: u32, algorithm: u32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = crate::video_buffer::VideoFrame::new(width, height, frame.pixel_format);

        match algorithm {
            0 => self.resize_nearest_neighbor(frame, &mut output_frame, width, height),
            1 => self.resize_bilinear(frame, &mut output_frame, width, height),
            2 => self.resize_bicubic(frame, &mut output_frame, width, height),
            _ => self.resize_bilinear(frame, &mut output_frame, width, height),
        }

        output_frame
    }

    fn resize_nearest_neighbor(&self, input: &crate::video_buffer::VideoFrame, output: &mut crate::video_buffer::VideoFrame, width: u32, height: u32) {
        let x_ratio = input.width as f32 / width as f32;
        let y_ratio = input.height as f32 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let src_x = (x as f32 * x_ratio).round() as u32;
                let src_y = (y as f32 * y_ratio).round() as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }
    }

    fn resize_bilinear(&self, input: &crate::video_buffer::VideoFrame, output: &mut crate::video_buffer::VideoFrame, width: u32, height: u32) {
        let x_ratio = input.width as f32 / width as f32;
        let y_ratio = input.height as f32 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let src_x = x as f32 * x_ratio;
                let src_y = y as f32 * y_ratio;
                
                let src_x0 = src_x.floor() as u32;
                let src_x1 = (src_x0 + 1).min(input.width - 1);
                let src_y0 = src_y.floor() as u32;
                let src_y1 = (src_y0 + 1).min(input.height - 1);
                
                let fx = src_x - src_x0 as f32;
                let fy = src_y - src_y0 as f32;
                
                if let (Some(p00), Some(p01), Some(p10), Some(p11)) = (
                    input.get_pixel(src_x0, src_y0),
                    input.get_pixel(src_x1, src_y0),
                    input.get_pixel(src_x0, src_y1),
                    input.get_pixel(src_x1, src_y1),
                ) {
                    let interpolated_pixel = crate::video_buffer::Pixel {
                        r: (p00.r * (1.0 - fx) * (1.0 - fy) + 
                           p01.r * fx * (1.0 - fy) + 
                           p10.r * (1.0 - fx) * fy + 
                           p11.r * fx * fy),
                        g: (p00.g * (1.0 - fx) * (1.0 - fy) + 
                           p01.g * fx * (1.0 - fy) + 
                           p10.g * (1.0 - fx) * fy + 
                           p11.g * fx * fy),
                        b: (p00.b * (1.0 - fx) * (1.0 - fy) + 
                           p01.b * fx * (1.0 - fy) + 
                           p10.b * (1.0 - fx) * fy + 
                           p11.b * fx * fy),
                        a: (p00.a * (1.0 - fx) * (1.0 - fy) + 
                           p01.a * fx * (1.0 - fy) + 
                           p10.a * (1.0 - fx) * fy + 
                           p11.a * fx * fy),
                    };
                    
                    output.set_pixel(x, y, interpolated_pixel);
                }
            }
        }
    }

    fn resize_bicubic(&self, input: &crate::video_buffer::VideoFrame, output: &mut crate::video_buffer::VideoFrame, width: u32, height: u32) {
        let x_ratio = input.width as f32 / width as f32;
        let y_ratio = input.height as f32 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let src_x = x as f32 * x_ratio;
                let src_y = y as f32 * y_ratio;
                
                let mut interpolated_pixel = crate::video_buffer::Pixel::default();
                
                for dy in -1..=2 {
                    for dx in -1..=2 {
                        let src_x0 = (src_x + dx as f32).floor() as u32;
                        let src_y0 = (src_y + dy as f32).floor() as u32;
                        
                        if let Some(pixel) = input.get_pixel(src_x0, src_y0) {
                            let weight = self.bicubic_weight(dx as f32, dy as f32);
                            interpolated_pixel.r += pixel.r * weight;
                            interpolated_pixel.g += pixel.g * weight;
                            interpolated_pixel.b += pixel.b * weight;
                            interpolated_pixel.a += pixel.a * weight;
                        }
                    }
                }
                
                output.set_pixel(x, y, interpolated_pixel);
            }
        }
    }

    fn bicubic_weight(&self, x: f32, y: f32) -> f32 {
        let abs_x = x.abs();
        let abs_y = y.abs();
        
        if abs_x <= 1.0 && abs_y <= 1.0 {
            let x_weight = if abs_x < 1.0 {
                1.0 - 2.0 * abs_x * abs_x + abs_x * abs_x * abs_x
            } else {
                4.0 - 8.0 * abs_x + 5.0 * abs_x * abs_x * abs_x
            };
            
            let y_weight = if abs_y < 1.0 {
                1.0 - 2.0 * abs_y * abs_y + abs_y * abs_y * abs_y
            } else {
                4.0 - 8.0 * abs_y + 5.0 * abs_y * abs_y * abs_y
            };
            
            x_weight * y_weight
        } else {
            0.0
        }
    }

    fn crop_frame(&self, frame: &crate::video_buffer::VideoFrame, x: u32, y: u32, width: u32, height: u32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = crate::video_buffer::VideoFrame::new(width, height, frame.pixel_format);

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                
                if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                    output_frame.set_pixel(dx, dy, pixel);
                }
            }
        }

        output_frame
    }

    fn rotate_frame(&self, frame: &crate::video_buffer::VideoFrame, angle: f32, center_x: f32, center_y: f32, background_color: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = crate::video_buffer::VideoFrame::new(frame.width, frame.height, frame.pixel_format);

        let angle_rad = angle.to_radians();
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        for y in 0..frame.height {
            for x in 0..frame.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                let src_x = (dx * cos_angle - dy * sin_angle + center_x).round() as u32;
                let src_y = (dx * sin_angle + dy * cos_angle + center_y).round() as u32;
                
                if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                    output_frame.set_pixel(x, y, pixel);
                } else {
                    let background_pixel = crate::video_buffer::Pixel {
                        r: background_color,
                        g: background_color,
                        b: background_color,
                        a: 255.0,
                    };
                    output_frame.set_pixel(x, y, background_pixel);
                }
            }
        }

        output_frame
    }

    fn flip_frame(&self, frame: &crate::video_buffer::VideoFrame, horizontal: bool, vertical: bool) -> crate::video_buffer::VideoFrame {
        let mut output_frame = crate::video_buffer::VideoFrame::new(frame.width, frame.height, frame.pixel_format);

        for y in 0..frame.height {
            for x in 0..frame.width {
                let src_x = if horizontal { frame.width - 1 - x } else { x };
                let src_y = if vertical { frame.height - 1 - y } else { y };
                
                if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        output_frame
    }

    fn apply_frame_filter(&self, frame: &crate::video_buffer::VideoFrame, filter_type: u32, strength: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();

        match filter_type {
            0 => self.apply_blur_filter(&mut output_frame, strength),
            1 => self.apply_sharpen_filter(&mut output_frame, strength),
            2 => self.apply_denoise_filter(&mut output_frame, strength),
            _ => output_frame,
        }

        output_frame
    }

    fn apply_blur_filter(&self, frame: &mut crate::video_buffer::VideoFrame, strength: f32) {
        let kernel_size = (strength * 5.0) as u32;
        let kernel = self.create_gaussian_kernel(kernel_size);
        
        for y in (kernel_size/2)..(frame.height - kernel_size/2) {
            for x in (kernel_size/2)..(frame.width - kernel_size/2) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                let mut total_weight = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let src_x = x + kx - kernel_size/2;
                        let src_y = y + ky - kernel_size/2;
                        
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
                    frame.set_pixel(x, y, blurred_pixel);
                }
            }
        }
    }

    fn apply_sharpen_filter(&self, frame: &mut crate::video_buffer::VideoFrame, strength: f32) {
        let kernel = vec![
            0.0, -strength, 0.0,
            -strength, 1.0 + 4.0 * strength, -strength,
            0.0, -strength, 0.0,
        ];

        for y in 1..(frame.height - 1) {
            for x in 1..(frame.width - 1) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                
                for ky in 0..3 {
                    for kx in 0..3 {
                        let src_x = x + kx - 1;
                        let src_y = y + ky - 1;
                        
                        if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                            let weight = kernel[ky * 3 + kx];
                            sum_r += pixel.r * weight;
                            sum_g += pixel.g * weight;
                            sum_b += pixel.b * weight;
                            sum_a += pixel.a * weight;
                        }
                    }
                }
                
                if let Some(mut pixel) = frame.get_pixel(x, y) {
                    pixel.r = (pixel.r + sum_r).clamp(0.0, 255.0);
                    pixel.g = (pixel.g + sum_g).clamp(0.0, 255.0);
                    pixel.b = (pixel.b + sum_b).clamp(0.0, 255.0);
                    pixel.a = (pixel.a + sum_a).clamp(0.0, 255.0);
                    frame.set_pixel(x, y, pixel);
                }
            }
        }
    }

    fn apply_denoise_filter(&self, frame: &mut crate::video_buffer::VideoFrame, strength: f32) {
        let kernel_size = 3;
        
        for y in 1..(frame.height - 1) {
            for x in 1..(frame.width - 1) {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let src_x = x + dx;
                        let src_y = y + dy;
                        
                        if let Some(pixel) = frame.get_pixel(src_x, src_y) {
                            sum_r += pixel.r;
                            sum_g += pixel.g;
                            sum_b += pixel.b;
                            sum_a += pixel.a;
                        }
                    }
                }
                
                let average_pixel = crate::video_buffer::Pixel {
                    r: sum_r / 9.0,
                    g: sum_g / 9.0,
                    b: sum_b / 9.0,
                    a: sum_a / 9.0,
                };
                
                if let Some(mut pixel) = frame.get_pixel(x, y) {
                    pixel.r = pixel.r * (1.0 - strength) + average_pixel.r * strength;
                    pixel.g = pixel.g * (1.0 - strength) + average_pixel.g * strength;
                    pixel.b = pixel.b * (1.0 - strength) + average_pixel.b * strength;
                    pixel.a = pixel.a;
                    frame.set_pixel(x, y, pixel);
                }
            }
        }
    }

    fn create_gaussian_kernel(&self, size: u32) -> Vec<f32> {
        let mut kernel = Vec::with_capacity((size * size) as usize);
        let center = size as f32 / 2.0;
        let sigma = 1.0;
        
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

    fn apply_color_correction(&self, frame: &crate::video_buffer::VideoFrame, brightness: f32, contrast: f32, saturation: f32, gamma: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = frame.get_pixel(x, y) {
                    pixel.r = (pixel.r + brightness).clamp(0.0, 255.0);
                    pixel.g = (pixel.g + brightness).clamp(0.0, 255.0);
                    pixel.b = (pixel.b + brightness).clamp(0.0, 255.0);
                    
                    pixel.r = ((pixel.r - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    pixel.g = ((pixel.g - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    pixel.b = ((pixel.b - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
                    
                    let gray = (0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b);
                    pixel.r = (gray + saturation * (pixel.r - gray)).clamp(0.0, 255.0);
                    pixel.g = (gray + saturation * (pixel.g - gray)).clamp(0.0, 255.0);
                    pixel.b = (gray + saturation * (pixel.b - gray)).clamp(0.0, 255.0);
                    
                    pixel.r = (pixel.r / 255.0).powf(1.0 / gamma) * 255.0;
                    pixel.g = (pixel.g / 255.0).powf(1.0 / gamma) * 255.0;
                    pixel.b = (pixel.b / 255.0).powf(1.0 / gamma) * 255.0;
                    
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }

        output_frame
    }

    fn apply_stabilization(&self, frame: &crate::video_buffer::VideoFrame, frame_index: usize, smoothing: f32, max_translation: f32, max_rotation: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();
        
        if frame_index > 0 {
            let stabilization_offset_x = (rand::random::<f32>() - 0.5) * max_translation;
            let stabilization_offset_y = (rand::random::<f32>() - 0.5) * max_translation;
            let stabilization_rotation = (rand::random::<f32>() - 0.5) * max_rotation;
            
            for y in 0..frame.height {
                for x in 0..frame.width {
                    if let Some(mut pixel) = frame.get_pixel(x, y) {
                        let src_x = (x as f32 + stabilization_offset_x).round() as u32;
                        let src_y = (y as f32 + stabilization_offset_y).round() as u32;
                        
                        if let Some(original_pixel) = frame.get_pixel(src_x, src_y) {
                            pixel.r = original_pixel.r * smoothing + pixel.r * (1.0 - smoothing);
                            pixel.g = original_pixel.g * smoothing + pixel.g * (1.0 - smoothing);
                            pixel.b = original_pixel.b * smoothing + pixel.b * (1.0 - smoothing);
                        }
                        
                        output_frame.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        output_frame
    }

    fn apply_tracking(&self, frame: &crate::video_buffer::VideoFrame, frame_index: usize, tracker_type: u32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();
        
        if frame_index > 0 {
            for y in 0..frame.height {
                for x in 0..frame.width {
                    if let Some(mut pixel) = frame.get_pixel(x, y) {
                        if (x + y) % 50 == 0 {
                            pixel.r = (pixel.r + 100.0).clamp(0.0, 255.0);
                            pixel.g = (pixel.g + 100.0).clamp(0.0, 255.0);
                            pixel.b = (pixel.b + 100.0).clamp(0.0, 255.0);
                        }
                        
                        output_frame.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        output_frame
    }

    fn apply_object_detection(&self, frame: &crate::video_buffer::VideoFrame, detection_type: u32, confidence_threshold: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();
        
        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(mut pixel) = frame.get_pixel(x, y) {
                    if (x + y) % 100 == 0 {
                        pixel.r = (pixel.r + 50.0).clamp(0.0, 255.0);
                        pixel.g = (pixel.g + 50.0).clamp(0.0, 255.0);
                        pixel.b = (pixel.b + 50.0).clamp(0.0, 255.0);
                    }
                    
                    output_frame.set_pixel(x, y, pixel);
                }
            }
        }
        
        output_frame
    }

    fn apply_scene_detection(&self, frame: &crate::video_buffer::VideoFrame, frame_index: usize, threshold: f32, min_scene_length: u32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();
        
        if frame_index > 0 && frame_index % 30 == 0 {
            for y in 0..frame.height {
                for x in 0..frame.width {
                    if let Some(mut pixel) = frame.get_pixel(x, y) {
                        pixel.r = (pixel.r + 25.0).clamp(0.0, 255.0);
                        pixel.g = (pixel.g + 25.0).clamp(0.0, 255.0);
                        pixel.b = (pixel.b + 25.0).clamp(0.0, 255.0);
                        
                        output_frame.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        output_frame
    }

    fn apply_motion_detection(&self, frame: &crate::video_buffer::VideoFrame, frame_index: usize, sensitivity: f32, threshold: f32) -> crate::video_buffer::VideoFrame {
        let mut output_frame = frame.clone();
        
        if frame_index > 0 {
            for y in 0..frame.height {
                for x in 0..frame.width {
                    if let Some(mut pixel) = frame.get_pixel(x, y) {
                        if (x + y) % 75 == 0 {
                            pixel.r = (pixel.r + 75.0).clamp(0.0, 255.0);
                            pixel.g = (pixel.g + 75.0).clamp(0.0, 255.0);
                            pixel.b = (pixel.b + 75.0).clamp(0.0, 255.0);
                        }
                        
                        output_frame.set_pixel(x, y, pixel);
                    }
                }
            }
        }
        
        output_frame
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

    pub fn clone_processor(&self) -> VideoProcessor {
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

impl Default for VideoProcessor {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Video Processor".to_string(),
            ProcessorType::Resize,
        )
    }
}

impl Default for ProcessorType {
    fn default() -> Self {
        ProcessorType::Resize
    }
}

impl Default for ProcessingResult {
    fn default() -> Self {
        Self {
            success: false,
            output_video: None,
            processing_time: std::time::Duration::from_millis(0),
            frames_processed: 0,
            error_message: None,
        }
    }
}
