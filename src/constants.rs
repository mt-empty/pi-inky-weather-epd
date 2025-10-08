use crate::{configs::settings::TemperatureUnit, utils::encode, CONFIG};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use url::Url;

pub const BOM_API_TEMP_UNIT: TemperatureUnit = TemperatureUnit::C;
pub const DEFAULT_AXIS_LABEL_FONT_SIZE: u16 = 19;

const BASE_WEATHER_URL: &str = "https://api.weather.bom.gov.au/v1/locations";
const NOT_AVAILABLE_ICON_NAME: &str = "not-available.svg";

fn build_forecast_url(frequency: &str) -> Url {
    let mut u = Url::parse(BASE_WEATHER_URL).expect("Failed to construct forecast endpoint URL");

    let geohash = encode(CONFIG.api.longitude.into_inner(), CONFIG.api.latitude.into_inner(), 6)
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
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=sunrise,sunset,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,uv_index,wind_speed_10m,wind_gusts_10m,relative_humidity_2m&current=is_day",
        CONFIG.api.latitude,
        CONFIG.api.longitude
    );
    Url::parse(&url).expect("Failed to construct Open Meteo endpoint URL")
});

pub static NOT_AVAILABLE_ICON_PATH: Lazy<PathBuf> = Lazy::new(|| {
    CONFIG
        .misc
        .weather_data_cache_path
        .join(NOT_AVAILABLE_ICON_NAME)
});
