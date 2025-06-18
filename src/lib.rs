pub mod app;
pub mod config;
pub mod git;
pub mod services;
pub mod types;
pub mod ui;
pub mod utils;

// Re-export commonly used items
pub use crate::types::*;
pub use crate::git::repository; 