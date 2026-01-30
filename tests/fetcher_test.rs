//! Tests for Fetcher error classification and retry logic
//!
//! These tests verify:
//! 1. HTTP 429 rate limit detection and Retry-After header parsing
//! 2. Error classification to appropriate DashboardError variants
//! 3. Retry logic for different error types
//! 4. Idiomatic reqwest error inspection
//!
//! Uses wiremock for HTTP mocking to avoid external dependencies

use pi_inky_weather_epd::errors::DashboardError;
use pi_inky_weather_epd::providers::fetcher::Fetcher;
use std::time::Duration;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Setup mock server that returns a timeout (delayed response)
async fn setup_timeout_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    // Response that takes longer than typical timeout
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/timeout"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    mock_server
}

/// Setup mock server that returns HTTP 404
async fn setup_404_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/not-found"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    mock_server
}

/// Setup mock server that returns HTTP 500
async fn setup_500_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/server-error"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    mock_server
}

/// Setup mock server that returns HTTP 429 without Retry-After header
async fn setup_429_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/rate-limited"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    mock_server
}

/// Setup mock server that returns HTTP 400
async fn setup_400_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/bad-request"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&mock_server)
        .await;

    mock_server
}

#[test]
fn test_parse_retry_after_integer_seconds() {
    // Test parsing integer seconds
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
fn test_parse_retry_after_http_date() {
    // Test parsing RFC 2822 date format
    // Note: This test uses a date in the future relative to test execution
    let future_date = chrono::Utc::now() + chrono::Duration::seconds(90);
    let rfc2822 = future_date.to_rfc2822();

    let result = Fetcher::parse_retry_after(&rfc2822);
    assert!(result.is_some());
    let seconds = result.unwrap();
    // Should be around 90 seconds, allow some tolerance for test execution time
    assert!(
        (85..=95).contains(&seconds.as_secs()),
        "Expected ~90 seconds, got {}",
        seconds.as_secs()
    );
}

#[test]
fn test_parse_retry_after_past_date_returns_none() {
    // Past dates should return None (can't retry in the past)
    let past_date = chrono::Utc::now() - chrono::Duration::seconds(60);
    let rfc2822 = past_date.to_rfc2822();

    assert_eq!(Fetcher::parse_retry_after(&rfc2822), None);
}

#[test]
fn test_parse_retry_after_invalid_format() {
    // Invalid formats should return None
    assert_eq!(Fetcher::parse_retry_after("invalid"), None);
    assert_eq!(Fetcher::parse_retry_after(""), None);
    assert_eq!(Fetcher::parse_retry_after("not-a-number"), None);
}

#[tokio::test]
async fn test_classify_error_timeout() {
    let mock_server = setup_timeout_mock().await;
    let url = format!("{}/timeout", mock_server.uri());

    let dashboard_error = tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let result = client
            .get(&url)
            .timeout(std::time::Duration::from_millis(100))
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
        _ => panic!(
            "Expected NetworkError for timeout, got {:?}",
            dashboard_error
        ),
    }
}

#[test]
fn test_classify_error_connection_failed() {
    // Try to connect to invalid port/host
    let client = reqwest::blocking::Client::new();
    let url = "http://localhost:59999/nonexistent"; // Port unlikely to be in use

    let result = client.get(url).send();

    assert!(result.is_err());
    let error = result.unwrap_err();

    let dashboard_error = Fetcher::classify_error(&error);
    match dashboard_error {
        DashboardError::NetworkError { .. } => {
            // Expected - connection errors are NetworkError
        }
        _ => panic!(
            "Expected NetworkError for connection failure, got {:?}",
            dashboard_error
        ),
    }
}

#[tokio::test]
async fn test_classify_error_http_404() {
    let mock_server = setup_404_mock().await;
    let url = format!("{}/not-found", mock_server.uri());

    let dashboard_error = tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let result = client.get(&url).send();

        assert!(result.is_ok());
        let response = result.unwrap();
        let error_result = response.error_for_status();
        assert!(error_result.is_err());

        let error = error_result.unwrap_err();
        assert_eq!(error.status().unwrap().as_u16(), 404);

        Fetcher::classify_error(&error)
    })
    .await
    .unwrap();

    match dashboard_error {
        DashboardError::ApiError { details } => {
            assert!(details.contains("404"));
        }
        _ => panic!("Expected ApiError for 404, got {:?}", dashboard_error),
    }
}

#[tokio::test]
async fn test_classify_error_http_500() {
    let mock_server = setup_500_mock().await;
    let url = format!("{}/server-error", mock_server.uri());

    let dashboard_error = tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let result = client.get(&url).send();

        assert!(result.is_ok());
        let response = result.unwrap();
        let error_result = response.error_for_status();
        assert!(error_result.is_err());

        let error = error_result.unwrap_err();
        assert_eq!(error.status().unwrap().as_u16(), 500);

        Fetcher::classify_error(&error)
    })
    .await
    .unwrap();

    match dashboard_error {
        DashboardError::ApiError { details } => {
            assert!(details.contains("500"));
        }
        _ => panic!("Expected ApiError for 500, got {:?}", dashboard_error),
    }
}

#[tokio::test]
async fn test_classify_error_http_429() {
    let mock_server = setup_429_mock().await;
    let url = format!("{}/rate-limited", mock_server.uri());

    let dashboard_error = tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let result = client.get(&url).send();

        assert!(result.is_ok());
        let response = result.unwrap();
        let error_result = response.error_for_status();
        assert!(error_result.is_err());

        let error = error_result.unwrap_err();
        assert_eq!(error.status().unwrap().as_u16(), 429);

        Fetcher::classify_error(&error)
    })
    .await
    .unwrap();

    match dashboard_error {
        DashboardError::ApiError { details } => {
            assert!(details.contains("429"));
        }
        _ => panic!("Expected ApiError for 429, got {:?}", dashboard_error),
    }
}

#[tokio::test]
async fn test_is_error_retryable_timeout() {
    let mock_server = setup_timeout_mock().await;
    let url = format!("{}/timeout", mock_server.uri());

    tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::new();
        let result = client
            .get(&url)
            .timeout(std::time::Duration::from_millis(100))
            .send();

        if let Err(error) = result {
            assert!(Fetcher::is_error_retryable(&error));
        }
    })
    .await
    .unwrap();
}

#[test]
fn test_is_error_retryable_connection() {
    let client = reqwest::blocking::Client::new();
    let url = "http://localhost:59999/nonexistent";

    let result = client.get(url).send();

    if let Err(error) = result {
        assert!(Fetcher::is_error_retryable(&error));
    }
}

#[tokio::test]
async fn test_is_error_retryable_500() {
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
async fn test_is_error_retryable_429() {
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
async fn test_is_error_not_retryable_404() {
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
async fn test_is_error_not_retryable_400() {
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

#[test]
fn test_dashboard_error_variants_have_correct_priority() {
    use pi_inky_weather_epd::errors::DiagnosticPriority;

    // ApiError should have High priority
    let api_error = DashboardError::ApiError {
        details: "test".to_string(),
    };
    assert_eq!(api_error.priority(), DiagnosticPriority::High);

    // NetworkError should have Medium priority
    let network_error = DashboardError::NetworkError {
        details: "test".to_string(),
    };
    assert_eq!(network_error.priority(), DiagnosticPriority::Medium);

    // IncompleteData should have Low priority
    let incomplete_error = DashboardError::IncompleteData {
        details: "test".to_string(),
    };
    assert_eq!(incomplete_error.priority(), DiagnosticPriority::Low);
}

#[test]
fn test_dashboard_error_descriptions() {
    use pi_inky_weather_epd::errors::Description;

    let network_error = DashboardError::NetworkError {
        details: "Connection failed".to_string(),
    };
    assert_eq!(
        network_error.short_description(),
        "API unreachable -> Stale Data"
    );
    assert!(network_error.long_description().contains("unable to reach"));
    assert!(network_error
        .long_description()
        .contains("Connection failed"));

    let api_error = DashboardError::ApiError {
        details: "HTTP 500".to_string(),
    };
    assert_eq!(api_error.short_description(), "API error -> Stale Data");
    assert!(api_error
        .long_description()
        .contains("API returned an error"));
    assert!(api_error.long_description().contains("HTTP 500"));
}

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
