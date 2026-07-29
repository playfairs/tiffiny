use crate::{
  AppState,
  Bootstrap,
  CommandExecutor,
  EventBus,
  ShutdownHandler,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tiffiny_core::{
  GraphManager,
  JobManager,
  TaskManager,
  prelude::*,
};
use tiffiny_ui::UiManager;
use tiffiny_utils::platform::Platform;

pub struct TiffinyApp {
  state: Arc<RwLock<AppState>>,
  event_bus: Arc<EventBus>,
  command_executor: Arc<CommandExecutor>,
  ui_manager: Arc<UiManager>,
  task_manager: Arc<TaskManager>,
  job_manager: Arc<JobManager>,
  graph_manager: Arc<GraphManager>,
  platform: Arc<Platform>,
  shutdown_requested: Arc<RwLock<bool>>,
}

impl TiffinyApp {
  pub async fn new(bootstrap: Bootstrap) -> Result<Self> {
    let state = Arc::new(RwLock::new(AppState::new()));
    let event_bus = Arc::new(EventBus::new());
    let command_executor = Arc::new(CommandExecutor::new(event_bus.clone()));

    let task_manager = Arc::new(TaskManager::new());
    let job_manager = Arc::new(JobManager::new());
    let graph_manager = Arc::new(GraphManager::new());

    let platform = Arc::new(Platform::new().await?);

    let ui_manager =
      Arc::new(UiManager::new(state.clone(), event_bus.clone(), platform.clone()).await?);

    let shutdown_requested = Arc::new(RwLock::new(false));

    Ok(Self {
      state,
      event_bus,
      command_executor,
      ui_manager,
      task_manager,
      job_manager,
      graph_manager,
      platform,
      shutdown_requested,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    self.initialize().await?;

    while !*self.shutdown_requested.read() {
      self.process_events().await?;
      self.update_ui().await?;
      self.process_tasks().await?;

      tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }

    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    *self.shutdown_requested.write() = true;

    self.ui_manager.shutdown().await?;
    self
      .task_manager
      .cleanup_completed_executions(std::time::Duration::from_secs(0))?;
    self
      .job_manager
      .cleanup_completed_jobs(std::time::Duration::from_secs(0))?;

    Ok(())
  }

  async fn initialize(&self) -> Result<()> {
    self.ui_manager.initialize().await?;
    self.setup_event_handlers().await?;

    tracing::info!("Tiffiny application initialized successfully");
    Ok(())
  }

  async fn process_events(&self) -> Result<()> {
    let events = self.event_bus.get_pending_events().await?;

    for event in events {
      self.handle_event(event).await?;
    }

    Ok(())
  }

  async fn handle_event(&self, event: crate::AppEvent) -> Result<()> {
    match event {
      crate::AppEvent::Quit => {
        *self.shutdown_requested.write() = true;
      }
      crate::AppEvent::NewProject => {
        self.create_new_project().await?;
      }
      crate::AppEvent::OpenProject(path) => {
        self.open_project(path).await?;
      }
      crate::AppEvent::SaveProject => {
        self.save_current_project().await?;
      }
      crate::AppEvent::ImportFile(path) => {
        self.import_file(path).await?;
      }
      crate::AppEvent::ExportFile(path, format) => {
        self.export_file(path, format).await?;
      }
      crate::AppEvent::ExecutePipeline(pipeline_id) => {
        self.execute_pipeline(pipeline_id).await?;
      }
      crate::AppEvent::ExecuteGraph(graph_id) => {
        self.execute_graph(graph_id).await?;
      }
    }

    Ok(())
  }

  async fn update_ui(&self) -> Result<()> {
    self.ui_manager.update().await?;
    Ok(())
  }

  async fn process_tasks(&self) -> Result<()> {
    let state = self.state.read();

    for task_execution in state.get_running_tasks() {
      if let Some(result) = self.task_manager.get_task_result(task_execution.task_id) {
        self
          .handle_task_completion(task_execution.task_id, result)
          .await?;
      }
    }

    Ok(())
  }

  async fn handle_task_completion(
    &self,
    task_id: Uuid,
    result: tiffiny_core::task::TaskResult,
  ) -> Result<()> {
    if result.success {
      tracing::info!("Task {} completed successfully", task_id);

      let event = crate::AppEvent::TaskCompleted {
        task_id,
        result: result.result,
      };

      self.event_bus.publish(event).await?;
    } else {
      tracing::error!("Task {} failed: {:?}", task_id, result.error);

      let event = crate::AppEvent::TaskFailed {
        task_id,
        error: result.error.unwrap_or_default(),
      };

      self.event_bus.publish(event).await?;
    }

    Ok(())
  }

  async fn setup_event_handlers(&self) -> Result<()> {
    let event_bus = self.event_bus.clone();
    let shutdown_requested = self.shutdown_requested.clone();

    tokio::spawn(async move {
      let mut quit_rx = event_bus.subscribe_quit().await;

      while let Some(_) = quit_rx.recv().await {
        *shutdown_requested.write() = true;
        break;
      }
    });

    Ok(())
  }

  async fn create_new_project(&self) -> Result<()> {
    let project_id = self
      .state
      .create_project("Untitled Project".to_string())
      .await?;

    let event = crate::AppEvent::ProjectCreated { project_id };
    self.event_bus.publish(event).await?;

    tracing::info!("Created new project with ID: {}", project_id);
    Ok(())
  }

  async fn open_project(&self, path: String) -> Result<()> {
    let project_id = self.state.open_project(&path).await?;

    let event = crate::AppEvent::ProjectOpened { project_id };
    self.event_bus.publish(event).await?;

    tracing::info!("Opened project from: {}", path);
    Ok(())
  }

  async fn save_current_project(&self) -> Result<()> {
    let project_id = self.state.get_current_project_id().await?;

    if let Some(project_id) = project_id {
      self.state.save_project(project_id).await?;

      let event = crate::AppEvent::ProjectSaved { project_id };
      self.event_bus.publish(event).await?;

      tracing::info!("Saved project: {}", project_id);
    }

    Ok(())
  }

  async fn import_file(&self, path: String) -> Result<()> {
    let asset_id = self.state.import_file(&path).await?;

    let event = crate::AppEvent::FileImported { path, asset_id };
    self.event_bus.publish(event).await?;

    tracing::info!("Imported file: {}", path);
    Ok(())
  }

  async fn export_file(&self, path: String, format: String) -> Result<()> {
    let export_id = self.state.export_file(&path, &format).await?;

    let event = crate::AppEvent::FileExported {
      path,
      format,
      export_id,
    };
    self.event_bus.publish(event).await?;

    tracing::info!("Exported file: {} as {}", path, format);
    Ok(())
  }

  async fn execute_pipeline(&self, pipeline_id: Uuid) -> Result<()> {
    let execution_id = self.state.execute_pipeline(pipeline_id).await?;

    let event = crate::AppEvent::PipelineExecutionStarted {
      pipeline_id,
      execution_id,
    };
    self.event_bus.publish(event).await?;

    tracing::info!("Started pipeline execution: {}", pipeline_id);
    Ok(())
  }

  async fn execute_graph(&self, graph_id: Uuid) -> Result<()> {
    let execution_id = self.graph_manager.execute_graph(graph_id).await?;

    let event = crate::AppEvent::GraphExecutionStarted {
      graph_id,
      execution_id,
    };
    self.event_bus.publish(event).await?;

    tracing::info!("Started graph execution: {}", graph_id);
    Ok(())
  }
}
