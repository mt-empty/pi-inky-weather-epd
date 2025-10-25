use anyhow::Error;
use serde::Deserialize;
use std::{fs, path::PathBuf};
use url::Url;

use crate::{errors::DashboardError, CONFIG};

/// Type alias for API-specific error checking function
pub type ErrorChecker = fn(&str) -> Result<(), DashboardError>;

/// Represents the outcome of a fetch operation
pub enum FetchOutcome<T> {
    /// Fresh data successfully fetched from API
    Fresh(T),
    /// Stale cached data used due to error
    Stale { data: T, error: DashboardError },
}

/// Shared fetcher for API data with caching fallback
pub struct Fetcher {
    cache_path: PathBuf,
}

impl Fetcher {
    pub fn new(cache_path: PathBuf) -> Self {
        Self { cache_path }
    }

    /// Load cached data from file
    fn load_cached<T: for<'de> Deserialize<'de>>(&self, file_path: &PathBuf) -> Result<T, Error> {
        let cached = fs::read_to_string(file_path).map_err(|e| {
            anyhow::anyhow!(
                "Weather data cache file not found at {:?}: {}. \
                 If this is your first time running, set 'disable_weather_api_requests = false' \
                 in the configuration so data can be cached.",
                file_path,
                e
            )
        })?;
        let data = serde_json::from_str(&cached).map_err(Error::msg)?;
        Ok(data)
    }

    /// Fallback to cached data when API fails
    fn fallback<T: for<'de> Deserialize<'de>>(
        &self,
        file_path: &PathBuf,
        dashboard_error: DashboardError,
    ) -> Result<FetchOutcome<T>, Error> {
        let data = self.load_cached(file_path)?;
        Ok(FetchOutcome::Stale {
            data,
            error: dashboard_error,
        })
    }

    /// Fetch data from API with caching fallback
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint URL
    /// * `cache_filename` - Name of cache file (e.g., "hourly_forecast.json")
    /// * `error_checker` - Optional function to check response for API-specific errors
    pub fn fetch_data<T>(
        &self,
        endpoint: Url,
        cache_filename: &str,
        error_checker: Option<ErrorChecker>,
    ) -> Result<FetchOutcome<T>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let file_path = self.cache_path.join(cache_filename);

        if !file_path.exists() {
            fs::create_dir_all(file_path.parent().unwrap())?;
        }

        if !CONFIG.debugging.disable_weather_api_requests {
            let client = reqwest::blocking::Client::new();
            let response = match client.get(endpoint).send() {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("API request failed: {e}");
                    return self.fallback(
                        &file_path,
                        DashboardError::NoInternet {
                            details: e.to_string(),
                        },
                    );
                }
            };

            let body = response.text().map_err(Error::msg)?;

            // Check for API-specific errors if checker provided
            if let Some(checker) = error_checker {
                if let Err(dashboard_error) = checker(&body) {
                    return self.fallback(&file_path, dashboard_error);
                }
            }

            fs::write(&file_path, &body)?;
            let data = serde_json::from_str(&body).map_err(Error::msg)?;
            Ok(FetchOutcome::Fresh(data))
        } else {
            Ok(FetchOutcome::Fresh(self.load_cached(&file_path)?))
        }
    }
}
