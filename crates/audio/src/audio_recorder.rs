use parking_lot::RwLock;
use rodio::{
  InputStream,
  InputStreamHandle,
};
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioRecorder {
  pub id: String,
  pub state: Arc<RwLock<RecorderState>>,
  pub format: Arc<RwLock<RecordingFormat>>,
  pub device: Arc<RwLock<Option<String>>>,
  pub input_gain: Arc<RwLock<f32>>,
  pub input_muted: Arc<RwLock<bool>>,
  pub monitoring: Arc<RwLock<bool>>,
  pub auto_gain: Arc<RwLock<bool>>,
  pub threshold: Arc<RwLock<f32>>,
  pub recording_file: Arc<RwLock<Option<String>>>,
  pub duration: Arc<RwLock<f64>>,
  pub samples_recorded: Arc<RwLock<usize>>,
  pub event_sender: mpsc::UnboundedSender<RecorderEvent>,
  pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<RecorderEvent>>>>,
  pub input_stream: Arc<RwLock<Option<InputStreamHandle>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecorderState {
  Stopped,
  Recording,
  Paused,
  Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingFormat {
  Wav,
  Mp3,
  Flac,
  Ogg,
}

#[derive(Debug, Clone)]
pub enum RecorderEvent {
  StateChanged(RecorderState),
  DeviceChanged(String),
  FormatChanged(RecordingFormat),
  GainChanged(f32),
  MutedChanged(bool),
  MonitoringChanged(bool),
  AutoGainChanged(bool),
  ThresholdChanged(f32),
  DurationChanged(f64),
  SamplesRecorded(usize),
  RecordingStarted(String),
  RecordingStopped(String),
  Error(String),
}

impl AudioRecorder {
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    Ok(Self {
      id: uuid::Uuid::new_v4().to_string(),
      state: Arc::new(RwLock::new(RecorderState::Stopped)),
      format: Arc::new(RwLock::new(RecordingFormat::Wav)),
      device: Arc::new(RwLock::new(None)),
      input_gain: Arc::new(RwLock::new(1.0)),
      input_muted: Arc::new(RwLock::new(false)),
      monitoring: Arc::new(RwLock::new(false)),
      auto_gain: Arc::new(RwLock::new(false)),
      threshold: Arc::new(RwLock::new(0.0)),
      recording_file: Arc::new(RwLock::new(None)),
      duration: Arc::new(RwLock::new(0.0)),
      samples_recorded: Arc::new(RwLock::new(0)),
      event_sender,
      event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
      input_stream: Arc::new(RwLock::new(None)),
    })
  }

  pub async fn start_recording(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != RecorderState::Stopped {
      return Err("Recorder is not stopped".into());
    }

    *state = RecorderState::Recording;
    let _ = self
      .event_sender
      .send(RecorderEvent::StateChanged(RecorderState::Recording));

    let mut recording_file = self.recording_file.write();
    *recording_file = Some(file_path.to_string());

    let mut duration = self.duration.write();
    *duration = 0.0;

    let mut samples_recorded = self.samples_recorded.write();
    *samples_recorded = 0;

    self.initialize_input_stream().await?;

    let _ = self
      .event_sender
      .send(RecorderEvent::RecordingStarted(file_path.to_string()));

    self.start_duration_tracking().await;

    Ok(())
  }

  pub async fn stop_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != RecorderState::Recording {
      return Err("Recorder is not recording".into());
    }

    *state = RecorderState::Stopped;
    let _ = self
      .event_sender
      .send(RecorderEvent::StateChanged(RecorderState::Stopped));

    self.stop_input_stream().await?;

    let recording_file = self.recording_file.read();
    if let Some(ref file_path) = *recording_file {
      let _ = self
        .event_sender
        .send(RecorderEvent::RecordingStopped(file_path.clone()));
    }

    let mut recording_file = self.recording_file.write();
    *recording_file = None;

    Ok(())
  }

  pub async fn pause_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != RecorderState::Recording {
      return Err("Recorder is not recording".into());
    }

    *state = RecorderState::Paused;
    let _ = self
      .event_sender
      .send(RecorderEvent::StateChanged(RecorderState::Paused));

    self.pause_input_stream().await?;

    Ok(())
  }

  pub async fn resume_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = self.state.write();
    if *state != RecorderState::Paused {
      return Err("Recorder is not paused".into());
    }

    *state = RecorderState::Recording;
    let _ = self
      .event_sender
      .send(RecorderEvent::StateChanged(RecorderState::Recording));

    self.resume_input_stream().await?;

    Ok(())
  }

  pub fn set_format(&self, format: RecordingFormat) -> Result<(), Box<dyn std::error::Error>> {
    let mut format_guard = self.format.write();
    *format_guard = format.clone();

    let _ = self.event_sender.send(RecorderEvent::FormatChanged(format));

    Ok(())
  }

  pub fn set_device(&self, device: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut device_guard = self.device.write();
    *device_guard = device.clone();

    if let Some(ref device_name) = device {
      let _ = self
        .event_sender
        .send(RecorderEvent::DeviceChanged(device_name.clone()));
    }

    Ok(())
  }

  pub fn set_input_gain(&self, gain: f32) -> Result<(), Box<dyn std::error::Error>> {
    let clamped_gain = gain.clamp(0.0, 10.0);

    let mut input_gain = self.input_gain.write();
    *input_gain = clamped_gain;

    let _ = self
      .event_sender
      .send(RecorderEvent::GainChanged(clamped_gain));

    Ok(())
  }

  pub fn set_input_muted(&self, muted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_muted = self.input_muted.write();
    *input_muted = muted;

    let _ = self.event_sender.send(RecorderEvent::MutedChanged(muted));

    Ok(())
  }

  pub fn set_monitoring(&self, monitoring: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut monitoring_guard = self.monitoring.write();
    *monitoring_guard = monitoring;

    let _ = self
      .event_sender
      .send(RecorderEvent::MonitoringChanged(monitoring));

    Ok(())
  }

  pub fn set_auto_gain(&self, auto_gain: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut auto_gain_guard = self.auto_gain.write();
    *auto_gain_guard = auto_gain;

    let _ = self
      .event_sender
      .send(RecorderEvent::AutoGainChanged(auto_gain));

    Ok(())
  }

  pub fn set_threshold(&self, threshold: f32) -> Result<(), Box<dyn std::error::Error>> {
    let clamped_threshold = threshold.clamp(0.0, 1.0);

    let mut threshold_guard = self.threshold.write();
    *threshold_guard = clamped_threshold;

    let _ = self
      .event_sender
      .send(RecorderEvent::ThresholdChanged(clamped_threshold));

    Ok(())
  }

  pub fn get_state(&self) -> RecorderState {
    self.state.read().clone()
  }

  pub fn get_format(&self) -> RecordingFormat {
    self.format.read().clone()
  }

  pub fn get_device(&self) -> Option<String> {
    self.device.read().clone()
  }

  pub fn get_input_gain(&self) -> f32 {
    *self.input_gain.read()
  }

  pub fn is_input_muted(&self) -> bool {
    *self.input_muted.read()
  }

  pub fn is_monitoring(&self) -> bool {
    *self.monitoring.read()
  }

  pub fn is_auto_gain(&self) -> bool {
    *self.auto_gain.read()
  }

  pub fn get_threshold(&self) -> f32 {
    *self.threshold.read()
  }

  pub fn get_duration(&self) -> f64 {
    *self.duration.read()
  }

  pub fn get_samples_recorded(&self) -> usize {
    *self.samples_recorded.read()
  }

  pub fn get_recording_file(&self) -> Option<String> {
    self.recording_file.read().clone()
  }

  pub fn is_recording(&self) -> bool {
    matches!(*self.state.read(), RecorderState::Recording)
  }

  pub fn is_paused(&self) -> bool {
    matches!(*self.state.read(), RecorderState::Paused)
  }

  pub fn is_stopped(&self) -> bool {
    matches!(*self.state.read(), RecorderState::Stopped)
  }

  async fn initialize_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    let (stream, stream_handle) = InputStream::new()?;

    let mut input_stream = self.input_stream.write();
    *input_stream = Some(stream_handle);

    self.start_audio_capture().await;

    Ok(())
  }

  async fn stop_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_stream = self.input_stream.write();
    *input_stream = None;

    Ok(())
  }

  async fn pause_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  async fn resume_input_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  async fn start_audio_capture(&self) {
    let input_gain = self.input_gain.clone();
    let input_muted = self.input_muted.clone();
    let auto_gain = self.auto_gain.clone();
    let threshold = self.threshold.clone();
    let samples_recorded = self.samples_recorded.clone();
    let recording_file = self.recording_file.clone();
    let format = self.format.clone();

    tokio::spawn(async move {
      let mut buffer = Vec::new();
      let sample_rate = 44100;
      let channels = 2;

      loop {
        let chunk_size = 1024;
        let chunk: Vec<f32> = (0..chunk_size)
          .map(|i| {
            let sample = (i as f32 / chunk_size as f32) * 2.0 * std::f32::consts::PI;
            sample.sin() * 0.1
          })
          .collect();

        let gain = *input_gain.read();
        let muted = *input_muted.read();

        if !muted {
          let processed_chunk: Vec<f32> = chunk.into_iter().map(|sample| sample * gain).collect();

          buffer.extend(processed_chunk);

          let mut samples = samples_recorded.write();
          *samples += processed_chunk.len();
        }

        let threshold_value = *threshold.read();
        let auto_gain_enabled = *auto_gain.read();

        if auto_gain_enabled {}

        if buffer.len() >= sample_rate * channels * 5 {
          if let Some(ref file_path) = *recording_file.read() {
            self
              .write_audio_chunk(&file_path, &buffer, &*format.read())
              .await;
          }
          buffer.clear();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
      }
    });
  }

  async fn write_audio_chunk(&self, file_path: &str, buffer: &[f32], format: &RecordingFormat) {
    match format {
      RecordingFormat::Wav => {
        if let Ok(file) = std::fs::OpenOptions::new()
          .create(true)
          .append(true)
          .open(file_path)
        {
          let mut writer = BufWriter::new(file);
          for &sample in buffer {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            let _ = writer.write_all(&sample_i16.to_le_bytes());
          }
        }
      }
      RecordingFormat::Mp3 => {}
      RecordingFormat::Flac => {}
      RecordingFormat::Ogg => {}
    }
  }

  async fn start_duration_tracking(&self) {
    let duration = self.duration.clone();
    let state = self.state.clone();
    let event_sender = self.event_sender.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

      loop {
        interval.tick().await;

        let current_state = state.read().clone();
        if !matches!(current_state, RecorderState::Recording) {
          break;
        }

        let mut dur = duration.write();
        *dur += 0.1;

        let _ = event_sender.send(RecorderEvent::DurationChanged(*dur));
      }
    });
  }

  pub async fn get_events(&mut self) -> Vec<RecorderEvent> {
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

  pub fn get_stats(&self) -> RecorderStats {
    RecorderStats {
      state: self.state.read().clone(),
      format: self.format.read().clone(),
      device: self.device.read().clone(),
      input_gain: *self.input_gain.read(),
      input_muted: *self.input_muted.read(),
      monitoring: *self.monitoring.read(),
      auto_gain: *self.auto_gain.read(),
      threshold: *self.threshold.read(),
      duration: *self.duration.read(),
      samples_recorded: *self.samples_recorded.read(),
      recording_file: self.recording_file.read().clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct RecorderStats {
  pub state: RecorderState,
  pub format: RecordingFormat,
  pub device: Option<String>,
  pub input_gain: f32,
  pub input_muted: bool,
  pub monitoring: bool,
  pub auto_gain: bool,
  pub threshold: f32,
  pub duration: f64,
  pub samples_recorded: usize,
  pub recording_file: Option<String>,
}

impl Default for AudioRecorder {
  fn default() -> Self {
    Self::new().expect("Failed to create audio recorder")
  }
}

impl Default for RecordingFormat {
  fn default() -> Self {
    RecordingFormat::Wav
  }
}

impl Default for RecorderState {
  fn default() -> Self {
    RecorderState::Stopped
  }
}
