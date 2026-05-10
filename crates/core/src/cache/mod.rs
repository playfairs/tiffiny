use crate::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub id: Uuid,
    pub key: String,
    pub data_type: CacheDataType,
    pub size_bytes: u64,
    pub created_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub access_count: u64,
    pub expires_at: Option<std::time::SystemTime>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheDataType {
    Audio,
    Image,
    Video,
    Raw,
    Metadata,
    Thumbnail,
    Spectrogram,
    Waveform,
    Analysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePolicy {
    pub max_size_bytes: u64,
    pub max_entries: usize,
    pub ttl_seconds: Option<u64>,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LeastRecentlyUsed,
    LeastFrequentlyUsed,
    FirstInFirstOut,
    SizeBased,
    TimeBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub hit_ratio: f64,
    pub memory_usage_mb: f64,
}

pub struct CacheManager {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    policy: Arc<RwLock<CachePolicy>>,
    stats: Arc<RwLock<CacheStats>>,
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf, policy: CachePolicy) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(RwLock::new(policy)),
            stats: Arc::new(RwLock::new(CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                hit_count: 0,
                miss_count: 0,
                eviction_count: 0,
                hit_ratio: 0.0,
                memory_usage_mb: 0.0,
            })),
            cache_dir,
        })
    }

    pub fn put(
        &self,
        key: String,
        data: Vec<u8>,
        data_type: CacheDataType,
        metadata: HashMap<String, String>,
    ) -> Result<Uuid> {
        let entry_id = Uuid::new_v4();
        let now = std::time::SystemTime::now();
        
        let policy = self.policy.read();
        let ttl_seconds = policy.ttl_seconds;
        drop(policy);

        let expires_at = ttl_seconds.map(|ttl| now + std::time::Duration::from_secs(ttl));

        let entry = CacheEntry {
            id: entry_id,
            key: key.clone(),
            data_type,
            size_bytes: data.len() as u64,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            expires_at,
            metadata,
        };

        self.write_cache_file(&entry, &data)?;

        {
            let mut entries = self.entries.write();
            entries.insert(key, entry);
        }

        self.update_stats();
        self.enforce_policy()?;

        Ok(entry_id)
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let now = std::time::SystemTime::now();
        
        let entry = {
            let entries = self.entries.read();
            entries.get(key).cloned()
        };

        if let Some(entry) = entry {
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    self.remove(key)?;
                    {
                        let mut stats = self.stats.write();
                        stats.miss_count += 1;
                    }
                    self.update_hit_ratio();
                    return Ok(None);
                }
            }

            let data = self.read_cache_file(&entry)?;
            
            {
                let mut entries = self.entries.write();
                if let Some(entry_ref) = entries.get_mut(key) {
                    entry_ref.last_accessed = now;
                    entry_ref.access_count += 1;
                }
            }

            {
                let mut stats = self.stats.write();
                stats.hit_count += 1;
            }
            
            self.update_hit_ratio();
            Ok(Some(data))
        } else {
            {
                let mut stats = self.stats.write();
                stats.miss_count += 1;
            }
            self.update_hit_ratio();
            Ok(None)
        }
    }

    pub fn remove(&self, key: &str) -> Result<bool> {
        let entry = {
            let mut entries = self.entries.write();
            entries.remove(key)
        };

        if let Some(entry) = entry {
            self.delete_cache_file(&entry)?;
            self.update_stats();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear(&self) -> Result<()> {
        let entries = self.entries.read();
        let keys: Vec<String> = entries.keys().cloned().collect();
        drop(entries);

        for key in keys {
            self.remove(&key)?;
        }

        Ok(())
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let now = std::time::SystemTime::now();
        let expired_keys: Vec<String> = {
            let entries = self.entries.read();
            entries
                .values()
                .filter(|e| {
                    if let Some(expires_at) = e.expires_at {
                        now > expires_at
                    } else {
                        false
                    }
                })
                .map(|e| e.key.clone())
                .collect()
        };

        let mut removed_count = 0;
        for key in expired_keys {
            if self.remove(&key)? {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    pub fn get_entry_info(&self, key: &str) -> Option<CacheEntry> {
        let entries = self.entries.read();
        entries.get(key).cloned()
    }

    pub fn list_entries(&self) -> Vec<CacheEntry> {
        let entries = self.entries.read();
        entries.values().cloned().collect()
    }

    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read();
        stats.clone()
    }

    pub fn update_policy(&self, new_policy: CachePolicy) -> Result<()> {
        {
            let mut policy = self.policy.write();
            *policy = new_policy;
        }
        
        self.enforce_policy()?;
        Ok(())
    }

    fn enforce_policy(&self) -> Result<()> {
        let policy = self.policy.read();
        let entries = self.entries.read();
        
        let current_size: u64 = entries.values().map(|e| e.size_bytes).sum();
        let current_count = entries.len();

        if current_size <= policy.max_size_bytes && current_count <= policy.max_entries {
            return Ok(());
        }

        let mut entries_to_evict: Vec<CacheEntry> = entries.values().cloned().collect();
        
        match policy.eviction_policy {
            EvictionPolicy::LeastRecentlyUsed => {
                entries_to_evict.sort_by_key(|e| e.last_accessed);
            },
            EvictionPolicy::LeastFrequentlyUsed => {
                entries_to_evict.sort_by_key(|e| e.access_count);
            },
            EvictionPolicy::FirstInFirstOut => {
                entries_to_evict.sort_by_key(|e| e.created_at);
            },
            EvictionPolicy::SizeBased => {
                entries_to_evict.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            },
            EvictionPolicy::TimeBased => {
                entries_to_evict.sort_by_key(|e| e.expires_at.unwrap_or(std::time::SystemTime::UNIX_EPOCH));
            },
        }

        drop(entries);
        
        let max_size_bytes = policy.max_size_bytes;
        let max_entries = policy.max_entries;
        drop(policy);

        let mut _evicted_count = 0;
        let mut current_size = current_size;
        let mut current_count = current_count;

        for entry in entries_to_evict {
            if current_size <= max_size_bytes && current_count <= max_entries {
                break;
            }

            if self.remove(&entry.key)? {
                current_size -= entry.size_bytes;
                current_count -= 1;
                _evicted_count += 1;

                {
                    let mut stats = self.stats.write();
                    stats.eviction_count += 1;
                }
            }
        }

        Ok(())
    }

    fn write_cache_file(&self, entry: &CacheEntry, data: &[u8]) -> Result<()> {
        let file_path = self.cache_dir.join(format!("{}.cache", entry.id));
        std::fs::write(file_path, data)?;
        Ok(())
    }

    fn read_cache_file(&self, entry: &CacheEntry) -> Result<Vec<u8>> {
        let file_path = self.cache_dir.join(format!("{}.cache", entry.id));
        Ok(std::fs::read(file_path)?)
    }

    fn delete_cache_file(&self, entry: &CacheEntry) -> Result<()> {
        let file_path = self.cache_dir.join(format!("{}.cache", entry.id));
        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn update_stats(&self) {
        let entries = self.entries.read();
        let total_entries = entries.len();
        let total_size_bytes: u64 = entries.values().map(|e| e.size_bytes).sum();
        let memory_usage_mb = total_size_bytes as f64 / 1024.0 / 1024.0;

        let mut stats = self.stats.write();
        stats.total_entries = total_entries;
        stats.total_size_bytes = total_size_bytes;
        stats.memory_usage_mb = memory_usage_mb;
    }

    fn update_hit_ratio(&self) {
        let mut stats = self.stats.write();
        if stats.hit_count + stats.miss_count > 0 {
            stats.hit_ratio = stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64;
        }
    }

    pub fn optimize_cache(&self) -> Result<()> {
        self.cleanup_expired()?;
        
        let entries = self.entries.read();
        let orphaned_files: Vec<_> = std::fs::read_dir(&self.cache_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".cache") {
                        let id_str = filename.trim_end_matches(".cache");
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            !entries.values().any(|e| e.id == id)
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .collect();

        drop(entries);

        for entry in orphaned_files {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                tracing::warn!("Failed to remove orphaned cache file {}: {}", entry.path().display(), e);
            }
        }

        Ok(())
    }

    pub fn export_cache_info(&self, output_path: &Path) -> Result<()> {
        let entries = self.entries.read();
        let stats = self.stats.read();
        
        let cache_info = serde_json::json!({
            "stats": *stats,
            "entries": entries.values().cloned().collect::<Vec<_>>(),
            "exported_at": std::time::SystemTime::now()
        });

        std::fs::write(output_path, serde_json::to_string_pretty(&cache_info)?)?;
        Ok(())
    }
}
