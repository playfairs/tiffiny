use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct DataStream {
    pub id: String,
    pub name: String,
    pub stream_type: StreamType,
    pub buffer_size: Arc<RwLock<usize>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<StreamEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Input,
    Output,
    Bidirectional,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    DataReceived(Vec<u8>),
    DataSent(Vec<u8>),
    PositionChanged(u64),
    Error(String),
    StreamOpened,
    StreamClosed,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub auto_flush: bool,
    pub timeout: std::time::Duration,
    pub retry_count: u32,
    pub retry_delay: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct StreamStats {
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub errors: u64,
    pub uptime: std::time::Duration,
    pub current_position: u64,
}

impl DataStream {
    pub fn new(id: String, name: String, stream_type: StreamType) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            buffer_size: Arc::new(RwLock::new(4096)),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn with_config(id: String, name: String, stream_type: StreamType, config: StreamConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            buffer_size: Arc::new(RwLock::new(config.buffer_size))),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub async fn read(&self, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Input | StreamType::Bidirectional => {
Simulate reading from input
                let mut data = Vec::new();
                for _ in 0..length {
                    data.push(rand::random::<u8>());
                }
                
                let _ = self.event_sender.send(StreamEvent::DataReceived(data.clone()));
                self.update_position(data.len() as u64);
                
                Ok(data)
            },
            StreamType::Output => Err("Cannot read from output stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for reading".into()),
        }
    }

    pub async fn read_exact(&self, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = self.read(length).await?;
        if data.len() == length {
            Ok(data)
        } else {
            Err("Insufficient data available".into())
        }
    }

    pub async fn read_until(&self, delimiter: u8, max_length: Option<usize>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let max_len = max_length.unwrap_or(4096);
        let mut data = Vec::new();
        
        for _ in 0..max_len {
            let byte = self.read(1).await?;
            data.extend_from_slice(&byte);
            
            if byte.last() == Some(&delimiter) {
                break;
            }
        }
        
        Ok(data)
    }

    pub async fn read_line(&self) -> Result<String, Box<dyn std::error::Error>> {
        let line_bytes = self.read_until(b'\n', Some(1024)).await?;
        Ok(String::from_utf8_lossy(&line_bytes).trim_end_matches('\r').to_string())
    }

    pub async fn write(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Output | StreamType::Bidirectional => {
                let _ = self.event_sender.send(StreamEvent::DataSent(data.to_vec()));
                self.update_position(data.len() as u64);
                
                Ok(())
            },
            StreamType::Input => Err("Cannot write to input stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for writing".into()),
        }
    }

    pub async fn write_all(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.write(data).await
    }

    pub async fn write_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write(string.as_bytes()).await
    }

    pub async fn write_line(&self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = line.as_bytes().to_vec();
        data.extend_from_slice(b"\r\n");
        self.write(&data).await
    }

    pub async fn write_formatted(&self, format: &str, args: &[&dyn std::fmt::Display]) -> Result<(), Box<dyn std::error::Error>> {
        let formatted = format!(format, args[0], args[1], args[2], args[3]);
        self.write_string(&formatted).await
    }

    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn peek(&self, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Input | StreamType::Bidirectional => {
                let mut data = Vec::new();
                for _ in 0..length {
                    data.push(rand::random::<u8>());
                }
                
                Ok(data)
            },
            StreamType::Output => Err("Cannot peek from output stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for peeking".into()),
        }
    }

    pub async fn seek(&self, position: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_position = self.position.write();
        *current_position = position;
        
        let _ = self.event_sender.send(StreamEvent::PositionChanged(position));
        Ok(())
    }

    pub async fn seek_relative(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let new_pos = (*current_pos as i64 + offset).max(0) as u64;
        self.seek(new_pos).await
    }

    pub async fn seek_from_end(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let new_pos = if offset >= 0 {
            offset as u64
        } else {
            0
        };
        self.seek(new_pos).await
    }

    pub fn position(&self) -> u64 {
        *self.position.read()
    }

    pub fn set_position(&self, position: u64) {
        let mut current_position = self.position.write();
        *current_position = position;
        
        let _ = self.event_sender.send(StreamEvent::PositionChanged(position));
    }

    pub fn get_buffer_size(&self) -> usize {
        *self.buffer_size.read()
    }

    pub fn set_buffer_size(&self, size: usize) {
        let mut buffer_size = self.buffer_size.write();
        *buffer_size = size;
    }

    pub fn get_stream_type(&self) -> StreamType {
        self.stream_type.clone()
    }

    pub fn is_input(&self) -> bool {
        matches!(self.stream_type, StreamType::Input | StreamType::Bidirectional)
    }

    pub fn is_output(&self) -> bool {
        matches!(self.stream_type, StreamType::Output | StreamType::Bidirectional)
    }

    pub fn is_bidirectional(&self) -> bool {
        matches!(self.stream_type, StreamType::Bidirectional)
    }

    pub fn available(&self) -> Result<usize, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Input | StreamType::Bidirectional => {
                Ok(rand::random::<usize>() % 4096)
            },
            StreamType::Output => Err("No available data in output stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for available".into()),
        }
    }

    pub fn is_empty(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let available = self.available()?;
        Ok(available == 0)
    }

    pub fn remaining(&self) -> Result<usize, Box<dyn std::error::Error>> {
        self.available()
    }

    pub async fn copy_from(&self, other: &DataStream) -> Result<(), Box<dyn std::error::Error>> {
        let buffer_size = self.get_buffer_size();
        let mut total_copied = 0;
        
        loop {
            let data = other.read(buffer_size).await?;
            if data.is_empty() {
                break;
            }
            
            self.write(&data).await?;
            total_copied += data.len();
        }
        
        Ok(())
    }

    pub async fn copy_to(&self, other: &DataStream) -> Result<(), Box<dyn std::error::Error>> {
        let buffer_size = self.get_buffer_size();
        let mut total_copied = 0;
        
        loop {
            let data = self.read(buffer_size).await?;
            if data.is_empty() {
                break;
            }
            
            other.write(&data).await?;
            total_copied += data.len();
        }
        
        Ok(())
    }

    pub async fn transfer(&self, other: &DataStream, length: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut transferred = 0;
        
        while transferred < length {
            let remaining = length - transferred;
            let chunk_size = remaining.min(self.get_buffer_size());
            
            let data = self.read(chunk_size).await?;
            if data.is_empty() {
                break;
            }
            
            other.write(&data).await?;
            transferred += data.len();
        }
        
        Ok(())
    }

    pub async fn get_events(&mut self) -> Vec<StreamEvent> {
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

    pub fn get_stats(&self) -> StreamStats {
        StreamStats {
            bytes_received: self.position(),
            bytes_sent: self.position(),
            packets_received: 0,
            packets_sent: 0,
            errors: 0,
            uptime: std::time::Duration::from_secs(0),
            current_position: self.position(),
        }
    }

    pub fn clone_stream(&self) -> DataStream {
        let mut new_stream = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            self.stream_type.clone(),
        );
        
        let buffer_size = self.get_buffer_size();
        new_stream.set_buffer_size(buffer_size);
        
        new_stream
    }

    pub fn reset(&self) {
        self.set_position(0);
    }

    pub fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(StreamEvent::StreamClosed);
        Ok(())
    }

    fn update_position(&self, bytes: u64) {
        let mut current_position = self.position.write();
        *current_position += bytes;
        
        let _ = self.event_sender.send(StreamEvent::PositionChanged(*current_position));
    }

    pub async fn read_timeout(&self, length: usize, timeout: std::time::Duration) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let _ = timeout;
        self.read(length).await
    }

    pub async fn write_timeout(&self, data: &[u8], timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
        let _ = timeout;
        self.write(data).await
    }

    pub async fn read_with_callback<F>(&self, length: usize, callback: F) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        F: Fn(Vec<u8>) + Send + Sync,
    {
        let data = self.read(length).await?;
        callback(data.clone());
        Ok(data)
    }

    pub async fn write_with_callback<F>(&self, data: &[u8], callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&[u8]) + Send + Sync,
    {
        self.write(data).await?;
        callback(data);
        Ok(())
    }

    pub async fn read_all(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut all_data = Vec::new();
        
        loop {
            let data = self.read(4096).await?;
            if data.is_empty() {
                break;
            }
            all_data.extend_from_slice(&data);
        }
        
        Ok(all_data)
    }

    pub async fn write_all(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_size = self.get_buffer_size();
        
        for chunk in data.chunks(chunk_size) {
            self.write(chunk).await?;
        }
        
        Ok(())
    }

    pub async fn read_until_timeout(&self, delimiter: u8, max_length: Option<usize>, timeout: std::time::Duration) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let _ = timeout;
        self.read_until(delimiter, max_length).await
    }

    pub async fn read_line_timeout(&self, timeout: std::time::Duration) -> Result<String, Box<dyn std::error::Error>> {
        let _ = timeout;
        self.read_line().await
    }

    pub async fn write_string_timeout(&self, string: &str, timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
        let _ = timeout;
        self.write_string(string).await
    }

    pub async fn write_line_timeout(&self, line: &str, timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
        let _ = timeout;
        self.write_line(line).await
    }

    pub async fn read_exact_timeout(&self, length: usize, timeout: std::time::Duration) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let _ = timeout;
        self.read_exact(length).await
    }

    pub async fn write_all_timeout(&self, data: &[u8], timeout: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
        let _ = timeout;
        self.write_all(data).await
    }

    pub async fn read_with_progress<F>(&self, length: usize, progress_callback: F) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        F: Fn(usize, usize) + Send + Sync,
    {
        let mut data = Vec::new();
        let mut total_read = 0;
        let chunk_size = self.get_buffer_size();
        
        while total_read < length {
            let remaining = length - total_read;
            let chunk_len = remaining.min(chunk_size);
            
            let chunk = self.read(chunk_len).await?;
            data.extend_from_slice(&chunk);
            total_read += chunk.len();
            
            progress_callback(total_read, length);
        }
        
        Ok(data)
    }

    pub async fn write_with_progress<F>(&self, data: &[u8], progress_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(usize, usize) + Send + Sync,
    {
        let chunk_size = self.get_buffer_size();
        let total_bytes = data.len();
        
        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            self.write(chunk).await?;
            progress_callback((i + 1) * chunk_size, total_bytes);
        }
        
        Ok(())
    }
}

impl Default for DataStream {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Data Stream".to_string(),
            StreamType::Input,
        )
    }
}

impl Default for StreamType {
    fn default() -> Self {
        StreamType::Input
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            auto_flush: false,
            timeout: std::time::Duration::from_secs(30),
            retry_count: 3,
            retry_delay: std::time::Duration::from_millis(100),
        }
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            errors: 0,
            uptime: std::time::Duration::from_secs(0),
            current_position: 0,
        }
    }
}
