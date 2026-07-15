use anyhow::Error;
use serde::Deserialize;
use std::{fmt, fs, path::PathBuf, time::Duration};
use url::Url;

use crate::configs::settings::DashboardSettings;
use crate::{errors::DashboardError, logger};

/// Type alias for API-specific error checking function
pub type ErrorChecker = fn(&str) -> Result<(), DashboardError>;

/// Wraps a `DashboardError` produced by an `ErrorChecker` (a body-level API error, as
/// opposed to a `reqwest::Error`) to signal to `try_fetch_with_retry` that this is a
/// transient error worth retrying with backoff, rather than an immediate fallback.
///
/// Retryability is decided by HTTP status in `process_successful_response`: a 4xx status
/// (e.g. Open-Meteo returning 400 for an invalid parameter) indicates a problem with the
/// request itself that will fail identically on every retry, so those fall back to cached
/// data immediately instead of being wrapped here. Anything else (5xx, or a 2xx response
/// whose body still signals an error, e.g. Open-Meteo's "The service is overloaded") is
/// assumed to be transient and gets wrapped so it flows through the same retry path as
/// network errors.
#[derive(Debug)]
struct TransientApiError(DashboardError);

impl fmt::Display for TransientApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for TransientApiError {}

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
    fn parse_retry_after(value: &str) -> Option<Duration> {
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
        status: reqwest::StatusCode,
        body: String,
        file_path: &PathBuf,
        error_checker: Option<ErrorChecker>,
    ) -> Result<FetchOutcome<T>, Error> {
        logger::debug(format!("Received API response: {} bytes", body.len()));

        // Check for API-specific errors if checker provided.
        if let Some(checker) = error_checker {
            if let Err(dashboard_error) = checker(&body) {
                use crate::errors::Description;
                logger::detail(format!(
                    "Raw error details: {}",
                    dashboard_error.long_description()
                ));

                // A 4xx status (other than 429, which is handled separately before this
                // point) means the request itself is the problem - e.g. Open-Meteo
                // rejecting an invalid parameter. That will fail identically on every
                // retry, so log it and fall back to cached data immediately instead of
                // wasting the retry budget.
                if status.is_client_error() {
                    logger::warning(format!(
                        "API request failed: {}",
                        dashboard_error.short_description()
                    ));
                    return self.fallback(file_path, dashboard_error);
                }

                // Otherwise (5xx, or a 2xx response whose body still signals an error,
                // e.g. Open-Meteo's "The service is overloaded") treat it as transient
                // and let `try_fetch_with_retry` retry it with backoff, the same way it
                // retries network failures, instead of giving up after a single attempt.
                return Err(TransientApiError(dashboard_error).into());
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
        Self::log_attempt_warning(&dashboard_error, attempt, max_retries);
    }

    /// Log a short warning for a failed fetch attempt, framed as either the initial
    /// attempt or a numbered retry. Shared by the network-error path
    /// (`handle_fetch_error`) and the body-level transient API error path in
    /// `try_fetch_with_retry`, so both log consistently.
    fn log_attempt_warning(dashboard_error: &DashboardError, attempt: usize, max_retries: usize) {
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
        let status = response.status();

        // Check for 429 Too Many Requests with Retry-After header
        if status.as_u16() == 429 {
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
        match self.process_successful_response(status, body, file_path, error_checker) {
            Ok(outcome) => Ok(outcome),
            Err(e) => match e.downcast::<TransientApiError>() {
                Ok(transient_error) => Err(Box::new(transient_error)),
                Err(e) => Err(Box::new(std::io::Error::other(e.to_string()))),
            },
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
                    } else if let Some(transient_error) = e.downcast_ref::<TransientApiError>() {
                        // Transient application-level error from the response body (e.g.
                        // Open-Meteo's "The service is overloaded"). Already classified as
                        // retryable by `process_successful_response` based on HTTP status,
                        // so just log and keep retrying with backoff, same as a network
                        // failure.
                        Self::log_attempt_warning(&transient_error.0, attempt, config.max_retries);

                        if attempt == config.max_retries - 1 {
                            last_error = Some(e);
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
            } else if let Some(transient_error) = e.downcast_ref::<TransientApiError>() {
                Ok(transient_error.0.clone())
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
    /// - Retries on: timeouts, connection failures, 5xx errors, 429 rate limits, and
    ///   transient body-level API errors (a 2xx/5xx response whose body still signals
    ///   an error, e.g. Open-Meteo's "The service is overloaded")
    /// - Does not retry a body-level API error accompanied by a 4xx status (other than
    ///   429), since that indicates a problem with the request itself that would fail
    ///   identically on every attempt
    /// - Falls back to cached data if all retries fail
    pub fn fetch_data<T>(
        &self,
        settings: &DashboardSettings,
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

        match settings.dev.disable_weather_api_requests {
            true => {
                let cached = self.load_cached(&file_path)?;
                Ok(FetchOutcome::Fresh(cached))
            }
            false => self.try_fetch_with_retry(&endpoint, &file_path, error_checker, &config),
        }
    }
}

/// Tests for error classification, Retry-After parsing, and retryability
/// decisions. `wiremock` is a dev-dependency, so tests that need a real
/// `reqwest::Error` (which has no public constructor) get one from an actual
/// failed request against a local mock server, same as the wiring tests in
/// `tests/fetcher_test.rs`.
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_timeout_mock() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/timeout"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn setup_404_mock() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/not-found"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn setup_500_mock() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/server-error"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn setup_429_mock() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/rate-limited"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn setup_400_mock() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/bad-request"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&mock_server)
            .await;
        mock_server
    }

    mod parse_retry_after {
        use super::*;

        #[test]
        fn integer_seconds() {
            assert_eq!(
                Fetcher::parse_retry_after("60"),
                Some(Duration::from_secs(60))
            );
            assert_eq!(
                Fetcher::parse_retry_after("120"),
                Some(Duration::from_secs(120))
            );
            assert_eq!(
                Fetcher::parse_retry_after("  30  "),
                Some(Duration::from_secs(30))
            ); // with whitespace
        }

        #[test]
        fn http_date() {
            // Uses a date in the future relative to test execution.
            let future_date = chrono::Utc::now() + chrono::Duration::seconds(90);
            let rfc2822 = future_date.to_rfc2822();

            let result = Fetcher::parse_retry_after(&rfc2822);
            assert!(result.is_some());
            let seconds = result.unwrap();
            // Should be around 90 seconds, allow some tolerance for test execution time
            assert!(
                (85..=95).contains(&seconds.as_secs()),
                "expected ~90 seconds, got {}",
                seconds.as_secs()
            );
        }

        #[test]
        fn past_date_returns_none() {
            let past_date = chrono::Utc::now() - chrono::Duration::seconds(60);
            let rfc2822 = past_date.to_rfc2822();
            assert_eq!(Fetcher::parse_retry_after(&rfc2822), None);
        }

        #[test]
        fn invalid_format_returns_none() {
            assert_eq!(Fetcher::parse_retry_after("invalid"), None);
            assert_eq!(Fetcher::parse_retry_after(""), None);
            assert_eq!(Fetcher::parse_retry_after("not-a-number"), None);
        }
    }

    mod classify_error {
        use super::*;

        #[tokio::test]
        async fn timeout() {
            let mock_server = setup_timeout_mock().await;
            let url = format!("{}/timeout", mock_server.uri());

            let dashboard_error = tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                let result = client
                    .get(&url)
                    .timeout(Duration::from_millis(100))
                    .send();

                assert!(result.is_err());
                let error = result.unwrap_err();
                assert!(error.is_timeout());

                Fetcher::classify_error(&error)
            })
            .await
            .unwrap();

            match dashboard_error {
                DashboardError::NetworkError { details } => {
                    assert!(details.contains("timeout") || details.contains("Timeout"));
                }
                _ => panic!("expected NetworkError for timeout, got {dashboard_error:?}"),
            }
        }

        #[test]
        fn connection_failed() {
            let client = reqwest::blocking::Client::new();
            let url = "http://localhost:59999/nonexistent"; // port unlikely to be in use

            let result = client.get(url).send();
            assert!(result.is_err());
            let error = result.unwrap_err();

            let dashboard_error = Fetcher::classify_error(&error);
            match dashboard_error {
                DashboardError::NetworkError { .. } => {}
                _ => panic!("expected NetworkError for connection failure, got {dashboard_error:?}"),
            }
        }

        #[tokio::test]
        async fn http_404() {
            let mock_server = setup_404_mock().await;
            let url = format!("{}/not-found", mock_server.uri());

            let dashboard_error = tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                let response = client.get(&url).send().unwrap();
                let error = response.error_for_status().unwrap_err();
                assert_eq!(error.status().unwrap().as_u16(), 404);
                Fetcher::classify_error(&error)
            })
            .await
            .unwrap();

            match dashboard_error {
                DashboardError::ApiError { details } => assert!(details.contains("404")),
                _ => panic!("expected ApiError for 404, got {dashboard_error:?}"),
            }
        }

        #[tokio::test]
        async fn http_500() {
            let mock_server = setup_500_mock().await;
            let url = format!("{}/server-error", mock_server.uri());

            let dashboard_error = tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                let response = client.get(&url).send().unwrap();
                let error = response.error_for_status().unwrap_err();
                assert_eq!(error.status().unwrap().as_u16(), 500);
                Fetcher::classify_error(&error)
            })
            .await
            .unwrap();

            match dashboard_error {
                DashboardError::ApiError { details } => assert!(details.contains("500")),
                _ => panic!("expected ApiError for 500, got {dashboard_error:?}"),
            }
        }

        #[tokio::test]
        async fn http_429() {
            let mock_server = setup_429_mock().await;
            let url = format!("{}/rate-limited", mock_server.uri());

            let dashboard_error = tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                let response = client.get(&url).send().unwrap();
                let error = response.error_for_status().unwrap_err();
                assert_eq!(error.status().unwrap().as_u16(), 429);
                Fetcher::classify_error(&error)
            })
            .await
            .unwrap();

            match dashboard_error {
                DashboardError::ApiError { details } => assert!(details.contains("429")),
                _ => panic!("expected ApiError for 429, got {dashboard_error:?}"),
            }
        }
    }

    mod is_error_retryable {
        use super::*;

        #[tokio::test]
        async fn timeout_is_retryable() {
            let mock_server = setup_timeout_mock().await;
            let url = format!("{}/timeout", mock_server.uri());

            tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                let result = client
                    .get(&url)
                    .timeout(Duration::from_millis(100))
                    .send();

                if let Err(error) = result {
                    assert!(Fetcher::is_error_retryable(&error));
                }
            })
            .await
            .unwrap();
        }

        #[test]
        fn connection_failure_is_retryable() {
            let client = reqwest::blocking::Client::new();
            let url = "http://localhost:59999/nonexistent";
            let result = client.get(url).send();
            if let Err(error) = result {
                assert!(Fetcher::is_error_retryable(&error));
            }
        }

        #[tokio::test]
        async fn http_500_is_retryable() {
            let mock_server = setup_500_mock().await;
            let url = format!("{}/server-error", mock_server.uri());

            tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                if let Ok(response) = client.get(&url).send() {
                    if let Err(error) = response.error_for_status() {
                        assert!(Fetcher::is_error_retryable(&error));
                    }
                }
            })
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn http_429_is_retryable() {
            let mock_server = setup_429_mock().await;
            let url = format!("{}/rate-limited", mock_server.uri());

            tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                if let Ok(response) = client.get(&url).send() {
                    if let Err(error) = response.error_for_status() {
                        assert!(Fetcher::is_error_retryable(&error));
                    }
                }
            })
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn http_404_is_not_retryable() {
            let mock_server = setup_404_mock().await;
            let url = format!("{}/not-found", mock_server.uri());

            tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                if let Ok(response) = client.get(&url).send() {
                    if let Err(error) = response.error_for_status() {
                        assert!(!Fetcher::is_error_retryable(&error));
                    }
                }
            })
            .await
            .unwrap();
        }

        #[tokio::test]
        async fn http_400_is_not_retryable() {
            let mock_server = setup_400_mock().await;
            let url = format!("{}/bad-request", mock_server.uri());

            tokio::task::spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                if let Ok(response) = client.get(&url).send() {
                    if let Err(error) = response.error_for_status() {
                        assert!(!Fetcher::is_error_retryable(&error));
                    }
                }
            })
            .await
            .unwrap();
        }
    }
}
