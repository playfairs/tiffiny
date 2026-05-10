use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct BinaryReader {
    pub id: String,
    pub name: String,
    pub source: Arc<RwLock<ReaderSource>>,
    pub endianness: Arc<RwLock<crate::binary_buffer::Endianness>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<ReaderEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ReaderEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ReaderSource {
    File(Arc<std::fs::File>),
    Memory(Arc<crate::binary_buffer::BinaryBuffer>),
    Stream(Arc<dyn std::io::Read + Send + Sync>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ReaderEvent {
    PositionChanged(u64),
    Error(String),
    EndOfStream,
    SourceChanged,
}

#[derive(Debug, Clone)]
pub struct ReaderConfig {
    pub buffer_size: usize,
    pub auto_rewind: bool,
    pub cache_enabled: bool,
    pub cache_size: usize,
    pub seek_strategy: SeekStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeekStrategy {
    Sequential,
    Random,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct ReaderStats {
    pub bytes_read: u64,
    pub total_bytes: u64,
    pub current_position: u64,
    pub reads_performed: u64,
    pub seeks_performed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl BinaryReader {
    pub fn new(id: String, name: String, source: ReaderSource) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            source: Arc::new(RwLock::new(source)),
            endianness: Arc::new(RwLock::new(crate::binary_buffer::Endianness::Native)),
            position: Arc::new(RwLock::new(0)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_file(id: String, name: String, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        Ok(Self::new(id, name, ReaderSource::File(Arc::new(file))))
    }

    pub fn from_memory(id: String, name: String, buffer: Arc<crate::binary_buffer::BinaryBuffer>) -> Self {
        Self::new(id, name, ReaderSource::Memory(buffer))
    }

    pub fn from_stream(id: String, name: String, stream: Arc<dyn std::io::Read + Send + Sync>) -> Self {
        Self::new(id, name, ReaderSource::Stream(stream))
    }

    pub fn read_u8(&self) -> Result<u8, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 1];
        self.read_bytes(&mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&self) -> Result<u16, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 2];
        self.read_bytes(&mut buffer)?;
        
        let endianness = self.endianness.read();
        match *endianness {
            crate::binary_buffer::Endianness::Little => Ok(u16::from_le_bytes(buffer)),
            crate::binary_buffer::Endianness::Big => Ok(u16::from_be_bytes(buffer)),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u16::from_le_bytes(buffer);
                #[cfg(target_endian = "big")]
                let result = u16::from_be_bytes(buffer);
                result
            },
        }
    }

    pub fn read_u32(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 4];
        self.read_bytes(&mut buffer)?;
        
        let endianness = self.endianness.read();
        match *endianness {
            crate::binary_buffer::Endianness::Little => Ok(u32::from_le_bytes(buffer)),
            crate::binary_buffer::Endianness::Big => Ok(u32::from_be_bytes(buffer)),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u32::from_le_bytes(buffer);
                #[cfg(target_endian = "big")]
                let result = u32::from_be_bytes(buffer);
                result
            },
        }
    }

    pub fn read_u64(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 8];
        self.read_bytes(&mut buffer)?;
        
        let endianness = self.endianness.read();
        match *endianness {
            crate::binary_buffer::Endianness::Little => Ok(u64::from_le_bytes(buffer)),
            crate::binary_buffer::Endianness::Big => Ok(u64::from_be_bytes(buffer)),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u64::from_le_bytes(buffer);
                #[cfg(target_endian = "big")]
                let result = u64::from_be_bytes(buffer);
                result
            },
        }
    }

    pub fn read_i8(&self) -> Result<i8, Box<dyn std::error::Error>> {
        self.read_u8().map(|v| v as i8)
    }

    pub fn read_i16(&self) -> Result<i16, Box<dyn std::error::Error>> {
        self.read_u16().map(|v| v as i16)
    }

    pub fn read_i32(&self) -> Result<i32, Box<dyn std::error::Error>> {
        self.read_u32().map(|v| v as i32)
    }

    pub fn read_i64(&self) -> Result<i64, Box<dyn std::error::Error>> {
        self.read_u64().map(|v| v as i64)
    }

    pub fn read_f32(&self) -> Result<f32, Box<dyn std::error::Error>> {
        self.read_u32().map(|v| f32::from_bits(v))
    }

    pub fn read_f64(&self) -> Result<f64, Box<dyn std::error::Error>> {
        self.read_u64().map(|v| f64::from_bits(v))
    }

    pub fn read_bytes(&self, buffer: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let source = self.source.read();
        
        match &*source {
            ReaderSource::File(file) => {
                let mut file = file.try_clone().ok_or("Failed to clone file handle")?;
                let bytes_read = file.read(buffer)?;
                self.update_position(bytes_read as u64);
                Ok(bytes_read)
            },
            ReaderSource::Memory(binary_buffer) => {
                let data = binary_buffer.bytes();
                let current_pos = self.position.read();
                let bytes_available = data.len().saturating_sub(*current_pos as usize);
                let bytes_to_read = buffer.len().min(bytes_available);
                
                buffer[..bytes_to_read].copy_from_slice(&data[*current_pos as usize..*current_pos as usize + bytes_to_read]);
                self.update_position(bytes_to_read as u64);
                
                Ok(bytes_to_read)
            },
            ReaderSource::Stream(stream) => {
                let bytes_read = stream.read(buffer)?;
                self.update_position(bytes_read as u64);
                Ok(bytes_read)
            },
            ReaderSource::Custom(_) => Err("Custom source not implemented for reading".into()),
        }
    }

    pub fn read_exact(&self, buffer: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes_read = 0;
        let total_bytes = buffer.len();
        
        while bytes_read < total_bytes {
            let read = self.read_bytes(&mut buffer[bytes_read..])?;
            if read == 0 {
                return Err("Unexpected end of stream".into());
            }
            bytes_read += read;
        }
        
        Ok(())
    }

    pub fn read_string(&self, length: usize) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    pub fn read_c_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn read_fixed_string(&self, length: usize) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).trim_end_matches(char::from(0)).to_string())
    }

    pub fn read_pascal_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let length = self.read_u8()? as usize;
        self.read_string(length)
    }

    pub fn read_pascal_string_u16(&self) -> Result<String, Box<dyn std::error::Error>> {
        let length = self.read_u16()? as usize;
        self.read_string(length)
    }

    pub fn read_line(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        
        loop {
            let byte = self.read_u8()?;
            if byte == b'\n' || byte == b'\r' {
                break;
            }
            bytes.push(byte);
        }
        
Handle CRLF
        if bytes.last() == Some(&b'\r') {
            let next_byte = self.read_u8().ok();
            if next_byte != Some(b'\n') {
            }
        }
        
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn skip(&self, bytes: u64) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.source.read();
        
        match &*source {
            ReaderSource::File(file) => {
                let mut file = file.try_clone().ok_or("Failed to clone file handle")?;
                file.seek(SeekFrom::Current(bytes))?;
                self.update_position(bytes);
                Ok(())
            },
            ReaderSource::Memory(binary_buffer) => {
                let current_pos = self.position.read();
                let data = binary_buffer.bytes();
                let new_pos = (*current_pos + bytes).min(data.len() as u64);
                
                let mut position = self.position.write();
                *position = new_pos;
                
                let _ = self.event_sender.send(ReaderEvent::PositionChanged(new_pos));
                Ok(())
            },
            ReaderSource::Stream(stream) => {
                let mut buffer = vec![0u8; 4096];
                let mut remaining = bytes;
                
                while remaining > 0 {
                    let to_read = (remaining as usize).min(buffer.len());
                    let read = stream.read(&mut buffer[..to_read])?;
                    self.update_position(read as u64);
                    remaining -= read as u64;
                    
                    if read == 0 {
                        return Err("Unexpected end of stream".into());
                    }
                }
                
                Ok(())
            },
            ReaderSource::Custom(_) => Err("Custom source not implemented for skipping".into()),
        }
    }

    pub fn seek(&self, position: u64) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.source.read();
        
        match &*source {
            ReaderSource::File(file) => {
                let mut file = file.try_clone().ok_or("Failed to clone file handle")?;
                file.seek(SeekFrom::Start(position))?;
                
                let mut current_position = self.position.write();
                *current_position = position;
                
                let _ = self.event_sender.send(ReaderEvent::PositionChanged(position));
                Ok(())
            },
            ReaderSource::Memory(binary_buffer) => {
                let data = binary_buffer.bytes();
                if position > data.len() as u64 {
                    return Err("Seek position beyond memory buffer length".into());
                }
                
                let mut current_position = self.position.write();
                *current_position = position;
                
                let _ = self.event_sender.send(ReaderEvent::PositionChanged(position));
                Ok(())
            },
            ReaderSource::Stream(stream) => {
                Err("Seeking not supported on stream sources".into())
            },
            ReaderSource::Custom(_) => Err("Custom source not implemented for seeking".into()),
        }
    }

    pub fn seek_relative(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let new_pos = (*current_pos as i64 + offset).max(0) as u64;
        self.seek(new_pos)
    }

    pub fn seek_from_end(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let total_size = self.get_size()?;
        let new_pos = (total_size as i64 + offset).max(0) as u64;
        self.seek(new_pos)
    }

    pub fn rewind(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.seek(0)
    }

    pub fn position(&self) -> u64 {
        *self.position.read()
    }

    pub fn get_size(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let source = self.source.read();
        
        match &*source {
            ReaderSource::File(file) => {
                let file = file.try_clone().ok_or("Failed to clone file handle")?;
                let metadata = file.metadata()?;
                Ok(metadata.len())
            },
            ReaderSource::Memory(binary_buffer) => {
                Ok(binary_buffer.len() as u64)
            },
            ReaderSource::Stream(_) => {
                Err("Cannot get size of stream source".into())
            },
            ReaderSource::Custom(_) => Err("Custom source not implemented for size".into()),
        }
    }

    pub fn remaining(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let total_size = self.get_size()?;
        let current_pos = self.position.read();
        Ok(total_size - *current_pos)
    }

    pub fn is_at_end(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let remaining = self.remaining()?;
        Ok(remaining == 0)
    }

    pub fn set_endianness(&self, endianness: crate::binary_buffer::Endianness) {
        let mut current_endianness = self.endianness.write();
        *current_endianness = endianness;
    }

    pub fn get_endianness(&self) -> crate::binary_buffer::Endianness {
        *self.endianness.read()
    }

    pub fn peek_u8(&self) -> Result<u8, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let byte = self.read_u8()?;
        self.seek(*current_pos)?;
        Ok(byte)
    }

    pub fn peek_u16(&self) -> Result<u16, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let value = self.read_u16()?;
        self.seek(*current_pos)?;
        Ok(value)
    }

    pub fn peek_u32(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let value = self.read_u32()?;
        self.seek(*current_pos)?;
        Ok(value)
    }

    pub fn peek_bytes(&self, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;
        self.seek(*current_pos)?;
        Ok(buffer)
    }

    pub fn find_byte(&self, byte: u8, max_search: Option<usize>) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let start_pos = self.position.read();
        let mut search_count = 0;
        let max_search = max_search.unwrap_or(usize::MAX);
        
        while !self.is_at_end()? && search_count < max_search {
            let current_byte = self.read_u8()?;
            if current_byte == byte {
                return Ok(Some(self.position() - 1));
            }
            search_count += 1;
        }
        
        self.seek(*start_pos)?;
        Ok(None)
    }

    pub fn find_bytes(&self, pattern: &[u8], max_search: Option<usize>) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let start_pos = self.position.read();
        let mut search_count = 0;
        let max_search = max_search.unwrap_or(usize::MAX);
        
        while !self.is_at_end()? && search_count < max_search {
            let current_byte = self.peek_u8()?;
            
            let mut matches = true;
            for (i, &pattern_byte) in pattern.iter().enumerate() {
                if i == 0 {
                    self.read_u8()?;
                } else {
                    let peeked = self.peek_u8()?;
                    if peeked != pattern_byte {
                        matches = false;
                        break;
                    }
                    self.read_u8()?;
                }
            }
            
            if matches {
                return Ok(Some(self.position() - pattern.len() as u64));
            }
            
            search_count += 1;
        }
        
        self.seek(*start_pos)?;
        Ok(None)
    }

    pub fn read_until(&self, delimiter: u8, max_length: Option<usize>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        let max_len = max_length.unwrap_or(usize::MAX);
        
        while bytes.len() < max_len {
            let byte = self.read_u8()?;
            if byte == delimiter {
                break;
            }
            bytes.push(byte);
        }
        
        Ok(bytes)
    }

    pub fn read_line_until(&self, delimiter: &str, max_length: Option<usize>) -> Result<String, Box<dyn std::error::Error>> {
        let delimiter_bytes = delimiter.as_bytes();
        let mut bytes = Vec::new();
        let max_len = max_length.unwrap_or(usize::MAX);
        
        while bytes.len() < max_len {
            let mut matches = true;
            for (i, &delimiter_byte) in delimiter_bytes.iter().enumerate() {
                if i == 0 {
                    let byte = self.read_u8()?;
                    if byte != delimiter_byte {
                        bytes.push(byte);
                        matches = false;
                        break;
                    }
                } else {
                    let peeked = self.peek_u8()?;
                    if peeked != delimiter_byte {
                        matches = false;
                        break;
                    }
                    self.read_u8()?;
                }
            }
            
            if matches {
                break;
            }
        }
        
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn align(&self, alignment: u64) -> Result<u64, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let aligned_pos = (current_pos + alignment - 1) / alignment * alignment;
        self.seek(aligned_pos)?;
        Ok(aligned_pos)
    }

    pub fn read_aligned(&self, alignment: u64, size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let aligned_pos = self.align(alignment)?;
        let mut buffer = vec![0u8; size];
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_bit(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let byte = self.read_u8()?;
        let current_pos = self.position.read();
        let bit_pos = (*current_pos - 1) % 8;
        Ok((byte >> bit_pos) & 1 == 1)
    }

    pub fn read_bits(&self, count: u8) -> Result<u64, Box<dyn std::error::Error>> {
        let mut result = 0u64;
        let mut bits_read = 0;
        
        while bits_read < count {
            let bit = self.read_bit()?;
            result |= (bit as u64) << bits_read;
            bits_read += 1;
        }
        
        Ok(result)
    }

    pub fn read_varint(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u64) << shift;
            
            if (byte & 0x80) == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 63 {
                return Err("Varint too large".into());
            }
        }
        
        Ok(result)
    }

    pub fn read_varint_u32(&self) -> Result<u32, Box<dyn std::error::Error>> {
        self.read_varint().map(|v| v as u32)
    }

    pub fn read_zigzag_i32(&self) -> Result<i32, Box<dyn std::error::Error>> {
        let encoded = self.read_u32()?;
        self.zigzag_decode_i32(encoded)
    }

    fn zigzag_decode_i32(&self, encoded: u32) -> i32 {
        let n = encoded as i32;
        let sign = if n < 0 { -1 } else { 1 };
        let abs_n = n.abs();
        
        let mut decoded = 0;
        let mut i = 0;
        
        while i < 16 {
            let bit_pos = if i < 8 {
                i * 2 + 1
            } else {
                (i - 8) * 2
            };
            
            if (abs_n >> bit_pos) & 1 == 1 {
                decoded |= 1 << i;
            }
            
            i += 1;
        }
        
        decoded * sign
    }

    fn update_position(&self, bytes_read: u64) {
        let mut current_position = self.position.write();
        *current_position += bytes_read;
        
        let _ = self.event_sender.send(ReaderEvent::PositionChanged(*current_position));
    }

    pub async fn get_events(&mut self) -> Vec<ReaderEvent> {
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

    pub fn get_stats(&self) -> ReaderStats {
        ReaderStats {
            bytes_read: self.position(),
            total_bytes: self.get_size().unwrap_or(0),
            current_position: self.position(),
            reads_performed: 0,
            seeks_performed: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn clone_reader(&self) -> BinaryReader {
        let source = self.source.read().clone();
        
        let mut new_reader = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            source,
        );
        
        let endianness = self.endianness.read();
        new_reader.set_endianness(*endianness);
        
        new_reader
    }

    pub fn reset(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.rewind()
    }

    pub fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.source.read();
        
        match &*source {
            ReaderSource::File(_) => {
                Ok(())
            },
            ReaderSource::Memory(_) => {
                Ok(())
            },
            ReaderSource::Stream(_) => {
                Ok(())
            },
            ReaderSource::Custom(_) => {
                Err("Cannot close custom source".into())
            },
        }
    }
}

impl Default for BinaryReader {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Reader".to_string(),
            ReaderSource::Memory(Arc::new(crate::binary_buffer::BinaryBuffer::default())),
        )
    }
}

impl Default for ReaderSource {
    fn default() -> Self {
        ReaderSource::Memory(Arc::new(crate::binary_buffer::BinaryBuffer::default()))
    }
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            auto_rewind: false,
            cache_enabled: false,
            cache_size: 1024 * 1024,
            seek_strategy: SeekStrategy::Sequential,
        }
    }
}

impl Default for SeekStrategy {
    fn default() -> Self {
        SeekStrategy::Sequential
    }
}

impl Default for ReaderStats {
    fn default() -> Self {
        Self {
            bytes_read: 0,
            total_bytes: 0,
            current_position: 0,
            reads_performed: 0,
            seeks_performed: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

trait FileClone {
    fn try_clone(&self) -> Result<std::fs::File, Box<dyn std::error::Error>>;
}

impl FileClone for std::fs::File {
    fn try_clone(&self) -> Result<std::fs::File, Box<dyn std::error::Error>> {
        use std::os::unix::io::AsRawFd;
        
        #[cfg(unix)]
        {
            let fd = self.as_raw_fd();
            unsafe {
                let new_fd = libc::dup(fd);
                if new_fd == -1 {
                    return Err("Failed to duplicate file descriptor".into());
                }
                std::fs::File::from_raw_fd(new_fd)
            }
        }
        
        #[cfg(not(unix))]
        {
            Err("File cloning not supported on this platform".into())
        }
    }
}
