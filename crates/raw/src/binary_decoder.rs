use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BinaryDecoder {
    pub id: String,
    pub name: String,
    pub format: Arc<RwLock<DecoderFormat>>,
    pub source: Arc<RwLock<crate::binary_reader::BinaryReader>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<DecoderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<DecoderEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecoderFormat {
    Custom,
    JSON,
    XML,
    CSV,
    INI,
    TOML,
    YAML,
    Binary,
    Hex,
    Base64,
    Protobuf,
    MessagePack,
    CBOR,
}

#[derive(Debug, Clone)]
pub enum DecoderEvent {
    DecodeStarted,
    DecodeProgress(f32),
    DecodeCompleted(DecodeResult),
    Error(String),
    PositionChanged(u64),
}

#[derive(Debug, Clone)]
pub struct DecodeResult {
    pub success: bool,
    pub data: DecodeData,
    pub metadata: DecodeMetadata,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DecodeData {
    JSON(serde_json::Value),
    XML(String),
    CSV(Vec<Vec<String>>),
    INI(std::collections::HashMap<String, String>),
    TOML(toml::Value),
    YAML(serde_yaml::Value),
    Binary(Vec<u8>),
    Hex(Vec<u8>),
    Base64(Vec<u8>),
    Custom(std::collections::HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone)]
pub struct DecodeMetadata {
    pub format: String,
    pub size: u64,
    pub encoding: Option<String>,
    pub checksum: Option<String>,
    pub created_time: Option<std::time::SystemTime>,
    pub modified_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct DecoderConfig {
    pub format: DecoderFormat,
    pub encoding: Option<String>,
    pub strict_mode: bool,
    pub max_depth: Option<usize>,
    pub allow_comments: bool,
    pub allow_trailing_commas: bool,
    pub custom_delimiters: Option<String>,
    pub custom_quote_chars: Option<String>,
}

impl BinaryDecoder {
    pub fn new(id: String, name: String, format: DecoderFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            format: Arc::new(RwLock::new(format)),
            source: Arc::new(RwLock::new(crate::binary_reader::BinaryReader::default())),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_reader(id: String, name: String, reader: crate::binary_reader::BinaryReader, format: DecoderFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            format: Arc::new(RwLock::new(format)),
            source: Arc::new(RwLock::new(Arc::new(reader))),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn decode(&self, config: DecoderConfig) -> Result<DecodeResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(DecoderEvent::DecodeStarted);
        let start_time = std::time::Instant::now();

        let result = match config.format {
            DecoderFormat::JSON => self.decode_json(config).await,
            DecoderFormat::XML => self.decode_xml(config).await,
            DecoderFormat::CSV => self.decode_csv(config).await,
            DecoderFormat::INI => self.decode_ini(config).await,
            DecoderFormat::TOML => self.decode_toml(config).await,
            DecoderFormat::YAML => self.decode_yaml(config).await,
            DecoderFormat::Binary => self.decode_binary(config).await,
            DecoderFormat::Hex => self.decode_hex(config).await,
            DecoderFormat::Base64 => self.decode_base64(config).await,
            DecoderFormat::Protobuf => self.decode_protobuf(config).await,
            DecoderFormat::MessagePack => self.decode_messagepack(config).await,
            DecoderFormat::CBOR => self.decode_cbor(config).await,
            DecoderFormat::Custom => self.decode_custom(config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(data) => {
                let metadata = self.create_metadata(&config, &data).await;
                let _ = self.event_sender.send(DecoderEvent::DecodeCompleted(DecodeResult {
                    success: true,
                    data,
                    metadata,
                    error_message: None,
                }));
                
                Ok(DecodeResult {
                    success: true,
                    data,
                    metadata,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Decode failed: {}", e);
                let _ = self.event_sender.send(DecoderEvent::Error(error_msg.clone()));
                
                Ok(DecodeResult {
                    success: false,
                    data: DecodeData::Custom(std::collections::HashMap::new()),
                    metadata: DecodeMetadata::default(),
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn decode_json(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
Report progress
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let json_value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(value) => value,
            Err(e) => {
                if config.strict_mode {
                    return Err(format!("JSON decoding error: {}", e).into());
                } else {
                    serde_json::from_str(&content[..content.len().saturating_sub(e.position().unwrap_or(0), content.len())])?
                        .unwrap_or(serde_json::Value::Null)
                }
            },
        };

        Ok(DecodeData::JSON(json_value))
    }

    async fn decode_xml(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        Ok(DecodeData::XML(content))
    }

    async fn decode_csv(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let delimiter = config.custom_delimiters.as_deref().unwrap_or(",");
        let quote_char = config.custom_quote_chars.as_deref().unwrap_or("\"");
        let mut rows = Vec::new();
        
        for line in content.lines() {
            let mut fields = Vec::new();
            let mut current_field = String::new();
            let mut in_quotes = false;
            let mut escape_next = false;
            
            for ch in line.chars() {
                if escape_next {
                    current_field.push(ch);
                    escape_next = false;
                    continue;
                }
                
                if ch == '\\' {
                    escape_next = true;
                    continue;
                }
                
                if ch == quote_char.chars().next().unwrap_or('"') && !in_quotes {
                    in_quotes = true;
                } else if ch == quote_char.chars().next().unwrap_or('"') && in_quotes {
                    in_quotes = false;
                } else if ch == delimiter.chars().next().unwrap_or(',') && !in_quotes {
                    fields.push(current_field.clone());
                    current_field.clear();
                } else {
                    current_field.push(ch);
                }
            }
            
            fields.push(current_field);
            rows.push(fields);
        }

        Ok(DecodeData::CSV(rows))
    }

    async fn decode_ini(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let mut ini_data = std::collections::HashMap::new();
        let mut current_section = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_string();
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                if !current_section.is_empty() {
                    ini_data.insert(format!("[{}].{}", current_section, key), value.to_string());
                } else {
                    ini_data.insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(DecodeData::INI(ini_data))
    }

    async fn decode_toml(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let toml_value: toml::Value = toml::from_str(&content)?;
        Ok(DecodeData::TOML(toml_value))
    }

    async fn decode_yaml(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        Ok(DecodeData::YAML(yaml_value))
    }

    async fn decode_binary(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut data = Vec::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            data.extend_from_slice(&bytes);
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        Ok(DecodeData::Binary(data))
    }

    async fn decode_hex(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let hex_data = hex::decode(&content)?;
        Ok(DecodeData::Hex(hex_data))
    }

    async fn decode_base64(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut content = String::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            content.push_str(&String::from_utf8_lossy(&bytes));
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let base64_data = base64::decode(&content)?;
        Ok(DecodeData::Base64(base64_data))
    }

    async fn decode_protobuf(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut data = Vec::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            data.extend_from_slice(&bytes);
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("protobuf_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(DecodeData::Custom(custom_data))
    }

    async fn decode_messagepack(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut data = Vec::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            data.extend_from_slice(&bytes);
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let value: serde_json::Value = rmp_serde::from_slice(&data)?;
        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("messagepack_data".to_string(), value);
        
        Ok(DecodeData::Custom(custom_data))
    }

    async fn decode_cbor(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut data = Vec::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            data.extend_from_slice(&bytes);
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let value: serde_json::Value = ciborium::de::from_reader(&data[..])?;
        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("cbor_data".to_string(), value);
        
        Ok(DecodeData::Custom(custom_data))
    }

    async fn decode_custom(&self, config: DecoderConfig) -> Result<DecodeData, Box<dyn std::error::Error>> {
        let source = self.source.read();
        let mut reader = source.clone_reader();
        
        let mut data = Vec::new();
        let mut bytes_read = 0;
        let total_size = reader.get_size().unwrap_or(0);
        
        while !reader.is_at_end()? {
            let chunk_size = 4096;
            let mut buffer = vec![0u8; chunk_size];
            let bytes = reader.read_bytes(&mut buffer)?;
            data.extend_from_slice(&bytes);
            
            bytes_read += bytes as u64;
            
            let progress = if total_size > 0 {
                (bytes_read as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            
            let _ = self.event_sender.send(DecoderEvent::DecodeProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("custom_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(DecodeData::Custom(custom_data))
    }

    async fn create_metadata(&self, config: &DecoderConfig, data: &DecodeData) -> DecodeMetadata {
        let source = self.source.read();
        let size = source.get_size().unwrap_or(0);
        
        DecodeMetadata {
            format: format!("{:?}", config.format),
            size,
            encoding: config.encoding.clone(),
            checksum: Some(self.calculate_checksum(data)),
            created_time: None,
            modified_time: None,
        }
    }

    fn calculate_checksum(&self, data: &DecodeData) -> String {
        match data {
            DecodeData::Binary(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            DecodeData::Hex(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            DecodeData::Base64(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            _ => "unknown".to_string(),
        }
    }

    pub fn set_format(&self, format: DecoderFormat) {
        let mut current_format = self.format.write();
        *current_format = format;
    }

    pub fn get_format(&self) -> DecoderFormat {
        self.format.read().clone()
    }

    pub fn set_source(&self, source: crate::binary_reader::BinaryReader) {
        let mut current_source = self.source.write();
        *current_source = Arc::new(source);
    }

    pub fn get_source(&self) -> crate::binary_reader::BinaryReader {
        self.source.read().clone()
    }

    pub fn set_position(&self, position: u64) {
        let mut current_position = self.position.write();
        *current_position = position;

        let _ = self.event_sender.send(DecoderEvent::PositionChanged(position));
    }

    pub fn get_position(&self) -> u64 {
        *self.position.read()
    }

    pub async fn get_events(&mut self) -> Vec<DecoderEvent> {
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

    pub fn get_supported_formats(&self) -> Vec<DecoderFormat> {
        vec![
            DecoderFormat::JSON,
            DecoderFormat::XML,
            DecoderFormat::CSV,
            DecoderFormat::INI,
            DecoderFormat::TOML,
            DecoderFormat::YAML,
            DecoderFormat::Binary,
            DecoderFormat::Hex,
            DecoderFormat::Base64,
            DecoderFormat::Protobuf,
            DecoderFormat::MessagePack,
            DecoderFormat::CBOR,
            DecoderFormat::Custom,
        ]
    }

    pub fn can_decode_format(&self, format: &DecoderFormat) -> bool {
        self.get_supported_formats().contains(format)
    }

    pub fn clone_decoder(&self) -> BinaryDecoder {
        let mut new_decoder = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_format(),
        );

        let source = self.get_source();
        new_decoder.set_source(source);

        new_decoder
    }

    pub fn reset(&self) {
        self.set_position(0);
    }
}

impl Default for BinaryDecoder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Decoder".to_string(),
            DecoderFormat::JSON,
        )
    }
}

impl Default for DecoderFormat {
    fn default() -> Self {
        DecoderFormat::JSON
    }
}

impl Default for DecoderEvent {
    fn default() -> Self {
        DecoderEvent::DecodeStarted
    }
}

impl Default for DecodeResult {
    fn default() -> Self {
        Self {
            success: false,
            data: DecodeData::Custom(std::collections::HashMap::new()),
            metadata: DecodeMetadata::default(),
            error_message: None,
        }
    }
}

impl Default for DecodeMetadata {
    fn default() -> Self {
        Self {
            format: "JSON".to_string(),
            size: 0,
            encoding: None,
            checksum: None,
            created_time: None,
            modified_time: None,
        }
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            format: DecoderFormat::JSON,
            encoding: None,
            strict_mode: false,
            max_depth: None,
            allow_comments: true,
            allow_trailing_commas: false,
            custom_delimiters: None,
            custom_quote_chars: None,
        }
    }
}
