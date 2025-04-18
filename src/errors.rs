use thiserror::Error;

use crate::dashboard::context::Context;
use crate::weather::icons::Icon;

#[derive(Error, Debug, Clone)]
pub enum DashboardError {
    #[error("No internet connection")]
    NoInternet { details: String },
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Incomplete data:")]
    IncompleteData { details: String },
    // TODO: to use this error, we need to call the update function before rendering the SVG
    #[error("Update failed: {0}")]
    UpdateFailed(String),
}

pub trait Description {
    fn short_description(&self) -> &'static str;
    fn long_description(&self) -> String;
}

impl Icon for DashboardError {
    fn get_icon_name(&self) -> String {
        match self {
            DashboardError::NoInternet { .. } => "code-orange.svg".to_string(),
            DashboardError::ApiError(_) => "code-red.svg".to_string(),
            DashboardError::IncompleteData { .. } => "code-yellow.svg".to_string(),
            DashboardError::UpdateFailed(_) => "code-green.svg".to_string(),
        }
    }
}

impl Description for DashboardError {
    fn short_description(&self) -> &'static str {
        match self {
            DashboardError::NoInternet { .. } => "API unreachable → Stale Data",
            DashboardError::ApiError(_) => "API error ➜ Stale Data",
            DashboardError::IncompleteData { .. } => "Incomplete Data",
            DashboardError::UpdateFailed(_) => "Update failed",
        }
    }

    fn long_description(&self) -> String {
        match self {
            DashboardError::NoInternet { details } => {
                format!(
                    "The application is unable to reach the API server. Details: {}",
                    details
                )
            }
            DashboardError::ApiError(msg) => {
                format!("The API returned an error. Details: {}", msg)
            }
            DashboardError::IncompleteData { details } => {
                format!("Received Incomplete data. Details: {}", details)
            }
            DashboardError::UpdateFailed(msg) => {
                format!("The application failed to update. Details: {}", msg)
            }
        }
    }
}

pub fn handle_errors<E: Icon + Description + std::error::Error>(context: &mut Context, error: E) {
    context.warning_message = error.short_description().to_string();
    context.warning_icon = error.get_icon_path().to_string();
    context.warning_visibility = "visible".to_string();
    eprintln!("Error: {}", error.long_description());
}
