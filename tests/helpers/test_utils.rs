//! Common test utilities for snapshot tests

use pi_inky_weather_epd::{configs::settings::Providers, CONFIG};

/// RAII guard for environment variables - automatically restores on drop
#[allow(dead_code)] // Used by provider-specific test files
pub struct EnvVarGuard {
    key: String,
    old_value: Option<String>,
}

impl EnvVarGuard {
    #[allow(dead_code)] // Used by provider-specific test files
    pub fn new(key: &str, value: &str) -> Self {
        let old_value = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key: key.to_string(),
            old_value,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.old_value {
            Some(val) => std::env::set_var(&self.key, val),
            None => std::env::remove_var(&self.key),
        }
    }
}

/// Check if the current provider matches the expected provider
#[allow(dead_code)] // Used by provider-specific test files
pub fn is_provider(expected: Providers) -> bool {
    matches!(CONFIG.api.provider, p if p == expected)
}

/// Test fixture paths
#[allow(dead_code)] // Used by provider-specific test files
pub mod fixtures {
    pub const OPEN_METEO: &str = "tests/fixtures/open_meteo_forecast.json";
    pub const NY_6PM: &str = "tests/fixtures/ny_6pm_before_gmt/open_meteo_forecast.json";
    pub const NY_7PM: &str = "tests/fixtures/ny_7pm_after_gmt/open_meteo_forecast.json";
}

/// Test output paths
#[allow(dead_code)] // Used by provider-specific test files
pub mod outputs {
    use std::path::{Path, PathBuf};

    pub fn open_meteo(name: &str) -> PathBuf {
        Path::new("tests/output").join(format!("snapshot_open_meteo_{}.svg", name))
    }

    pub fn bom(name: &str) -> PathBuf {
        Path::new("tests/output").join(format!("snapshot_bom_{}.svg", name))
    }
}
