/// Comprehensive integration tests for the update system with mocked I/O
use anyhow::Error;
use pi_inky_weather_epd::update::{read_update_status_from_dir, write_update_status};
use std::fs::{self};
use tempfile::TempDir;

#[test]
fn test_write_and_read_update_status_success() {
    let temp_dir = TempDir::new().unwrap();

    // Test write_update_status with Ok result
    let result: Result<(), Error> = Ok(());
    write_update_status(temp_dir.path(), &result);

    // Test read_update_status_from_dir
    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, None); // Success returns None (no error)

    // Verify file content
    let content = fs::read_to_string(temp_dir.path().join("update_status.txt")).unwrap();
    assert_eq!(content, "success");
}

#[test]
fn test_write_and_read_update_status_failure() {
    let temp_dir = TempDir::new().unwrap();

    // Test write_update_status with Err result
    let error_msg = "Network timeout after 30 seconds";
    let result: Result<(), Error> = Err(anyhow::anyhow!("{}", error_msg));
    write_update_status(temp_dir.path(), &result);

    // Test read_update_status_from_dir
    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, Some(error_msg.to_string()));

    // Verify file content
    let content = fs::read_to_string(temp_dir.path().join("update_status.txt")).unwrap();
    assert!(content.starts_with("failed: "));
    assert!(content.contains(error_msg));
}

#[test]
fn test_read_update_status_from_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();

    // Test read_update_status_from_dir when file doesn't exist
    let status = read_update_status_from_dir(temp_dir.path());
    assert_eq!(status, None);
}
