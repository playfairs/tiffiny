use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPool {
    pub id: Uuid,
    pub name: String,
    pub total_size_bytes: u64,
    pub allocated_bytes: u64,
    pub block_size_bytes: u64,
    pub allocation_strategy: AllocationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    FirstFit,
    BestFit,
    WorstFit,
    BuddySystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub id: Uuid,
    pub pool_id: Uuid,
    pub offset: u64,
    pub size: u64,
    pub is_allocated: bool,
    pub allocation_id: Option<Uuid>,
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    pub id: Uuid,
    pub pool_id: Uuid,
    pub size_bytes: u64,
    pub allocated_at: std::time::SystemTime,
    pub access_count: u64,
    pub last_accessed: Option<std::time::SystemTime>,
    pub data_type: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_pools: usize,
    pub total_memory_bytes: u64,
    pub allocated_bytes: u64,
    pub free_bytes: u64,
    pub fragmentation_ratio: f64,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub peak_usage_bytes: u64,
}

pub struct MemoryManager {
    pools: Arc<RwLock<HashMap<Uuid, MemoryPool>>>,
    blocks: Arc<RwLock<HashMap<Uuid, MemoryBlock>>>,
    allocations: Arc<RwLock<HashMap<Uuid, MemoryAllocation>>>,
    stats: Arc<RwLock<MemoryStats>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            blocks: Arc::new(RwLock::new(HashMap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MemoryStats {
                total_pools: 0,
                total_memory_bytes: 0,
                allocated_bytes: 0,
                free_bytes: 0,
                fragmentation_ratio: 0.0,
                allocation_count: 0,
                deallocation_count: 0,
                peak_usage_bytes: 0,
            })),
        }
    }

    pub fn create_pool(
        &self,
        name: String,
        size_bytes: u64,
        block_size_bytes: u64,
        strategy: AllocationStrategy,
    ) -> Result<Uuid> {
        let pool_id = Uuid::new_v4();
        let pool = MemoryPool {
            id: pool_id,
            name,
            total_size_bytes: size_bytes,
            allocated_bytes: 0,
            block_size_bytes,
            allocation_strategy: strategy,
        };

        {
            let mut pools = self.pools.write();
            pools.insert(pool_id, pool);
        }

        let initial_block = MemoryBlock {
            id: Uuid::new_v4(),
            pool_id,
            offset: 0,
            size: size_bytes,
            is_allocated: false,
            allocation_id: None,
            data_type: None,
        };

        {
            let mut blocks = self.blocks.write();
            blocks.insert(initial_block.id, initial_block);
        }

        self.update_stats();

        Ok(pool_id)
    }

    pub fn allocate(
        &self,
        pool_id: Uuid,
        size_bytes: u64,
        data_type: String,
        metadata: HashMap<String, String>,
    ) -> Result<Uuid> {
        let pool = {
            let pools = self.pools.read();
            pools.get(&pool_id).cloned()
                .ok_or_else(|| CoreError::Memory(format!("Pool {} not found", pool_id)))?
        };

        let allocation_id = Uuid::new_v4();
        let allocation = MemoryAllocation {
            id: allocation_id,
            pool_id,
            size_bytes,
            allocated_at: std::time::SystemTime::now(),
            access_count: 0,
            last_accessed: None,
            data_type,
            metadata,
        };

        let block_id = self.find_suitable_block(&pool, size_bytes)?;
        self.split_and_allocate_block(block_id, allocation_id, size_bytes)?;

        {
            let mut allocations = self.allocations.write();
            allocations.insert(allocation_id, allocation);
        }

        {
            let mut pools = self.pools.write();
            if let Some(pool) = pools.get_mut(&pool_id) {
                pool.allocated_bytes += size_bytes;
            }
        }

        {
            let mut stats = self.stats.write();
            stats.allocation_count += 1;
            stats.allocated_bytes += size_bytes;
            stats.free_bytes = stats.total_memory_bytes - stats.allocated_bytes;
            if stats.allocated_bytes > stats.peak_usage_bytes {
                stats.peak_usage_bytes = stats.allocated_bytes;
            }
        }

        self.update_fragmentation_ratio();

        Ok(allocation_id)
    }

    pub fn deallocate(&self, allocation_id: Uuid) -> Result<()> {
        let allocation = {
            let allocations = self.allocations.read();
            allocations.get(&allocation_id).cloned()
                .ok_or_else(|| CoreError::Memory(format!("Allocation {} not found", allocation_id)))?
        };

        self.free_allocation_blocks(allocation_id)?;

        {
            let mut pools = self.pools.write();
            if let Some(pool) = pools.get_mut(&allocation.pool_id) {
                pool.allocated_bytes -= allocation.size_bytes;
            }
        }

        {
            let allocations = self.allocations.read();
            allocations.remove(&allocation_id);
        }

        {
            let mut stats = self.stats.write();
            stats.deallocation_count += 1;
            stats.allocated_bytes -= allocation.size_bytes;
            stats.free_bytes = stats.total_memory_bytes - stats.allocated_bytes;
        }

        self.update_fragmentation_ratio();

        Ok(())
    }

    pub fn access_allocation(&self, allocation_id: Uuid) -> Result<()> {
        let mut allocations = self.allocations.write();
        if let Some(allocation) = allocations.get_mut(&allocation_id) {
            allocation.access_count += 1;
            allocation.last_accessed = Some(std::time::SystemTime::now());
            Ok(())
        } else {
            Err(CoreError::Memory(format!("Allocation {} not found", allocation_id)))
        }
    }

    pub fn get_allocation_info(&self, allocation_id: Uuid) -> Option<MemoryAllocation> {
        let allocations = self.allocations.read();
        allocations.get(&allocation_id).cloned()
    }

    pub fn get_pool_info(&self, pool_id: Uuid) -> Option<MemoryPool> {
        let pools = self.pools.read();
        pools.get(&pool_id).cloned()
    }

    pub fn list_pools(&self) -> Vec<MemoryPool> {
        let pools = self.pools.read();
        pools.values().cloned().collect()
    }

    pub fn list_allocations(&self, pool_id: Option<Uuid>) -> Vec<MemoryAllocation> {
        let allocations = self.allocations.read();
        if let Some(pool_id) = pool_id {
            allocations
                .values()
                .filter(|a| a.pool_id == pool_id)
                .cloned()
                .collect()
        } else {
            allocations.values().cloned().collect()
        }
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        let stats = self.stats.read();
        stats.clone()
    }

    pub fn compact_pool(&self, pool_id: Uuid) -> Result<()> {
        let pool = {
            let pools = self.pools.read();
            pools.get(&pool_id).cloned()
                .ok_or_else(|| CoreError::Memory(format!("Pool {} not found", pool_id)))?
        };

        self.coalesce_free_blocks(&pool);

        Ok(())
    }

    pub fn defragment_pools(&self) -> Result<()> {
        let pools = self.pools.read();
        for pool in pools.values() {
            if let Err(e) = self.coalesce_free_blocks(pool) {
                tracing::warn!("Failed to coalesce blocks in pool {}: {}", pool.id, e);
            }
        }
        Ok(())
    }

    pub fn cleanup_old_allocations(&self, older_than: std::time::Duration) -> Result<usize> {
        let cutoff = std::time::SystemTime::now() - older_than;
        let mut removed_count = 0;

        let allocations_to_remove: Vec<Uuid> = {
            let allocations = self.allocations.read();
            allocations
                .values()
                .filter(|a| {
                    a.last_accessed
                        .map(|last| last < cutoff)
                        .unwrap_or(false)
                })
                .map(|a| a.id)
                .collect()
        };

        for allocation_id in allocations_to_remove {
            if self.deallocate(allocation_id).is_ok() {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    fn find_suitable_block(&self, pool: &MemoryPool, size_bytes: u64) -> Result<Uuid> {
        let blocks = self.blocks.read();
        let suitable_blocks: Vec<_> = blocks
            .values()
            .filter(|block| {
                block.pool_id == pool.id && !block.is_allocated && block.size >= size_bytes
            })
            .collect();

        if suitable_blocks.is_empty() {
            return Err(CoreError::Memory("No suitable block found".to_string()));
        }

        let selected_block = match pool.allocation_strategy {
            AllocationStrategy::FirstFit => {
                suitable_blocks.into_iter().min_by_key(|b| b.offset)
            },
            AllocationStrategy::BestFit => {
                suitable_blocks.into_iter().min_by_key(|b| b.size)
            },
            AllocationStrategy::WorstFit => {
                suitable_blocks.into_iter().max_by_key(|b| b.size)
            },
            AllocationStrategy::BuddySystem => {
                suitable_blocks.into_iter().min_by_key(|b| b.size)
            },
        };

        selected_block
            .map(|block| block.id)
            .ok_or_else(|| CoreError::Memory("Failed to select block".to_string()))
    }

    fn split_and_allocate_block(
        &self,
        block_id: Uuid,
        allocation_id: Uuid,
        size_bytes: u64,
    ) -> Result<()> {
        let mut blocks = self.blocks.write();
        let block = blocks.get_mut(&block_id)
            .ok_or_else(|| CoreError::Memory("Block not found".to_string()))?;

        if block.size < size_bytes {
            return Err(CoreError::Memory("Block too small".to_string()));
        }

        let remaining_size = block.size - size_bytes;
        block.size = size_bytes;
        block.is_allocated = true;
        block.allocation_id = Some(allocation_id);

        if remaining_size > 0 {
            let new_block = MemoryBlock {
                id: Uuid::new_v4(),
                pool_id: block.pool_id,
                offset: block.offset + size_bytes,
                size: remaining_size,
                is_allocated: false,
                allocation_id: None,
                data_type: None,
            };
            blocks.insert(new_block.id, new_block);
        }

        Ok(())
    }

    fn free_allocation_blocks(&self, allocation_id: Uuid) -> Result<()> {
        let mut blocks = self.blocks.write();
        for block in blocks.values_mut() {
            if block.allocation_id == Some(allocation_id) {
                block.is_allocated = false;
                block.allocation_id = None;
            }
        }
        Ok(())
    }

    fn coalesce_free_blocks(&self, pool: &MemoryPool) -> Result<()> {
        let mut blocks = self.blocks.write();
        let mut pool_blocks: Vec<_> = blocks
            .values_mut()
            .filter(|b| b.pool_id == pool.id)
            .collect();

        pool_blocks.sort_by_key(|b| b.offset);

        let mut coalesced = Vec::new();
        let mut current = pool_blocks.first()
            .ok_or_else(|| CoreError::Memory("No blocks found".to_string()))?
            .clone();

        for block in pool_blocks.iter().skip(1) {
            if !current.is_allocated && !block.is_allocated 
                && current.offset + current.size == block.offset {
                current.size += block.size;
            } else {
                coalesced.push(current);
                current = block.clone();
            }
        }

        coalesced.push(current);

        for block in &coalesced {
            blocks.insert(block.id, (*block).clone());
        }

        Ok(())
    }

    fn update_stats(&self) {
        let pools = self.pools.read();
        let blocks = self.blocks.read();
        let allocations = self.allocations.read();

        let total_memory: u64 = pools.values().map(|p| p.total_size_bytes).sum();
        let allocated_memory: u64 = pools.values().map(|p| p.allocated_bytes).sum();
        let free_blocks = blocks.values().filter(|b| !b.is_allocated).count();

        let mut stats = self.stats.write();
        stats.total_pools = pools.len();
        stats.total_memory_bytes = total_memory;
        stats.allocated_bytes = allocated_memory;
        stats.free_bytes = total_memory - allocated_memory;
        stats.allocation_count = allocations.len();
    }

    fn update_fragmentation_ratio(&self) {
        let blocks = self.blocks.read();
        let free_blocks = blocks.values().filter(|b| !b.is_allocated).count();
        let total_blocks = blocks.len();

        let fragmentation_ratio = if total_blocks > 0 {
            free_blocks as f64 / total_blocks as f64
        } else {
            0.0
        };

        let mut stats = self.stats.write();
        stats.fragmentation_ratio = fragmentation_ratio;
    }
}
