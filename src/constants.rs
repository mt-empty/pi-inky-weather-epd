use crate::configs::settings::{DashboardSettings, TemperatureUnit};
use crate::utils::encode;
use std::path::PathBuf;
use url::Url;

pub const NOT_AVAILABLE: &str = "N/A";
pub const BOM_API_TEMP_UNIT: TemperatureUnit = TemperatureUnit::C;
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: u16 = 19;

pub const HOURLY_CACHE_SUFFIX: &str = "hourly_forecast.json";
pub const DAILY_CACHE_SUFFIX: &str = "daily_forecast.json";
pub const CACHE_SUFFIX: &str = "forecast.json";

fn build_forecast_url(settings: &DashboardSettings, frequency: &str) -> Url {
    let mut u = settings.api.bom_base_url.clone();

    let geohash = encode(settings.api.longitude, settings.api.latitude, 6)
        .expect("Failed to encode latitude and longitude to geohash");

    u.path_segments_mut()
        .unwrap()
        .push(&geohash)
        .push("forecasts")
        .push(frequency);
    u
}

// These endpoints are built from the injected settings so tests can point
// providers, coordinates and base URLs at per-test values.
pub fn daily_forecast_endpoint(settings: &DashboardSettings) -> Url {
    build_forecast_url(settings, "daily")
}

pub fn hourly_forecast_endpoint(settings: &DashboardSettings) -> Url {
    build_forecast_url(settings, "hourly")
}

/// Open-Meteo endpoint for HOURLY forecasts (uses UTC timezone)
///
/// Hourly data is requested in UTC and later converted to local time during processing.
/// This ensures consistent timestamp handling across all timezones.
pub fn open_meteo_hourly_endpoint(settings: &DashboardSettings) -> Url {
    let base_url = settings.api.open_meteo_base_url.clone();

    let url = format!(
        "{}/v1/forecast?\
        latitude={}&\
        longitude={}&\
        hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,uv_index,wind_speed_10m,wind_gusts_10m,relative_humidity_2m,snowfall,cloud_cover,weather_code&\
        current=is_day&\
        forecast_days=14&\
        timezone=UTC",
        base_url.as_str().trim_end_matches('/'),
        settings.api.latitude,
        settings.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo hourly endpoint URL")
}

/// Open-Meteo endpoint for DAILY forecasts (uses auto timezone for correct aggregation)
///
/// Daily aggregations (max/min temp, precipitation totals) are computed over the location's
/// local 24-hour window (midnight-to-midnight in the coordinates' timezone), not UTC's 24-hour window.
/// This ensures "today's high" reflects the actual hottest hour in the user's local day.
///
/// Uses `past_days=1` to include yesterday's data, ensuring users in timezones behind UTC
/// still have access to "today's" forecast even after UTC midnight crosses into the next
/// calendar day.
///
/// The `timezone=auto` parameter automatically determines the timezone from the lat/lon coordinates.
pub fn open_meteo_daily_endpoint(settings: &DashboardSettings) -> Url {
    let base_url = settings.api.open_meteo_base_url.clone();

    let url = format!(
        "{}/v1/forecast?\
        latitude={}&\
        longitude={}&\
        daily=sunrise,sunset,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,snowfall_sum,cloud_cover_mean,weather_code&\
        current=is_day&\
        forecast_days=14&\
        past_days=1&\
        timezone=auto",
        base_url.as_str().trim_end_matches('/'),
        settings.api.latitude,
        settings.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo daily endpoint URL")
}

pub fn not_available_icon_path(settings: &DashboardSettings) -> PathBuf {
    settings.misc.svg_icons_directory.join("not-available.svg")
}
