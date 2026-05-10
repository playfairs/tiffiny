use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageTransform {
    pub id: String,
    pub name: String,
    pub transform_type: TransformType,
    pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
    pub enabled: Arc<RwLock<bool>>,
    pub event_sender: mpsc::UnboundedSender<TransformEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TransformEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformType {
    Translate,
    Rotate,
    Scale,
    Skew,
    Perspective,
    Affine,
    FlipHorizontal,
    FlipVertical,
    Crop,
    Resize,
    Warp,
    Distort,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum TransformEvent {
    ParameterChanged(String, f32),
    EnabledChanged(bool),
    TransformApplied,
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
    pub m: [[f32; 3]; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ImageTransform {
    pub fn new(id: String, name: String, transform_type: TransformType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            transform_type,
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

        let _ = self.event_sender.send(TransformEvent::TransformApplied);

        match self.transform_type {
            TransformType::Translate => self.apply_translate(input),
            TransformType::Rotate => self.apply_rotate(input),
            TransformType::Scale => self.apply_scale(input),
            TransformType::Skew => self.apply_skew(input),
            TransformType::Perspective => self.apply_perspective(input),
            TransformType::Affine => self.apply_affine(input),
            TransformType::FlipHorizontal => self.apply_flip_horizontal(input),
            TransformType::FlipVertical => self.apply_flip_vertical(input),
            TransformType::Crop => self.apply_crop(input),
            TransformType::Resize => self.apply_resize(input),
            TransformType::Warp => self.apply_warp(input),
            TransformType::Distort => self.apply_distort(input),
            TransformType::Custom(_) => self.apply_custom(input),
        }
    }

    fn apply_translate(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let dx = parameters.get("dx").copied().unwrap_or(0.0);
        let dy = parameters.get("dy").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let src_x = (x as f32 - dx).clamp(0.0, input.width as f32 - 1.0) as u32;
                let src_y = (y as f32 - dy).clamp(0.0, input.height as f32 - 1.0) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_rotate(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let angle = parameters.get("angle").copied().unwrap_or(0.0);degrees
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let background_color = parameters.get("background_r").copied().unwrap_or(0.0);

        let angle_rad = angle.to_radians();
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

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

    fn apply_scale(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let scale_x = parameters.get("scale_x").copied().unwrap_or(1.0);
        let scale_y = parameters.get("scale_y").copied().unwrap_or(1.0);
        let interpolation = parameters.get("interpolation").copied().unwrap_or(1.0);

        let new_width = (input.width as f32 * scale_x).round() as u32;
        let new_height = (input.height as f32 * scale_y).round() as u32;
        let mut output = crate::image_buffer::ImageBuffer::new(new_width, new_height, input.pixel_format.clone());

        match interpolation as i32 {
            0 => self.scale_nearest_neighbor(input, &mut output, scale_x, scale_y),
            1 => self.scale_bilinear(input, &mut output, scale_x, scale_y),
            2 => self.scale_bicubic(input, &mut output, scale_x, scale_y),
            _ => self.scale_bilinear(input, &mut output, scale_x, scale_y),
        }

        Ok(output)
    }

    fn scale_nearest_neighbor(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, scale_x: f32, scale_y: f32) {
        for y in 0..output.height {
            for x in 0..output.width {
                let src_x = (x as f32 / scale_x).round() as u32;
                let src_y = (y as f32 / scale_y).round() as u32;
                
                if src_x < input.width && src_y < input.height {
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }
    }

    fn scale_bilinear(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, scale_x: f32, scale_y: f32) {
        for y in 0..output.height {
            for x in 0..output.width {
                let src_x = x as f32 / scale_x;
                let src_y = y as f32 / scale_y;
                
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
                    let interpolated_pixel = crate::image_buffer::Pixel {
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

    fn scale_bicubic(&self, input: &crate::image_buffer::ImageBuffer, output: &mut crate::image_buffer::ImageBuffer, scale_x: f32, scale_y: f32) {
        for y in 0..output.height {
            for x in 0..output.width {
                let src_x = x as f32 / scale_x;
                let src_y = y as f32 / scale_y;
                
                let src_x0 = src_x.floor() as u32;
                let src_y0 = src_y.floor() as u32;
                
                let fx = src_x - src_x0 as f32;
                let fy = src_y - src_y0 as f32;
                
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;
                
                for dy in -1..=2 {
                    for dx in -1..=2 {
                        let sx = (src_x0 as i32 + dx).clamp(0, input.width as i32 - 1) as u32;
                        let sy = (src_y0 as i32 + dy).clamp(0, input.height as i32 - 1) as u32;
                        
                        if let Some(pixel) = input.get_pixel(sx, sy) {
                            let weight = self.bicubic_weight(dx as f32 - fx, dy as f32 - fy);
                            sum_r += pixel.r * weight;
                            sum_g += pixel.g * weight;
                            sum_b += pixel.b * weight;
                            sum_a += pixel.a * weight;
                        }
                    }
                }
                
                let interpolated_pixel = crate::image_buffer::Pixel {
                    r: sum_r,
                    g: sum_g,
                    b: sum_b,
                    a: sum_a,
                };
                
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
                4.0 - 8.0 * abs_x + 5.0 * abs_x * abs_x - abs_x * abs_x * abs_x
            };
            
            let y_weight = if abs_y < 1.0 {
                1.0 - 2.0 * abs_y * abs_y + abs_y * abs_y * abs_y
            } else {
                4.0 - 8.0 * abs_y + 5.0 * abs_y * abs_y - abs_y * abs_y * abs_y
            };
            
            x_weight * y_weight
        } else {
            0.0
        }
    }

    fn apply_skew(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let skew_x = parameters.get("skew_x").copied().unwrap_or(0.0);
        let skew_y = parameters.get("skew_y").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let src_x = (x as f32 + y as f32 * skew_x).clamp(0.0, input.width as f32 - 1.0) as u32;
                let src_y = (y as f32 + x as f32 * skew_y).clamp(0.0, input.height as f32 - 1.0) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_perspective(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let corners = self.get_perspective_corners(&parameters);
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
        
        let matrix = self.calculate_perspective_matrix(&corners);
        
        for y in 0..input.height {
            for x in 0..input.width {
                let point = self.apply_perspective_transform(x as f32, y as f32, &matrix);
                
                let src_x = point.x.clamp(0.0, input.width as f32 - 1.0) as u32;
                let src_y = point.y.clamp(0.0, input.height as f32 - 1.0) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn get_perspective_corners(&self, parameters: &std::collections::HashMap<String, f32>) -> Vec<Point2D> {
        vec![
            Point2D {
                x: parameters.get("x0").copied().unwrap_or(0.0),
                y: parameters.get("y0").copied().unwrap_or(0.0),
            },
            Point2D {
                x: parameters.get("x1").copied().unwrap_or(input.width as f32 - 1.0),
                y: parameters.get("y1").copied().unwrap_or(0.0),
            },
            Point2D {
                x: parameters.get("x2").copied().unwrap_or(input.width as f32 - 1.0),
                y: parameters.get("y2").copied().unwrap_or(input.height as f32 - 1.0),
            },
            Point2D {
                x: parameters.get("x3").copied().unwrap_or(0.0),
                y: parameters.get("y3").copied().unwrap_or(input.height as f32 - 1.0),
            },
        ]
    }

    fn calculate_perspective_matrix(&self, corners: &[Point2D]) -> Matrix3x3 {
        let src_corners = [
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 0.0 },
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 0.0, y: 1.0 },
        ];
        
        Matrix3x3 {
            m: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    fn apply_perspective_transform(&self, x: f32, y: f32, matrix: &Matrix3x3) -> Point2D {
        let w = matrix.m[2][0] * x + matrix.m[2][1] * y + matrix.m[2][2];
        if w.abs() < 0.0001 {
            Point2D { x, y }
        } else {
            Point2D {
                x: (matrix.m[0][0] * x + matrix.m[0][1] * y + matrix.m[0][2]) / w,
                y: (matrix.m[1][0] * x + matrix.m[1][1] * y + matrix.m[1][2]) / w,
            }
        }
    }

    fn apply_affine(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let matrix = self.get_affine_matrix(&parameters);
        
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let point = self.apply_affine_transform(x as f32, y as f32, &matrix);
                
                let src_x = point.x.clamp(0.0, input.width as f32 - 1.0) as u32;
                let src_y = point.y.clamp(0.0, input.height as f32 - 1.0) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn get_affine_matrix(&self, parameters: &std::collections::HashMap<String, f32>) -> Matrix3x3 {
        let a = parameters.get("a").copied().unwrap_or(1.0);
        let b = parameters.get("b").copied().unwrap_or(0.0);
        let c = parameters.get("c").copied().unwrap_or(0.0);
        let d = parameters.get("d").copied().unwrap_or(1.0);
        let e = parameters.get("e").copied().unwrap_or(0.0);
        let f = parameters.get("f").copied().unwrap_or(0.0);
        
        Matrix3x3 {
            m: [
                [a, b, c],
                [d, e, f],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    fn apply_affine_transform(&self, x: f32, y: f32, matrix: &Matrix3x3) -> Point2D {
        Point2D {
            x: matrix.m[0][0] * x + matrix.m[0][1] * y + matrix.m[0][2],
            y: matrix.m[1][0] * x + matrix.m[1][1] * y + matrix.m[1][2],
        }
    }

    fn apply_flip_horizontal(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let src_x = input.width - 1 - x;
                if let Some(pixel) = input.get_pixel(src_x, y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_flip_vertical(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let src_y = input.height - 1 - y;
                if let Some(pixel) = input.get_pixel(x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_crop(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let x = parameters.get("x").copied().unwrap_or(0.0) as u32;
        let y = parameters.get("y").copied().unwrap_or(0.0) as u32;
        let width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;

        let mut output = crate::image_buffer::ImageBuffer::new(width, height, input.pixel_format.clone());

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

    fn apply_resize(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let width = parameters.get("width").copied().unwrap_or(input.width as f32) as u32;
        let height = parameters.get("height").copied().unwrap_or(input.height as f32) as u32;
        let maintain_aspect = parameters.get("maintain_aspect").copied().unwrap_or(0.0) > 0.5;

        let (final_width, final_height) = if maintain_aspect {
            let aspect_ratio = input.width as f32 / input.height as f32;
            let target_aspect = width as f32 / height as f32;
            
            if target_aspect > aspect_ratio {
                (width, (width as f32 / aspect_ratio) as u32)
            } else {
                ((height as f32 * aspect_ratio) as u32, height)
            }
        } else {
            (width, height)
        };

        let scale_x = final_width as f32 / input.width as f32;
        let scale_y = final_height as f32 / input.height as f32;
        
        self.scale_with_interpolation(input, scale_x, scale_y, 1)
    }

    fn scale_with_interpolation(&self, input: &crate::image_buffer::ImageBuffer, scale_x: f32, scale_y: f32, interpolation: i32) -> crate::image_buffer::ImageBuffer {
        let new_width = (input.width as f32 * scale_x).round() as u32;
        let new_height = (input.height as f32 * scale_y).round() as u32;
        let mut output = crate::image_buffer::ImageBuffer::new(new_width, new_height, input.pixel_format.clone());

        match interpolation {
            0 => self.scale_nearest_neighbor(input, &mut output, scale_x, scale_y),
            1 => self.scale_bilinear(input, &mut output, scale_x, scale_y),
            2 => self.scale_bicubic(input, &mut output, scale_x, scale_y),
            _ => self.scale_bilinear(input, &mut output, scale_x, scale_y),
        }

        output
    }

    fn apply_warp(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let warp_type = parameters.get("warp_type").copied().unwrap_or(0.0);
        
        match warp_type as i32 {
            0 => self.apply_radial_warp(input),
            1 => self.apply_wave_warp(input),
            2 => self.apply_swirl_warp(input),
            _ => self.apply_radial_warp(input),
        }
    }

    fn apply_radial_warp(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let strength = parameters.get("strength").copied().unwrap_or(0.1);
        let radius = parameters.get("radius").copied().unwrap_or(input.width.min(input.height) as f32 / 4.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let warp_factor = if distance < radius {
                    1.0 + strength * (1.0 - distance / radius)
                } else {
                    1.0
                };
                
                let src_x = (center_x + dx * warp_factor).clamp(0.0, input.width as f32 - 1.0) as u32;
                let src_y = (center_y + dy * warp_factor).clamp(0.0, input.height as f32 - 1.0) as u32;
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_wave_warp(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let amplitude = parameters.get("amplitude").copied().unwrap_or(10.0);
        let frequency = parameters.get("frequency").copied().unwrap_or(0.05);
        let direction = parameters.get("direction").copied().unwrap_or(0.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let offset = if direction as i32 == 0 {
                    amplitude * (2.0 * std::f32::consts::PI * frequency * y as f32).sin()
                } else {
                    amplitude * (2.0 * std::f32::consts::PI * frequency * x as f32).sin()
                };
                
                let src_x = if direction as i32 == 0 {
                    (x as f32 + offset).clamp(0.0, input.width as f32 - 1.0) as u32
                } else {
                    x
                };
                
                let src_y = if direction as i32 == 1 {
                    (y as f32 + offset).clamp(0.0, input.height as f32 - 1.0) as u32
                } else {
                    y
                };
                
                if let Some(pixel) = input.get_pixel(src_x, src_y) {
                    output.set_pixel(x, y, pixel);
                }
            }
        }

        Ok(output)
    }

    fn apply_swirl_warp(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let strength = parameters.get("strength").copied().unwrap_or(0.5);
        let radius = parameters.get("radius").copied().unwrap_or(input.width.min(input.height) as f32 / 3.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < radius {
                    let angle = dy.atan2(dx);
                    let swirl_angle = strength * (1.0 - distance / radius) * 2.0 * std::f32::consts::PI;
                    
                    let src_x = (center_x + distance * (angle + swirl_angle).cos()).clamp(0.0, input.width as f32 - 1.0) as u32;
                    let src_y = (center_y + distance * (angle + swirl_angle).sin()).clamp(0.0, input.height as f32 - 1.0) as u32;
                    
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(x, y, pixel);
                    }
                } else {
                    if let Some(pixel) = input.get_pixel(x, y) {
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }

        Ok(output)
    }

    fn apply_distort(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let distortion_type = parameters.get("distortion_type").copied().unwrap_or(0.0);

        match distortion_type as i32 {
            0 => self.apply_barrel_distortion(input),
            1 => self.apply_pincushion_distortion(input),
            2 => self.apply_fisheye_distortion(input),
            _ => self.apply_barrel_distortion(input),
        }
    }

    fn apply_barrel_distortion(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let strength = parameters.get("strength").copied().unwrap_or(0.5);
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let radius = parameters.get("radius").copied().unwrap_or(input.width.min(input.height) as f32 / 2.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < radius {
                    let factor = 1.0 + strength * (1.0 - distance / radius) * (1.0 - distance / radius);
                    
                    let src_x = (center_x + dx * factor).clamp(0.0, input.width as f32 - 1.0) as u32;
                    let src_y = (center_y + dy * factor).clamp(0.0, input.height as f32 - 1.0) as u32;
                    
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(x, y, pixel);
                    }
                } else {
                    if let Some(pixel) = input.get_pixel(x, y) {
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }

        Ok(output)
    }

    fn apply_pincushion_distortion(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let strength = parameters.get("strength").copied().unwrap_or(0.5);
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let radius = parameters.get("radius").copied().unwrap_or(input.width.min(input.height) as f32 / 2.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < radius {
                    let factor = 1.0 - strength * (1.0 - distance / radius) * (1.0 - distance / radius);
                    
                    let src_x = (center_x + dx * factor).clamp(0.0, input.width as f32 - 1.0) as u32;
                    let src_y = (center_y + dy * factor).clamp(0.0, input.height as f32 - 1.0) as u32;
                    
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(x, y, pixel);
                    }
                } else {
                    if let Some(pixel) = input.get_pixel(x, y) {
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }

        Ok(output)
    }

    fn apply_fisheye_distortion(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let parameters = self.parameters.read();
        let strength = parameters.get("strength").copied().unwrap_or(0.5);
        let center_x = parameters.get("center_x").copied().unwrap_or(input.width as f32 / 2.0);
        let center_y = parameters.get("center_y").copied().unwrap_or(input.height as f32 / 2.0);
        let radius = parameters.get("radius").copied().unwrap_or(input.width.min(input.height) as f32 / 2.0);

        let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < radius {
                    let factor = (distance / radius).atan() / (std::f32::consts::PI / 2.0) * strength;
                    
                    let src_x = (center_x + dx * factor).clamp(0.0, input.width as f32 - 1.0) as u32;
                    let src_y = (center_y + dy * factor).clamp(0.0, input.height as f32 - 1.0) as u32;
                    
                    if let Some(pixel) = input.get_pixel(src_x, src_y) {
                        output.set_pixel(x, y, pixel);
                    }
                } else {
                    if let Some(pixel) = input.get_pixel(x, y) {
                        output.set_pixel(x, y, pixel);
                    }
                }
            }
        }

        Ok(output)
    }

    fn apply_custom(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        Ok(input.clone())
    }

    pub fn set_parameter(&self, name: &str, value: f32) {
        let mut parameters = self.parameters.write();
        parameters.insert(name.to_string(), value);
        
        let _ = self.event_sender.send(TransformEvent::ParameterChanged(name.to_string(), value));
    }

    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        let parameters = self.parameters.read();
        parameters.get(name).copied()
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled_state = self.enabled.write();
        *enabled_state = enabled;
        
        let _ = self.event_sender.send(TransformEvent::EnabledChanged(enabled));
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    pub async fn get_events(&mut self) -> Vec<TransformEvent> {
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

    pub fn clone_transform(&self) -> ImageTransform {
        let mut new_transform = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.transform_type.clone(),
        );
        
        let parameters = self.parameters.read();
        *new_transform.parameters.write() = parameters.clone();
        
        new_transform
    }
}

impl Default for ImageTransform {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Default Transform".to_string(),
            TransformType::Translate,
        )
    }
}

impl Default for TransformType {
    fn default() -> Self {
        TransformType::Translate
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Default for Point2D {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Default for Point3D {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}
