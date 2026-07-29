use parking_lot::RwLock;
use std::sync::Arc;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone)]
pub struct AudioCodec {
  pub id: String,
  pub name: String,
  pub codec_type: CodecType,
  pub supported_formats: Vec<crate::audio_format::AudioFormat>,
  pub supported_containers: Vec<crate::audio_format::AudioContainer>,
  pub encoder: Option<Arc<dyn AudioEncoder>>,
  pub decoder: Option<Arc<dyn AudioDecoder>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodecType {
  Lossless,
  Lossy,
  Hybrid,
}

#[derive(Debug, Clone)]
pub struct CodecParameters {
  pub sample_rate: Option<u32>,
  pub channels: Option<u16>,
  pub bit_rate: Option<u32>,
  pub quality: Option<f32>,
  pub compression_level: Option<u8>,
  pub vbr: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct EncodingResult {
  pub success: bool,
  pub data: Vec<u8>,
  pub format: crate::audio_format::AudioFormatInfo,
  pub samples_encoded: usize,
  pub encoding_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct DecodingResult {
  pub success: bool,
  pub buffer: crate::audio_buffer::AudioBuffer,
  pub format: crate::audio_format::AudioFormatInfo,
  pub samples_decoded: usize,
  pub decoding_time: std::time::Duration,
}

pub trait AudioEncoder: Send + Sync {
  fn encode(
    &mut self,
    input: &crate::audio_buffer::AudioBuffer,
  ) -> Result<EncodingResult, Box<dyn std::error::Error>>;
  fn flush(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
  fn set_parameters(&mut self, params: CodecParameters);
  fn get_parameters(&self) -> CodecParameters;
  fn reset(&mut self);
  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormat>;
}

pub trait AudioDecoder: Send + Sync {
  fn decode(&mut self, data: &[u8]) -> Result<DecodingResult, Box<dyn std::error::Error>>;
  fn set_format(&mut self, format: &crate::audio_format::AudioFormatInfo);
  fn get_format(&self) -> Option<crate::audio_format::AudioFormatInfo>;
  fn reset(&mut self);
  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormatInfo>;
}

impl AudioCodec {
  pub fn new(id: String, name: String, codec_type: CodecType) -> Self {
    Self {
      id,
      name,
      codec_type,
      supported_formats: Vec::new(),
      supported_containers: Vec::new(),
      encoder: None,
      decoder: None,
    }
  }

  pub fn with_encoder(mut self, encoder: Arc<dyn AudioEncoder>) -> Self {
    self.encoder = Some(encoder);
    self
  }

  pub fn with_decoder(mut self, decoder: Arc<dyn AudioDecoder>) -> Self {
    self.decoder = Some(decoder);
    self
  }

  pub fn add_supported_format(mut self, format: crate::audio_format::AudioFormat) -> Self {
    self.supported_formats.push(format);
    self
  }

  pub fn add_supported_container(mut self, container: crate::audio_format::AudioContainer) -> Self {
    self.supported_containers.push(container);
    self
  }

  pub fn supports_format(&self, format: &crate::audio_format::AudioFormat) -> bool {
    self.supported_formats.contains(format)
  }

  pub fn supports_container(&self, container: &crate::audio_format::AudioContainer) -> bool {
    self.supported_containers.contains(container)
  }

  pub fn encode(
    &mut self,
    input: &crate::audio_buffer::AudioBuffer,
    params: CodecParameters,
  ) -> Result<EncodingResult, Box<dyn std::error::Error>> {
    if let Some(ref mut encoder) = self.encoder {
      encoder.set_parameters(params);
      encoder.encode(input)
    } else {
      Err("No encoder available".into())
    }
  }

  pub fn decode(
    &mut self,
    data: &[u8],
    format: &crate::audio_format::AudioFormatInfo,
  ) -> Result<DecodingResult, Box<dyn std::error::Error>> {
    if let Some(ref mut decoder) = self.decoder {
      decoder.set_format(format);
      decoder.decode(data)
    } else {
      Err("No decoder available".into())
    }
  }

  pub fn get_optimal_parameters(
    &self,
    input: &crate::audio_buffer::AudioBuffer,
    target_bitrate: Option<u32>,
  ) -> CodecParameters {
    let mut params = CodecParameters {
      sample_rate: Some(input.sample_rate),
      channels: Some(input.channels),
      bit_rate: target_bitrate,
      quality: None,
      compression_level: None,
      vbr: None,
    };

    match self.codec_type {
      CodecType::Lossless => {
        params.compression_level = Some(8);
        params.quality = Some(1.0);
      }
      CodecType::Lossy => {
        params.quality = Some(0.8);
        params.vbr = Some(true);
        if let Some(bitrate) = target_bitrate {
          params.bit_rate = Some(bitrate);
        }
      }
      CodecType::Hybrid => {
        params.quality = Some(0.9);
        params.compression_level = Some(6);
      }
    }

    params
  }

  pub fn estimate_output_size(
    &self,
    input: &crate::audio_buffer::AudioBuffer,
    params: &CodecParameters,
  ) -> usize {
    let samples = input.length * input.channels as usize;
    let bytes_per_sample = match self.codec_type {
      CodecType::Lossless => 4,
      CodecType::Lossy => {
        if let Some(bitrate) = params.bit_rate {
          (bitrate / 8 / input.sample_rate) as usize
        } else {
          2
        }
      }
      CodecType::Hybrid => 3,
    };

    samples * bytes_per_sample
  }

  pub fn get_codec_info(&self) -> CodecInfo {
    CodecInfo {
      id: self.id.clone(),
      name: self.name.clone(),
      codec_type: self.codec_type.clone(),
      supported_formats: self.supported_formats.clone(),
      supported_containers: self.supported_containers.clone(),
      has_encoder: self.encoder.is_some(),
      has_decoder: self.decoder.is_some(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CodecInfo {
  pub id: String,
  pub name: String,
  pub codec_type: CodecType,
  pub supported_formats: Vec<crate::audio_format::AudioFormat>,
  pub supported_containers: Vec<crate::audio_format::AudioContainer>,
  pub has_encoder: bool,
  pub has_decoder: bool,
}

pub struct PcmEncoder {
  params: CodecParameters,
  format: crate::audio_format::AudioFormat,
}

impl PcmEncoder {
  pub fn new(format: crate::audio_format::AudioFormat) -> Self {
    Self {
      params: CodecParameters {
        sample_rate: None,
        channels: None,
        bit_rate: None,
        quality: None,
        compression_level: None,
        vbr: None,
      },
      format,
    }
  }
}

impl AudioEncoder for PcmEncoder {
  fn encode(
    &mut self,
    input: &crate::audio_buffer::AudioBuffer,
  ) -> Result<EncodingResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let data = input.to_bytes();

    Ok(EncodingResult {
      success: true,
      data,
      format: crate::audio_format::AudioFormatInfo::new()
        .with_format(self.format.clone())
        .with_sample_rate(input.sample_rate)
        .with_channels(input.channels),
      samples_encoded: input.length,
      encoding_time: start_time.elapsed(),
    })
  }

  fn flush(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(Vec::new())
  }

  fn set_parameters(&mut self, params: CodecParameters) {
    self.params = params;
  }

  fn get_parameters(&self) -> CodecParameters {
    self.params.clone()
  }

  fn reset(&mut self) {}

  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormat> {
    vec![self.format.clone()]
  }
}

pub struct Mp3Encoder {
  params: CodecParameters,
  quality: f32,
  bitrate: u32,
}

impl Mp3Encoder {
  pub fn new() -> Self {
    Self {
      params: CodecParameters {
        sample_rate: None,
        channels: None,
        bit_rate: Some(128000),
        quality: Some(0.8),
        compression_level: None,
        vbr: Some(true),
      },
      quality: 0.8,
      bitrate: 128000,
    }
  }
}

impl AudioEncoder for Mp3Encoder {
  fn encode(
    &mut self,
    input: &crate::audio_buffer::AudioBuffer,
  ) -> Result<EncodingResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let data = self.simulate_mp3_encoding(input);

    Ok(EncodingResult {
      success: true,
      data,
      format: crate::audio_format::AudioFormatInfo::new()
        .with_codec(crate::audio_format::AudioCodec::MP3)
        .with_container(crate::audio_format::AudioContainer::MP3)
        .with_sample_rate(input.sample_rate)
        .with_channels(input.channels)
        .with_bit_rate(self.bitrate),
      samples_encoded: input.length,
      encoding_time: start_time.elapsed(),
    })
  }

  fn flush(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(Vec::new())
  }

  fn set_parameters(&mut self, params: CodecParameters) {
    self.params = params.clone();
    if let Some(quality) = params.quality {
      self.quality = quality;
    }
    if let Some(bitrate) = params.bit_rate {
      self.bitrate = bitrate;
    }
  }

  fn get_parameters(&self) -> CodecParameters {
    self.params.clone()
  }

  fn reset(&mut self) {}

  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormat> {
    vec![
      crate::audio_format::AudioFormat::I16,
      crate::audio_format::AudioFormat::F32,
    ]
  }

  fn simulate_mp3_encoding(&self, input: &crate::audio_buffer::AudioBuffer) -> Vec<u8> {
    let data = input.to_bytes();
    let mut compressed = Vec::with_capacity(data.len() / 2);

    for chunk in data.chunks(4) {
      for &byte in chunk {
        let compressed_byte = (byte as f32 * self.quality) as u8;
        compressed.push(compressed_byte);
      }
    }

    compressed
  }
}

pub struct PcmDecoder {
  format: Option<crate::audio_format::AudioFormatInfo>,
}

impl PcmDecoder {
  pub fn new() -> Self {
    Self { format: None }
  }
}

impl AudioDecoder for PcmDecoder {
  fn decode(&mut self, data: &[u8]) -> Result<DecodingResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    if let Some(ref format) = self.format {
      let buffer = crate::audio_buffer::AudioBuffer::from_bytes(
        data,
        format.channels,
        format.sample_rate,
        format.sample_format.clone(),
      )?;

      Ok(DecodingResult {
        success: true,
        buffer,
        format: format.clone(),
        samples_decoded: buffer.length,
        decoding_time: start_time.elapsed(),
      })
    } else {
      Err("No format specified".into())
    }
  }

  fn set_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
    self.format = Some(format.clone());
  }

  fn get_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
    self.format.clone()
  }

  fn reset(&mut self) {}

  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormatInfo> {
    vec![
      crate::audio_format::AudioFormatInfo::new()
        .with_format(crate::audio_format::AudioFormat::F32)
        .with_codec(crate::audio_format::AudioCodec::PCM)
        .with_container(crate::audio_format::AudioContainer::WAV),
      crate::audio_format::AudioFormatInfo::new()
        .with_format(crate::audio_format::AudioFormat::I16)
        .with_codec(crate::audio_format::AudioCodec::PCM)
        .with_container(crate::audio_format::AudioContainer::WAV),
    ]
  }
}

pub struct Mp3Decoder {
  format: Option<crate::audio_format::AudioFormatInfo>,
}

impl Mp3Decoder {
  pub fn new() -> Self {
    Self { format: None }
  }
}

impl AudioDecoder for Mp3Decoder {
  fn decode(&mut self, data: &[u8]) -> Result<DecodingResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let (buffer, format) = self.simulate_mp3_decoding(data)?;

    Ok(DecodingResult {
      success: true,
      buffer,
      format,
      samples_decoded: buffer.length,
      decoding_time: start_time.elapsed(),
    })
  }

  fn set_format(&mut self, format: &crate::audio_format::AudioFormatInfo) {
    self.format = Some(format.clone());
  }

  fn get_format(&self) -> Option<crate::audio_format::AudioFormatInfo> {
    self.format.clone()
  }

  fn reset(&mut self) {}

  fn get_supported_formats(&self) -> Vec<crate::audio_format::AudioFormatInfo> {
    vec![
      crate::audio_format::AudioFormatInfo::new()
        .with_format(crate::audio_format::AudioFormat::F32)
        .with_codec(crate::audio_format::AudioCodec::MP3)
        .with_container(crate::audio_format::AudioContainer::MP3),
      crate::audio_format::AudioFormatInfo::new()
        .with_format(crate::audio_format::AudioFormat::I16)
        .with_codec(crate::audio_format::AudioCodec::MP3)
        .with_container(crate::audio_format::AudioContainer::MP3),
    ]
  }

  fn simulate_mp3_decoding(
    &self,
    data: &[u8],
  ) -> Result<
    (
      crate::audio_buffer::AudioBuffer,
      crate::audio_format::AudioFormatInfo,
    ),
    Box<dyn std::error::Error>,
  > {
    let format = crate::audio_format::AudioFormatInfo::new()
      .with_format(crate::audio_format::AudioFormat::F32)
      .with_codec(crate::audio_format::AudioCodec::MP3)
      .with_container(crate::audio_format::AudioContainer::MP3)
      .with_sample_rate(44100)
      .with_channels(2);

    let samples_per_channel = data.len() / 8;
    let buffer = crate::audio_buffer::AudioBuffer::new(
      2,
      44100,
      samples_per_channel,
      crate::audio_format::AudioFormat::F32,
    );

    Ok((buffer, format))
  }
}

impl Default for AudioCodec {
  fn default() -> Self {
    Self::new(
      uuid::Uuid::new_v4().to_string(),
      "Default Codec".to_string(),
      CodecType::Lossy,
    )
  }
}

impl Default for CodecParameters {
  fn default() -> Self {
    Self {
      sample_rate: None,
      channels: None,
      bit_rate: None,
      quality: None,
      compression_level: None,
      vbr: None,
    }
  }
}

impl Default for PcmEncoder {
  fn default() -> Self {
    Self::new(crate::audio_format::AudioFormat::F32)
  }
}

impl Default for Mp3Encoder {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for PcmDecoder {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for Mp3Decoder {
  fn default() -> Self {
    Self::new()
  }
}
