use std::env;
use std::io::{ErrorKind, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, path::Path};

use crate::utils::has_write_permission;
use crate::CONFIG;
use anyhow::{Context, Error, Result};
use chrono::{DateTime, Duration, Utc};
use semver::Version;
use serde::Deserialize;
use tempfile::NamedTempFile;
use zip::ZipArchive;

const LAST_CHECKED_FILE_NAME: &str = "last_checked";

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
    let package_name = env!("CARGO_PKG_NAME");
    println!("Current version: {}", current_version);

    let client = reqwest::blocking::Client::new();
    let header_value = format!("{}/{}", package_name, current_version);
    let release_info = fetch_release_info(&client, &header_value)?;
    let latest_version = parse_latest_version(&release_info)?;

    if latest_version > current_version {
        println!("Newer version available: {}", latest_version);
        download_and_extract_release(&client, &header_value, &latest_version, package_name)?;
    } else {
        println!("You're already using the latest version.");
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

/// Renames the current executable by appending the `.old` suffix.
/// This is done before updating the application to the latest version.
///
/// # Errors
///
/// Returns an error if the current executable path cannot be determined,
/// if the new executable path cannot be determined, or if the rename operation fails.
fn rename_current_executable() -> Result<(), std::io::Error> {
    let exe = env::current_exe()?;
    let mut new_exe = exe.clone();
    new_exe.set_file_name(format!(
        "{}.old",
        exe.file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or("pi-inky-weather-epd")
    ));
    fs::rename(&exe, &new_exe)?;
    Ok(())
}

/// Downloads and extracts the latest release from the GitHub repository.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `header_value` - The value to use for the User-Agent header.
/// * `latest_version` - The latest version to download.
///
/// # Errors
///
/// Returns an error if the download fails, if the ZIP archive cannot be read,
/// if the binary base directory cannot be created, or if the archive cannot be extracted.
fn download_and_extract_release(
    client: &reqwest::blocking::Client,
    header_value: &str,
    latest_version: &semver::Version,
    package_name: &str,
) -> Result<(), anyhow::Error> {
    let download_url = format!(
        "{}/v{}/{}.zip",
        CONFIG.release.download_base_url.as_str(),
        latest_version,
        TARGET_ARTIFACT
    );

    let mut response = client
        .get(&download_url)
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

    // Reset the file cursor so we can read from the start.
    temp_zip
        .as_file_mut()
        .seek(SeekFrom::Start(0))
        .context("Failed to seek to start of the temporary ZIP file")?;

    let binary_base_dir = get_base_dir_path()?;
    if has_write_permission(binary_base_dir.clone())
        .context("Failed to check write permissions for binary base directory")?
    {
        // Open a ZipArchive on the temporary file.
        let mut archive =
            ZipArchive::new(temp_zip.as_file()).context("Could not read downloaded ZIP archive")?;

        // Rename the current executable to *.old before extracting.
        rename_current_executable()
            .context("Failed to rename current executable before extracting")?;

        // Extract the downloaded archive into the binary base directory.
        archive
            .extract(&binary_base_dir)
            .context("Could not decompress downloaded ZIP archive")?;

        // Set executable permissions on the binary
        let binary_path = &binary_base_dir.join(package_name);
        let mut perms = fs::metadata(binary_path)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(binary_path, perms).context("Failed to set executable permissions")?;

        println!(
            "Successfully updated application to version {}",
            latest_version
        );
    }

    Ok(())
}

/// Gets the base directory path of the current executable.
///
/// # Errors
///
/// Returns an error if the executable path cannot be determined.
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
    println!("Checking for updates...");
    // create a file to store the last time we checked for an update
    let last_checked_path = get_base_dir_path()?.join(LAST_CHECKED_FILE_NAME);
    if !Path::new(&last_checked_path).exists() {
        // File doesn't exist; create it with the current timestamp
        let now_str = Utc::now().to_rfc3339();
        fs::write(&last_checked_path, now_str)?;
        fetch_latest_release()?;
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
        if elapsed > Duration::days(CONFIG.release.update_interval_days) {
            println!(
                "It's been more than {} days ({:.1} days elapsed). .",
                CONFIG.release.update_interval_days,
                elapsed.num_days()
            );
            fetch_latest_release()?;

            fs::write(&last_checked_path, now_utc.to_rfc3339())?;
        } else {
            println!(
                "{:.1} days have passed since last check.",
                elapsed.num_days()
            );
        }
    }
    Ok(())
}
