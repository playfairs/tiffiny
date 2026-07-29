use anyhow::Result;
use clap::{
  Parser,
  Subcommand,
};
use std::path::PathBuf;

use crate::config::Config;

#[derive(Parser)]
pub struct ProcessCommands {
  #[command(subcommand)]
  pub command: ProcessSubCommand,
}

#[derive(Subcommand)]
pub enum ProcessSubCommand {
  File {
    input: PathBuf,
    output: PathBuf,
    #[arg(short, long)]
    operation: String,
    #[arg(short, long)]
    params: Option<String>,
  },
  Batch {
    input: String,
    output: PathBuf,
    #[arg(short, long)]
    operation: String,
    #[arg(short, long)]
    params: Option<String>,
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
  List {
    #[arg(short, long)]
    media_type: Option<String>,
    #[arg(short, long)]
    detailed: bool,
  },
  Info {
    operation: String,
  },
}

pub async fn handle(command: ProcessCommands, config: &Config) -> Result<()> {
  match command.command {
    ProcessSubCommand::File {
      input,
      output,
      operation,
      params,
    } => process_file(input, output, operation, params, config).await,
    ProcessSubCommand::Batch {
      input,
      output,
      operation,
      params,
      parallel,
      workers,
    } => process_batch(input, output, operation, params, parallel, workers, config).await,
    ProcessSubCommand::Pipeline {
      config,
      input,
      output,
      dry_run,
    } => process_pipeline(config, input, output, dry_run, config).await,
    ProcessSubCommand::List {
      media_type,
      detailed,
    } => list_operations(media_type, detailed, config).await,
    ProcessSubCommand::Info { operation } => show_operation_info(operation, config).await,
  }
}

async fn process_file(
  input: PathBuf,
  output: PathBuf,
  operation: String,
  params: Option<String>,
  config: &Config,
) -> Result<()> {
  println!("START");

  if !input.exists() {
    return Err(anyhow::anyhow!(
      "Input file does not exist: {}",
      input.display()
    ));
  }

  if let Some(parent) = output.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

  println!("DONE");

  Ok(())
}

async fn process_batch(
  input: String,
  output: PathBuf,
  operation: String,
  params: Option<String>,
  parallel: bool,
  workers: usize,
  config: &Config,
) -> Result<()> {
  println!("START");

  tokio::fs::create_dir_all(&output).await?;

  let files = find_input_files(&input)?;

  if files.is_empty() {
    return Ok(());
  }

  if parallel {
    process_files_parallel(files, &output, &operation, params.as_deref(), workers).await?;
  } else {
    process_files_sequential(files, &output, &operation, params.as_deref()).await?;
  }

  println!("DONE");

  Ok(())
}

async fn process_pipeline(
  config: PathBuf,
  input: String,
  output: PathBuf,
  dry_run: bool,
  config_data: &Config,
) -> Result<()> {
  println!("START");

  if dry_run {
    return Ok(());
  }

  println!("DONE");

  Ok(())
}

async fn list_operations(
  media_type: Option<String>,
  detailed: bool,
  config: &Config,
) -> Result<()> {
  let operations = vec![
    ("databend", "Apply databending effects"),
    ("glitch", "Apply glitch effects"),
    ("pixel_sort", "Apply pixel sorting"),
    ("bit_crush", "Apply bit crushing"),
    ("data_mosh", "Apply data moshing"),
    ("compression_artifacts", "Apply compression artifacts"),
    ("error_diffusion", "Apply error diffusion"),
    ("procedural_noise", "Apply procedural noise"),
    ("color_manipulation", "Apply color manipulation"),
    ("texture_synthesis", "Apply texture synthesis"),
  ];

  if detailed {
    for (name, description) in operations {
      println!("{} {}", name, description);
    }
  } else {
    for (name, _) in operations {
      println!("{}", name);
    }
  }

  Ok(())
}

async fn show_operation_info(operation: String, config: &Config) -> Result<()> {
  match operation.as_str() {
    "databend" => {
      println!("effect_type intensity seed");
    }
    "glitch" => {
      println!("glitch_type amount frequency");
    }
    _ => {
      println!("ERROR");
    }
  }

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
        Err(_) => {}
      }
    }
  }

  Ok(files)
}

async fn process_files_sequential(
  files: Vec<PathBuf>,
  output_dir: &PathBuf,
  operation: &str,
  params: Option<&str>,
  config: &Config,
) -> Result<()> {
  use indicatif::{
    ProgressBar,
    ProgressStyle,
  };

  let pb = ProgressBar::new(files.len() as u64);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
      .unwrap(),
  );

  for (i, file) in files.iter().enumerate() {
    pb.set_message(format!(
      "Processing {}",
      file.file_name().unwrap_or_default().to_string_lossy()
    ));

    let output_filename = generate_output_filename(file, operation);
    let output_path = output_dir.join(output_filename);

    process_single_file(file, &output_path, operation, params).await?;

    pb.inc(1);
  }

  pb.finish_with_message("DONE");

  Ok(())
}

async fn process_files_parallel(
  files: Vec<PathBuf>,
  output_dir: &PathBuf,
  operation: &str,
  params: Option<&str>,
  workers: usize,
  config: &Config,
) -> Result<()> {
  use indicatif::{
    ProgressBar,
    ProgressStyle,
  };

  let pb = ProgressBar::new(files.len() as u64);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
      .unwrap(),
  );

  let semaphore = tokio::sync::Semaphore::new(workers);
  let mut tasks = Vec::new();

  for file in files {
    let permit = semaphore.acquire().await?;
    let output_dir = output_dir.clone();
    let operation = operation.to_string();
    let params = params.map(|p| p.to_string());
    let pb = pb.clone();

    let task = tokio::spawn(async move {
      let _permit = permit;

      let output_filename = generate_output_filename(&file, &operation);
      let output_path = output_dir.join(output_filename);

      let result = process_single_file(&file, &output_path, &operation, params.as_deref()).await;

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

async fn process_single_file(
  input: &PathBuf,
  output: &PathBuf,
  operation: &str,
  params: Option<&str>,
  config: &Config,
) -> Result<()> {
  tokio::fs::copy(input, output).await?;
  Ok(())
}

fn generate_output_filename(input: &PathBuf, operation: &str) -> String {
  if let Some(stem) = input.file_stem() {
    if let Some(extension) = input.extension() {
      format!(
        "{}_{}.{}",
        stem.to_string_lossy(),
        operation,
        extension.to_string_lossy()
      )
    } else {
      format!("{}_{}", stem.to_string_lossy(), operation)
    }
  } else {
    format!("processed_{}", operation)
  }
}
