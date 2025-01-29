use std::io::{Cursor, ErrorKind};
use std::path::PathBuf;
use std::{fs, path::Path};

use anyhow::{Context, Error, Result};
use chrono::{DateTime, Duration, Utc};
use pi_inky_weather_epd::{has_write_permission, CONFIG};
use semver::Version;
use serde::Deserialize;
use zip::ZipArchive;

const BINARY_BASE_DIR: &str = "./archives"; // TODO: Change this to a more appropriate location
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
        download_and_extract_release(&client, &header_value, &latest_version)?;
    } else {
        println!("You're already using the latest version.");
    }

    Ok(())
}

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
    // let release_info: GithubRelease = response
    //     .json()
    //     .context("Failed to parse latest release info")?;
    Ok(GithubRelease {
        tag_name: "v0.1.0".to_string(),
    })
}

fn parse_latest_version(release_info: &GithubRelease) -> Result<Version, anyhow::Error> {
    let latest_version = release_info
        .tag_name
        .trim_start_matches('v')
        .parse::<Version>()
        .context("Failed to parse latest version")?;
    Ok(latest_version)
}

fn download_and_extract_release(
    client: &reqwest::blocking::Client,
    header_value: &str,
    latest_version: &semver::Version,
) -> Result<(), anyhow::Error> {
    let download_url = format!(
        "{}/v{}/{}.zip",
        CONFIG.release.download_base_url.as_str(),
        latest_version,
        TARGET_ARTIFACT
    );
    let response = client
        .get(download_url)
        .header(reqwest::header::USER_AGENT, header_value)
        .send()
        .context("Failed to download ZIP archive")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download ZIP archive: HTTP {}",
            response.status()
        ));
    }

    let binary_base_dir = get_base_dir_path()?.join(BINARY_BASE_DIR);

    fs::create_dir_all(&binary_base_dir).context("Failed to create binary base directory")?;

    let archive_bytes = response
        .bytes()
        .context("Could not read bytes from downloaded ZIP archive")?
        .to_vec();
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
        .context("Could not read downloaded ZIP archive")?;

    if has_write_permission(PathBuf::from(&binary_base_dir))
        .context("Failed to check write permissions for binary base directory")?
    {
        archive
            .extract(&binary_base_dir)
            .context("Could not decompress downloaded ZIP archive")?;
        println!("Successfully updated application to version");
    }

    Ok(())
}

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

pub fn update() -> Result<(), anyhow::Error> {
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

            // (Optionally) update the timestamp after performing the action
            fs::write(&last_checked_path, now_utc.to_rfc3339())?;
        } else {
            println!(
                "Only {:.1} days have passed since last check.",
                elapsed.num_days()
            );
        }
    }
    Ok(())
}
