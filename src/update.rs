use crate::logger;
use crate::CONFIG;
use anyhow::{Context, Error, Result};
use chrono::{DateTime, Duration, Utc};
use semver::Version;
use serde::Deserialize;
use std::env;
use std::io::{ErrorKind, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, path::Path};
use tempfile::NamedTempFile;
use url::Url;
use zip::ZipArchive;

const LAST_CHECKED_FILE_NAME: &str = "last_checked";
const UPDATE_STATUS_FILE_NAME: &str = "update_status.txt";
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(target_arch = "arm")]
const TARGET_ARTIFACT: &str = "arm-unknown-linux-gnueabihf";

#[cfg(target_arch = "aarch64")]
const TARGET_ARTIFACT: &str = "aarch64-unknown-linux-gnu";

#[cfg(target_arch = "x86_64")]
const TARGET_ARTIFACT: &str = "x86_64-unknown-linux-gnu";

// Fall-back if needed
#[cfg(not(any(target_arch = "arm", target_arch = "aarch64", target_arch = "x86_64")))]
const TARGET_ARTIFACT: &str = "unknown";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

// TODO: use self_update crate once this is merged https://github.com/jaemk/self_update/pull/147

/// Fetches the latest release from the GitHub repository and updates the application if a newer version is available.
///
/// # Errors
///
/// Returns an error if the current version cannot be parsed, if the release info cannot be fetched,
/// if the latest version cannot be parsed, or if the release cannot be downloaded and extracted.
fn fetch_latest_release() -> Result<(), anyhow::Error> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    logger::debug(format!("Current version: {}", current_version));

    let client = reqwest::blocking::Client::new();
    let header_value = format!("{PACKAGE_NAME}/{current_version}");
    let release_info = fetch_release_info(&client, &header_value)?;
    let latest_version = parse_latest_version(&release_info)?;

    if latest_version > current_version {
        logger::debug(format!("Newer version available: {}", latest_version));

        // return early if CONFIG.debugging.allow_pre_release_version is false and the latest version is a pre-release
        if !latest_version.pre.is_empty() && !CONFIG.debugging.allow_pre_release_version {
            logger::debug(format!("Skipping pre-release version: {}", latest_version));
            return Ok(());
        }
        download_and_extract_release(&client, &header_value, &latest_version)?;
    } else {
        logger::debug("You're already using the latest version.");
    }

    Ok(())
}

/// Fetches the release information from the GitHub API.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `header_value` - The value to use for the User-Agent header.
///
/// # Errors
///
/// Returns an error if the request fails or if the response cannot be parsed.
fn fetch_release_info(
    client: &reqwest::blocking::Client,
    header_value: &str,
) -> Result<GithubRelease, anyhow::Error> {
    let response = client
        .get(CONFIG.release.release_info_url.as_str())
        .header(reqwest::header::USER_AGENT, header_value)
        .send()
        .context("Failed to fetch latest release info")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch latest release info: HTTP {}",
            response.status()
        ));
    }
    let release_info: GithubRelease = response
        .json()
        .context("Failed to parse latest release info")?;
    Ok(release_info)
}

/// Parses the latest version from the GitHub release information.
///
/// # Arguments
///
/// * `release_info` - The release information fetched from the GitHub API.
///
/// # Errors
///
/// Returns an error if the version string cannot be parsed.
fn parse_latest_version(release_info: &GithubRelease) -> Result<Version, anyhow::Error> {
    let latest_version = release_info
        .tag_name
        .trim_start_matches('v')
        .parse::<Version>()
        .context("Failed to parse latest version")?;
    Ok(latest_version)
}

/// Downloads and extracts the latest release from the GitHub repository.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `header_value` - The value to use for the User-Agent header.
/// * `download_url` - The URL to download the ZIP archive from.
///
/// # Errors
///
/// Returns an error if the download fails
fn download_zip_archive(
    client: &reqwest::blocking::Client,
    header_value: &str,
    download_url: Url,
) -> Result<NamedTempFile, anyhow::Error> {
    let mut response = client
        .get(download_url)
        .header(reqwest::header::USER_AGENT, header_value)
        .send()
        .context("Failed to send request for ZIP archive")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download ZIP archive: HTTP {}",
            response.status()
        ));
    }

    let mut temp_zip =
        NamedTempFile::new().context("Failed to create a temporary file for the ZIP archive")?;
    response
        .copy_to(&mut temp_zip)
        .context("Failed to write ZIP archive into temporary file")?;
    temp_zip
        .as_file_mut()
        .seek(SeekFrom::Start(0))
        .context("Failed to seek to start of the temporary ZIP file")?;

    Ok(temp_zip)
}

/// Creates backup of existing binary.
///
/// # Arguments
///
/// * `bin_path` - The path to the existing binary.
/// * `backup_link` - The path to the backup link.
///
/// # Errors
///
/// Returns an error if the backup link cannot be created or if the existing binary cannot be renamed.
fn backup_existing_binary(bin_path: &Path, backup_link: &Path) -> Result<(), anyhow::Error> {
    if bin_path.exists() {
        let _ = fs::remove_file(backup_link);
        fs::rename(bin_path, backup_link).context("Failed to rename old binary for backup")?;
    }
    Ok(())
}

/// Updates file permissions to make binary executable.
///
/// # Arguments
///
/// * `bin_path` - The path to the binary file.
///
/// # Errors
///
/// Returns an error if the file permissions cannot be set.
fn set_executable_permissions(bin_path: &Path) -> Result<(), anyhow::Error> {
    let mut perms = fs::metadata(bin_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(bin_path, perms)?;
    Ok(())
}

/// Swaps in new files from temporary directory to base directory.
///
/// # Arguments
///
/// * `temp_dir` - The path to the temporary directory.
/// * `base_dir` - The path to the base directory.
///
/// # Errors
///
/// Returns an error if the file operations fail.
fn swap_in_new_files(temp_dir: &Path, base_dir: &Path) -> Result<(), anyhow::Error> {
    for entry in fs::read_dir(temp_dir)? {
        let from = entry?.path();
        let to = base_dir.join(from.file_name().unwrap());
        if to.exists() {
            if to.is_dir() {
                fs::remove_dir_all(&to)?;
            } else {
                fs::remove_file(&to)?;
            }
        }
        fs::rename(&from, &to)?;
    }
    Ok(())
}

/// Downloads and extracts the latest release from the GitHub repository.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `header_value` - The value to use for the User-Agent header.
/// * `latest_version` - The latest version of the application.
///
/// # Errors
///
/// Returns an error if the download fails, if the extraction fails, or if the file operations fail.
fn download_and_extract_release(
    client: &reqwest::blocking::Client,
    header_value: &str,
    latest_version: &semver::Version,
) -> Result<(), anyhow::Error> {
    let download_url = {
        let mut u = CONFIG.release.download_base_url.clone();
        u.path_segments_mut()
            .unwrap()
            .push(&format!("v{latest_version}"))
            .push(&format!("{TARGET_ARTIFACT}.zip"));
        u
    };

    let temp_zip = download_zip_archive(client, header_value, download_url)?;
    let base_dir = get_base_dir_path()?;
    let temp_dir =
        tempfile::tempdir_in(&base_dir).context("Failed to create temporary directory")?;

    let bin_path = base_dir.join(PACKAGE_NAME);
    let backup_link = base_dir.join(format!("{PACKAGE_NAME}.old"));

    // Extract archive
    let mut archive = ZipArchive::new(temp_zip.as_file())?;
    archive.extract(temp_dir.path())?;

    backup_existing_binary(&bin_path, &backup_link)?;
    swap_in_new_files(temp_dir.path(), &base_dir)?;
    set_executable_permissions(&bin_path)?;

    logger::success(format!("Updated to version {}", latest_version));
    Ok(())
}

/// Gets the base directory path of the current executable.
fn get_base_dir_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let base_dir = exe_path.parent().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::NotFound,
            "Could not determine executable directory",
        )
    })?;
    Ok(base_dir.to_path_buf())
}

/// Checks for updates and updates the application if a newer version is available.
///
/// # Errors
///
/// Returns an error if the last checked timestamp cannot be read or written,
/// if the timestamp cannot be parsed, or if the update process fails.
pub fn update_app() -> Result<(), anyhow::Error> {
    // create a file to store the last time we checked for an update
    let base_dir = get_base_dir_path()?;
    let last_checked_path = base_dir.join(LAST_CHECKED_FILE_NAME);

    let update_result = if !Path::new(&last_checked_path).exists() {
        // File doesn't exist; create it with the current timestamp
        let now_str = Utc::now().to_rfc3339();
        fs::write(&last_checked_path, now_str)?;
        fetch_latest_release()
    } else {
        //  File exists; read and parse the timestamp
        let contents = fs::read_to_string(&last_checked_path)?;
        // Parse the RFC3339 timestamp and convert it to a UTC DateTime
        let last_check_utc = DateTime::parse_from_rfc3339(contents.trim())
            .map_err(Error::msg)?
            .with_timezone(&Utc);

        let now_utc = Utc::now();
        // Compare the difference
        let elapsed = now_utc.signed_duration_since(last_check_utc);
        if elapsed > Duration::days(CONFIG.release.update_interval_days.into_inner().into()) {
            logger::info(format!(
                "It's been more than {} days ({:.1} days elapsed), Checking for latest version...",
                CONFIG.release.update_interval_days,
                elapsed.num_days()
            ));
            let result = fetch_latest_release();

            if result.is_ok() {
                fs::write(&last_checked_path, now_utc.to_rfc3339())?;
            }

            result
        } else {
            logger::info(format!(
                "Update check skipped: {:.1} days since last check (threshold: {} days)",
                elapsed.num_days(),
                CONFIG.release.update_interval_days
            ));
            logger::debug(format!(
                "Last checked: {}, Next check after: {}",
                last_check_utc.format("%Y-%m-%d %H:%M UTC"),
                (last_check_utc
                    + Duration::days(CONFIG.release.update_interval_days.into_inner().into()))
                .format("%Y-%m-%d %H:%M UTC")
            ));
            // We delete the backup link here because we couldn't delete it in the update function
            // This is a workaround for the fact that we can't delete the file while it's in use.
            let backup_link = base_dir.join(format!("{PACKAGE_NAME}.old"));
            if backup_link.exists() {
                logger::debug(format!(
                    "Deleting old backup link: {}",
                    backup_link.display()
                ));
                fs::remove_file(&backup_link)?
            }
            Ok(())
        }
    };

    // Write the update status for the dashboard to read
    write_update_status(&base_dir, &update_result);

    update_result
}

/// Writes the update status to a file for later retrieval
///
/// This allows the dashboard to display update errors without blocking on the update process.
/// The status file contains either "success" or "failed: <error message>".
///
/// # Arguments
/// * `base_dir` - The directory where the status file will be written
/// * `result` - The result of the update operation
pub fn write_update_status(base_dir: &Path, result: &Result<(), Error>) {
    let status_path = base_dir.join(UPDATE_STATUS_FILE_NAME);
    let status = match result {
        Ok(_) => "success".to_string(),
        Err(e) => format!("failed: {e}"),
    };

    if let Err(e) = fs::write(&status_path, status) {
        logger::error(format!("Failed to write update status: {}", e));
    }
}

/// Reads the last update status from the status file in the given directory
///
/// Returns Some(error_message) if the last update failed, None otherwise.
///
/// # Arguments
/// * `base_dir` - The directory where the status file is located
pub fn read_update_status_from_dir(base_dir: &Path) -> Option<String> {
    let status_path = base_dir.join(UPDATE_STATUS_FILE_NAME);
    let status = fs::read_to_string(status_path).ok()?;

    status
        .strip_prefix("failed: ")
        .map(|error_msg| error_msg.to_string())
}

/// Reads the last update status from the status file
///
/// Returns Some(error_message) if the last update failed, None otherwise.
/// This is used by the dashboard to display update failures.
pub fn read_last_update_status() -> Option<String> {
    let base_dir = get_base_dir_path().ok()?;
    read_update_status_from_dir(&base_dir)
}
