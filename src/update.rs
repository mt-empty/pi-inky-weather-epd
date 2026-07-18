use crate::clock::Clock;
use crate::configs::settings::DashboardSettings;
use crate::logger;
use anyhow::{Context, Error, Result};
use chrono::{DateTime, Duration, Utc};
use semver::Version;
use serde::Deserialize;
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

pub struct UpdateService {
    client: reqwest::blocking::Client,
    base_dir: PathBuf,
    user_agent: String,
    release_info_url: Url,
    download_base_url: Url,
    update_interval_days: i64,
    allow_pre_release: bool,
    current_version: Version,
}

impl UpdateService {
    pub fn new(settings: &DashboardSettings) -> Result<Self> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        Ok(Self {
            client: reqwest::blocking::Client::new(),
            base_dir: base_dir_path()?,
            user_agent: format!("{PACKAGE_NAME}/{current_version}"),
            release_info_url: settings.release.release_info_url.clone(),
            download_base_url: settings.release.download_base_url.clone(),
            update_interval_days: settings.release.update_interval_days.into_inner().into(),
            allow_pre_release: settings.release.allow_pre_release_version,
            current_version,
        })
    }

    pub fn new_for_testing(
        base_dir: PathBuf,
        release_info_url: Url,
        download_base_url: Url,
        update_interval_days: i64,
        allow_pre_release: bool,
    ) -> Result<Self> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        Ok(Self {
            client: reqwest::blocking::Client::new(),
            base_dir,
            user_agent: format!("{PACKAGE_NAME}/{current_version}"),
            release_info_url,
            download_base_url,
            update_interval_days,
            allow_pre_release,
            current_version,
        })
    }

    pub fn check_and_update(&self, clock: &dyn Clock) -> Result<()> {
        let last_checked_path = self.base_dir.join(LAST_CHECKED_FILE_NAME);
        let now_utc = clock.now_utc();

        if last_checked_path.exists() {
            let contents = fs::read_to_string(&last_checked_path)?;
            let last_check_utc = DateTime::parse_from_rfc3339(contents.trim())
                .map_err(Error::msg)?
                .with_timezone(&Utc);

            let elapsed = now_utc.signed_duration_since(last_check_utc);
            if elapsed <= Duration::days(self.update_interval_days) {
                logger::info(format!(
                    "Update check skipped: {} days since last check (threshold: {} days)",
                    elapsed.num_days(),
                    self.update_interval_days
                ));
                logger::debug(format!(
                    "Last checked: {}, Next check after: {}",
                    last_check_utc.format("%Y-%m-%d %H:%M UTC"),
                    (last_check_utc + Duration::days(self.update_interval_days))
                        .format("%Y-%m-%d %H:%M UTC")
                ));
                return self.remove_stale_backup();
            }

            logger::info(format!(
                "It's been more than {} days ({} days elapsed), Checking for latest version...",
                self.update_interval_days,
                elapsed.num_days()
            ));
        }

        let result = match self.check_for_newer_version() {
            Ok(Some(latest_version)) => self.download_and_extract_release(&latest_version),
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        };

        if result.is_ok() {
            fs::write(&last_checked_path, now_utc.to_rfc3339())?;
        }
        write_update_status(&self.base_dir, &result);

        result
    }

    fn remove_stale_backup(&self) -> Result<()> {
        let backup_link = self.base_dir.join(format!("{PACKAGE_NAME}.old"));
        if backup_link.exists() {
            logger::debug(format!(
                "Deleting old backup link: {}",
                backup_link.display()
            ));
            fs::remove_file(&backup_link)?
        }
        Ok(())
    }

    fn check_for_newer_version(&self) -> Result<Option<Version>> {
        logger::debug(format!("Current version: {}", self.current_version));

        let release_info = self.fetch_release_info()?;
        let latest_version = parse_latest_version(&release_info)?;

        if latest_version <= self.current_version {
            logger::debug("You're already using the latest version.");
            return Ok(None);
        }

        logger::debug(format!("Newer version available: {}", latest_version));

        if !latest_version.pre.is_empty() && !self.allow_pre_release {
            logger::debug(format!("Skipping pre-release version: {}", latest_version));
            return Ok(None);
        }

        Ok(Some(latest_version))
    }

    fn fetch_release_info(&self) -> Result<GithubRelease> {
        let response = self
            .client
            .get(self.release_info_url.as_str())
            .header(reqwest::header::USER_AGENT, &self.user_agent)
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

    fn download_zip_archive(&self, download_url: Url) -> Result<NamedTempFile> {
        let mut response = self
            .client
            .get(download_url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .context("Failed to send request for ZIP archive")?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download ZIP archive: HTTP {}",
                response.status()
            ));
        }

        let mut temp_zip = NamedTempFile::new()
            .context("Failed to create a temporary file for the ZIP archive")?;
        response
            .copy_to(&mut temp_zip)
            .context("Failed to write ZIP archive into temporary file")?;
        temp_zip
            .as_file_mut()
            .seek(SeekFrom::Start(0))
            .context("Failed to seek to start of the temporary ZIP file")?;

        Ok(temp_zip)
    }

    fn download_and_extract_release(&self, latest_version: &semver::Version) -> Result<()> {
        let download_url = {
            let mut u = self.download_base_url.clone();
            u.path_segments_mut()
                .unwrap()
                .push(&format!("v{latest_version}"))
                .push(&format!("{TARGET_ARTIFACT}.zip"));
            u
        };

        let temp_zip = self.download_zip_archive(download_url)?;
        let temp_dir =
            tempfile::tempdir_in(&self.base_dir).context("Failed to create temporary directory")?;

        let bin_path = self.base_dir.join(PACKAGE_NAME);
        let backup_link = self.base_dir.join(format!("{PACKAGE_NAME}.old"));

        let mut archive = ZipArchive::new(temp_zip.as_file())?;
        archive.extract(temp_dir.path())?;

        backup_existing_binary(&bin_path, &backup_link)?;
        swap_in_new_files(temp_dir.path(), &self.base_dir)?;
        set_executable_permissions(&bin_path)?;

        logger::success(format!("Updated to version {}", latest_version));
        Ok(())
    }
}

fn parse_latest_version(release_info: &GithubRelease) -> Result<Version> {
    let latest_version = release_info
        .tag_name
        .trim_start_matches('v')
        .parse::<Version>()
        .context("Failed to parse latest version")?;
    Ok(latest_version)
}

fn backup_existing_binary(bin_path: &Path, backup_link: &Path) -> Result<()> {
    if bin_path.exists() {
        let _ = fs::remove_file(backup_link);
        fs::rename(bin_path, backup_link).context("Failed to rename old binary for backup")?;
    }
    Ok(())
}

fn set_executable_permissions(bin_path: &Path) -> Result<()> {
    let mut perms = fs::metadata(bin_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(bin_path, perms)?;
    Ok(())
}

fn swap_in_new_files(temp_dir: &Path, base_dir: &Path) -> Result<()> {
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

fn base_dir_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let base_dir = exe_path.parent().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::NotFound,
            "Could not determine executable directory",
        )
    })?;
    Ok(base_dir.to_path_buf())
}

pub fn update_app(settings: &DashboardSettings, clock: &dyn Clock) -> Result<()> {
    UpdateService::new(settings)?.check_and_update(clock)
}

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

pub fn read_update_status_from_dir(base_dir: &Path) -> Option<String> {
    let status_path = base_dir.join(UPDATE_STATUS_FILE_NAME);
    let status = fs::read_to_string(status_path).ok()?;

    status
        .strip_prefix("failed: ")
        .map(|error_msg| error_msg.to_string())
}

pub fn read_last_update_status() -> Option<String> {
    let base_dir = base_dir_path().ok()?;
    read_update_status_from_dir(&base_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag_name: &str) -> GithubRelease {
        GithubRelease {
            tag_name: tag_name.to_string(),
        }
    }

    #[test]
    fn v_prefixed_tag_parses() {
        let version = parse_latest_version(&release("v1.2.3")).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn tag_without_v_prefix_parses() {
        let version = parse_latest_version(&release("1.2.3")).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn pre_release_tag_parses() {
        let version = parse_latest_version(&release("v1.2.3-beta.1")).unwrap();
        assert_eq!(version.pre.as_str(), "beta.1");
    }

    #[test]
    fn malformed_tag_is_an_error() {
        assert!(parse_latest_version(&release("not-a-version")).is_err());
    }

    #[test]
    fn empty_tag_is_an_error() {
        assert!(parse_latest_version(&release("")).is_err());
    }
}
