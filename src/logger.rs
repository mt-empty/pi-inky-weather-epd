//! Simple, professional logging utility for the weather dashboard
//!
//! Provides structured logging with visual indicators and clean formatting.

use std::fmt::Display;
use std::io::IsTerminal;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// File logging
// ---------------------------------------------------------------------------

/// Global log file handle.  Set once by [`init_file_log`] at application
/// startup; every subsequent log call mirrors its output here as plain text
/// (no ANSI codes).
static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

/// Open (and truncate) `pi-inky-weather-epd` in the current working directory and store
/// the handle for the lifetime of the process.
///
/// Must be called before the first log message.  Calling it more than once is
/// safe – the second call is a no-op.  On failure a warning is printed to
/// stdout and the application continues without file logging.
pub fn init_file_log() {
    match std::fs::File::create("pi-inky-weather-epd") {
        Ok(file) => {
            // OnceLock::set fails silently if already initialized (second call).
            let _ = LOG_FILE.set(Mutex::new(file));
        }
        Err(e) => {
            // Cannot use warning() here – it would recurse. Print directly.
            println!("⚠ WARNING Could not open log file pi-inky-weather-epd: {e}");
        }
    }
}

/// Write a plain-text line to the log file.  Any I/O error is silently
/// swallowed to avoid infinite recursion (we cannot call the logger from
/// within the logger).
fn write_to_file(text: &str) {
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let _ = writeln!(file, "{text}");
        }
    }
}

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
    write_to_file(&format!("{} {} {}", level.symbol(), level.label(), message));
}

/// Log a section header (major step in the process)
pub fn section(title: impl Display) {
    println!(
        "\n{}{}▶ {title}{}",
        ansi("\x1b[34m"),
        ansi("\x1b[1m"),
        ansi("\x1b[0m")
    );
    write_to_file(&format!("\n▶ {title}"));
}

/// Log a subsection (minor step within a major step)
pub fn subsection(title: impl Display) {
    println!("  {}→{} {title}", ansi("\x1b[36m"), ansi("\x1b[0m"));
    write_to_file(&format!("  → {title}"));
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
    if crate::CONFIG.dev.enable_debug_logs {
        log_message(LogLevel::Debug, message);
    }
}

/// Log a configuration group header
pub fn config_group(title: impl Display) {
    println!("  {}[{}]{}", ansi("\x1b[1m"), title, ansi("\x1b[0m"));
    write_to_file(&format!("  [{title}]"));
}

/// Log a key-value pair (useful for configuration or data display)
pub fn kvp(key: impl Display, value: impl Display) {
    println!("  {}•{} {key}: {value}", ansi("\x1b[90m"), ansi("\x1b[0m"));
    write_to_file(&format!("  • {key}: {value}"));
}

/// Log raw data detail (like API responses)
pub fn detail(message: impl Display) {
    println!("    {}{}{}", ansi("\x1b[90m"), message, ansi("\x1b[0m"));
    write_to_file(&format!("    {message}"));
}

/// Log a separator line
#[allow(dead_code)]
pub fn separator() {
    println!("{}{}{}", ansi("\x1b[90m"), "─".repeat(60), ansi("\x1b[0m"));
    write_to_file(&"─".repeat(60));
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
    write_to_file(&format!("\n{app_name} v{version}"));
    write_to_file(&"=".repeat(60));
}

/// Log the end of the application
pub fn app_end() {
    println!(
        "\n{}{}{}",
        ansi("\x1b[90m"),
        "=".repeat(60),
        ansi("\x1b[0m")
    );
    write_to_file(&format!("\n{}", "=".repeat(60)));
}
