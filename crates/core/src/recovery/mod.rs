use crate::prelude::*;
use parking_lot::RwLock;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub created_at: std::time::SystemTime,
  pub recovery_type: RecoveryType,
  pub data: RecoveryData,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
  AutoSave,
  ManualSave,
  CrashRecovery,
  ProjectSnapshot,
  SessionBackup,
  EmergencySave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryData {
  Project {
    project_id: Uuid,
    project_data: serde_json::Value,
    assets: HashMap<String, Vec<u8>>,
  },
  Session {
    session_id: Uuid,
    session_data: serde_json::Value,
  },
  Workspace {
    layout_data: serde_json::Value,
    open_files: Vec<String>,
    recent_projects: Vec<String>,
  },
  System {
    application_state: serde_json::Value,
    cache_state: serde_json::Value,
    temporary_files: Vec<String>,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
  pub auto_save_interval_seconds: u64,
  pub max_recovery_points: usize,
  pub recovery_directory: PathBuf,
  pub compression_enabled: bool,
  pub encryption_enabled: bool,
  pub cleanup_old_after_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
  pub total_recovery_points: usize,
  pub total_size_bytes: u64,
  pub oldest_recovery: Option<std::time::SystemTime>,
  pub newest_recovery: Option<std::time::SystemTime>,
  pub auto_saves: usize,
  pub manual_saves: usize,
  pub crash_recoveries: usize,
  pub cleanup_runs: u32,
}

pub struct RecoveryManager {
  recovery_points: Arc<RwLock<HashMap<Uuid, RecoveryPoint>>>,
  config: Arc<RwLock<RecoveryConfig>>,
  stats: Arc<RwLock<RecoveryStats>>,
  cleanup_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl RecoveryManager {
  pub fn new(config: RecoveryConfig) -> Result<Self> {
    std::fs::create_dir_all(&config.recovery_directory)?;

    let manager = Self {
      recovery_points: Arc::new(RwLock::new(HashMap::new())),
      config: Arc::new(RwLock::new(config)),
      stats: Arc::new(RwLock::new(RecoveryStats::default())),
      cleanup_task: Arc::new(RwLock::new(None)),
    };

    manager.load_existing_recovery_points()?;
    manager.update_stats()?;

    Ok(manager)
  }

  pub fn create_recovery_point(
    &self,
    name: String,
    description: String,
    recovery_type: RecoveryType,
    data: RecoveryData,
  ) -> Result<Uuid> {
    let recovery_id = Uuid::new_v4();
    let now = std::time::SystemTime::now();

    let recovery_point = RecoveryPoint {
      id: recovery_id,
      name,
      description,
      created_at: now,
      recovery_type,
      data,
      metadata: HashMap::new(),
    };

    self.save_recovery_point(&recovery_point)?;

    {
      let mut recovery_points = self.recovery_points.write();
      recovery_points.insert(recovery_id, recovery_point);
    }

    self.update_stats()?;
    self.enforce_max_recovery_points()?;

    Ok(recovery_id)
  }

  pub fn create_auto_save(
    &self,
    project_id: Uuid,
    project_data: serde_json::Value,
    assets: HashMap<String, Vec<u8>>,
  ) -> Result<Uuid> {
    let data = RecoveryData::Project {
      project_id,
      project_data,
      assets,
    };

    self.create_recovery_point(
      format!("Auto-save Project {}", project_id),
      "Automatic project save".to_string(),
      RecoveryType::AutoSave,
      data,
    )
  }

  pub fn create_crash_recovery(
    &self,
    session_id: Uuid,
    session_data: serde_json::Value,
  ) -> Result<Uuid> {
    let data = RecoveryData::Session {
      session_id,
      session_data,
    };

    self.create_recovery_point(
      format!("Crash Recovery {}", session_id),
      "Application crash recovery point".to_string(),
      RecoveryType::CrashRecovery,
      data,
    )
  }

  pub fn restore_recovery_point(&self, recovery_id: Uuid) -> Result<RecoveryData> {
    let recovery_point = {
      let recovery_points = self.recovery_points.read();
      recovery_points
        .get(&recovery_id)
        .cloned()
        .ok_or_else(|| CoreError::Recovery(format!("Recovery point {} not found", recovery_id)))?
    };

    Ok(recovery_point.data)
  }

  pub fn delete_recovery_point(&self, recovery_id: Uuid) -> Result<bool> {
    let recovery_point = {
      let mut recovery_points = self.recovery_points.write();
      recovery_points.remove(&recovery_id)
    };

    if let Some(recovery_point) = recovery_point {
      self.delete_recovery_file(&recovery_point)?;
      self.update_stats()?;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn list_recovery_points(&self) -> Vec<RecoveryPoint> {
    let recovery_points = self.recovery_points.read();
    let mut points: Vec<_> = recovery_points.values().cloned().collect();
    points.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    points
  }

  pub fn get_recovery_point(&self, recovery_id: Uuid) -> Option<RecoveryPoint> {
    let recovery_points = self.recovery_points.read();
    recovery_points.get(&recovery_id).cloned()
  }

  pub fn get_recovery_points_by_type(&self, _recovery_type: RecoveryType) -> Vec<RecoveryPoint> {
    let recovery_points = self.recovery_points.read();
    recovery_points
      .values()
      .filter(|rp| matches!(&rp.recovery_type, _recovery_type))
      .cloned()
      .collect()
  }

  pub fn cleanup_old_recovery_points(&self) -> Result<usize> {
    let config = self.config.read();
    let cutoff_duration =
      std::time::Duration::from_secs(config.cleanup_old_after_days as u64 * 24 * 60 * 60);
    let cutoff_time = std::time::SystemTime::now() - cutoff_duration;

    let old_recovery_points: Vec<Uuid> = {
      let recovery_points = self.recovery_points.read();
      recovery_points
        .values()
        .filter(|rp| rp.created_at < cutoff_time)
        .map(|rp| rp.id)
        .collect()
    };

    let mut removed_count = 0;
    for recovery_id in old_recovery_points {
      if self.delete_recovery_point(recovery_id)? {
        removed_count += 1;
      }
    }

    {
      let mut stats = self.stats.write();
      stats.cleanup_runs += 1;
    }

    Ok(removed_count)
  }

  pub fn get_recovery_stats(&self) -> RecoveryStats {
    let stats = self.stats.read();
    stats.clone()
  }

  pub fn start_auto_save(&self) -> Result<()> {
    let config = self.config.read();
    let interval_seconds = config.auto_save_interval_seconds;
    drop(config);

    let recovery_points = self.recovery_points.clone();
    let stats = self.stats.clone();

    let task = tokio::spawn(async move {
      let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

      loop {
        interval.tick().await;

        if let Err(e) = Self::perform_auto_save(&recovery_points, &stats).await {
          tracing::error!("Auto-save failed: {}", e);
        }
      }
    });

    {
      let mut cleanup_task = self.cleanup_task.write();
      *cleanup_task = Some(task);
    }

    Ok(())
  }

  pub fn stop_auto_save(&self) -> Result<()> {
    let mut cleanup_task = self.cleanup_task.write();
    if let Some(task) = cleanup_task.take() {
      task.abort();
    }
    Ok(())
  }

  pub fn export_recovery_point(&self, recovery_id: Uuid, output_path: &Path) -> Result<()> {
    let recovery_point = self
      .get_recovery_point(recovery_id)
      .ok_or_else(|| CoreError::Recovery(format!("Recovery point {} not found", recovery_id)))?;

    let export_data = serde_json::json!({
        "recovery_point": recovery_point,
        "exported_at": std::time::SystemTime::now()
    });

    std::fs::write(output_path, serde_json::to_string_pretty(&export_data)?)?;
    Ok(())
  }

  pub fn import_recovery_point(&self, import_path: &Path) -> Result<Uuid> {
    let import_data: serde_json::Value =
      serde_json::from_str(&std::fs::read_to_string(import_path)?)?;
    let recovery_point: RecoveryPoint =
      serde_json::from_value(import_data["recovery_point"].clone())?;
    let recovery_id = recovery_point.id;

    self.save_recovery_point(&recovery_point)?;

    {
      let mut recovery_points = self.recovery_points.write();
      recovery_points.insert(recovery_id, recovery_point);
    }

    self.update_stats()?;

    Ok(recovery_id)
  }

  fn save_recovery_point(&self, recovery_point: &RecoveryPoint) -> Result<()> {
    let config = self.config.read();
    let file_path = config
      .recovery_directory
      .join(format!("{}.recovery", recovery_point.id));
    drop(config);

    let recovery_data = serde_json::to_vec(recovery_point)?;

    {
      let config = self.config.read();
      if config.compression_enabled {
        let compressed_data = Self::compress_data(&recovery_data)?;
        std::fs::write(file_path.with_extension("recovery.gz"), compressed_data)?;
      } else {
        std::fs::write(file_path, recovery_data)?;
      }
    }

    Ok(())
  }

  fn load_existing_recovery_points(&self) -> Result<()> {
    let recovery_dir = {
      let config = self.config.read();
      config.recovery_directory.clone()
    };

    if !recovery_dir.exists() {
      return Ok(());
    }

    for entry in std::fs::read_dir(recovery_dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.extension().and_then(|s| s.to_str()) == Some("recovery")
        || path.extension().and_then(|s| s.to_str()) == Some("gz")
      {
        let data = if path.extension().and_then(|s| s.to_str()) == Some("gz") {
          let compressed_data = std::fs::read(&path)?;
          Self::decompress_data(&compressed_data)?
        } else {
          std::fs::read(&path)?
        };

        if let Ok(recovery_point) = serde_json::from_slice::<RecoveryPoint>(&data) {
          let mut recovery_points = self.recovery_points.write();
          recovery_points.insert(recovery_point.id, recovery_point);
        }
      }
    }

    Ok(())
  }

  fn delete_recovery_file(&self, recovery_point: &RecoveryPoint) -> Result<()> {
    let config = self.config.read();
    let file_path = config
      .recovery_directory
      .join(format!("{}.recovery", recovery_point.id));
    let compressed_path = file_path.with_extension("recovery.gz");
    drop(config);

    if file_path.exists() {
      std::fs::remove_file(file_path)?;
    }
    if compressed_path.exists() {
      std::fs::remove_file(compressed_path)?;
    }

    Ok(())
  }

  fn enforce_max_recovery_points(&self) -> Result<()> {
    let config = self.config.read();
    let max_points = config.max_recovery_points;
    drop(config);

    let points: Vec<_> = {
      let recovery_points = self.recovery_points.read();
      if recovery_points.len() <= max_points {
        return Ok(());
      }

      let mut points: Vec<_> = recovery_points.values().cloned().collect();
      points.sort_by(|a, b| a.created_at.cmp(&b.created_at));
      points
    };

    let points_to_remove = points.len() - max_points;
    let points_to_remove = &points[..points_to_remove];

    for recovery_point in points_to_remove {
      self.delete_recovery_point(recovery_point.id)?;
    }

    Ok(())
  }

  fn update_stats(&self) -> Result<()> {
    let recovery_points = self.recovery_points.read();
    let total_points = recovery_points.len();
    let mut total_size = 0u64;
    let mut oldest = None;
    let mut newest = None;
    let mut auto_saves = 0;
    let mut manual_saves = 0;
    let mut crash_recoveries = 0;

    for recovery_point in recovery_points.values() {
      let point_size = serde_json::to_vec(recovery_point)?.len() as u64;
      total_size += point_size;

      match oldest {
        None => oldest = Some(recovery_point.created_at),
        Some(current_oldest) if recovery_point.created_at < current_oldest => {
          oldest = Some(recovery_point.created_at);
        }
        _ => {}
      }

      match newest {
        None => newest = Some(recovery_point.created_at),
        Some(current_newest) if recovery_point.created_at > current_newest => {
          newest = Some(recovery_point.created_at);
        }
        _ => {}
      }

      match recovery_point.recovery_type {
        RecoveryType::AutoSave => auto_saves += 1,
        RecoveryType::ManualSave => manual_saves += 1,
        RecoveryType::CrashRecovery => crash_recoveries += 1,
        _ => {}
      }
    }

    let mut stats = self.stats.write();
    stats.total_recovery_points = total_points;
    stats.total_size_bytes = total_size;
    stats.oldest_recovery = oldest;
    stats.newest_recovery = newest;
    stats.auto_saves = auto_saves;
    stats.manual_saves = manual_saves;
    stats.crash_recoveries = crash_recoveries;

    Ok(())
  }

  async fn perform_auto_save(
    _recovery_points: &Arc<RwLock<HashMap<Uuid, RecoveryPoint>>>,
    stats: &Arc<RwLock<RecoveryStats>>,
  ) -> Result<()> {
    tracing::debug!("Performing auto-save");

    let current_time = std::time::SystemTime::now();
    let auto_save_threshold = std::time::Duration::from_secs(300);

    let needs_auto_save = {
      let stats = stats.read();
      if let Some(newest) = stats.newest_recovery {
        current_time.duration_since(newest).unwrap_or_default() > auto_save_threshold
      } else {
        true
      }
    };

    if needs_auto_save {
      tracing::info!("Creating auto-save recovery point");
    }

    Ok(())
  }

  fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::prelude::*;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
  }

  fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    use std::io::prelude::*;
    let mut decoder = flate2::read::GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
  }
}

impl Default for RecoveryConfig {
  fn default() -> Self {
    Self {
      auto_save_interval_seconds: 300,
      max_recovery_points: 50,
      recovery_directory: dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tiffiny")
        .join("recovery"),
      compression_enabled: true,
      encryption_enabled: false,
      cleanup_old_after_days: 30,
    }
  }
}

impl Default for RecoveryStats {
  fn default() -> Self {
    Self {
      total_recovery_points: 0,
      total_size_bytes: 0,
      oldest_recovery: None,
      newest_recovery: None,
      auto_saves: 0,
      manual_saves: 0,
      crash_recoveries: 0,
      cleanup_runs: 0,
    }
  }
}
