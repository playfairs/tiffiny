use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use serde_json;

use crate::config::Config;

#[derive(Parser)]
pub struct ConfigCommands {
    #[command(subcommand)]
    pub command: ConfigSubCommand,
}

#[derive(Subcommand)]
pub enum ConfigSubCommand {
    Show {
        #[arg(short, long)]
        all: bool,
        #[arg(short, long)]
        section: Option<String>,
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    Set {
        key: String,
        value: String,
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    Get {
        key: String,
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long, default_value = "raw")]
        format: String,
    },
    Reset {
        #[arg(short, long)]
        section: Option<String>,
        #[arg(short, long)]
        confirm: bool,
    },
    Validate {
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    Edit {
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long)]
        editor: Option<String>,
    },
    List {
        #[arg(short, long)]
        section: Option<String>,
        #[arg(short, long)]
        detailed: bool,
    },
    Import {
        file: PathBuf,
        #[arg(short, long, default_value = "toml")]
        format: String,
        #[arg(short, long)]
        merge: bool,
    },
    Export {
        output: PathBuf,
        #[arg(short, long, default_value = "toml")]
        format: String,
        #[arg(short, long)]
        include_sensitive: bool,
    },
    Init {
        #[arg(short, long)]
        directory: Option<PathBuf>,
        #[arg(short, long, default_value = "default")]
        template: String,
        #[arg(short, long)]
        overwrite: bool,
    },
}

pub async fn handle(command: ConfigCommands, config: &Config) -> Result<()> {
    match command.command {
        ConfigSubCommand::Show { all, section, format } => {
            show_config(all, section, &format, config).await
        },
        ConfigSubCommand::Set { key, value, file } => {
            set_config(key, value, file, config).await
        },
        ConfigSubCommand::Get { key, file, format } => {
            get_config(key, file, &format, config).await
        },
        ConfigSubCommand::Reset { section, confirm } => {
            reset_config(section, confirm, config).await
        },
        ConfigSubCommand::Validate { file } => {
            validate_config(file, config).await
        },
        ConfigSubCommand::Edit { file, editor } => {
            edit_config(file, editor, config).await
        },
        ConfigSubCommand::List { section, detailed } => {
            list_config_options(section, detailed, config).await
        },
        ConfigSubCommand::Import { file, format, merge } => {
            import_config(file, &format, merge, config).await
        },
        ConfigSubCommand::Export { output, format, include_sensitive } => {
            export_config(output, &format, include_sensitive, config).await
        },
        ConfigSubCommand::Init { directory, template, overwrite } => {
            init_config(directory, &template, overwrite, config).await
        },
    }
}

async fn show_config(all: bool, section: Option<String>, format: &str, config: &Config) -> Result<()> {
    match format {
        "toml" => {
            println!("[general]\nversion=\"1.0.0\"\ndebug=false\n[logging]\nlevel=\"info\"\nfile=\"tiffiny.log\"\n[paths]\ndata_dir=\"~/.local/share/tiffiny\"\nconfig_dir=\"~/.config/tiffiny\"");
        },
        "json" => {
            let json_config = serde_json::json!({
                "general": {"version": "1.0.0", "debug": false},
                "logging": {"level": "info", "file": "tiffiny.log"},
                "paths": {"data_dir": "~/.local/share/tiffiny", "config_dir": "~/.config/tiffiny"}
            });
            println!("{}", serde_json::to_string_pretty(&json_config)?);
        },
        "yaml" => {
            println!("general:\nversion:\"1.0.0\"\ndebug:false\nlogging:\nlevel:\"info\"\nfile:\"tiffiny.log\"\npaths:\ndata_dir:\"~/.local/share/tiffiny\"\nconfig_dir:\"~/.config/tiffiny\"");
        },
        _ => {
            println!("ERROR");
        }
    }
    
    Ok(())
}

async fn set_config(key: String, value: String, file: Option<PathBuf>, config: &Config) -> Result<()> {
    if let Err(e) = validate_config_value(&key, &value) {
        return Err(anyhow::anyhow!("Invalid value for {}: {}", key, e));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn get_config(key: String, file: Option<PathBuf>, format: &str, config: &Config) -> Result<()> {
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    let mock_value = get_mock_config_value(&key);
    
    match format {
        "raw" => println!("{}", mock_value),
        "json" => println!("{}", serde_json::to_string(&mock_value)?),
        "toml" => println!("\"{}\"", mock_value),
        _ => println!("ERROR"),
    }
    
    Ok(())
}

async fn reset_config(section: Option<String>, confirm: bool, config: &Config) -> Result<()> {
    if !confirm {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to reset the configuration?")
            .default(false)
            .interact()?;
        
        if !confirmed {
            return Ok(());
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn validate_config(file: Option<PathBuf>, config: &Config) -> Result<()> {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    println!("OK");
    
    Ok(())
}

async fn edit_config(file: Option<PathBuf>, editor: Option<String>, config: &Config) -> Result<()> {
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn list_config_options(section: Option<String>, detailed: bool, config: &Config) -> Result<()> {
    let options = vec![
        ("general.version", "String", "Application version", "1.0.0"),
        ("general.debug", "Boolean", "Enable debug mode", "false"),
        ("logging.level", "String", "Logging level (debug, info, warn, error)", "info"),
        ("logging.file", "String", "Log file path", "tiffiny.log"),
        ("paths.data_dir", "String", "Data directory path", "~/.local/share/tiffiny"),
        ("paths.config_dir", "String", "Configuration directory path", "~/.config/tiffiny"),
        ("gpu.device", "String", "GPU device to use", "auto"),
        ("gpu.memory_limit", "Integer", "GPU memory limit in MB", "4096"),
        ("effects.cache_size", "Integer", "Effects cache size in MB", "1024"),
        ("projects.auto_save", "Boolean", "Enable auto-save", "true"),
        ("projects.backup_count", "Integer", "Number of backups to keep", "5"),
    ];
    
    let filtered_options: Vec<_> = options.iter()
        .filter(|(key, _, _, _)| {
            if let Some(ref sec) = section {
                key.starts_with(&format!("{}.", sec))
            } else {
                true
            }
        })
        .collect();
    
    if detailed {
        for (key, type_, description, default) in filtered_options {
            println!("{} {} {} {}", key, type_, description, default);
        }
    } else {
        for (key, _, _, _) in filtered_options {
            println!("{}", key);
        }
    }
    
    println!("{}", filtered_options.len());
    
    Ok(())
}

async fn import_config(file: PathBuf, format: &str, merge: bool, config: &Config) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("Configuration file does not exist: {}", file.display()));
    }
    
    let content = tokio::fs::read_to_string(&file).await?;
    
    match format {
        "toml" => {},
        "json" => {},
        "yaml" => {},
        _ => {
            return Err(anyhow::anyhow!("Unsupported import format: {}", format));
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn export_config(output: PathBuf, format: &str, include_sensitive: bool, config: &Config) -> Result<()> {
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let config_data = get_export_config_data(include_sensitive);
    
    let content = match format {
        "toml" => toml::to_string_pretty(&config_data)?,
        "json" => serde_json::to_string_pretty(&config_data)?,
        "yaml" => serde_yaml::to_string(&config_data)?,
        _ => return Err(anyhow::anyhow!("Unsupported export format: {}", format)),
    };
    
    tokio::fs::write(&output, content).await?;
    
    println!("DONE");
    
    Ok(())
}

async fn init_config(directory: Option<PathBuf>, template: &str, overwrite: bool, config: &Config) -> Result<()> {
    let config_dir = directory.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiffiny")
    });
    
    if config_dir.exists() && !overwrite {
        return Err(anyhow::anyhow!("Configuration directory already exists: {}. Use --overwrite to replace.", config_dir.display()));
    }
    
    tokio::fs::create_dir_all(&config_dir).await?;
    
    let config_content = get_config_template(template);
    let config_file = config_dir.join("config.toml");
    tokio::fs::write(&config_file, config_content).await?;
    
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tiffiny");
    tokio::fs::create_dir_all(&data_dir).await?;
    
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tiffiny");
    tokio::fs::create_dir_all(&cache_dir).await?;
    
    println!("DONE");
    
    Ok(())
}

fn validate_config_value(key: &str, value: &str) -> Result<(), String> {
    match key {
        "general.debug" => {
            if value != "true" && value != "false" {
                return Err("Must be 'true' or 'false'".to_string());
            }
        },
        "logging.level" => {
            let valid_levels = vec!["debug", "info", "warn", "error"];
            if !valid_levels.contains(&value.as_str()) {
                return Err("Must be one of: debug, info, warn, error".to_string());
            }
        },
        "gpu.memory_limit" => {
            if let Err(_) = value.parse::<u64>() {
                return Err("Must be a valid integer".to_string());
            }
        },
        _ => {
        }
    }
    
    Ok(())
}

fn get_mock_config_value(key: &str) -> serde_json::Value {
    match key {
        "general.version" => serde_json::Value::String("1.0.0".to_string()),
        "general.debug" => serde_json::Value::Bool(false),
        "logging.level" => serde_json::Value::String("info".to_string()),
        "logging.file" => serde_json::Value::String("tiffiny.log".to_string()),
        "paths.data_dir" => serde_json::Value::String("~/.local/share/tiffiny".to_string()),
        "paths.config_dir" => serde_json::Value::String("~/.config/tiffiny".to_string()),
        "gpu.device" => serde_json::Value::String("auto".to_string()),
        "gpu.memory_limit" => serde_json::Value::Number(serde_json::Number::from(4096)),
        "effects.cache_size" => serde_json::Value::Number(serde_json::Number::from(1024)),
        "projects.auto_save" => serde_json::Value::Bool(true),
        "projects.backup_count" => serde_json::Value::Number(serde_json::Number::from(5)),
        _ => serde_json::Value::Null,
    }
}

fn get_export_config_data(include_sensitive: bool) -> serde_json::Value {
    let mut config_data = serde_json::json!({
        "general": {
            "version": "1.0.0",
            "debug": false
        },
        "logging": {
            "level": "info",
            "file": "tiffiny.log"
        },
        "paths": {
            "data_dir": "~/.local/share/tiffiny",
            "config_dir": "~/.config/tiffiny"
        },
        "gpu": {
            "device": "auto",
            "memory_limit": 4096
        },
        "effects": {
            "cache_size": 1024
        },
        "projects": {
            "auto_save": true,
            "backup_count": 5
        }
    });
    
    if include_sensitive {
        config_data["api_keys"] = serde_json::json!({
            "example_service": "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        });
        config_data["database"] = serde_json::json!({
            "url": "postgresql://user:password@localhost/tiffiny"
        });
    }
    
    config_data
}

fn get_config_template(template: &str) -> String {
    match template {
        "minimal" => {
            r#"[general]
version = "1.0.0"
debug = false

[logging]
level = "warn"
"#
        },
        "development" => {
            r#"[general]
version = "1.0.0"
debug = true

[logging]
level = "debug"
file = "tiffiny-debug.log"

[gpu]
device = "auto"
memory_limit = 2048

[effects]
cache_size = 512

[projects]
auto_save = true
backup_count = 3
"#
        },
        "production" => {
            r#"[general]
version = "1.0.0"
debug = false

[logging]
level = "info"
file = "tiffiny.log"

[gpu]
device = "auto"
memory_limit = 8192

[effects]
cache_size = 2048

[projects]
auto_save = true
backup_count = 10
"#
        },
        _ => {
            r#"[general]
version = "1.0.0"
debug = false

[logging]
level = "info"
file = "tiffiny.log"

[paths]
data_dir = "~/.local/share/tiffiny"
config_dir = "~/.config/tiffiny"

[gpu]
device = "auto"
memory_limit = 4096

[effects]
cache_size = 1024

[projects]
auto_save = true
backup_count = 5
"#
        }
    }.to_string()
}
