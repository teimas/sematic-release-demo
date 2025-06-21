//! CLI Output Formatters
//! 
//! This module provides different output formats for CLI command results.

#[cfg(feature = "new-domains")]
use std::fmt::Display;
#[cfg(feature = "new-domains")]
use clap::ValueEnum;
#[cfg(feature = "new-domains")]
use serde::Serialize;

/// Available output formats
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text format
    Text,
}

/// Format data for output
#[cfg(feature = "new-domains")]
pub fn format_output<T>(data: &T, format: OutputFormat) -> Result<String, FormatError>
where
    T: Serialize + Display,
{
    match format {
        OutputFormat::Table => Ok(format_as_table(data)),
        OutputFormat::Json => format_as_json(data),
        OutputFormat::Yaml => format_as_yaml(data),
        OutputFormat::Text => Ok(data.to_string()),
    }
}

/// Format data as a human-readable table
#[cfg(feature = "new-domains")]
fn format_as_table<T: Display>(data: &T) -> String {
    // For now, just use the Display implementation
    // In a real implementation, you might use a crate like `tabled` for proper table formatting
    data.to_string()
}

/// Format data as JSON
#[cfg(feature = "new-domains")]
fn format_as_json<T: Serialize>(data: &T) -> Result<String, FormatError> {
    serde_json::to_string_pretty(data)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))
}

/// Format data as YAML
#[cfg(feature = "new-domains")]
fn format_as_yaml<T: Serialize>(data: &T) -> Result<String, FormatError> {
    serde_yaml::to_string(data)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))
}

/// Formatting errors
#[cfg(feature = "new-domains")]
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Unsupported format for data type")]
    UnsupportedFormat,
}
