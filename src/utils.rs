use std::fs::OpenOptions;
use std::io::Write;

/// Log a message to the debug.log file with a component prefix
pub fn log_debug(component: &str, message: &str) {
    if let Ok(mut debug_file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
    {
        writeln!(debug_file, "[{}] {}", component, message).ok();
    }
}

/// Log an error to the debug.log file with a component prefix
pub fn log_error(component: &str, error: &dyn std::fmt::Display) {
    log_debug(component, &format!("❌ {}", error));
}

/// Log a success message to the debug.log file with a component prefix
pub fn log_success(component: &str, message: &str) {
    log_debug(component, &format!("✅ {}", message));
}
