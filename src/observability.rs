use std::io;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
    EnvFilter,
};

use crate::error::Result;

/// Initialize the observability system with structured logging
/// Console output is minimal (only errors), everything else goes to files
pub fn init_observability(debug_enabled: bool, verbose_enabled: bool) -> Result<()> {
    // Determine the file log level based on flags
    let file_level = if debug_enabled {
        LevelFilter::DEBUG
    } else if verbose_enabled {
        LevelFilter::INFO
    } else {
        LevelFilter::INFO // Always log INFO+ to files
    };

    // Console only shows errors and critical warnings
    let console_level = LevelFilter::ERROR;

    // Create environment filter for files (comprehensive logging)
    let file_env_filter = EnvFilter::builder()
        .with_default_directive(file_level.into())
        .from_env_lossy()
        .add_directive("semantic_release_tui=debug".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("tokio=info".parse().unwrap());

    // Create environment filter for console (minimal logging)
    let console_env_filter = EnvFilter::builder()
        .with_default_directive(console_level.into())
        .from_env_lossy()
        .add_directive("semantic_release_tui=error".parse().unwrap());

    // Minimal console output - only errors and critical issues
    let console_layer = fmt::layer()
        .with_timer(ChronoUtc::rfc_3339())
        .with_target(false) // Hide target for cleaner console
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true)
        .with_writer(io::stderr)
        .with_filter(console_env_filter);

    // Comprehensive file output for debugging and monitoring
    let file_appender = tracing_appender::rolling::daily("logs", "semantic-release-tui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let file_layer = fmt::layer()
        .with_timer(ChronoUtc::rfc_3339())
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .json()
        .with_writer(non_blocking)
        .with_filter(file_env_filter);

    // Initialize the subscriber with multiple layers
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    // Only log startup to file, not console
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        debug = debug_enabled,
        verbose = verbose_enabled,
        "🚀 Semantic Release TUI initialized"
    );

    // Store the guard to prevent the non-blocking writer from being dropped
    std::mem::forget(_guard);

    Ok(())
}

/// Initialize development-friendly logging with tree output to files
/// Console stays clean for development too
pub fn init_development_observability() -> Result<()> {
    // Development mode: comprehensive file logging, minimal console
    let console_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env_lossy()
        .add_directive("semantic_release_tui=error".parse().unwrap());

    // Minimal console layer for development
    let console_layer = fmt::layer()
        .with_timer(ChronoUtc::rfc_3339())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true)
        .with_writer(io::stderr)
        .with_filter(console_env_filter);

    // Development file logging with tree structure
    let file_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy()
        .add_directive("semantic_release_tui=trace".parse().unwrap());

    let dev_file_appender = tracing_appender::rolling::daily("logs", "semantic-release-tui-dev.log");
    let (dev_non_blocking, _dev_guard) = tracing_appender::non_blocking(dev_file_appender);

    let dev_file_layer = tracing_tree::HierarchicalLayer::new(2)
        .with_targets(true)
        .with_bracketed_fields(true)
        .with_indent_lines(true)
        .with_writer(dev_non_blocking)
        .with_filter(file_env_filter);

    // Use both console (minimal) and file (comprehensive tree)
    tracing_subscriber::registry()
        .with(console_layer)
        .with(dev_file_layer)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        mode = "development",
        "🛠️ Semantic Release TUI (Development Mode) initialized"
    );

    // Store the guard to prevent the non-blocking writer from being dropped
    std::mem::forget(_dev_guard);

    Ok(())
}

/// Log a user-friendly message to console (bypasses normal filtering)
pub fn log_user_message(message: &str) {
    eprintln!("{}", message);
}

/// Log an error to console (these will show up)
pub fn log_error_to_console(message: &str) {
    tracing::error!("{}", message);
}

/// Log a warning to console only if it's critical
pub fn log_critical_warning(message: &str) {
    tracing::error!("WARNING: {}", message); // Use error level to ensure it shows
}

/// Macro for creating spans with operation timing
#[macro_export]
macro_rules! timed_operation {
    ($name:expr, $body:expr) => {{
        let span = tracing::info_span!("operation", name = $name);
        let _enter = span.enter();
        let start = std::time::Instant::now();
        
        let result = $body;
        
        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
        
        result
    }};
}

/// Macro for creating async spans with operation timing
#[macro_export]
macro_rules! timed_async_operation {
    ($name:expr, $body:expr) => {{
        async move {
            let span = tracing::info_span!("async_operation", name = $name);
            let start = std::time::Instant::now();
            
            let result = tracing::Instrument::instrument($body, span).await;
            
            let duration = start.elapsed();
            tracing::info!(
                duration_ms = duration.as_millis(),
                operation = $name,
                "Async operation completed"
            );
            
            result
        }
    }};
}

/// Log performance metrics for monitoring (file only)
pub fn log_performance_metrics(operation: &str, duration_ms: u128, success: bool) {
    tracing::info!(
        operation = operation,
        duration_ms = duration_ms,
        success = success,
        "Performance metric recorded"
    );
}

/// Log error with context for better debugging (file only)
pub fn log_error_with_context(
    component: &str,
    operation: &str,
    error: &dyn std::error::Error,
    context: Option<&str>,
) {
    tracing::error!(
        component = component,
        operation = operation,
        error = %error,
        context = context,
        "Operation failed"
    );
}

/// Log successful operation with context (file only)
pub fn log_success_with_context(
    component: &str,
    operation: &str,
    context: Option<&str>,
) {
    tracing::info!(
        component = component,
        operation = operation,
        context = context,
        "Operation succeeded"
    );
}

/// Log warning with context (file only)
pub fn log_warning_with_context(
    component: &str,
    operation: &str,
    warning: &str,
    context: Option<&str>,
) {
    tracing::warn!(
        component = component,
        operation = operation,
        warning = warning,
        context = context,
        "Operation warning"
    );
}

/// Create a debug span for function entry/exit tracking
#[macro_export]
macro_rules! debug_span {
    ($func_name:expr) => {
        tracing::debug_span!("function", name = $func_name)
    };
    ($func_name:expr, $($field:tt)*) => {
        tracing::debug_span!("function", name = $func_name, $($field)*)
    };
}

/// Create an info span for high-level operations
#[macro_export]
macro_rules! info_span {
    ($operation:expr) => {
        tracing::info_span!("operation", name = $operation)
    };
    ($operation:expr, $($field:tt)*) => {
        tracing::info_span!("operation", name = $operation, $($field)*)
    };
} 