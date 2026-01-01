use crate::{configs::settings::TemperatureUnit, utils::encode, CONFIG};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use url::Url;

pub const BOM_API_TEMP_UNIT: TemperatureUnit = TemperatureUnit::C;
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: u16 = 19;

pub const HOURLY_CACHE_SUFFIX: &str = "hourly_forecast.json";
pub const DAILY_CACHE_SUFFIX: &str = "daily_forecast.json";
pub const CACHE_SUFFIX: &str = "forecast.json";

const NOT_AVAILABLE_ICON_NAME: &str = "not-available.svg";

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
pub static OPEN_METEO_ENDPOINT: Lazy<Url> = Lazy::new(|| {
    // Using timezone=UTC with past_days=1 to ensure we always have enough forecast data
    // even when crossing GMT boundary. Without past_days=1, users in western timezones lose
    // access to "today's" forecast after UTC midnight (even though their local day hasn't ended).
    // past_days=1 returns yesterday+today+next 14 days, ensuring current day data is always available.

    // Allow test override via environment variable (for wiremock)
    let base_url = std::env::var("OPEN_METEO_BASE_URL")
        .unwrap_or_else(|_| "https://api.open-meteo.com".to_string());

    let url = format!(
        "{}/v1/forecast?\
        latitude={}&\
        longitude={}&\
        daily=sunrise,sunset,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,cloud_cover_mean&\
        hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,uv_index,wind_speed_10m,wind_gusts_10m,relative_humidity_2m,cloud_cover&\
        current=is_day&\
        forecast_days=14&\
        past_days=1&\
        timezone=UTC",
        base_url,
        CONFIG.api.latitude,
        CONFIG.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo endpoint URL")
});

pub static NOT_AVAILABLE_ICON_PATH: Lazy<PathBuf> = Lazy::new(|| {
    CONFIG
        .misc
        .svg_icons_directory
        .join(NOT_AVAILABLE_ICON_NAME)
});
