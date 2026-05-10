use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct AudioResampler {
    pub id: String,
    pub input_sample_rate: u32,
    pub output_sample_rate: u32,
    pub channels: u16,
    pub quality: ResampleQuality,
    pub algorithm: ResampleAlgorithm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResampleQuality {
    Fast,
    Good,
    Best,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResampleAlgorithm {
    Linear,
    Cubic,
    Sinc,
    Lanczos,
}

impl AudioResampler {
    pub fn new(input_sample_rate: u32, output_sample_rate: u32, channels: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_sample_rate,
            output_sample_rate,
            channels,
            quality: ResampleQuality::Good,
            algorithm: ResampleAlgorithm::Cubic,
        }
    }

    pub fn with_quality(mut self, quality: ResampleQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_algorithm(mut self, algorithm: ResampleAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn resample(&self, input_buffer: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        if input_buffer.channels != self.channels {
            return Err("Channel mismatch".into());
        }

        let ratio = self.output_sample_rate as f64 / self.input_sample_rate as f64;
        let output_length = ((input_buffer.length as f64 * ratio) as usize).max(1);
        
        let input_data = input_buffer.clone_data();
        let mut output_data = Vec::with_capacity(output_length * self.channels as usize);
        
        match self.algorithm {
            ResampleAlgorithm::Linear => self.resample_linear(&input_data, ratio, output_length),
            ResampleAlgorithm::Cubic => self.resample_cubic(&input_data, ratio, output_length),
            ResampleAlgorithm::Sinc => self.resample_sinc(&input_data, ratio, output_length),
            ResampleAlgorithm::Lanczos => self.resample_lanczos(&input_data, ratio, output_length),
        }
        
        Ok(crate::audio_buffer::AudioBuffer::from_samples(
            output_data,
            self.channels,
            self.output_sample_rate,
            input_buffer.format.clone(),
        ))
    }

    fn resample_linear(&self, input_data: &[f32], ratio: f64, output_length: usize) -> Vec<f32> {
        let mut output_data = Vec::with_capacity(output_length * self.channels as usize);
        
        for output_frame in 0..output_length {
            let input_position = output_frame as f64 / ratio;
            let input_frame = input_position as usize;
            let fraction = input_position - input_frame as f64;
            
            for channel in 0..self.channels {
                let input_index = input_frame * self.channels as usize + channel as usize;
                
                if input_index + self.channels as usize < input_data.len() {
                    let sample1 = input_data[input_index];
                    let sample2 = input_data[input_index + self.channels as usize];
                    
                    let interpolated = sample1 + (sample2 - sample1) * fraction as f32;
                    output_data.push(interpolated);
                } else {
                    output_data.push(0.0);
                }
            }
        }
        
        output_data
    }

    fn resample_cubic(&self, input_data: &[f32], ratio: f64, output_length: usize) -> Vec<f32> {
        let mut output_data = Vec::with_capacity(output_length * self.channels as usize);
        
        for output_frame in 0..output_length {
            let input_position = output_frame as f64 / ratio;
            let input_frame = input_position as usize;
            let fraction = input_position - input_frame as f64;
            
            for channel in 0..self.channels {
                let base_index = input_frame * self.channels as usize + channel as usize;
                
                let samples = [
                    if base_index >= self.channels as usize { 
                        input_data[base_index - self.channels as usize] 
                    } else { 
                        0.0 
                    },
                    if base_index < input_data.len() { 
                        input_data[base_index] 
                    } else { 
                        0.0 
                    },
                    if base_index + self.channels as usize < input_data.len() { 
                        input_data[base_index + self.channels as usize] 
                    } else { 
                        0.0 
                    },
                    if base_index + 2 * self.channels as usize < input_data.len() { 
                        input_data[base_index + 2 * self.channels as usize] 
                    } else { 
                        0.0 
                    },
                ];
                
                let interpolated = self.cubic_interpolate(&samples, fraction as f32);
                output_data.push(interpolated);
            }
        }
        
        output_data
    }

    fn resample_sinc(&self, input_data: &[f32], ratio: f64, output_length: usize) -> Vec<f32> {
        let mut output_data = Vec::with_capacity(output_length * self.channels as usize);
        
        let window_size = match self.quality {
            ResampleQuality::Fast => 4,
            ResampleQuality::Good => 8,
            ResampleQuality::Best => 16,
        };
        
        for output_frame in 0..output_length {
            let input_position = output_frame as f64 / ratio;
            let input_frame = input_position as usize;
            let fraction = input_position - input_frame as f64;
            
            for channel in 0..self.channels {
                let mut interpolated = 0.0;
                let mut weight_sum = 0.0;
                
                for i in -(window_size / 2)..=(window_size / 2) {
                    let sample_index = input_frame as i32 + i;
                    
                    if sample_index >= 0 && sample_index < input_data.len() as i32 {
                        let input_index = sample_index as usize * self.channels as usize + channel as usize;
                        
                        if input_index < input_data.len() {
                            let x = fraction + i as f64;
                            let weight = if x == 0.0 { 1.0 } else { 
                                (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x) 
                            };
                            
                            interpolated += input_data[input_index] * weight as f32;
                            weight_sum += weight as f32;
                        }
                    }
                }
                
                if weight_sum > 0.0 {
                    interpolated /= weight_sum;
                }
                
                output_data.push(interpolated);
            }
        }
        
        output_data
    }

    fn resample_lanczos(&self, input_data: &[f32], ratio: f64, output_length: usize) -> Vec<f32> {
        let mut output_data = Vec::with_capacity(output_length * self.channels as usize);
        
        let a = match self.quality {
            ResampleQuality::Fast => 2.0,
            ResampleQuality::Good => 3.0,
            ResampleQuality::Best => 5.0,
        };
        
        for output_frame in 0..output_length {
            let input_position = output_frame as f64 / ratio;
            let input_frame = input_position as usize;
            let fraction = input_position - input_frame as f64;
            
            for channel in 0..self.channels {
                let mut interpolated = 0.0;
                let mut weight_sum = 0.0;
                
                for i in -3..=3 {
                    let sample_index = input_frame as i32 + i;
                    
                    if sample_index >= 0 && sample_index < input_data.len() as i32 {
                        let input_index = sample_index as usize * self.channels as usize + channel as usize;
                        
                        if input_index < input_data.len() {
                            let x = fraction + i as f64;
                            let weight = self.lanczos_weight(x, a);
                            
                            interpolated += input_data[input_index] * weight as f32;
                            weight_sum += weight as f32;
                        }
                    }
                }
                
                if weight_sum > 0.0 {
                    interpolated /= weight_sum;
                }
                
                output_data.push(interpolated);
            }
        }
        
        output_data
    }

    fn cubic_interpolate(&self, samples: &[f32; 4], fraction: f32) -> f32 {
        let a0 = samples[0];
        let a1 = samples[1];
        let a2 = samples[2];
        let a3 = samples[3];
        
        let a = a3 - a2 - a0 + a1;
        let b = a0 - a1 - a;
        let c = a2 - a0;
        let d = fraction;
        
        a * d * d * d + b * d * d + c * d + a1
    }

    fn lanczos_weight(&self, x: f64, a: f64) -> f64 {
        if x == 0.0 {
            1.0
        } else if x.abs() >= a {
            0.0
        } else {
            let pi_x = std::f64::consts::PI * x;
            let sin_pi_x = pi_x.sin();
            let sin_pi_x_a = (pi_x / a).sin();
            
            sin_pi_x * sin_pi_x_a / (pi_x * pi_x / a)
        }
    }

    pub fn set_input_sample_rate(&mut self, sample_rate: u32) {
        self.input_sample_rate = sample_rate;
    }

    pub fn set_output_sample_rate(&mut self, sample_rate: u32) {
        self.output_sample_rate = sample_rate;
    }

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
    }

    pub fn set_quality(&mut self, quality: ResampleQuality) {
        self.quality = quality;
    }

    pub fn set_algorithm(&mut self, algorithm: ResampleAlgorithm) {
        self.algorithm = algorithm;
    }

    pub fn get_ratio(&self) -> f64 {
        self.output_sample_rate as f64 / self.input_sample_rate as f64
    }

    pub fn is_upsampling(&self) -> bool {
        self.output_sample_rate > self.input_sample_rate
    }

    pub fn is_downsampling(&self) -> bool {
        self.output_sample_rate < self.input_sample_rate
    }

    pub fn get_quality_factor(&self) -> f32 {
        match self.quality {
            ResampleQuality::Fast => 0.5,
            ResampleQuality::Good => 0.75,
            ResampleQuality::Best => 1.0,
        }
    }

    pub fn estimate_output_length(&self, input_length: usize) -> usize {
        let ratio = self.get_ratio();
        ((input_length as f64 * ratio) as usize).max(1)
    }

    pub fn get_processing_time_estimate(&self, input_length: usize) -> std::time::Duration {
        let output_length = self.estimate_output_length(input_length);
        let samples_per_second = input_length.max(output_length) as f64;
        
        let complexity_factor = match self.algorithm {
            ResampleAlgorithm::Linear => 1.0,
            ResampleAlgorithm::Cubic => 2.0,
            ResampleAlgorithm::Sinc => 4.0,
            ResampleAlgorithm::Lanczos => 6.0,
        };
        
        let quality_factor = match self.quality {
            ResampleQuality::Fast => 0.5,
            ResampleQuality::Good => 1.0,
            ResampleQuality::Best => 2.0,
        };
        
        let processing_time_ms = (samples_per_second * complexity_factor * quality_factor) / 1000.0;
        std::time::Duration::from_millis(processing_time_ms as u64)
    }

    pub fn clone_with_new_rates(&self, input_rate: u32, output_rate: u32) -> AudioResampler {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_sample_rate: input_rate,
            output_sample_rate: output_rate,
            channels: self.channels,
            quality: self.quality.clone(),
            algorithm: self.algorithm.clone(),
        }
    }
}

impl Default for AudioResampler {
    fn default() -> Self {
        Self::new(44100, 48000, 2)
    }
}

impl Default for ResampleQuality {
    fn default() -> Self {
        ResampleQuality::Good
    }
}

impl Default for ResampleAlgorithm {
    fn default() -> Self {
        ResampleAlgorithm::Cubic
    }
}
