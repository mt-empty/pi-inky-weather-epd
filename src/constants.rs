use crate::{configs::settings::TemperatureUnit, utils::encode, CONFIG};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use url::Url;

pub const BOM_API_TEMP_UNIT: TemperatureUnit = TemperatureUnit::C;
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: u16 = 19;

pub const HOURLY_CACHE_SUFFIX: &str = "hourly_forecast.json";
pub const DAILY_CACHE_SUFFIX: &str = "daily_forecast.json";
pub const CACHE_SUFFIX: &str = "forecast.json";

fn build_forecast_url(frequency: &str) -> Url {
    // Allow test override via environment variable (for wiremock/fixtures)
    let base_url = std::env::var("BOM_BASE_URL")
        .unwrap_or_else(|_| "https://api.weather.bom.gov.au/v1/locations".to_string());

    let mut u = Url::parse(&base_url).expect("Failed to construct forecast endpoint URL");

    let geohash = encode(
        CONFIG.api.longitude.into_inner(),
        CONFIG.api.latitude.into_inner(),
        6,
    )
    .expect("Failed to encode latitude and longitude to geohash");

    u.path_segments_mut()
        .unwrap()
        .push(&geohash)
        .push("forecasts")
        .push(frequency);
    u
}

pub static DAILY_FORECAST_ENDPOINT: Lazy<Url> = Lazy::new(|| build_forecast_url("daily"));
pub static HOURLY_FORECAST_ENDPOINT: Lazy<Url> = Lazy::new(|| build_forecast_url("hourly"));

/// Open-Meteo endpoint for HOURLY forecasts (uses UTC timezone)
///
/// Hourly data is requested in UTC and later converted to local time during processing.
/// This ensures consistent timestamp handling across all timezones.
pub static OPEN_METEO_HOURLY_ENDPOINT: Lazy<Url> = Lazy::new(|| {
    let base_url = std::env::var("OPEN_METEO_BASE_URL")
        .unwrap_or_else(|_| "https://api.open-meteo.com".to_string());

    let url = format!(
        "{}/v1/forecast?\
        latitude={}&\
        longitude={}&\
        hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,uv_index,wind_speed_10m,wind_gusts_10m,relative_humidity_2m,snowfall,cloud_cover,weather_code&\
        current=is_day&\
        forecast_days=14&\
        timezone=UTC",
        base_url,
        CONFIG.api.latitude,
        CONFIG.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo hourly endpoint URL")
});

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
pub static OPEN_METEO_DAILY_ENDPOINT: Lazy<Url> = Lazy::new(|| {
    let base_url = std::env::var("OPEN_METEO_BASE_URL")
        .unwrap_or_else(|_| "https://api.open-meteo.com".to_string());

    let url = format!(
        "{}/v1/forecast?\
        latitude={}&\
        longitude={}&\
        daily=sunrise,sunset,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,snowfall_sum,cloud_cover_mean,weather_code&\
        current=is_day&\
        forecast_days=14&\
        past_days=1&\
        timezone=auto",
        base_url,
        CONFIG.api.latitude,
        CONFIG.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo daily endpoint URL")
});

pub static NOT_AVAILABLE_ICON_PATH: Lazy<PathBuf> =
    Lazy::new(|| CONFIG.misc.svg_icons_directory.join("not-available.svg"));
