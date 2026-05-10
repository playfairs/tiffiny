use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BatchConverter {
    pub id: String,
    pub name: String,
    pub files: Arc<RwLock<Vec<BatchFile>>>,
    pub status: Arc<RwLock<BatchStatus>>,
    pub event_sender: mpsc::UnboundedSender<BatchEvent>,
    pub event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<BatchEvent>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchStatus {
    Idle,
    Preparing,
    Processing,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum BatchEvent {
    BatchStarted,
    BatchProgress(f32),
    FileStarted(usize),
    FileCompleted(usize, BatchFileResult),
    FileFailed(usize, String),
    BatchCompleted(BatchResult),
    BatchFailed(String),
    BatchCancelled,
}

#[derive(Debug, Clone)]
pub struct BatchFile {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub status: Arc<RwLock<FileStatus>>,
    pub progress: Arc<RwLock<f32>>,
    pub output_path: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone)]
pub struct BatchFileResult {
    pub file_id: String,
    pub success: bool,
    pub input_path: String,
    pub output_path: String,
    pub file_size_before: u64,
    pub file_size_after: u64,
    pub processing_time: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_id: String,
    pub success: bool,
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub total_processing_time: std::time::Duration,
    pub total_bytes_processed: u64,
    pub file_results: Vec<BatchFileResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_concurrent_files: usize,
    pub continue_on_error: bool,
    pub skip_existing: bool,
    pub create_subdirectories: bool,
    pub preserve_structure: bool,
    pub output_directory: String,
    pub file_pattern: Option<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone)]
pub struct BatchStats {
    pub total_files: usize,
    pub pending_files: usize,
    pub processing_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub average_processing_time: std::time::Duration,
    pub throughput: f64,
}

impl BatchConverter {
    pub fn new(id: String, name: String) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            id,
            name,
            files: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(BatchStatus::Idle))),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    pub fn from_config(id: String, name: String, config: BatchConfig) -> Self {
        let mut converter = Self::new(id, name);
        converter.add_files_from_directory(&config.output_directory, &config).unwrap();
        converter
    }

    pub async fn process_batch(&self, config: BatchConfig) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let _ = self.event_sender.send(BatchEvent::BatchStarted);
        let start_time = std::time::Instant::now();

Set status to preparing
        let mut status = self.status.write();
        *status = BatchStatus::Preparing;

Prepare files for processing
        self.prepare_files(&config).await?;

Set status to processing
        *status = BatchStatus::Processing;

        let result = self.process_files_concurrent(&config).await;

        let processing_time = start_time.elapsed();

        match result {
            Ok(file_results) => {
                *status = BatchStatus::Completed;

                let total_files = self.files.read().len();
                let completed_files = file_results.iter().filter(|r| r.success).count();
                let failed_files = file_results.iter().filter(|r| !r.success).count();
                let skipped_files = file_results.iter().filter(|r| r.output_path.is_empty()).count();
                let total_bytes_processed = file_results.iter().map(|r| r.file_size_before).sum();

                let batch_result = BatchResult {
                    batch_id: self.id.clone(),
                    success: completed_files > 0,
                    total_files,
                    completed_files,
                    failed_files,
                    skipped_files,
                    total_processing_time: processing_time,
                    total_bytes_processed,
                    file_results,
                    error_message: None,
                };

                let _ = self.event_sender.send(BatchEvent::BatchCompleted(batch_result.clone()));
                Ok(batch_result)
            },
            Err(e) => {
                let error_msg = format!("Batch processing failed: {}", e);
                
                *status = BatchStatus::Failed(error_msg.clone());
                
                let _ = self.event_sender.send(BatchEvent::BatchFailed(error_msg.clone()));
                
                Ok(BatchResult {
                    batch_id: self.id.clone(),
                    success: false,
                    total_files: self.files.read().len(),
                    completed_files: 0,
                    failed_files: 0,
                    skipped_files: 0,
                    total_processing_time: processing_time,
                    total_bytes_processed: 0,
                    file_results: Vec::new(),
                    error_message: Some(error_msg),
                })
            },
        }
    }

    async fn prepare_files(&self, config: &BatchConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut files = self.files.write();
        
        for (index, file) in files.iter_mut().enumerate() {
            if file.output_path.is_none() {
                let output_path = self.generate_output_path(&file.path, config, index)?;
                file.output_path = Some(output_path);
            }

            let mut status = file.status.write();
            *status = FileStatus::Pending;

            let mut progress = file.progress.write();
            *progress = 0.0;
        }

        Ok(())
    }

    async fn process_files_concurrent(&self, config: &BatchConfig) -> Result<Vec<BatchFileResult>, Box<dyn std::error::Error>> {
        let files = self.files.read();
        let mut results = Vec::new();
        let mut tasks = Vec::new();

        for (index, file) in files.iter().enumerate() {
            let file_clone = file.clone();
            let config_clone = config.clone();
            let event_sender = self.event_sender.clone();

            let task = tokio::spawn(async move {
                let _ = event_sender.send(BatchEvent::FileStarted(index));
                
                let mut status = file_clone.status.write();
                *status = FileStatus::Processing;

                let start_time = std::time::Instant::now();

                let result = Self::process_single_file(&file_clone, &config_clone).await;

                let processing_time = start_time.elapsed();

                match result {
                    Ok(output_path) => {
                        *status = FileStatus::Completed;
                        
                        let mut progress = file_clone.progress.write();
                        *progress = 100.0;

                        let file_result = BatchFileResult {
                            file_id: file_clone.id.clone(),
                            success: true,
                            input_path: file_clone.path.clone(),
                            output_path,
                            file_size_before: file_clone.size,
                            file_size_after: file_clone.size,
                            processing_time,
                            error_message: None,
                        };

                        let _ = event_sender.send(BatchEvent::FileCompleted(index, file_result.clone()));
                        Ok(file_result)
                    },
                    Err(e) => {
                        let error_msg = format!("File processing failed: {}", e);
                        
                        *status = FileStatus::Failed(error_msg.clone());
                        
                        let file_result = BatchFileResult {
                            file_id: file_clone.id.clone(),
                            success: false,
                            input_path: file_clone.path.clone(),
                            output_path: file_clone.output_path.clone().unwrap_or_default(),
                            file_size_before: file_clone.size,
                            file_size_after: 0,
                            processing_time,
                            error_message: Some(error_msg),
                        };

                        let _ = event_sender.send(BatchEvent::FileFailed(index, error_msg.clone()));
                        Ok(file_result)
                    },
                }
            });

            tasks.push(task);

            if tasks.len() >= config.max_concurrent_files {
                let mut completed = 0;
                while completed < tasks.len() / 2 {
                    for (i, task) in tasks.iter().enumerate() {
                        if task.is_finished() {
                            if let Ok(result) = task.try_get() {
                                results.push(result);
                                completed += 1;
                            }
                        }
                    }
                    if completed == 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }

                tasks.retain(|task| !task.is_finished());
            }
        }

        for task in tasks {
            if let Ok(result) = task.await {
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn process_single_file(file: &BatchFile, config: &BatchConfig) -> Result<String, Box<dyn std::error::Error>> {
        let processing_time = std::time::Duration::from_millis(100 + (file.size % 1000) as u64);
        tokio::time::sleep(processing_time).await;

        for i in 0..=100 {
            let mut progress = file.progress.write();
            *progress = i as f32;
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(file.output_path.clone().unwrap_or_default())
    }

    fn generate_output_path(&self, input_path: &str, config: &BatchConfig, index: usize) -> Result<String, Box<dyn std::error::Error>> {
        let path_obj = std::path::Path::new(input_path);
        let file_name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let output_dir = if config.preserve_structure {
            let relative_path = path_obj.strip_prefix(&config.output_directory).unwrap_or(path_obj);
            let output_dir = std::path::Path::new(&config.output_directory).join(relative_path.parent().unwrap_or(std::path::Path::new("")));
            output_dir.to_string_lossy().to_string()
        } else {
            config.output_directory.clone()
        };

        Ok(format!("{}/{}", output_dir, file_name))
    }

    pub fn add_file(&self, file: BatchFile) {
        let mut files = self.files.write();
        files.push(file);
    }

    pub fn add_files(&self, files: Vec<BatchFile>) {
        let mut batch_files = self.files.write();
        batch_files.extend(files);
    }

    pub fn add_files_from_directory(&self, directory: &str, config: &BatchConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        if config.recursive {
            for entry in walkdir::WalkDir::new(directory) {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            if let Some(file) = self.create_batch_file(&entry.path().to_string_lossy(), config)? {
                                files.push(file);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(BatchEvent::BatchFailed(format!("Error reading directory: {}", e)));
                    },
                }
            }
        } else {
            for entry in std::fs::read_dir(directory)? {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(file) = self.create_batch_file(&path.to_string_lossy(), config)? {
                                files.push(file);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = self.event_sender.send(BatchEvent::BatchFailed(format!("Error reading directory: {}", e)));
                    },
                }
            }
        }

        self.add_files(files);
        Ok(())
    }

    fn create_batch_file(&self, path: &str, config: &BatchConfig) -> Result<Option<BatchFile>, Box<dyn std::error::Error>> {
        let path_obj = std::path::Path::new(path);
        
        if let Some(ref pattern) = config.file_pattern {
            let file_name = path_obj
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if !self.matches_pattern(file_name, pattern) {
                return Ok(None);
            }
        }

        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        let file_name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Some(BatchFile {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string(),
            name: file_name,
            size: file_size,
            status: Arc::new(RwLock::new(FileStatus::Pending)),
            progress: Arc::new(RwLock::new(0.0)),
            output_path: None,
            metadata: std::collections::HashMap::new(),
        }))
    }

    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        file_name.contains(pattern)
    }

    pub fn remove_file(&self, file_id: &str) -> bool {
        let mut files = self.files.write();
        if let Some(index) = files.iter().position(|f| f.id == file_id) {
            files.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_files(&self) {
        let mut files = self.files.write();
        files.clear();
    }

    pub fn get_file(&self, file_id: &str) -> Option<BatchFile> {
        let files = self.files.read();
        files.iter().find(|f| f.id == file_id).cloned()
    }

    pub fn get_files(&self) -> Vec<BatchFile> {
        self.files.read().clone()
    }

    pub fn get_status(&self) -> BatchStatus {
        self.status.read().clone()
    }

    pub fn pause(&self) {
        let mut status = self.status.write();
        *status = BatchStatus::Paused;
    }

    pub fn resume(&self) {
        let mut status = self.status.write();
        *status = BatchStatus::Processing;
    }

    pub fn cancel(&self) {
        let mut status = self.status.write();
        *status = BatchStatus::Cancelled;
        
        let _ = self.event_sender.send(BatchEvent::BatchCancelled);
    }

    pub fn reset(&self) {
        let mut status = self.status.write();
        *status = BatchStatus::Idle;
        
        let files = self.files.read();
        for file in files.iter() {
            let mut file_status = file.status.write();
            *file_status = FileStatus::Pending;
            
            let mut progress = file.progress.write();
            *progress = 0.0;
        }
    }

    pub fn get_progress(&self) -> f32 {
        let files = self.files.read();
        if files.is_empty() {
            return 0.0;
        }

        let total_progress: f32 = files.iter().map(|f| *f.progress.read()).sum();
        total_progress / files.len() as f32
    }

    pub fn get_stats(&self) -> BatchStats {
        let files = self.files.read();
        let total_files = files.len();
        let pending_files = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Pending)).count();
        let processing_files = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Processing)).count();
        let completed_files = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Completed)).count();
        let failed_files = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Failed(_))).count();
        let skipped_files = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Skipped)).count();
        let total_bytes = files.iter().map(|f| f.size).sum();
        let processed_bytes = files.iter().filter(|f| matches!(*f.status.read(), FileStatus::Completed)).map(|f| f.size).sum();

        BatchStats {
            total_files,
            pending_files,
            processing_files,
            completed_files,
            failed_files,
            skipped_files,
            total_bytes,
            processed_bytes,
            average_processing_time: std::time::Duration::from_secs(0),
            throughput: 0.0,
        }
    }

    pub async fn get_events(&mut self) -> Vec<BatchEvent> {
        let mut receiver = self.event_receiver.write();
        if let Some(ref mut rx) = *receiver {
            let mut events = Vec::new();
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
            events
        } else {
            Vec::new()
        }
    }

    pub fn clone_converter(&self) -> BatchConverter {
        let mut new_converter = Self::new(
            uuid::Uuid::new_v4().to_string(),
            format!("{} Clone", self.name),
        );

        let files = self.files.read();
        new_converter.add_files(files.clone());

        new_converter
    }
}

impl Default for BatchConverter {
    fn default() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "Batch Converter".to_string(),
        )
    }
}

impl Default for BatchStatus {
    fn default() -> Self {
        BatchStatus::Idle
    }
}

impl Default for BatchEvent {
    fn default() -> Self {
        BatchEvent::BatchStarted
    }
}

impl Default for BatchFile {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: String::new(),
            name: String::new(),
            size: 0,
            status: Arc::new(RwLock::new(FileStatus::Pending)),
            progress: Arc::new(RwLock::new(0.0)),
            output_path: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for FileStatus {
    fn default() -> Self {
        FileStatus::Pending
    }
}

impl Default for BatchFileResult {
    fn default() -> Self {
        Self {
            file_id: String::new(),
            success: false,
            input_path: String::new(),
            output_path: String::new(),
            file_size_before: 0,
            file_size_after: 0,
            processing_time: std::time::Duration::from_millis(0),
            error_message: None,
        }
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self {
            batch_id: String::new(),
            success: false,
            total_files: 0,
            completed_files: 0,
            failed_files: 0,
            skipped_files: 0,
            total_processing_time: std::time::Duration::from_millis(0),
            total_bytes_processed: 0,
            file_results: Vec::new(),
            error_message: None,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: 4,
            continue_on_error: true,
            skip_existing: false,
            create_subdirectories: true,
            preserve_structure: false,
            output_directory: String::new(),
            file_pattern: None,
            recursive: false,
        }
    }
}

impl Default for BatchStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            pending_files: 0,
            processing_files: 0,
            completed_files: 0,
            failed_files: 0,
            skipped_files: 0,
            total_bytes: 0,
            processed_bytes: 0,
            average_processing_time: std::time::Duration::from_secs(0),
            throughput: 0.0,
        }
    }
}
