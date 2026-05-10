use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use memmap2::MmapOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buffer {
    pub id: Uuid,
    pub name: String,
    pub buffer_type: BufferType,
    pub size_bytes: u64,
    pub format: DataFormat,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BufferType {
    Audio,
    Image,
    Video,
    Raw,
    Metadata,
    Temporary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    AudioPCM,
    AudioFloat32,
    AudioInt16,
    ImageRGBA8,
    ImageRGB8,
    ImageGray8,
    VideoRaw,
    VideoEncoded,
    RawBinary,
    RawText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferHandle {
    pub buffer_id: Uuid,
    pub offset: u64,
    pub length: u64,
    pub access_mode: AccessMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct BufferManager {
    buffers: Arc<RwLock<HashMap<Uuid, Buffer>>>,
    memory_pool: Arc<RwLock<MemoryPool>>,
    file_buffers: Arc<RwLock<HashMap<Uuid, FileBuffer>>>,
}

impl BufferManager {
    pub fn new(total_memory_mb: u64) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(HashMap::new())),
            memory_pool: Arc::new(RwLock::new(MemoryPool::new(total_memory_mb * 1024 * 1024))),
            file_buffers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_buffer(
        &self,
        name: String,
        buffer_type: BufferType,
        size_bytes: u64,
        format: DataFormat,
    ) -> Result<BufferHandle> {
        let buffer_id = Uuid::new_v4();
        let buffer = Buffer {
            id: buffer_id,
            name,
            buffer_type,
            size_bytes,
            format,
            metadata: HashMap::new(),
        };

        {
            let mut buffers = self.buffers.write();
            buffers.insert(buffer_id, buffer);
        }

        let handle = BufferHandle {
            buffer_id,
            offset: 0,
            length: size_bytes,
            access_mode: AccessMode::ReadWrite,
        };

        Ok(handle)
    }

    pub fn allocate_memory(&self, size_bytes: u64) -> Result<Uuid> {
        let mut pool = self.memory_pool.write();
        pool.allocate(size_bytes)
    }

    pub fn deallocate_memory(&self, allocation_id: Uuid) -> Result<()> {
        let mut pool = self.memory_pool.write();
        pool.deallocate(allocation_id)
    }

    pub fn create_file_buffer(
        &self,
        file_path: String,
        buffer_type: BufferType,
        format: DataFormat,
    ) -> Result<BufferHandle> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        let file_size = file.metadata()?.len();
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        let buffer_id = Uuid::new_v4();
        let file_buffer = FileBuffer {
            path: file_path,
            mmap,
            size: file_size,
        };

        {
            let mut file_buffers = self.file_buffers.write();
            file_buffers.insert(buffer_id, file_buffer);
        }

        let buffer = Buffer {
            id: buffer_id,
            name: format!("file:{}", file_path),
            buffer_type,
            size_bytes: file_size,
            format,
            metadata: HashMap::new(),
        };

        {
            let mut buffers = self.buffers.write();
            buffers.insert(buffer_id, buffer);
        }

        let handle = BufferHandle {
            buffer_id,
            offset: 0,
            length: file_size,
            access_mode: AccessMode::ReadWrite,
        };

        Ok(handle)
    }

    pub fn read_buffer(&self, handle: &BufferHandle) -> Result<Vec<u8>> {
        let file_buffers = self.file_buffers.read();
        if let Some(file_buffer) = file_buffers.get(&handle.buffer_id) {
            if handle.offset + handle.length > file_buffer.size {
                return Err(CoreError::Memory("Read beyond buffer bounds".to_string()));
            }

            let start = handle.offset as usize;
            let end = (handle.offset + handle.length) as usize;
            Ok(file_buffer.mmap[start..end].to_vec())
        } else {
            Err(CoreError::Memory("Buffer not found".to_string()))
        }
    }

    pub fn write_buffer(&self, handle: &mut BufferHandle, data: &[u8]) -> Result<()> {
        let mut file_buffers = self.file_buffers.write();
        if let Some(file_buffer) = file_buffers.get_mut(&handle.buffer_id) {
            if handle.offset + data.len() as u64 > file_buffer.size {
                return Err(CoreError::Memory("Write beyond buffer bounds".to_string()));
            }

            let start = handle.offset as usize;
            let end = start + data.len();
            file_buffer.mmap[start..end].copy_from_slice(data);
            Ok(())
        } else {
            Err(CoreError::Memory("Buffer not found".to_string()))
        }
    }

    pub fn get_buffer_info(&self, buffer_id: Uuid) -> Option<Buffer> {
        let buffers = self.buffers.read();
        buffers.get(&buffer_id).cloned()
    }

    pub fn list_buffers(&self) -> Vec<Buffer> {
        let buffers = self.buffers.read();
        buffers.values().cloned().collect()
    }

    pub fn delete_buffer(&self, buffer_id: Uuid) -> Result<()> {
        {
            let mut buffers = self.buffers.write();
            buffers.remove(&buffer_id);
        }

        {
            let mut file_buffers = self.file_buffers.write();
            file_buffers.remove(&buffer_id);
        }

        Ok(())
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        let pool = self.memory_pool.read();
        MemoryUsage {
            total_bytes: pool.total_size,
            allocated_bytes: pool.allocated_size,
            free_bytes: pool.total_size - pool.allocated_size,
            fragmentation_ratio: if pool.total_size > 0 {
                pool.fragmentation_count as f64 / pool.total_size as f64
            } else {
                0.0
            },
        }
    }

    pub fn compact_memory(&self) -> Result<()> {
        let mut pool = self.memory_pool.write();
        pool.compact();
        Ok(())
    }
}

struct MemoryPool {
    total_size: u64,
    allocated_size: u64,
    allocations: HashMap<Uuid, MemoryAllocation>,
    free_blocks: Vec<MemoryBlock>,
    fragmentation_count: usize,
}

impl MemoryPool {
    fn new(total_size: u64) -> Self {
        Self {
            total_size,
            allocated_size: 0,
            allocations: HashMap::new(),
            free_blocks: vec![MemoryBlock {
                offset: 0,
                size: total_size,
            }],
            fragmentation_count: 0,
        }
    }

    fn allocate(&mut self, size: u64) -> Result<Uuid> {
        let block_index = self.find_free_block(size)?;
        let block = &mut self.free_blocks[block_index];
        
        let allocation_id = Uuid::new_v4();
        let allocation = MemoryAllocation {
            id: allocation_id,
            offset: block.offset,
            size,
        };

        if block.size == size {
            self.free_blocks.remove(block_index);
        } else {
            let remaining_block = MemoryBlock {
                offset: block.offset + size,
                size: block.size - size,
            };
            self.free_blocks[block_index] = remaining_block;
        }

        self.allocations.insert(allocation_id, allocation);
        self.allocated_size += size;
        self.update_fragmentation();

        Ok(allocation_id)
    }

    fn deallocate(&mut self, allocation_id: Uuid) -> Result<()> {
        let allocation = self.allocations.remove(&allocation_id)
            .ok_or_else(|| CoreError::Memory("Allocation not found".to_string()))?;

        self.allocated_size -= allocation.size;
        
        let free_block = MemoryBlock {
            offset: allocation.offset,
            size: allocation.size,
        };
        
        self.free_blocks.push(free_block);
        self.coalesce_free_blocks();
        self.update_fragmentation();

        Ok(())
    }

    fn find_free_block(&self, size: u64) -> Result<usize> {
        self.free_blocks
            .iter()
            .position(|block| block.size >= size)
            .ok_or_else(|| CoreError::Memory("Insufficient memory".to_string()))
    }

    fn coalesce_free_blocks(&mut self) {
        if self.free_blocks.len() <= 1 {
            return;
        }

        self.free_blocks.sort_by_key(|block| block.offset);

        let mut coalesced = Vec::new();
        let mut current = self.free_blocks[0].clone();

        for block in &self.free_blocks[1..] {
            if current.offset + current.size == block.offset {
                current.size += block.size;
            } else {
                coalesced.push(current);
                current = block.clone();
            }
        }

        coalesced.push(current);
        self.free_blocks = coalesced;
    }

    fn update_fragmentation(&mut self) {
        self.fragmentation_count = self.free_blocks.len();
    }

    fn compact(&mut self) {
        self.coalesce_free_blocks();
    }
}

#[derive(Debug, Clone)]
struct MemoryAllocation {
    id: Uuid,
    offset: u64,
    size: u64,
}

#[derive(Debug, Clone)]
struct MemoryBlock {
    offset: u64,
    size: u64,
}

struct FileBuffer {
    path: String,
    mmap: memmap2::Mmap,
    size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_bytes: u64,
    pub allocated_bytes: u64,
    pub free_bytes: u64,
    pub fragmentation_ratio: f64,
}
