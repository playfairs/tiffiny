use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use std::io::{self, Write, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct BinaryWriter {
    pub id: String,
    pub name: String,
    pub destination: Arc<RwLock<WriterDestination>>,
    pub endianness: Arc<RwLock<crate::binary_buffer::Endianness>>,
    pub position: Arc<RwLock<u64>>,
    pub event_sender: mpsc::UnboundedSender<WriterEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<WriterEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum WriterDestination {
    File(Arc<std::fs::File>),
    Memory(Arc<crate::binary_buffer::BinaryBuffer>),
    Stream(Arc<dyn std::io::Write + Send + Sync>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum WriterEvent {
    PositionChanged(u64),
    Error(String),
    FlushCompleted,
    DestinationChanged,
}

#[derive(Debug, Clone)]
pub struct WriterConfig {
    pub buffer_size: usize,
    pub auto_flush: bool,
    pub create_dirs: bool,
    pub append_mode: bool,
    pub truncate_mode: bool,
}

#[derive(Debug, Clone)]
pub struct WriterStats {
    pub bytes_written: u64,
    pub flushes_performed: u64,
    pub writes_performed: u64,
    pub current_position: u64,
    pub buffer_utilization: f32,
}

impl BinaryWriter {
    pub fn new(id: String, name: String, destination: WriterDestination) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            destination: Arc::new(RwLock::new(destination)),
            endianness: Arc::new(RwLock::new(crate::binary_buffer::Endianness::Native)),
            position: Arc::new(RwLock::new(0))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn to_file(id: String, name: String, path: &str, config: WriterConfig) -> Result<Self, Box<dyn std::error::Error>> {
        if config.create_dirs {
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(config.append_mode)
            .truncate(config.truncate_mode)
            .open(path)?;

        Ok(Self::new(id, name, WriterDestination::File(Arc::new(file)))))
    }

    pub fn to_memory(id: String, name: String, capacity: usize) -> Self {
        let buffer = crate::binary_buffer::BinaryBuffer::with_capacity(capacity);
        Self::new(id, name, WriterDestination::Memory(Arc::new(buffer))))
    }

    pub fn to_stream(id: String, name: String, stream: Arc<dyn std::io::Write + Send + Sync>) -> Self {
        Self::new(id, name, WriterDestination::Stream(stream)))
    }

    pub fn write_u8(&self, value: u8) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                file_clone.write_all(&[value])?;
                self.update_position(1)?;
            },
            WriterDestination::Memory(buffer) => {
                buffer.write_u8(buffer.len(), value)?;
                self.update_position(1)?;
            },
            WriterDestination::Stream(stream) => {
                let mut stream_clone = &**stream;
                stream_clone.write_all(&[value])?;
                self.update_position(1)?;
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for writing".into());
            },
        }
        
        let _ = self.event_sender.send(WriterEvent::PositionChanged(self.position()));
        Ok(())
    }

    pub fn write_u16(&self, value: u16) -> Result<(), Box<dyn std::error::Error>> {
        let endianness = self.endianness.read();
        let bytes = match *endianness {
            crate::binary_buffer::Endianness::Little => value.to_le_bytes(),
            crate::binary_buffer::Endianness::Big => value.to_be_bytes(),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        self.write_bytes(&bytes)
    }

    pub fn write_u32(&self, value: u32) -> Result<(), Box<dyn std::error::Error>> {
        let endianness = self.endianness.read();
        let bytes = match *endianness {
            crate::binary_buffer::Endianness::Little => value.to_le_bytes(),
            crate::binary_buffer::Endianness::Big => value.to_be_bytes(),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        self.write_bytes(&bytes)
    }

    pub fn write_u64(&self, value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let endianness = self.endianness.read();
        let bytes = match *endianness {
            crate::binary_buffer::Endianness::Little => value.to_le_bytes(),
            crate::binary_buffer::Endianness::Big => value.to_be_bytes(),
            crate::binary_buffer::Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        self.write_bytes(&bytes)
    }

    pub fn write_i8(&self, value: i8) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u8(value as u8)
    }

    pub fn write_i16(&self, value: i16) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u16(value as u16)
    }

    pub fn write_i32(&self, value: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u32(value as u32)
    }

    pub fn write_i64(&self, value: i64) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u64(value as u64)
    }

    pub fn write_f32(&self, value: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u32(value.to_bits())
    }

    pub fn write_f64(&self, value: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u64(value.to_bits())
    }

    pub fn write_bytes(&self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                file_clone.write_all(bytes)?;
                self.update_position(bytes.len() as u64)?;
            },
            WriterDestination::Memory(buffer) => {
                buffer.append_bytes(bytes)?;
                self.update_position(bytes.len() as u64)?;
            },
            WriterDestination::Stream(stream) => {
                let mut stream_clone = &**stream;
                stream_clone.write_all(bytes)?;
                self.update_position(bytes.len() as u64)?;
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for writing".into());
            },
        }
        
        let _ = self.event_sender.send(WriterEvent::PositionChanged(self.position()));
        Ok(())
    }

    pub fn write_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_bytes(string.as_bytes())
    }

    pub fn write_c_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_bytes(string.as_bytes())?;
        self.write_u8(0)Null terminator
    }

    pub fn write_fixed_string(&self, string: &str, length: usize) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        let mut padded_bytes = vec![0u8; length];
        
        let copy_len = bytes.len().min(length);
        padded_bytes[..copy_len].copy_from_slice(&bytes[..copy_len]);
        
        self.write_bytes(&padded_bytes)
    }

    pub fn write_pascal_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        let length = bytes.len();
        
        if length > 255 {
            return Err("Pascal string too long (max 255 bytes)".into());
        }
        
        self.write_u8(length as u8)?;
        self.write_bytes(&bytes)
    }

    pub fn write_pascal_string_u16(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        let length = bytes.len();
        
        if length > 65535 {
            return Err("Pascal string too long (max 65535 bytes)".into());
        }
        
        self.write_u16(length as u16)?;
        self.write_bytes(&bytes)
    }

    pub fn write_varint(&self, value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut remaining = value;
        let mut bytes_written = 0;
        
        while remaining >= 0x80 {
            let mut byte = (remaining & 0x7F) | 0x80;
            self.write_u8(byte)?;
            bytes_written += 1;
            remaining >>= 7;
        }
        
        self.write_u8(remaining as u8)?;
        bytes_written += 1;
        
        self.update_position(bytes_written as u64)
    }

    pub fn write_varint_u32(&self, value: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.write_varint(value as u64)
    }

    pub fn write_zigzag_i32(&self, value: i32) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = self.zigzag_encode_i32(value);
        self.write_u32(encoded)
    }

    fn zigzag_encode_i32(&self, value: i32) -> u32 {
        let n = value;
        let sign = if n < 0 { -1 } else { 1 };
        let abs_n = n.abs();
        
        let mut encoded = 0u32;
        let mut i = 0;
        
        while i < 16 {
            let bit_pos = if i < 8 {
                i * 2 + 1
            } else {
                (i - 8) * 2
            };
            
            if (abs_n >> bit_pos) & 1 == 1 {
                encoded |= 1 << i;
            }
            
            i += 1;
        }
        
        (encoded as i32) * sign
    }

    pub fn write_aligned(&self, data: &[u8], alignment: u64) -> Result<(), Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let padding_needed = (alignment - (current_pos % alignment)) % alignment;
        
        self.write_bytes(data)?;
        
        for _ in 0..padding_needed {
            self.write_u8(0)?;
        }
        
        Ok(())
    }

    pub fn write_zero_terminated(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.write_bytes(data)?;
        self.write_u8(0)
    }

    pub fn write_line(&self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_string(line)?;
        self.write_bytes(b"\r\n")
    }

    pub fn write_c_line(&self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_string(line)?;
        self.write_u8(0)
    }

    pub fn write_buffer(&self, buffer: &crate::binary_buffer::BinaryBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = buffer.bytes();
        self.write_bytes(&bytes)
    }

    pub fn write_buffer_slice(&self, buffer: &crate::binary_buffer::BinaryBuffer, offset: usize, length: usize) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = buffer.bytes();
        let slice = &bytes[offset..offset + length];
        self.write_bytes(slice)
    }

    pub fn write_hex(&self, hex_string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = hex::decode(hex_string)?;
        self.write_bytes(&bytes)
    }

    pub fn write_base64(&self, base64_string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = base64::decode(base64_string)?;
        self.write_bytes(&bytes)
    }

    pub fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                file_clone.flush()?;
            },
            WriterDestination::Memory(_) => {
            },
            WriterDestination::Stream(stream) => {
                let mut stream_clone = &**stream;
                stream_clone.flush()?;
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for flushing".into());
            },
        }
        
        let _ = self.event_sender.send(WriterEvent::FlushCompleted);
        Ok(())
    }

    pub fn seek(&self, position: u64) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                file_clone.seek(SeekFrom::Start(position))?;
                
                let mut current_position = self.position.write();
                *current_position = position;
                
                let _ = self.event_sender.send(WriterEvent::PositionChanged(position));
            },
            WriterDestination::Memory(buffer) => {
                if position > buffer.len() as u64 {
                    return Err("Seek position beyond memory buffer length".into());
                }
                
                let mut current_position = self.position.write();
                *current_position = position;
                
                let _ = self.event_sender.send(WriterEvent::PositionChanged(position));
            },
            WriterDestination::Stream(_) => {
                return Err("Seeking not supported on stream destinations".into());
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for seeking".into());
            },
        }
        
        Ok(())
    }

    pub fn seek_relative(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let new_pos = (current_pos as i64 + offset).max(0) as u64;
        self.seek(new_pos)
    }

    pub fn seek_from_end(&self, offset: i64) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                let metadata = file_clone.metadata()?;
                let file_size = metadata.len();
                let new_pos = (file_size as i64 + offset).max(0) as u64;
                file_clone.seek(SeekFrom::End(new_pos))?;
                
                let mut current_position = self.position.write();
                *current_position = new_pos;
                
                let _ = self.event_sender.send(WriterEvent::PositionChanged(new_pos));
            },
            WriterDestination::Memory(buffer) => {
                let buffer_size = buffer.len() as u64;
                let new_pos = (buffer_size as i64 + offset).max(0) as u64;
                
                if new_pos > buffer_size {
                    return Err("Seek position beyond memory buffer length".into());
                }
                
                let mut current_position = self.position.write();
                *current_position = new_pos;
                
                let _ = self.event_sender.send(WriterEvent::PositionChanged(new_pos));
            },
            WriterDestination::Stream(_) => {
                return Err("Seeking from end not supported on stream destinations".into());
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for seeking".into());
            },
        }
        
        Ok(())
    }

    pub fn truncate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let mut file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                file_clone.set_len(self.position.read())?;
            },
            WriterDestination::Memory(buffer) => {
                let current_pos = self.position.read();
                buffer.resize(current_pos as usize, 0)?;
            },
            WriterDestination::Stream(_) => {
                return Err("Truncating not supported on stream destinations".into());
            },
            WriterDestination::Custom(_) => {
                return Err("Custom destination not implemented for truncating".into());
            },
        }
        
        Ok(())
    }

    pub fn position(&self) -> u64 {
        *self.position.read()
    }

    pub fn set_endianness(&self, endianness: crate::binary_buffer::Endianness) {
        let mut current_endianness = self.endianness.write();
        *current_endianness = endianness;
    }

    pub fn get_endianness(&self) -> crate::binary_buffer::Endianness {
        *self.endianness.read()
    }

    pub fn get_size(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let destination = self.destination.read();
        
        match &*destination {
            WriterDestination::File(file) => {
                let file_clone = file.try_clone().ok_or("Failed to clone file handle")?;
                let metadata = file_clone.metadata()?;
                Ok(metadata.len())
            },
            WriterDestination::Memory(buffer) => {
                Ok(buffer.len() as u64)
            },
            WriterDestination::Stream(_) => {
                Err("Cannot get size of stream destination".into())
            },
            WriterDestination::Custom(_) => {
                Err("Cannot get size of custom destination".into())
            },
        }
    }

    pub fn remaining(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let total_size = self.get_size()?;
        let current_pos = self.position.read();
        Ok(total_size - current_pos)
    }

    pub fn is_at_end(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let remaining = self.remaining()?;
        Ok(remaining == 0)
    }

    pub fn align(&self, alignment: u64) -> Result<u64, Box<dyn std::error::Error>> {
        let current_pos = self.position.read();
        let padding = (alignment - (current_pos % alignment)) % alignment;
        self.write_bytes(&vec![0u8; padding as usize])
    }

    pub fn clone_writer(&self) -> BinaryWriter {
        let destination = self.destination.read().clone();
        
        let mut new_writer = Self::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            destination,
        );
        
        let endianness = self.endianness.read();
        new_writer.set_endianness(*endianness);
        
        new_writer
    }

    fn update_position(&self, bytes_written: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_position = self.position.write();
        *current_position += bytes_written;
        
        let _ = self.event_sender.send(WriterEvent::PositionChanged(*current_position));
        Ok(())
    }

    pub async fn get_events(&mut self) -> Vec<WriterEvent> {
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

    pub fn get_stats(&self) -> WriterStats {
        WriterStats {
            bytes_written: self.position(),
            flushes_performed: 0,
            writes_performed: 0,
            current_position: self.position(),
            buffer_utilization: 0.0,
        }
    }

    pub fn set_config(&self, config: WriterConfig) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn get_config(&self) -> WriterConfig {
        WriterConfig {
            buffer_size: 4096,
            auto_flush: false,
            create_dirs: false,
            append_mode: false,
            truncate_mode: true,
        }
    }
}

impl Default for BinaryWriter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Writer".to_string(),
            WriterDestination::Memory(Arc::new(crate::binary_buffer::BinaryBuffer::default())),
        )
    }
}

impl Default for WriterDestination {
    fn default() -> Self {
        WriterDestination::Memory(Arc::new(crate::binary_buffer::BinaryBuffer::default()))
    }
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            auto_flush: false,
            create_dirs: false,
            append_mode: false,
            truncate_mode: true,
        }
    }
}

impl Default for WriterStats {
    fn default() -> Self {
        Self {
            bytes_written: 0,
            flushes_performed: 0,
            writes_performed: 0,
            current_position: 0,
            buffer_utilization: 0.0,
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
