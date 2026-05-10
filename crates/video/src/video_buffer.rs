use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgb8,
    Rgba8,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
    Yuv420,
    Nv12,
    Nv21,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Pixel {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 255.0 }
    }

    pub fn gray(value: f32) -> Self {
        Self { r: value, g: value, b: value, a: 255.0 }
    }

    pub fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0] as f32,
            g: rgba[1] as f32,
            b: rgba[2] as f32,
            a: rgba[3] as f32,
        }
    }

    pub fn from_rgb8(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0] as f32,
            g: rgb[1] as f32,
            b: rgb[2] as f32,
            a: 255.0,
        }
    }

    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 255.0) as u8),
            (self.g.clamp(0.0, 255.0) as u8),
            (self.b.clamp(0.0, 255.0) as u8),
            (self.a.clamp(0.0, 255.0) as u8),
        ]
    }

    pub fn to_rgb8(&self) -> [u8; 3] {
        [
            (self.r.clamp(0.0, 255.0) as u8),
            (self.g.clamp(0.0, 255.0) as u8),
            (self.b.clamp(0.0, 255.0) as u8),
        ]
    }

    pub fn luma(&self) -> f32 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }

    pub fn luma_rec709(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    pub fn luma_rec601(&self) -> f32 {
        0.299 * self.r + 0.587 * self.g + 0.114 * self.b
    }

    pub fn chroma_u(&self) -> f32 {
        -0.14713 * self.r - 0.28886 * self.g + 0.436 * self.b
    }

    pub fn chroma_v(&self) -> f32 {
        0.615 * self.r - 0.51499 * self.g - 0.10001 * self.b
    }

    pub fn brightness(&self) -> f32 {
        (self.r + self.g + self.b) / 3.0
    }

    pub fn max(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    pub fn min(&self) -> f32 {
        self.r.min(self.g).min(self.b)
    }

    pub fn clamp(&self, min: f32, max: f32) -> Pixel {
        Pixel {
            r: self.r.clamp(min, max),
            g: self.g.clamp(min, max),
            b: self.b.clamp(min, max),
            a: self.a.clamp(min, max),
        }
    }

    pub fn add(&self, other: &Pixel) -> Pixel {
        Pixel {
            r: (self.r + other.r).clamp(0.0, 255.0),
            g: (self.g + other.g).clamp(0.0, 255.0),
            b: (self.b + other.b).clamp(0.0, 255.0),
            a: (self.a + other.a).clamp(0.0, 255.0),
        }
    }

    pub fn subtract(&self, other: &Pixel) -> Pixel {
        Pixel {
            r: (self.r - other.r).clamp(0.0, 255.0),
            g: (self.g - other.g).clamp(0.0, 255.0),
            b: (self.b - other.b).clamp(0.0, 255.0),
            a: (self.a - other.a).clamp(0.0, 255.0),
        }
    }

    pub fn multiply(&self, other: &Pixel) -> Pixel {
        Pixel {
            r: (self.r * other.r / 255.0).clamp(0.0, 255.0),
            g: (self.g * other.g / 255.0).clamp(0.0, 255.0),
            b: (self.b * other.b / 255.0).clamp(0.0, 255.0),
            a: (self.a * other.a / 255.0).clamp(0.0, 255.0),
        }
    }

    pub fn blend(&self, other: &Pixel, factor: f32) -> Pixel {
        let inv_factor = 1.0 - factor;
        Pixel {
            r: self.r * inv_factor + other.r * factor,
            g: self.g * inv_factor + other.g * factor,
            b: self.b * inv_factor + other.b * factor,
            a: self.a * inv_factor + other.a * factor,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub data: Arc<RwLock<Vec<Pixel>>>,
    pub timestamp: Option<std::time::Duration>,
    pub frame_number: u32,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            pixel_format,
            data: Arc::new(RwLock::new(vec![Pixel::default(); pixel_count])),
            timestamp: None,
            frame_number: 0,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let data = self.data.read();
        let index = (y * self.width + x) as usize;
        data.get(index).copied()
    }

    pub fn set_pixel(&self, x: u32, y: u32, pixel: Pixel) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        let mut data = self.data.write();
        let index = (y * self.width + x) as usize;
        
        if let Some(p) = data.get_mut(index) {
            *p = pixel;
            true
        } else {
            false
        }
    }

    pub fn get_pixel_safe(&self, x: i32, y: i32) -> Pixel {
        let clamped_x = x.clamp(0, self.width as i32 - 1) as u32;
        let clamped_y = y.clamp(0, self.height as i32 - 1) as u32;
        self.get_pixel(clamped_x, clamped_y).unwrap_or(Pixel::default())
    }

    pub fn set_pixel_safe(&self, x: i32, y: i32, pixel: Pixel) {
        let clamped_x = x.clamp(0, self.width as i32 - 1) as u32;
        let clamped_y = y.clamp(0, self.height as i32 - 1) as u32;
        self.set_pixel(clamped_x, clamped_y, pixel);
    }

    pub fn get_region(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Vec<Pixel>> {
        if x + width > self.width || y + height > self.height {
            return None;
        }

        let data = self.data.read();
        let mut region = Vec::with_capacity((width * height) as usize);

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                let index = (src_y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get(index) {
                    region.push(pixel);
                }
            }
        }

        Some(region)
    }

    pub fn set_region(&self, x: u32, y: u32, width: u32, height: u32, region: &[Pixel]) -> bool {
        if x + width > self.width || y + height > self.height || region.len() != (width * height) as usize {
            return false;
        }

        let mut data = self.data.write();

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                let index = (src_y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get_mut(index) {
                    *pixel = region[dy as usize * width as usize + dx as usize];
                }
            }
        }

        true
    }

    pub fn fill(&self, pixel: Pixel) {
        let mut data = self.data.write();
        for p in data.iter_mut() {
            *p = pixel;
        }
    }

    pub fn fill_region(&self, x: u32, y: u32, width: u32, height: u32, pixel: Pixel) -> bool {
        if x + width > self.width || y + height > self.height {
            return false;
        }

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                self.set_pixel(src_x, src_y, pixel);
            }
        }

        true
    }

    pub fn clear(&self) {
        self.fill(Pixel::default());
    }

    pub fn clone(&self) -> VideoFrame {
        let data = self.data.read();
        VideoFrame {
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(data.clone())),
            timestamp: self.timestamp,
            frame_number: self.frame_number,
        }
    }

    pub fn resize(&self, new_width: u32, new_height: u32) -> VideoFrame {
        let pixel_count = (new_width * new_height) as usize;
        VideoFrame {
            width: new_width,
            height: new_height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(vec![Pixel::default(); pixel_count])),
            timestamp: self.timestamp,
            frame_number: self.frame_number,
        }
    }

    pub fn convert_format(&self, new_format: PixelFormat) -> VideoFrame {
        let data = self.data.read();
        let mut converted_data = Vec::with_capacity(data.len());

        for pixel in data.iter() {
            let converted_pixel = match (&self.pixel_format, &new_format) {
                (PixelFormat::Rgb8, PixelFormat::Rgba8) => {
                    Pixel::new(pixel.r, pixel.g, pixel.b, 255.0)
                },
                (PixelFormat::Rgba8, PixelFormat::Rgb8) => {
                    Pixel::new(pixel.r, pixel.g, pixel.b, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Rgb16) => {
                    Pixel::new(pixel.r * 257.0, pixel.g * 257.0, pixel.b * 257.0, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Rgb32F) => {
                    Pixel::new(pixel.r, pixel.g, pixel.b, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Yuv420) => {
Simple RGB to YUV420 conversion
                    let y = pixel.luma_rec709();
                    let u = pixel.chroma_u();
                    let v = pixel.chroma_v();
                    Pixel::new(y, u, v, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Nv12) => {
                    let y = pixel.luma_rec709();
                    let u = pixel.chroma_u();
                    let v = pixel.chroma_v();
                    Pixel::new(y, u, v, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Nv21) => {
                    let y = pixel.luma_rec709();
                    let u = pixel.chroma_u();
                    let v = pixel.chroma_v();
                    Pixel::new(y, u, v, pixel.a)
                },
                _ => {
                    *pixel
                },
            };
            converted_data.push(converted_pixel);
        }

        VideoFrame {
            width: self.width,
            height: self.height,
            pixel_format: new_format,
            data: Arc::new(RwLock::new(converted_data)),
            timestamp: self.timestamp,
            frame_number: self.frame_number,
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn get_byte_size(&self) -> usize {
        let bytes_per_pixel = match self.pixel_format {
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8 => 4,
            PixelFormat::Rgb16 => 6,
            PixelFormat::Rgba16 => 8,
            PixelFormat::Rgb32F => 12,
            PixelFormat::Rgba32F => 16,
            PixelFormat::Yuv420 => 1,
            PixelFormat::Nv12 => 1,
            PixelFormat::Nv21 => 1,
        };
        
        self.get_pixel_count() * bytes_per_pixel
    }

    pub fn get_data_slice(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Vec<Pixel>> {
        if x + width > self.width || y + height > self.height {
            return None;
        }

        let data = self.data.read();
        let mut slice = Vec::with_capacity((width * height) as usize);

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                let index = (src_y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get(index) {
                    slice.push(pixel);
                }
            }
        }

        Some(slice)
    }

    pub fn clone_data(&self) -> Vec<Pixel> {
        self.data.read().clone()
    }

    pub fn get_timestamp(&self) -> Option<std::time::Duration> {
        self.timestamp
    }

    pub fn set_timestamp(&self, timestamp: std::time::Duration) {
        let _ = timestamp;
    }

    pub fn get_frame_number(&self) -> u32 {
        self.frame_number
    }

    pub fn set_frame_number(&self, frame_number: u32) {
        let _ = frame_number;
    }

    pub fn get_stats(&self) -> FrameStats {
        let data = self.data.read();
        
        let mut min_r = f32::MAX;
        let mut max_r = f32::MIN;
        let mut min_g = f32::MAX;
        let mut max_g = f32::MIN;
        let mut min_b = f32::MAX;
        let mut max_b = f32::MIN;
        let mut min_a = f32::MAX;
        let mut max_a = f32::MIN;
        
        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;
        let mut sum_a = 0.0;
        
        for pixel in data.iter() {
            min_r = min_r.min(pixel.r);
            max_r = max_r.max(pixel.r);
            min_g = min_g.min(pixel.g);
            max_g = max_g.max(pixel.g);
            min_b = min_b.min(pixel.b);
            max_b = max_b.max(pixel.b);
            min_a = min_a.min(pixel.a);
            max_a = max_a.max(pixel.a);
            
            sum_r += pixel.r;
            sum_g += pixel.g;
            sum_b += pixel.b;
            sum_a += pixel.a;
        }
        
        let count = data.len() as f32;
        
        FrameStats {
            width: self.width,
            height: self.height,
            pixel_count: self.get_pixel_count(),
            pixel_format: self.pixel_format.clone(),
            min_color: Pixel::new(min_r, min_g, min_b, min_a),
            max_color: Pixel::new(max_r, max_g, max_b, max_a),
            average_color: if count > 0.0 {
                Pixel::new(sum_r / count, sum_g / count, sum_b / count, sum_a / count)
            } else {
                Pixel::default()
            },
        }
    }

    pub fn apply_alpha(&self, alpha: f32) -> VideoFrame {
        let data = self.data.read();
        let mut alpha_data = Vec::with_capacity(data.len());

        for pixel in data.iter() {
            let mut alpha_pixel = *pixel;
            alpha_pixel.a = pixel.a * alpha.clamp(0.0, 1.0);
            alpha_data.push(alpha_pixel);
        }

        VideoFrame {
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(alpha_data)),
            timestamp: self.timestamp,
            frame_number: self.frame_number,
        }
    }

    pub fn to_image_buffer(&self) -> crate::image_buffer::ImageBuffer {
        let data = self.data.read();
        crate::image_buffer::ImageBuffer::from_samples(
            data,
            self.width,
            self.height,
            self.pixel_format.clone(),
        )
    }

    pub fn from_image_buffer(image: &crate::image_buffer::ImageBuffer) -> VideoFrame {
        let data = image.clone_data();
        let pixel_format = match image.pixel_format {
            crate::image_buffer::PixelFormat::Rgb8 => PixelFormat::Rgb8,
            crate::image_buffer::PixelFormat::Rgba8 => PixelFormat::Rgba8,
            crate::image_buffer::PixelFormat::Rgb16 => PixelFormat::Rgb16,
            crate::image_buffer::PixelFormat::Rgba16 => PixelFormat::Rgba16,
            crate::image_buffer::PixelFormat::Rgb32F => PixelFormat::Rgb32F,
            crate::image_buffer::PixelFormat::Rgba32F => PixelFormat::Rgba32F,
            crate::image_buffer::PixelFormat::Grayscale8 => PixelFormat::Rgb8,
            _ => PixelFormat::Rgb8,
        };

        VideoFrame {
            width: image.width,
            height: image.height,
            pixel_format,
            data: Arc::new(RwLock::new(data)),
            timestamp: None,
            frame_number: 0,
        }
    }

    pub fn to_yuv420(&self) -> VideoFrame {
        self.convert_format(PixelFormat::Yuv420)
    }

    pub fn to_nv12(&self) -> VideoFrame {
        self.convert_format(PixelFormat::Nv12)
    }

    pub fn to_nv21(&self) -> VideoFrame {
        self.convert_format(PixelFormat::Nv21)
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub frames: Arc<RwLock<Vec<VideoFrame>>>,
    pub frame_rate: f32,
    pub duration: Option<std::time::Duration>,
    pub metadata: Arc<RwLock<VideoMetadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub description: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub creation_time: Option<std::time::SystemTime>,
    pub bitrate: Option<u32>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub pixel_aspect_ratio: Option<String>,
    pub color_space: Option<String>,
    pub language: Option<String>,
    pub chapters: Vec<Chapter>,
    pub tags: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: u32,
    pub title: String,
    pub start_time: std::time::Duration,
    pub end_time: std::time::Duration,
    pub description: Option<String>,
}

impl Buffer {
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat, frame_rate: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            width,
            height,
            pixel_format,
            frames: Arc::new(RwLock::new(Vec::new())),
            frame_rate,
            duration: None,
            metadata: Arc::new(RwLock::new(VideoMetadata::default())),
        }
    }

    pub fn add_frame(&self, frame: VideoFrame) -> bool {
        if frame.width != self.width || frame.height != self.height {
            return false;
        }

        let mut frames = self.frames.write();
        frames.push(frame);
        true
    }

    pub fn get_frame(&self, index: usize) -> Option<VideoFrame> {
        let frames = self.frames.read();
        frames.get(index).cloned()
    }

    pub fn get_frame_count(&self) -> usize {
        self.frames.read().len()
    }

    pub fn get_last_frame(&self) -> Option<VideoFrame> {
        let frames = self.frames.read();
        frames.last().cloned()
    }

    pub fn remove_frame(&self, index: usize) -> Option<VideoFrame> {
        let mut frames = self.frames.write();
        if index < frames.len() {
            Some(frames.remove(index))
        } else {
            None
        }
    }

    pub fn clear_frames(&self) {
        let mut frames = self.frames.write();
        frames.clear();
    }

    pub fn resize(&self, new_width: u32, new_height: u32) -> Buffer {
        let mut frames = self.frames.write();
        let mut resized_frames = Vec::with_capacity(frames.len());

        for frame in frames.iter() {
            let resized_frame = frame.resize(new_width, new_height);
            resized_frames.push(resized_frame);
        }

        Buffer {
            id: self.id.clone(),
            width: new_width,
            height: new_height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(resized_frames)),
            frame_rate: self.frame_rate,
            duration: self.duration,
            metadata: self.metadata.clone(),
        }
    }

    pub fn convert_format(&self, new_format: PixelFormat) -> Buffer {
        let frames = self.frames.read();
        let mut converted_frames = Vec::with_capacity(frames.len());

        for frame in frames.iter() {
            let converted_frame = frame.convert_format(new_format);
            converted_frames.push(converted_frame);
        }

        Buffer {
            id: self.id.clone(),
            width: self.width,
            height: self.height,
            pixel_format: new_format,
            frames: Arc::new(RwLock::new(converted_frames))),
            frame_rate: self.frame_rate,
            duration: self.duration,
            metadata: self.metadata.clone(),
        }
    }

    pub fn get_duration(&self) -> Option<std::time::Duration> {
        if let Some(duration) = self.duration {
            Some(duration)
        } else if self.frame_count > 0 {
            Some(std::time::Duration::from_secs_f64(self.frame_count as f64 / self.frame_rate as f64))
        } else {
            None
        }
    }

    pub fn set_duration(&self, duration: std::time::Duration) {
        let _ = duration;
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_pixel_format(&self) -> PixelFormat {
        self.pixel_format.clone()
    }

    pub fn get_frame_rate(&self) -> f32 {
        self.frame_rate
    }

    pub fn get_metadata(&self) -> VideoMetadata {
        self.metadata.read().clone()
    }

    pub fn set_metadata(&self, metadata: VideoMetadata) {
        let mut current_metadata = self.metadata.write();
        *current_metadata = metadata;
    }

    pub fn get_stats(&self) -> BufferStats {
        let frames = self.frames.read();
        BufferStats {
            width: self.width,
            height: self.height,
            frame_count: frames.len(),
            frame_rate: self.frame_rate,
            pixel_format: self.pixel_format.clone(),
            duration: self.get_duration(),
            metadata: self.get_metadata(),
        }
    }

    pub fn clone_buffer(&self) -> Buffer {
        let frames = self.frames.read();
        Buffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(frames.clone())),
            frame_rate: self.frame_rate,
            duration: self.duration,
            metadata: self.metadata.clone(),
        }
    }

    pub fn extract_audio(&self) -> Option<Arc<tiffiny_audio::audio_buffer::AudioBuffer>> {
        None
    }

    pub fn extract_keyframes(&self, interval: u32) -> Vec<VideoFrame> {
        let frames = self.frames.read();
        let mut keyframes = Vec::new();

        for (index, frame) in frames.iter().enumerate() {
            if index % interval as usize == 0 {
                keyframes.push(frame.clone());
            }
        }

        keyframes
    }

    pub fn create_thumbnail(&self, width: u32, height: u32) -> Option<VideoFrame> {
        if let Some(first_frame) = self.get_frame(0) {
            Some(first_frame.resize(width, height))
        } else {
            None
        }
    }

    pub fn create_preview(&self, start_time: std::time::Duration, duration: std::time::Duration) -> Buffer {
        let frames = self.frames.read();
        let start_frame = (start_time.as_secs_f64() * self.frame_rate) as usize;
        let end_frame = ((start_time + duration).as_secs_f64() * self.frame_rate) as usize).min(frames.len());

        let mut preview_frames = Vec::new();
        for index in start_frame..end_frame {
            if let Some(frame) = frames.get(index) {
                preview_frames.push(frame.clone());
            }
        }

        Buffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(preview_frames)),
            frame_rate: self.frame_rate,
            duration: Some(duration),
            metadata: self.metadata.clone(),
        }
    }

    pub fn slice(&self, start_frame: usize, end_frame: usize) -> Buffer {
        let frames = self.frames.read();
        let mut sliced_frames = Vec::new();

        for index in start_frame..end_frame.min(frames.len()) {
            if let Some(frame) = frames.get(index) {
                sliced_frames.push(frame.clone());
            }
        }

        Buffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(sliced_frames))),
            frame_rate: self.frame_rate,
            duration: None,
            metadata: self.metadata.clone(),
        }
    }

    pub fn concatenate(&self, other: &Buffer) -> Result<Buffer, Box<dyn std::error::Error>> {
        if self.width != other.width || self.height != other.height || self.pixel_format != other.pixel_format {
            return Err("Incompatible video buffers".into());
        }

        let self_frames = self.frames.read();
        let other_frames = other.frames.read();
        let mut combined_frames = Vec::with_capacity(self_frames.len() + other_frames.len());

        for frame in self_frames.iter() {
            combined_frames.push(frame.clone());
        }

        for frame in other_frames.iter() {
            combined_frames.push(frame.clone());
        }

        Ok(Buffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(combined_frames))),
            frame_rate: self.frame_rate,
            duration: None,
            metadata: self.metadata.clone(),
        })
    }

    pub fn blend(&self, other: &Buffer, factor: f32) -> Result<Buffer, Box<dyn std::error::Error>> {
        if self.width != other.width || self.height != other.height {
            return Err("Incompatible video buffers".into());
        }

        let self_frames = self.frames.read();
        let other_frames = other.frames.read();
        let min_frames = self_frames.len().min(other_frames.len());
        let mut blended_frames = Vec::with_capacity(min_frames);

        for i in 0..min_frames {
            if let (Some(self_frame), Some(other_frame)) = (self_frames.get(i), other_frames.get(i)) {
                let blended_frame = VideoFrame::new(
                    self.width,
                    self.height,
                    self.pixel_format,
                );
                
                for y in 0..self.height {
                    for x in 0..self.width {
                        if let (Some(self_pixel), Some(other_pixel)) = (self_frame.get_pixel(x, y), other_frame.get_pixel(x, y)) {
                            let blended_pixel = self_pixel.blend(other_pixel, factor);
                            blended_frame.set_pixel(x, y, blended_pixel);
                        }
                    }
                }
                
                blended_frames.push(blended_frame);
            }
        }

        Ok(Buffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            frames: Arc::new(RwLock::new(blended_frames))),
            frame_rate: self.frame_rate,
            duration: None,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BufferStats {
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub frame_rate: f32,
    pub pixel_format: PixelFormat,
    pub duration: Option<std::time::Duration>,
    pub metadata: VideoMetadata,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 255.0,
        }
    }
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Rgba8
    }
}

impl Default for VideoFrame {
    fn default() -> Self {
        Self::new(1, 1, PixelFormat::Rgba8)
    }
}

impl Default for VideoMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            copyright: None,
            description: None,
            duration: None,
            creation_time: None,
            bitrate: None,
            codec: None,
            container: None,
            width: 0,
            height: 0,
            frame_rate: 0.0,
            pixel_aspect_ratio: None,
            color_space: None,
            language: None,
            chapters: Vec::new(),
            tags: std::collections::HashMap::new(),
        }
    }
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            start_time: std::time::Duration::from_secs(0),
            end_time: std::time::Duration::from_secs(0),
            description: None,
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new(1920, 1080, PixelFormat::Rgba8, 30.0)
    }
}

impl Default for BufferStats {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_count: 0,
            frame_rate: 30.0,
            pixel_format: PixelFormat::Rgba8,
            duration: None,
            metadata: VideoMetadata::default(),
        }
    }
}
