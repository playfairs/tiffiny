use parking_lot::RwLock;
use rustfft::{
  FftPlanner,
  num_complex::Complex,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AudioAnalyzer {
  pub id: String,
  pub sample_rate: u32,
  pub fft_size: usize,
  pub window_function: WindowFunction,
  pub overlap_ratio: f32,
  pub frequency_resolution: f32,
  pub time_resolution: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowFunction {
  Hann,
  Hamming,
  Blackman,
  Bartlett,
  Rectangular,
}

#[derive(Debug, Clone)]
pub struct SpectrumAnalysis {
  pub frequencies: Vec<f32>,
  pub magnitudes: Vec<f32>,
  pub phases: Vec<f32>,
  pub peak_frequency: f32,
  pub peak_magnitude: f32,
  pub rms_level: f32,
  pub peak_level: f32,
  pub crest_factor: f32,
}

#[derive(Debug, Clone)]
pub struct WaveformAnalysis {
  pub samples: Vec<f32>,
  pub rms_level: f32,
  pub peak_level: f32,
  pub zero_crossings: usize,
  pub dc_offset: f32,
  pub dynamic_range: f32,
}

#[derive(Debug, Clone)]
pub struct SpectralAnalysis {
  pub spectrogram: Vec<Vec<f32>>,
  pub time_bins: usize,
  pub frequency_bins: usize,
  pub frequency_resolution: f32,
  pub time_resolution: f32,
}

impl AudioAnalyzer {
  pub fn new(sample_rate: u32, fft_size: usize) -> Self {
    Self {
      id: uuid::Uuid::new_v4().to_string(),
      sample_rate,
      fft_size,
      window_function: WindowFunction::Hann,
      overlap_ratio: 0.5,
      frequency_resolution: sample_rate as f32 / fft_size as f32,
      time_resolution: fft_size as f32 / sample_rate as f32,
    }
  }

  pub fn analyze_spectrum(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
  ) -> SpectrumAnalysis {
    let samples = audio_buffer.clone_data();
    let windowed_samples = self.apply_window(&samples);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(self.fft_size);

    let mut fft_buffer: Vec<Complex<f32>> = windowed_samples
      .into_iter()
      .take(self.fft_size)
      .map(|sample| Complex::new(sample, 0.0))
      .collect();

    fft.process(&mut fft_buffer);

    let mut frequencies = Vec::new();
    let mut magnitudes = Vec::new();
    let mut phases = Vec::new();

    for (i, complex) in fft_buffer.iter().take(self.fft_size / 2).enumerate() {
      let frequency = (i as f32 * self.frequency_resolution) as f32;
      let magnitude = complex.norm();
      let phase = complex.arg();

      frequencies.push(frequency);
      magnitudes.push(magnitude);
      phases.push(phase);
    }

    let (peak_frequency, peak_magnitude) = self.find_peak(&frequencies, &magnitudes);
    let rms_level = self.calculate_rms(&samples);
    let peak_level = self.calculate_peak(&samples);
    let crest_factor = if rms_level > 0.0 {
      peak_level / rms_level
    } else {
      0.0
    };

    SpectrumAnalysis {
      frequencies,
      magnitudes,
      phases,
      peak_frequency,
      peak_magnitude,
      rms_level,
      peak_level,
      crest_factor,
    }
  }

  pub fn analyze_waveform(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
  ) -> WaveformAnalysis {
    let samples = audio_buffer.clone_data();

    let rms_level = self.calculate_rms(&samples);
    let peak_level = self.calculate_peak(&samples);
    let zero_crossings = self.count_zero_crossings(&samples);
    let dc_offset = self.calculate_dc_offset(&samples);
    let dynamic_range = self.calculate_dynamic_range(&samples);

    WaveformAnalysis {
      samples,
      rms_level,
      peak_level,
      zero_crossings,
      dc_offset,
      dynamic_range,
    }
  }

  pub fn analyze_spectrogram(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
  ) -> SpectralAnalysis {
    let samples = audio_buffer.clone_data();
    let hop_size = (self.fft_size as f32 * (1.0 - self.overlap_ratio)) as usize;
    let num_frames = (samples.len() - self.fft_size) / hop_size + 1;

    let mut spectrogram = Vec::with_capacity(num_frames);

    for frame_idx in 0..num_frames {
      let start_idx = frame_idx * hop_size;
      let end_idx = start_idx + self.fft_size;

      if end_idx <= samples.len() {
        let frame_samples = &samples[start_idx..end_idx];
        let windowed_samples = self.apply_window_slice(frame_samples);

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_size);

        let mut fft_buffer: Vec<Complex<f32>> = windowed_samples
          .iter()
          .map(|&sample| Complex::new(sample, 0.0))
          .collect();

        fft.process(&mut fft_buffer);

        let magnitudes: Vec<f32> = fft_buffer
          .iter()
          .take(self.fft_size / 2)
          .map(|complex| complex.norm())
          .collect();

        spectrogram.push(magnitudes);
      }
    }

    SpectralAnalysis {
      spectrogram,
      time_bins: num_frames,
      frequency_bins: self.fft_size / 2,
      frequency_resolution: self.frequency_resolution,
      time_resolution: hop_size as f32 / self.sample_rate as f32,
    }
  }

  pub fn detect_pitch(&self, audio_buffer: &crate::audio_buffer::AudioBuffer) -> Option<f32> {
    let samples = audio_buffer.clone_data();

    let autocorr = self.autocorrelation(&samples);

    let mut peak_index = 0;
    let mut peak_value = 0.0;

    for i in (self.sample_rate as usize / 800)..(self.sample_rate as usize / 80) {
      if autocorr[i] > peak_value {
        peak_value = autocorr[i];
        peak_index = i;
      }
    }

    if peak_index > 0 {
      Some(self.sample_rate as f32 / peak_index as f32)
    } else {
      None
    }
  }

  pub fn detect_onsets(&self, audio_buffer: &crate::audio_buffer::AudioBuffer) -> Vec<usize> {
    let samples = audio_buffer.clone_data();
    let mut onsets = Vec::new();

    let hop_size = 512;
    let num_frames = (samples.len() - hop_size) / hop_size + 1;

    let mut previous_spectrum = None;

    for frame_idx in 0..num_frames {
      let start_idx = frame_idx * hop_size;
      let end_idx = (start_idx + hop_size).min(samples.len());

      if end_idx > start_idx {
        let frame_samples = &samples[start_idx..end_idx];
        let windowed_samples = self.apply_window_slice(frame_samples);

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(hop_size);

        let mut fft_buffer: Vec<Complex<f32>> = windowed_samples
          .iter()
          .map(|&sample| Complex::new(sample, 0.0))
          .collect();

        fft.process(&mut fft_buffer);

        let current_spectrum: Vec<f32> = fft_buffer
          .iter()
          .take(hop_size / 2)
          .map(|complex| complex.norm())
          .collect();

        if let Some(ref prev_spectrum) = previous_spectrum {
          let flux: f32 = current_spectrum
            .iter()
            .zip(prev_spectrum.iter())
            .map(|(curr, prev)| (curr - prev).max(0.0))
            .sum();

          if flux > 0.1 {
            onsets.push(frame_idx * hop_size);
          }
        }

        previous_spectrum = Some(current_spectrum);
      }
    }

    onsets
  }

  pub fn calculate_loudness(&self, audio_buffer: &crate::audio_buffer::AudioBuffer) -> f32 {
    let samples = audio_buffer.clone_data();

    let rms = self.calculate_rms(&samples);
    let loudness_db = 20.0 * rms.log10();

    loudness_db - 0.691
  }

  pub fn calculate_spectral_centroid(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
  ) -> f32 {
    let spectrum = self.analyze_spectrum(audio_buffer);

    let weighted_sum: f32 = spectrum
      .frequencies
      .iter()
      .zip(spectrum.magnitudes.iter())
      .map(|(freq, mag)| freq * mag)
      .sum();

    let magnitude_sum: f32 = spectrum.magnitudes.iter().sum();

    if magnitude_sum > 0.0 {
      weighted_sum / magnitude_sum
    } else {
      0.0
    }
  }

  pub fn calculate_spectral_rolloff(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
    rolloff_percent: f32,
  ) -> f32 {
    let spectrum = self.analyze_spectrum(audio_buffer);

    let total_energy: f32 = spectrum.magnitudes.iter().map(|mag| mag * mag).sum();
    let rolloff_threshold = total_energy * rolloff_percent;

    let mut cumulative_energy = 0.0;

    for (freq, &mag) in spectrum.frequencies.iter().zip(spectrum.magnitudes.iter()) {
      cumulative_energy += mag * mag;
      if cumulative_energy >= rolloff_threshold {
        return *freq;
      }
    }

    spectrum.frequencies.last().copied().unwrap_or(0.0)
  }

  pub fn calculate_zero_crossing_rate(
    &self,
    audio_buffer: &crate::audio_buffer::AudioBuffer,
  ) -> f32 {
    let samples = audio_buffer.clone_data();
    let zero_crossings = self.count_zero_crossings(&samples);

    zero_crossings as f32 / samples.len() as f32 * self.sample_rate as f32
  }

  fn apply_window(&self, samples: &[f32]) -> Vec<f32> {
    let window = self.generate_window(samples.len().min(self.fft_size));
    samples
      .iter()
      .take(self.fft_size)
      .zip(window.iter())
      .map(|(sample, window_coeff)| sample * window_coeff)
      .collect()
  }

  fn apply_window_slice(&self, samples: &[f32]) -> Vec<f32> {
    let window = self.generate_window(samples.len());
    samples
      .iter()
      .zip(window.iter())
      .map(|(sample, window_coeff)| sample * window_coeff)
      .collect()
  }

  fn generate_window(&self, size: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(size);

    for i in 0..size {
      let n = i as f32;
      let n_minus_1 = (size - 1) as f32;

      let coeff = match self.window_function {
        WindowFunction::Hann => 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n / n_minus_1).cos()),
        WindowFunction::Hamming => 0.54 - 0.46 * (2.0 * std::f32::consts::PI * n / n_minus_1).cos(),
        WindowFunction::Blackman => {
          let a0 = 0.42;
          let a1 = 0.5;
          let a2 = 0.08;
          a0 - a1 * (2.0 * std::f32::consts::PI * n / n_minus_1).cos()
            + a2 * (4.0 * std::f32::consts::PI * n / n_minus_1).cos()
        }
        WindowFunction::Bartlett => 1.0 - 2.0 * (n - n_minus_1 / 2.0).abs() / n_minus_1,
        WindowFunction::Rectangular => 1.0,
      };

      window.push(coeff);
    }

    window
  }

  fn find_peak(&self, frequencies: &[f32], magnitudes: &[f32]) -> (f32, f32) {
    if magnitudes.is_empty() {
      return (0.0, 0.0);
    }

    let mut peak_index = 0;
    let mut peak_magnitude = magnitudes[0];

    for (i, &magnitude) in magnitudes.iter().enumerate() {
      if magnitude > peak_magnitude {
        peak_magnitude = magnitude;
        peak_index = i;
      }
    }

    let peak_frequency = frequencies.get(peak_index).copied().unwrap_or(0.0);

    (peak_frequency, peak_magnitude)
  }

  fn calculate_rms(&self, samples: &[f32]) -> f32 {
    if samples.is_empty() {
      return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
    (sum_squares / samples.len() as f32).sqrt()
  }

  fn calculate_peak(&self, samples: &[f32]) -> f32 {
    samples
      .iter()
      .fold(0.0f32, |acc, &sample| acc.max(sample.abs()))
  }

  fn count_zero_crossings(&self, samples: &[f32]) -> usize {
    if samples.len() < 2 {
      return 0;
    }

    let mut crossings = 0;
    for i in 1..samples.len() {
      if (samples[i] >= 0.0 && samples[i - 1] < 0.0) || (samples[i] < 0.0 && samples[i - 1] >= 0.0)
      {
        crossings += 1;
      }
    }

    crossings
  }

  fn calculate_dc_offset(&self, samples: &[f32]) -> f32 {
    if samples.is_empty() {
      return 0.0;
    }

    samples.iter().sum::<f32>() / samples.len() as f32
  }

  fn calculate_dynamic_range(&self, samples: &[f32]) -> f32 {
    if samples.is_empty() {
      return 0.0;
    }

    let min_sample = samples
      .iter()
      .fold(f32::INFINITY, |acc, &sample| acc.min(sample));
    let max_sample = samples
      .iter()
      .fold(f32::NEG_INFINITY, |acc, &sample| acc.max(sample));

    max_sample - min_sample
  }

  fn autocorrelation(&self, samples: &[f32]) -> Vec<f32> {
    let size = samples.len();
    let mut autocorr = vec![0.0; size];

    for lag in 0..size {
      let mut sum = 0.0;
      for i in 0..size - lag {
        sum += samples[i] * samples[i + lag];
      }
      autocorr[lag] = sum / (size - lag) as f32;
    }

    autocorr
  }

  pub fn set_window_function(&mut self, window_function: WindowFunction) {
    self.window_function = window_function;
  }

  pub fn set_overlap_ratio(&mut self, overlap_ratio: f32) {
    self.overlap_ratio = overlap_ratio.clamp(0.0, 0.9);
    self.time_resolution =
      self.fft_size as f32 * (1.0 - self.overlap_ratio) / self.sample_rate as f32;
  }

  pub fn set_fft_size(&mut self, fft_size: usize) {
    self.fft_size = fft_size;
    self.frequency_resolution = self.sample_rate as f32 / fft_size as f32;
    self.time_resolution = fft_size as f32 / self.sample_rate as f32;
  }

  pub fn get_frequency_resolution(&self) -> f32 {
    self.frequency_resolution
  }

  pub fn get_time_resolution(&self) -> f32 {
    self.time_resolution
  }
}

impl Default for AudioAnalyzer {
  fn default() -> Self {
    Self::new(44100, 2048)
  }
}

impl Default for WindowFunction {
  fn default() -> Self {
    WindowFunction::Hann
  }
}
