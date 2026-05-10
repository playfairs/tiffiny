use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Parser)]
pub struct ConvertCommands {
    #[command(subcommand)]
    pub command: ConvertSubCommand,
}

#[derive(Subcommand)]
pub enum ConvertSubCommand {
    File {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        format: String,
        #[arg(short, long)]
        quality: Option<u8>,
        #[arg(short, long)]
        options: Option<String>,
    },
    Batch {
        input: String,
        output: PathBuf,
        #[arg(short, long)]
        format: String,
        #[arg(short, long)]
        quality: Option<u8>,
        #[arg(short, long)]
        parallel: bool,
        #[arg(short, long, default_value = "4")]
        workers: usize,
    },
    Pipeline {
        config: PathBuf,
        input: String,
        output: PathBuf,
        #[arg(short, long)]
        dry_run: bool,
    },
    Formats {
        #[arg(short, long)]
        media_type: Option<String>,
        #[arg(short, long)]
        detailed: bool,
    },
    Options {
        input_format: String,
        output_format: String,
    },
    Analyze {
        input: PathBuf,
        #[arg(short, long)]
        detailed: bool,
    },
    Preset {
        #[command(subcommand)]
        command: PresetCommand,
    },
}

#[derive(Subcommand)]
pub enum PresetCommand {
    Create {
        name: String,
        input_format: String,
        output_format: String,
        options: String,
    },
    List {
        #[arg(short, long)]
        input_format: Option<String>,
        #[arg(short, long)]
        output_format: Option<String>,
    },
    Show {
        name: String,
    },
    Apply {
        name: String,
        input: PathBuf,
        output: PathBuf,
    },
    Delete {
        name: String,
    },
}

pub async fn handle(command: ConvertCommands, config: &Config) -> Result<()> {
    match command.command {
        ConvertSubCommand::File { input, output, format, quality, options } => {
            convert_file(input, output, format, quality, options, config).await
        },
        ConvertSubCommand::Batch { input, output, format, quality, parallel, workers } => {
            convert_batch(input, output, format, quality, parallel, workers, config).await
        },
        ConvertSubCommand::Pipeline { config, input, output, dry_run } => {
            convert_pipeline(config, input, output, dry_run, config).await
        },
        ConvertSubCommand::Formats { media_type, detailed } => {
            list_formats(media_type, detailed, config).await
        },
        ConvertSubCommand::Options { input_format, output_format } => {
            show_options(input_format, output_format, config).await
        },
        ConvertSubCommand::Analyze { input, detailed } => {
            analyze_file(input, detailed, config).await
        },
        ConvertSubCommand::Preset { command } => {
            handle_preset(command, config).await
        },
    }
}

async fn convert_file(input: PathBuf, output: PathBuf, format: String, quality: Option<u8>, options: Option<String>, config: &Config) -> Result<()> {
    println!("START");
    
    if !input.exists() {
        return Err(anyhow::anyhow!("Input file does not exist: {}", input.display()));
    }
    
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn convert_batch(input: String, output: PathBuf, format: String, quality: Option<u8>, parallel: bool, workers: usize, config: &Config) -> Result<()> {
    println!("START");
    
    tokio::fs::create_dir_all(&output).await?;
    
    let files = find_input_files(&input)?;
    
    if files.is_empty() {
        return Ok(());
    }
    
    if parallel {
        convert_files_parallel(files, &output, &format, quality, workers).await?;
    } else {
        convert_files_sequential(files, &output, &format, quality).await?;
    }
    
    println!("DONE");
    
    Ok(())
}

async fn convert_pipeline(config: PathBuf, input: String, output: PathBuf, dry_run: bool, config_data: &Config) -> Result<()> {
    println!("START");
    
    if dry_run {
        return Ok(());
    }
    
    println!("DONE");
    
    Ok(())
}

async fn list_formats(media_type: Option<String>, detailed: bool, config: &Config) -> Result<()> {
    let formats = vec![
        ("png", "Portable Network Graphics", "Image"),
        ("jpg", "JPEG Image", "Image"),
        ("gif", "Graphics Interchange Format", "Image"),
        ("mp4", "MPEG-4 Video", "Video"),
        ("avi", "Audio Video Interleave", "Video"),
        ("mov", "QuickTime Movie", "Video"),
        ("mp3", "MPEG Audio Layer 3", "Audio"),
        ("wav", "Waveform Audio File Format", "Audio"),
        ("flac", "Free Lossless Audio Codec", "Audio"),
    ];
    
    let filtered_formats: Vec<_> = formats.iter()
        .filter(|(_, _, mt)| {
            if let Some(ref filter_mt) = media_type {
                mt.to_lowercase() == filter_mt.to_lowercase()
            } else {
                true
            }
        })
        .collect();
    
    if detailed {
        for (ext, name, media_type) in filtered_formats {
            println!("{} {} {}", ext, name, media_type);
        }
    } else {
        for (ext, _, _) in filtered_formats {
            println!("{}", ext);
        }
    }
    
    println!("{}", filtered_formats.len());
    
    Ok(())
}

async fn show_options(input_format: String, output_format: String, config: &Config) -> Result<()> {
    match (input_format.as_str(), output_format.as_str()) {
        ("png", "jpg") => {
            println!("quality progressive optimize");
        },
        ("mp4", "mp3") => {
            println!("bitrate sample_rate channels");
        },
        _ => {
            println!("quality optimize metadata");
        }
    }
    
    Ok(())
}

async fn analyze_file(input: PathBuf, detailed: bool, config: &Config) -> Result<()> {
    if !input.exists() {
        return Err(anyhow::anyhow!("Input file does not exist: {}", input.display()));
    }
    
    let metadata = std::fs::metadata(&input)?;
    let file_size = metadata.len();
    
    println!("{} {}", input.display(), format_bytes(file_size));
    
    if let Some(extension) = input.extension() {
        println!("{}", extension.to_string_lossy());
    }
    
    if detailed {
        println!("{} true 5", detect_media_type(&input));
    }
    
    Ok(())
}

async fn handle_preset(command: PresetCommand, config: &Config) -> Result<()> {
    match command {
        PresetCommand::Create { name, input_format, output_format, options } => {
            create_preset(name, input_format, output_format, options, config).await
        },
        PresetCommand::List { input_format, output_format } => {
            list_presets(input_format, output_format, config).await
        },
        PresetCommand::Show { name } => {
            show_preset(name, config).await
        },
        PresetCommand::Apply { name, input, output } => {
            apply_preset(name, input, output, config).await
        },
        PresetCommand::Delete { name } => {
            delete_preset(name, config).await
        },
    }
}

async fn create_preset(name: String, input_format: String, output_format: String, options: String, config: &Config) -> Result<()> {
    println!("DONE");
    
    Ok(())
}

async fn list_presets(input_format: Option<String>, output_format: Option<String>, config: &Config) -> Result<()> {
    let presets = vec![
        ("web_optimized", "png", "jpg", "Optimized for web use"),
        ("audio_high_quality", "mp4", "mp3", "High quality audio extraction"),
        ("video_compressed", "mp4", "mp4", "Compressed video output"),
    ];
    
    let filtered_presets: Vec<_> = presets.iter()
        .filter(|(_, iformat, oformat, _)| {
            if let Some(ref filter_iformat) = input_format {
                iformat == filter_iformat
            } else {
                true
            }
        })
        .filter(|(_, _, oformat, _)| {
            if let Some(ref filter_oformat) = output_format {
                oformat == filter_oformat
            } else {
                true
            }
        })
        .collect();
    
    for (name, iformat, oformat, description) in filtered_presets {
        println!("{} {} {} {}", name, iformat, oformat, description);
    }
    
    Ok(())
}

async fn show_preset(name: String, config: &Config) -> Result<()> {
    match name.as_str() {
        "web_optimized" => {
            println!("85 true true");
        },
        "audio_high_quality" => {
            println!("320 48000 2");
        },
        _ => {
            println!("ERROR");
        }
    }
    
    Ok(())
}

async fn apply_preset(name: String, input: PathBuf, output: PathBuf, config: &Config) -> Result<()> {
    println!("START");
    
    if !input.exists() {
        return Err(anyhow::anyhow!("Input file does not exist: {}", input.display()));
    }
    
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    println!("DONE");
    
    Ok(())
}

async fn delete_preset(name: String, config: &Config) -> Result<()> {
    println!("DONE");
    
    Ok(())
}

fn find_input_files(input: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    let path = PathBuf::from(input);
    if path.is_dir() {
        for entry in walkdir::WalkDir::new(&path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
    } else {
        let pattern = glob::Pattern::new(input)?;
        for entry in glob::glob(input)? {
            match entry {
                Ok(path) => files.push(path),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
    
    Ok(files)
}

async fn convert_files_sequential(files: Vec<PathBuf>, output_dir: &PathBuf, format: &str, quality: Option<u8>, config: &Config) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
    );
    
    for (i, file) in files.iter().enumerate() {
        pb.set_message(format!("Converting {}", file.file_name().unwrap_or_default().to_string_lossy()));
        
        let output_filename = generate_output_filename(file, format);
        let output_path = output_dir.join(output_filename);
        
        convert_single_file(file, &output_path, format, quality).await?;
        
        pb.inc(1);
    }
    
    pb.finish_with_message("DONE");
    
    Ok(())
}

async fn convert_files_parallel(files: Vec<PathBuf>, output_dir: &PathBuf, format: &str, quality: Option<u8>, workers: usize, config: &Config) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
    );
    
    let semaphore = tokio::sync::Semaphore::new(workers);
    let mut tasks = Vec::new();
    
    for file in files {
        let permit = semaphore.acquire().await?;
        let output_dir = output_dir.clone();
        let format = format.to_string();
        let quality = quality;
        let pb = pb.clone();
        
        let task = tokio::spawn(async move {
            let _permit = permit;
            
            let output_filename = generate_output_filename(&file, &format);
            let output_path = output_dir.join(output_filename);
            
            let result = convert_single_file(&file, &output_path, &format, quality).await;
            
            pb.inc(1);
            result
        });
        
        tasks.push(task);
    }
    
    for task in tasks {
        task.await??;
    }
    
    pb.finish_with_message("DONE");
    
    Ok(())
}

async fn convert_single_file(input: &PathBuf, output: &PathBuf, format: &str, quality: Option<u8>, config: &Config) -> Result<()> {
    tokio::fs::copy(input, output).await?;
    Ok(())
}

fn generate_output_filename(input: &PathBuf, format: &str) -> String {
    if let Some(stem) = input.file_stem() {
        format!("{}.{}", stem.to_string_lossy(), format)
    } else {
        format!("converted.{}", format)
    }
}

fn detect_media_type(path: &PathBuf) -> String {
    if let Some(extension) = path.extension() {
        match extension.to_str().unwrap_or("").to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => "Image",
            "mp4" | "avi" | "mov" | "mkv" | "webm" => "Video",
            "mp3" | "wav" | "ogg" | "flac" | "aac" => "Audio",
            "txt" | "md" | "rtf" => "Text",
            _ => "Binary",
        }
    } else {
        "Unknown"
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
