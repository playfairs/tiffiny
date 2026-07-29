use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoLoader {
  pub id: String,
  pub name: String,
  pub cache_enabled: Arc<RwLock<bool>>,
  pub cache_size: Arc<RwLock<usize>>,
  pub event_sender: mpsc::UnboundedSender<LoaderEvent>,
  pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<LoaderEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum LoaderEvent {
  LoadingStarted(String),
  LoadingProgress(f32),
  LoadingCompleted(Arc<crate::video_buffer::Buffer>),
  Error(String),
  CacheUpdated,
}

#[derive(Debug, Clone)]
pub struct LoadOptions {
  pub seek_time: Option<std::time::Duration>,
  pub frame_count: Option<usize>,
  pub start_frame: Option<usize>,
  pub end_frame: Option<usize>,
  pub extract_audio: bool,
  pub extract_subtitles: bool,
  pub quality: VideoQuality,
  pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoQuality {
  Low,
  Medium,
  High,
  Ultra,
  Custom(u32, u32),
  width,
  height,
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
  pub path: String,
  pub format: String,
  pub duration: std::time::Duration,
  pub width: u32,
  pub height: u32,
  pub frame_rate: f32,
  pub bitrate: u64,
  pub codec: String,
  pub audio_streams: Vec<AudioStreamInfo>,
  pub subtitle_streams: Vec<SubtitleStreamInfo>,
  pub metadata: crate::video_buffer::VideoMetadata,
}

#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
  pub index: u32,
  pub codec: String,
  pub sample_rate: u32,
  pub channels: u32,
  pub bitrate: u64,
  pub language: Option<String>,
  pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubtitleStreamInfo {
  pub index: u32,
  pub codec: String,
  pub language: Option<String>,
  pub title: Option<String>,
  pub format: String,
}

impl VideoLoader {
  pub fn new(id: String, name: String) -> Self {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    Self {
      id,
      name,
      cache_enabled: Arc::new(RwLock::new(true)),
      cache_size: Arc::new(RwLock::new(100 * 1024 * 1024)),
      event_sender,
      event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
    }
  }

  pub async fn load_from_path(
    &self,
    path: &str,
    options: Option<LoadOptions>,
  ) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingStarted(path.to_string()));

    let load_options = options.unwrap_or_default();

    let video_info = self.get_video_info(path).await?;
    let video_buffer = self
      .create_buffer_from_info(&video_info, &load_options)
      .await?;

    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingCompleted(Arc::new(video_buffer)));

    Ok(Arc::new(video_buffer))
  }

  pub async fn load_from_url(
    &self,
    url: &str,
    options: Option<LoadOptions>,
  ) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingStarted(url.to_string()));

    let load_options = options.unwrap_or_default();

    let video_info = self.get_video_info_from_url(url).await?;
    let video_buffer = self
      .create_buffer_from_info(&video_info, &load_options)
      .await?;

    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingCompleted(Arc::new(video_buffer)));

    Ok(Arc::new(video_buffer))
  }

  pub async fn load_from_bytes(
    &self,
    data: &[u8],
    options: Option<LoadOptions>,
  ) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingStarted("From bytes".to_string()));

    let load_options = options.unwrap_or_default();

    let video_info = self.get_video_info_from_bytes(data).await?;
    let video_buffer = self
      .create_buffer_from_info(&video_info, &load_options)
      .await?;

    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingCompleted(Arc::new(video_buffer)));

    Ok(Arc::new(video_buffer))
  }

  pub async fn load_batch(
    &self,
    paths: &[String],
    options: Option<LoadOptions>,
  ) -> Result<Vec<Arc<crate::video_buffer::Buffer>>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    for (index, path) in paths.iter().enumerate() {
      let progress = (index as f32 / paths.len() as f32) * 100.0;
      let _ = self
        .event_sender
        .send(LoaderEvent::LoadingProgress(progress));

      let video_buffer = self.load_from_path(path, options.clone()).await?;
      results.push(video_buffer);
    }

    Ok(results)
  }

  pub async fn get_video_info(&self, path: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    let duration = std::time::Duration::from_secs(120);
    let width = 1920;
    let height = 1080;
    let frame_rate = 30.0;
    let bitrate = 5_000_000;

    Ok(VideoInfo {
      path: path.to_string(),
      format: "mp4".to_string(),
      duration,
      width,
      height,
      frame_rate,
      bitrate,
      codec: "h264".to_string(),
      audio_streams: vec![AudioStreamInfo {
        index: 0,
        codec: "aac".to_string(),
        sample_rate: 48000,
        channels: 2,
        bitrate: 128_000,
        language: Some("en".to_string()),
        title: Some("Audio Track".to_string()),
      }],
      subtitle_streams: vec![SubtitleStreamInfo {
        index: 0,
        codec: "srt".to_string(),
        language: Some("en".to_string()),
        title: Some("English Subtitles".to_string()),
        format: "srt".to_string(),
      }],
      metadata: crate::video_buffer::VideoMetadata {
        title: Some("Sample Video".to_string()),
        author: Some("Tiffiny Studio".to_string()),
        copyright: Some("© 2024 Tiffiny Studio".to_string()),
        description: Some("A sample video for testing".to_string()),
        duration: Some(duration),
        creation_time: Some(std::time::SystemTime::now()),
        bitrate: Some(bitrate as u32),
        codec: Some("h264".to_string()),
        container: Some("mp4".to_string()),
        width,
        height,
        frame_rate,
        pixel_aspect_ratio: Some("16:9".to_string()),
        color_space: Some("yuv420p".to_string()),
        language: Some("en".to_string()),
        chapters: vec![],
        tags: std::collections::HashMap::new(),
      },
    })
  }

  pub async fn get_video_info_from_url(
    &self,
    url: &str,
  ) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    self.get_video_info(url).await
  }

  pub async fn get_video_info_from_bytes(
    &self,
    data: &[u8],
  ) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    let duration = std::time::Duration::from_secs(60);
    let width = 1280;
    let height = 720;
    let frame_rate = 25.0;
    let bitrate = 2_500_000;

    Ok(VideoInfo {
      path: "From bytes".to_string(),
      format: "mp4".to_string(),
      duration,
      width,
      height,
      frame_rate,
      bitrate,
      codec: "h264".to_string(),
      audio_streams: vec![],
      subtitle_streams: vec![],
      metadata: crate::video_buffer::VideoMetadata {
        title: Some("Byte Video".to_string()),
        author: None,
        copyright: None,
        description: None,
        duration: Some(duration),
        creation_time: None,
        bitrate: Some(bitrate as u32),
        codec: Some("h264".to_string()),
        container: Some("mp4".to_string()),
        width,
        height,
        frame_rate,
        pixel_aspect_ratio: Some("16:9".to_string()),
        color_space: Some("yuv420p".to_string()),
        language: None,
        chapters: vec![],
        tags: std::collections::HashMap::new(),
      },
    })
  }

  async fn create_buffer_from_info(
    &self,
    info: &VideoInfo,
    options: &LoadOptions,
  ) -> Result<crate::video_buffer::Buffer, Box<dyn std::error::Error>> {
    let (target_width, target_height) = match options.quality {
      VideoQuality::Low => (640, 360),
      VideoQuality::Medium => (1280, 720),
      VideoQuality::High => (1920, 1080),
      VideoQuality::Ultra => (3840, 2160),
      VideoQuality::Custom(w, h) => (w, h),
    };

    let pixel_format = crate::video_buffer::PixelFormat::Rgba8;
    let mut video_buffer =
      crate::video_buffer::Buffer::new(target_width, target_height, pixel_format, info.frame_rate);

    video_buffer.set_metadata(info.metadata.clone());

    let total_frames = (info.duration.as_secs_f64() * info.frame_rate) as usize;
    let start_frame = options.start_frame.unwrap_or(0);
    let end_frame = options.end_frame.unwrap_or(total_frames);
    let frame_count = end_frame - start_frame;
    let frame_step = if let Some(count) = options.frame_count {
      (frame_count as f64 / count as f64).ceil() as usize
    } else {
      1
    };

    for i in (start_frame..end_frame).step_by(frame_step) {
      let frame = self.create_sample_frame(target_width, target_height, pixel_format, i as u32);
      video_buffer.add_frame(frame);

      let progress = ((i - start_frame) as f32 / frame_count as f32) * 100.0;
      let _ = self
        .event_sender
        .send(LoaderEvent::LoadingProgress(progress));

      tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    Ok(video_buffer)
  }

  fn create_sample_frame(
    &self,
    width: u32,
    height: u32,
    pixel_format: crate::video_buffer::PixelFormat,
    frame_number: u32,
  ) -> crate::video_buffer::VideoFrame {
    let mut frame = crate::video_buffer::VideoFrame::new(width, height, pixel_format);
    frame.frame_number = frame_number;

    for y in 0..height {
      for x in 0..width {
        let r = ((x as f32 / width as f32) * 255.0) as f32;
        let g = ((y as f32 / height as f32) * 255.0) as f32;
        let b = ((frame_number % 255) as f32);

        let pixel = crate::video_buffer::Pixel::new(r, g, b, 255.0);
        frame.set_pixel(x, y, pixel);
      }
    }

    frame
  }

  pub async fn load_thumbnail(
    &self,
    path: &str,
    width: u32,
    height: u32,
  ) -> Result<crate::video_buffer::VideoFrame, Box<dyn std::error::Error>> {
    let video_buffer = self
      .load_from_path(
        path,
        Some(LoadOptions {
          frame_count: Some(1),
          quality: VideoQuality::Custom(width, height),
          ..Default::default()
        }),
      )
      .await?;

    if let Some(frame) = video_buffer.get_frame(0) {
      Ok(frame.resize(width, height))
    } else {
      Err("No frames found".into())
    }
  }

  pub async fn load_keyframes(
    &self,
    path: &str,
    interval: u32,
  ) -> Result<Vec<crate::video_buffer::VideoFrame>, Box<dyn std::error::Error>> {
    let video_buffer = self.load_from_path(path, None).await?;
    Ok(video_buffer.extract_keyframes(interval))
  }

  pub async fn load_segment(
    &self,
    path: &str,
    start_time: std::time::Duration,
    duration: std::time::Duration,
  ) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>> {
    let video_info = self.get_video_info(path).await?;
    let frame_rate = video_info.frame_rate;
    let start_frame = (start_time.as_secs_f64() * frame_rate) as usize;
    let end_frame = ((start_time + duration).as_secs_f64() * frame_rate) as usize;

    let options = LoadOptions {
      start_frame: Some(start_frame),
      end_frame: Some(end_frame),
      ..Default::default()
    };

    self.load_from_path(path, Some(options)).await
  }

  pub async fn load_with_audio(
    &self,
    path: &str,
  ) -> Result<
    (
      Arc<crate::video_buffer::Buffer>,
      Option<Arc<tiffiny_audio::audio_buffer::AudioBuffer>>,
    ),
    Box<dyn std::error::Error>,
  > {
    let video_buffer = self
      .load_from_path(
        path,
        Some(LoadOptions {
          extract_audio: true,
          ..Default::default()
        }),
      )
      .await?;

    let audio_buffer = self.extract_audio_from_video(path).await?;

    Ok((video_buffer, audio_buffer))
  }

  async fn extract_audio_from_video(
    &self,
    path: &str,
  ) -> Result<Option<Arc<tiffiny_audio::audio_buffer::AudioBuffer>>, Box<dyn std::error::Error>> {
    let audio_buffer = tiffiny_audio::audio_buffer::AudioBuffer::new(
      48000,
      2,
      16,
      tiffiny_audio::audio_format::AudioFormat::Wav,
    );

    Ok(Some(Arc::new(audio_buffer)))
  }

  pub async fn load_subtitles(
    &self,
    path: &str,
  ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(vec![
      "Sample subtitle 1".to_string(),
      "Sample subtitle 2".to_string(),
    ])
  }

  pub async fn load_chapters(
    &self,
    path: &str,
  ) -> Result<Vec<crate::video_buffer::Chapter>, Box<dyn std::error::Error>> {
    Ok(vec![
      crate::video_buffer::Chapter {
        id: 0,
        title: "Chapter 1".to_string(),
        start_time: std::time::Duration::from_secs(0),
        end_time: std::time::Duration::from_secs(60),
        description: Some("First chapter".to_string()),
      },
      crate::video_buffer::Chapter {
        id: 1,
        title: "Chapter 2".to_string(),
        start_time: std::time::Duration::from_secs(60),
        end_time: std::time::Duration::from_secs(120),
        description: Some("Second chapter".to_string()),
      },
    ])
  }

  pub fn set_cache_enabled(&self, enabled: bool) {
    let mut cache_enabled = self.cache_enabled.write();
    *cache_enabled = enabled;
  }

  pub fn is_cache_enabled(&self) -> bool {
    *self.cache_enabled.read()
  }

  pub fn set_cache_size(&self, size: usize) {
    let mut cache_size = self.cache_size.write();
    *cache_size = size;
  }

  pub fn get_cache_size(&self) -> usize {
    *self.cache_size.read()
  }

  pub fn clear_cache(&self) {
    let _ = self.event_sender.send(LoaderEvent::CacheUpdated);
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

  pub fn get_supported_formats(&self) -> Vec<String> {
    vec![
      "mp4".to_string(),
      "avi".to_string(),
      "mov".to_string(),
      "mkv".to_string(),
      "webm".to_string(),
      "flv".to_string(),
      "wmv".to_string(),
      "m4v".to_string(),
      "3gp".to_string(),
    ]
  }

  pub fn get_supported_codecs(&self) -> Vec<String> {
    vec![
      "h264".to_string(),
      "h265".to_string(),
      "vp9".to_string(),
      "av1".to_string(),
      "mpeg2".to_string(),
      "mpeg4".to_string(),
      "theora".to_string(),
      "vp8".to_string(),
    ]
  }

  pub fn can_load_format(&self, format: &str) -> bool {
    self
      .get_supported_formats()
      .contains(&format.to_lowercase())
  }

  pub fn can_load_codec(&self, codec: &str) -> bool {
    self.get_supported_codecs().contains(&codec.to_lowercase())
  }

  pub fn get_file_info(
    &self,
    path: &str,
  ) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut info = std::collections::HashMap::new();

    if let Ok(metadata) = std::fs::metadata(path) {
      info.insert("size".to_string(), metadata.len().to_string());
      info.insert("modified".to_string(), format!("{:?}", metadata.modified()));
      info.insert("created".to_string(), format!("{:?}", metadata.created()));
    }

    if let Some(extension) = std::path::Path::new(path).extension() {
      if let Some(ext_str) = extension.to_str() {
        info.insert("extension".to_string(), ext_str.to_lowercase());
      }
    }

    Ok(info)
  }

  pub fn validate_path(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(path).exists() {
      return Err(format!("File does not exist: {}", path).into());
    }

    if !std::path::Path::new(path).is_file() {
      return Err(format!("Path is not a file: {}", path).into());
    }

    let extension = std::path::Path::new(path)
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("");

    if !self.can_load_format(extension) {
      return Err(format!("Unsupported format: {}", extension).into());
    }

    Ok(())
  }

  pub async fn load_with_progress<F>(
    &self,
    path: &str,
    options: Option<LoadOptions>,
    progress_callback: F,
  ) -> Result<Arc<crate::video_buffer::Buffer>, Box<dyn std::error::Error>>
  where
    F: Fn(f32) + Send + Sync,
  {
    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingStarted(path.to_string()));

    let load_options = options.unwrap_or_default();
    let video_info = self.get_video_info(path).await?;

    for i in 0..=100 {
      progress_callback(i as f32);
      let _ = self
        .event_sender
        .send(LoaderEvent::LoadingProgress(i as f32));
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let video_buffer = self
      .create_buffer_from_info(&video_info, &load_options)
      .await?;
    let _ = self
      .event_sender
      .send(LoaderEvent::LoadingCompleted(Arc::new(video_buffer)));

    Ok(Arc::new(video_buffer))
  }

  pub fn clone_loader(&self) -> VideoLoader {
    let mut new_loader = Self::new(uuid::Uuid::new_v4().to_string(), self.name.clone());

    let cache_enabled = self.cache_enabled.read();
    let cache_size = self.cache_size.read();

    *new_loader.cache_enabled = *cache_enabled;
    *new_loader.cache_size = *cache_size;

    new_loader
  }
}

impl Default for VideoLoader {
  fn default() -> Self {
    Self::new(
      uuid::Uuid::new_v4().to_string(),
      "Default Video Loader".to_string(),
    )
  }
}

impl Default for LoadOptions {
  fn default() -> Self {
    Self {
      seek_time: None,
      frame_count: None,
      start_frame: None,
      end_frame: None,
      extract_audio: false,
      extract_subtitles: false,
      quality: VideoQuality::Medium,
      hardware_acceleration: false,
    }
  }
}

impl Default for VideoQuality {
  fn default() -> Self {
    VideoQuality::Medium
  }
}

impl Default for VideoInfo {
  fn default() -> Self {
    Self {
      path: String::new(),
      format: String::new(),
      duration: std::time::Duration::from_secs(0),
      width: 0,
      height: 0,
      frame_rate: 0.0,
      bitrate: 0,
      codec: String::new(),
      audio_streams: Vec::new(),
      subtitle_streams: Vec::new(),
      metadata: crate::video_buffer::VideoMetadata::default(),
    }
  }
}

impl Default for AudioStreamInfo {
  fn default() -> Self {
    Self {
      index: 0,
      codec: String::new(),
      sample_rate: 0,
      channels: 0,
      bitrate: 0,
      language: None,
      title: None,
    }
  }
}

impl Default for SubtitleStreamInfo {
  fn default() -> Self {
    Self {
      index: 0,
      codec: String::new(),
      language: None,
      title: None,
      format: String::new(),
    }
  }
}
