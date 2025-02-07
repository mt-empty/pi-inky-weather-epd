use thiserror::Error;

use crate::{context::Context, Icon};

#[derive(Error, Debug, Clone)]
pub enum DashboardError {
    #[error("No internet connection")]
    NoInternet { details: String },
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Application crashed: {0}")]
    ApplicationCrashed(String),
}

pub trait Description {
    fn short_description(&self) -> &'static str;
    fn long_description(&self) -> String;
}

impl Icon for DashboardError {
    fn get_icon_name(&self) -> String {
        match self {
            DashboardError::NoInternet { .. } => "code-yellow.svg".to_string(),
            DashboardError::ApiError(_) => "code-orange.svg".to_string(),
            DashboardError::ApplicationCrashed(_) => "code-red.svg".to_string(),
        }
    }
}

impl Description for DashboardError {
    fn short_description(&self) -> &'static str {
        match self {
            DashboardError::NoInternet { .. } => "API server unreachable",
            DashboardError::ApiError(_) => "API error",
            DashboardError::ApplicationCrashed(_) => "Application crashed",
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
            DashboardError::ApplicationCrashed(msg) => {
                format!("The application has crashed. Details: {}", msg)
            }
        }
    }
}

pub fn handle_errors<E: Icon + Description + std::error::Error>(context: &mut Context, error: E) {
    context.warning_message = error.short_description().to_string();
    context.warning_icon = error.get_icon_name().to_string();
    context.warning_visibility = "visible".to_string();
    eprintln!("Error: {}", error.long_description());
}
