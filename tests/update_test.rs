mod helpers;

use anyhow::Error;
use chrono::{Duration, TimeZone, Utc};
use pi_inky_weather_epd::{
    update::{read_update_status_from_dir, write_update_status, UpdateService},
    FixedClock,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use url::Url;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── Status file helpers ────────────────────────────────────────────────────

#[test]
fn test_write_and_read_update_status_success() {
    let temp_dir = TempDir::new().unwrap();

    let result: Result<(), Error> = Ok(());
    write_update_status(temp_dir.path(), &result);

    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, None);

    let content = fs::read_to_string(temp_dir.path().join("update_status.txt")).unwrap();
    assert_eq!(content, "success");
}

#[test]
fn test_write_and_read_update_status_failure() {
    let temp_dir = TempDir::new().unwrap();

    let error_msg = "Network timeout after 30 seconds";
    let result: Result<(), Error> = Err(anyhow::anyhow!("{}", error_msg));
    write_update_status(temp_dir.path(), &result);

    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, Some(error_msg.to_string()));

    let content = fs::read_to_string(temp_dir.path().join("update_status.txt")).unwrap();
    assert!(content.starts_with("failed: "));
    assert!(content.contains(error_msg));
}

#[test]
fn test_read_update_status_from_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, None);
}

// ── UpdateService scheduling helpers ──────────────────────────────────────

const INTERVAL_DAYS: i64 = 7;

fn write_last_checked(dir: &TempDir, days_ago: i64, now: chrono::DateTime<Utc>) {
    let timestamp = (now - Duration::days(days_ago)).to_rfc3339();
    fs::write(dir.path().join("last_checked"), timestamp).unwrap();
}

// Build the service inside spawn_blocking to avoid constructing reqwest::blocking::Client
// in an async context, which panics. Takes owned values so they can be moved into the closure.
fn build_service(base_dir: PathBuf, server_uri: String, allow_pre_release: bool) -> UpdateService {
    let release_info_url = Url::parse(&format!("{}/releases/latest", server_uri)).unwrap();
    let download_base_url = Url::parse(&format!("{}/releases/download", server_uri)).unwrap();
    UpdateService::new_for_testing(
        base_dir,
        release_info_url,
        download_base_url,
        INTERVAL_DAYS,
        allow_pre_release,
    )
    .unwrap()
}

fn build_release_zip(binary_name: &str, contents: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut buf = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut buf);
    writer
        .start_file(binary_name, zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(contents).unwrap();
    writer.finish().unwrap();
    buf.into_inner()
}

async fn mount_current_version_mock(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": format!("v{}", env!("CARGO_PKG_VERSION"))
        })))
        .mount(server)
        .await;
}

// ── UpdateService scheduling tests ────────────────────────────────────────

/// On the very first run (no `last_checked` file), the service should check for
/// updates immediately and write the current time to `last_checked`.
#[tokio::test]
async fn test_first_run_checks_for_update_and_writes_last_checked() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;
    mount_current_version_mock(&mock_server).await;

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();
    let now = Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap();

    tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap()
    .unwrap();

    let content = fs::read_to_string(temp_dir.path().join("last_checked")).unwrap();
    assert!(
        content.contains("2025-06-01"),
        "last_checked should record the clock time, got: {content}"
    );

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "Should make exactly one HTTP request on first run"
    );
}

/// When the interval has not yet elapsed, no HTTP request should be made.
#[tokio::test]
async fn test_skips_http_when_interval_not_elapsed() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;

    let now = Utc.with_ymd_and_hms(2025, 6, 10, 12, 0, 0).unwrap();
    write_last_checked(&temp_dir, 3, now); // 3 days elapsed < 7-day threshold

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap()
    .unwrap();

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        0,
        "No HTTP requests should be made when interval has not elapsed"
    );
}

/// When the interval has elapsed, the service should check for updates via HTTP.
#[tokio::test]
async fn test_checks_when_interval_has_elapsed() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;
    mount_current_version_mock(&mock_server).await;

    let now = Utc.with_ymd_and_hms(2025, 6, 10, 12, 0, 0).unwrap();
    write_last_checked(&temp_dir, 10, now); // 10 days elapsed > 7-day threshold

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap()
    .unwrap();

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "Should make one HTTP request when interval has elapsed"
    );
}

/// If the update check fails (HTTP error), `last_checked` should keep its
/// original timestamp so the check is retried next run.
#[tokio::test]
async fn test_last_checked_not_updated_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let now = Utc.with_ymd_and_hms(2025, 6, 10, 12, 0, 0).unwrap();
    let original_timestamp = (now - Duration::days(10)).to_rfc3339();
    write_last_checked(&temp_dir, 10, now); // triggers a check

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    let result = tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap();

    assert!(result.is_err(), "Should return an error on HTTP 500");

    let content = fs::read_to_string(temp_dir.path().join("last_checked")).unwrap();
    assert_eq!(
        content.trim(),
        original_timestamp.trim(),
        "last_checked should not be updated when the check fails"
    );
}

/// When the check is skipped (interval not elapsed), any stale `.old` backup
/// left from a previous update should be deleted.
#[tokio::test]
async fn test_deletes_stale_backup_when_check_is_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;

    let now = Utc.with_ymd_and_hms(2025, 6, 10, 12, 0, 0).unwrap();
    write_last_checked(&temp_dir, 3, now); // 3 days < 7-day threshold → skip

    let backup_path = temp_dir
        .path()
        .join(format!("{}.old", env!("CARGO_PKG_NAME")));
    fs::write(&backup_path, "old binary bytes").unwrap();
    assert!(
        backup_path.exists(),
        "Precondition: backup file should exist"
    );

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap()
    .unwrap();

    assert!(
        !backup_path.exists(),
        ".old backup should be cleaned up when the update check is skipped"
    );
}

/// When a newer stable version is available, the service should download the
/// release archive, back up the old binary as `.old`, swap in the new binary
/// with executable permissions, and record a successful status.
#[tokio::test]
async fn test_downloads_and_installs_newer_stable_version() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v99.0.0"
        })))
        .mount(&mock_server)
        .await;

    let zip_bytes = build_release_zip(env!("CARGO_PKG_NAME"), b"new binary bytes");
    Mock::given(method("GET"))
        .and(path_regex(r"^/releases/download/v99\.0\.0/.+\.zip$"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes))
        .mount(&mock_server)
        .await;

    let bin_path = temp_dir.path().join(env!("CARGO_PKG_NAME"));
    fs::write(&bin_path, "old binary bytes").unwrap();

    let now = Utc.with_ymd_and_hms(2025, 6, 10, 12, 0, 0).unwrap();
    write_last_checked(&temp_dir, 10, now); // triggers an interval-elapsed check

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false);
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap()
    .unwrap();

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        2,
        "Should make a version-check request and a download request"
    );

    assert_eq!(
        fs::read_to_string(&bin_path).unwrap(),
        "new binary bytes",
        "New binary should replace the old one"
    );

    let mode = fs::metadata(&bin_path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o755, "New binary should be executable");

    let backup_path = temp_dir
        .path()
        .join(format!("{}.old", env!("CARGO_PKG_NAME")));
    assert_eq!(
        fs::read_to_string(&backup_path).unwrap(),
        "old binary bytes",
        "Old binary should be preserved as a .old backup"
    );

    let status = fs::read_to_string(temp_dir.path().join("update_status.txt")).unwrap();
    assert_eq!(status, "success");
}

/// When a newer pre-release version is available but `allow_pre_release` is false,
/// the service should skip the download and return Ok.
#[tokio::test]
async fn test_skips_pre_release_version_when_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v99.0.0-alpha.1"
        })))
        .mount(&mock_server)
        .await;

    let now = Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap();
    write_last_checked(&temp_dir, 10, now); // triggers an interval-elapsed check

    let base_dir = temp_dir.path().to_path_buf();
    let server_uri = mock_server.uri();

    let result = tokio::task::spawn_blocking(move || {
        let service = build_service(base_dir, server_uri, false); // allow_pre_release = false
        let clock = FixedClock::new(now);
        service.check_and_update(&clock)
    })
    .await
    .unwrap();

    assert!(
        result.is_ok(),
        "Should return Ok when skipping a pre-release: {result:?}"
    );

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "Should only make the version-check request, not a download request"
    );
}
