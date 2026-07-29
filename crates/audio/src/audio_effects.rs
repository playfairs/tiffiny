use parking_lot::RwLock;
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffect {
  pub id: String,
  pub name: String,
  pub effect_type: EffectType,
  pub parameters: Arc<RwLock<std::collections::HashMap<String, f32>>>,
  pub enabled: Arc<RwLock<bool>>,
  pub bypass: Arc<RwLock<bool>>,
  pub wet_dry_mix: Arc<RwLock<f32>>,
  pub preset_manager: Arc<RwLock<EffectPresetManager>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectType {
  Reverb,
  Delay,
  Chorus,
  Flanger,
  Phaser,
  Distortion,
  Overdrive,
  Compressor,
  Limiter,
  Gate,
  Equalizer,
  Filter,
  PitchShifter,
  TimeStretch,
  BitCrusher,
  RingModulator,
  Tremolo,
  Vibrato,
  Panner,
  StereoEnhancer,
  Custom(String),
}

#[derive(Debug, Clone)]
pub struct EffectPreset {
  pub id: String,
  pub name: String,
  pub description: String,
  pub parameters: std::collections::HashMap<String, f32>,
  pub effect_type: EffectType,
}

#[derive(Debug, Clone)]
pub struct EffectPresetManager {
  pub presets: Arc<RwLock<Vec<EffectPreset>>>,
  pub user_presets: Arc<RwLock<Vec<EffectPreset>>>,
}

#[derive(Debug, Clone)]
pub struct EffectChain {
  pub id: String,
  pub name: String,
  pub effects: Arc<RwLock<Vec<Arc<AudioEffect>>>>,
  pub enabled: Arc<RwLock<bool>>,
  pub bypass: Arc<RwLock<bool>>,
}

impl AudioEffect {
  pub fn new(id: String, name: String, effect_type: EffectType) -> Self {
    let mut effect = Self {
      id,
      name,
      effect_type: effect_type.clone(),
      parameters: Arc::new(RwLock::new(std::collections::HashMap::new())),
      enabled: Arc::new(RwLock::new(true)),
      bypass: Arc::new(RwLock::new(false)),
      wet_dry_mix: Arc::new(RwLock::new(0.5)),
      preset_manager: Arc::new(RwLock::new(EffectPresetManager::new())),
    };

    effect.set_default_parameters();
    effect
  }

  pub async fn process(
    &self,
    input_buffer: &mut crate::audio_buffer::AudioBuffer,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if !*self.enabled.read() || *self.bypass.read() {
      return Ok(());
    }

    let wet_dry_mix = *self.wet_dry_mix.read();
    let dry_buffer = input_buffer.copy();

    match self.effect_type {
      EffectType::Reverb => self.process_reverb(input_buffer),
      EffectType::Delay => self.process_delay(input_buffer),
      EffectType::Chorus => self.process_chorus(input_buffer),
      EffectType::Flanger => self.process_flanger(input_buffer),
      EffectType::Phaser => self.process_phaser(input_buffer),
      EffectType::Distortion => self.process_distortion(input_buffer),
      EffectType::Overdrive => self.process_overdrive(input_buffer),
      EffectType::Compressor => self.process_compressor(input_buffer),
      EffectType::Limiter => self.process_limiter(input_buffer),
      EffectType::Gate => self.process_gate(input_buffer),
      EffectType::Equalizer => self.process_equalizer(input_buffer),
      EffectType::Filter => self.process_filter(input_buffer),
      EffectType::PitchShifter => self.process_pitch_shifter(input_buffer),
      EffectType::TimeStretch => self.process_time_stretch(input_buffer),
      EffectType::BitCrusher => self.process_bit_crusher(input_buffer),
      EffectType::RingModulator => self.process_ring_modulator(input_buffer),
      EffectType::Tremolo => self.process_tremolo(input_buffer),
      EffectType::Vibrato => self.process_vibrato(input_buffer),
      EffectType::Panner => self.process_panner(input_buffer),
      EffectType::StereoEnhancer => self.process_stereo_enhancer(input_buffer),
      EffectType::Custom(_) => self.process_custom(input_buffer),
    }

    self.mix_wet_dry(input_buffer, &dry_buffer, wet_dry_mix);

    Ok(())
  }

  fn process_reverb(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let room_size = parameters.get("room_size").copied().unwrap_or(0.5);
    let damping = parameters.get("damping").copied().unwrap_or(0.5);
    let pre_delay = parameters.get("pre_delay").copied().unwrap_or(0.03);
    let diffusion = parameters.get("diffusion").copied().unwrap_or(0.8);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    let sample_rate = buffer.sample_rate as f32;
    let pre_delay_samples = (pre_delay * sample_rate) as usize;

    for i in pre_delay_samples..data.len() {
      let mut reverb_sample = 0.0;

      for delay_ms in &[20, 30, 45, 67] {
        let delay_samples = (delay_ms as f32 * sample_rate / 1000.0) as usize;
        if i >= delay_samples {
          let reflection = data[i - delay_samples] * diffusion;
          reverb_sample += reflection;
        }
      }

      let late_delay = (room_size * sample_rate * 0.1) as usize;
      if i >= late_delay {
        let late_sample = processed_data[i - late_delay] * damping;
        reverb_sample += late_sample;
      }

      processed_data[i] = reverb_sample;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_delay(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let delay_time = parameters.get("delay_time").copied().unwrap_or(0.3);
    let feedback = parameters.get("feedback").copied().unwrap_or(0.4);
    let filter_cutoff = parameters.get("filter_cutoff").copied().unwrap_or(1000.0);

    let sample_rate = buffer.sample_rate as f32;
    let delay_samples = (delay_time * sample_rate) as usize;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in delay_samples..data.len() {
      let delayed_sample = processed_data[i - delay_samples] * feedback;

      let filter_coeff = 2.0 * std::f32::consts::PI * filter_cutoff / sample_rate;
      let filtered_sample = delayed_sample * filter_coeff;

      processed_data[i] = data[i] + filtered_sample;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_chorus(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(1.5);
    let depth = parameters.get("depth").copied().unwrap_or(0.02);
    let voices = parameters.get("voices").copied().unwrap_or(3.0) as usize;

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for voice in 0..voices {
      let voice_rate = rate * (1.0 + voice as f32 * 0.1);
      let voice_depth = depth * (1.0 + voice as f32 * 0.05);
      let phase_offset = (voice as f32 * 2.0 * std::f32::consts::PI / voices as f32);

      for i in 0..data.len() {
        let time = i as f32 / sample_rate;
        let modulation =
          voice_depth * (2.0 * std::f32::consts::PI * voice_rate * time + phase_offset).sin();
        let delay_samples = (modulation * sample_rate) as usize;

        if i >= delay_samples {
          processed_data[i] += data[i - delay_samples] / voices as f32;
        }
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

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let time = i as f32 / sample_rate;
      let modulation = depth * (2.0 * std::f32::consts::PI * rate * time).sin();
      let delay_samples = (modulation * sample_rate) as usize;

      if i >= delay_samples {
        let delayed_sample = processed_data[i - delay_samples] * feedback;
        processed_data[i] = data[i] + delayed_sample;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_phaser(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(1.0);
    let depth = parameters.get("depth").copied().unwrap_or(0.8);
    let feedback = parameters.get("feedback").copied().unwrap_or(0.7);
    let stages = parameters.get("stages").copied().unwrap_or(4.0) as usize;

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let time = i as f32 / sample_rate;
      let lfo = (2.0 * std::f32::consts::PI * rate * time).sin();
      let phase_shift = (depth * lfo + 1.0) * std::f32::consts::PI;

      let mut allpass_output = data[i];
      for _ in 0..stages {
        allpass_output = self.allpass_filter(allpass_output, phase_shift);
      }

      processed_data[i] = data[i] * (1.0 - feedback) + allpass_output * feedback;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_distortion(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let drive = parameters.get("drive").copied().unwrap_or(5.0);
    let tone = parameters.get("tone").copied().unwrap_or(0.5);
    let level = parameters.get("level").copied().unwrap_or(0.5);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for sample in data.iter().enumerate() {
      let driven = sample.1 * drive;

      let distorted = if driven > 0.0 {
        1.0 - (-driven).exp()
      } else {
        -1.0 + driven.exp()
      };

      let tone_filtered = distorted * (1.0 - tone) + distorted * tone;

      processed_data[sample.0] = tone_filtered * level;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_overdrive(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let drive = parameters.get("drive").copied().unwrap_or(3.0);
    let tone = parameters.get("tone").copied().unwrap_or(0.5);
    let level = parameters.get("level").copied().unwrap_or(0.5);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for sample in data.iter().enumerate() {
      let driven = sample.1 * drive;

      let overdriven = if driven > 0.0 {
        driven / (1.0 + driven.abs())
      } else {
        driven / (1.0 + driven.abs())
      };

      let tone_filtered = overdriven * (1.0 - tone) + overdriven * tone;

      processed_data[sample.0] = tone_filtered * level;
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
    let makeup_gain = parameters.get("makeup_gain").copied().unwrap_or(0.0);

    let threshold_linear = 10.0f32.powf(threshold / 20.0);
    let sample_rate = buffer.sample_rate as f32;
    let attack_coeff = (-1.0 / (attack * sample_rate)).exp();
    let release_coeff = (-1.0 / (release * sample_rate)).exp();

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
        let gain = gain_reduction / envelope * 10.0f32.powf(makeup_gain / 20.0);
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_limiter(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let threshold = parameters.get("threshold").copied().unwrap_or(-1.0);
    let release = parameters.get("release").copied().unwrap_or(0.01);

    let threshold_linear = 10.0f32.powf(threshold / 20.0);
    let sample_rate = buffer.sample_rate as f32;
    let release_coeff = (-1.0 / (release * sample_rate)).exp();

    let data = buffer.data.read();
    let mut processed_data = data.clone();
    let mut gain = 1.0;

    for &sample in data.iter() {
      let input_level = sample.abs();

      if input_level * gain > threshold_linear {
        gain = threshold_linear / input_level;
      } else {
        gain = gain + (1.0 - gain) * release_coeff;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_gate(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let threshold = parameters.get("threshold").copied().unwrap_or(-40.0);
    let attack = parameters.get("attack").copied().unwrap_or(0.001);
    let hold = parameters.get("hold").copied().unwrap_or(0.1);
    let release = parameters.get("release").copied().unwrap_or(0.1);

    let threshold_linear = 10.0f32.powf(threshold / 20.0);
    let sample_rate = buffer.sample_rate as f32;
    let attack_coeff = (-1.0 / (attack * sample_rate)).exp();
    let release_coeff = (-1.0 / (release * sample_rate)).exp();
    let hold_samples = (hold * sample_rate) as usize;

    let data = buffer.data.read();
    let mut processed_data = data.clone();
    let mut envelope = 0.0;
    let mut gate_open = false;
    let mut hold_counter = 0;

    for &sample in data.iter() {
      let input_level = sample.abs();

      if input_level > threshold_linear {
        gate_open = true;
        hold_counter = hold_samples;
      }

      if hold_counter > 0 {
        hold_counter -= 1;
      } else {
        gate_open = false;
      }

      if gate_open {
        envelope = input_level + (envelope - input_level) * attack_coeff;
      } else {
        envelope = envelope * release_coeff;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_equalizer(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();

    let bands = vec![
      ("low_freq", 200.0),
      ("low_mid_freq", 800.0),
      ("mid_freq", 2000.0),
      ("high_mid_freq", 6000.0),
      ("high_freq", 12000.0),
    ];

    for (band_name, center_freq) in bands {
      let gain = parameters
        .get(&format!("{}_gain", band_name))
        .copied()
        .unwrap_or(0.0);
      let q = parameters
        .get(&format!("{}_q", band_name))
        .copied()
        .unwrap_or(1.0);
    }
  }

  fn process_filter(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let cutoff = parameters.get("cutoff").copied().unwrap_or(1000.0);
    let resonance = parameters.get("resonance").copied().unwrap_or(0.5);
    let filter_type = parameters.get("filter_type").copied().unwrap_or(0.0);

    let sample_rate = buffer.sample_rate as f32;
    let nyquist = sample_rate * 0.5;
    let normalized_cutoff = (cutoff / nyquist).min(0.99);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    match filter_type as i32 {
      0 => self.process_lowpass(
        &mut processed_data,
        normalized_cutoff,
        resonance,
        sample_rate,
      ),
      1 => self.process_highpass(
        &mut processed_data,
        normalized_cutoff,
        resonance,
        sample_rate,
      ),
      2 => self.process_bandpass(
        &mut processed_data,
        normalized_cutoff,
        resonance,
        sample_rate,
      ),
      _ => {}
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_pitch_shifter(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let pitch_shift = parameters.get("pitch_shift").copied().unwrap_or(0.0);
    let window_size = parameters.get("window_size").copied().unwrap_or(1024.0) as usize;

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

  fn process_bit_crusher(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let bit_depth = parameters.get("bit_depth").copied().unwrap_or(8.0);
    let sample_rate = parameters.get("sample_rate").copied().unwrap_or(11025.0);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    let quantization_levels = 2.0f32.powf(bit_depth) - 1.0;

    for sample in data.iter().enumerate() {
      let quantized = (sample.1 * quantization_levels).round() / quantization_levels;

      let sample_index = sample.0;
      let target_sample_rate = sample_rate;
      let original_sample_rate = buffer.sample_rate as f32;

      if sample_index as f32 % (original_sample_rate / target_sample_rate) < 1.0 {
        processed_data[sample_index] = quantized;
      } else {
        processed_data[sample_index] = processed_data[sample_index.saturating_sub(1)];
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_ring_modulator(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let frequency = parameters.get("frequency").copied().unwrap_or(440.0);
    let depth = parameters.get("depth").copied().unwrap_or(1.0);

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for (i, &sample) in data.iter().enumerate() {
      let time = i as f32 / sample_rate;
      let modulator = (2.0 * std::f32::consts::PI * frequency * time).sin();
      processed_data[i] = sample * (1.0 + depth * modulator);
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_tremolo(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(4.0);
    let depth = parameters.get("depth").copied().unwrap_or(0.5);

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for (i, &sample) in data.iter().enumerate() {
      let time = i as f32 / sample_rate;
      let modulation = 1.0 + depth * (2.0 * std::f32::consts::PI * rate * time).sin();
      processed_data[i] = sample * modulation;
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_vibrato(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let rate = parameters.get("rate").copied().unwrap_or(5.0);
    let depth = parameters.get("depth").copied().unwrap_or(0.02);

    let sample_rate = buffer.sample_rate as f32;
    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in 0..data.len() {
      let time = i as f32 / sample_rate;
      let modulation = depth * (2.0 * std::f32::consts::PI * rate * time).sin();
      let delay_samples = (modulation * sample_rate) as usize;

      if i >= delay_samples {
        processed_data[i] = data[i - delay_samples];
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_panner(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let pan = parameters.get("pan").copied().unwrap_or(0.0);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    let left_gain = ((1.0 - pan) / 2.0).sqrt();
    let right_gain = ((1.0 + pan) / 2.0).sqrt();

    for i in (0..data.len()).step_by(2) {
      if i + 1 < data.len() {
        let left = data[i] * left_gain;
        let right = data[i + 1] * right_gain;
        processed_data[i] = left;
        processed_data[i + 1] = right;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_stereo_enhancer(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {
    let parameters = self.parameters.read();
    let width = parameters.get("width").copied().unwrap_or(1.0);

    let data = buffer.data.read();
    let mut processed_data = data.clone();

    for i in (0..data.len()).step_by(2) {
      if i + 1 < data.len() {
        let left = data[i];
        let right = data[i + 1];

        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5 * width;

        processed_data[i] = mid + side;
        processed_data[i + 1] = mid - side;
      }
    }

    let mut data = buffer.data.write();
    *data = processed_data;
  }

  fn process_custom(&self, buffer: &mut crate::audio_buffer::AudioBuffer) {}

  fn mix_wet_dry(
    &self,
    wet_buffer: &mut crate::audio_buffer::AudioBuffer,
    dry_buffer: &crate::audio_buffer::AudioBuffer,
    mix: f32,
  ) {
    let wet_data = wet_buffer.data.read();
    let dry_data = dry_buffer.data.read();
    let mut mixed_data = wet_data.clone();

    for (i, &wet_sample) in wet_data.iter().enumerate() {
      if i < dry_data.len() {
        mixed_data[i] = wet_sample * mix + dry_data[i] * (1.0 - mix);
      }
    }

    let mut wet_data = wet_buffer.data.write();
    *wet_data = mixed_data;
  }

  fn allpass_filter(&self, input: f32, phase_shift: f32) -> f32 {
    let delay = 1.0;
    let feedback = 0.5;

    input * (1.0 - feedback) + delay * feedback
  }

  fn process_lowpass(&self, data: &mut [f32], cutoff: f32, resonance: f32, sample_rate: f32) {
    let omega = 2.0 * std::f32::consts::PI * cutoff;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * resonance);

    let b0 = (1.0 - cos_omega) / 2.0;
    let b1 = 1.0 - cos_omega;
    let b2 = (1.0 - cos_omega) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    let b0_norm = b0 / a0;
    let b1_norm = b1 / a0;
    let b2_norm = b2 / a0;
    let a1_norm = a1 / a0;
    let a2_norm = a2 / a0;

    let mut x1 = 0.0;
    let mut x2 = 0.0;
    let mut y1 = 0.0;
    let mut y2 = 0.0;

    for i in 0..data.len() {
      let output = b0_norm * data[i] + b1_norm * x1 + b2_norm * x2 - a1_norm * y1 - a2_norm * y2;

      x2 = x1;
      x1 = data[i];
      y2 = y1;
      y1 = output;

      data[i] = output;
    }
  }

  fn process_highpass(&self, data: &mut [f32], cutoff: f32, resonance: f32, sample_rate: f32) {
    let omega = 2.0 * std::f32::consts::PI * cutoff;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * resonance);

    let b0 = (1.0 + cos_omega) / 2.0;
    let b1 = -(1.0 + cos_omega);
    let b2 = (1.0 + cos_omega) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    self.apply_biquad(data, b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0);
  }

  fn process_bandpass(&self, data: &mut [f32], cutoff: f32, resonance: f32, sample_rate: f32) {
    let omega = 2.0 * std::f32::consts::PI * cutoff;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * resonance);

    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    self.apply_biquad(data, b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0);
  }

  fn apply_biquad(&self, data: &mut [f32], b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) {
    let mut x1 = 0.0;
    let mut x2 = 0.0;
    let mut y1 = 0.0;
    let mut y2 = 0.0;

    for i in 0..data.len() {
      let output = b0 * data[i] + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;

      x2 = x1;
      x1 = data[i];
      y2 = y1;
      y1 = output;

      data[i] = output;
    }
  }

  fn set_default_parameters(&self) {
    let mut parameters = self.parameters.write();

    match self.effect_type {
      EffectType::Reverb => {
        parameters.insert("room_size".to_string(), 0.5);
        parameters.insert("damping".to_string(), 0.5);
        parameters.insert("pre_delay".to_string(), 0.03);
        parameters.insert("diffusion".to_string(), 0.8);
      }
      EffectType::Delay => {
        parameters.insert("delay_time".to_string(), 0.3);
        parameters.insert("feedback".to_string(), 0.4);
        parameters.insert("filter_cutoff".to_string(), 1000.0);
      }
      EffectType::Chorus => {
        parameters.insert("rate".to_string(), 1.5);
        parameters.insert("depth".to_string(), 0.02);
        parameters.insert("voices".to_string(), 3.0);
      }
      EffectType::Flanger => {
        parameters.insert("rate".to_string(), 0.5);
        parameters.insert("depth".to_string(), 0.005);
        parameters.insert("feedback".to_string(), 0.7);
      }
      EffectType::Phaser => {
        parameters.insert("rate".to_string(), 1.0);
        parameters.insert("depth".to_string(), 0.8);
        parameters.insert("feedback".to_string(), 0.7);
        parameters.insert("stages".to_string(), 4.0);
      }
      EffectType::Distortion => {
        parameters.insert("drive".to_string(), 5.0);
        parameters.insert("tone".to_string(), 0.5);
        parameters.insert("level".to_string(), 0.5);
      }
      EffectType::Overdrive => {
        parameters.insert("drive".to_string(), 3.0);
        parameters.insert("tone".to_string(), 0.5);
        parameters.insert("level".to_string(), 0.5);
      }
      EffectType::Compressor => {
        parameters.insert("threshold".to_string(), -20.0);
        parameters.insert("ratio".to_string(), 4.0);
        parameters.insert("attack".to_string(), 0.001);
        parameters.insert("release".to_string(), 0.1);
        parameters.insert("makeup_gain".to_string(), 0.0);
      }
      EffectType::Limiter => {
        parameters.insert("threshold".to_string(), -1.0);
        parameters.insert("release".to_string(), 0.01);
      }
      EffectType::Gate => {
        parameters.insert("threshold".to_string(), -40.0);
        parameters.insert("attack".to_string(), 0.001);
        parameters.insert("hold".to_string(), 0.1);
        parameters.insert("release".to_string(), 0.1);
      }
      EffectType::Equalizer => {
        for band in ["low", "low_mid", "mid", "high_mid", "high"] {
          parameters.insert(format!("{}_gain", band), 0.0);
          parameters.insert(format!("{}_q", band), 1.0);
        }
      }
      EffectType::Filter => {
        parameters.insert("cutoff".to_string(), 1000.0);
        parameters.insert("resonance".to_string(), 0.5);
        parameters.insert("filter_type".to_string(), 0.0);
      }
      EffectType::PitchShifter => {
        parameters.insert("pitch_shift".to_string(), 0.0);
        parameters.insert("window_size".to_string(), 1024.0);
      }
      EffectType::TimeStretch => {
        parameters.insert("stretch_factor".to_string(), 1.0);
      }
      EffectType::BitCrusher => {
        parameters.insert("bit_depth".to_string(), 8.0);
        parameters.insert("sample_rate".to_string(), 11025.0);
      }
      EffectType::RingModulator => {
        parameters.insert("frequency".to_string(), 440.0);
        parameters.insert("depth".to_string(), 1.0);
      }
      EffectType::Tremolo => {
        parameters.insert("rate".to_string(), 4.0);
        parameters.insert("depth".to_string(), 0.5);
      }
      EffectType::Vibrato => {
        parameters.insert("rate".to_string(), 5.0);
        parameters.insert("depth".to_string(), 0.02);
      }
      EffectType::Panner => {
        parameters.insert("pan".to_string(), 0.0);
      }
      EffectType::StereoEnhancer => {
        parameters.insert("width".to_string(), 1.0);
      }
      EffectType::Custom(_) => {}
    }
  }

  pub fn set_parameter(&self, name: &str, value: f32) {
    let mut parameters = self.parameters.write();
    parameters.insert(name.to_string(), value);
  }

  pub fn get_parameter(&self, name: &str) -> Option<f32> {
    let parameters = self.parameters.read();
    parameters.get(name).copied()
  }

  pub fn set_enabled(&self, enabled: bool) {
    let mut enabled_state = self.enabled.write();
    *enabled_state = enabled;
  }

  pub fn set_bypass(&self, bypass: bool) {
    let mut bypass_state = self.bypass.write();
    *bypass_state = bypass;
  }

  pub fn set_wet_dry_mix(&self, mix: f32) {
    let mut wet_dry_mix = self.wet_dry_mix.write();
    *wet_dry_mix = mix.clamp(0.0, 1.0);
  }

  pub fn is_enabled(&self) -> bool {
    *self.enabled.read()
  }

  pub fn is_bypassed(&self) -> bool {
    *self.bypass.read()
  }

  pub fn get_wet_dry_mix(&self) -> f32 {
    *self.wet_dry_mix.read()
  }

  pub fn get_parameters(&self) -> std::collections::HashMap<String, f32> {
    self.parameters.read().clone()
  }

  pub fn save_preset(&self, name: String, description: String) -> EffectPreset {
    EffectPreset {
      id: uuid::Uuid::new_v4().to_string(),
      name,
      description,
      parameters: self.get_parameters(),
      effect_type: self.effect_type.clone(),
    }
  }

  pub fn load_preset(&self, preset: &EffectPreset) {
    let mut parameters = self.parameters.write();
    *parameters = preset.parameters.clone();
  }

  pub fn clone_effect(&self) -> AudioEffect {
    let mut new_effect = Self::new(
      uuid::Uuid::new_v4().to_string(),
      self.name.clone(),
      self.effect_type.clone(),
    );

    let parameters = self.parameters.read();
    *new_effect.parameters.write() = parameters.clone();

    new_effect
  }
}

impl EffectPresetManager {
  pub fn new() -> Self {
    Self {
      presets: Arc::new(RwLock::new(Vec::new())),
      user_presets: Arc::new(RwLock::new(Vec::new())),
    }
  }

  pub fn add_preset(&self, preset: EffectPreset) {
    let mut presets = self.presets.write();
    presets.push(preset);
  }

  pub fn add_user_preset(&self, preset: EffectPreset) {
    let mut user_presets = self.user_presets.write();
    user_presets.push(preset);
  }

  pub fn get_presets(&self) -> Vec<EffectPreset> {
    self.presets.read().clone()
  }

  pub fn get_user_presets(&self) -> Vec<EffectPreset> {
    self.user_presets.read().clone()
  }

  pub fn find_preset(&self, name: &str) -> Option<EffectPreset> {
    let presets = self.presets.read();
    presets.iter().find(|p| p.name == name).cloned()
  }

  pub fn find_user_preset(&self, name: &str) -> Option<EffectPreset> {
    let user_presets = self.user_presets.read();
    user_presets.iter().find(|p| p.name == name).cloned()
  }
}

impl EffectChain {
  pub fn new(id: String, name: String) -> Self {
    Self {
      id,
      name,
      effects: Arc::new(RwLock::new(Vec::new())),
      enabled: Arc::new(RwLock::new(true)),
      bypass: Arc::new(RwLock::new(false)),
    }
  }

  pub async fn process(
    &self,
    input_buffer: &mut crate::audio_buffer::AudioBuffer,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if !*self.enabled.read() || *self.bypass.read() {
      return Ok(());
    }

    let effects = self.effects.read();

    for effect in effects.iter() {
      effect.process(input_buffer).await?;
    }

    Ok(())
  }

  pub fn add_effect(&self, effect: Arc<AudioEffect>) {
    let mut effects = self.effects.write();
    effects.push(effect);
  }

  pub fn remove_effect(&self, effect_id: &str) -> Option<Arc<AudioEffect>> {
    let mut effects = self.effects.write();
    let index = effects.iter().position(|e| e.id == effect_id);
    if let Some(index) = index {
      Some(effects.remove(index))
    } else {
      None
    }
  }

  pub fn move_effect(&self, from_index: usize, to_index: usize) -> bool {
    let mut effects = self.effects.write();

    if from_index < effects.len() && to_index < effects.len() {
      let effect = effects.remove(from_index);
      effects.insert(to_index, effect);
      true
    } else {
      false
    }
  }

  pub fn get_effects(&self) -> Vec<Arc<AudioEffect>> {
    self.effects.read().clone()
  }

  pub fn set_enabled(&self, enabled: bool) {
    let mut enabled_state = self.enabled.write();
    *enabled_state = enabled;
  }

  pub fn set_bypass(&self, bypass: bool) {
    let mut bypass_state = self.bypass.write();
    *bypass_state = bypass;
  }

  pub fn is_enabled(&self) -> bool {
    *self.enabled.read()
  }

  pub fn is_bypassed(&self) -> bool {
    *self.bypass.read()
  }
}

impl Default for AudioEffect {
  fn default() -> Self {
    Self::new(
      uuid::Uuid::new_v4().to_string(),
      "Default Effect".to_string(),
      EffectType::Gain,
    )
  }
}

impl Default for EffectPresetManager {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for EffectChain {
  fn default() -> Self {
    Self::new(
      uuid::Uuid::new_v4().to_string(),
      "Default Chain".to_string(),
    )
  }
}

impl Default for EffectPreset {
  fn default() -> Self {
    Self {
      id: uuid::Uuid::new_v4().to_string(),
      name: "Default Preset".to_string(),
      description: "Default preset".to_string(),
      parameters: std::collections::HashMap::new(),
      effect_type: EffectType::Gain,
    }
  }
}
