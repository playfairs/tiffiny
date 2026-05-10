use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BinaryEncoder {
    pub id: String,
    pub name: String,
    pub format: Arc<RwLock<EncoderFormat>>,
    pub destination: Arc<RwLock<crate::binary_writer::BinaryWriter>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<EncoderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<EncoderEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncoderFormat {
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
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum EncoderEvent {
    EncodeStarted,
    EncodeProgress(f32),
    EncodeCompleted,
    Error(String),
    PositionChanged(u64),
}

#[derive(Debug, Clone)]
pub struct EncodeResult {
    pub success: bool,
    pub bytes_written: u64,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub format: EncoderFormat,
    pub encoding: Option<String>,
    pub pretty_print: bool,
    pub indent: Option<usize>,
    pub custom_delimiters: Option<String>,
    pub custom_quote_chars: Option<String>,
    pub include_metadata: bool,
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Deflate,
    Brotli,
    LZ4,
    Zstd,
}

impl BinaryEncoder {
    pub fn new(id: String, name: String, format: EncoderFormat, destination: crate::binary_writer::BinaryWriter) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            format: Arc::new(RwLock::new(format)),
            destination: Arc::new(RwLock::new(Arc::new(destination))),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn to_file(id: String, name: String, format: EncoderFormat, path: &str, config: EncoderConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let writer_config = crate::binary_writer::WriterConfig::default();
        let writer = crate::binary_writer::BinaryWriter::to_file(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Writer", name),
            path,
            writer_config,
        )?;
        
        Ok(Self::new(id, name, format, writer))
    }

    pub fn to_memory(id: String, name: String, format: EncoderFormat, capacity: usize) -> Self {
        let writer = crate::binary_writer::BinaryWriter::to_memory(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Writer", name),
            capacity,
        );
        
        Self::new(id, name, format, writer)
    }

    pub async fn encode(&self, data: &EncodeData, config: EncoderConfig) -> Result<EncodeResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(EncoderEvent::EncodeStarted);
        let start_time = std::time::Instant::now();

        let result = match config.format {
            EncoderFormat::JSON => self.encode_json(data, &config).await,
            EncoderFormat::XML => self.encode_xml(data, &config).await,
            EncoderFormat::CSV => self.encode_csv(data, &config).await,
            EncoderFormat::INI => self.encode_ini(data, &config).await,
            EncoderFormat::TOML => self.encode_toml(data, &config).await,
            EncoderFormat::YAML => self.encode_yaml(data, &config).await,
            EncoderFormat::Binary => self.encode_binary(data, &config).await,
            EncoderFormat::Hex => self.encode_hex(data, &config).await,
            EncoderFormat::Base64 => self.encode_base64(data, &config).await,
            EncoderFormat::Protobuf => self.encode_protobuf(data, &config).await,
            EncoderFormat::MessagePack => self.encode_messagepack(data, &config).await,
            EncoderFormat::CBOR => self.encode_cbor(data, &config).await,
            EncoderFormat::Custom(_) => self.encode_custom(data, &config).await,
        };

        let processing_time = start_time.elapsed();
        let bytes_written = self.destination.read().position();

        match result {
            Ok(_) => {
                let _ = self.event_sender.send(EncoderEvent::EncodeCompleted);
                
                Ok(EncodeResult {
                    success: true,
                    bytes_written,
                    processing_time,
                    error_message: None,
                })
            },
            Err(e) => {
                let error_msg = format!("Encode failed: {}", e);
                let _ = self.event_sender.send(EncoderEvent::Error(error_msg.clone()));
                
                Ok(EncodeResult {
                    success: false,
                    bytes_written,
                    processing_time,
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn encode_json(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let json_value = self.data_to_json_value(data)?;
        let json_string = if config.pretty_print {
            serde_json::to_string_pretty(&json_value)?
        } else {
            serde_json::to_string(&json_value)?
        };

        let destination = self.destination.read();
        destination.clone().write_string(&json_string)?;
        
        Ok(())
    }

    async fn encode_xml(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
Simplified XML encoding
        let xml_string = self.data_to_xml_string(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_string(&xml_string)?;
        
        Ok(())
    }

    async fn encode_csv(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let csv_string = self.data_to_csv_string(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_string(&csv_string)?;
        
        Ok(())
    }

    async fn encode_ini(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let ini_string = self.data_to_ini_string(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_string(&ini_string)?;
        
        Ok(())
    }

    async fn encode_toml(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let toml_value = self.data_to_toml_value(data)?;
        let toml_string = toml::to_string_pretty(&toml_value)?;
        
        let destination = self.destination.read();
        destination.clone().write_string(&toml_string)?;
        
        Ok(())
    }

    async fn encode_yaml(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_value = self.data_to_yaml_value(data)?;
        let yaml_string = serde_yaml::to_string(&yaml_value)?;
        
        let destination = self.destination.read();
        destination.clone().write_string(&yaml_string)?;
        
        Ok(())
    }

    async fn encode_binary(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.data_to_binary(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_bytes(&bytes)?;
        
        Ok(())
    }

    async fn encode_hex(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.data_to_binary(data, config)?;
        let hex_string = hex::encode(&bytes);
        
        let destination = self.destination.read();
        destination.clone().write_string(&hex_string)?;
        
        Ok(())
    }

    async fn encode_base64(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.data_to_binary(data, config)?;
        let base64_string = base64::encode(&bytes);
        
        let destination = self.destination.read();
        destination.clone().write_string(&base64_string)?;
        
        Ok(())
    }

    async fn encode_protobuf(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.data_to_binary(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_bytes(&bytes)?;
        
        Ok(())
    }

    async fn encode_messagepack(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let json_value = self.data_to_json_value(data)?;
        let bytes = rmp_serde::to_vec(&json_value)?;
        
        let destination = self.destination.read();
        destination.clone().write_bytes(&bytes)?;
        
        Ok(())
    }

    async fn encode_cbor(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let json_value = self.data_to_json_value(data)?;
        let bytes = ciborium::ser::into_writer(&json_value)?;
        
        let destination = self.destination.read();
        destination.clone().write_bytes(&bytes)?;
        
        Ok(())
    }

    async fn encode_custom(&self, data: &EncodeData, config: &EncoderConfig) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.data_to_binary(data, config)?;
        
        let destination = self.destination.read();
        destination.clone().write_bytes(&bytes)?;
        
        Ok(())
    }

    fn data_to_json_value(&self, data: &EncodeData) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeData::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (key, value) in obj {
                    map.insert(key.clone(), self.encode_data_to_json_value(value)?);
                }
                Ok(serde_json::Value::Object(map))
            },
            EncodeData::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_json_value(value)?);
                }
                Ok(serde_json::Value::Array(vec))
            },
            EncodeData::String(s) => Ok(serde_json::Value::String(s.clone())),
            EncodeData::Number(n) => Ok(serde_json::Value::Number(serde_json::Number::from(*n))),
            EncodeData::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
            EncodeData::Null => Ok(serde_json::Value::Null),
            EncodeData::Binary(bytes) => {
                Ok(serde_json::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn encode_data_to_json_value(&self, data: &EncodeDataValue) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (key, value) in obj {
                    map.insert(key.clone(), self.encode_data_to_json_value(value)?);
                }
                Ok(serde_json::Value::Object(map))
            },
            EncodeDataValue::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_json_value(value)?);
                }
                Ok(serde_json::Value::Array(vec))
            },
            EncodeDataValue::String(s) => Ok(serde_json::Value::String(s.clone())),
            EncodeDataValue::Number(n) => Ok(serde_json::Value::Number(serde_json::Number::from(*n))),
            EncodeDataValue::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
            EncodeDataValue::Null => Ok(serde_json::Value::Null),
            EncodeDataValue::Binary(bytes) => {
                Ok(serde_json::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn data_to_xml_string(&self, data: &EncodeData, config: &EncoderConfig) -> Result<String, Box<dyn std::error::Error>> {
        let indent_str = if let Some(indent) = config.indent {
            " ".repeat(indent)
        } else {
            String::new()
        };

        match data {
            EncodeData::Object(obj) => {
                let mut xml = String::from("<root>\n");
                for (key, value) in obj {
                    xml.push_str(&indent_str);
                    xml.push_str(&format!("<{key}>{value}</{key}>\n", 
                        key = key, 
                        value = self.encode_data_to_xml_string(value, &indent_str)?));
                }
                xml.push_str("</root>");
                Ok(xml)
            },
            EncodeData::Array(arr) => {
                let mut xml = String::from("<array>\n");
                for (i, value) in arr.iter().enumerate() {
                    xml.push_str(&indent_str);
                    xml.push_str(&format!("<item index=\"{i}\">{value}</item>\n", 
                        i = i, 
                        value = self.encode_data_to_xml_string(value, &indent_str)?));
                }
                xml.push_str("</array>");
                Ok(xml)
            },
            EncodeData::String(s) => Ok(s.clone()),
            EncodeData::Number(n) => Ok(n.to_string()),
            EncodeData::Boolean(b) => Ok(b.to_string()),
            EncodeData::Null => Ok("null".to_string()),
            EncodeData::Binary(bytes) => Ok(base64::encode(bytes)),
        }
    }

    fn encode_data_to_xml_string(&self, data: &EncodeDataValue, indent: &str) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::Object(obj) => {
                let mut xml = String::from("<object>\n");
                for (key, value) in obj {
                    xml.push_str(&format!("{indent}<{key}>{value}</{key}>\n", 
                        key = key, 
                        value = self.encode_data_to_xml_string(value, &format!("{}  ", indent))?));
                }
                xml.push_str(&format!("{}</object>", indent));
                Ok(xml)
            },
            EncodeDataValue::Array(arr) => {
                let mut xml = String::from("<array>\n");
                for (i, value) in arr.iter().enumerate() {
                    xml.push_str(&format!("{indent}<item index=\"{i}\">{value}</item>\n", 
                        i = i, 
                        value = self.encode_data_to_xml_string(value, &format!("{}  ", indent))?));
                }
                xml.push_str(&format!("{}</array>", indent));
                Ok(xml)
            },
            EncodeDataValue::String(s) => Ok(s.clone()),
            EncodeDataValue::Number(n) => Ok(n.to_string()),
            EncodeDataValue::Boolean(b) => Ok(b.to_string()),
            EncodeDataValue::Null => Ok("null".to_string()),
            EncodeDataValue::Binary(bytes) => Ok(base64::encode(bytes)),
        }
    }

    fn data_to_csv_string(&self, data: &EncodeData, config: &EncoderConfig) -> Result<String, Box<dyn std::error::Error>> {
        let delimiter = config.custom_delimiters.as_deref().unwrap_or(",");
        let quote_char = config.custom_quote_chars.as_deref().unwrap_or("\"");

        match data {
            EncodeData::Array(arr) => {
                let mut csv = String::new();
                for (i, value) in arr.iter().enumerate() {
                    if i > 0 {
                        csv.push_str(delimiter);
                    }
                    csv.push_str(&self.encode_data_to_csv_string(value, quote_char)?);
                }
                Ok(csv)
            },
            EncodeData::Object(obj) => {
                let mut csv = String::new();
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        csv.push_str(delimiter);
                    }
                    csv.push_str(&format!("\"{}\"{}{}", key, delimiter, self.encode_data_to_csv_string(value, quote_char)?));
                }
                Ok(csv)
            },
            _ => Ok(self.encode_data_to_csv_string(&EncodeDataValue::from(data.clone()), quote_char)?),
        }
    }

    fn encode_data_to_csv_string(&self, data: &EncodeDataValue, quote_char: &str) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::String(s) => {
                let escaped = s.replace(quote_char, &format!("\\{}", quote_char));
                Ok(format!("{}{}{}", quote_char, escaped, quote_char))
            },
            EncodeDataValue::Number(n) => Ok(n.to_string()),
            EncodeDataValue::Boolean(b) => Ok(b.to_string()),
            EncodeDataValue::Null => Ok("".to_string()),
            EncodeDataValue::Binary(bytes) => Ok(base64::encode(bytes)),
            EncodeDataValue::Object(_) => Ok("".to_string()),
            EncodeDataValue::Array(_) => Ok("".to_string()),
        }
    }

    fn data_to_ini_string(&self, data: &EncodeData, config: &EncoderConfig) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            EncodeData::Object(obj) => {
                let mut ini = String::new();
                for (key, value) in obj {
                    ini.push_str(&format!("{}={}\n", key, self.encode_data_to_ini_string(value)?));
                }
                Ok(ini)
            },
            _ => Err("INI format requires object data".into()),
        }
    }

    fn encode_data_to_ini_string(&self, data: &EncodeDataValue) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::String(s) => Ok(s.clone()),
            EncodeDataValue::Number(n) => Ok(n.to_string()),
            EncodeDataValue::Boolean(b) => Ok(b.to_string()),
            EncodeDataValue::Null => Ok("".to_string()),
            EncodeDataValue::Binary(bytes) => Ok(base64::encode(bytes)),
            EncodeDataValue::Object(_) => Err("Nested objects not supported in INI".into()),
            EncodeDataValue::Array(_) => Err("Arrays not supported in INI".into()),
        }
    }

    fn data_to_toml_value(&self, data: &EncodeData) -> Result<toml::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeData::Object(obj) => {
                let mut table = toml::value::Table::new();
                for (key, value) in obj {
                    table.insert(key.clone(), self.encode_data_to_toml_value(value)?);
                }
                Ok(toml::Value::Table(table))
            },
            EncodeData::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_toml_value(value)?);
                }
                Ok(toml::Value::Array(vec))
            },
            EncodeData::String(s) => Ok(toml::Value::String(s.clone())),
            EncodeData::Number(n) => Ok(toml::Value::Float(*n)),
            EncodeData::Boolean(b) => Ok(toml::Value::Boolean(*b)),
            EncodeData::Null => Ok(toml::Value::Datetime(toml::value::Datetime::from_str("1970-01-01T00:00:00Z").unwrap())),
            EncodeData::Binary(bytes) => {
                Ok(toml::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn encode_data_to_toml_value(&self, data: &EncodeDataValue) -> Result<toml::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::Object(obj) => {
                let mut table = toml::value::Table::new();
                for (key, value) in obj {
                    table.insert(key.clone(), self.encode_data_to_toml_value(value)?);
                }
                Ok(toml::Value::Table(table))
            },
            EncodeDataValue::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_toml_value(value)?);
                }
                Ok(toml::Value::Array(vec))
            },
            EncodeDataValue::String(s) => Ok(toml::Value::String(s.clone())),
            EncodeDataValue::Number(n) => Ok(toml::Value::Float(*n)),
            EncodeDataValue::Boolean(b) => Ok(toml::Value::Boolean(*b)),
            EncodeDataValue::Null => Ok(toml::Value::Datetime(toml::value::Datetime::from_str("1970-01-01T00:00:00Z").unwrap())),
            EncodeDataValue::Binary(bytes) => {
                Ok(toml::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn data_to_yaml_value(&self, data: &EncodeData) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeData::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (key, value) in obj {
                    map.insert(serde_yaml::Value::String(key.clone()), self.encode_data_to_yaml_value(value)?);
                }
                Ok(serde_yaml::Value::Mapping(map))
            },
            EncodeData::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_yaml_value(value)?);
                }
                Ok(serde_yaml::Value::Sequence(vec))
            },
            EncodeData::String(s) => Ok(serde_yaml::Value::String(s.clone())),
            EncodeData::Number(n) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(*n))),
            EncodeData::Boolean(b) => Ok(serde_yaml::Value::Bool(*b)),
            EncodeData::Null => Ok(serde_yaml::Value::Null),
            EncodeData::Binary(bytes) => {
                Ok(serde_yaml::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn encode_data_to_yaml_value(&self, data: &EncodeDataValue) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (key, value) in obj {
                    map.insert(serde_yaml::Value::String(key.clone()), self.encode_data_to_yaml_value(value)?);
                }
                Ok(serde_yaml::Value::Mapping(map))
            },
            EncodeDataValue::Array(arr) => {
                let mut vec = Vec::new();
                for value in arr {
                    vec.push(self.encode_data_to_yaml_value(value)?);
                }
                Ok(serde_yaml::Value::Sequence(vec))
            },
            EncodeDataValue::String(s) => Ok(serde_yaml::Value::String(s.clone())),
            EncodeDataValue::Number(n) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(*n))),
            EncodeDataValue::Boolean(b) => Ok(serde_yaml::Value::Bool(*b)),
            EncodeDataValue::Null => Ok(serde_yaml::Value::Null),
            EncodeDataValue::Binary(bytes) => {
                Ok(serde_yaml::Value::String(base64::encode(bytes)))
            },
        }
    }

    fn data_to_binary(&self, data: &EncodeData, config: &EncoderConfig) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match data {
            EncodeData::Binary(bytes) => Ok(bytes.clone()),
            EncodeData::String(s) => Ok(s.as_bytes().to_vec()),
            EncodeData::Number(n) => Ok(n.to_string().as_bytes().to_vec()),
            EncodeData::Boolean(b) => Ok(if *b { b"1" } else { b"0" }.to_vec()),
            EncodeData::Null => Ok(Vec::new()),
            EncodeData::Object(obj) => {
                let mut bytes = Vec::new();
                for (key, value) in obj {
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.push(b'=');
                    bytes.extend_from_slice(&self.encode_data_to_binary(value)?);
                    bytes.push(b'\n');
                }
                Ok(bytes)
            },
            EncodeData::Array(arr) => {
                let mut bytes = Vec::new();
                for value in arr {
                    bytes.extend_from_slice(&self.encode_data_to_binary(value)?);
                    bytes.push(b',');
                }
                Ok(bytes)
            },
        }
    }

    fn encode_data_to_binary(&self, data: &EncodeDataValue) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match data {
            EncodeDataValue::Binary(bytes) => Ok(bytes.clone()),
            EncodeDataValue::String(s) => Ok(s.as_bytes().to_vec()),
            EncodeDataValue::Number(n) => Ok(n.to_string().as_bytes().to_vec()),
            EncodeDataValue::Boolean(b) => Ok(if *b { b"1" } else { b"0" }.to_vec()),
            EncodeDataValue::Null => Ok(Vec::new()),
            EncodeDataValue::Object(obj) => {
                let mut bytes = Vec::new();
                for (key, value) in obj {
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.push(b'=');
                    bytes.extend_from_slice(&self.encode_data_to_binary(value)?);
                    bytes.push(b'\n');
                }
                Ok(bytes)
            },
            EncodeDataValue::Array(arr) => {
                let mut bytes = Vec::new();
                for value in arr {
                    bytes.extend_from_slice(&self.encode_data_to_binary(value)?);
                    bytes.push(b',');
                }
                Ok(bytes)
            },
        }
    }

    pub fn set_format(&self, format: EncoderFormat) {
        let mut current_format = self.format.write();
        *current_format = format;
    }

    pub fn get_format(&self) -> EncoderFormat {
        self.format.read().clone()
    }

    pub fn set_destination(&self, destination: crate::binary_writer::BinaryWriter) {
        let mut current_destination = self.destination.write();
        *current_destination = Arc::new(destination);
    }

    pub fn get_destination(&self) -> crate::binary_writer::BinaryWriter {
        self.destination.read().clone()
    }

    pub fn set_position(&self, position: u64) {
        let mut current_position = self.position.write();
        *current_position = position;

        let _ = self.event_sender.send(EncoderEvent::PositionChanged(position));
    }

    pub fn get_position(&self) -> u64 {
        *self.position.read()
    }

    pub async fn get_events(&mut self) -> Vec<EncoderEvent> {
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

    pub fn get_supported_formats(&self) -> Vec<EncoderFormat> {
        vec![
            EncoderFormat::JSON,
            EncoderFormat::XML,
            EncoderFormat::CSV,
            EncoderFormat::INI,
            EncoderFormat::TOML,
            EncoderFormat::YAML,
            EncoderFormat::Binary,
            EncoderFormat::Hex,
            EncoderFormat::Base64,
            EncoderFormat::Protobuf,
            EncoderFormat::MessagePack,
            EncoderFormat::CBOR,
        ]
    }

    pub fn can_encode_format(&self, format: &EncoderFormat) -> bool {
        self.get_supported_formats().contains(format)
    }

    pub fn clone_encoder(&self) -> BinaryEncoder {
        let mut new_encoder = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.get_format(),
            self.get_destination(),
        );

        new_encoder
    }

    pub fn reset(&self) {
        self.set_position(0);
    }
}

#[derive(Debug, Clone)]
pub enum EncodeData {
    Object(std::collections::HashMap<String, EncodeDataValue>),
    Array(Vec<EncodeDataValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Binary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum EncodeDataValue {
    Object(std::collections::HashMap<String, EncodeDataValue>),
    Array(Vec<EncodeDataValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Binary(Vec<u8>),
}

impl From<EncodeData> for EncodeDataValue {
    fn from(data: EncodeData) -> Self {
        match data {
            EncodeData::Object(obj) => EncodeDataValue::Object(obj),
            EncodeData::Array(arr) => EncodeDataValue::Array(arr),
            EncodeData::String(s) => EncodeDataValue::String(s),
            EncodeData::Number(n) => EncodeDataValue::Number(n),
            EncodeData::Boolean(b) => EncodeDataValue::Boolean(b),
            EncodeData::Null => EncodeDataValue::Null,
            EncodeData::Binary(bytes) => EncodeDataValue::Binary(bytes),
        }
    }
}

impl Default for BinaryEncoder {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Encoder".to_string(),
            EncoderFormat::JSON,
            crate::binary_writer::BinaryWriter::default(),
        )
    }
}

impl Default for EncoderFormat {
    fn default() -> Self {
        EncoderFormat::JSON
    }
}

impl Default for EncoderEvent {
    fn default() -> Self {
        EncoderEvent::EncodeStarted
    }
}

impl Default for EncodeResult {
    fn default() -> Self {
        Self {
            success: false,
            bytes_written: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            format: EncoderFormat::JSON,
            encoding: None,
            pretty_print: false,
            indent: None,
            custom_delimiters: None,
            custom_quote_chars: None,
            include_metadata: false,
            compression: Some(CompressionType::None),
        }
    }
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

impl Default for EncodeData {
    fn default() -> Self {
        EncodeData::Null
    }
}

impl Default for EncodeDataValue {
    fn default() -> Self {
        EncodeDataValue::Null
    }
}
