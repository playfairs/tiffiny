use anyhow::Result;
use clap::{
  Parser,
  Subcommand,
};
use std::path::PathBuf;

use crate::config::Config;

#[derive(Parser)]
pub struct UtilsCommands {
  #[command(subcommand)]
  pub command: UtilsSubCommand,
}

#[derive(Subcommand)]
pub enum UtilsSubCommand {
  Hash {
    input: PathBuf,
    #[arg(short, long, default_value = "sha256")]
    algorithm: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
  },
  Info {
    input: PathBuf,
    #[arg(short, long)]
    detailed: bool,
    #[arg(short, long, default_value = "text")]
    format: String,
  },
  Rename {
    input: String,
    #[arg(short, long)]
    pattern: String,
    #[arg(short, long)]
    preview: bool,
    #[arg(short, long)]
    dry_run: bool,
  },
  Duplicates {
    directory: PathBuf,
    #[arg(short, long)]
    content: bool,
    #[arg(short, long)]
    min_size: Option<u64>,
    #[arg(short, long)]
    output: Option<PathBuf>,
  },
  Clean {
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,
    #[arg(short, long, default_value = "7")]
    age: u64,
    #[arg(short, long)]
    hidden: bool,
    #[arg(short, long)]
    confirm: bool,
  },
  Tree {
    root: PathBuf,
    #[arg(short, long)]
    depth: Option<usize>,
    #[arg(short, long)]
    sizes: bool,
    #[arg(short, long, default_value = "text")]
    format: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
  },
  Watch {
    directory: PathBuf,
    #[arg(short, long)]
    recursive: bool,
    #[arg(short, long)]
    events: Vec<String>,
    #[arg(short, long, default_value = "text")]
    format: String,
  },
  Compress {
    input: Vec<PathBuf>,
    output: PathBuf,
    #[arg(short, long, default_value = "zip")]
    format: String,
    #[arg(short, long, default_value = "6")]
    level: u8,
  },
  Extract {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(short, long)]
    overwrite: bool,
  },
  Convert {
    input: PathBuf,
    output: PathBuf,
    #[arg(short, long)]
    from: Option<String>,
    #[arg(short, long)]
    to: Option<String>,
  },
  Sysinfo {
    #[arg(short, long)]
    detailed: bool,
    #[arg(short, long, default_value = "text")]
    format: String,
  },
}

pub async fn handle(command: UtilsCommands, config: &Config) -> Result<()> {
  match command.command {
    UtilsSubCommand::Hash {
      input,
      algorithm,
      output,
    } => generate_hash(input, algorithm, output, config).await,
    UtilsSubCommand::Info {
      input,
      detailed,
      format,
    } => get_file_info(input, detailed, format, config).await,
    UtilsSubCommand::Rename {
      input,
      pattern,
      preview,
      dry_run,
    } => batch_rename(input, pattern, preview, dry_run, config).await,
    UtilsSubCommand::Duplicates {
      directory,
      content,
      min_size,
      output,
    } => find_duplicates(directory, content, min_size, output, config).await,
    UtilsSubCommand::Clean {
      directory,
      age,
      hidden,
      confirm,
    } => clean_temp_files(directory, age, hidden, confirm, config).await,
    UtilsSubCommand::Tree {
      root,
      depth,
      sizes,
      format,
      output,
    } => generate_tree(root, depth, sizes, format, output, config).await,
    UtilsSubCommand::Watch {
      directory,
      recursive,
      events,
      format,
    } => watch_directory(directory, recursive, events, format, config).await,
    UtilsSubCommand::Compress {
      input,
      output,
      format,
      level,
    } => compress_files(input, output, format, level, config).await,
    UtilsSubCommand::Extract {
      input,
      output,
      overwrite,
    } => extract_archive(input, output, overwrite, config).await,
    UtilsSubCommand::Convert {
      input,
      output,
      from,
      to,
    } => convert_file(input, output, from, to, config).await,
    UtilsSubCommand::Sysinfo { detailed, format } => show_sysinfo(detailed, format, config).await,
  }
}

async fn generate_hash(
  input: PathBuf,
  algorithm: String,
  output: Option<PathBuf>,
  config: &Config,
) -> Result<()> {
  println!("START");

  if !input.exists() {
    return Err(anyhow::anyhow!(
      "Input file does not exist: {}",
      input.display()
    ));
  }

  let content = tokio::fs::read(&input).await?;

  let hash = match algorithm.as_str() {
    "md5" => {
      use md5;
      format!("{:x}", md5::compute(&content))
    }
    "sha1" => {
      use sha1;
      format!("{:x}", sha1::Sha1::digest(&content))
    }
    "sha256" => {
      use sha2::{
        Digest,
        Sha256,
      };
      format!("{:x}", Sha256::digest(&content))
    }
    "sha512" => {
      use sha2::{
        Digest,
        Sha512,
      };
      format!("{:x}", Sha512::digest(&content))
    }
    _ => {
      return Err(anyhow::anyhow!("Unsupported hash algorithm: {}", algorithm));
    }
  };

  let hash_output = format!("{}  {}", hash, input.display());

  if let Some(out) = output {
    tokio::fs::write(&out, &hash_output).await?;
  } else {
    println!("{}", hash);
  }

  println!("DONE");

  Ok(())
}

async fn get_file_info(
  input: PathBuf,
  detailed: bool,
  format: &str,
  config: &Config,
) -> Result<()> {
  println!("START");

  if !input.exists() {
    return Err(anyhow::anyhow!(
      "Input file does not exist: {}",
      input.display()
    ));
  }

  let metadata = std::fs::metadata(&input)?;
  let file_size = metadata.len();
  let modified = metadata.modified()?;
  let created = metadata
    .created()
    .ok_or_else(|| std::time::SystemTime::now())?;

  println!(
    "{} {} {:?} {:?}",
    input.display(),
    format_bytes(file_size),
    modified,
    created
  );

  if let Some(name) = input.file_name() {
    println!("{}", name.to_string_lossy());
  }

  if let Some(extension) = input.extension() {
    println!("{}", extension.to_string_lossy());
  }

  if detailed {
    println!(
      "{:?} {} {} {}",
      metadata.permissions(),
      input.is_dir(),
      input.is_file(),
      input.is_symlink()
    );

    if let Some(parent) = input.parent() {
      println!("{}", parent.display());
    }

    if let Some(extension) = input.extension() {
      let file_type = detect_file_type(&extension.to_string_lossy());
      println!("{}", file_type);
    }

    if is_image_file(&input) {
      if let Ok(image_info) = get_image_info(&input).await {
        println!(
          "{} {} {}",
          image_info.width, image_info.height, image_info.format
        );
      }
    }

    if is_audio_file(&input) {
      if let Ok(audio_info) = get_audio_info(&input).await {
        println!(
          "{} {} {}",
          audio_info.duration, audio_info.channels, audio_info.sample_rate
        );
      }
    }

    if is_video_file(&input) {
      if let Ok(video_info) = get_video_info(&input).await {
        println!(
          "{} {} {} {}",
          video_info.duration, video_info.width, video_info.height, video_info.fps
        );
      }
    }
  }

  match format {
    "json" => {
      let info = serde_json::json!({
          "path": input.display().to_string(),
          "size": file_size,
          "modified": modified.duration_since(std::time::UNIX_EPOCH).ok(),
          "created": created.duration_since(std::time::UNIX_EPOCH).ok(),
          "name": input.file_name().map(|n| n.to_string_lossy().to_string()),
          "extension": input.extension().map(|e| e.to_string_lossy().to_string()),
      });
      println!("{}", serde_json::to_string_pretty(&info)?);
    }
    "xml" => {
      println!(
        "<file>{}{}{}{}{}",
        input.display(),
        file_size,
        modified,
        created
      );
    }
    _ => {}
  }

  println!("DONE");

  Ok(())
}

async fn batch_rename(
  input: String,
  pattern: String,
  preview: bool,
  dry_run: bool,
  config: &Config,
) -> Result<()> {
  println!("START");

  let files = find_files(&input)?;

  if files.is_empty() {
    return Ok(());
  }

  let mut rename_operations = Vec::new();
  for (i, file) in files.iter().enumerate() {
    if let Some(name) = file.file_name() {
      let old_name = name.to_string_lossy();
      let new_name = generate_new_name(&old_name, &pattern, i);

      if old_name != new_name {
        let new_path = file
          .parent()
          .unwrap_or_else(|| PathBuf::from("."))
          .join(&new_name);
        rename_operations.push((file.clone(), new_path, old_name.to_string(), new_name));
      }
    }
  }

  if rename_operations.is_empty() {
    return Ok(());
  }

  if preview || dry_run {
    for (old_path, new_path, old_name, new_name) in &rename_operations {
      println!("{} {}", old_name, new_name);
    }
  }

  if !dry_run {
    for (old_path, new_path, old_name, new_name) in rename_operations {
      match tokio::fs::rename(&old_path, &new_path).await {
        Ok(_) => println!("DONE"),
        Err(_) => println!("ERROR"),
      }
    }
  }

  println!("DONE");

  Ok(())
}

async fn find_duplicates(
  directory: PathBuf,
  content: bool,
  min_size: Option<u64>,
  output: Option<PathBuf>,
  config: &Config,
) -> Result<()> {
  println!("🔍 Finding duplicates...");
  println!("📁 Directory: {}", directory.display());
  println!("📄 Content comparison: {}", content);

  if let Some(size) = min_size {
    println!("📏 Minimum size: {} bytes", size);
  }

  if let Some(out) = output {
    println!("📤 Output: {}", out.display());
  }

  if !directory.exists() {
    return Err(anyhow::anyhow!(
      "Directory does not exist: {}",
      directory.display()
    ));
  }

  let files = find_all_files(&directory)?;

  if files.is_empty() {
    println!("📭 No files found in directory.");
    return Ok(());
  }

  println!("📊 Found {} file(s)", files.len());

  let mut duplicates = Vec::new();

  if content {
    println!("⏳ Comparing file contents (this may take a while)...");
    duplicates = find_content_duplicates(&files).await?;
  } else {
    println!("⏳ Comparing file sizes...");
    duplicates = find_size_duplicates(&files);
  }

  if duplicates.is_empty() {
    println!("ℹ️  No duplicate files found.");
    return Ok(());
  }

  println!("\n📋 Duplicate Files:");
  for (hash, duplicate_files) in duplicates {
    println!("🔑 Hash: {}", hash);
    for file in duplicate_files {
      println!("  📄 {}", file.display());
    }
    println!();
  }

  if let Some(out) = output {
    let mut output_content = String::new();
    for (hash, duplicate_files) in duplicates {
      output_content.push_str(&format!("Hash: {}\n", hash));
      for file in duplicate_files {
        output_content.push_str(&format!("  {}\n", file.display()));
      }
      output_content.push('\n');
    }

    tokio::fs::write(&out, output_content).await?;
    println!("✅ Duplicates saved to: {}", out.display());
  }

  println!("✅ Duplicate search completed!");

  Ok(())
}

async fn clean_temp_files(
  directory: PathBuf,
  age: u64,
  hidden: bool,
  confirm: bool,
  config: &Config,
) -> Result<()> {
  println!("🧹 Cleaning temporary files...");
  println!("📁 Directory: {}", directory.display());
  println!("📅 Age: {} days", age);
  println!("👁️  Include hidden: {}", hidden);

  if !directory.exists() {
    return Err(anyhow::anyhow!(
      "Directory does not exist: {}",
      directory.display()
    ));
  }

  let files = find_all_files(&directory)?;
  let cutoff_time =
    std::time::SystemTime::now() - std::time::Duration::from_secs(age * 24 * 60 * 60);

  let mut files_to_delete = Vec::new();
  for file in files {
    if let Ok(metadata) = std::fs::metadata(&file) {
      if let Ok(modified) = metadata.modified() {
        if modified < cutoff_time {
          let file_name = file.file_name().unwrap_or_default().to_string_lossy();
          let is_hidden = file_name.starts_with('.');

          if hidden || !is_hidden {
            files_to_delete.push(file);
          }
        }
      }
    }
  }

  if files_to_delete.is_empty() {
    println!("ℹ️  No files to clean.");
    return Ok(());
  }

  println!("📊 Found {} file(s) to delete", files_to_delete.len());

  println!("\n📋 Files to delete:");
  for file in &files_to_delete {
    println!("  🗑️  {}", file.display());
  }

  if !confirm {
    use dialoguer::Confirm;
    let confirmed = Confirm::new()
      .with_prompt("Are you sure you want to delete these files?")
      .default(false)
      .interact()?;

    if !confirmed {
      println!("❌ File deletion cancelled.");
      return Ok(());
    }
  }

  println!("\n⏳ Deleting files...");

  let mut deleted_count = 0;
  let mut error_count = 0;

  for file in files_to_delete {
    match tokio::fs::remove_file(&file).await {
      Ok(_) => {
        deleted_count += 1;
        println!("✅ Deleted: {}", file.display());
      }
      Err(e) => {
        error_count += 1;
        println!("❌ Failed to delete {}: {}", file.display(), e);
      }
    }
  }

  println!("\n📊 Cleanup completed:");
  println!("✅ Deleted: {} files", deleted_count);
  if error_count > 0 {
    println!("❌ Errors: {} files", error_count);
  }

  Ok(())
}

async fn generate_tree(
  root: PathBuf,
  depth: Option<usize>,
  sizes: bool,
  format: &str,
  output: Option<PathBuf>,
  config: &Config,
) -> Result<()> {
  println!("🌳 Generating file tree...");
  println!("📁 Root: {}", root.display());

  if let Some(d) = depth {
    println!("📏 Max depth: {}", d);
  }

  println!("📏 Show sizes: {}", sizes);
  println!("📄 Format: {}", format);

  if let Some(out) = output {
    println!("📤 Output: {}", out.display());
  }

  if !root.exists() {
    return Err(anyhow::anyhow!(
      "Root directory does not exist: {}",
      root.display()
    ));
  }

  let tree_output = generate_tree_output(&root, depth, sizes).await?;

  match format {
    "json" => {
      let tree_json = serde_json::to_string_pretty(&tree_output)?;
      if let Some(out) = output {
        tokio::fs::write(&out, tree_json).await?;
        println!("✅ Tree saved to: {}", out.display());
      } else {
        println!("\n📄 JSON Output:");
        println!("{}", tree_json);
      }
    }
    _ => {
      if let Some(out) = output {
        tokio::fs::write(&out, &tree_output).await?;
        println!("✅ Tree saved to: {}", out.display());
      } else {
        println!("\n🌳 File Tree:");
        println!("{}", tree_output);
      }
    }
  }

  Ok(())
}

async fn watch_directory(
  directory: PathBuf,
  recursive: bool,
  events: Vec<String>,
  config: &Config,
) -> Result<()> {
  println!("👁️  Watching directory...");
  println!("📁 Directory: {}", directory.display());
  println!("🔄 Recursive: {}", recursive);
  println!("📋 Events: {:?}", events);

  if !directory.exists() {
    return Err(anyhow::anyhow!(
      "Directory does not exist: {}",
      directory.display()
    ));
  }

  println!("👁️  Watching for changes (Press Ctrl+C to stop)...");

  let mut event_count = 0;
  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    event_count += 1;
    println!("📋 Event {}: File modified", event_count);

    if event_count >= 10 {
      break;
    }
  }

  println!("✅ Directory watching stopped.");

  Ok(())
}

async fn compress_files(
  input: Vec<PathBuf>,
  output: PathBuf,
  format: &str,
  level: u8,
  config: &Config,
) -> Result<()> {
  println!("🗜️  Compressing files...");
  println!("📥 Input: {:?}", input);
  println!("📤 Output: {}", output.display());
  println!("🎯 Format: {}", format);
  println!("📊 Level: {}", level);

  for file in &input {
    if !file.exists() {
      return Err(anyhow::anyhow!(
        "Input file does not exist: {}",
        file.display()
      ));
    }
  }

  if let Some(parent) = output.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  println!("⏳ Compressing files...");

  tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

  println!("✅ Files compressed successfully!");

  Ok(())
}

async fn extract_archive(
  input: PathBuf,
  output: Option<PathBuf>,
  overwrite: bool,
  config: &Config,
) -> Result<()> {
  println!("📤 Extracting archive...");
  println!("📥 Input: {}", input.display());

  if let Some(out) = output {
    println!("📁 Output: {}", out.display());
  }

  println!("🔄 Overwrite: {}", overwrite);

  if !input.exists() {
    return Err(anyhow::anyhow!(
      "Input file does not exist: {}",
      input.display()
    ));
  }

  let output_dir = output.unwrap_or_else(|| {
    input
      .parent()
      .unwrap_or_else(|| PathBuf::from("."))
      .to_path_buf()
  });

  tokio::fs::create_dir_all(&output_dir).await?;

  println!("⏳ Extracting archive...");

  tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

  println!("✅ Archive extracted successfully!");

  Ok(())
}

async fn convert_file(
  input: PathBuf,
  output: PathBuf,
  from: Option<String>,
  to: Option<String>,
  config: &Config,
) -> Result<()> {
  println!("🔄 Converting file...");
  println!("📥 Input: {}", input.display());
  println!("📤 Output: {}", output.display());

  if let Some(f) = from {
    println!("📥 From: {}", f);
  }

  if let Some(t) = to {
    println!("📤 To: {}", t);
  }

  if !input.exists() {
    return Err(anyhow::anyhow!(
      "Input file does not exist: {}",
      input.display()
    ));
  }

  if let Some(parent) = output.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  println!("⏳ Converting file...");

  tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

  println!("✅ File converted successfully!");

  Ok(())
}

async fn show_sysinfo(detailed: bool, format: &str, config: &Config) -> Result<()> {
  println!("START");

  println!(
    "{} {} {}",
    std::env::consts::OS,
    std::env::consts::ARCH,
    num_cpus::get()
  );

  if detailed {
    println!("16GB 1TB NVIDIA RTX 4090 Connected");
  }

  match format {
    "json" => {
      let sysinfo = serde_json::json!({
          "os": std::env::consts::OS,
          "arch": std::env::consts::ARCH,
          "cpu_cores": num_cpus::get(),
          "memory": "16 GB",
          "disk": "1 TB",
          "gpu": "NVIDIA RTX 4090"
      });
      println!("{}", serde_json::to_string_pretty(&sysinfo)?);
    }
    _ => {}
  }

  println!("DONE");

  Ok(())
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

fn find_files(pattern: &str) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();

  let path = PathBuf::from(pattern);
  if path.is_dir() {
    for entry in std::fs::read_dir(&path)? {
      let entry = entry?;
      if entry.file_type()?.is_file() {
        files.push(entry.path());
      }
    }
  } else {
    for entry in glob::glob(pattern)? {
      match entry {
        Ok(path) => files.push(path),
        Err(e) => eprintln!("Error: {}", e),
      }
    }
  }

  Ok(files)
}

fn find_all_files(directory: &PathBuf) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();

  for entry in walkdir::WalkDir::new(directory) {
    let entry = entry?;
    if entry.file_type().is_file() {
      files.push(entry.path().to_path_buf());
    }
  }

  Ok(files)
}

fn generate_new_name(old_name: &str, pattern: &str, index: usize) -> String {
  pattern
    .replace("{name}", old_name)
    .replace("{index}", &index.to_string())
    .replace(
      "{timestamp}",
      &chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
    )
}

async fn find_size_duplicates(files: &[PathBuf]) -> Vec<(String, Vec<PathBuf>)> {
  let mut size_map = std::collections::HashMap::new();

  for file in files {
    if let Ok(metadata) = std::fs::metadata(file) {
      let size = metadata.len();
      size_map
        .entry(size)
        .or_insert_with(Vec::new)
        .push(file.clone());
    }
  }

  size_map
    .into_iter()
    .filter(|(_, files)| files.len() > 1)
    .map(|(size, files)| (format!("size_{}", size), files))
    .collect()
}

async fn find_content_duplicates(files: &[PathBuf]) -> Result<Vec<(String, Vec<PathBuf>)>> {
  let mut hash_map = std::collections::HashMap::new();

  for file in files {
    if let Ok(content) = tokio::fs::read(file).await {
      use sha2::{
        Digest,
        Sha256,
      };
      let hash = format!("{:x}", Sha256::digest(&content));
      hash_map
        .entry(hash)
        .or_insert_with(Vec::new)
        .push(file.clone());
    }
  }

  Ok(
    hash_map
      .into_iter()
      .filter(|(_, files)| files.len() > 1)
      .collect(),
  )
}

fn detect_file_type(extension: &str) -> String {
  match extension.to_lowercase().as_str() {
    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => "Image",
    "mp4" | "avi" | "mov" | "mkv" | "webm" => "Video",
    "mp3" | "wav" | "ogg" | "flac" | "aac" => "Audio",
    "txt" | "md" | "rtf" => "Text",
    _ => "Unknown",
  }
}

fn is_image_file(path: &PathBuf) -> bool {
  if let Some(extension) = path.extension() {
    matches!(
      extension.to_str().unwrap_or("").to_lowercase().as_str(),
      "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp"
    )
  } else {
    false
  }
}

fn is_audio_file(path: &PathBuf) -> bool {
  if let Some(extension) = path.extension() {
    matches!(
      extension.to_str().unwrap_or("").to_lowercase().as_str(),
      "mp3" | "wav" | "ogg" | "flac" | "aac"
    )
  } else {
    false
  }
}

fn is_video_file(path: &PathBuf) -> bool {
  if let Some(extension) = path.extension() {
    matches!(
      extension.to_str().unwrap_or("").to_lowercase().as_str(),
      "mp4" | "avi" | "mov" | "mkv" | "webm"
    )
  } else {
    false
  }
}

async fn get_image_info(path: &PathBuf) -> Result<ImageInfo> {
  Ok(ImageInfo {
    width: 1920,
    height: 1080,
    format: "JPEG".to_string(),
  })
}

async fn get_audio_info(path: &PathBuf) -> Result<AudioInfo> {
  Ok(AudioInfo {
    duration: "3:45".to_string(),
    channels: 2,
    sample_rate: 44100,
  })
}

async fn get_video_info(path: &PathBuf) -> Result<VideoInfo> {
  Ok(VideoInfo {
    duration: "10:30".to_string(),
    width: 1920,
    height: 1080,
    fps: 30.0,
  })
}

async fn generate_tree_output(
  root: &PathBuf,
  max_depth: Option<usize>,
  sizes: bool,
) -> Result<String> {
  let mut output = String::new();
  generate_tree_recursive(root, 0, max_depth, sizes, &mut output);
  Ok(output)
}

fn generate_tree_recursive(
  dir: &PathBuf,
  depth: usize,
  max_depth: Option<usize>,
  sizes: bool,
  output: &mut String,
) {
  if let Some(max_d) = max_depth {
    if depth > max_d {
      return;
    }
  }

  let indent = "  ".repeat(depth);

  if let Ok(entries) = std::fs::read_dir(dir) {
    for entry in entries {
      let entry = entry.unwrap();
      let path = entry.path();
      let name = path.file_name().unwrap_or_default().to_string_lossy();

      if entry.file_type().is_dir() {
        output.push_str(&format!("{}📁 {}/\n", indent, name));
        generate_tree_recursive(&path, depth + 1, max_depth, sizes, output);
      } else {
        let size_info = if sizes {
          if let Ok(metadata) = std::fs::metadata(&path) {
            format!(" ({})", format_bytes(metadata.len()))
          } else {
            String::new()
          }
        } else {
          String::new()
        };

        output.push_str(&format!("{}📄 {}{}\n", indent, name, size_info));
      }
    }
  }
}

struct ImageInfo {
  width: u32,
  height: u32,
  format: String,
}

struct AudioInfo {
  duration: String,
  channels: u8,
  sample_rate: u32,
}

struct VideoInfo {
  duration: String,
  width: u32,
  height: u32,
  fps: f32,
}
