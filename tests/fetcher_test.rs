//! Integration tests for `Fetcher`'s retry/backoff wiring end to end against a
//! real (mocked) HTTP server.
//!
//! Error classification, Retry-After parsing, and retryability decisions are
//! pure logic and are unit-tested in `src/providers/fetcher.rs` instead — see
//! docs/test-suite-review.md.

use pi_inky_weather_epd::errors::DashboardError;
use pi_inky_weather_epd::providers::fetcher::Fetcher;
use std::time::Duration;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test struct for deserializing API responses in retry tests
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
struct TestData {
    value: String,
}

#[tokio::test]
async fn test_retry_succeeds_on_third_attempt() {
    // STEP 1: Create mock server that fails twice, succeeds once
    let mock_server = MockServer::start().await;

    // First 2 requests return HTTP 500 (server error - retryable)
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2) // This mock will handle exactly 2 requests
        .named("First two failures")
        .mount(&mock_server)
        .await;

    // Third request returns success
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": "success"})),
        )
        .named("Final success")
        .mount(&mock_server)
        .await;

    // STEP 2: Setup test environment
    let url = format!("{}/test", mock_server.uri());

    // Use tokio::task::spawn_blocking because Fetcher uses blocking reqwest client
    let result = tokio::task::spawn_blocking(move || {
        // Create temporary directory for cache
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().to_path_buf();

        // Create fetcher
        let fetcher = Fetcher::new(cache_path.clone());

        // Create cache file with fallback data (in case all retries fail)
        let cache_file = cache_path.join("test_data.json");
        std::fs::write(
            &cache_file,
            serde_json::to_string(&TestData {
                value: "cached".to_string(),
            })
            .unwrap(),
        )
        .expect("Failed to write cache file");

        // STEP 3: Create custom retry config
        // 3 retries with 1-second delays (fast for testing)
        const RETRY_DELAYS: &[Duration; 3] = &[
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        ];
        let config = pi_inky_weather_epd::providers::fetcher::RetryConfig::new(
            3,
            RETRY_DELAYS,
            Duration::from_secs(10),
        );

        // STEP 4: Call try_fetch_with_retry
        let endpoint = url::Url::parse(&url).expect("Invalid URL");
        fetcher.try_fetch_with_retry::<TestData>(
            &endpoint,
            &cache_file,
            None, // no error_checker needed for this test
            &config,
        )
    })
    .await
    .expect("Task panicked");

    // STEP 5: Verify results
    match result {
        Ok(outcome) => match outcome {
            pi_inky_weather_epd::providers::fetcher::FetchOutcome::Fresh(data) => {
                assert_eq!(
                    data.value, "success",
                    "Expected 'success', got '{}'",
                    data.value
                );
            }
            pi_inky_weather_epd::providers::fetcher::FetchOutcome::Stale { data, error } => {
                panic!(
                    "Expected Fresh data, got Stale: {:?}, error: {:?}",
                    data, error
                );
            }
        },
        Err(e) => panic!("Expected success, got error: {}", e),
    }
}

/// Minimal stand-in for `check_open_meteo_error`/`check_bom_error`: treats any response
/// body of the form `{"error": true, "reason": "..."}` as an API-level error. Used to
/// exercise the generic retry behavior in `Fetcher` without depending on a specific
/// provider's response schema.
fn test_error_checker(body: &str) -> Result<(), DashboardError> {
    #[derive(serde::Deserialize)]
    struct TestApiError {
        error: bool,
        reason: String,
    }

    match serde_json::from_str::<TestApiError>(body) {
        Ok(err) if err.error => Err(DashboardError::ApiError {
            details: err.reason,
        }),
        _ => Ok(()),
    }
}

#[tokio::test]
async fn test_body_level_transient_error_is_retried_and_recovers() {
    // A body-level error on a 2xx response (e.g. Open-Meteo's "The service is
    // overloaded", which comes back without a 429/5xx status) should be retried with
    // backoff just like a network failure, not treated as a final answer.
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "error": true,
            "reason": "The service is overloaded"
        })))
        .up_to_n_times(2)
        .named("First two overloaded responses")
        .mount(&mock_server)
        .await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": "success"})),
        )
        .named("Final success")
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    let result = tokio::task::spawn_blocking(move || {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().to_path_buf();
        let fetcher = Fetcher::new(cache_path.clone());

        let cache_file = cache_path.join("test_data.json");
        std::fs::write(
            &cache_file,
            serde_json::to_string(&TestData {
                value: "cached".to_string(),
            })
            .unwrap(),
        )
        .expect("Failed to write cache file");

        const RETRY_DELAYS: &[Duration; 3] = &[
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        ];
        let config = pi_inky_weather_epd::providers::fetcher::RetryConfig::new(
            3,
            RETRY_DELAYS,
            Duration::from_secs(10),
        );

        let endpoint = url::Url::parse(&url).expect("Invalid URL");
        fetcher.try_fetch_with_retry::<TestData>(
            &endpoint,
            &cache_file,
            Some(test_error_checker),
            &config,
        )
    })
    .await
    .expect("Task panicked");

    match result {
        Ok(pi_inky_weather_epd::providers::fetcher::FetchOutcome::Fresh(data)) => {
            assert_eq!(data.value, "success");
        }
        Ok(pi_inky_weather_epd::providers::fetcher::FetchOutcome::Stale { data, error }) => {
            panic!(
                "Expected Fresh data after recovering from a transient body-level error, \
                 got Stale: {:?}, error: {:?}",
                data, error
            );
        }
        Err(e) => panic!("Expected success, got error: {}", e),
    }

    let requests = mock_server
        .received_requests()
        .await
        .expect("request recording enabled");
    assert_eq!(
        requests.len(),
        3,
        "Expected exactly 3 requests (2 retried failures + 1 success)"
    );
}

#[tokio::test]
async fn test_body_level_client_error_is_not_retried() {
    // A body-level error accompanied by a 4xx status (e.g. Open-Meteo rejecting an
    // invalid parameter with HTTP 400) indicates a problem with the request itself.
    // Retrying it will fail identically every time, so it should fall back to cached
    // data after a single attempt instead of burning the retry budget.
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": true,
            "reason": "Invalid parameter: bad_variable"
        })))
        .named("Permanent client error")
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    let result = tokio::task::spawn_blocking(move || {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().to_path_buf();
        let fetcher = Fetcher::new(cache_path.clone());

        let cache_file = cache_path.join("test_data.json");
        std::fs::write(
            &cache_file,
            serde_json::to_string(&TestData {
                value: "cached".to_string(),
            })
            .unwrap(),
        )
        .expect("Failed to write cache file");

        const RETRY_DELAYS: &[Duration; 5] = &[
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        ];
        let config = pi_inky_weather_epd::providers::fetcher::RetryConfig::new(
            5,
            RETRY_DELAYS,
            Duration::from_secs(10),
        );

        let endpoint = url::Url::parse(&url).expect("Invalid URL");
        fetcher.try_fetch_with_retry::<TestData>(
            &endpoint,
            &cache_file,
            Some(test_error_checker),
            &config,
        )
    })
    .await
    .expect("Task panicked");

    match result {
        Ok(pi_inky_weather_epd::providers::fetcher::FetchOutcome::Stale { data, error }) => {
            assert_eq!(data.value, "cached");
            match error {
                DashboardError::ApiError { details } => {
                    assert!(details.contains("Invalid parameter"));
                }
                _ => panic!("Expected ApiError, got {:?}", error),
            }
        }
        Ok(pi_inky_weather_epd::providers::fetcher::FetchOutcome::Fresh(_)) => {
            panic!("Expected Stale fallback for a permanent client error");
        }
        Err(e) => panic!("Expected fallback to cache, got error: {}", e),
    }

    let requests = mock_server
        .received_requests()
        .await
        .expect("request recording enabled");
    assert_eq!(
        requests.len(),
        1,
        "Expected exactly 1 request - a 4xx body-level error must not be retried"
    );
}
