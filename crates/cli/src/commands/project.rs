use anyhow::Result;
use clap::{
  Parser,
  Subcommand,
};
use serde::{
  Deserialize,
  Serialize,
};
use std::path::PathBuf;
use tabled::{
  Table,
  Tabled,
  settings::Style,
};

use crate::config::Config;

#[derive(Parser)]
pub struct ProjectCommands {
  #[command(subcommand)]
  pub command: ProjectSubCommand,
}

#[derive(Subcommand)]
pub enum ProjectSubCommand {
  New {
    #[arg(short, long)]
    name: String,
    #[arg(short, long, default_value = "default")]
    template: String,
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,
  },
  List {
    #[arg(short, long)]
    tag: Option<String>,
    #[arg(short, long)]
    author: Option<String>,
    #[arg(short, long, default_value = "modified")]
    sort: String,
    #[arg(short, long)]
    reverse: bool,
  },
  Open {
    project: String,
    #[arg(short, long)]
    interactive: bool,
  },
  Close {
    project: Option<String>,
    #[arg(short, long)]
    save: bool,
  },
  Save {
    project: Option<String>,
    #[arg(short, long)]
    force: bool,
  },
  Delete {
    project: String,
    #[arg(short, long)]
    confirm: bool,
  },
  Info {
    project: String,
    #[arg(short, long)]
    detailed: bool,
  },
  Import {
    file: PathBuf,
    #[arg(short, long)]
    directory: Option<PathBuf>,
    #[arg(short, long)]
    overwrite: bool,
  },
  Export {
    project: String,
    path: PathBuf,
    #[arg(short, long, default_value = "tiffiny")]
    format: String,
    #[arg(short, long)]
    include_assets: bool,
  },
  Backup {
    project: String,
    #[arg(short, long)]
    directory: Option<PathBuf>,
    #[arg(short, long)]
    compress: bool,
  },
  Restore {
    backup: PathBuf,
    #[arg(short, long)]
    directory: Option<PathBuf>,
    #[arg(short, long)]
    overwrite: bool,
  },
  Search {
    query: String,
    #[arg(short, long)]
    fields: Vec<String>,
    #[arg(short, long)]
    case_sensitive: bool,
    #[arg(short, long)]
    limit: Option<usize>,
  },
  AddAsset {
    project: String,
    file: PathBuf,
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long)]
    asset_type: Option<String>,
  },
  RemoveAsset {
    project: String,
    asset: String,
    #[arg(short, long)]
    confirm: bool,
  },
  ListAssets {
    project: String,
    #[arg(short, long)]
    asset_type: Option<String>,
    #[arg(short, long, default_value = "name")]
    sort: String,
  },
}

#[derive(Tabled, Serialize, Deserialize)]
struct ProjectInfo {
  id: String,
  name: String,
  author: String,
  created: String,
  modified: String,
  size: String,
  assets: usize,
}

#[derive(Tabled, Serialize, Deserialize)]
struct AssetInfo {
  id: String,
  name: String,
  asset_type: String,
  size: String,
  created: String,
  path: String,
}

pub async fn handle(command: ProjectCommands, config: &Config) -> Result<()> {
  match command.command {
    ProjectSubCommand::New {
      name,
      template,
      directory,
    } => create_project(name, template, directory, config).await,
    ProjectSubCommand::List {
      tag,
      author,
      sort,
      reverse,
    } => list_projects(tag, author, sort, reverse, config).await,
    ProjectSubCommand::Open {
      project,
      interactive,
    } => open_project(project, interactive, config).await,
    ProjectSubCommand::Close { project, save } => close_project(project, save, config).await,
    ProjectSubCommand::Save { project, force } => save_project(project, force, config).await,
    ProjectSubCommand::Delete { project, confirm } => {
      delete_project(project, confirm, config).await
    }
    ProjectSubCommand::Info { project, detailed } => {
      show_project_info(project, detailed, config).await
    }
    ProjectSubCommand::Import {
      file,
      directory,
      overwrite,
    } => import_project(file, directory, overwrite, config).await,
    ProjectSubCommand::Export {
      project,
      path,
      format,
      include_assets,
    } => export_project(project, path, format, include_assets, config).await,
    ProjectSubCommand::Backup {
      project,
      directory,
      compress,
    } => backup_project(project, directory, compress, config).await,
    ProjectSubCommand::Restore {
      backup,
      directory,
      overwrite,
    } => restore_project(backup, directory, overwrite, config).await,
    ProjectSubCommand::Search {
      query,
      fields,
      case_sensitive,
      limit,
    } => search_projects(query, fields, case_sensitive, limit, config).await,
    ProjectSubCommand::AddAsset {
      project,
      file,
      name,
      asset_type,
    } => add_asset(project, file, name, asset_type, config).await,
    ProjectSubCommand::RemoveAsset {
      project,
      asset,
      confirm,
    } => remove_asset(project, asset, confirm, config).await,
    ProjectSubCommand::ListAssets {
      project,
      asset_type,
      sort,
    } => list_assets(project, asset_type, sort, config).await,
  }
}

async fn create_project(
  name: String,
  template: String,
  directory: PathBuf,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let project_id = manager
    .create_project(name.clone(), directory.clone())
    .await?;

  println!("DONE");

  Ok(())
}

async fn list_projects(
  tag: Option<String>,
  author: Option<String>,
  sort: String,
  reverse: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let projects = manager.get_all_projects();

  let filtered_projects: Vec<_> = projects
    .iter()
    .filter(|p| {
      if let Some(ref tag_filter) = tag {
        p.has_tag(tag_filter)
      } else {
        true
      }
    })
    .filter(|p| {
      if let Some(ref author_filter) = author {
        p.author == *author_filter
      } else {
        true
      }
    })
    .collect();

  if filtered_projects.is_empty() {
    return Ok(());
  }

  let project_infos: Vec<ProjectInfo> = filtered_projects
    .iter()
    .map(|p| ProjectInfo {
      id: p.id.clone(),
      name: p.name.clone(),
      author: p.author.clone(),
      created: p.created_at.format("%Y-%m-%d").to_string(),
      modified: p.modified_at.format("%Y-%m-%d").to_string(),
      size: format_bytes(p.get_total_size()),
      assets: p.get_asset_count(),
    })
    .collect();

  let mut sorted_infos = project_infos;
  match sort.as_str() {
    "name" => sorted_infos.sort_by(|a, b| a.name.cmp(&b.name)),
    "created" => sorted_infos.sort_by(|a, b| a.created.cmp(&b.created)),
    "modified" => sorted_infos.sort_by(|a, b| a.modified.cmp(&b.modified)),
    "size" => sorted_infos.sort_by(|a, b| a.size.cmp(&b.size)),
    "assets" => sorted_infos.sort_by(|a, b| a.assets.cmp(&b.assets)),
    _ => sorted_infos.sort_by(|a, b| a.modified.cmp(&b.modified)),
  }

  if reverse {
    sorted_infos.reverse();
  }

  let table = Table::new(&sorted_infos).with(Style::modern()).to_string();

  println!("{}", table);
  println!("{}", sorted_infos.len());

  Ok(())
}

async fn open_project(project: String, interactive: bool, config: &Config) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let opened_project = manager.open_project(&project).await?;

  println!("DONE");

  Ok(())
}

async fn close_project(project: Option<String>, save: bool, config: &Config) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let project_id = project.unwrap_or_else(|| "current".to_string());

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  manager.close_project(&project_id).await?;

  println!("DONE");

  Ok(())
}

async fn save_project(project: Option<String>, force: bool, config: &Config) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let project_id = project.unwrap_or_else(|| "current".to_string());

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  manager.save_project(&project_id).await?;

  println!("DONE");

  Ok(())
}

async fn delete_project(project: String, confirm: bool, config: &Config) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  if !confirm {
    use dialoguer::Confirm;
    let confirmed = Confirm::new()
      .with_prompt(&format!(
        "Are you sure you want to delete project '{}'?",
        project
      ))
      .default(false)
      .interact()?;

    if !confirmed {
      return Ok(());
    }
  }

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  manager.delete_project(&project).await?;

  println!("DONE");

  Ok(())
}

async fn show_project_info(project: String, detailed: bool, config: &Config) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let project_info = manager.get_project(&project);

  if let Some(p) = project_info {
    println!(
      "{} {} {} {} {} {} {} {} {}",
      p.id,
      p.name,
      p.description,
      p.author,
      p.version,
      p.created_at.format("%Y-%m-%d %H:%M:%S"),
      p.modified_at.format("%Y-%m-%d %H:%M:%S"),
      format_bytes(p.get_total_size()),
      p.get_asset_count()
    );

    if !p.tags.is_empty() {
      println!("{}", p.tags.join(", "));
    }

    if detailed {
      println!(
        "{} {} {} {}",
        p.settings.auto_save,
        p.settings.backup_enabled,
        p.settings.compression_enabled,
        p.settings.encryption_enabled
      );

      if !p.assets.is_empty() {
        for (id, asset) in &p.assets {
          println!(
            "{} {} {}",
            asset.name,
            asset.asset_type,
            format_bytes(asset.size)
          );
        }
      }
    }
  } else {
    println!("ERROR");
  }

  Ok(())
}

async fn import_project(
  file: PathBuf,
  directory: Option<PathBuf>,
  overwrite: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let project_id = manager.import_project(file.clone()).await?;

  println!("DONE");

  Ok(())
}

async fn export_project(
  project: String,
  path: PathBuf,
  format: String,
  include_assets: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  manager.export_project(&project, path.clone()).await?;

  println!("DONE");

  Ok(())
}

async fn backup_project(
  project: String,
  directory: Option<PathBuf>,
  compress: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let backup_path = manager.backup_project(&project).await?;

  println!("DONE");

  Ok(())
}

async fn restore_project(
  backup: PathBuf,
  directory: Option<PathBuf>,
  overwrite: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let project_id = manager.restore_project(&backup).await?;

  println!("DONE");

  Ok(())
}

async fn search_projects(
  query: String,
  fields: Vec<String>,
  case_sensitive: bool,
  limit: Option<usize>,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let results = manager.search_projects(&query);

  let limited_results: Vec<_> = if let Some(limit) = limit {
    results.into_iter().take(limit).collect()
  } else {
    results
  };

  for project in &limited_results {
    println!("{} {}", project.name, project.id);
  }

  Ok(())
}

async fn add_asset(
  project: String,
  file: PathBuf,
  name: Option<String>,
  asset_type: Option<String>,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project::{
    Asset,
    AssetType,
  };
  use tiffiny_project::project_manager::ProjectManager;

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let mut project_data = manager
    .get_project(&project)
    .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

  let detected_type = if let Some(at) = asset_type {
    match at.to_lowercase().as_str() {
      "image" => AssetType::Image,
      "video" => AssetType::Video,
      "audio" => AssetType::Audio,
      "text" => AssetType::Text,
      "binary" => AssetType::Binary,
      _ => AssetType::Binary,
    }
  } else {
    detect_asset_type(&file)
  };

  let asset_name = name.unwrap_or_else(|| {
    file
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string()
  });

  let metadata = std::fs::metadata(&file)?;
  let asset = Asset {
    id: uuid::Uuid::new_v4().to_string(),
    name: asset_name,
    asset_type: detected_type,
    path: file,
    size: metadata.len(),
    created_at: metadata
      .created()
      .ok_or_else(|_| std::time::SystemTime::now())?,
    modified_at: metadata
      .modified()
      .ok_or_else(|_| std::time::SystemTime::now())?,
    metadata: tiffiny_project::project::AssetMetadata::default(),
  };

  project_data.add_asset(asset);

  println!("DONE");

  Ok(())
}

async fn remove_asset(
  project: String,
  asset: String,
  confirm: bool,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  if !confirm {
    use dialoguer::Confirm;
    let confirmed = Confirm::new()
      .with_prompt(&format!(
        "Are you sure you want to remove asset '{}'?",
        asset
      ))
      .default(false)
      .interact()?;

    if !confirmed {
      return Ok(());
    }
  }

  println!("START");

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let mut project_data = manager
    .get_project(&project)
    .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
  project_data.remove_asset(&asset);

  println!("DONE");

  Ok(())
}

async fn list_assets(
  project: String,
  asset_type: Option<String>,
  sort: String,
  config: &Config,
) -> Result<()> {
  use tiffiny_project::project_manager::ProjectManager;

  let manager = ProjectManager::new(
    uuid::Uuid::new_v4().to_string(),
    "CLI Project Manager".to_string(),
  );

  let project_data = manager
    .get_project(&project)
    .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

  let filtered_assets: Vec<_> = project_data
    .assets
    .values()
    .filter(|asset| {
      if let Some(ref at) = asset_type {
        format!("{:?}", asset.asset_type).to_lowercase() == at.to_lowercase()
      } else {
        true
      }
    })
    .collect();

  if filtered_assets.is_empty() {
    return Ok(());
  }

  let asset_infos: Vec<AssetInfo> = filtered_assets
    .iter()
    .map(|a| AssetInfo {
      id: a.id.clone(),
      name: a.name.clone(),
      asset_type: format!("{:?}", a.asset_type),
      size: format_bytes(a.size),
      created: a.created_at.format("%Y-%m-%d").to_string(),
      path: a.path.to_string_lossy().to_string(),
    })
    .collect();

  let mut sorted_infos = asset_infos;
  match sort.as_str() {
    "name" => sorted_infos.sort_by(|a, b| a.name.cmp(&b.name)),
    "type" => sorted_infos.sort_by(|a, b| a.asset_type.cmp(&b.asset_type)),
    "size" => sorted_infos.sort_by(|a, b| a.size.cmp(&b.size)),
    "created" => sorted_infos.sort_by(|a, b| a.created.cmp(&b.created)),
    _ => sorted_infos.sort_by(|a, b| a.name.cmp(&b.name)),
  }

  let table = Table::new(&sorted_infos).with(Style::modern()).to_string();

  println!("{}", table);
  println!("{}", sorted_infos.len());

  Ok(())
}

fn detect_asset_type(path: &PathBuf) -> AssetType {
  use tiffiny_project::project::AssetType;

  if let Some(extension) = path.extension() {
    match extension.to_str().unwrap_or("").to_lowercase().as_str() {
      "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => AssetType::Image,
      "mp4" | "avi" | "mov" | "mkv" | "webm" => AssetType::Video,
      "mp3" | "wav" | "ogg" | "flac" | "aac" => AssetType::Audio,
      "txt" | "md" | "rtf" => AssetType::Text,
      _ => AssetType::Binary,
    }
  } else {
    AssetType::Binary
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
