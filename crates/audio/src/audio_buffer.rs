use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub id: String,
    pub data: Arc<RwLock<Vec<f32>>>,
    pub channels: u16,
    pub sample_rate: u32,
    pub length: usize,
    pub format: AudioFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    F32,
    F64,
    I16,
    I24,
    I32,
}

impl AudioBuffer {
    pub fn new(channels: u16, sample_rate: u32, length: usize, format: AudioFormat) -> Self {
        let data_size = length * channels as usize;
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(vec![0.0; data_size])),
            channels,
            sample_rate,
            length,
            format,
        }
    }

    pub fn from_samples(samples: Vec<f32>, channels: u16, sample_rate: u32, format: AudioFormat) -> Self {
        let length = samples.len() / channels as usize;
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(samples)),
            channels,
            sample_rate,
            length,
            format,
        }
    }

    pub fn get_sample(&self, channel: u16, frame: usize) -> Option<f32> {
        if channel >= self.channels || frame >= self.length {
            return None;
        }

        let data = self.data.read();
        let index = frame * self.channels as usize + channel as usize;
        data.get(index).copied()
    }

    pub fn set_sample(&self, channel: u16, frame: usize, value: f32) -> bool {
        if channel >= self.channels || frame >= self.length {
            return false;
        }

        let mut data = self.data.write();
        let index = frame * self.channels as usize + channel as usize;
        if let Some(sample) = data.get_mut(index) {
            *sample = value;
            true
        } else {
            false
        }
    }

    pub fn get_channel_data(&self, channel: u16) -> Option<Vec<f32>> {
        if channel >= self.channels {
            return None;
        }

        let data = self.data.read();
        let mut channel_data = Vec::with_capacity(self.length);
        
        for frame in 0..self.length {
            let index = frame * self.channels as usize + channel as usize;
            if let Some(&sample) = data.get(index) {
                channel_data.push(sample);
            }
        }
        
        Some(channel_data)
    }

    pub fn set_channel_data(&self, channel: u16, channel_data: &[f32]) -> bool {
        if channel >= self.channels || channel_data.len() != self.length {
            return false;
        }

        let mut data = self.data.write();
        
        for (frame, &sample) in channel_data.iter().enumerate() {
            let index = frame * self.channels as usize + channel as usize;
            if let Some(data_sample) = data.get_mut(index) {
                *data_sample = sample;
            }
        }
        
        true
    }

    pub fn get_frame(&self, frame: usize) -> Option<Vec<f32>> {
        if frame >= self.length {
            return None;
        }

        let data = self.data.read();
        let mut frame_data = Vec::with_capacity(self.channels as usize);
        
        for channel in 0..self.channels {
            let index = frame * self.channels as usize + channel as usize;
            if let Some(&sample) = data.get(index) {
                frame_data.push(sample);
            }
        }
        
        Some(frame_data)
    }

    pub fn set_frame(&self, frame: usize, frame_data: &[f32]) -> bool {
        if frame >= self.length || frame_data.len() != self.channels as usize {
            return false;
        }

        let mut data = self.data.write();
        
        for (channel, &sample) in frame_data.iter().enumerate() {
            let index = frame * self.channels as usize + channel;
            if let Some(data_sample) = data.get_mut(index) {
                *data_sample = sample;
            }
        }
        
        true
    }

    pub fn get_slice(&self, start_frame: usize, end_frame: usize) -> Option<AudioBuffer> {
        if start_frame >= self.length || end_frame > self.length || start_frame >= end_frame {
            return None;
        }

        let slice_length = end_frame - start_frame;
        let mut slice_data = Vec::with_capacity(slice_length * self.channels as usize);
        
        let data = self.data.read();
        for frame in start_frame..end_frame {
            for channel in 0..self.channels {
                let index = frame * self.channels as usize + channel as usize;
                if let Some(&sample) = data.get(index) {
                    slice_data.push(sample);
                }
            }
        }

        Some(AudioBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(slice_data)),
            channels: self.channels,
            sample_rate: self.sample_rate,
            length: slice_length,
            format: self.format.clone(),
        })
    }

    pub fn copy(&self) -> AudioBuffer {
        let data = self.data.read();
        AudioBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(data.clone())),
            channels: self.channels,
            sample_rate: self.sample_rate,
            length: self.length,
            format: self.format.clone(),
        }
    }

    pub fn resize(&mut self, new_length: usize) {
        let mut data = self.data.write();
        data.resize(new_length * self.channels as usize, 0.0);
        self.length = new_length;
    }

    pub fn clear(&self) {
        let mut data = self.data.write();
        data.fill(0.0);
    }

    pub fn fill(&self, value: f32) {
        let mut data = self.data.write();
        data.fill(value);
    }

    pub fn apply_gain(&self, gain: f32) {
        let mut data = self.data.write();
        for sample in data.iter_mut() {
            *sample *= gain;
        }
    }

    pub fn mix_with(&self, other: &AudioBuffer, gain: f32) -> bool {
        if self.channels != other.channels || self.sample_rate != other.sample_rate || self.length != other.length {
            return false;
        }

        let mut data = self.data.write();
        let other_data = other.data.read();
        
        for (sample, &other_sample) in data.iter_mut().zip(other_data.iter()) {
            *sample += other_sample * gain;
        }
        
        true
    }

    pub fn get_duration(&self) -> f64 {
        self.length as f64 / self.sample_rate as f64
    }

    pub fn get_size_bytes(&self) -> usize {
        self.length * self.channels as usize * std::mem::size_of::<f32>()
    }

    pub fn get_peak(&self) -> f32 {
        let data = self.data.read();
        data.iter().fold(0.0f32, |acc, &sample| acc.max(sample.abs()))
    }

    pub fn get_rms(&self) -> f32 {
        let data = self.data.read();
        let sum_squares: f32 = data.iter().map(|&sample| sample * sample).sum();
        (sum_squares / data.len() as f32).sqrt()
    }

    pub fn normalize(&self) {
        let peak = self.get_peak();
        if peak > 0.0 {
            self.apply_gain(1.0 / peak);
        }
    }

    pub fn convert_format(&self, target_format: AudioFormat) -> AudioBuffer {
        let data = self.data.read();
        let converted_data = match (&self.format, &target_format) {
            (AudioFormat::F32, AudioFormat::F64) => {
                data.iter().map(|&sample| sample as f64).collect::<Vec<f64>>()
            },
            (AudioFormat::F32, AudioFormat::I16) => {
                data.iter().map(|&sample| (sample * i16::MAX as f32) as i16).collect::<Vec<i16>>()
            },
            (AudioFormat::F32, AudioFormat::I24) => {
                data.iter().map(|&sample| (sample * ((1 << 23) - 1) as f32) as i32).collect::<Vec<i32>>()
            },
            (AudioFormat::F32, AudioFormat::I32) => {
                data.iter().map(|&sample| (sample * i32::MAX as f32) as i32).collect::<Vec<i32>>()
            },
            _ => panic!("Format conversion not implemented"),
        };

        let f32_data = match target_format {
            AudioFormat::F64 => converted_data.iter().map(|&sample| sample as f32).collect(),
            AudioFormat::I16 => converted_data.iter().map(|&sample| sample as f32 / i16::MAX as f32).collect(),
            AudioFormat::I24 => converted_data.iter().map(|&sample| sample as f32 / ((1 << 23) - 1) as f32).collect(),
            AudioFormat::I32 => converted_data.iter().map(|&sample| sample as f32 / i32::MAX as f32).collect(),
            _ => data.clone(),
        };

        AudioBuffer {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(f32_data)),
            channels: self.channels,
            sample_rate: self.sample_rate,
            length: self.length,
            format: target_format,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let data = self.data.read();
        let mut bytes = Vec::with_capacity(data.len() * 4);
        
        for &sample in data.iter() {
            let sample_bytes = sample.to_le_bytes();
            bytes.extend_from_slice(&sample_bytes);
        }
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8], channels: u16, sample_rate: u32, format: AudioFormat) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() % 4 != 0 {
            return Err("Invalid byte length for f32 audio data".into());
        }

        let mut samples = Vec::with_capacity(bytes.len() / 4);
        
        for chunk in bytes.chunks_exact(4) {
            let sample = f32::from_le_bytes(chunk.try_into().unwrap());
            samples.push(sample);
        }

        let length = samples.len() / channels as usize;
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: Arc::new(RwLock::new(samples)),
            channels,
            sample_rate,
            length,
            format,
        })
    }

    pub fn clone_data(&self) -> Vec<f32> {
        let data = self.data.read();
        data.clone()
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new(2, 44100, 0, AudioFormat::F32)
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::F32
    }
}
