use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ImageLoader {
    pub id: String,
    pub supported_formats: Vec<ImageFormat>,
    pub cache_enabled: bool,
    pub cache_size: usize,
    pub cache: Arc<RwLock<std::collections::HashMap<String, Arc<crate::image_buffer::ImageBuffer>>>>,
    pub event_sender: mpsc::UnboundedSender<LoaderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<LoaderEvent>>>>,
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
    Dds,
    Pvr,
    Ktx2,
}

#[derive(Debug, Clone)]
pub enum LoaderEvent {
    ImageLoaded(String, Arc<crate::image_buffer::ImageBuffer>),
    LoadFailed(String, String),
    CacheHit(String),
    CacheMiss(String),
    CacheCleared,
    FormatDetected(String, ImageFormat),
}

impl ImageLoader {
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
            cache_enabled: true,
            cache_size: 100,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn load_from_path(&self, path: &str) -> Result<Arc<crate::image_buffer::ImageBuffer>, Box<dyn std::error::Error>> {
Check cache first
        if self.cache_enabled {
            if let Some(cached_image) = self.get_from_cache(path) {
                let _ = self.event_sender.send(LoaderEvent::CacheHit(path.to_string()));
                return Ok(cached_image);
            }
            let _ = self.event_sender.send(LoaderEvent::CacheMiss(path.to_string()));
        }

        let image = image::open(path)?;
        let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
        let image_arc = Arc::new(image_buffer);

        if self.cache_enabled {
            self.add_to_cache(path, image_arc.clone());
        }

        if let Some(format) = self.detect_format(path) {
            let _ = self.event_sender.send(LoaderEvent::FormatDetected(path.to_string(), format));
        }

        let _ = self.event_sender.send(LoaderEvent::ImageLoaded(path.to_string(), image_arc.clone()));
        Ok(image_arc)
    }

    pub async fn load_from_bytes(&self, data: &[u8], format_hint: Option<ImageFormat>) -> Result<Arc<crate::image_buffer::ImageBuffer>, Box<dyn std::error::Error>> {
        let image = if let Some(format) = format_hint {
            self.load_with_format(data, &format)?
        } else {
            self.load_auto_detect(data)?
        };

        let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
        let image_arc = Arc::new(image_buffer);

        let _ = self.event_sender.send(LoaderEvent::ImageLoaded("memory".to_string(), image_arc.clone()));
        Ok(image_arc)
    }

    pub async fn load_from_url(&self, url: &str) -> Result<Arc<crate::image_buffer::ImageBuffer>, Box<dyn std::error::Error>> {
        if self.cache_enabled {
            if let Some(cached_image) = self.get_from_cache(url) {
                let _ = self.event_sender.send(LoaderEvent::CacheHit(url.to_string()));
                return Ok(cached_image);
            }
            let _ = self.event_sender.send(LoaderEvent::CacheMiss(url.to_string()));
        }

        let response = reqwest::get(url).await?;
        let data = response.bytes().await?;

        let image = image::load_from_memory(&data)?;
        let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
        let image_arc = Arc::new(image_buffer);

        if self.cache_enabled {
            self.add_to_cache(url, image_arc.clone());
        }

        let _ = self.event_sender.send(LoaderEvent::ImageLoaded(url.to_string(), image_arc.clone()));
        Ok(image_arc)
    }

    pub async fn load_batch(&self, paths: &[String]) -> Result<Vec<Arc<crate::image_buffer::ImageBuffer>>, Box<dyn std::error::Error>> {
        let mut results = Vec::with_capacity(paths.len());
        let mut handles = Vec::new();

        for path in paths {
            let loader = self.clone();
            let path = path.clone();
            
            let handle = tokio::spawn(async move {
                loader.load_from_path(&path).await
            });
            
            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(image)) => results.push(image),
                Ok(Err(e)) => {
                    let error_msg = format!("Batch load failed: {}", e);
                    let _ = self.event_sender.send(LoaderEvent::LoadFailed("batch".to_string(), error_msg));
                },
                Err(e) => {
                    let error_msg = format!("Batch task failed: {}", e);
                    let _ = self.event_sender.send(LoaderEvent::LoadFailed("batch".to_string(), error_msg));
                },
            }
        }

        Ok(results)
    }

    pub async fn load_animation(&self, path: &str) -> Result<Vec<Arc<crate::image_buffer::ImageBuffer>>, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        
        match image {
            image::DynamicImage::ImageRgba8(img) => {
                if path.to_lowercase().ends_with(".gif") {
                    self.load_gif_frames(path).await
                } else {
                    let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
                    Ok(vec![Arc::new(image_buffer)])
                }
            },
            _ => {
                let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
                Ok(vec![Arc::new(image_buffer)])
            },
        }
    }

    async fn load_gif_frames(&self, path: &str) -> Result<Vec<Arc<crate::image_buffer::ImageBuffer>>, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
        Ok(vec![Arc::new(image_buffer)])
    }

    fn load_with_format(&self, data: &[u8], format: &ImageFormat) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
        match format {
            ImageFormat::Png => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Png)?),
            ImageFormat::Jpeg => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)?),
            ImageFormat::Gif => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Gif)?),
            ImageFormat::Bmp => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Bmp)?),
            ImageFormat::Tiff => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Tiff)?),
            ImageFormat::WebP => Ok(image::load_from_memory_with_format(data, image::ImageFormat::WebP)?),
            ImageFormat::Ico => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Ico)?),
            ImageFormat::Pnm => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Pnm)?),
            ImageFormat::Tga => Ok(image::load_from_memory_with_format(data, image::ImageFormat::Tga)?),
            _ => Err("Unsupported format".into()),
        }
    }

    fn load_auto_detect(&self, data: &[u8]) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
        Ok(image::load_from_memory(data)?)
    }

    fn detect_format(&self, path: &str) -> Option<ImageFormat> {
        let path_lower = path.to_lowercase();
        
        if path_lower.ends_with(".png") {
            Some(ImageFormat::Png)
        } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".jpeg") {
            Some(ImageFormat::Jpeg)
        } else if path_lower.ends_with(".gif") {
            Some(ImageFormat::Gif)
        } else if path_lower.ends_with(".bmp") {
            Some(ImageFormat::Bmp)
        } else if path_lower.ends_with(".tiff") || path_lower.ends_with(".tif") {
            Some(ImageFormat::Tiff)
        } else if path_lower.ends_with(".webp") {
            Some(ImageFormat::WebP)
        } else if path_lower.ends_with(".ico") {
            Some(ImageFormat::Ico)
        } else if path_lower.ends_with(".pnm") || path_lower.ends_with(".pbm") || 
                  path_lower.ends_with(".pgm") || path_lower.ends_with(".ppm") {
            Some(ImageFormat::Pnm)
        } else if path_lower.ends_with(".tga") {
            Some(ImageFormat::Tga)
        } else {
            None
        }
    }

    fn add_to_cache(&self, key: &str, image: Arc<crate::image_buffer::ImageBuffer>) {
        let mut cache = self.cache.write();
        
        if cache.len() >= self.cache_size {
            if let Some(oldest_key) = cache.keys().next() {
                cache.remove(oldest_key);
            }
        }
        
        cache.insert(key.to_string(), image);
    }

    fn get_from_cache(&self, key: &str) -> Option<Arc<crate::image_buffer::ImageBuffer>> {
        let cache = self.cache.read();
        cache.get(key).cloned()
    }

    pub fn remove_from_cache(&self, key: &str) -> Option<Arc<crate::image_buffer::ImageBuffer>> {
        let mut cache = self.cache.write();
        cache.remove(key)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        let _ = self.event_sender.send(LoaderEvent::CacheCleared);
    }

    pub fn set_cache_enabled(&self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    pub fn set_cache_size(&self, size: usize) {
        self.cache_size = size;
        
        let cache = self.cache.read();
        if cache.len() > size {
            drop(cache);
            self.clear_cache();
        }
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache.read().len()
    }

    pub fn get_supported_formats(&self) -> Vec<ImageFormat> {
        self.supported_formats.clone()
    }

    pub fn supports_format(&self, format: &ImageFormat) -> bool {
        self.supported_formats.contains(format)
    }

    pub async fn get_image_info(&self, path: &str) -> Result<ImageInfo, Box<dyn std::error::Error>> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        let color_type = image.color();
        let format = self.detect_format(path);
        
        Ok(ImageInfo {
            path: path.to_string(),
            width,
            height,
            format,
            color_type: self.color_type_to_string(color_type),
            file_size: std::fs::metadata(path)?.len(),
            has_transparency: self.has_transparency(color_type),
            is_animated: self.is_animated(path),
        })
    }

    fn color_type_to_string(&self, color_type: image::ColorType) -> String {
        match color_type {
            image::ColorType::L8 => "Grayscale 8-bit".to_string(),
            image::ColorType::L16 => "Grayscale 16-bit".to_string(),
            image::ColorType::La8 => "Grayscale with Alpha 8-bit".to_string(),
            image::ColorType::La16 => "Grayscale with Alpha 16-bit".to_string(),
            image::ColorType::Rgb8 => "RGB 8-bit".to_string(),
            image::ColorType::Rgb16 => "RGB 16-bit".to_string(),
            image::ColorType::Rgba8 => "RGBA 8-bit".to_string(),
            image::ColorType::Rgba16 => "RGBA 16-bit".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn has_transparency(&self, color_type: image::ColorType) -> bool {
        matches!(color_type, image::ColorType::La8 | image::ColorType::La16 | image::ColorType::Rgba8 | image::ColorType::Rgba16)
    }

    fn is_animated(&self, path: &str) -> bool {
        path.to_lowercase().ends_with(".gif")
    }

    pub async fn get_events(&mut self) -> Vec<LoaderEvent> {
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

    pub fn get_loader_stats(&self) -> LoaderStats {
        let cache = self.cache.read();
        
        LoaderStats {
            supported_formats: self.supported_formats.clone(),
            cache_enabled: self.cache_enabled,
            cache_size: self.cache_size,
            cache_usage: cache.len(),
            supported_count: self.supported_formats.len(),
        }
    }

    pub fn preload_thumbnails(&self, paths: &[String], thumbnail_size: (u32, u32)) -> Result<Vec<Arc<crate::image_buffer::ImageBuffer>>, Box<dyn std::error::Error>> {
        let mut thumbnails = Vec::with_capacity(paths.len());
        
        for path in paths {
            if let Ok(image) = self.load_from_path(path) {
                let thumbnail = self.create_thumbnail(&image, thumbnail_size.0, thumbnail_size.1)?;
                thumbnails.push(Arc::new(thumbnail));
            }
        }
        
        Ok(thumbnails)
    }

    fn create_thumbnail(&self, image: &Arc<crate::image_buffer::ImageBuffer>, width: u32, height: u32) -> Result<crate::image_buffer::ImageBuffer, Box<dyn std::error::Error>> {
        let (original_width, original_height) = (image.width, image.height);
        
        let aspect_ratio = original_width as f32 / original_height as f32;
        let (thumb_width, thumb_height) = if aspect_ratio > (width as f32 / height as f32) {
            (width, (width as f32 / aspect_ratio) as u32)
        } else {
            ((height as f32 * aspect_ratio) as u32, height)
        };
        
        Ok(image.resize(thumb_width, thumb_height))
    }

    pub async fn load_with_progress<F>(&self, path: &str, progress_callback: F) -> Result<Arc<crate::image_buffer::ImageBuffer>, Box<dyn std::error::Error>>
    where
        F: Fn(f32) + Send + Sync,
    {
        if self.cache_enabled {
            if let Some(cached_image) = self.get_from_cache(path) {
                progress_callback(1.0);
                let _ = self.event_sender.send(LoaderEvent::CacheHit(path.to_string()));
                return Ok(cached_image);
            }
            let _ = self.event_sender.send(LoaderEvent::CacheMiss(path.to_string()));
        }

        progress_callback(0.0);
        
        let image = image::open(path)?;
        progress_callback(0.5);
        
        let image_buffer = crate::image_buffer::ImageBuffer::from_image(&image)?;
        let image_arc = Arc::new(image_buffer);
        
        progress_callback(1.0);
        
        if self.cache_enabled {
            self.add_to_cache(path, image_arc.clone());
        }

        let _ = self.event_sender.send(LoaderEvent::ImageLoaded(path.to_string(), image_arc.clone()));
        Ok(image_arc)
    }
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub format: Option<ImageFormat>,
    pub color_type: String,
    pub file_size: u64,
    pub has_transparency: bool,
    pub is_animated: bool,
}

#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub supported_formats: Vec<ImageFormat>,
    pub cache_enabled: bool,
    pub cache_size: usize,
    pub cache_usage: usize,
    pub supported_count: usize,
}

impl Default for ImageLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        ImageFormat::Png
    }
}

impl Default for LoaderStats {
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
            cache_enabled: true,
            cache_size: 100,
            cache_usage: 0,
            supported_count: 9,
        }
    }
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            path: String::new(),
            width: 0,
            height: 0,
            format: None,
            color_type: "Unknown".to_string(),
            file_size: 0,
            has_transparency: false,
            is_animated: false,
        }
    }
}
