#[allow(clippy::module_inception)]
pub mod app;
pub mod background_operations;
pub mod cli_operations;
pub mod commit_operations;
pub mod event_handlers;
pub mod input_handlers;
pub mod release_notes;
pub mod semantic_release_operations;
pub mod task_operations;

pub use app::App;
