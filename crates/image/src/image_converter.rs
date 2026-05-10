use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageConverter {
    pub id: String,
    pub input_format: Arc<RwLock<Option<crate::image_format::ImageFormat>>>,
    pub output_format: Arc<RwLock<Option<crate::image_format::ImageFormat>>>,
    pub converter: Arc<RwLock<Option<Arc<dyn ImageFormatConverter>>>>,
    pub event_sender: mpsc::UnboundedSender<ConverterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ConverterEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ConverterEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted(String),
    ConversionFailed(String),
    FormatChanged(crate::image_format::ImageFormat),
}

#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub current_pixel: usize,
    pub total_pixels: usize,
    pub current_row: usize,
    pub total_rows: usize,
    pub processing_speed: f64,
    pub eta: Option<std::time::Duration>,
}

pub trait ImageFormatConverter: Send + Sync {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>>;
    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat);
    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat);
    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat>;
    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat>;
    fn reset(&mut self);
    fn get_conversion_info(&self) -> ConversionInfo;
}

#[derive(Debug, Clone)]
pub struct ConversionInfo {
    pub input_format: crate::image_format::ImageFormat,
    pub output_format: crate::image_format::ImageFormat,
    pub conversion_type: ConversionType,
    pub quality_loss: Option<f32>,
    pub processing_time: Option<std::time::Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionType {
    PixelFormat,
    ColorSpace,
    BitDepth,
    Compression,
    Complex,
}

impl ImageConverter {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_format: Arc::new(RwLock::new(None)),
            output_format: Arc::new(RwLock::new(None)),
            converter: Arc::new(RwLock::new(None)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn set_input_format(&self, format: crate::image_format::ImageFormat) {
        let mut input_format = self.input_format.write();
        *input_format = Some(format.clone());
        
        let _ = self.event_sender.send(ConverterEvent::FormatChanged(format));
        
Update converter
        self.update_converter();
    }

    pub fn set_output_format(&self, format: crate::image_format::ImageFormat) {
        let mut output_format = self.output_format.write();
        *output_format = Some(format.clone());
        
        let _ = self.event_sender.send(ConverterEvent::FormatChanged(format));
        
        self.update_converter();
    }

    pub async fn convert(&self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        
        let mut converter = self.converter.write();
        
        if let Some(ref mut conv) = *converter {
            let input_format = self.input_format.read();
            let output_format = self.output_format.read();
            
            if let Some(ref in_fmt) = input_format {
                conv.set_input_format(in_fmt);
            }
            if let Some(ref out_fmt) = output_format {
                conv.set_output_format(out_fmt);
            }
            
            let result = conv.convert(input);
            
            match result {
                Ok(output) => {
                    let _ = self.event_sender.send(ConverterEvent::ConversionCompleted("Conversion successful".to_string()));
                    Ok(output)
                },
                Err(e) => {
                    let error_msg = format!("Conversion failed: {}", e);
                    let _ = self.event_sender.send(ConverterEvent::ConversionFailed(error_msg.clone()));
                    Err(e)
                },
            }
        } else {
            Err("No converter available".into())
        }
    }

    pub async fn convert_with_progress<F>(&self, input: &crate::image_buffer::ImageBuffer, progress_callback: F) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>>
    where
        F: Fn(ConversionProgress) + Send + Sync,
    {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        
        let mut converter = self.converter.write();
        
        if let Some(ref mut conv) = *converter {
            let input_format = self.input_format.read();
            let output_format = self.output_format.read();
            
            if let Some(ref in_fmt) = input_format {
                conv.set_input_format(in_fmt);
            }
            if let Some(ref out_fmt) = output_format {
                conv.set_output_format(out_fmt);
            }
            
            let chunk_size = 1024;
            let mut output = crate::image_buffer::ImageBuffer::new(input.width, input.height, input.pixel_format.clone());
            
            for y in (0..input.height).step_by(chunk_size) {
                let end_y = (y + chunk_size).min(input.height);
                
                for x in 0..input.width {
                    for dy in y..end_y {
                        if let Some(pixel) = input.get_pixel(x, dy) {
                            output.set_pixel(x, dy, pixel);
                        }
                    }
                }
                
                let progress = ConversionProgress {
                    current_pixel: (end_y * input.width) as usize,
                    total_pixels: (input.height * input.width) as usize,
                    current_row: end_y as usize,
                    total_rows: input.height as usize,
                    processing_speed: chunk_size as f64 / 0.1,
                    eta: if end_y > 0 {
                        let remaining_rows = input.height - end_y;
                        Some(std::time::Duration::from_secs_f64(remaining_rows as f64 / (chunk_size as f64 / 0.1)))
                    } else {
                        None
                    },
                };
                
                progress_callback(progress.clone());
                
                let progress_percent = (end_y as f32 / input.height as f32) * 100.0;
                let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress_percent));
                
                tokio::task::yield_now().await;
            }
            
            let _ = self.event_sender.send(ConverterEvent::ConversionCompleted("Conversion successful".to_string()));
            Ok(output)
        } else {
            Err("No converter available".into())
        }
    }

    pub fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        self.input_format.read().clone()
    }

    pub fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        self.output_format.read().clone()
    }

    pub fn get_conversion_type(&self) -> ConversionType {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if let (Some(in_fmt), Some(out_fmt)) = (input_format.as_ref(), output_format.as_ref()) {
            if in_fmt.pixel_format != out_fmt.pixel_format {
                ConversionType::PixelFormat
            } else if in_fmt.color_space != out_fmt.color_space {
                ConversionType::ColorSpace
            } else if in_fmt.bit_depth != out_fmt.bit_depth {
                ConversionType::BitDepth
            } else if in_fmt.compression != out_fmt.compression {
                ConversionType::Compression
            } else {
                ConversionType::Complex
            }
        } else {
            ConversionType::Complex
        }
    }

    pub fn is_conversion_needed(&self) -> bool {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if let (Some(in_fmt), Some(out_fmt)) = (input_format.as_ref(), output_format.as_ref()) {
            in_fmt != out_fmt
        } else {
            false
        }
    }

    pub fn estimate_quality_loss(&self) -> Option<f32> {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if let (Some(in_fmt), Some(out_fmt)) = (input_format.as_ref(), output_format.as_ref()) {
            if in_fmt.is_lossless() && out_fmt.is_lossless() {
                None
            } else if in_fmt.is_lossless() && out_fmt.is_lossy() {
                Some(100.0)
            } else if in_fmt.is_lossy() && out_fmt.is_lossless() {
                Some(0.0)
            } else {
                self.estimate_lossy_quality_loss(in_fmt, out_fmt)
            }
        } else {
            None
        }
    }

    fn estimate_lossy_quality_loss(&self, input: &crate::image_format::ImageFormat, output: &crate::image_format::ImageFormat) -> Option<f32> {
        if let (Some(input_quality), Some(output_quality)) = (input.quality, output.quality) {
            if output_quality >= input_quality {
                Some(0.0)
            } else {
                Some(((input_quality - output_quality) as f32 / input_quality as f32) * 100.0)
            }
        } else {
            Some(10.0)
        }
    }

    fn update_converter(&self) {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if let (Some(in_fmt), Some(out_fmt)) = (input_format.as_ref(), output_format.as_ref()) {
            let converter = self.create_converter(in_fmt, out_fmt);
            let mut converter_guard = self.converter.write();
            *converter_guard = Some(converter);
        }
    }

    fn create_converter(&self, input: &crate::image_format::ImageFormat, output: &crate::image_format::ImageFormat) -> Arc<dyn ImageFormatConverter> {
        if input.pixel_format != output.pixel_format {
            Arc::new(PixelFormatConverter::new(input.clone(), output.clone()))
        } else if input.color_space != output.color_space {
            Arc::new(ColorSpaceConverter::new(input.clone(), output.clone()))
        } else if input.bit_depth != output.bit_depth {
            Arc::new(BitDepthConverter::new(input.clone(), output.clone()))
        } else if input.compression != output.compression {
            Arc::new(CompressionConverter::new(input.clone(), output.clone()))
        } else {
            Arc::new(PassthroughConverter::new(input.clone()))
        }
    }

    pub async fn get_events(&mut self) -> Vec<ConverterEvent> {
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

    pub fn get_conversion_stats(&self) -> ConversionStats {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        let conversion_type = self.get_conversion_type();
        let quality_loss = self.estimate_quality_loss();
        
        ConversionStats {
            conversion_type,
            input_format: input_format.clone(),
            output_format: output_format.clone(),
            quality_loss,
            is_conversion_needed: self.is_conversion_needed(),
            estimated_processing_time: self.estimate_processing_time(&input_format, &output_format),
        }
    }

    fn estimate_processing_time(&self, input: &Option<crate::image_format::ImageFormat>, output: &Option<crate::image_format::ImageFormat>) -> Option<std::time::Duration> {
        if let (Some(in_fmt), Some(out_fmt)) = (input.as_ref(), output.as_ref()) {
            let complexity_factor = match self.get_conversion_type() {
                ConversionType::PixelFormat => 1.5,
                ConversionType::ColorSpace => 2.0,
                ConversionType::BitDepth => 1.2,
                ConversionType::Compression => 3.0,
                ConversionType::Complex => 4.0,
            };
            
            let pixels = in_fmt.width * in_fmt.height;
            let processing_time_ms = (pixels as f64 * complexity_factor) / 1_000_000.0;
            
            Some(std::time::Duration::from_millis(processing_time_ms as u64))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub conversion_type: ConversionType,
    pub input_format: Option<crate::image_format::ImageFormat>,
    pub output_format: Option<crate::image_format::ImageFormat>,
    pub quality_loss: Option<f32>,
    pub is_conversion_needed: bool,
    pub estimated_processing_time: Option<std::time::Duration>,
}

struct PixelFormatConverter {
    input_format: crate::image_format::ImageFormat,
    output_format: crate::image_format::ImageFormat,
}

impl PixelFormatConverter {
    fn new(input: crate::image_format::ImageFormat, output: crate::image_format::ImageFormat) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl ImageFormatConverter for PixelFormatConverter {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let input_data = input.clone_data();
        let mut output_data = Vec::with_capacity(input_data.len());
        
        for &pixel in &input_data {
            let converted_pixel = self.convert_pixel_format(pixel);
            output_data.push(converted_pixel);
        }
        
        Ok(crate::image_buffer::ImageBuffer::from_samples(
            output_data,
            self.output_format.width,
            self.output_format.height,
            self.output_format.pixel_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::PixelFormat,
            quality_loss: None,
            processing_time: None,
        }
    }

    fn convert_pixel_format(&self, pixel: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        match (&self.input_format.pixel_format, &self.output_format.pixel_format) {
            (crate::image_buffer::PixelFormat::Rgb8, crate::image_buffer::PixelFormat::Rgba8) => {
                crate::image_buffer::Pixel::new(pixel.r, pixel.g, pixel.b, 255.0)
            },
            (crate::image_buffer::PixelFormat::Rgba8, crate::image_buffer::PixelFormat::Rgb8) => {
                crate::image_buffer::Pixel::new(pixel.r, pixel.g, pixel.b, pixel.a)
            },
            (crate::image_buffer::PixelFormat::Rgb8, crate::image_buffer::PixelFormat::Grayscale8) => {
                crate::image_buffer::Pixel::gray(pixel.luma())
            },
            (crate::image_buffer::PixelFormat::Grayscale8, crate::image_buffer::PixelFormat::Rgb8) => {
                crate::image_buffer::Pixel::rgb(pixel.r, pixel.r, pixel.r)
            },
            _ => pixel,
        }
    }
}

struct ColorSpaceConverter {
    input_format: crate::image_format::ImageFormat,
    output_format: crate::image_format::ImageFormat,
}

impl ColorSpaceConverter {
    fn new(input: crate::image_format::ImageFormat, output: crate::image_format::ImageFormat) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl ImageFormatConverter for ColorSpaceConverter {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let input_data = input.clone_data();
        let mut output_data = Vec::with_capacity(input_data.len());
        
        for &pixel in &input_data {
            let converted_pixel = self.convert_color_space(pixel);
            output_data.push(converted_pixel);
        }
        
        Ok(crate::image_buffer::ImageBuffer::from_samples(
            output_data,
            self.output_format.width,
            self.output_format.height,
            self.output_format.pixel_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::ColorSpace,
            quality_loss: None,
            processing_time: None,
        }
    }

    fn convert_color_space(&self, pixel: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        match (&self.input_format.color_space, &self.output_format.color_space) {
            ("sRGB", "HSV") => {
                let (h, s, v) = self.rgb_to_hsv(pixel.r, pixel.g, pixel.b);
                crate::image_buffer::Pixel::new(h, s, v, pixel.a)
            },
            ("HSV", "sRGB") => {
                let (r, g, b) = self.hsv_to_rgb(pixel.r, pixel.g, pixel.b);
                crate::image_buffer::Pixel::new(r, g, b, pixel.a)
            },
            _ => pixel,
        }
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
}

struct BitDepthConverter {
    input_format: crate::image_format::ImageFormat,
    output_format: crate::image_format::ImageFormat,
}

impl BitDepthConverter {
    fn new(input: crate::image_format::ImageFormat, output: crate::image_format::ImageFormat) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl ImageFormatConverter for BitDepthConverter {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let input_data = input.clone_data();
        let mut output_data = Vec::with_capacity(input_data.len());
        
        for &pixel in &input_data {
            let converted_pixel = self.convert_bit_depth(pixel);
            output_data.push(converted_pixel);
        }
        
        Ok(crate::image_buffer::ImageBuffer::from_samples(
            output_data,
            self.output_format.width,
            self.output_format.height,
            self.output_format.pixel_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::BitDepth,
            quality_loss: self.estimate_quality_loss(),
            processing_time: None,
        }
    }

    fn convert_bit_depth(&self, pixel: crate::image_buffer::Pixel) -> crate::image_buffer::Pixel {
        match (&self.input_format.bit_depth, &self.output_format.bit_depth) {
            (8, 16) => {
                crate::image_buffer::Pixel::new(
                    pixel.r * 257.0,
                    pixel.g * 257.0,
                    pixel.b * 257.0,
                    pixel.a * 257.0,
                )
            },
            (16, 8) => {
                crate::image_buffer::Pixel::new(
                    pixel.r / 257.0,
                    pixel.g / 257.0,
                    pixel.b / 257.0,
                    pixel.a / 257.0,
                )
            },
            (8, 32) => {
                crate::image_buffer::Pixel::new(pixel.r, pixel.g, pixel.b, pixel.a)
            },
            (32, 8) => {
                crate::image_buffer::Pixel::new(
                    pixel.r.clamp(0.0, 255.0),
                    pixel.g.clamp(0.0, 255.0),
                    pixel.b.clamp(0.0, 255.0),
                    pixel.a.clamp(0.0, 255.0),
                )
            },
            _ => pixel,
        }
    }

    fn estimate_quality_loss(&self) -> Option<f32> {
        let input_bits = self.input_format.bit_depth;
        let output_bits = self.output_format.bit_depth;
        
        if output_bits >= input_bits {
            None
        } else {
            Some(((input_bits - output_bits) as f32 / input_bits as f32) * 100.0)
        }
    }
}

struct CompressionConverter {
    input_format: crate::image_format::ImageFormat,
    output_format: crate::image_format::ImageFormat,
}

impl CompressionConverter {
    fn new(input: crate::image_format::ImageFormat, output: crate::image_format::ImageFormat) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl ImageFormatConverter for CompressionConverter {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        Ok(crate::image_buffer::ImageBuffer::from_samples(
            input.clone_data(),
            self.output_format.width,
            self.output_format.height,
            self.output_format.pixel_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::Compression,
            quality_loss: self.estimate_quality_loss(),
            processing_time: None,
        }
    }

    fn estimate_quality_loss(&self) -> Option<f32> {
        if self.input_format.is_lossless() && self.output_format.is_lossy() {
            Some(100.0)
        } else if self.input_format.is_lossy() && self.output_format.is_lossless() {
            Some(0.0)
        } else {
            Some(10.0)
        }
    }
}

struct PassthroughConverter {
    format: crate::image_format::ImageFormat,
}

impl PassthroughConverter {
    fn new(format: crate::image_format::ImageFormat) -> Self {
        Self {
            format,
        }
    }
}

impl ImageFormatConverter for PassthroughConverter {
    fn convert(&mut self, input: &crate::image_buffer::ImageBuffer) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        Ok(input.clone())
    }

    fn set_input_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::image_format::ImageFormat) {
        self.format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.format.clone())
    }

    fn get_output_format(&self) -> Option<crate::image_format::ImageFormat> {
        Some(self.format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.format.clone(),
            output_format: self.format.clone(),
            conversion_type: ConversionType::Complex,
            quality_loss: None,
            processing_time: None,
        }
    }
}

impl Default for ImageConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            current_pixel: 0,
            total_pixels: 0,
            current_row: 0,
            total_rows: 0,
            processing_speed: 0.0,
            eta: None,
        }
    }
}

impl Default for ConversionInfo {
    fn default() -> Self {
        Self {
            input_format: crate::image_format::ImageFormat::default(),
            output_format: crate::image_format::ImageFormat::default(),
            conversion_type: ConversionType::Complex,
            quality_loss: None,
            processing_time: None,
        }
    }
}

impl Default for ConversionStats {
    fn default() -> Self {
        Self {
            conversion_type: ConversionType::Complex,
            input_format: None,
            output_format: None,
            quality_loss: None,
            is_conversion_needed: false,
            estimated_processing_time: None,
        }
    }
}
