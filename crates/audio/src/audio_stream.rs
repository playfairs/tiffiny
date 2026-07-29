use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioStream {
  pub id: String,
  pub name: String,
  pub stream_type: StreamType,
  pub sample_rate: u32,
  pub channels: u16,
  pub buffer_size: usize,
  pub device_id: Option<String>,
  pub latency: Option<u32>,
  pub state: Arc<RwLock<StreamState>>,
  pub buffer: Arc<RwLock<crate::audio_buffer::AudioBuffer>>,
  pub event_sender: mpsc::UnboundedSender<StreamEvent>,
  pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
  pub callback: Option<Arc<dyn Fn(&mut crate::audio_buffer::AudioBuffer) + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
  Input,
  Output,
  Duplex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
  Stopped,
  Starting,
  Running,
  Stopping,
  Error(String),
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
  StateChanged(StreamState),
  BufferProcessed,
  Underrun,
  Overrun,
  DeviceChanged(String),
  Error(String),
}

impl AudioStream {
  pub fn new(
    config: crate::audio_engine::StreamConfig,
    sample_rate: u32,
    channels: u16,
    buffer_size: usize,
  ) -> Self {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    Self {
      id: uuid::Uuid::new_v4().to_string(),
      name: config.name,
      stream_type: config.stream_type,
      sample_rate,
      channels,
      buffer_size,
      device_id: config.output_device.or(config.input_device),
      latency: config.latency,
      state: Arc::new(RwLock::new(StreamState::Stopped)),
      buffer: Arc::new(RwLock::new(crate::audio_buffer::AudioBuffer::new(
        channels,
        sample_rate,
        buffer_size,
        crate::audio_buffer::AudioFormat::F32,
      ))),
      event_sender,
      event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
      callback: None,
    }
  }

  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != StreamState::Stopped {
      return Err("Stream is not stopped".into());
    }

    *state = StreamState::Starting;
    let _ = self
      .event_sender
      .send(StreamEvent::StateChanged(StreamState::Starting));

    match self.stream_type {
      StreamType::Input => self.start_input_stream().await?,
      StreamType::Output => self.start_output_stream().await?,
      StreamType::Duplex => self.start_duplex_stream().await?,
    }

    *state = StreamState::Running;
    let _ = self
      .event_sender
      .send(StreamEvent::StateChanged(StreamState::Running));

    self.start_processing_loop().await;

    Ok(())
  }

  pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != StreamState::Running {
      return Err("Stream is not running".into());
    }

    *state = StreamState::Stopping;
    let _ = self
      .event_sender
      .send(StreamEvent::StateChanged(StreamState::Stopping));

    match self.stream_type {
      StreamType::Input => self.stop_input_stream().await?,
      StreamType::Output => self.stop_output_stream().await?,
      StreamType::Duplex => self.stop_duplex_stream().await?,
    }

    *state = StreamState::Stopped;
    let _ = self
      .event_sender
      .send(StreamEvent::StateChanged(StreamState::Stopped));

    Ok(())
  }

  pub async fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
    let state = self.state.read();
    if *state != StreamState::Running {
      return Err("Stream is not running".into());
    }

    match self.stream_type {
      StreamType::Input => self.pause_input_stream().await?,
      StreamType::Output => self.pause_output_stream().await?,
      StreamType::Duplex => self.pause_duplex_stream().await?,
    }

    Ok(())
  }

  pub async fn resume(&self) -> Result<(), Box<dyn std::error::Error>> {
    let state = self.state.read();
    if *state != StreamState::Running {
      return Err("Stream is not running".into());
    }

    match self.stream_type {
      StreamType::Input => self.resume_input_stream().await?,
      StreamType::Output => self.resume_output_stream().await?,
      StreamType::Duplex => self.resume_duplex_stream().await?,
    }

    Ok(())
  }

  pub fn set_callback(
    &mut self,
    callback: Arc<dyn Fn(&mut crate::audio_buffer::AudioBuffer) + Send + Sync>,
  ) {
    self.callback = Some(callback);
  }

  pub fn clear_callback(&mut self) {
    self.callback = None;
  }

  pub fn get_state(&self) -> StreamState {
    self.state.read().clone()
  }

  pub fn is_running(&self) -> bool {
    matches!(*self.state.read(), StreamState::Running)
  }

  pub fn is_stopped(&self) -> bool {
    matches!(*self.state.read(), StreamState::Stopped)
  }

  pub fn get_buffer(&self) -> Arc<RwLock<crate::audio_buffer::AudioBuffer>> {
    self.buffer.clone()
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;

    let mut buffer = self.buffer.write();
    *buffer = crate::audio_buffer::AudioBuffer::new(
      self.channels,
      sample_rate,
      self.buffer_size,
      crate::audio_buffer::AudioFormat::F32,
    );
  }

  pub fn set_channels(&mut self, channels: u16) {
    self.channels = channels;

    let mut buffer = self.buffer.write();
    *buffer = crate::audio_buffer::AudioBuffer::new(
      channels,
      self.sample_rate,
      self.buffer_size,
      crate::audio_buffer::AudioFormat::F32,
    );
  }

  pub fn set_buffer_size(&mut self, buffer_size: usize) {
    self.buffer_size = buffer_size;

    let mut buffer = self.buffer.write();
    *buffer = crate::audio_buffer::AudioBuffer::new(
      self.channels,
      self.sample_rate,
      buffer_size,
      crate::audio_buffer::AudioFormat::F32,
    );
  }

  pub fn set_device_id(&mut self, device_id: Option<String>) {
    self.device_id = device_id;
  }

  pub fn set_latency(&mut self, latency: Option<u32>) {
    self.latency = latency;
  }

  pub async fn get_events(&mut self) -> Vec<StreamEvent> {
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

  pub fn get_stats(&self) -> StreamStats {
    StreamStats {
      id: self.id.clone(),
      name: self.name.clone(),
      stream_type: self.stream_type.clone(),
      sample_rate: self.sample_rate,
      channels: self.channels,
      buffer_size: self.buffer_size,
      device_id: self.device_id.clone(),
      latency: self.latency,
      state: self.state.read().clone(),
    }
  }

  async fn start_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting input stream: {}", self.name);
    Ok(())
  }

  async fn start_output_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting output stream: {}", self.name);
    Ok(())
  }

  async fn start_duplex_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting duplex stream: {}", self.name);
    Ok(())
  }

  async fn stop_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Stopping input stream: {}", self.name);
    Ok(())
  }

  async fn stop_output_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Stopping output stream: {}", self.name);
    Ok(())
  }

  async fn stop_duplex_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Stopping duplex stream: {}", self.name);
    Ok(())
  }

  async fn pause_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Pausing input stream: {}", self.name);
    Ok(())
  }

  async fn pause_output_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Pausing output stream: {}", self.name);
    Ok(())
  }

  async fn pause_duplex_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Pausing duplex stream: {}", self.name);
    Ok(())
  }

  async fn resume_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Resuming input stream: {}", self.name);
    Ok(())
  }

  async fn resume_output_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Resuming output stream: {}", self.name);
    Ok(())
  }

  async fn resume_duplex_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Resuming duplex stream: {}", self.name);
    Ok(())
  }

  async fn start_processing_loop(&self) {
    let buffer = self.buffer.clone();
    let state = self.state.clone();
    let event_sender = self.event_sender.clone();
    let callback = self.callback.clone();
    let sample_rate = self.sample_rate;
    let buffer_size = self.buffer_size;

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        (buffer_size as u64 * 1000) / sample_rate as u64,
      ));

      loop {
        interval.tick().await;

        let current_state = state.read().clone();
        if !matches!(current_state, StreamState::Running) {
          break;
        }

        if let Some(ref callback) = callback {
          let mut audio_buffer = buffer.write();
          callback(&mut audio_buffer);
        }

        let _ = event_sender.send(StreamEvent::BufferProcessed);

        let audio_buffer = buffer.read();
        let peak = audio_buffer.get_peak();

        if peak > 0.95 {
          let _ = event_sender.send(StreamEvent::Overrun);
        } else if peak < 0.01 {
          let _ = event_sender.send(StreamEvent::Underrun);
        }
      }
    });
  }

  pub fn process_input_data(&self, input_data: &[f32]) {
    let mut buffer = self.buffer.write();

    for (i, &sample) in input_data.iter().enumerate() {
      if i < buffer.length * buffer.channels as usize {
        let channel = (i % buffer.channels as usize) as u16;
        let frame = i / buffer.channels as usize;
        if frame < buffer.length {
          buffer.set_sample(channel, frame, sample);
        }
      }
    }
  }

  pub fn get_output_data(&self) -> Vec<f32> {
    let buffer = self.buffer.read();
    buffer.clone_data()
  }

  pub fn clear_buffer(&self) {
    let buffer = self.buffer.read();
    buffer.clear();
  }

  pub fn fill_buffer(&self, value: f32) {
    let buffer = self.buffer.read();
    buffer.fill(value);
  }

  pub fn apply_gain(&self, gain: f32) {
    let buffer = self.buffer.read();
    buffer.apply_gain(gain);
  }

  pub fn get_buffer_level(&self) -> f32 {
    let buffer = self.buffer.read();
    buffer.get_rms()
  }

  pub fn get_buffer_peak(&self) -> f32 {
    let buffer = self.buffer.read();
    buffer.get_peak()
  }
}

#[derive(Debug, Clone)]
pub struct StreamStats {
  pub id: String,
  pub name: String,
  pub stream_type: StreamType,
  pub sample_rate: u32,
  pub channels: u16,
  pub buffer_size: usize,
  pub device_id: Option<String>,
  pub latency: Option<u32>,
  pub state: StreamState,
}

impl Default for AudioStream {
  fn default() -> Self {
    Self::new(crate::audio_engine::StreamConfig::default(), 44100, 2, 512)
  }
}

impl Default for StreamState {
  fn default() -> Self {
    StreamState::Stopped
  }
}

impl Default for StreamType {
  fn default() -> Self {
    StreamType::Output
  }
}
