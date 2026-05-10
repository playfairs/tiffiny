use tiffiny_core::{prelude::*, CacheManager, MemoryManager, RecoveryManager};
use tiffiny_utils::{fs::FileSystemHelper, platform::Platform};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct Bootstrap {
    pub cache_manager: Arc<CacheManager>,
    pub memory_manager: Arc<MemoryManager>,
    pub recovery_manager: Arc<RecoveryManager>,
    pub platform: Arc<Platform>,
    pub config: BootstrapConfig,
}

pub struct BootstrapConfig {
    pub cache_directory: PathBuf,
    pub cache_size_mb: u64,
    pub memory_pool_size_mb: u64,
    pub recovery_directory: PathBuf,
    pub auto_save_enabled: bool,
    pub auto_save_interval_seconds: u64,
    pub max_recovery_points: usize,
}

impl Bootstrap {
    pub async fn new() -> Result<Self> {
        let config = Self::load_config().await?;
        
        let platform = Arc::new(Platform::new().await?);
        
        let cache_manager = Arc::new(CacheManager::new(
            config.cache_directory.clone(),
            tiffiny_core::cache::CachePolicy {
                max_size_bytes: config.cache_size_mb * 1024 * 1024,
                max_entries: 1000,
                ttl_seconds: Some(3600),
                eviction_policy: tiffiny_core::cache::EvictionPolicy::LeastRecentlyUsed,
            },
        )?);
        
        let memory_manager = Arc::new(MemoryManager::new());
        
        let recovery_config = tiffiny_core::recovery::RecoveryConfig {
            auto_save_interval_seconds: config.auto_save_interval_seconds,
            max_recovery_points: config.max_recovery_points,
            recovery_directory: config.recovery_directory.clone(),
            compression_enabled: true,
            encryption_enabled: false,
            cleanup_old_after_days: 30,
        };
        
        let recovery_manager = Arc::new(RecoveryManager::new(recovery_config)?);
        
        if config.auto_save_enabled {
            recovery_manager.start_auto_save()?;
        }

        Ok(Self {
            cache_manager,
            memory_manager,
            recovery_manager,
            platform,
            config,
        })
    }

    async fn load_config() -> Result<BootstrapConfig> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiffiny");
        
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("tiffiny");
        
        let recovery_dir = cache_dir.join("recovery");
        
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&recovery_dir)?;
        
        let config_file = config_dir.join("bootstrap.json");
        
        if config_file.exists() {
            let config_content = std::fs::read_to_string(&config_file)?;
            let config: BootstrapConfig = serde_json::from_str(&config_content)?;
            Ok(config)
        } else {
            let default_config = BootstrapConfig::default(&cache_dir, &recovery_dir);
            let config_json = serde_json::to_string_pretty(&default_config)?;
            std::fs::write(&config_file, config_json)?;
            Ok(default_config)
        }
    }

    pub async fn initialize_systems(&self) -> Result<()> {
        self.cache_manager.optimize_cache()?;
        self.memory_manager.defragment_pools()?;
        self.recovery_manager.cleanup_old_recovery_points()?;
        
        tracing::info("Bootstrap systems initialized successfully");
        Ok(())
    }

    pub async fn perform_startup_checks(&self) -> Result<()> {
        let system_info = self.platform.get_system_info().await?;
        
        tracing::info("System information: {:?}", system_info);
        
        if system_info.available_memory_mb < 1024 {
            tracing::warn!("Low memory detected: {} MB available", system_info.available_memory_mb);
        }
        
        if system_info.available_disk_space_gb < 10 {
            tracing::warn!("Low disk space detected: {} GB available", system_info.available_disk_space_gb);
        }
        
        let recovery_points = self.recovery_manager.list_recovery_points();
        if !recovery_points.is_empty() {
            tracing::info("Found {} recovery points from previous session", recovery_points.len());
            
            for recovery_point in &recovery_points {
                if matches!(recovery_point.recovery_type, tiffiny_core::recovery::RecoveryType::CrashRecovery) {
                    tracing::warn!("Crash recovery point found from: {:?}", recovery_point.created_at);
                }
            }
        }
        
        Ok(())
    }

    pub async fn create_memory_pools(&self) -> Result<()> {
        let system_info = self.platform.get_system_info().await?;
        let pool_size = (self.config.memory_pool_size_mb * 1024 * 1024).min(
            (system_info.available_memory_mb / 2) * 1024 * 1024
        );
        
        let main_pool_id = self.memory_manager.create_pool(
            "Main Memory Pool".to_string(),
            pool_size,
            1024 * 1024,
            tiffiny_core::memory::AllocationStrategy::BuddySystem,
        )?;
        
        let temp_pool_id = self.memory_manager.create_pool(
            "Temporary Memory Pool".to_string(),
            pool_size / 4,
            256 * 1024,
            tiffiny_core::memory::AllocationStrategy::FirstFit,
        )?;
        
        tracing::info!("Created memory pools: main={}, temp={}", main_pool_id, temp_pool_id);
        
        Ok(())
    }

    pub async fn setup_directories(&self) -> Result<()> {
        let directories = vec![
            &self.config.cache_directory,
            &self.config.recovery_directory,
            &self.config.cache_directory.join("thumbnails"),
            &self.config.cache_directory.join("waveforms"),
            &self.config.cache_directory.join("spectrograms"),
            &self.config.cache_directory.join("temp"),
        ];
        
        for directory in directories {
            std::fs::create_dir_all(directory)?;
            tracing::debug!("Created directory: {}", directory.display());
        }
        
        Ok(())
    }

    pub async fn cleanup_on_startup(&self) -> Result<()> {
        let temp_dir = self.config.cache_directory.join("temp");
        if temp_dir.exists() {
            for entry in std::fs::read_dir(&temp_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let metadata = std::fs::metadata(&path)?;
                    let modified = metadata.modified()?;
                    let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                    
                    if age > std::time::Duration::from_secs(24 * 60 * 60) {
                        if let Err(e) = std::fs::remove_file(&path) {
                            tracing::warn!("Failed to remove old temp file {}: {}", path.display(), e);
                        } else {
                            tracing::debug!("Removed old temp file: {}", path.display());
                        }
                    }
                }
            }
        }
        
        self.cache_manager.cleanup_expired()?;
        
        Ok(())
    }

    pub fn get_cache_manager(&self) -> Arc<CacheManager> {
        self.cache_manager.clone()
    }

    pub fn get_memory_manager(&self) -> Arc<MemoryManager> {
        self.memory_manager.clone()
    }

    pub fn get_recovery_manager(&self) -> Arc<RecoveryManager> {
        self.recovery_manager.clone()
    }

    pub fn get_platform(&self) -> Arc<Platform> {
        self.platform.clone()
    }
}

impl BootstrapConfig {
    fn default(cache_dir: &PathBuf, recovery_dir: &PathBuf) -> Self {
        Self {
            cache_directory: cache_dir.clone(),
            cache_size_mb: 1024,
            memory_pool_size_mb: 512,
            recovery_directory: recovery_dir.clone(),
            auto_save_enabled: true,
            auto_save_interval_seconds: 300,
            max_recovery_points: 50,
        }
    }
}
