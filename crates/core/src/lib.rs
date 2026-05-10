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

pub mod prelude {
    pub use super::{CoreError, Result};
    pub use uuid::Uuid;
    pub use std::sync::Arc;
    pub use parking_lot::RwLock;
}
