use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub logging: LoggingConfig,
    pub paths: PathsConfig,
    pub gpu: GpuConfig,
    pub effects: EffectsConfig,
    pub projects: ProjectsConfig,
    pub cli: CliConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub version: String,
    pub debug: bool,
    pub auto_check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub console: bool,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub plugins_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub device: String,
    pub memory_limit: u64,
    pub enable_compute: bool,
    pub shader_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    pub cache_size: u64,
    pub cache_dir: Option<PathBuf>,
    pub max_concurrent: usize,
    pub default_quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsConfig {
    pub default_directory: PathBuf,
    pub auto_save: bool,
    pub auto_save_interval: u64,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub default_format: String,
    pub color_output: bool,
    pub progress_bar: bool,
    pub confirm_destructive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            logging: LoggingConfig::default(),
            paths: PathsConfig::default(),
            gpu: GpuConfig::default(),
            effects: EffectsConfig::default(),
            projects: ProjectsConfig::default(),
            cli: CliConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            debug: false,
            auto_check_updates: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: Some("tiffiny.log".to_string()),
            console: true,
            format: "text".to_string(),
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            data_dir: home.join(".local/share/tiffiny"),
            config_dir: home.join(".config/tiffiny"),
            cache_dir: home.join(".cache/tiffiny"),
            temp_dir: std::env::temp_dir(),
            plugins_dir: home.join(".local/share/tiffiny/plugins"),
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            device: "auto".to_string(),
            memory_limit: 4 * 1024 * 1024 * 1024,4GB
            enable_compute: true,
            shader_cache: true,
        }
    }
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            cache_size: 1024 * 1024 * 1024,
            cache_dir: None,
            max_concurrent: 4,
            default_quality: 80,
        }
    }
}

impl Default for ProjectsConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            default_directory: home.join("TiffinyStudio/Projects"),
            auto_save: true,
            auto_save_interval: 300,
            backup_enabled: true,
            backup_count: 5,
            compression_enabled: false,
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_format: "tiffiny".to_string(),
            color_output: true,
            progress_bar: true,
            confirm_destructive: true,
        }
    }
}

pub async fn load_config(config_path: &std::path::Path) -> Result<Config> {
    if config_path.exists() {
        let content = tokio::fs::read_to_string(config_path).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub async fn save_config(config: &Config, config_path: &std::path::Path) -> Result<()> {
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let content = toml::to_string_pretty(config)?;
    
    tokio::fs::write(config_path, content).await?;
    
    Ok(())
}

pub fn get_default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tiffiny")
        .join("config.toml")
}

pub fn merge_configs(base: &Config, override_config: &Config) -> Config {
    Config {
        general: merge_general(&base.general, &override_config.general),
        logging: merge_logging(&base.logging, &override_config.logging),
        paths: merge_paths(&base.paths, &override_config.paths),
        gpu: merge_gpu(&base.gpu, &override_config.gpu),
        effects: merge_effects(&base.effects, &override_config.effects),
        projects: merge_projects(&base.projects, &override_config.projects),
        cli: merge_cli(&base.cli, &override_config.cli),
    }
}

fn merge_general(base: &GeneralConfig, override_config: &GeneralConfig) -> GeneralConfig {
    GeneralConfig {
        version: override_config.version.clone(),
        debug: override_config.debug,
        auto_check_updates: override_config.auto_check_updates,
    }
}

fn merge_logging(base: &LoggingConfig, override_config: &LoggingConfig) -> LoggingConfig {
    LoggingConfig {
        level: if override_config.level != "info" { override_config.level.clone() } else { base.level.clone() },
        file: override_config.file.clone().or_else(|| base.file.clone()),
        console: override_config.console,
        format: if override_config.format != "text" { override_config.format.clone() } else { base.format.clone() },
    }
}

fn merge_paths(base: &PathsConfig, override_config: &PathsConfig) -> PathsConfig {
    PathsConfig {
        data_dir: if !override_config.data_dir.as_os_str().is_empty() { override_config.data_dir.clone() } else { base.data_dir.clone() },
        config_dir: if !override_config.config_dir.as_os_str().is_empty() { override_config.config_dir.clone() } else { base.config_dir.clone() },
        cache_dir: if !override_config.cache_dir.as_os_str().is_empty() { override_config.cache_dir.clone() } else { base.cache_dir.clone() },
        temp_dir: if !override_config.temp_dir.as_os_str().is_empty() { override_config.temp_dir.clone() } else { base.temp_dir.clone() },
        plugins_dir: if !override_config.plugins_dir.as_os_str().is_empty() { override_config.plugins_dir.clone() } else { base.plugins_dir.clone() },
    }
}

fn merge_gpu(base: &GpuConfig, override_config: &GpuConfig) -> GpuConfig {
    GpuConfig {
        device: if override_config.device != "auto" { override_config.device.clone() } else { base.device.clone() },
        memory_limit: if override_config.memory_limit != 4 * 1024 * 1024 * 1024 { override_config.memory_limit } else { base.memory_limit },
        enable_compute: override_config.enable_compute,
        shader_cache: override_config.shader_cache,
    }
}

fn merge_effects(base: &EffectsConfig, override_config: &EffectsConfig) -> EffectsConfig {
    EffectsConfig {
        cache_size: if override_config.cache_size != 1024 * 1024 * 1024 { override_config.cache_size } else { base.cache_size },
        cache_dir: override_config.cache_dir.clone().or_else(|| base.cache_dir.clone()),
        max_concurrent: if override_config.max_concurrent != 4 { override_config.max_concurrent } else { base.max_concurrent },
        default_quality: if override_config.default_quality != 80 { override_config.default_quality } else { base.default_quality },
    }
}

fn merge_projects(base: &ProjectsConfig, override_config: &ProjectsConfig) -> ProjectsConfig {
    ProjectsConfig {
        default_directory: if !override_config.default_directory.as_os_str().is_empty() { override_config.default_directory.clone() } else { base.default_directory.clone() },
        auto_save: override_config.auto_save,
        auto_save_interval: if override_config.auto_save_interval != 300 { override_config.auto_save_interval } else { base.auto_save_interval },
        backup_enabled: override_config.backup_enabled,
        backup_count: if override_config.backup_count != 5 { override_config.backup_count } else { base.backup_count },
        compression_enabled: override_config.compression_enabled,
    }
}

fn merge_cli(base: &CliConfig, override_config: &CliConfig) -> CliConfig {
    CliConfig {
        default_format: if override_config.default_format != "tiffiny" { override_config.default_format.clone() } else { base.default_format.clone() },
        color_output: override_config.color_output,
        progress_bar: override_config.progress_bar,
        confirm_destructive: override_config.confirm_destructive,
    }
}
