use anyhow::Error;
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Duration};
use url::Url;

use crate::{errors::DashboardError, logger, CONFIG};

/// Type alias for API-specific error checking function
pub type ErrorChecker = fn(&str) -> Result<(), DashboardError>;

/// Configuration for retry behavior
///
/// **Internal API** - Not intended for external use, may change without notice.
/// Used for testing retry logic with custom configurations.
pub struct RetryConfig<'a> {
    pub max_retries: usize,
    pub retry_delays: &'a [Duration],
    pub max_retry_after_secs: Duration,
}

impl<'a> RetryConfig<'a> {
    pub fn new(
        max_retries: usize,
        retry_delays: &'static [Duration],
        max_retry_after_secs: Duration,
    ) -> Self {
        assert!(
            max_retries <= retry_delays.len(),
            "max_retries cannot exceed the length of retry_delays array"
        );
        Self {
            max_retries,
            retry_delays,
            max_retry_after_secs,
        }
    }
}

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
    pub fn classify_error(error: &reqwest::Error) -> DashboardError {
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
    pub fn is_error_retryable(error: &reqwest::Error) -> bool {
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
    pub fn parse_retry_after(value: &str) -> Option<Duration> {
        // Try parsing as integer seconds first
        if let Ok(seconds) = value.trim().parse::<u64>() {
            return Some(Duration::from_secs(seconds));
        }

        // Try parsing as HTTP-date (RFC 7231)
        if let Ok(date) = chrono::DateTime::parse_from_rfc2822(value) {
            let now = chrono::Utc::now();
            let duration = date.signed_duration_since(now);
            if duration.num_seconds() > 0 {
                return Some(Duration::from_secs(duration.num_seconds() as u64));
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
    fn handle_fetch_error(&self, error: &reqwest::Error, attempt: usize, max_retries: usize) {
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

    /// Handle HTTP 429 rate limiting with Retry-After header
    ///
    /// Returns Ok(retry_delay) if should retry, Err if should abort
    fn handle_rate_limit_response(
        response: &reqwest::blocking::Response,
        attempt: usize,
        config: &RetryConfig,
    ) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(Self::parse_retry_after)
            .unwrap_or(
                config
                    .retry_delays
                    .get(attempt)
                    .copied()
                    .unwrap_or(Duration::from_secs(32)),
            );

        // Cap retry delay at maximum
        if retry_after > config.max_retry_after_secs {
            return Err(Box::new(std::io::Error::other(format!(
                "Rate limited with Retry-After of {} seconds exceeding max of {} seconds",
                retry_after.as_secs(),
                config.max_retry_after_secs.as_secs()
            ))));
        }

        logger::warning(format!(
            "Rate limited (HTTP 429). Retrying after {} seconds",
            retry_after.as_secs()
        ));

        // Check if we have more retries available
        if attempt >= config.max_retries - 1 {
            return Err(Box::new(std::io::Error::other(
                "Rate limited after max retries",
            )));
        }

        Ok(retry_after)
    }

    /// Execute a single fetch attempt and process the response
    fn execute_fetch_attempt<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &Url,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
        attempt: usize,
        config: &RetryConfig,
    ) -> Result<FetchOutcome<T>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(endpoint.as_str()).send()?;

        // Check for 429 Too Many Requests with Retry-After header
        if response.status().as_u16() == 429 {
            let retry_after = Self::handle_rate_limit_response(&response, attempt, config)?;
            std::thread::sleep(retry_after);
            // Return error to trigger retry
            return Err(Box::new(std::io::Error::other("Rate limited, retrying")));
        }

        if attempt > 0 {
            logger::success(format!(
                "Request succeeded on attempt {}/{}",
                attempt + 1,
                config.max_retries
            ));
        }

        let body = response.text()?;
        match self.process_successful_response(body, file_path, error_checker) {
            Ok(outcome) => Ok(outcome),
            Err(e) => Err(Box::new(std::io::Error::other(e.to_string()))),
        }
    }

    /// Try to fetch data with retry logic and rate limit handling
    ///
    /// **⚠️ Internal API** - Not intended for external use, may change without notice.
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint URL
    /// * `file_path` - Path to cache file
    /// * `error_checker` - Optional function to check for API-specific errors
    /// * `config` - Retry configuration (max retries, delays, rate limit cap)
    pub fn try_fetch_with_retry<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &Url,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
        config: &RetryConfig,
    ) -> Result<FetchOutcome<T>, Error> {
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 0..config.max_retries {
            // Apply delay before retry attempts (not on first attempt)
            if attempt > 0 {
                let delay_secs = config
                    .retry_delays
                    .get(attempt - 1)
                    .copied()
                    .unwrap_or(Duration::from_secs(32));
                logger::detail(format!(
                    "Retrying in {} second{}... (attempt {}/{})",
                    delay_secs.as_secs(),
                    if delay_secs.as_secs() == 1 { "" } else { "s" },
                    attempt + 1,
                    config.max_retries
                ));
                std::thread::sleep(delay_secs);
            }

            // Execute fetch attempt
            match self.execute_fetch_attempt(endpoint, file_path, error_checker, attempt, config) {
                Ok(outcome) => return Ok(outcome),
                Err(e) => {
                    // Try to downcast to reqwest::Error for proper error handling
                    if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                        self.handle_fetch_error(reqwest_err, attempt, config.max_retries);

                        // Check if error is retryable
                        let is_retryable = Self::is_error_retryable(reqwest_err);

                        if !is_retryable || attempt == config.max_retries - 1 {
                            // Non-retryable error or final attempt - fall back to cache
                            last_error = Some(e);
                            // No point of further retries, since the error is not retryable
                            break;
                        }
                    }

                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted, use cached data
        let dashboard_error = self.classify_final_error(last_error)?;
        self.fallback(file_path, dashboard_error)
    }

    /// Classify the final error after all retries are exhausted
    fn classify_final_error(
        &self,
        last_error: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Result<DashboardError, Error> {
        if let Some(e) = last_error {
            logger::warning("All retry attempts exhausted. Attempting to use cached data");

            // Try to downcast to reqwest::Error for proper classification
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                Ok(Self::classify_error(reqwest_err))
            } else {
                Ok(DashboardError::NetworkError {
                    details: e.to_string(),
                })
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to fetch data and no error information available"
            ))
        }
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
        const MAX_RETRIES: usize = 5;
        const RETRY_DELAYS: &[Duration] = &[
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(16),
            Duration::from_secs(32),
        ]; // Exponential backoff in seconds
        const MAX_RETRY_AFTER_SECS: Duration = Duration::from_secs(60); // Cap Retry-After at 60 seconds

        let config = RetryConfig {
            max_retries: MAX_RETRIES,
            retry_delays: RETRY_DELAYS,
            max_retry_after_secs: MAX_RETRY_AFTER_SECS,
        };

        let file_path = self.cache_path.join(cache_filename);

        if !file_path.exists() {
            fs::create_dir_all(file_path.parent().unwrap())?;
        }

        match CONFIG.debugging.disable_weather_api_requests {
            true => {
                let cached = self.load_cached(&file_path)?;
                Ok(FetchOutcome::Fresh(cached))
            }
            false => self.try_fetch_with_retry(&endpoint, &file_path, error_checker, &config),
        }
    }
}
