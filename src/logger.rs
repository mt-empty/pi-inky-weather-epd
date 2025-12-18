//! Simple, professional logging utility for the weather dashboard
//!
//! Provides structured logging with visual indicators and clean formatting.

use std::fmt::Display;

/// Log levels with visual indicators
#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    /// Get the colour code for this log level (ANSI colours)
    fn colour_code(&self) -> &str {
        match self {
            LogLevel::Info => "\x1b[36m",    // Cyan
            LogLevel::Success => "\x1b[32m", // Green
            LogLevel::Warning => "\x1b[33m", // Yellow
            LogLevel::Error => "\x1b[31m",   // Red
            LogLevel::Debug => "\x1b[90m",   // Gray
        }
    }

    /// Get the symbol for this log level
    fn symbol(&self) -> &str {
        match self {
            LogLevel::Info => "ℹ",
            LogLevel::Success => "✓",
            LogLevel::Warning => "⚠",
            LogLevel::Error => "✗",
            LogLevel::Debug => "•",
        }
    }

    /// Get the label for this log level
    fn label(&self) -> &str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Success => "SUCCESS",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }

    /// Reset colour code
    const RESET: &'static str = "\x1b[0m";
}

/// Log a message with the specified level
fn log_message(level: LogLevel, message: impl Display) {
    println!(
        "{}{} {}{} {}",
        level.colour_code(),
        level.symbol(),
        level.label(),
        LogLevel::RESET,
        message
    );
}

/// Log a section header (major step in the process)
pub fn section(title: impl Display) {
    println!("\n\x1b[34m\x1b[1m▶ {title}\x1b[0m");
}

/// Log a subsection (minor step within a major step)
pub fn subsection(title: impl Display) {
    println!("  \x1b[36m→\x1b[0m {title}");
}

/// Log an info message
#[allow(dead_code)]
pub fn info(message: impl Display) {
    log_message(LogLevel::Info, message);
}

/// Log a success message
pub fn success(message: impl Display) {
    log_message(LogLevel::Success, message);
}

/// Log a warning message
pub fn warning(message: impl Display) {
    log_message(LogLevel::Warning, message);
}

/// Log an error message
pub fn error(message: impl Display) {
    log_message(LogLevel::Error, message);
}

/// Log a debug message
#[allow(dead_code)]
pub fn debug(message: impl Display) {
    if crate::CONFIG.debugging.enable_debug_logs {
        log_message(LogLevel::Debug, message);
    }
}

/// Log a configuration group header
pub fn config_group(title: impl Display) {
    println!("  \x1b[1m[{}]\x1b[0m", title);
}

/// Log a key-value pair (useful for configuration or data display)
pub fn kvp(key: impl Display, value: impl Display) {
    let bullet = "\x1b[90m•\x1b[0m";
    println!("  {bullet} {key}: {value}");
}

/// Log raw data detail (like API responses)
pub fn detail(message: impl Display) {
    println!("    \x1b[90m{}\x1b[0m", message);
}

/// Log a separator line
#[allow(dead_code)]
pub fn separator() {
    println!("\x1b[90m{}\x1b[0m", "─".repeat(60));
}

/// Log the start of the application
pub fn app_start(app_name: &str, version: &str) {
    println!("\n\x1b[1m{} v{}\x1b[0m", app_name, version);
    println!("\x1b[90m{}\x1b[0m", "=".repeat(60));
}

/// Log the end of the application
pub fn app_end() {
    println!("\n\x1b[90m{}\x1b[0m", "=".repeat(60));
}
