use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageSaver {
    pub id: String,
    pub supported_formats: Vec<ImageFormat>,
    pub default_quality: u8,
    pub compression_level: u8,
    pub event_sender: mpsc::UnboundedSender<SaverEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SaverEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Tiff,
    WebP,
    Ico,
    Pnm,
    Tga,
}

#[derive(Debug, Clone)]
pub enum SaverEvent {
    ImageSaved(String),
    SaveFailed(String, String),
    Progress(String, f32),
    CompressionStarted(String),
    CompressionCompleted(String),
}

#[derive(Debug, Clone)]
pub struct SaveOptions {
    pub format: ImageFormat,
    pub quality: Option<u8>,
    pub compression: Option<u8>,
    pub progressive: bool,
    pub optimize: bool,
    pub metadata: Option<ImageMetadata>,
}

#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub dpi: Option<(u32, u32)>,
    pub color_profile: Option<Vec<u8>>,
    pub exif: Option<std::collections::HashMap<String, String>>,
    pub comment: Option<String>,
}

impl ImageSaver {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            supported_formats: vec![
                ImageFormat::Png,
                ImageFormat::Jpeg,
                ImageFormat::Gif,
                ImageFormat::Bmp,
                ImageFormat::Tiff,
                ImageFormat::WebP,
                ImageFormat::Ico,
                ImageFormat::Pnm,
                ImageFormat::Tga,
            ],
            default_quality: 85,
            compression_level: 6,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn save(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(SaverEvent::CompressionStarted(path.to_string()));
        
        let result = match options.format {
            ImageFormat::Png => self.save_png(image, path, &options),
            ImageFormat::Jpeg => self.save_jpeg(image, path, &options),
            ImageFormat::Gif => self.save_gif(image, path, &options),
            ImageFormat::Bmp => self.save_bmp(image, path, &options),
            ImageFormat::Tiff => self.save_tiff(image, path, &options),
            ImageFormat::WebP => self.save_webp(image, path, &options),
            ImageFormat::Ico => self.save_ico(image, path, &options),
            ImageFormat::Pnm => self.save_pnm(image, path, &options),
            ImageFormat::Tga => self.save_tga(image, path, &options),
        };

        match result {
            Ok(()) => {
                let _ = self.event_sender.send(SaverEvent::ImageSaved(path.to_string()));
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to save {}: {}", path, e);
                let _ = self.event_sender.send(SaverEvent::SaveFailed(path.to_string(), error_msg));
                Err(e)
            },
        }
    }

    pub async fn save_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let _ = self.event_sender.send(SaverEvent::CompressionStarted(path.to_string()));
        
        progress_callback(0.0);
        
        let result = match options.format {
            ImageFormat::Png => self.save_png_with_progress(image, path, &options, progress_callback),
            ImageFormat::Jpeg => self.save_jpeg_with_progress(image, path, &options, progress_callback),
            ImageFormat::Gif => self.save_gif_with_progress(image, path, &options, progress_callback),
            ImageFormat::Bmp => self.save_bmp_with_progress(image, path, &options, progress_callback),
            ImageFormat::Tiff => self.save_tiff_with_progress(image, path, &options, progress_callback),
            ImageFormat::WebP => self.save_webp_with_progress(image, path, &options, progress_callback),
            ImageFormat::Ico => self.save_ico_with_progress(image, path, &options, progress_callback),
            ImageFormat::Pnm => self.save_pnm_with_progress(image, path, &options, progress_callback),
            ImageFormat::Tga => self.save_tga_with_progress(image, path, &options, progress_callback),
        };

        progress_callback(1.0);
        
        match result {
            Ok(()) => {
                let _ = self.event_sender.send(SaverEvent::ImageSaved(path.to_string()));
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to save {}: {}", path, e);
                let _ = self.event_sender.send(SaverEvent::SaveFailed(path.to_string(), error_msg));
                Err(e)
            },
        }
    }

    pub async fn save_batch(&self, images: &[Arc<crate::image_buffer::ImageBuffer>], paths: &[String], options: SaveOptions) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if images.len() != paths.len() {
            return Err("Images and paths length mismatch".into());
        }

        let mut successful_paths = Vec::new();
        let mut handles = Vec::new();

        for (image, path) in images.iter().zip(paths.iter()) {
            let saver = self.clone();
            let image = image.clone();
            let path = path.clone();
            let options = options.clone();
            
            let handle = tokio::spawn(async move {
                saver.save(&image, &path, options).await
            });
            
            handles.push(handle);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(Ok(())) => {
                    successful_paths.push(paths[i].clone());
                },
                Ok(Err(e)) => {
                    let error_msg = format!("Batch save failed for {}: {}", paths[i], e);
                    let _ = self.event_sender.send(SaverEvent::SaveFailed(paths[i].clone(), error_msg));
                },
                Err(e) => {
                    let error_msg = format!("Batch task failed for {}: {}", paths[i], e);
                    let _ = self.event_sender.send(SaverEvent::SaveFailed(paths[i].clone(), error_msg));
                },
            }
        }

        Ok(successful_paths)
    }

    fn save_png(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        let compression = options.compression.unwrap_or(self.compression_level);
        
Convert to PNG format
        let png_data = match dynamic_image {
            image::DynamicImage::ImageRgba8(img) => {
                let mut buffer = Vec::new();
                {
                    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                    img.write_with_encoder(encoder)?;
                }
                buffer
            },
            image::DynamicImage::ImageRgb8(img) => {
                let mut buffer = Vec::new();
                {
                    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                    img.write_with_encoder(encoder)?;
                }
                buffer
            },
            _ => {
                let rgba_img = dynamic_image.to_rgba8();
                let mut buffer = Vec::new();
                {
                    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                    rgba_img.write_with_encoder(encoder)?;
                }
                buffer
            },
        };

        std::fs::write(path, png_data)?;
        Ok(())
    }

    fn save_jpeg(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        let quality = options.quality.unwrap_or(self.default_quality);
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_gif(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::gif::GifEncoder::new(&mut buffer);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_bmp(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::bmp::BmpEncoder::new(&mut buffer);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_tiff(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::tiff::TiffEncoder::new(&mut buffer);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_webp(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        let quality = options.quality.unwrap_or(self.default_quality);
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::webp::WebPEncoder::new_with_quality(&mut buffer, quality as f32);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_ico(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgba_img = dynamic_image.to_rgba8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::ico::IcoEncoder::new(&mut buffer);
            rgba_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_pnm(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::pnm::PnmEncoder::new(&mut buffer);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_tga(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_image = image.to_image_buffer();
        
        let rgb_img = dynamic_image.to_rgb8();
        
        let mut buffer = Vec::new();
        {
            let encoder = image::codecs::tga::TgaEncoder::new(&mut buffer);
            rgb_img.write_with_encoder(encoder)?;
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    fn save_png_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_png(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_jpeg_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_jpeg(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_gif_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_gif(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_bmp_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_bmp(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_tiff_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_tiff(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_webp_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_webp(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_ico_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_ico(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_pnm_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_pnm(image, path, options);
        progress_callback(1.0);
        result
    }

    fn save_tga_with_progress<F>(&self, image: &crate::image_buffer::ImageBuffer, path: &str, options: &SaveOptions, progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        progress_callback(0.1);
        let result = self.save_tga(image, path, options);
        progress_callback(1.0);
        result
    }

    pub async fn save_animation(&self, frames: &[crate::image_buffer::ImageBuffer], path: &str, options: SaveOptions, delay_ms: u16) -> Result<(), Box<dyn std::error::Error>> {
        match options.format {
            ImageFormat::Gif => self.save_animated_gif(frames, path, &options, delay_ms),
            _ => Err("Animation only supported for GIF format".into()),
        }
    }

    fn save_animated_gif(&self, frames: &[crate::image_buffer::ImageBuffer], path: &str, options: &SaveOptions, delay_ms: u16) -> Result<(), Box<dyn std::error::Error>> {
        let rgb_frames: Vec<image::RgbImage> = frames
            .iter()
            .map(|frame| frame.to_image_buffer().to_rgb8())
            .collect();

        let mut buffer = Vec::new();
        {
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut buffer);
            
            for frame in rgb_frames {
                let frame = image::Frame::new(
                    frame.into(),
                    0, 0,
                    delay_ms,
                    None,
                );
                encoder.encode(frame)?;
            }
        }

        std::fs::write(path, buffer)?;
        Ok(())
    }

    pub fn get_supported_formats(&self) -> Vec<ImageFormat> {
        self.supported_formats.clone()
    }

    pub fn supports_format(&self, format: &ImageFormat) -> bool {
        self.supported_formats.contains(format)
    }

    pub fn get_default_options(&self, format: ImageFormat) -> SaveOptions {
        match format {
            ImageFormat::Png => SaveOptions {
                format,
                quality: None,
                compression: Some(self.compression_level),
                progressive: false,
                optimize: true,
                metadata: None,
            },
            ImageFormat::Jpeg => SaveOptions {
                format,
                quality: Some(self.default_quality),
                compression: None,
                progressive: true,
                optimize: true,
                metadata: None,
            },
            ImageFormat::Gif => SaveOptions {
                format,
                quality: None,
                compression: None,
                progressive: false,
                optimize: false,
                metadata: None,
            },
            ImageFormat::WebP => SaveOptions {
                format,
                quality: Some(self.default_quality),
                compression: None,
                progressive: false,
                optimize: true,
                metadata: None,
            },
            _ => SaveOptions {
                format,
                quality: None,
                compression: None,
                progressive: false,
                optimize: false,
                metadata: None,
            },
        }
    }

    pub fn set_default_quality(&mut self, quality: u8) {
        self.default_quality = quality.clamp(1, 100);
    }

    pub fn set_compression_level(&mut self, level: u8) {
        self.compression_level = level.clamp(0, 9);
    }

    pub async fn get_events(&mut self) -> Vec<SaverEvent> {
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

    pub fn estimate_file_size(&self, image: &crate::image_buffer::ImageBuffer, format: &ImageFormat, quality: Option<u8>) -> u64 {
        let pixel_count = (image.width * image.height) as u64;
        
        match format {
            ImageFormat::Png => {
                let bytes_per_pixel = if image.pixel_format.has_alpha() { 4 } else { 3 };
                pixel_count * bytes_per_pixel as u64 / 2
            },
            ImageFormat::Jpeg => {
                let bytes_per_pixel = 3;
                let quality_factor = quality.unwrap_or(self.default_quality) as f64 / 100.0;
                pixel_count * bytes_per_pixel as u64 * (1.0 - quality_factor * 0.8) as u64
            },
            ImageFormat::Bmp => {
                let bytes_per_pixel = if image.pixel_format.has_alpha() { 4 } else { 3 };
                pixel_count * bytes_per_pixel as u64 + 54
            },
            ImageFormat::Tiff => {
                let bytes_per_pixel = if image.pixel_format.has_alpha() { 4 } else { 3 };
                pixel_count * bytes_per_pixel as u64 / 3
            },
            ImageFormat::WebP => {
                let bytes_per_pixel = 3;
                let quality_factor = quality.unwrap_or(self.default_quality) as f64 / 100.0;
                pixel_count * bytes_per_pixel as u64 * (1.0 - quality_factor * 0.7) as u64
            },
            _ => pixel_count * 3,
        }
    }

    pub fn get_saver_stats(&self) -> SaverStats {
        SaverStats {
            supported_formats: self.supported_formats.clone(),
            default_quality: self.default_quality,
            compression_level: self.compression_level,
            supported_count: self.supported_formats.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaverStats {
    pub supported_formats: Vec<ImageFormat>,
    pub default_quality: u8,
    pub compression_level: u8,
    pub supported_count: usize,
}

impl Default for ImageSaver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        ImageFormat::Png
    }
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            quality: None,
            compression: None,
            progressive: false,
            optimize: true,
            metadata: None,
        }
    }
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            dpi: None,
            color_profile: None,
            exif: None,
            comment: None,
        }
    }
}

impl Default for SaverStats {
    fn default() -> Self {
        Self {
            supported_formats: vec![
                ImageFormat::Png,
                ImageFormat::Jpeg,
                ImageFormat::Gif,
                ImageFormat::Bmp,
                ImageFormat::Tiff,
                ImageFormat::WebP,
                ImageFormat::Ico,
                ImageFormat::Pnm,
                ImageFormat::Tga,
            ],
            default_quality: 85,
            compression_level: 6,
            supported_count: 9,
        }
    }
}
