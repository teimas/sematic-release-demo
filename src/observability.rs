/// Log a user-friendly message to console (bypasses normal filtering)
pub fn log_user_message(message: &str) {
    eprintln!("{}", message);
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
        tracing::info!(duration_ms = duration.as_millis(), "Operation completed");

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
