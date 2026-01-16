use anyhow::Error;
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Duration};
use url::Url;

use crate::{errors::DashboardError, logger, CONFIG};

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
    client: reqwest::blocking::Client,
}

impl Fetcher {
    pub fn new(cache_path: PathBuf) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(2) // Connection pooling for reuse
            .user_agent(format!(
                "{}/{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                "Raspberry Pi Weather Dashboard"
            ))
            .build()
            .expect("Failed to build HTTP client");

        Self { cache_path, client }
    }

    /// Classify the error to provide better diagnostics
    fn classify_error(error: &reqwest::Error) -> String {
        if error.is_timeout() {
            "Request timeout - server took too long to respond".to_string()
        } else if error.is_connect() {
            "Connection failed - unable to reach server (check network/firewall)".to_string()
        } else if error.is_request() {
            format!("Request error - {}", error)
        } else if let Some(status) = error.status() {
            format!(
                "HTTP {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )
        } else {
            // Could be DNS resolution failure, TLS error, etc.
            let err_str = error.to_string();
            if err_str.contains("dns") || err_str.contains("resolve") {
                "DNS resolution failed - cannot find server address".to_string()
            } else if err_str.contains("certificate")
                || err_str.contains("tls")
                || err_str.contains("ssl")
            {
                "TLS/SSL error - certificate validation failed".to_string()
            } else {
                format!("Network error - {}", error)
            }
        }
    }

    /// Load cached data from file
    fn load_cached<T: for<'de> Deserialize<'de>>(&self, file_path: &PathBuf) -> Result<T, Error> {
        logger::detail("Attempting to use cached data");
        let cached = fs::read_to_string(file_path).map_err(|e| {
            anyhow::anyhow!(
                "No cached weather data available at {:?}. {}. \
                 This happens on first run or when 'disable_weather_api_requests` is set to true. \
                 The application needs at least one successful API call to create the cache.",
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

    /// Check if an error is retryable (transient network issues)
    fn is_error_retryable(error: &reqwest::Error) -> bool {
        error.is_timeout()
            || error.is_connect()
            || error.status().is_some_and(|s| s.is_server_error())
    }

    /// Process a successful API response
    fn process_successful_response<T: for<'de> Deserialize<'de>>(
        &self,
        body: String,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
    ) -> Result<FetchOutcome<T>, Error> {
        logger::debug(format!("Received API response: {} bytes", body.len()));

        // Check for API-specific errors if checker provided
        if let Some(checker) = error_checker {
            if let Err(dashboard_error) = checker(&body) {
                use crate::errors::Description;
                logger::warning(dashboard_error.long_description());
                return self.fallback(file_path, dashboard_error);
            }
        }

        fs::write(file_path, &body)?;
        logger::debug(format!("Cached response to: {}", file_path.display()));
        let data = serde_json::from_str(&body).map_err(Error::msg)?;
        Ok(FetchOutcome::Fresh(data))
    }

    /// Handle fetch errors with logging
    fn handle_fetch_error(&self, error: &reqwest::Error, attempt: u32, max_retries: u32) {
        let error_classification = Self::classify_error(error);

        if attempt == 0 {
            logger::warning(format!("API request failed: {}", error_classification));
        } else {
            logger::warning(format!(
                "Retry {}/{} failed: {}",
                attempt + 1,
                max_retries,
                error_classification
            ));
        }
    }

    /// Try to fetch data with retry logic
    fn try_fetch_with_retry<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &Url,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
    ) -> Result<FetchOutcome<T>, Error> {
        const MAX_RETRIES: u32 = 3;
        let retry_delays = [1, 2, 4]; // Exponential backoff in seconds

        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            // Wait before retry (skip first attempt)
            if attempt > 0 {
                let delay_secs = retry_delays[(attempt - 1) as usize];
                logger::detail(format!(
                    "Retrying in {} second{}... (attempt {}/{})",
                    delay_secs,
                    if delay_secs == 1 { "" } else { "s" },
                    attempt + 1,
                    MAX_RETRIES
                ));
                std::thread::sleep(Duration::from_secs(delay_secs));
            }

            match self.client.get(endpoint.clone()).send() {
                Ok(response) => {
                    if attempt > 0 {
                        logger::success(format!(
                            "Request succeeded on attempt {}/{}",
                            attempt + 1,
                            MAX_RETRIES
                        ));
                    }

                    let body = response.text().map_err(Error::msg)?;
                    return self.process_successful_response(body, file_path, error_checker);
                }
                Err(e) => {
                    self.handle_fetch_error(&e, attempt, MAX_RETRIES);

                    // Check if error is retryable
                    let is_retryable = Self::is_error_retryable(&e);

                    if !is_retryable || attempt == MAX_RETRIES - 1 {
                        // Non-retryable error or final attempt - fall back to cache
                        last_error = Some(e);
                        break;
                    }

                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted, use cached data
        if let Some(e) = last_error {
            logger::warning("All retry attempts exhausted. Attempting to use cached data");
            return self.fallback(
                file_path,
                DashboardError::NoInternet {
                    details: Self::classify_error(&e),
                },
            );
        }

        // Shouldn't reach here, but just in case
        unreachable!("Retry loop should either succeed or have last_error set");
    }

    /// Fetch data from API with caching fallback and retry logic
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint URL
    /// * `cache_filename` - Name of cache file (e.g., "hourly_forecast.json")
    /// * `error_checker` - Optional function to check response for API-specific errors
    ///
    /// # Retry Strategy
    /// Attempts up to 3 retries with exponential backoff (1s, 2s, 4s) on transient errors
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
            self.try_fetch_with_retry(&endpoint, &file_path, error_checker)
        } else {
            Ok(FetchOutcome::Fresh(self.load_cached(&file_path)?))
        }
    }
}
