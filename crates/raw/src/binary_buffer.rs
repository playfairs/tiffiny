use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BinaryBuffer {
    pub id: String,
    pub name: String,
    pub data: Arc<RwLock<Vec<u8>>>,
    pub capacity: Arc<RwLock<usize>>,
    pub position: Arc<RwLock<usize>>,
    pub endianness: Arc<RwLock<Endianness>>,
    pub event_sender: mpsc::UnboundedSender<BufferEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<BufferEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Endianness {
    Little,
    Big,
    Native,
}

#[derive(Debug, Clone)]
pub enum BufferEvent {
    DataChanged,
    PositionChanged(usize),
    CapacityChanged(usize),
    EndiannessChanged(Endianness),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct BufferView {
    pub buffer: Arc<BinaryBuffer>,
    pub offset: usize,
    pub length: usize,
    pub endianness: Option<Endianness>,
}

#[derive(Debug, Clone)]
pub struct BufferSlice {
    pub data: Vec<u8>,
    pub offset: usize,
    pub length: usize,
}

impl BinaryBuffer {
    pub fn new(id: String, name: String, capacity: usize) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            data: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            capacity: Arc::new(RwLock::new(capacity)),
            position: Arc::new(RwLock::new(0)),
            endianness: Arc::new(RwLock::new(Endianness::Native)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_bytes(id: String, name: String, bytes: Vec<u8>) -> Self {
        let capacity = bytes.len();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            data: Arc::new(RwLock::new(bytes)),
            capacity: Arc::new(RwLock::new(capacity)),
            position: Arc::new(RwLock::new(0)),
            endianness: Arc::new(RwLock::new(Endianness::Native)),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Binary Buffer".to_string(),
            capacity,
        )
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8, Box<dyn std::error::Error>> {
        let data = self.data.read();
        if offset < data.len() {
            Ok(data[offset])
        } else {
            Err("Offset out of bounds".into())
        }
    }

    pub fn read_u16(&self, offset: usize) -> Result<u16, Box<dyn std::error::Error>> {
        let data = self.data.read();
        let endianness = self.endianness.read();
        
        if offset + 1 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = &data[offset..=offset + 1];
        
        match *endianness {
            Endianness::Little => Ok(u16::from_le_bytes([bytes[0], bytes[1]])),
            Endianness::Big => Ok(u16::from_be_bytes([bytes[0], bytes[1]])),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u16::from_le_bytes([bytes[0], bytes[1]]);
                #[cfg(target_endian = "big")]
                let result = u16::from_be_bytes([bytes[0], bytes[1]]);
                result
            },
        }
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, Box<dyn std::error::Error>> {
        let data = self.data.read();
        let endianness = self.endianness.read();
        
        if offset + 3 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = &data[offset..=offset + 3];
        
        match *endianness {
            Endianness::Little => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            Endianness::Big => Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                #[cfg(target_endian = "big")]
                let result = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                result
            },
        }
    }

    pub fn read_u64(&self, offset: usize) -> Result<u64, Box<dyn std::error::Error>> {
        let data = self.data.read();
        let endianness = self.endianness.read();
        
        if offset + 7 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = &data[offset..=offset + 7];
        
        match *endianness {
            Endianness::Little => Ok(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            Endianness::Big => Ok(u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                #[cfg(target_endian = "big")]
                let result = u64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                result
            },
        }
    }

    pub fn read_i8(&self, offset: usize) -> Result<i8, Box<dyn std::error::Error>> {
        self.read_u8(offset).map(|v| v as i8)
    }

    pub fn read_i16(&self, offset: usize) -> Result<i16, Box<dyn std::error::Error>> {
        self.read_u16(offset).map(|v| v as i16)
    }

    pub fn read_i32(&self, offset: usize) -> Result<i32, Box<dyn std::error::Error>> {
        self.read_u32(offset).map(|v| v as i32)
    }

    pub fn read_i64(&self, offset: usize) -> Result<i64, Box<dyn std::error::Error>> {
        self.read_u64(offset).map(|v| v as i64)
    }

    pub fn read_f32(&self, offset: usize) -> Result<f32, Box<dyn std::error::Error>> {
        self.read_u32(offset).map(|v| f32::from_bits(v))
    }

    pub fn read_f64(&self, offset: usize) -> Result<f64, Box<dyn std::error::Error>> {
        self.read_u64(offset).map(|v| f64::from_bits(v))
    }

    pub fn read_bytes(&self, offset: usize, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = self.data.read();
        
        if offset + length > data.len() {
            return Err("Read out of bounds".into());
        }

        Ok(data[offset..offset + length].to_vec())
    }

    pub fn read_string(&self, offset: usize, length: usize) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.read_bytes(offset, length)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn read_c_string(&self, offset: usize) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.data.read();
        let mut end_offset = offset;
        
Find null terminator
        while end_offset < data.len() && data[end_offset] != 0 {
            end_offset += 1;
        }

        if end_offset >= data.len() {
            return Err("No null terminator found".into());
        }

        let bytes = &data[offset..end_offset];
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    pub fn write_u8(&self, offset: usize, value: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        
        if offset >= data.len() {
            return Err("Offset out of bounds".into());
        }

        data[offset] = value;
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn write_u16(&self, offset: usize, value: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let endianness = self.endianness.read();
        
        if offset + 1 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = match *endianness {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        data[offset] = bytes[0];
        data[offset + 1] = bytes[1];
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn write_u32(&self, offset: usize, value: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let endianness = self.endianness.read();
        
        if offset + 3 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = match *endianness {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        data[offset] = bytes[0];
        data[offset + 1] = bytes[1];
        data[offset + 2] = bytes[2];
        data[offset + 3] = bytes[3];
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn write_u64(&self, offset: usize, value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let endianness = self.endianness.read();
        
        if offset + 7 >= data.len() {
            return Err("Offset out of bounds".into());
        }

        let bytes = match *endianness {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
            Endianness::Native => {
                #[cfg(target_endian = "little")]
                let result = value.to_le_bytes();
                #[cfg(target_endian = "big")]
                let result = value.to_be_bytes();
                result
            },
        };

        data[offset] = bytes[0];
        data[offset + 1] = bytes[1];
        data[offset + 2] = bytes[2];
        data[offset + 3] = bytes[3];
        data[offset + 4] = bytes[4];
        data[offset + 5] = bytes[5];
        data[offset + 6] = bytes[6];
        data[offset + 7] = bytes[7];
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn write_i8(&self, offset: usize, value: i8) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u8(offset, value as u8)
    }

    pub fn write_i16(&self, offset: usize, value: i16) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u16(offset, value as u16)
    }

    pub fn write_i32(&self, offset: usize, value: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u32(offset, value as u32)
    }

    pub fn write_i64(&self, offset: usize, value: i64) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u64(offset, value as u64)
    }

    pub fn write_f32(&self, offset: usize, value: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u32(offset, value.to_bits())
    }

    pub fn write_f64(&self, offset: usize, value: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.write_u64(offset, value.to_bits())
    }

    pub fn write_bytes(&self, offset: usize, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        
        if offset + bytes.len() > data.len() {
            return Err("Write out of bounds".into());
        }

        data[offset..offset + bytes.len()].copy_from_slice(bytes);
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn write_string(&self, offset: usize, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        self.write_bytes(offset, bytes)
    }

    pub fn write_c_string(&self, offset: usize, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        let mut data = self.data.write();
        
        if offset + bytes.len() + 1 > data.len() {
            return Err("Write out of bounds".into());
        }

        data[offset..offset + bytes.len()].copy_from_slice(bytes);
        data[offset + bytes.len()] = 0;
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn append_bytes(&self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let mut capacity = self.capacity.write();
        
        let current_len = data.len();
        let additional_len = bytes.len();
        
        if current_len + additional_len > *capacity {
            data.resize(current_len + additional_len, 0);
            *capacity = current_len + additional_len;
            
            let _ = self.event_sender.send(BufferEvent::CapacityChanged(*capacity));
        }

        data[current_len..].copy_from_slice(bytes);
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn append_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.append_bytes(string.as_bytes())
    }

    pub fn append_c_string(&self, string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = string.as_bytes();
        let mut data = self.data.write();
        let mut capacity = self.capacity.write();
        
        let current_len = data.len();
        let additional_len = bytes.len() + 1;
        
        if current_len + additional_len > *capacity {
            data.resize(current_len + additional_len, 0);
            *capacity = current_len + additional_len;
            
            let _ = self.event_sender.send(BufferEvent::CapacityChanged(*capacity));
        }

        data[current_len..current_len + bytes.len()].copy_from_slice(bytes);
        data[current_len + bytes.len()] = 0;
        
        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(())
    }

    pub fn resize(&self, new_capacity: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let mut capacity = self.capacity.write();
        
        if new_capacity < data.len() {
            return Err("Cannot resize to smaller than current data length".into());
        }

        data.resize(new_capacity, 0);
        *capacity = new_capacity;
        
        let _ = self.event_sender.send(BufferEvent::CapacityChanged(new_capacity));
        Ok(())
    }

    pub fn clear(&self) {
        let mut data = self.data.write();
        let mut position = self.position.write();
        
        data.clear();
        *position = 0;
        
        let _ = self.event_sender.send(BufferEvent::PositionChanged(0));
        let _ = self.event_sender.send(BufferEvent::DataChanged);
    }

    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    pub fn capacity(&self) -> usize {
        *self.capacity.read()
    }

    pub fn position(&self) -> usize {
        *self.position.read()
    }

    pub fn set_position(&self, position: usize) {
        let mut current_position = self.position.write();
        *current_position = position.clamp(0, self.len());
        
        let _ = self.event_sender.send(BufferEvent::PositionChanged(*current_position));
    }

    pub fn remaining(&self) -> usize {
        self.len() - self.position()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.data.read().clone()
    }

    pub fn slice(&self, offset: usize, length: usize) -> Result<BufferSlice, Box<dyn std::error::Error>> {
        let data = self.data.read();
        
        if offset + length > data.len() {
            return Err("Slice out of bounds".into());
        }

        Ok(BufferSlice {
            data: data[offset..offset + length].to_vec(),
            offset,
            length,
        })
    }

    pub fn view(&self, offset: usize, length: usize) -> Result<BufferView, Box<dyn std::error::Error>> {
        let data = self.data.read();
        
        if offset + length > data.len() {
            return Err("View out of bounds".into());
        }

        Ok(BufferView {
            buffer: Arc::new(self.clone()),
            offset,
            length,
            endianness: Some(*self.endianness.read()),
        })
    }

    pub fn find_bytes(&self, pattern: &[u8], start_offset: usize) -> Option<usize> {
        let data = self.data.read();
        
        if start_offset >= data.len() || pattern.is_empty() {
            return None;
        }

        for i in start_offset..=data.len() - pattern.len() {
            if data[i..i + pattern.len()] == pattern {
                return Some(i);
            }
        }

        None
    }

    pub fn find_string(&self, pattern: &str, start_offset: usize) -> Option<usize> {
        self.find_bytes(pattern.as_bytes(), start_offset)
    }

    pub fn replace_bytes(&self, pattern: &[u8], replacement: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut data = self.data.write();
        let mut count = 0;
        let mut i = 0;
        
        while i <= data.len() - pattern.len() {
            if data[i..i + pattern.len()] == pattern {
                data[i..i + replacement.len()].copy_from_slice(replacement);
                count += 1;
                i += replacement.len();
            } else {
                i += 1;
            }
        }

        let _ = self.event_sender.send(BufferEvent::DataChanged);
        Ok(count)
    }

    pub fn replace_string(&self, pattern: &str, replacement: &str) -> Result<usize, Box<dyn std::error::Error>> {
        self.replace_bytes(pattern.as_bytes(), replacement.as_bytes())
    }

    pub fn copy_from(&self, other: &BinaryBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let other_data = other.bytes();
        self.clear();
        self.append_bytes(&other_data)
    }

    pub fn clone_buffer(&self) -> BinaryBuffer {
        let data = self.bytes();
        BinaryBuffer::from_bytes(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", self.name),
            data,
        )
    }

    pub fn set_endianness(&self, endianness: Endianness) {
        let mut current_endianness = self.endianness.write();
        *current_endianness = endianness;
        
        let _ = self.event_sender.send(BufferEvent::EndiannessChanged(endianness));
    }

    pub fn get_endianness(&self) -> Endianness {
        *self.endianness.read()
    }

    pub fn to_hex(&self) -> String {
        let data = self.data.read();
        hex::encode(&data)
    }

    pub fn from_hex(&self, hex_string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = hex::decode(hex_string)?;
        self.clear();
        self.append_bytes(&bytes)
    }

    pub fn calculate_checksum(&self, algorithm: ChecksumAlgorithm) -> Vec<u8> {
        let data = self.data.read();
        
        match algorithm {
            ChecksumAlgorithm::CRC32 => {
                let mut hasher = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
                hasher.update(&data);
                hasher.finalize().to_be_bytes().to_vec()
            },
            ChecksumAlgorithm::MD5 => {
                use md5;
                let mut hasher = md5::Context::new();
                hasher.consume(&data);
                hasher.compute().0.to_vec()
            },
            ChecksumAlgorithm::SHA1 => {
                use sha1;
                let mut hasher = sha1::Sha1::new();
                hasher.update(&data);
                hasher.digest().bytes().to_vec()
            },
            ChecksumAlgorithm::SHA256 => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hasher.finalize().to_vec()
            },
        }
    }

    pub async fn get_events(&mut self) -> Vec<BufferEvent> {
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

    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            id: self.id.clone(),
            name: self.name.clone(),
            length: self.len(),
            capacity: self.capacity(),
            position: self.position(),
            remaining: self.remaining(),
            endianness: self.get_endianness(),
            is_empty: self.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChecksumAlgorithm {
    CRC32,
    MD5,
    SHA1,
    SHA256,
}

#[derive(Debug, Clone)]
pub struct BufferStats {
    pub id: String,
    pub name: String,
    pub length: usize,
    pub capacity: usize,
    pub position: usize,
    pub remaining: usize,
    pub endianness: Endianness,
    pub is_empty: bool,
}

impl BufferView {
    pub fn read_u8(&self, offset: usize) -> Result<u8, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset < self.offset + self.length {
            self.buffer.read_u8(actual_offset)
        } else {
            Err("View offset out of bounds".into())
        }
    }

    pub fn read_u16(&self, offset: usize) -> Result<u16, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset < self.offset + self.length {
            self.buffer.read_u16(actual_offset)
        } else {
            Err("View offset out of bounds".into())
        }
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset < self.offset + self.length {
            self.buffer.read_u32(actual_offset)
        } else {
            Err("View offset out of bounds".into())
        }
    }

    pub fn read_u64(&self, offset: usize) -> Result<u64, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset < self.offset + self.length {
            self.buffer.read_u64(actual_offset)
        } else {
            Err("View offset out of bounds".into())
        }
    }

    pub fn read_bytes(&self, offset: usize, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset + length <= self.offset + self.length {
            self.buffer.read_bytes(actual_offset, length)
        } else {
            Err("View read out of bounds".into())
        }
    }

    pub fn read_string(&self, offset: usize, length: usize) -> Result<String, Box<dyn std::error::Error>> {
        let actual_offset = self.offset + offset;
        if actual_offset + length <= self.offset + self.length {
            self.buffer.read_string(actual_offset, length)
        } else {
            Err("View read out of bounds".into())
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn to_slice(&self) -> BufferSlice {
        let data = self.buffer.bytes();
        BufferSlice {
            data: data[self.offset..self.offset + self.length].to_vec(),
            offset: self.offset,
            length: self.length,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let data = self.buffer.bytes();
        data[self.offset..self.offset + self.length].to_vec()
    }
}

impl BufferSlice {
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.length {
            Some(self.data[index])
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

impl Default for BinaryBuffer {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::Native
    }
}

impl Default for BufferView {
    fn default() -> Self {
        Self {
            buffer: Arc::new(BinaryBuffer::default()),
            offset: 0,
            length: 0,
            endianness: None,
        }
    }
}

impl Default for BufferSlice {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            offset: 0,
            length: 0,
        }
    }
}

impl Default for BufferStats {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            length: 0,
            capacity: 0,
            position: 0,
            remaining: 0,
            endianness: Endianness::Native,
            is_empty: true,
        }
    }
}
