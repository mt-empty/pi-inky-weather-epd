use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result};
use pi_inky_weather_epd::CONFIG;
use zip::ZipArchive;

const BINARY_BASE_DIR: &str = "./archives"; // TODO: Change this to a more appropriate location

fn has_write_permission(path: PathBuf) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();
    Ok(permissions.mode() & 0o200 != 0)
}
pub fn update() -> Result<()> {
    fs::create_dir_all(BINARY_BASE_DIR).context("Failed to create binary base directory")?;

    let response = reqwest::blocking::get(CONFIG.misc.download_release_url.as_str())
        .context("Failed to download ZIP archive")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download ZIP archive: HTTP {}",
            response.status()
        ));
    }
    let archive_bytes = response
        .bytes()
        .context("Could not read bytes from downloaded ZIP archive")?
        .to_vec();
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
        .context("Could not read downloaded ZIP archive")?;

    if has_write_permission(PathBuf::from(BINARY_BASE_DIR))
        .context("Failed to check write permissions for binary base directory")?
    {
        archive
            .extract(BINARY_BASE_DIR)
            .context("Could not decompress downloaded ZIP archive")?;
        println!("Successfully updated binary");
    }

    Ok(())
}
