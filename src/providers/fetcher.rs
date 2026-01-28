// TODO: this files violates the single responsibility principle and should be refactored
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

    /// Classify reqwest error to appropriate DashboardError using idiomatic error inspection
    fn classify_error(error: &reqwest::Error) -> DashboardError {
        logger::detail(format!("Raw error details: {:?}", error));

        // Check for HTTP status codes first (4xx/5xx)
        if let Some(status) = error.status() {
            let details = format!(
                "HTTP {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            );

            // 4xx and 5xx are API errors (server returned error response)
            return DashboardError::ApiError { details };
        }

        if error.is_timeout() {
            return DashboardError::NetworkError {
                details: "Request timeout - server took too long to respond".to_string(),
            };
        }

        if error.is_connect() {
            return DashboardError::NetworkError {
                details: error.to_string(),
            };
        }

        if error.is_request() {
            return DashboardError::ApiError {
                details: format!("Request error: {}", error),
            };
        }

        // Catch-all for other errors
        DashboardError::NetworkError {
            details: format!("Network error: {}", error),
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

    /// Check if an error is retryable (transient network issues, rate limits, server errors)
    fn is_error_retryable(error: &reqwest::Error) -> bool {
        // Network connectivity issues are retryable
        if error.is_timeout() || error.is_connect() {
            return true;
        }

        // Check HTTP status codes
        if let Some(status) = error.status() {
            // 429 Too Many Requests - always retryable with backoff
            if status.as_u16() == 429 {
                return true;
            }
            // 5xx server errors are retryable
            if status.is_server_error() {
                return true;
            }
        }

        false
    }

    /// Parse Retry-After header value (seconds as integer or HTTP-date)
    fn parse_retry_after(value: &str) -> Option<u64> {
        // Try parsing as integer seconds first
        if let Ok(seconds) = value.trim().parse::<u64>() {
            return Some(seconds);
        }

        // Try parsing as HTTP-date (RFC 7231)
        if let Ok(date) = chrono::DateTime::parse_from_rfc2822(value) {
            let now = chrono::Utc::now();
            let duration = date.signed_duration_since(now);
            if duration.num_seconds() > 0 {
                return Some(duration.num_seconds() as u64);
            }
        }

        None
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

    /// Handle fetch errors with logging (classify_error logs raw details internally)
    fn handle_fetch_error(&self, error: &reqwest::Error, attempt: u32, max_retries: u32) {
        let dashboard_error = Self::classify_error(error);
        use crate::errors::Description;

        if attempt == 0 {
            logger::warning(format!(
                "API request failed: {}",
                dashboard_error.short_description()
            ));
        } else {
            logger::warning(format!(
                "Retry {}/{} failed: {}",
                attempt + 1,
                max_retries,
                dashboard_error.short_description()
            ));
        }
    }

    /// Try to fetch data with retry logic and rate limit handling
    fn try_fetch_with_retry<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &Url,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
    ) -> Result<FetchOutcome<T>, Error> {
        const MAX_RETRIES: u32 = 5;
        const MAX_RETRY_AFTER_SECS: u64 = 60; // Cap Retry-After at 60 seconds
        let retry_delays = [1, 2, 4, 16, 32]; // Exponential backoff in seconds

        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 0..MAX_RETRIES {
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
                    // Check for 429 Too Many Requests with Retry-After header
                    if response.status().as_u16() == 429 {
                        let retry_after = response
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(Self::parse_retry_after)
                            .unwrap_or(retry_delays.get((attempt) as usize).copied().unwrap_or(32));

                        // Cap retry delay at maximum
                        if retry_after > MAX_RETRY_AFTER_SECS {
                            // return error immediately if Retry-After is unreasonably high
                            last_error = Some(Box::new(std::io::Error::other(
                                format!(
                                    "Rate limited with Retry-After of {} seconds exceeding max of {} seconds",
                                    retry_after, MAX_RETRY_AFTER_SECS
                                ),
                            )));
                            break;
                        }

                        logger::warning(format!(
                            "Rate limited (HTTP 429). Retrying after {} seconds",
                            retry_after
                        ));

                        // Don't count as last attempt yet - respect rate limit
                        if attempt < MAX_RETRIES - 1 {
                            std::thread::sleep(Duration::from_secs(retry_after));
                            continue;
                        } else {
                            // Final attempt - fall back to cache
                            last_error = Some(Box::new(std::io::Error::other(
                                "Rate limited after max retries",
                            )));
                            break;
                        }
                    }

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
                        last_error = Some(Box::new(e));
                        break;
                    }

                    last_error = Some(Box::new(e));
                }
            }
        }

        // All retries exhausted, use cached data
        let dashboard_error = if let Some(e) = last_error {
            logger::warning("All retry attempts exhausted. Attempting to use cached data");

            // Try to downcast to reqwest::Error for proper classification
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                Self::classify_error(reqwest_err)
            } else {
                DashboardError::NetworkError {
                    details: e.to_string(),
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "Failed to fetch data and no error information available"
            ));
        };

        self.fallback(file_path, dashboard_error)
    }

    /// Fetch data from API with caching fallback and retry logic
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint URL
    /// * `cache_filename` - Name of cache file (e.g., "hourly_forecast.json")
    /// * `error_checker` - Optional function to check response for API-specific errors
    ///
    /// # Retry Strategy
    /// - Attempts up to 5 retries with exponential backoff (1s, 2s, 4s, 16s, 32s)
    /// - Respects HTTP 429 rate limit responses with Retry-After header
    /// - Retries on: timeouts, connection failures, 5xx errors, 429 rate limits
    /// - Falls back to cached data if all retries fail
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

        match CONFIG.debugging.disable_weather_api_requests {
            true => {
                let cached = self.load_cached(&file_path)?;
                Ok(FetchOutcome::Fresh(cached))
            }
            false => self.try_fetch_with_retry(&endpoint, &file_path, error_checker),
        }
    }
}
