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
    Grayscale8,
    Grayscale16,
    Grayscale32F,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn from_rgba16(rgba: [u16; 4]) -> Self {
        Self {
            r: rgba[0] as f32 / 65535.0,
            g: rgba[1] as f32 / 65535.0,
            b: rgba[2] as f32 / 65535.0,
            a: rgba[3] as f32 / 65535.0,
        }
    }

    pub fn from_rgb16(rgb: [u16; 3]) -> Self {
        Self {
            r: rgb[0] as f32 / 65535.0,
            g: rgb[1] as f32 / 65535.0,
            b: rgb[2] as f32 / 65535.0,
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

    pub fn to_rgba16(&self) -> [u16; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 65535.0) as u16,
            (self.g.clamp(0.0, 1.0) * 65535.0) as u16,
            (self.b.clamp(0.0, 1.0) * 65535.0) as u16,
            (self.a.clamp(0.0, 1.0) * 65535.0) as u16,
        ]
    }

    pub fn to_rgb16(&self) -> [u16; 3] {
        [
            (self.r.clamp(0.0, 1.0) * 65535.0) as u16,
            (self.g.clamp(0.0, 1.0) * 65535.0) as u16,
            (self.b.clamp(0.0, 1.0) * 65535.0) as u16,
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
pub struct ImageBuffer {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub data: Arc<RwLock<Vec<Pixel>>>,
    pub metadata: Arc<RwLock<ImageMetadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub color_space: String,
    pub gamma: f32,
    pub dpi: Option<(f32, f32)>,
    pub exif: Option<std::collections::HashMap<String, String>>,
    pub icc_profile: Option<Vec<u8>>,
}

impl ImageBuffer {
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        let data_size = (width * height) as usize;
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            width,
            height,
            pixel_format,
            data: Arc::new(RwLock::new(vec![Pixel::new(0.0, 0.0, 0.0, 255.0); data_size])),
            metadata: Arc::new(RwLock::new(ImageMetadata {
                width,
                height,
                pixel_format: pixel_format.clone(),
                color_space: "sRGB".to_string(),
                gamma: 2.2,
                dpi: None,
                exif: None,
                icc_profile: None,
            })),
        }
    }

    pub fn from_image(image: &image::DynamicImage) -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = (image.width(), image.height());
        let pixel_format = match image {
            image::DynamicImage::ImageLuma8(_) => PixelFormat::Grayscale8,
            image::DynamicImage::ImageLuma16(_) => PixelFormat::Grayscale16,
            image::DynamicImage::ImageRgb8(_) => PixelFormat::Rgb8,
            image::DynamicImage::ImageRgba8(_) => PixelFormat::Rgba8,
            image::DynamicImage::ImageRgb16(_) => PixelFormat::Rgb16,
            image::DynamicImage::ImageRgba16(_) => PixelFormat::Rgba16,
            image::DynamicImage::ImageRgb32F(_) => PixelFormat::Rgb32F,
            image::DynamicImage::ImageRgba32F(_) => PixelFormat::Rgba32F,
        };

        let data = match image {
            image::DynamicImage::ImageLuma8(img) => {
                img.pixels()
                    .map(|p| Pixel::gray(p[0] as f32))
                    .collect()
            },
            image::DynamicImage::ImageLuma16(img) => {
                img.pixels()
                    .map(|p| Pixel::gray(p[0] as f32 / 65535.0))
                    .collect()
            },
            image::DynamicImage::ImageRgb8(img) => {
                img.pixels()
                    .map(|p| Pixel::from_rgb8(p.0))
                    .collect()
            },
            image::DynamicImage::ImageRgba8(img) => {
                img.pixels()
                    .map(|p| Pixel::from_rgba8(p.0))
                    .collect()
            },
            image::DynamicImage::ImageRgb16(img) => {
                img.pixels()
                    .map(|p| Pixel::from_rgb16(p.0))
                    .collect()
            },
            image::DynamicImage::ImageRgba16(img) => {
                img.pixels()
                    .map(|p| Pixel::from_rgba16(p.0))
                    .collect()
            },
            image::DynamicImage::ImageRgb32F(img) => {
                img.pixels()
                    .map(|p| Pixel::rgb(p[0], p[1], p[2]))
                    .collect()
            },
            image::DynamicImage::ImageRgba32F(img) => {
                img.pixels()
                    .map(|p| Pixel::new(p[0], p[1], p[2], p[3]))
                    .collect()
            },
        };

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            width,
            height,
            pixel_format,
            data: Arc::new(RwLock::new(data)),
            metadata: Arc::new(RwLock::new(ImageMetadata {
                width,
                height,
                pixel_format: pixel_format.clone(),
                color_space: "sRGB".to_string(),
                gamma: 2.2,
                dpi: None,
                exif: None,
                icc_profile: None,
            })),
        })
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
        self.get_pixel(clamped_x, clamped_y).unwrap_or(Pixel::new(0.0, 0.0, 0.0, 255.0))
    }

    pub fn set_pixel_safe(&self, x: i32, y: i32, pixel: Pixel) {
        let clamped_x = x.clamp(0, self.width as i32 - 1) as u32;
        let clamped_y = y.clamp(0, self.height as i32 - 1) as u32;
        self.set_pixel(clamped_x, clamped_y, pixel);
    }

    pub fn get_region(&self, x: u32, y: u32, width: u32, height: u32) -> Option<ImageBuffer> {
        if x + width > self.width || y + height > self.height {
            return None;
        }

        let data = self.data.read();
        let mut region_data = Vec::with_capacity((width * height) as usize);

        for dy in 0..height {
            for dx in 0..width {
                let src_x = x + dx;
                let src_y = y + dy;
                let src_index = (src_y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get(src_index) {
                    region_data.push(*pixel);
                }
            }
        }

        Some(ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width,
            height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(region_data)),
            metadata: self.metadata.clone(),
        })
    }

    pub fn set_region(&self, x: u32, y: u32, width: u32, height: u32, source: &ImageBuffer) -> bool {
        if x + width > self.width || y + height > self.height ||
           source.width != width || source.height != height {
            return false;
        }

        let mut dest_data = self.data.write();
        let source_data = source.data.read();

        for dy in 0..height {
            for dx in 0..width {
                let dest_x = x + dx;
                let dest_y = y + dy;
                let src_index = (dy * source.width + dx) as usize;
                let dest_index = (dest_y * self.width + dest_x) as usize;
                
                if let (Some(src_pixel), Some(dest_pixel)) = (source_data.get(src_index), dest_data.get_mut(dest_index)) {
                    *dest_pixel = *src_pixel;
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
                self.set_pixel(x + dx, y + dy, pixel);
            }
        }

        true
    }

    pub fn clear(&self) {
        self.fill(Pixel::new(0.0, 0.0, 0.0, 0.0));
    }

    pub fn clone(&self) -> ImageBuffer {
        let data = self.data.read();
        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(data.clone())),
            metadata: self.metadata.clone(),
        }
    }

    pub fn resize(&self, new_width: u32, new_height: u32) -> ImageBuffer {
        let data = self.data.read();
        let mut resized_data = Vec::with_capacity((new_width * new_height) as usize);

        let x_ratio = self.width as f32 / new_width as f32;
        let y_ratio = self.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * x_ratio) as u32;
                let src_y = (y as f32 * y_ratio) as u32;
                
                if let Some(pixel) = self.get_pixel(src_x.min(self.width - 1), src_y.min(self.height - 1)) {
                    resized_data.push(pixel);
                } else {
                    resized_data.push(Pixel::new(0.0, 0.0, 0.0, 255.0));
                }
            }
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: new_width,
            height: new_height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(resized_data)),
            metadata: Arc::new(RwLock::new(ImageMetadata {
                width: new_width,
                height: new_height,
                pixel_format: self.pixel_format.clone(),
                color_space: "sRGB".to_string(),
                gamma: 2.2,
                dpi: None,
                exif: None,
                icc_profile: None,
            })),
        }
    }

    pub fn flip_horizontal(&self) -> ImageBuffer {
        let data = self.data.read();
        let mut flipped_data = Vec::with_capacity(data.len());

        for y in 0..self.height {
            for x in 0..self.width {
                let src_x = self.width - 1 - x;
                let src_index = (y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get(src_index) {
                    flipped_data.push(*pixel);
                }
            }
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(flipped_data)),
            metadata: self.metadata.clone(),
        }
    }

    pub fn flip_vertical(&self) -> ImageBuffer {
        let data = self.data.read();
        let mut flipped_data = Vec::with_capacity(data.len());

        for y in 0..self.height {
            let src_y = self.height - 1 - y;
            for x in 0..self.width {
                let src_index = (src_y * self.width + x) as usize;
                
                if let Some(pixel) = data.get(src_index) {
                    flipped_data.push(*pixel);
                }
            }
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(flipped_data)),
            metadata: self.metadata.clone(),
        }
    }

    pub fn rotate_90(&self) -> ImageBuffer {
        let data = self.data.read();
        let mut rotated_data = Vec::with_capacity(data.len());

        for x in 0..self.width {
            for y in (0..self.height).rev() {
                let src_index = (y * self.width + x) as usize;
                
                if let Some(pixel) = data.get(src_index) {
                    rotated_data.push(*pixel);
                }
            }
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.height,
            height: self.width,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(rotated_data)),
            metadata: Arc::new(RwLock::new(ImageMetadata {
                width: self.height,
                height: self.width,
                pixel_format: self.pixel_format.clone(),
                color_space: "sRGB".to_string(),
                gamma: 2.2,
                dpi: None,
                exif: None,
                icc_profile: None,
            })),
        }
    }

    pub fn to_image_buffer(&self) -> image::DynamicImage {
        let data = self.data.read();
        
        match self.pixel_format {
            PixelFormat::Grayscale8 => {
                let gray_data: Vec<u8> = data.iter().map(|p| p.r.clamp(0.0, 255.0) as u8).collect();
                image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(
                    self.width,
                    self.height,
                    gray_data,
                ).unwrap())
            },
            PixelFormat::Grayscale16 => {
                let gray_data: Vec<u16> = data.iter().map(|p| (p.r.clamp(0.0, 1.0) * 65535.0) as u16).collect();
                image::DynamicImage::ImageLuma16(image::GrayImage::from_raw(
                    self.width,
                    self.height,
                    gray_data,
                ).unwrap())
            },
            PixelFormat::Rgb8 => {
                let rgb_data: Vec<u8> = data.iter().flat_map(|p| {
                    let rgba = p.to_rgba8();
                    Some(rgba[0]).into_iter().chain(Some(rgba[1])).chain(Some(rgba[2]))
                }).collect();
                image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(
                    self.width,
                    self.height,
                    rgb_data,
                ).unwrap())
            },
            PixelFormat::Rgba8 => {
                let rgba_data: Vec<u8> = data.iter().flat_map(|p| {
                    let rgba = p.to_rgba8();
                    Some(rgba[0]).into_iter().chain(Some(rgba[1])).chain(Some(rgba[2])).chain(Some(rgba[3]))
                }).collect();
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
                    self.width,
                    self.height,
                    rgba_data,
                ).unwrap())
            },
            PixelFormat::Rgb16 => {
                let rgb_data: Vec<u16> = data.iter().flat_map(|p| {
                    let rgba = p.to_rgba16();
                    Some(rgba[0]).into_iter().chain(Some(rgba[1])).chain(Some(rgba[2]))
                }).collect();
                image::DynamicImage::ImageRgb16(image::RgbImage::from_raw(
                    self.width,
                    self.height,
                    rgb_data,
                ).unwrap())
            },
            PixelFormat::Rgba16 => {
                let rgba_data: Vec<u16> = data.iter().flat_map(|p| {
                    let rgba = p.to_rgba16();
                    Some(rgba[0]).into_iter().chain(Some(rgba[1])).chain(Some(rgba[2])).chain(Some(rgba[3]))
                }).collect();
                image::DynamicImage::ImageRgba16(image::RgbaImage::from_raw(
                    self.width,
                    self.height,
                    rgba_data,
                ).unwrap())
            },
            PixelFormat::Rgb32F => {
                let rgb_data: Vec<f32> = data.iter().flat_map(|p| {
                    Some(p.r).into_iter().chain(Some(p.g)).chain(Some(p.b))
                }).collect();
                image::DynamicImage::ImageRgb32F(image::RgbImage::from_raw(
                    self.width,
                    self.height,
                    rgb_data,
                ).unwrap())
            },
            PixelFormat::Rgba32F => {
                let rgba_data: Vec<f32> = data.iter().flat_map(|p| {
                    Some(p.r).into_iter().chain(Some(p.g)).chain(Some(p.b)).chain(Some(p.a))
                }).collect();
                image::DynamicImage::ImageRgba32F(image::RgbaImage::from_raw(
                    self.width,
                    self.height,
                    rgba_data,
                ).unwrap())
            },
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
            PixelFormat::Grayscale8 => 1,
            PixelFormat::Grayscale16 => 2,
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8 => 4,
            PixelFormat::Rgb16 => 6,
            PixelFormat::Rgba16 => 8,
            PixelFormat::Rgb32F => 12,
            PixelFormat::Rgba32F => 16,
        };
        
        self.get_pixel_count() * bytes_per_pixel
    }

    pub fn get_metadata(&self) -> ImageMetadata {
        self.metadata.read().clone()
    }

    pub fn set_metadata(&self, metadata: ImageMetadata) {
        let mut meta = self.metadata.write();
        *meta = metadata;
    }

    pub fn get_average_color(&self) -> Pixel {
        let data = self.data.read();
        let count = data.len() as f32;
        
        if count == 0.0 {
            return Pixel::new(0.0, 0.0, 0.0, 255.0);
        }

        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;
        let mut sum_a = 0.0;

        for pixel in data.iter() {
            sum_r += pixel.r;
            sum_g += pixel.g;
            sum_b += pixel.b;
            sum_a += pixel.a;
        }

        Pixel::new(
            sum_r / count,
            sum_g / count,
            sum_b / count,
            sum_a / count,
        )
    }

    pub fn get_bounds(&self) -> (u32, u32, u32, u32) {
        let data = self.data.read();
        
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0;
        let mut max_y = 0;

        for (y, pixel) in data.iter().enumerate() {
            let y_coord = (y as u32) / self.width;
            let x_coord = (y as u32) % self.width;
            
            if pixel.a > 0.0 {
                min_x = min_x.min(x_coord);
                min_y = min_y.min(y_coord);
                max_x = max_x.max(x_coord);
                max_y = max_y.max(y_coord);
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    pub fn crop_bounds(&self) -> ImageBuffer {
        let (min_x, min_y, max_x, max_y) = self.get_bounds();
        
        if min_x >= max_x || min_y >= max_y {
            return self.clone();
        }

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        
        let mut cropped = ImageBuffer::new(width, height, self.pixel_format.clone());
        
        for y in 0..height {
            for x in 0..width {
                if let Some(pixel) = self.get_pixel(min_x + x, min_y + y) {
                    cropped.set_pixel(x, y, pixel);
                }
            }
        }

        cropped
    }

    pub fn apply_alpha(&self, alpha: f32) -> ImageBuffer {
        let data = self.data.read();
        let mut alpha_data = Vec::with_capacity(data.len());

        for pixel in data.iter() {
            alpha_data.push(Pixel {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
                a: pixel.a * alpha.clamp(0.0, 1.0),
            });
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: self.pixel_format.clone(),
            data: Arc::new(RwLock::new(alpha_data)),
            metadata: self.metadata.clone(),
        }
    }

    pub fn convert_format(&self, new_format: PixelFormat) -> ImageBuffer {
        if self.pixel_format == new_format {
            return self.clone();
        }

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
                (PixelFormat::Rgb8, PixelFormat::Grayscale8) => {
                    Pixel::gray(pixel.luma())
                },
                (PixelFormat::Grayscale8, PixelFormat::Rgb8) => {
                    Pixel::gray(pixel.r)Use red channel for grayscale
                },
                (PixelFormat::Rgb8, PixelFormat::Rgb16) => {
                    Pixel::new(pixel.r * 257.0, pixel.g * 257.0, pixel.b * 257.0, pixel.a)
                },
                (PixelFormat::Rgb16, PixelFormat::Rgb8) => {
                    Pixel::new(pixel.r / 257.0, pixel.g / 257.0, pixel.b / 257.0, pixel.a)
                },
                (PixelFormat::Rgb8, PixelFormat::Rgb32F) => {
                    Pixel::new(pixel.r, pixel.g, pixel.b, pixel.a)
                },
                (PixelFormat::Rgb32F, PixelFormat::Rgb8) => {
                    Pixel::new(pixel.r.clamp(0.0, 255.0), pixel.g.clamp(0.0, 255.0), pixel.b.clamp(0.0, 255.0), pixel.a)
                },
                _ => pixel.clone(),
            };
            converted_data.push(converted_pixel);
        }

        ImageBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            width: self.width,
            height: self.height,
            pixel_format: new_format,
            data: Arc::new(RwLock::new(converted_data)),
            metadata: Arc::new(RwLock::new(ImageMetadata {
                width: self.width,
                height: self.height,
                pixel_format: new_format,
                color_space: "sRGB".to_string(),
                gamma: 2.2,
                dpi: None,
                exif: None,
                icc_profile: None,
            })),
        }
    }

    pub fn clone_data(&self) -> Vec<Pixel> {
        self.data.read().clone()
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
                let src_index = (src_y * self.width + src_x) as usize;
                
                if let Some(pixel) = data.get(src_index) {
                    slice.push(*pixel);
                }
            }
        }

        Some(slice)
    }

    pub fn get_stats(&self) -> ImageStats {
        let data = self.data.read();
        let mut min_r = f32::MAX;
        let mut max_r = f32::MIN;
        let mut min_g = f32::MAX;
        let mut max_g = f32::MIN;
        let mut min_b = f32::MAX;
        let mut max_b = f32::MIN;
        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;
        let mut count = 0;

        for pixel in data.iter() {
            if pixel.a > 0.0 {
                min_r = min_r.min(pixel.r);
                max_r = max_r.max(pixel.r);
                min_g = min_g.min(pixel.g);
                max_g = max_g.max(pixel.g);
                min_b = min_b.min(pixel.b);
                max_b = max_b.max(pixel.b);
                
                sum_r += pixel.r;
                sum_g += pixel.g;
                sum_b += pixel.b;
                count += 1;
            }
        }

        ImageStats {
            width: self.width,
            height: self.height,
            pixel_count: self.get_pixel_count(),
            pixel_format: self.pixel_format.clone(),
            min_color: Pixel::new(min_r, min_g, min_b, 0.0),
            max_color: Pixel::new(max_r, max_g, max_b, 0.0),
            average_color: if count > 0 {
                Pixel::new(
                    sum_r / count as f32,
                    sum_g / count as f32,
                    sum_b / count as f32,
                    0.0,
                )
            } else {
                Pixel::new(0.0, 0.0, 0.0, 0.0)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageStats {
    pub width: u32,
    pub height: u32,
    pub pixel_count: usize,
    pub pixel_format: PixelFormat,
    pub min_color: Pixel,
    pub max_color: Pixel,
    pub average_color: Pixel,
}

impl Default for ImageBuffer {
    fn default() -> Self {
        Self::new(1, 1, PixelFormat::Rgba8)
    }
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Rgba8
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 255.0)
    }
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixel_format: PixelFormat::Rgba8,
            color_space: "sRGB".to_string(),
            gamma: 2.2,
            dpi: None,
            exif: None,
            icc_profile: None,
        }
    }
}

impl Default for ImageStats {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixel_count: 1,
            pixel_format: PixelFormat::Rgba8,
            min_color: Pixel::new(0.0, 0.0, 0.0, 0.0),
            max_color: Pixel::new(255.0, 255.0, 255.0, 255.0),
            average_color: Pixel::new(127.5, 127.5, 127.5, 255.0),
        }
    }
}
