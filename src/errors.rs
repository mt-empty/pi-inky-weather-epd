use std::fmt;
use strum_macros::Display;
use thiserror::Error;

use crate::weather::icons::Icon;

/// Priority levels for dashboard diagnostics (higher value = higher priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticPriority {
    Low = 1,    // IncompleteData - yellow
    Medium = 2, // NoInternet - orange
    High = 3,   // ApiError - red
}

#[derive(Error, Debug, Clone)]
pub enum DashboardError {
    #[error("No internet connection")]
    NoInternet { details: String },
    #[error("API error")]
    ApiError { details: String },
    #[error("Incomplete data")]
    IncompleteData { details: String },
    // TODO: to use this error, we need to call the update function before rendering the SVG
    // #[error("Update failed")]
    // UpdateFailed { details: String },
}

#[derive(Debug, Display)]
pub enum DashboardErrorIconName {
    #[strum(to_string = "code-orange.svg")]
    NoInternet,
    #[strum(to_string = "code-red.svg")]
    ApiError,
    #[strum(to_string = "code-yellow.svg")]
    IncompleteData,
    // #[strum(to_string = "code-green.svg")]
    // UpdateFailed,
}

pub trait Description {
    fn short_description(&self) -> &'static str;
    fn long_description(&self) -> String;
}

impl Icon for DashboardError {
    fn get_icon_name(&self) -> String {
        match self {
            DashboardError::NoInternet { .. } => DashboardErrorIconName::NoInternet,
            DashboardError::ApiError { .. } => DashboardErrorIconName::ApiError,
            DashboardError::IncompleteData { .. } => DashboardErrorIconName::IncompleteData,
            // DashboardError::UpdateFailed { .. } => DashboardErrorIconName::UpdateFailed,
        }
        .to_string()
    }
}

impl DashboardError {
    /// Returns the priority of this error for display purposes.
    /// Higher priority errors take precedence when multiple errors occur.
    pub fn priority(&self) -> DiagnosticPriority {
        match self {
            DashboardError::ApiError { .. } => DiagnosticPriority::High,
            DashboardError::NoInternet { .. } => DiagnosticPriority::Medium,
            DashboardError::IncompleteData { .. } => DiagnosticPriority::Low,
        }
    }
}

impl Description for DashboardError {
    fn short_description(&self) -> &'static str {
        match self {
            DashboardError::NoInternet { .. } => "API unreachable -> Stale Data",
            DashboardError::ApiError { .. } => "API error -> Stale Data",
            DashboardError::IncompleteData { .. } => "Incomplete Data",
        }
    }

    fn long_description(&self) -> String {
        match self {
            DashboardError::NoInternet { details } => {
                format!("The application is unable to reach the API server. Details: {details}")
            }
            DashboardError::ApiError { details } => {
                format!("The API returned an error. Details: {details}")
            }
            DashboardError::IncompleteData { details } => {
                format!("Received Incomplete data. Details: {details}")
            } // DashboardError::UpdateFailed { details } => {
              //     format!("The application failed to update. Details: {details}")
              // }
        }
    }
}

#[derive(Debug, Error)]
pub enum GeohashError {
    InvalidCoordinateRange(f64, f64),
    InvalidLength(usize),
}

impl fmt::Display for GeohashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeohashError::InvalidCoordinateRange(x, y) => {
                write!(f, "invalid coordinate range: lat={x}, lon={y}")
            }
            GeohashError::InvalidLength(len) => write!(
                f,
                "Invalid length specified: {len}. Accepted values are between 1 and 12, inclusive"
            ),
        }
    }
}
