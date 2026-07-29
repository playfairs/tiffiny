use crate::AppEvent;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tiffiny_core::prelude::*;

#[derive(Debug, Clone)]
pub struct Command {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub category: CommandCategory,
  pub parameters: Vec<CommandParameter>,
  pub handler: CommandHandler,
  pub enabled: bool,
  pub shortcut: Option<String>,
  pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
  File,
  Edit,
  View,
  Project,
  Assets,
  Processing,
  Effects,
  Export,
  Settings,
  Help,
  Window,
}

#[derive(Debug, Clone)]
pub struct CommandParameter {
  pub name: String,
  pub parameter_type: ParameterType,
  pub required: bool,
  pub default_value: Option<serde_json::Value>,
  pub description: String,
  pub validation: Option<ParameterValidation>,
}

#[derive(Debug, Clone)]
pub enum ParameterType {
  String,
  Integer,
  Float,
  Boolean,
  FilePath,
  DirectoryPath,
  AssetReference,
  ProjectReference,
  PipelineReference,
  GraphReference,
  Choice(Vec<String>),
  Color,
  Point,
  Rectangle,
  Size,
}

#[derive(Debug, Clone)]
pub enum ParameterValidation {
  MinValue(f64),
  MaxValue(f64),
  Range(f64, f64),
  MinLength(usize),
  MaxLength(usize),
  Pattern(String),
  FileExists,
  DirectoryExists,
  Custom(Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>),
}

pub type CommandHandler =
  Arc<dyn Fn(HashMap<String, serde_json::Value>) -> Result<CommandResult> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct CommandResult {
  pub success: bool,
  pub message: String,
  pub data: Option<serde_json::Value>,
  pub events: Vec<AppEvent>,
}

pub struct CommandExecutor {
  commands: Arc<RwLock<HashMap<String, Command>>>,
  event_bus: Arc<crate::EventBus>,
  command_history: Arc<RwLock<Vec<CommandExecution>>>,
  max_history_size: usize,
}

#[derive(Debug, Clone)]
pub struct CommandExecution {
  pub command_id: String,
  pub parameters: HashMap<String, serde_json::Value>,
  pub result: CommandResult,
  pub timestamp: std::time::SystemTime,
  pub duration: std::time::Duration,
}

impl CommandExecutor {
  pub fn new(event_bus: Arc<crate::EventBus>) -> Self {
    Self {
      commands: Arc::new(RwLock::new(HashMap::new())),
      event_bus,
      command_history: Arc::new(RwLock::new(Vec::new())),
      max_history_size: 1000,
    }
  }

  pub fn register_command(&self, command: Command) -> Result<()> {
    let mut commands = self.commands.write();
    commands.insert(command.name.clone(), command);
    Ok(())
  }

  pub fn unregister_command(&self, command_name: &str) -> Result<bool> {
    let mut commands = self.commands.write();
    Ok(commands.remove(command_name).is_some())
  }

  pub async fn execute_command(
    &self,
    command_name: &str,
    parameters: HashMap<String, serde_json::Value>,
  ) -> Result<CommandResult> {
    let start_time = std::time::Instant::now();

    let command = {
      let commands = self.commands.read();
      commands
        .get(command_name)
        .cloned()
        .ok_or_else(|| CoreError::Task(format!("Command '{}' not found", command_name)))?
    };

    if !command.enabled {
      return Err(CoreError::Task(format!(
        "Command '{}' is disabled",
        command_name
      )));
    }

    self.validate_parameters(&command, &parameters)?;

    let result = (command.handler)(parameters.clone())?;

    for event in &result.events {
      self.event_bus.publish(event.clone()).await?;
    }

    let execution = CommandExecution {
      command_id: command_name.to_string(),
      parameters,
      result: result.clone(),
      timestamp: std::time::SystemTime::now(),
      duration: start_time.elapsed(),
    };

    {
      let mut history = self.command_history.write();
      history.push(execution);
      if history.len() > self.max_history_size {
        history.remove(0);
      }
    }

    Ok(result)
  }

  fn validate_parameters(
    &self,
    command: &Command,
    parameters: &HashMap<String, serde_json::Value>,
  ) -> Result<()> {
    for param in &command.parameters {
      if param.required && !parameters.contains_key(&param.name) {
        return Err(CoreError::Task(format!(
          "Required parameter '{}' is missing",
          param.name
        )));
      }

      if let Some(value) = parameters.get(&param.name) {
        self.validate_parameter_value(param, value)?;
      }
    }

    Ok(())
  }

  fn validate_parameter_value(
    &self,
    param: &CommandParameter,
    value: &serde_json::Value,
  ) -> Result<()> {
    match param.parameter_type {
      ParameterType::String => {
        if !value.is_string() {
          return Err(CoreError::Task(format!(
            "Parameter '{}' must be a string",
            param.name
          )));
        }
      }
      ParameterType::Integer => {
        if !value.is_i64() {
          return Err(CoreError::Task(format!(
            "Parameter '{}' must be an integer",
            param.name
          )));
        }
      }
      ParameterType::Float => {
        if !value.is_number() {
          return Err(CoreError::Task(format!(
            "Parameter '{}' must be a number",
            param.name
          )));
        }
      }
      ParameterType::Boolean => {
        if !value.is_boolean() {
          return Err(CoreError::Task(format!(
            "Parameter '{}' must be a boolean",
            param.name
          )));
        }
      }
      _ => {}
    }

    if let Some(validation) = &param.validation {
      self.apply_validation(param, value, validation)?;
    }

    Ok(())
  }

  fn apply_validation(
    &self,
    param: &CommandParameter,
    value: &serde_json::Value,
    validation: &ParameterValidation,
  ) -> Result<()> {
    match validation {
      ParameterValidation::MinValue(min) => {
        if let Some(num) = value.as_f64() {
          if num < *min {
            return Err(CoreError::Task(format!(
              "Parameter '{}' must be at least {}",
              param.name, min
            )));
          }
        }
      }
      ParameterValidation::MaxValue(max) => {
        if let Some(num) = value.as_f64() {
          if num > *max {
            return Err(CoreError::Task(format!(
              "Parameter '{}' must be at most {}",
              param.name, max
            )));
          }
        }
      }
      ParameterValidation::Range(min, max) => {
        if let Some(num) = value.as_f64() {
          if num < *min || num > *max {
            return Err(CoreError::Task(format!(
              "Parameter '{}' must be between {} and {}",
              param.name, min, max
            )));
          }
        }
      }
      ParameterValidation::MinLength(min) => {
        if let Some(s) = value.as_str() {
          if s.len() < *min {
            return Err(CoreError::Task(format!(
              "Parameter '{}' must be at least {} characters long",
              param.name, min
            )));
          }
        }
      }
      ParameterValidation::MaxLength(max) => {
        if let Some(s) = value.as_str() {
          if s.len() > *max {
            return Err(CoreError::Task(format!(
              "Parameter '{}' must be at most {} characters long",
              param.name, max
            )));
          }
        }
      }
      ParameterValidation::Pattern(pattern) => {
        if let Some(s) = value.as_str() {
          let regex = regex::Regex::new(pattern)
            .map_err(|e| CoreError::Task(format!("Invalid regex pattern: {}", e)))?;
          if !regex.is_match(s) {
            return Err(CoreError::Task(format!(
              "Parameter '{}' does not match required pattern",
              param.name
            )));
          }
        }
      }
      ParameterValidation::FileExists => {
        if let Some(path) = value.as_str() {
          if !std::path::Path::new(path).exists() {
            return Err(CoreError::Task(format!("File '{}' does not exist", path)));
          }
        }
      }
      ParameterValidation::DirectoryExists => {
        if let Some(path) = value.as_str() {
          if !std::path::Path::new(path).is_dir() {
            return Err(CoreError::Task(format!(
              "Directory '{}' does not exist",
              path
            )));
          }
        }
      }
      ParameterValidation::Custom(validator) => {
        if !validator(value) {
          return Err(CoreError::Task(format!(
            "Parameter '{}' failed custom validation",
            param.name
          )));
        }
      }
    }

    Ok(())
  }

  pub fn get_command(&self, command_name: &str) -> Option<Command> {
    let commands = self.commands.read();
    commands.get(command_name).cloned()
  }

  pub fn list_commands(&self) -> Vec<Command> {
    let commands = self.commands.read();
    commands.values().cloned().collect()
  }

  pub fn list_commands_by_category(&self, category: CommandCategory) -> Vec<Command> {
    let commands = self.commands.read();
    commands
      .values()
      .filter(|cmd| cmd.category == category)
      .cloned()
      .collect()
  }

  pub fn search_commands(&self, query: &str) -> Vec<Command> {
    let commands = self.commands.read();
    let query_lower = query.to_lowercase();

    commands
      .values()
      .filter(|cmd| {
        cmd.name.to_lowercase().contains(&query_lower)
          || cmd.description.to_lowercase().contains(&query_lower)
      })
      .cloned()
      .collect()
  }

  pub fn get_command_history(&self) -> Vec<CommandExecution> {
    let history = self.command_history.read();
    history.clone()
  }

  pub fn clear_command_history(&self) {
    let mut history = self.command_history.write();
    history.clear();
  }

  pub fn enable_command(&self, command_name: &str) -> Result<bool> {
    let mut commands = self.commands.write();
    if let Some(command) = commands.get_mut(command_name) {
      command.enabled = true;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn disable_command(&self, command_name: &str) -> Result<bool> {
    let mut commands = self.commands.read();
    if let Some(command) = commands.get_mut(command_name) {
      command.enabled = false;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn set_command_shortcut(&self, command_name: &str, shortcut: Option<String>) -> Result<bool> {
    let mut commands = self.commands.write();
    if let Some(command) = commands.get_mut(command_name) {
      command.shortcut = shortcut;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn get_command_by_shortcut(&self, shortcut: &str) -> Option<Command> {
    let commands = self.commands.read();
    commands
      .values()
      .find(|cmd| cmd.shortcut.as_ref().map_or(false, |s| s == shortcut))
      .cloned()
  }

  pub fn get_statistics(&self) -> CommandStatistics {
    let commands = self.commands.read();
    let history = self.command_history.read();

    let mut stats = CommandStatistics::default();
    stats.total_commands = commands.len();
    stats.enabled_commands = commands.values().filter(|cmd| cmd.enabled).count();
    stats.total_executions = history.len();

    for category in [
      CommandCategory::File,
      CommandCategory::Edit,
      CommandCategory::View,
      CommandCategory::Project,
      CommandCategory::Assets,
      CommandCategory::Processing,
      CommandCategory::Effects,
      CommandCategory::Export,
      CommandCategory::Settings,
      CommandCategory::Help,
      CommandCategory::Window,
    ] {
      let count = commands
        .values()
        .filter(|cmd| cmd.category == category)
        .count();
      stats.commands_by_category.insert(category, count);
    }

    stats
  }
}

#[derive(Debug, Clone, Default)]
pub struct CommandStatistics {
  pub total_commands: usize,
  pub enabled_commands: usize,
  pub total_executions: usize,
  pub commands_by_category: HashMap<CommandCategory, usize>,
}

pub fn create_new_project_command(event_bus: Arc<crate::EventBus>) -> Command {
  Command {
    id: Uuid::new_v4(),
    name: "new_project".to_string(),
    description: "Create a new project".to_string(),
    category: CommandCategory::Project,
    parameters: vec![
      CommandParameter {
        name: "name".to_string(),
        parameter_type: ParameterType::String,
        required: true,
        default_value: Some(serde_json::Value::String("Untitled Project".to_string())),
        description: "Project name".to_string(),
        validation: Some(ParameterValidation::MinLength(1)),
      },
      CommandParameter {
        name: "description".to_string(),
        parameter_type: ParameterType::String,
        required: false,
        default_value: None,
        description: "Project description".to_string(),
        validation: None,
      },
    ],
    handler: Arc::new(move |params| {
      let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Project");

      let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

      let event = AppEvent::NewProject;

      Ok(CommandResult {
        success: true,
        message: format!("Created new project: {}", name),
        data: Some(serde_json::json!({
            "name": name,
            "description": description
        })),
        events: vec![event],
      })
    }),
    enabled: true,
    shortcut: Some("Ctrl+N".to_string()),
    icon: Some("new".to_string()),
  }
}

pub fn open_project_command(event_bus: Arc<crate::EventBus>) -> Command {
  Command {
    id: Uuid::new_v4(),
    name: "open_project".to_string(),
    description: "Open an existing project".to_string(),
    category: CommandCategory::Project,
    parameters: vec![CommandParameter {
      name: "path".to_string(),
      parameter_type: ParameterType::FilePath,
      required: true,
      default_value: None,
      description: "Path to project file".to_string(),
      validation: Some(ParameterValidation::FileExists),
    }],
    handler: Arc::new(move |params| {
      let path = params
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::Task("Path parameter is required".to_string()))?;

      let event = AppEvent::OpenProject(path.to_string());

      Ok(CommandResult {
        success: true,
        message: format!("Opening project: {}", path),
        data: Some(serde_json::json!({
            "path": path
        })),
        events: vec![event],
      })
    }),
    enabled: true,
    shortcut: Some("Ctrl+O".to_string()),
    icon: Some("open".to_string()),
  }
}

pub fn quit_command(event_bus: Arc<crate::EventBus>) -> Command {
  Command {
    id: Uuid::new_v4(),
    name: "quit".to_string(),
    description: "Quit the application".to_string(),
    category: CommandCategory::File,
    parameters: vec![],
    handler: Arc::new(move |_params| {
      let event = AppEvent::Quit;

      Ok(CommandResult {
        success: true,
        message: "Quitting application".to_string(),
        data: None,
        events: vec![event],
      })
    }),
    enabled: true,
    shortcut: Some("Ctrl+Q".to_string()),
    icon: Some("quit".to_string()),
  }
}
