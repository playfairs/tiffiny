use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BinaryParser {
    pub id: String,
    pub name: String,
    pub format: Arc<RwLock<ParserFormat>>,
    pub source: Arc<RwLock<crate::binary_reader::BinaryReader>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<ParserEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ParserEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserFormat {
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
pub enum ParserEvent {
    ParseStarted,
    ParseProgress(f32),
    ParseCompleted(ParseResult),
    Error(String),
    PositionChanged(u64),
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub success: bool,
    pub data: ParseData,
    pub metadata: ParseMetadata,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ParseData {
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
pub struct ParseMetadata {
    pub format: String,
    pub size: u64,
    pub encoding: Option<String>,
    pub checksum: Option<String>,
    pub created_time: Option<std::time::SystemTime>,
    pub modified_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub format: ParserFormat,
    pub encoding: Option<String>,
    pub strict_mode: bool,
    pub max_depth: Option<usize>,
    pub allow_comments: bool,
    pub allow_trailing_commas: bool,
    pub custom_delimiters: Option<String>,
    pub custom_quote_chars: Option<String>,
}

impl BinaryParser {
    pub fn new(id: String, name: String, format: ParserFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            format: Arc::new(RwLock::new(format)),
            source: Arc::new(RwLock::new(crate::binary_reader::BinaryReader::default())),
            position: Arc::new(RwLock::new(0)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_reader(id: String, name: String, reader: crate::binary_reader::BinaryReader, format: ParserFormat) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            format: Arc::new(RwLock::new(format)),
            source: Arc::new(RwLock::new(Arc::new(reader))),
            position: Arc::new(RwLock::new(0)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn parse(&self, config: ParserConfig) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(ParserEvent::ParseStarted);
        let start_time = std::time::Instant::now();

        let result = match config.format {
            ParserFormat::JSON => self.parse_json(config).await,
            ParserFormat::XML => self.parse_xml(config).await,
            ParserFormat::CSV => self.parse_csv(config).await,
            ParserFormat::INI => self.parse_ini(config).await,
            ParserFormat::TOML => self.parse_toml(config).await,
            ParserFormat::YAML => self.parse_yaml(config).await,
            ParserFormat::Binary => self.parse_binary(config).await,
            ParserFormat::Hex => self.parse_hex(config).await,
            ParserFormat::Base64 => self.parse_base64(config).await,
            ParserFormat::Protobuf => self.parse_protobuf(config).await,
            ParserFormat::MessagePack => self.parse_messagepack(config).await,
            ParserFormat::CBOR => self.parse_cbor(config).await,
            ParserFormat::Custom => self.parse_custom(config).await,
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(data) => {
                let metadata = self.create_metadata(&config, &data).await;
                let _ = self.event_sender.send(ParserEvent::ParseCompleted(ParseResult {
                    success: true,
                    data,
                    metadata,
                    error_message: None,
                }));
                
                Ok(ParseResult {
                    success: true,
                    data,
                    metadata,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Parse failed: {}", e);
                let _ = self.event_sender.send(ParserEvent::Error(error_msg.clone()));
                
                Ok(ParseResult {
                    success: false,
                    data: ParseData::Custom(std::collections::HashMap::new()),
                    metadata: ParseMetadata::default(),
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn parse_json(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let json_value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(value) => value,
            Err(e) => {
                if config.strict_mode {
                    return Err(format!("JSON parsing error: {}", e).into());
                } else {
                    serde_json::from_str(&content[..content.len().saturating_sub(e.position().unwrap_or(0), content.len())])?
                        .unwrap_or(serde_json::Value::Null)
                }
            },
        };

        Ok(ParseData::JSON(json_value))
    }

    async fn parse_xml(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        Ok(ParseData::XML(content))
    }

    async fn parse_csv(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
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

        Ok(ParseData::CSV(rows))
    }

    async fn parse_ini(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
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

        Ok(ParseData::INI(ini_data))
    }

    async fn parse_toml(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let toml_value: toml::Value = toml::from_str(&content)?;
        Ok(ParseData::TOML(toml_value))
    }

    async fn parse_yaml(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        Ok(ParseData::YAML(yaml_value))
    }

    async fn parse_binary(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        Ok(ParseData::Binary(data))
    }

    async fn parse_hex(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let hex_data = hex::decode(&content)?;
        Ok(ParseData::Hex(hex_data))
    }

    async fn parse_base64(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let base64_data = base64::decode(&content)?;
        Ok(ParseData::Base64(base64_data))
    }

    async fn parse_protobuf(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("protobuf_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(ParseData::Custom(custom_data))
    }

    async fn parse_messagepack(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("messagepack_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(ParseData::Custom(custom_data))
    }

    async fn parse_cbor(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("cbor_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(ParseData::Custom(custom_data))
    }

    async fn parse_custom(&self, config: ParserConfig) -> Result<ParseData, Box<dyn std::error::Error>> {
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
            
            let _ = self.event_sender.send(ParserEvent::ParseProgress(progress));
        }

        let mut custom_data = std::collections::HashMap::new();
        custom_data.insert("custom_data".to_string(), serde_json::Value::String(base64::encode(&data)));
        
        Ok(ParseData::Custom(custom_data))
    }

    async fn create_metadata(&self, config: &ParserConfig, data: &ParseData) -> ParseMetadata {
        let source = self.source.read();
        let size = source.get_size().unwrap_or(0);
        
        ParseMetadata {
            format: format!("{:?}", config.format),
            size,
            encoding: config.encoding.clone(),
            checksum: Some(self.calculate_checksum(data)),
            created_time: None,
            modified_time: None,
        }
    }

    fn calculate_checksum(&self, data: &ParseData) -> String {
        match data {
            ParseData::Binary(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            ParseData::Hex(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            ParseData::Base64(ref bytes) => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(bytes);
                format!("{:08x}", hasher.finalize())
            },
            _ => "unknown".to_string(),
        }
    }

    pub fn set_format(&self, format: ParserFormat) {
        let mut current_format = self.format.write();
        *current_format = format;
    }

    pub fn get_format(&self) -> ParserFormat {
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

        let _ = self.event_sender.send(ParserEvent::PositionChanged(position));
    }

    pub fn get_position(&self) -> u64 {
        *self.position.read()
    }

    pub async fn get_events(&mut self) -> Vec<ParserEvent> {
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

    pub fn get_supported_formats(&self) -> Vec<ParserFormat> {
        vec![
            ParserFormat::JSON,
            ParserFormat::XML,
            ParserFormat::CSV,
            ParserFormat::INI,
            ParserFormat::TOML,
            ParserFormat::YAML,
            ParserFormat::Binary,
            ParserFormat::Hex,
            ParserFormat::Base64,
            ParserFormat::Protobuf,
            ParserFormat::MessagePack,
            ParserFormat::CBOR,
            ParserFormat::Custom,
        ]
    }

    pub fn can_parse_format(&self, format: &ParserFormat) -> bool {
        self.get_supported_formats().contains(format)
    }

    pub fn clone_parser(&self) -> BinaryParser {
        let mut new_parser = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_format(),
        );

        let source = self.get_source();
        new_parser.set_source(source);

        new_parser
    }

    pub fn reset(&self) {
        self.set_position(0);
    }
}

impl Default for BinaryParser {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Parser".to_string(),
            ParserFormat::JSON,
        )
    }
}

impl Default for ParserFormat {
    fn default() -> Self {
        ParserFormat::JSON
    }
}

impl Default for ParserEvent {
    fn default() -> Self {
        ParserEvent::ParseStarted
    }
}

impl Default for ParseResult {
    fn default() -> Self {
        Self {
            success: false,
            data: ParseData::Custom(std::collections::HashMap::new()),
            metadata: ParseMetadata::default(),
            error_message: None,
        }
    }
}

impl Default for ParseMetadata {
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

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            format: ParserFormat::JSON,
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
