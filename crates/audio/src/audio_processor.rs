use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioProcessor {
  pub id: String,
  pub name: String,
  pub processor_type: ProcessorType,
  pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
  pub enabled: Arc<RwLock<bool>>,
  pub sample_rate: u32,
  pub channels: u16,
  pub buffer_size: usize,
  pub event_sender: Option<mpsc::UnboundedSender<ProcessorEvent>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorType {
  Gain,
  Equalizer,
  Compressor,
  Limiter,
  Reverb,
  Delay,
  Chorus,
  Flanger,
  Distortion,
  Filter,
  Pitch,
  TimeStretch,
  Custom(String),
}

#[derive(Debug, Clone)]
pub enum ProcessorEvent {
  ParameterChanged(String, f32),
  EnabledChanged(bool),
  Error(String),
}

impl AudioProcessor {
  pub fn new(
    id: String,
    name: String,
    processor_type: ProcessorType,
    sample_rate: u32,
    channels: u16,
    buffer_size: usize,
  ) -> Self {
    Self {
      id,
      name,
      processor_type,
      parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
      enabled: Arc::new(RwLock::new(true)),
      sample_rate,
      channels,
      buffer_size,
      event_sender: None,
    }
  }

  pub async fn process(
    &self,
    input_buffer: &mut crate::audio_buffer::AudioBuffer,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if !*self.enabled.read() {
      return Ok(());
    }

    match self.processor_type {
      ProcessorType::Gain => self.process_gain(input_buffer),
      ProcessorType::Equalizer => self.process_equalizer(input_buffer),
      ProcessorType::Compressor => self.process_compressor(input_buffer),
      ProcessorType::Limiter => self.process_limiter(input_buffer),
      ProcessorType::Reverb => self.process_reverb(input_buffer),
      ProcessorType::Delay => self.process_delay(input_buffer),
      ProcessorType::Chorus => self.process_chorus(input_buffer),
      ProcessorType::Flanger => self.process_flanger(input_buffer),
      ProcessorType::Distortion => self.process_distortion(input_buffer),
      ProcessorType::Filter => self.process_filter(input_buffer),
      ProcessorType::Pitch => self.process_pitch(input_buffer),
      ProcessorType::TimeStretch => self.process_time_stretch(input_buffer),
      ProcessorType::Custom(_) => self.process_custom(input_buffer),
    }

    Ok(())
  }

  fn process_gain(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let gain = parameters.get("gain").copied().unwrap_or(1.0);
    buffer.apply_gain(gain);
  }

  fn process_equalizer(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let low_gain = parameters.get("low_gain").copied().unwrap_or(1.0);
    let mid_gain = parameters.get("mid_gain").copied().unwrap_or(1.0);
    let high_gain = parameters.get("high_gain").copied().unwrap_or(1.0);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for (i, &sample) in data.iter().enumerate() {
      let frequency = (i as f32 / buffer.sample_rate as f32) * 2.0 * std::f32::consts::PI;
      let low_factor = if frequency < 500.0 { low_gain } else { 1.0 };
      let mid_factor = if frequency >= 500.0 && frequency < 4000.0 {
        mid_gain
      } else {
        1.0
      };
      let high_factor = if frequency >= 4000.0 { high_gain } else { 1.0 };

      processed_data[i] = sample * low_factor * mid_factor * high_factor;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_compressor(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let threshold = parameters.get("threshold").copied().unwrap_or(-20.0);
    let ratio = parameters.get("ratio").copied().unwrap_or(4.0);
    let attack = parameters.get("attack").copied().unwrap_or(0.001);
    let release = parameters.get("release").copied().unwrap_or(0.1);

    let threshold_linear = 10.0f32.powf(threshold / 20.0);
    let attack_coeff = (-1.0 / (attack * buffer.sample_rate as f32)).exp();
    let release_coeff = (-1.0 / (release * buffer.sample_rate as f32)).exp();

    let data = buffer.data.read();
    let mut processed_data = data.clone();
    let mut envelope = 0.0;

    for &sample in data.iter() {
      let input_level = sample.abs();

      if input_level > envelope {
        envelope = input_level + (envelope - input_level) * attack_coeff;
      } else {
        envelope = input_level + (envelope - input_level) * release_coeff;
      }

      if envelope > threshold_linear {
        let gain_reduction = threshold_linear + (envelope - threshold_linear) / ratio;
        let gain = gain_reduction / envelope;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_limiter(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let threshold = parameters.get("threshold").copied().unwrap_or(-1.0);
    let threshold_linear = 10.0f32.powf(threshold / 20.0);

    let mut data = buffer.data.write();
    for sample in data.iter_mut() {
      if *sample > threshold_linear {
        *sample = threshold_linear;
      } else if *sample < -threshold_linear {
        *sample = -threshold_linear;
      }
    }
  }

  fn process_reverb(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let room_size = parameters.get("room_size").copied().unwrap_or(0.5);
    let damping = parameters.get("damping").copied().unwrap_or(0.5);
    let wet_level = parameters.get("wet_level").copied().unwrap_or(0.3);
    let dry_level = parameters.get("dry_level").copied().unwrap_or(0.7);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let delay_samples = (room_size * buffer.sample_rate as f32 * 0.1) as usize;
      if i >= delay_samples {
        let delayed_sample = data[i - delay_samples] * damping;
        processed_data[i] = data[i] * dry_level + delayed_sample * wet_level;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_delay(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let delay_time = parameters.get("delay_time").copied().unwrap_or(0.3);
    let feedback = parameters.get("feedback").copied().unwrap_or(0.4);
    let wet_level = parameters.get("wet_level").copied().unwrap_or(0.5);
    let dry_level = parameters.get("dry_level").copied().unwrap_or(0.5);

    let delay_samples = (delay_time * buffer.sample_rate as f32) as usize;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      if i >= delay_samples {
        let delayed_sample = processed_data[i - delay_samples] * feedback;
        processed_data[i] = data[i] * dry_level + delayed_sample * wet_level;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_chorus(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(1.5);
    let depth = parameters.get("depth").copied().unwrap_or(0.02);
    let delay = parameters.get("delay").copied().unwrap_or(0.025);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let phase = (i as f32 / buffer.sample_rate as f32) * rate * 2.0 * std::f32::consts::PI;
      let modulation = depth * phase.sin();
      let delay_samples = ((delay + modulation) * buffer.sample_rate as f32) as usize;

      if i >= delay_samples {
        processed_data[i] = (data[i] + data[i - delay_samples]) * 0.5;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_flanger(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(0.5);
    let depth = parameters.get("depth").copied().unwrap_or(0.005);
    let feedback = parameters.get("feedback").copied().unwrap_or(0.7);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let phase = (i as f32 / buffer.sample_rate as f32) * rate * 2.0 * std::f32::consts::PI;
      let modulation = depth * phase.sin();
      let delay_samples = (modulation * buffer.sample_rate as f32) as usize;

      if i >= delay_samples {
        let delayed_sample = processed_data[i - delay_samples] * feedback;
        processed_data[i] = data[i] + delayed_sample;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_distortion(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let drive = parameters.get("drive").copied().unwrap_or(5.0);
    let level = parameters.get("level").copied().unwrap_or(0.5);

    let mut data = buffer.data.write();
    for sample in data.iter_mut() {
      let driven = *sample * drive;
      let distorted = if driven > 0.0 {
        1.0 - (-driven).exp()
      } else {
        -1.0 + driven.exp()
      };
      *sample = distorted * level;
    }
  }

  fn process_filter(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let cutoff = parameters.get("cutoff").copied().unwrap_or(1000.0);
    let resonance = parameters.get("resonance").copied().unwrap_or(0.5);
    let filter_type = parameters.get("filter_type").copied().unwrap_or(0.0);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    let nyquist = buffer.sample_rate as f32 * 0.5;
    let normalized_cutoff = (cutoff / nyquist).min(0.99);

    for i in 1..data.len() {
      match filter_type as i32 {
        0 => {
          processed_data[i] =
            data[i] * (1.0 - normalized_cutoff) + processed_data[i - 1] * normalized_cutoff;
        }
        1 => {
          processed_data[i] = data[i] - processed_data[i - 1] * normalized_cutoff;
        }
        2 => {
          processed_data[i] =
            (data[i] - processed_data[i - 1]) * normalized_cutoff + processed_data[i - 1];
        }
        _ => processed_data[i] = data[i],
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_pitch(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let pitch_shift = parameters.get("pitch_shift").copied().unwrap_or(0.0);

    if pitch_shift.abs() < 0.001 {
      return;
    }

    let pitch_ratio = 2.0f32.powf(pitch_shift / 12.0);
    let data = buffer.data.read();
    let mut processed_data = Vec::with_capacity(data.len());

    for i in 0..data.len() {
      let source_index = (i as f32 / pitch_ratio) as usize;
      if source_index < data.len() {
        processed_data.push(data[source_index]);
      } else {
        processed_data.push(0.0);
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_time_stretch(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let stretch_factor = parameters.get("stretch_factor").copied().unwrap_or(1.0);

    if (stretch_factor - 1.0).abs() < 0.001 {
      return;
    }

    let data = buffer.data.read();
    let mut processed_data = Vec::with_capacity((data.len() as f32 / stretch_factor) as usize);

    for i in 0..data.len() {
      let source_index = (i as f32 * stretch_factor) as usize;
      if source_index < data.len() {
        processed_data.push(data[source_index]);
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_custom(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {}

  pub fn set_parameter(&self, name: &str, value: f32) {
    let mut parameters = self.parameters.write();
    parameters.insert(name.to_string(), value);

    if let Some(ref sender) = self.event_sender {
      let _ = sender.send(ProcessorEvent::ParameterChanged(name.to_string(), value));
    }
  }

  pub fn get_parameter(&self, name: &str) -> Option<f32> {
    let parameters = self.parameters.read();
    parameters.get(name).copied()
  }

  pub fn set_enabled(&self, enabled: bool) {
    let mut enabled_state = self.enabled.write();
    *enabled_state = enabled;

    if let Some(ref sender) = self.event_sender {
      let _ = sender.send(ProcessorEvent::EnabledChanged(enabled));
    }
  }

  pub fn is_enabled(&self) -> bool {
    *self.enabled.read()
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn set_channels(&mut self, channels: u16) {
    self.channels = channels;
  }

  pub fn set_buffer_size(&mut self, buffer_size: usize) {
    self.buffer_size = buffer_size;
  }

  pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
    self.parameters.read().clone()
  }

  pub fn reset(&self) {
    let mut parameters = self.parameters.write();
    parameters.clear();

    match self.processor_type {
      ProcessorType::Gain => {
        parameters.insert("gain".to_string(), 1.0);
      }
      ProcessorType::Equalizer => {
        parameters.insert("low_gain".to_string(), 1.0);
        parameters.insert("mid_gain".to_string(), 1.0);
        parameters.insert("high_gain".to_string(), 1.0);
      }
      ProcessorType::Compressor => {
        parameters.insert("threshold".to_string(), -20.0);
        parameters.insert("ratio".to_string(), 4.0);
        parameters.insert("attack".to_string(), 0.001);
        parameters.insert("release".to_string(), 0.1);
      }
      ProcessorType::Limiter => {
        parameters.insert("threshold".to_string(), -1.0);
      }
      ProcessorType::Reverb => {
        parameters.insert("room_size".to_string(), 0.5);
        parameters.insert("damping".to_string(), 0.5);
        parameters.insert("wet_level".to_string(), 0.3);
        parameters.insert("dry_level".to_string(), 0.7);
      }
      ProcessorType::Delay => {
        parameters.insert("delay_time".to_string(), 0.3);
        parameters.insert("feedback".to_string(), 0.4);
        parameters.insert("wet_level".to_string(), 0.5);
        parameters.insert("dry_level".to_string(), 0.5);
      }
      ProcessorType::Chorus => {
        parameters.insert("rate".to_string(), 1.5);
        parameters.insert("depth".to_string(), 0.02);
        parameters.insert("delay".to_string(), 0.025);
      }
      ProcessorType::Flanger => {
        parameters.insert("rate".to_string(), 0.5);
        parameters.insert("depth".to_string(), 0.005);
        parameters.insert("feedback".to_string(), 0.7);
      }
      ProcessorType::Distortion => {
        parameters.insert("drive".to_string(), 5.0);
        parameters.insert("level".to_string(), 0.5);
      }
      ProcessorType::Filter => {
        parameters.insert("cutoff".to_string(), 1000.0);
        parameters.insert("resonance".to_string(), 0.5);
        parameters.insert("filter_type".to_string(), 0.0);
      }
      ProcessorType::Pitch => {
        parameters.insert("pitch_shift".to_string(), 0.0);
      }
      ProcessorType::TimeStretch => {
        parameters.insert("stretch_factor".to_string(), 1.0);
      }
      ProcessorType::Custom(_) => {}
    }
  }

  pub fn clone_processor(&self) -> AudioProcessor {
    let mut new_processor = Self::new(
      uuid::Uuid::new_v4().to_string(),
      self.name.clone(),
      self.processor_type.clone(),
      self.sample_rate,
      self.channels,
      self.buffer_size,
    );

    let parameters = self.parameters.read();
    *new_processor.parameters.write() = parameters.clone();

    new_processor
  }
}

impl Default for AudioProcessor {
  fn default() -> Self {
    Self::new(
      uuid::Uuid::new_v4().to_string(),
      "Default Processor".to_string(),
      ProcessorType::Gain,
      44100,
      2,
      512,
    )
  }
}
