pub mod buffers;
pub mod cache;
pub mod graph;
pub mod jobs;
pub mod memory;
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

use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct CoreError {
    pub message: String,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CoreError {}

pub type Result<T> = std::result::Result<T, CoreError>;

impl From<String> for CoreError {
    fn from(message: String) -> Self {
        CoreError { message }
    }
}

impl From<&str> for CoreError {
    fn from(message: &str) -> Self {
        CoreError { message: message.to_string() }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        CoreError { message: err.to_string() }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        CoreError { message: err.to_string() }
    }
}

#[derive(Debug, Clone)]
pub enum CoreError {
    Task(String),
    Memory(String),
    Recovery(String),
    Session(String),
    Io(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Task(msg) => write!(f, "Task: {}", msg),
            CoreError::Memory(msg) => write!(f, "Memory: {}", msg),
            CoreError::Recovery(msg) => write!(f, "Recovery: {}", msg),
            CoreError::Session(msg) => write!(f, "Session: {}", msg),
            CoreError::Io(msg) => write!(f, "IO: {}", msg),
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

pub mod prelude {
    pub use super::{CoreError, Result};
    pub use uuid::Uuid;
    pub use std::sync::Arc;
    pub use parking_lot::RwLock;
}
