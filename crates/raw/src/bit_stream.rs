use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BitStream {
    pub id: String,
    pub name: String,
    pub stream_type: StreamType,
    pub buffer: Arc<RwLock<Vec<u8>>>,
    pub position: Arc<RwLock<usize>>,
    pub bit_position: Arc<RwLock<usize>>,
    pub endianness: Arc<RwLock<crate::binary_buffer::Endianness>>,
    pub event_sender: mpsc::UnboundedSender<BitStreamEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<BitStreamEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Read,
    Write,
    ReadWrite,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum BitStreamEvent {
    BitRead(u8),
    BitWritten(u8),
    ByteRead(u8),
    ByteWritten(u8),
    PositionChanged(usize),
    BitPositionChanged(usize),
    Error(String),
    BufferFull,
    BufferEmpty,
}

#[derive(Debug, Clone)]
pub struct BitStreamConfig {
    pub buffer_size: usize,
    pub auto_expand: bool,
    pub expand_size: usize,
    pub bit_order: BitOrder,
    pub bit_padding: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitOrder {
    MSBFirst,
    LSBFirst,
}

#[derive(Debug, Clone)]
pub struct BitStreamStats {
    pub bits_read: u64,
    pub bits_written: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub current_position: usize,
    pub current_bit_position: usize,
    pub buffer_size: usize,
    pub buffer_utilization: f32,
}

impl BitStream {
    pub fn new(id: String, name: String, stream_type: StreamType, buffer_size: usize) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_size)))),
            position: Arc::new(RwLock::new(0))),
            bit_position: Arc::new(RwLock::new(0))),
            endianness: Arc::new(RwLock::new(crate::binary_buffer::Endianness::Native)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn with_config(id: String, name: String, stream_type: StreamType, config: BitStreamConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(config.buffer_size)))),
            position: Arc::new(RwLock::new(0))),
            bit_position: Arc::new(RwLock::new(0))),
            endianness: Arc::new(RwLock::new(crate::binary_buffer::Endianness::Native)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub fn from_bytes(id: String, name: String, stream_type: StreamType, bytes: Vec<u8>) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            stream_type,
            buffer: Arc::new(RwLock::new(bytes))),
            position: Arc::new(RwLock::new(0))),
            bit_position: Arc::new(RwLock::new(0))),
            endianness: Arc::new(RwLock::new(crate::binary_buffer::Endianness::Native)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_sender))),
        }
    }

    pub fn read_bit(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Read | StreamType::ReadWrite => {
                let buffer = self.buffer.read();
                let position = self.position.read();
                let bit_position = self.bit_position.read();
                
                if *position >= buffer.len() {
                    return Err("Buffer empty".into());
                }

                let byte = buffer[*position];
                let bit = self.extract_bit(byte, *bit_position)?;
                
                let mut new_bit_position = *bit_position + 1;
                let mut new_position = *position;
                
                if new_bit_position >= 8 {
                    new_bit_position = 0;
                    new_position += 1;
                }
                
                let mut bit_pos = self.bit_position.write();
                *bit_pos = new_bit_position;
                
                let mut pos = self.position.write();
                *pos = new_position;
                
                let _ = self.event_sender.send(BitStreamEvent::BitRead(bit));
                let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(new_bit_position));
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(new_position));
                
                Ok(bit)
            },
            StreamType::Write => Err("Cannot read from write-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for reading".into()),
        }
    }

    pub fn read_bits(&self, count: u8) -> Result<u64, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Read | StreamType::ReadWrite => {
                let mut result = 0u64;
                
                for i in 0..count {
                    let bit = self.read_bit()?;
                    result |= (bit as u64) << i;
                }
                
                Ok(result)
            },
            StreamType::Write => Err("Cannot read from write-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for reading".into()),
        }
    }

    pub fn read_byte(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Read | StreamType::ReadWrite => {
                let buffer = self.buffer.read();
                let position = self.position.read();
                let bit_position = self.bit_position.read();
                
                if *position >= buffer.len() {
                    return Err("Buffer empty".into());
                }

                let byte = buffer[*position];
                
Reset bit position and advance byte position
                let mut bit_pos = self.bit_position.write();
                *bit_pos = 0;
                
                let mut pos = self.position.write();
                *pos += 1;
                
                let _ = self.event_sender.send(BitStreamEvent::ByteRead(byte));
                let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(0));
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(*pos));
                
                Ok(byte)
            },
            StreamType::Write => Err("Cannot read from write-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for reading".into()),
        }
    }

    pub fn read_bytes(&self, count: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Read | StreamType::ReadWrite => {
                let buffer = self.buffer.read();
                let position = self.position.read();
                let bit_position = self.bit_position.read();
                
                if *bit_position != 0 {
                    return Err("Cannot read multiple bytes when not byte-aligned".into());
                }

                if *position + count > buffer.len() {
                    return Err("Not enough bytes available".into());
                }

                let mut result = Vec::with_capacity(count);
                for i in 0..count {
                    result.push(buffer[*position + i]);
                }
                
                let mut pos = self.position.write();
                *pos += count;
                
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(*pos));
                
                Ok(result)
            },
            StreamType::Write => Err("Cannot read from write-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for reading".into()),
        }
    }

    pub fn read_aligned(&self, alignment: u8) -> Result<u64, Box<dyn std::error::Error>> {
        let bit_position = self.bit_position.read();
        let padding_needed = (alignment - (*bit_position % alignment)) % alignment;
        
        for _ in 0..padding_needed {
            self.read_bit()?;
        }
        
        self.read_bits(alignment)
    }

    pub fn read_varint(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let byte = self.read_byte()?;
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
        let encoded = self.read_varint_u32()?;
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

    pub fn write_bit(&self, bit: u8) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Write | StreamType::ReadWrite => {
                let mut buffer = self.buffer.write();
                let position = self.position.read();
                let bit_position = self.bit_position.read();
                
                if *position >= buffer.len() {
                    buffer.push(0);
                }
                
                let current_byte = buffer[*position];
                let modified_byte = self.set_bit(current_byte, *bit_position, bit)?;
                buffer[*position] = modified_byte;
                
                let mut new_bit_position = *bit_position + 1;
                let mut new_position = *position;
                
                if new_bit_position >= 8 {
                    new_bit_position = 0;
                    new_position += 1;
                }
                
                let mut bit_pos = self.bit_position.write();
                *bit_pos = new_bit_position;
                
                let mut pos = self.position.write();
                *pos = new_position;
                
                let _ = self.event_sender.send(BitStreamEvent::BitWritten(bit));
                let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(new_bit_position));
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(new_position));
                
                Ok(())
            },
            StreamType::Read => Err("Cannot write to read-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for writing".into()),
        }
    }

    pub fn write_bits(&self, bits: u64, count: u8) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Write | StreamType::ReadWrite => {
                for i in 0..count {
                    let bit = ((bits >> i) & 1) as u8;
                    self.write_bit(bit)?;
                }
                Ok(())
            },
            StreamType::Read => Err("Cannot write to read-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for writing".into()),
        }
    }

    pub fn write_byte(&self, byte: u8) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Write | StreamType::ReadWrite => {
                let mut buffer = self.buffer.write();
                let position = self.position.read();
                let bit_position = self.bit_position.read();
                
                if *bit_position != 0 {
                    return Err("Cannot write byte when not bit-aligned".into());
                }
                
                if *position >= buffer.len() {
                    buffer.push(0);
                }
                
                buffer[*position] = byte;
                
                let mut bit_pos = self.bit_position.write();
                *bit_pos = 0;
                
                let mut pos = self.position.write();
                *pos += 1;
                
                let _ = self.event_sender.send(BitStreamEvent::ByteWritten(byte));
                let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(0));
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(*pos));
                
                Ok(())
            },
            StreamType::Read => Err("Cannot write to read-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for writing".into()),
        }
    }

    pub fn write_bytes(&self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream_type {
            StreamType::Write | StreamType::ReadWrite => {
                let bit_position = self.bit_position.read();
                
                if *bit_position != 0 {
                    return Err("Cannot write multiple bytes when not bit-aligned".into());
                }
                
                let mut buffer = self.buffer.write();
                let position = self.position.read();
                
                let required_size = *position + bytes.len();
                if required_size > buffer.len() {
                    buffer.resize(required_size, 0);
                }
                
                for (i, &byte) in bytes.iter().enumerate() {
                    buffer[*position + i] = byte;
                }
                
                let mut pos = self.position.write();
                *pos += bytes.len();
                
                let _ = self.event_sender.send(BitStreamEvent::PositionChanged(*pos));
                
                Ok(())
            },
            StreamType::Read => Err("Cannot write to read-only stream".into()),
            StreamType::Custom(_) => Err("Custom stream not implemented for writing".into()),
        }
    }

    pub fn write_aligned(&self, bits: u64, alignment: u8) -> Result<(), Box<dyn std::error::Error>> {
        let bit_position = self.bit_position.read();
        let padding_needed = (alignment - (*bit_position % alignment)) % alignment;
        
        for _ in 0..padding_needed {
            self.write_bit(0)?;
        }
        
        self.write_bits(bits, alignment)
    }

    pub fn write_varint(&self, value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut remaining = value;
        
        while remaining >= 0x80 {
            let mut byte = (remaining & 0x7F) | 0x80;
            self.write_byte(byte)?;
            remaining >>= 7;
        }
        
        self.write_byte(remaining as u8)
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
        
        let mut encoded = 0;
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

    pub fn write_u32(&self, value: u32) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = value.to_le_bytes();
        self.write_bytes(&bytes)
    }

    pub fn read_u32(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn extract_bit(&self, byte: u8, bit_position: usize) -> Result<u8, Box<dyn std::error::Error>> {
        if bit_position >= 8 {
            return Err("Bit position out of range".into());
        }
        
        Ok((byte >> bit_position) & 1)
    }

    fn set_bit(&self, byte: u8, bit_position: usize, bit: u8) -> Result<u8, Box<dyn std::error::Error>> {
        if bit_position >= 8 {
            return Err("Bit position out of range".into());
        }
        
        let mask = 1 << bit_position;
        Ok((byte & !mask) | ((bit & 1) << bit_position))
    }

    pub fn align_to_byte(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bit_position = self.bit_position.read();
        let padding_needed = (8 - (*bit_position % 8)) % 8;
        
        for _ in 0..padding_needed {
            self.write_bit(0)?;
        }
        
        Ok(())
    }

    pub fn is_byte_aligned(&self) -> bool {
        *self.bit_position.read() == 0
    }

    pub fn bits_remaining(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let buffer = self.buffer.read();
        let position = self.position.read();
        let bit_position = self.bit_position.read();
        
        if *position >= buffer.len() {
            Ok(0)
        } else {
            Ok((buffer.len() - *position - 1) * 8 + (8 - *bit_position))
        }
    }

    pub fn bytes_remaining(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let buffer = self.buffer.read();
        let position = self.position.read();
        
        if *position >= buffer.len() {
            Ok(0)
        } else {
            Ok(buffer.len() - *position)
        }
    }

    pub fn position(&self) -> usize {
        *self.position.read()
    }

    pub fn bit_position(&self) -> usize {
        *self.bit_position.read()
    }

    pub fn set_position(&self, position: usize) {
        let mut pos = self.position.write();
        *pos = position;
        
        let mut bit_pos = self.bit_position.write();
        *bit_pos = 0;
        
        let _ = self.event_sender.send(BitStreamEvent::PositionChanged(position));
        let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(0));
    }

    pub fn set_bit_position(&self, position: usize, bit_position: usize) {
        let mut pos = self.position.write();
        *pos = position;
        
        let mut bit_pos = self.bit_position.write();
        *bit_pos = bit_position % 8;
        
        let _ = self.event_sender.send(BitStreamEvent::PositionChanged(position));
        let _ = self.event_sender.send(BitStreamEvent::BitPositionChanged(bit_position));
    }

    pub fn seek(&self, position: usize) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.buffer.read();
        
        if position > buffer.len() {
            return Err("Seek position out of bounds".into());
        }
        
        self.set_position(position);
        Ok(())
    }

    pub fn seek_bit(&self, position: usize, bit_position: usize) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.buffer.read();
        
        if position > buffer.len() {
            return Err("Seek position out of bounds".into());
        }
        
        self.set_bit_position(position, bit_position);
        Ok(())
    }

    pub fn rewind(&self) {
        self.set_position(0);
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        self.buffer.read().clone()
    }

    pub fn set_buffer(&self, bytes: Vec<u8>) {
        let mut buffer = self.buffer.write();
        *buffer = bytes;
        
        self.set_position(0);
    }

    pub fn clear(&self) {
        let mut buffer = self.buffer.write();
        buffer.clear();
        
        self.set_position(0);
    }

    pub fn resize(&self, new_size: usize) {
        let mut buffer = self.buffer.write();
        buffer.resize(new_size, 0);
        
        let position = self.position.read();
        if position > new_size {
            self.set_position(new_size);
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.read().len()
    }

    pub fn set_endianness(&self, endianness: crate::binary_buffer::Endianness) {
        let mut current_endianness = self.endianness.write();
        *current_endianness = endianness;
    }

    pub fn get_endianness(&self) -> crate::binary_buffer::Endianness {
        *self.endianness.read()
    }

    pub async fn get_events(&mut self) -> Vec<BitStreamEvent> {
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

    pub fn get_stats(&self) -> BitStreamStats {
        BitStreamStats {
            bits_read: self.position() as u64 * 8 + self.bit_position() as u64,
            bits_written: self.position() as u64 * 8 + self.bit_position() as u64,
            bytes_read: self.position() as u64,
            bytes_written: self.position() as u64,
            current_position: self.position(),
            current_bit_position: self.bit_position(),
            buffer_size: self.capacity(),
            buffer_utilization: (self.position() as f32 / self.capacity() as f32) * 100.0,
        }
    }

    pub fn clone_stream(&self) -> BitStream {
        let buffer = self.get_buffer();
        
        let mut new_stream = Self::from_bytes(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", self.name),
            self.stream_type.clone(),
            buffer,
        );
        
        let endianness = self.get_endianness();
        new_stream.set_endianness(endianness);
        
        new_stream
    }

    pub fn reset(&self) {
        self.clear();
    }
}

impl Default for BitStream {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Bit Stream".to_string(),
            StreamType::ReadWrite,
            4096,
        )
    }
}

impl Default for StreamType {
    fn default() -> Self {
        StreamType::ReadWrite
    }
}

impl Default for BitStreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            auto_expand: true,
            expand_size: 1024,
            bit_order: BitOrder::LSBFirst,
            bit_padding: false,
        }
    }
}

impl Default for BitOrder {
    fn default() -> Self {
        BitOrder::LSBFirst
    }
}

impl Default for BitStreamStats {
    fn default() -> Self {
        Self {
            bits_read: 0,
            bits_written: 0,
            bytes_read: 0,
            bytes_written: 0,
            current_position: 0,
            current_bit_position: 0,
            buffer_size: 0,
            buffer_utilization: 0.0,
        }
    }
}
