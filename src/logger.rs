//! Simple, professional logging utility for the weather dashboard
//!
//! Provides structured logging with visual indicators and clean formatting.

use std::fmt::Display;
use std::io::IsTerminal;
use std::sync::OnceLock;

/// Color policy: auto, always, or never
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColourPolicy {
    Auto,
    Always,
    Never,
}

/// Determine the colour policy based on environment variables and TTY status
fn colour_policy() -> &'static ColourPolicy {
    static POLICY: OnceLock<ColourPolicy> = OnceLock::new();
    POLICY.get_or_init(|| {
        // Check for explicit colour override
        if let Ok(val) = std::env::var("APP_COLOR") {
            return match val.to_lowercase().as_str() {
                "always" | "1" | "true" => ColourPolicy::Always,
                "never" | "0" | "false" => ColourPolicy::Never,
                _ => ColourPolicy::Auto,
            };
        }

        // Check FORCE_COLOR (common convention)
        if std::env::var("FORCE_COLOR").is_ok() {
            return ColourPolicy::Always;
        }

        // Honor NO_COLOR (https://no-color.org/)
        if std::env::var("NO_COLOR").is_ok() {
            return ColourPolicy::Never;
        }

        ColourPolicy::Auto
    })
}

/// Check if we should use colours for stdout
fn use_colours_stdout() -> bool {
    match colour_policy() {
        ColourPolicy::Always => true,
        ColourPolicy::Never => false,
        ColourPolicy::Auto => std::io::stdout().is_terminal(),
    }
}

/// Check if we should use colours for stderr
#[allow(dead_code)]
fn use_colours_stderr() -> bool {
    match colour_policy() {
        ColourPolicy::Always => true,
        ColourPolicy::Never => false,
        ColourPolicy::Auto => std::io::stderr().is_terminal(),
    }
}

/// Helper to conditionally return ANSI code or empty string for stdout
fn ansi(code: &'static str) -> &'static str {
    if use_colours_stdout() {
        code
    } else {
        ""
    }
}

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
    fn colour_code(&self) -> &'static str {
        match self {
            LogLevel::Info => ansi("\x1b[36m"),    // Cyan
            LogLevel::Success => ansi("\x1b[32m"), // Green
            LogLevel::Warning => ansi("\x1b[33m"), // Yellow
            LogLevel::Error => ansi("\x1b[31m"),   // Red
            LogLevel::Debug => ansi("\x1b[90m"),   // Gray
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
}

/// Log a message with the specified level
fn log_message(level: LogLevel, message: impl Display) {
    println!(
        "{}{} {}{} {}",
        level.colour_code(),
        level.symbol(),
        level.label(),
        ansi("\x1b[0m"),
        message
    );
}

/// Log a section header (major step in the process)
pub fn section(title: impl Display) {
    println!(
        "\n{}{}▶ {title}{}",
        ansi("\x1b[34m"),
        ansi("\x1b[1m"),
        ansi("\x1b[0m")
    );
}

/// Log a subsection (minor step within a major step)
pub fn subsection(title: impl Display) {
    println!("  {}→{} {title}", ansi("\x1b[36m"), ansi("\x1b[0m"));
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
    println!("  {}[{}]{}", ansi("\x1b[1m"), title, ansi("\x1b[0m"));
}

/// Log a key-value pair (useful for configuration or data display)
pub fn kvp(key: impl Display, value: impl Display) {
    println!("  {}•{} {key}: {value}", ansi("\x1b[90m"), ansi("\x1b[0m"));
}

/// Log raw data detail (like API responses)
pub fn detail(message: impl Display) {
    println!("    {}{}{}", ansi("\x1b[90m"), message, ansi("\x1b[0m"));
}

/// Log a separator line
#[allow(dead_code)]
pub fn separator() {
    println!("{}{}{}", ansi("\x1b[90m"), "─".repeat(60), ansi("\x1b[0m"));
}

/// Log the start of the application
pub fn app_start(app_name: &str, version: &str) {
    println!(
        "\n{}{} v{}{}",
        ansi("\x1b[1m"),
        app_name,
        version,
        ansi("\x1b[0m")
    );
    println!("{}{}{}", ansi("\x1b[90m"), "=".repeat(60), ansi("\x1b[0m"));
}

/// Log the end of the application
pub fn app_end() {
    println!(
        "\n{}{}{}",
        ansi("\x1b[90m"),
        "=".repeat(60),
        ansi("\x1b[0m")
    );
}
