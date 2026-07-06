//! Common test utilities for snapshot and integration tests

use pi_inky_weather_epd::configs::settings::{DashboardSettings, Providers};
use std::ops::{Deref, DerefMut};
use tempfile::TempDir;
use url::Url;

/// Settings value returned by [`test_settings`] and friends. Derefs to
/// `DashboardSettings` so existing call sites (`&settings`, `settings.field`)
/// work unchanged; the held `TempDir` is what actually matters — it deletes
/// the test's cache directory automatically when this value drops, instead
/// of leaking a new directory under `tests/output/cached_data` on every run.
#[allow(dead_code)] // Used by provider-specific test files
pub struct TestSettings {
    settings: DashboardSettings,
    _cache_dir: TempDir,
}

impl Deref for TestSettings {
    type Target = DashboardSettings;

    fn deref(&self) -> &DashboardSettings {
        &self.settings
    }
}

impl DerefMut for TestSettings {
    fn deref_mut(&mut self) -> &mut DashboardSettings {
        &mut self.settings
    }
}

/// Loads the deterministic test configuration (config/default.toml merged
/// with config/test.toml, no user config or `APP_*` env vars), gives the
/// test its own temp cache directory so parallel tests cannot clobber each
/// other's cache files, applies `mutate`, and returns the settings value.
///
/// Settings are plain values passed into the code under test — there is no
/// process-global configuration, so tests run fully in parallel.
#[allow(dead_code)] // Used by provider-specific test files
pub fn test_settings(mutate: impl FnOnce(&mut DashboardSettings)) -> TestSettings {
    let mut settings =
        DashboardSettings::load_test_config().expect("failed to load test configuration");
    let cache_dir = TempDir::new().expect("failed to create temp cache dir");
    settings.misc.weather_data_cache_path = cache_dir.path().to_path_buf();
    mutate(&mut settings);
    TestSettings {
        settings,
        _cache_dir: cache_dir,
    }
}

/// Test settings for the Open-Meteo provider, pointed at a wiremock server.
#[allow(dead_code)] // Used by provider-specific test files
pub fn open_meteo_settings(mock_base_url: &str) -> TestSettings {
    let base_url = Url::parse(mock_base_url).expect("invalid mock server URL");
    test_settings(|settings| {
        settings.api.provider = Providers::OpenMeteo;
        settings.api.open_meteo_base_url = base_url;
    })
}

/// Like [`open_meteo_settings`], but rendering in the given display timezone
/// instead of the test default (Australia/Melbourne).
#[allow(dead_code)] // Used by provider-specific test files
pub fn open_meteo_settings_in_tz(mock_base_url: &str, timezone: chrono_tz::Tz) -> TestSettings {
    let base_url = Url::parse(mock_base_url).expect("invalid mock server URL");
    test_settings(|settings| {
        settings.api.provider = Providers::OpenMeteo;
        settings.api.open_meteo_base_url = base_url;
        settings.misc.timezone = timezone;
    })
}

/// Test settings for the BOM provider, pointed at a wiremock server.
///
/// BOM URLs are constructed as `{base_url}/{geohash}/forecasts/{frequency}`,
/// so the mock base keeps the `/v1/locations` prefix the real API uses.
#[allow(dead_code)] // Used by provider-specific test files
pub fn bom_settings(mock_base_url: &str) -> TestSettings {
    let base_url =
        Url::parse(&format!("{}/v1/locations", mock_base_url)).expect("invalid mock server URL");
    test_settings(|settings| {
        settings.api.provider = Providers::Bom;
        settings.api.bom_base_url = base_url;
    })
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
