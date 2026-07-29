pub mod buffers;
pub mod cache;
pub mod graph;
pub mod jobs;
pub mod memory;
pub mod pipeline;
pub mod project;
pub mod recovery;
pub mod session;
pub mod task;

pub use buffers::*;
pub use cache::*;
pub use graph::*;
pub use jobs::*;
pub use memory::*;
pub use project::*;
pub use recovery::*;
pub use session::*;
pub use task::*;

#[derive(Debug, Clone)]
pub enum CoreError {
  Task(String),
  Memory(String),
  Recovery(String),
  Session(String),
  Io(String),
  Project(String),
}

impl std::fmt::Display for CoreError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CoreError::Task(msg) => write!(f, "Task: {}", msg),
      CoreError::Memory(msg) => write!(f, "Memory: {}", msg),
      CoreError::Recovery(msg) => write!(f, "Recovery: {}", msg),
      CoreError::Session(msg) => write!(f, "Session: {}", msg),
      CoreError::Io(msg) => write!(f, "IO: {}", msg),
      CoreError::Project(msg) => write!(f, "Project: {}", msg),
    }
  }
}

impl std::error::Error for CoreError {}

impl From<String> for CoreError {
  fn from(message: String) -> Self {
    CoreError::Task(message)
  }
}

impl From<&str> for CoreError {
  fn from(message: &str) -> Self {
    CoreError::Task(message.to_string())
  }
}

impl From<std::io::Error> for CoreError {
  fn from(err: std::io::Error) -> Self {
    CoreError::Io(err.to_string())
  }
}

impl From<serde_json::Error> for CoreError {
  fn from(err: serde_json::Error) -> Self {
    CoreError::Io(err.to_string())
  }
}

pub type Result<T> = std::result::Result<T, CoreError>;

pub mod prelude {
  pub use super::{
    CoreError,
    Result,
  };
  pub use parking_lot::RwLock;
  pub use std::sync::Arc;
  pub use uuid::Uuid;
}
