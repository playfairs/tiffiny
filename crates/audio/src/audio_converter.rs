use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioConverter {
    pub id: String,
    pub input_format: Arc<RwLock<crate::audio_format::AudioFormatInfo>>,
    pub output_format: Arc<RwLock<crate::audio_format::AudioFormatInfo>>,
    pub converter: Arc<RwLock<Option<Arc<dyn AudioFormatConverter>>>>,
    pub event_sender: mpsc::UnboundedSender<ConverterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ConverterEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ConverterEvent {
    ConversionStarted,
    ConversionProgress(f32),
    ConversionCompleted(String),
    ConversionFailed(String),
    FormatChanged(crate::audio_format::AudioFormatInfo),
}

#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub current_frame: usize,
    pub total_frames: usize,
    pub current_time: f64,
    pub total_time: f64,
    pub processing_speed: f64,
    pub eta: Option<std::time::Duration>,
}

pub trait AudioFormatConverter: Send + Sync {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>>;
    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo);
    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo);
    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo>;
    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo>;
    fn reset(&mut self);
    fn get_conversion_info(&self) -> ConversionInfo;
}

#[derive(Debug, Clone)]
pub struct ConversionInfo {
    pub input_format: crate::audio_format::AudioFormatInfo,
    pub output_format: crate::audio_format::AudioFormatInfo,
    pub conversion_type: ConversionType,
    pub quality_loss: Option<f32>,
    pub processing_time: Option<std::time::Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionType {
    SampleRateConversion,
    BitDepthConversion,
    ChannelConversion,
    CodecConversion,
    ContainerConversion,
    Complex,
}

impl AudioConverter {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_format: Arc::new(RwLock::new(crate::audio_format::AudioFormatInfo::new())),
            output_format: Arc::new(RwLock::new(crate::audio_format::AudioFormatInfo::new())),
            converter: Arc::new(RwLock::new(None)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn set_input_format(&self, format: crate::audio_format::AudioFormatInfo) {
        let mut input_format = self.input_format.write();
        *input_format = format;
        
        let _ = self.event_sender.send(ConverterEvent::FormatChanged(format));
        
        self.update_converter();
    }

    pub fn set_output_format(&self, format: crate::audio_format::AudioFormatInfo) {
        let mut output_format = self.output_format.write();
        *output_format = format;
        
        let _ = self.event_sender.send(ConverterEvent::FormatChanged(format));
        
        self.update_converter();
    }

    pub async fn convert(&self, input_buffer: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        
        let mut converter = self.converter.write();
        
        if let Some(ref mut conv) = *converter {
            conv.set_input_format(&self.input_format.read());
            conv.set_output_format(&self.output_format.read());
            
            let result = conv.convert(input_buffer);
            
            match result {
                Ok(output_buffer) => {
                    let _ = self.event_sender.send(ConverterEvent::ConversionCompleted("Conversion successful".to_string()));
                    Ok(output_buffer)
                },
                Err(e) => {
                    let error_msg = format!("Conversion failed: {}", e);
                    let _ = self.event_sender.send(ConverterEvent::ConversionFailed(error_msg.clone()));
                    Err(e)
                },
            }
        } else {
            Err("No converter available".into())
        }
    }

    pub async fn convert_with_progress<F>(&self, input_buffer: &crate::audio_buffer::AudioBuffer, progress_callback: F) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>>
    where
        F: Fn(ConversionProgress) + Send + Sync,
    {
        let _ = self.event_sender.send(ConverterEvent::ConversionStarted);
        
        let total_frames = input_buffer.length;
        let mut converter = self.converter.write();
        
        if let Some(ref mut conv) = *converter {
            conv.set_input_format(&self.input_format.read());
            conv.set_output_format(&self.output_format.read());
            
            let chunk_size = 1024;
            let mut output_buffer = crate::audio_buffer::AudioBuffer::new(
                self.output_format.read().channels,
                self.output_format.read().sample_rate,
                input_buffer.length,
                self.output_format.read().sample_format.clone(),
            );
            
            for chunk_start in (0..input_buffer.length).step_by(chunk_size) {
                let chunk_end = (chunk_start + chunk_size).min(input_buffer.length);
                let chunk = input_buffer.get_slice(chunk_start, chunk_end)
                    .ok_or("Invalid chunk range")?;
                
                let converted_chunk = conv.convert(&chunk)?;
                
                for frame in 0..converted_chunk.length {
                    for channel in 0..converted_chunk.channels {
                        if let Some(sample) = converted_chunk.get_sample(channel, frame) {
                            output_buffer.set_sample(channel, chunk_start + frame, sample);
                        }
                    }
                }
                
                let progress = ConversionProgress {
                    current_frame: chunk_end,
                    total_frames,
                    current_time: chunk_end as f64 / input_buffer.sample_rate as f64,
                    total_time: total_frames as f64 / input_buffer.sample_rate as f64,
                    processing_speed: chunk_size as f64 / 0.1,
                    eta: if chunk_end > 0 {
                        let remaining_frames = total_frames - chunk_end;
                        Some(std::time::Duration::from_secs_f64(remaining_frames as f64 / (chunk_end as f64 / 0.1)))
                    } else {
                        None
                    },
                };
                
                progress_callback(progress.clone());
                
                let progress_percent = (chunk_end as f32 / total_frames as f32) * 100.0;
                let _ = self.event_sender.send(ConverterEvent::ConversionProgress(progress_percent));
                
                tokio::task::yield_now().await;
            }
            
            let _ = self.event_sender.send(ConverterEvent::ConversionCompleted("Conversion successful".to_string()));
            Ok(output_buffer)
        } else {
            Err("No converter available".into())
        }
    }

    pub fn get_input_format(&self) -> crate::audio_format::AudioFormatInfo {
        self.input_format.read().clone()
    }

    pub fn get_output_format(&self) -> crate::audio_format::AudioFormatInfo {
        self.output_format.read().clone()
    }

    pub fn get_conversion_type(&self) -> ConversionType {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if input_format.sample_rate != output_format.sample_rate {
            ConversionType::SampleRateConversion
        } else if input_format.sample_format != output_format.sample_format {
            ConversionType::BitDepthConversion
        } else if input_format.channels != output_format.channels {
            ConversionType::ChannelConversion
        } else if input_format.codec != output_format.codec {
            ConversionType::CodecConversion
        } else if input_format.container != output_format.container {
            ConversionType::ContainerConversion
        } else {
            ConversionType::Complex
        }
    }

    pub fn is_conversion_needed(&self) -> bool {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        input_format.sample_rate != output_format.sample_rate ||
        input_format.sample_format != output_format.sample_format ||
        input_format.channels != output_format.channels ||
        input_format.codec != output_format.codec ||
        input_format.container != output_format.container
    }

    pub fn estimate_quality_loss(&self) -> Option<f32> {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        if input_format.codec.is_lossless() && output_format.codec.is_lossless() {
            None
        } else if input_format.codec.is_lossless() && output_format.codec.is_lossy() {
            Some(100.0)
        } else if input_format.codec.is_lossy() && output_format.codec.is_lossless() {
            Some(0.0)
        } else {
            self.estimate_lossy_quality_loss(&input_format, &output_format)
        }
    }

    fn estimate_lossy_quality_loss(&self, input: &crate::audio_format::AudioFormatInfo, output: &crate::audio_format::AudioFormatInfo) -> Option<f32> {
        if let (Some(input_bitrate), Some(output_bitrate)) = (input.bit_rate, output.bit_rate) {
            if output_bitrate >= input_bitrate {
                Some(0.0)
            } else {
                Some(((input_bitrate - output_bitrate) as f32 / input_bitrate as f32) * 100.0)
            }
        } else {
            Some(10.0)
        }
    }

    fn update_converter(&self) {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        
        let converter = self.create_converter(&input_format, &output_format);
        let mut converter_guard = self.converter.write();
        *converter_guard = Some(converter);
    }

    fn create_converter(&self, input: &crate::audio_format::AudioFormatInfo, output: &crate::audio_format::AudioFormatInfo) -> Arc<dyn AudioFormatConverter> {
        if input.sample_rate != output.sample_rate {
            Arc::new(SampleRateConverter::new(input.clone(), output.clone()))
        } else if input.sample_format != output.sample_format {
            Arc::new(BitDepthConverter::new(input.clone(), output.clone()))
        } else if input.channels != output.channels {
            Arc::new(ChannelConverter::new(input.clone(), output.clone()))
        } else if input.codec != output.codec {
            Arc::new(CodecConverter::new(input.clone(), output.clone()))
        } else {
            Arc::new(PassthroughConverter::new(input.clone()))
        }
    }

    pub async fn get_events(&mut self) -> Vec<ConverterEvent> {
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

    pub fn get_conversion_stats(&self) -> ConversionStats {
        let input_format = self.input_format.read();
        let output_format = self.output_format.read();
        let conversion_type = self.get_conversion_type();
        let quality_loss = self.estimate_quality_loss();
        
        ConversionStats {
            conversion_type,
            input_format: input_format.clone(),
            output_format: output_format.clone(),
            quality_loss,
            is_conversion_needed: self.is_conversion_needed(),
            estimated_processing_time: self.estimate_processing_time(&input_format, &output_format),
        }
    }

    fn estimate_processing_time(&self, input: &crate::audio_format::AudioFormatInfo, output: &crate::audio_format::AudioFormatInfo) -> Option<std::time::Duration> {
        let complexity_factor = match self.get_conversion_type() {
            ConversionType::SampleRateConversion => 1.5,
            ConversionType::BitDepthConversion => 1.2,
            ConversionType::ChannelConversion => 1.8,
            ConversionType::CodecConversion => 3.0,
            ConversionType::ContainerConversion => 1.1,
            ConversionType::Complex => 4.0,
        };
        
        let base_time_ms = 1000.0 * complexity_factor;
        
        if let Some(duration) = input.duration {
            Some(std::time::Duration::from_millis((duration * base_time_ms) as u64))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub conversion_type: ConversionType,
    pub input_format: crate::audio_format::AudioFormatInfo,
    pub output_format: crate::audio_format::AudioFormatInfo,
    pub quality_loss: Option<f32>,
    pub is_conversion_needed: bool,
    pub estimated_processing_time: Option<std::time::Duration>,
}

struct SampleRateConverter {
    input_format: crate::audio_format::AudioFormatInfo,
    output_format: crate::audio_format::AudioFormatInfo,
    resampler: crate::audio_resampler::AudioResampler,
}

impl SampleRateConverter {
    fn new(input: crate::audio_format::AudioFormatInfo, output: crate::audio_format::AudioFormatInfo) -> Self {
        let resampler = crate::audio_resampler::AudioResampler::new(
            input.sample_rate,
            output.sample_rate,
            input.channels,
        );
        
        Self {
            input_format: input,
            output_format: output,
            resampler,
        }
    }
}

impl AudioFormatConverter for SampleRateConverter {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        self.resampler.resample(input)
    }

    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.input_format = format.clone();
        self.resampler.set_input_sample_rate(format.sample_rate);
        self.resampler.set_channels(format.channels);
    }

    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.output_format = format.clone();
        self.resampler.set_output_sample_rate(format.sample_rate);
        self.resampler.set_channels(format.channels);
    }

    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::SampleRateConversion,
            quality_loss: None,
            processing_time: None,
        }
    }
}

struct BitDepthConverter {
    input_format: crate::audio_format::AudioFormatInfo,
    output_format: crate::audio_format::AudioFormatInfo,
}

impl BitDepthConverter {
    fn new(input: crate::audio_format::AudioFormatInfo, output: crate::audio_format::AudioFormatInfo) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl AudioFormatConverter for BitDepthConverter {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        let input_data = input.clone_data();
        let mut output_data = Vec::with_capacity(input_data.len());
        
        for &sample in &input_data {
            let converted = self.output_format.sample_format.convert_from_f32(sample);
            output_data.push(converted);
        }
        
        Ok(crate::audio_buffer::AudioBuffer::from_samples(
            output_data,
            self.output_format.channels,
            self.output_format.sample_rate,
            self.output_format.sample_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::BitDepthConversion,
            quality_loss: self.estimate_quality_loss(),
            processing_time: None,
        }
    }

    fn estimate_quality_loss(&self) -> Option<f32> {
        let input_bits = self.input_format.sample_format.bits_per_sample();
        let output_bits = self.output_format.sample_format.bits_per_sample();
        
        if output_bits >= input_bits {
            None
        } else {
            Some(((input_bits - output_bits) as f32 / input_bits as f32) * 100.0)
        }
    }
}

struct ChannelConverter {
    input_format: crate::audio_format::AudioFormatInfo,
    output_format: crate::audio_format::AudioFormatInfo,
}

impl ChannelConverter {
    fn new(input: crate::audio_format::AudioFormatInfo, output: crate::audio_format::AudioFormatInfo) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl AudioFormatConverter for ChannelConverter {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        let input_data = input.clone_data();
        let mut output_data = Vec::with_capacity(input.length * self.output_format.channels as usize);
        
        for frame in 0..input.length {
            for channel in 0..self.output_format.channels {
                let sample = if channel < input.channels {
                    input.get_sample(channel, frame).unwrap_or(0.0)
                } else if input.channels > 0 {
                    input.get_sample(input.channels - 1, frame).unwrap_or(0.0)
                } else {
                    0.0
                };
                
                output_data.push(sample);
            }
        }
        
        Ok(crate::audio_buffer::AudioBuffer::from_samples(
            output_data,
            self.output_format.channels,
            self.output_format.sample_rate,
            self.output_format.sample_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::ChannelConversion,
            quality_loss: None,
            processing_time: None,
        }
    }
}

struct CodecConverter {
    input_format: crate::audio_format::AudioFormatInfo,
    output_format: crate::audio_format::AudioFormatInfo,
}

impl CodecConverter {
    fn new(input: crate::audio_format::AudioFormatInfo, output: crate::audio_format::AudioFormatInfo) -> Self {
        Self {
            input_format: input,
            output_format: output,
        }
    }
}

impl AudioFormatConverter for CodecConverter {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        Ok(crate::audio_buffer::AudioBuffer::from_samples(
            input.clone_data(),
            self.output_format.channels,
            self.output_format.sample_rate,
            self.output_format.sample_format.clone(),
        ))
    }

    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.input_format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.output_format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.input_format.clone())
    }

    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.output_format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.input_format.clone(),
            output_format: self.output_format.clone(),
            conversion_type: ConversionType::CodecConversion,
            quality_loss: self.estimate_quality_loss(),
            processing_time: None,
        }
    }

    fn estimate_quality_loss(&self) -> Option<f32> {
        if self.input_format.codec.is_lossless() && self.output_format.codec.is_lossy() {
            Some(100.0)
        } else if self.input_format.codec.is_lossy() && self.output_format.codec.is_lossless() {
            Some(0.0)
        } else {
            Some(10.0)
        }
    }
}

struct PassthroughConverter {
    format: crate::audio_format::AudioFormatInfo,
}

impl PassthroughConverter {
    fn new(format: crate::audio_format::AudioFormatInfo) -> Self {
        Self {
            format,
        }
    }
}

impl AudioFormatConverter for PassthroughConverter {
    fn convert(&mut self, input: &crate::audio_buffer::AudioBuffer) -> Result<crate::audio_buffer::AudioBuffer, Box<dyn std::error::Error>> {
        Ok(input.copy())
    }

    fn set_input_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.format = format.clone();
    }

    fn set_output_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
        self.format = format.clone();
    }

    fn get_input_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.format.clone())
    }

    fn get_output_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
        Some(self.format.clone())
    }

    fn reset(&mut self) {
    }

    fn get_conversion_info(&self) -> ConversionInfo {
        ConversionInfo {
            input_format: self.format.clone(),
            output_format: self.format.clone(),
            conversion_type: ConversionType::Complex,
            quality_loss: None,
            processing_time: None,
        }
    }
}

impl Default for AudioConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            current_frame: 0,
            total_frames: 0,
            current_time: 0.0,
            total_time: 0.0,
            processing_speed: 0.0,
            eta: None,
        }
    }
}

impl Default for ConversionInfo {
    fn default() -> Self {
        Self {
            input_format: crate::audio_format::AudioFormatInfo::new(),
            output_format: crate::audio_format::AudioFormatInfo::new(),
            conversion_type: ConversionType::Complex,
            quality_loss: None,
            processing_time: None,
        }
    }
}

impl Default for ConversionStats {
    fn default() -> Self {
        Self {
            conversion_type: ConversionType::Complex,
            input_format: crate::audio_format::AudioFormatInfo::new(),
            output_format: crate::audio_format::AudioFormatInfo::new(),
            quality_loss: None,
            is_conversion_needed: false,
            estimated_processing_time: None,
        }
    }
}
